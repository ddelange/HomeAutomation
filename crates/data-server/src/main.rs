use color_eyre::eyre::Context;
use std::mem;
use std::net::SocketAddr;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufStream};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc::{self, Sender};

use color_eyre::{Result, Section};
use protocol::{Reading, SensorMessage};

use clap::Parser;
use tokio::select;
use tracing::{info, warn};

#[derive(Parser)]
#[command(name = "data server")]
#[command(version = "1.0")]
#[command(about = "Receives sensor events and spreads those to subscribed services")]
struct Cli {
    /// Optional name to operate on
    #[arg(short, long)]
    subscribe_port: u16,

    /// Sets a custom config file
    #[arg(short, long)]
    update_port: u16,
}

async fn handle_client(stream: TcpStream, tx: Sender<Event>) {
    let mut reader = BufStream::new(stream);
    let mut buf = Vec::new();
    loop {
        buf.clear();
        let n_read = match reader.read_until(0, &mut buf).await {
            Err(e) => {
                warn!("Connection failed/closed: {e}");
                return;
            }
            Ok(bytes) => bytes,
        };

        let bytes = &mut buf[0..n_read];
        if bytes.is_empty() { //eof
            warn!("end of stream");
            return;
        }

        tracing::trace!("{:?}", &bytes);
        let decoded = match protocol::Msg::<50>::decode(bytes) {
            Ok(msg) => msg,
            Err(e) => {
                warn!("decode failed: {e:?}");
                return;
            }
        };
        match decoded {
            protocol::Msg::Readings(readings) => {
                for value in readings.values {
                    tx.send(Event::NewReading(Ok(value))).await.unwrap();
                }
            }
            protocol::Msg::ErrorReport(report) => {
                tx.send(Event::NewReading(Err(report.error))).await.unwrap()
            }
        }
    }
}

enum Event {
    NewSub(TcpStream),
    NewReading(Result<Reading, protocol::Error>),
}

async fn spread_updates(mut events: mpsc::Receiver<Event>) -> Result<()> {
    let mut subscribers = Vec::new();

    while let Some(event) = events.recv().await {
        let msg = match event {
            Event::NewSub(sub) => {
                subscribers.push(sub);
                continue;
            }
            Event::NewReading(Ok(reading)) => {
                // TODO use futures-util's peekable with next_if
                // to get up to 49 extra messages for efficiency
                let mut readings: SensorMessage<50> = SensorMessage::new();
                readings
                    .values
                    .push(reading)
                    .expect("capacity should be > 0");
                protocol::Msg::Readings(readings)
            }
            Event::NewReading(Err(err)) => {
                let report = protocol::ErrorReport::new(err);
                protocol::Msg::ErrorReport(report)
            }
        };

        let bytes = msg.encode();
        let subs = mem::take(&mut subscribers);
        for mut sub in subs.into_iter() {
            if let Err(e) = sub.write_all(&bytes).await {
                warn!("Error writing to subscriber: {e}");
            } else {
                subscribers.push(sub);
            }
        }
    }

    Ok(())
}

async fn handle_data_sources(port: u16, share: &mpsc::Sender<Event>) -> Result<()> {
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = TcpListener::bind(addr)
        .await
        .wrap_err("Could not start listening for new subscribers")
        .with_note(|| format!("trying to listen on port: {port}"))?;

    loop {
        let res = listener.accept().await;
        match res {
            Ok((stream, source)) => {
                info!("new data source connected from: {source}");
                tokio::spawn(handle_client(stream, share.clone()));
            }
            Err(e) => {
                println!("new connection failed: {e}");
                continue;
            }
        };
    }
}

async fn register_subs(port: u16, tx: &mpsc::Sender<Event>) -> Result<()> {
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = TcpListener::bind(addr)
        .await
        .wrap_err("Could not start receiving updates")
        .with_note(|| format!("trying to listen on port: {port}"))?;

    loop {
        let res = listener.accept().await;
        match res {
            Ok((stream, source)) => {
                info!("new subscriber connected from: {source}");
                tx.send(Event::NewSub(stream)).await.unwrap();
            }
            Err(e) => {
                warn!("new connection failed: {e}");
                continue;
            }
        };
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    setup_tracing().unwrap();

    let Cli {
        subscribe_port,
        update_port,
    } = Cli::parse();
    assert_ne!(subscribe_port, update_port);

    info!("listening for updates on port: {update_port}");
    info!("serving subscribers on port: {subscribe_port}");

    let (tx, rx) = mpsc::channel(2000);
    select! {
        e = register_subs(subscribe_port, &tx) => e,
        e = handle_data_sources(update_port, &tx) => e,
        e = spread_updates(rx) => e,
    }
}

fn setup_tracing() -> Result<()> {
    use tracing_error::ErrorLayer;
    use tracing_subscriber::{self, layer::SubscriberExt, util::SubscriberInitExt, Layer};

    color_eyre::install().unwrap();

    let file_subscriber = tracing_subscriber::fmt::layer()
        .with_file(true)
        .with_line_number(true)
        .with_target(false)
        .with_ansi(false)
        .with_filter(tracing_subscriber::filter::EnvFilter::from_default_env());
    tracing_subscriber::registry()
        .with(file_subscriber)
        .with(ErrorLayer::default())
        .init();
    Ok(())
}
