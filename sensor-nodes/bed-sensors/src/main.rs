#![no_std]
#![no_main]

use defmt::{error, trace, unwrap};
use embassy_executor::Spawner;
use embassy_futures::select;
use embassy_futures::select::Either;
use embassy_net::{Ipv4Address, Ipv4Cidr, Stack, StackResources};
use embassy_net_wiznet::{chip::W5500, Device, Runner, State};
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::gpio::{Level, Output, Pull, Speed};
use embassy_stm32::i2c::{self, I2c};
use embassy_stm32::interrupt;
use embassy_stm32::mode::Async;
use embassy_stm32::peripherals::IWDG;
use embassy_stm32::spi::{Config as SpiConfig, Spi};
use embassy_stm32::time::Hertz;
use embassy_stm32::usart::{self, DataBits, StopBits, Uart};
use embassy_stm32::wdg::IndependentWatchdog;
use embassy_stm32::Config;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_time::{Delay, Timer};
use embedded_hal_bus::spi::ExclusiveDevice;
use futures::pin_mut;
use heapless::Vec;
use static_cell::StaticCell;

use {defmt_rtt as _, panic_probe as _};

mod channel;
mod error_cache;
mod network;
mod rng;
mod sensors;
use crate::channel::Queues;

embassy_stm32::bind_interrupts!(struct Irqs {
    I2C1_EV => embassy_stm32::i2c::EventInterruptHandler<embassy_stm32::peripherals::I2C1>;
    I2C1_ER => embassy_stm32::i2c::ErrorInterruptHandler<embassy_stm32::peripherals::I2C1>;
    USART1 => embassy_stm32::usart::InterruptHandler<embassy_stm32::peripherals::USART1>;
    USART2 => embassy_stm32::usart::InterruptHandler<embassy_stm32::peripherals::USART2>;
});

use embassy_executor::InterruptExecutor;
static EXECUTOR_HIGH: InterruptExecutor = InterruptExecutor::new();

use embassy_stm32::interrupt::InterruptExt;
#[interrupt]
unsafe fn USART6() {
    EXECUTOR_HIGH.on_interrupt()
}

#[embassy_executor::task]
async fn print_if_running_task() -> ! {
    // use embassy_time::Instant;
    // loop {
    //     let mut now = Instant::now();
    //     let mut biggest = 0;
    //     for _ in 0..1000 {
    //         Timer::after_millis(1).await;
    //         let new_now = Instant::now();
    //         let elapsed = (new_now - now).as_millis();
    //         now = new_now;
    //
    //         if elapsed > biggest {
    //             biggest = elapsed;
    //         }
    //     }
    //     defmt::info!("still running, largest delay was: {}ms", biggest);
    // }

    loop {
        Timer::after_secs(1).await;
        defmt::info!("still running");
    }
}

type EthernetSPI = ExclusiveDevice<Spi<'static, Async>, Output<'static>, Delay>;
#[embassy_executor::task]
async fn ethernet_task(
    runner: Runner<'static, W5500, EthernetSPI, ExtiInput<'static>, Output<'static>>,
) -> ! {
    runner.run().await
}

#[embassy_executor::task]
async fn net_task(stack: &'static Stack<Device<'static>>) -> ! {
    stack.run().await
}

// 84 Mhz clock stm32f401
fn config() -> Config {
    use embassy_stm32::rcc::{
        AHBPrescaler, APBPrescaler, Hse, HseMode, Pll, PllMul, PllPDiv, PllPreDiv, PllSource,
        Sysclk,
    };

    let mut config = Config::default();
    config.rcc.hse = Some(Hse {
        freq: Hertz(25_000_000),
        mode: HseMode::Oscillator,
    });
    config.rcc.pll_src = PllSource::HSE;
    config.rcc.pll = Some(Pll {
        prediv: PllPreDiv::DIV25,
        mul: PllMul::MUL336,
        divp: Some(PllPDiv::DIV4),
        divq: None,
        divr: None,
    });
    config.rcc.ahb_pre = AHBPrescaler::DIV1;
    config.rcc.apb1_pre = APBPrescaler::DIV2;
    config.rcc.apb2_pre = APBPrescaler::DIV1;
    config.rcc.sys = Sysclk::PLL1_P;
    config
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(config());
    let seed = rng::generate_seed_blocking();
    defmt::info!("random seed: {}", seed);
    // let dog = IndependentWatchdog::new(p.IWDG, 20 * 1000 * 1000);

    let mut usart_config = usart::Config::default();
    usart_config.baudrate = 9600;
    usart_config.data_bits = DataBits::DataBits8;
    usart_config.stop_bits = StopBits::STOP1;
    // usart_config.parity = Parity::ParityEven;
    let usart_mhz = unwrap!(Uart::new(
        p.USART1,
        p.PB7,
        p.PB6,
        Irqs,
        p.DMA2_CH7,
        p.DMA2_CH2,
        usart_config,
    ));

    let mut usart_config = usart::Config::default();
    usart_config.baudrate = 115200;
    usart_config.data_bits = DataBits::DataBits8;
    usart_config.stop_bits = StopBits::STOP1;
    // usart_config.parity = Parity::ParityEven;
    let usart_sps30 = unwrap!(Uart::new(
        p.USART2,
        p.PA3,
        p.PA2,
        Irqs,
        p.DMA1_CH6,
        p.DMA1_CH5,
        usart_config,
    ));

    let i2c = I2c::new(
        p.I2C1,
        p.PB8,
        p.PB9,
        Irqs,
        p.DMA1_CH7,
        p.DMA1_CH0,
        // extra slow, helps with longer cable runs
        Hertz(150_000),
        i2c::Config::default(),
    );
    let i2c: Mutex<NoopRawMutex, _> = Mutex::new(i2c);

    /*
    // let buttons = ButtonInputs {
    //     top_left: ExtiInput::new(p.PA13, p.EXTI13, Pull::Down),
    //     top_right: ExtiInput::new(p.PA14, p.EXTI14, Pull::Down),
    //     middle_inner: ExtiInput::new(p.PA9, p.EXTI9, Pull::Down),
    //     middle_center: ExtiInput::new(p.PA10, p.EXTI10, Pull::Down),
    //     middle_outer: ExtiInput::new(p.PA11, p.EXTI11, Pull::Down),
    //     lower_inner: ExtiInput::new(p.PA12, p.EXTI12, Pull::Down),
    //     lower_center: ExtiInput::new(p.PA15, p.EXTI15, Pull::Down),
    //     lower_outer: ExtiInput::new(p.PB5, p.EXTI5, Pull::Down),
    // };
     */

    let mut spi_cfg = SpiConfig::default();
    spi_cfg.frequency = Hertz(50_000_000); // up to 50m works
    let (miso, mosi, clk) = (p.PA6, p.PA7, p.PA5);
    let spi = Spi::new(p.SPI1, clk, mosi, miso, p.DMA2_CH3, p.DMA2_CH0, spi_cfg);
    let cs = Output::new(p.PA4, Level::High, Speed::VeryHigh);
    let spi = unwrap!(ExclusiveDevice::new(spi, cs, Delay));

    let w5500_int = ExtiInput::new(p.PB0, p.EXTI0, Pull::Up);
    let w5500_reset = Output::new(p.PB1, Level::High, Speed::VeryHigh);

    let mac_addr = [0x02, 234, 3, 4, 82, 231];
    static STATE: StaticCell<State<3, 2>> = StaticCell::new();
    let state = STATE.init(State::<3, 2>::new());
    let (device, runner) =
        embassy_net_wiznet::new(mac_addr, state, spi, w5500_int, w5500_reset).await;
    unwrap!(spawner.spawn(ethernet_task(runner)));

    // Init network stack
    let mut dns_servers: Vec<_, 3> = Vec::new();
    unwrap!(dns_servers.push(Ipv4Address([192, 168, 1, 1])));
    unwrap!(dns_servers.push(Ipv4Address([192, 168, 1, 1])));
    unwrap!(dns_servers.push(Ipv4Address([192, 168, 1, 1])));
    static STACK: StaticCell<Stack<Device>> = StaticCell::new();
    static RESOURCES: StaticCell<StackResources<2>> = StaticCell::new();
    let stack = &*STACK.init(Stack::new(
        device,
        embassy_net::Config::ipv4_static(embassy_net::StaticConfigV4 {
            address: Ipv4Cidr::new(Ipv4Address([192, 168, 1, 6]), 24),
            gateway: Some(Ipv4Address([192, 168, 1, 1])),
            dns_servers,
        }),
        RESOURCES.init(StackResources::<2>::new()),
        seed,
    ));

    // Launch network task
    unwrap!(spawner.spawn(net_task(stack)));

    embassy_stm32::interrupt::USART6.set_priority(embassy_stm32::interrupt::Priority::P6);
    let spawner = EXECUTOR_HIGH.start(embassy_stm32::interrupt::USART6);
    unwrap!(spawner.spawn(print_if_running_task()));

    let publish = Queues::new();
    let send_published = network::send_published(stack, &publish);
    pin_mut!(send_published);
    // let keep_dog_happy = keep_dog_happy(dog);
    // let send_and_pet_dog = join::join(&mut send_published, keep_dog_happy);
    let send_and_pet_dog = &mut send_published;

    let init_then_measure = sensors::init_then_measure(&publish, i2c, usart_mhz, usart_sps30);
    let res = select::select(send_and_pet_dog, init_then_measure).await;
    let unrecoverable_err = match res {
        Either::First(_) => defmt::unreachable!(),
        Either::Second(Ok(())) => defmt::unreachable!(),
        Either::Second(Err(err)) => err,
    };

    // at this point no other errors have occurred
    error!("unrecoverable error, resetting: {}", unrecoverable_err);
    publish.queue_error(unrecoverable_err);
    send_published.await; // if this takes too long the dog will get us
}

async fn keep_dog_happy(mut dog: IndependentWatchdog<'_, IWDG>) {
    loop {
        dog.unleash();
        Timer::after_secs(8).await;
        trace!("petting dog");
        dog.pet();
    }
}
