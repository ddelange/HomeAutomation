#include "decode.h"

uint32_t unix_timestamp() {
  time_t t = std::time(0);
  uint32_t now = static_cast<uint32_t> (t);
  return now;
}

void checkSensorData(std::shared_ptr<PirData> pirData, 
										 std::shared_ptr<SlowData> slowData, 
										 std::shared_ptr<MainState> state){
  
  uint32_t Tstamp;
	uint8_t data[SLOWDATA_SIZE]; 
  uint8_t x; 
 
  Serial arduino("/dev/ttyUSB0", config::ARDUINO_BAUDRATE);
	while (true){
    x = arduino.readHeader();
    switch (x){      
      case headers::FAST_UPDATE:
				std::cout<<"update fast\n";
				Tstamp = unix_timestamp();
				arduino.readMessage(data, FASTDATA_SIZE);			
				decodeFastData(Tstamp, data, pirData, slowData, state);           
        break;             
      case headers::SLOW_UPDATE:
				std::cout<<"update slow\n";
				Tstamp = unix_timestamp();
				arduino.readMessage(data, SLOWDATA_SIZE);				
				decodeSlowData(Tstamp, data, pirData, slowData, state);
				break;        
      default:
        std::cout << "error no code matched, header: " << +x <<"\n";     
    }
  }
}

void decodeFastData(uint32_t Tstamp, uint8_t data[SLOWDATA_SIZE],
										std::shared_ptr<PirData> pirData, 
										std::shared_ptr<SlowData> slowData, 
										std::shared_ptr<MainState> state){
	uint8_t temp;
	//process movement values
	//if the there has been movement recently the value temp will be one this indicates that
	//movement[] needs to be updated for that sensor. Instead of an if statement we use multiplication 
	//with temp, as temp is either 1 or 0.
	for (int i = 0; i<8; i++){
		temp = (data[0] & (1<<i)) & (data[2] & (1<<i));
		state->movement[i] = !temp * state->movement[i] + temp*Tstamp;
		temp = (data[1] & (1<<i)) & (data[3] & (1<<i));
		state->movement[i+8] = !temp * state->movement[i+8] + temp*Tstamp;
	}

	//process light values
	state->lightValues[lght::BED] = decode(data, Idx_fast::LIGHT_BED, Idx_fast::LEN_LIGHT);
	state->lightValues[lght::KITCHEN] = decode(data, Idx_fast::LIGHT_KITCHEN, Idx_fast::LEN_LIGHT);
	state->lightValues[lght::DOOR] = decode(data, Idx_fast::LIGHT_DOOR, Idx_fast::LEN_LIGHT);
	state->lightValues_updated = true;

	//store
	pirData->process(data, Tstamp);
	slowData->preProcess_light(state->lightValues, Tstamp);
}


void decodeSlowData(uint32_t Tstamp, uint8_t data[SLOWDATA_SIZE],
										std::shared_ptr<PirData> pirData, 
										std::shared_ptr<SlowData> slowData, 
										std::shared_ptr<MainState> state){

	//decode temp, humidity, co2 and store in state
	state->tempValues[temp::BED] = decode(data, Idx_slow::TEMP_BED, Idx_slow::LEN_TEMP);
	state->tempValues[temp::BATHROOM] = decode(data, Idx_slow::TEMP_BATHROOM, Idx_slow::LEN_TEMP);
	state->tempValues[temp::DOOR] = decode(data, Idx_slow::TEMP_DOOR, Idx_slow::LEN_TEMP);
	state->tempValues_updated = true;

	state->humidityValues[hum::BED] = decode(data, Idx_slow::HUM_BED, Idx_slow::LEN_TEMP);
	state->humidityValues[hum::BATHROOM] = decode(data, Idx_slow::HUM_BATHROOM, Idx_slow::LEN_TEMP);
	state->humidityValues[hum::DOOR] = decode(data, Idx_slow::HUM_DOOR, Idx_slow::LEN_TEMP);
	state->humidityValues_updated = true;

	state->CO2ppm = decode(data, Idx_slow::CO2, Idx_slow::LEN_CO2);
	
	//store
	slowData->process(data,Tstamp);
}
