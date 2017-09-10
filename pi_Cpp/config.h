#ifndef CONFIG
#define CONFIG

#include <cstdint> //uint16_t
#include "encodingScheme.h"

constexpr uint16_t MAXPLOTRESOLUTION = 1000;
constexpr uint16_t MAX_FETCHED_ELEMENTS = 1000;

//note also change this in config of decoder class (USED FOR DECODING/ENCODING AIRPRESSURE)
constexpr uint32_t MINIMUM_MEASURABLE_PRESSURE = 93600; //Pa

enum Command {LIGHTS_ALLON, LIGHTS_ALLOFF, MS_SLEEPING, MOVIEMODE};

namespace stateConf {
	constexpr int MAXMINIMALDURATION = 3600; //seconds: 1 hour
}

namespace headers {
	constexpr uint8_t SETUP_DONE = 200;
	constexpr uint8_t STARTUP_DONE = 201;
	constexpr uint8_t FAST_UPDATE = 255;
	constexpr uint8_t SLOW_UPDATE = 26;
}

namespace wakeup {
	constexpr int total_duration = 15*60;    //seconds;
	constexpr int lampsOnly_duration = 5*60; //seconds;
}

namespace config {
	constexpr int HTTPSERVER_PORT = 8444;
	constexpr const char* HTTPSERVER_USER = "test";
	constexpr const char* HTTPSERVER_PASS = "test"; //using random strings as passw
	constexpr const char* HTTPSERVER_USERPASS_B64 = "dGVzdDp0ZXN0"; //USER:PASS 
	//encoded to base 64 for use in Authorisation header
  constexpr int POSTBUFFERSIZE = 512;
	constexpr int MAXNAMESIZE = 20;
	constexpr int MAXANSWERSIZE = 512;

	constexpr const char* HUE_USER = "ZKK0CG0rOZY3nfhQsZbIkhH0y6P92EaaR-iBlBsk";
	constexpr const char* HUE_IP = "192.168.1.11";
  constexpr const char* HUE_URL = "http://192.168.1.11/api/ZKK0CG0rOZY3nfhQsZbIkhH0y6P92EaaR-iBlBsk";
	constexpr const char* HUE_RESOURCE = "/api/ZKK0CG0rOZY3nfhQsZbIkhH0y6P92EaaR-iBlBsk";

	constexpr uint16_t ARDUINO_BAUDRATE = 9600;

	constexpr int ALERT_TEMP_ABOVE = 240; //in 0.1 Celcius, 24 deg
	constexpr int ALERT_TEMP_BELOW = 140; //in 0.1 Celcius, 14 deg
	
	constexpr int ALARM_TEMP_ABOVE = 350; //in 0.1 Celcius, 35 deg
	constexpr int ALARM_TEMP_BELOW = 50;  //in 0.1 Celcius, 5 deg
	
	constexpr int ALERT_HUMIDITY_ABOVE = 50; //in 0.1 Celcius, 24 deg
	constexpr int ALERT_HUMIDITY_BELOW = 30; //in 0.1 Celcius, 14 deg
	
	constexpr int ALARM_HUMIDITY_ABOVE = 90; //in 0.1 Celcius, 35 deg
	constexpr int ALARM_HUMIDITY_BELOW = 10;  //in 0.1 Celcius, 5 deg
	
	constexpr int ALERT_CO2PPM = 400;
	constexpr int ALARM_CO2PPM = 500;	
	
	constexpr int WCPIR_TIMEOUT = 60; //timout for bathroom lamp in seconds
	constexpr int KTCHN_TIMEOUT = 30; //timout for kitchen lamp in seconds
	
	constexpr int DT_HUMIDALARM_SHOWER = 600; //time allowed for humidity to
																						//drop in the bathroom
}


//wakeup config
constexpr int WAKEUP_DURATION_MIN = 15; 	//sec
#ifdef DEBUG
	constexpr int WAKEUP_DURATION = 10; 	//sec //FIXME //TODO
#else
	constexpr int WAKEUP_DURATION = WAKEUP_DURATION_MIN*60; 	//sec
#endif
//constexpr int UPDATEPERIOD = std::max(1, WAKEUP_DURATION/400); //TODO use when supported
constexpr int UPDATEPERIOD = 1;

constexpr int DOORLAMPON = 			(int)(WAKEUP_DURATION/5); 	//sec
constexpr int ALLLAMPSON = 			(int)(WAKEUP_DURATION/3);		//sec
constexpr int WAKEUP_MUSIC_ON = (int)(WAKEUP_DURATION/2);		//sec

constexpr int BRI_MAX = 254;
constexpr float BRI_PER_SEC = 254/WAKEUP_DURATION;
constexpr int CT_MIN = 153; 	//coldest
constexpr int CT_MAX = 500;		//warmest
constexpr float CT_PER_SEC = (CT_MAX-CT_MIN)/WAKEUP_DURATION;
constexpr int VOL_MIN = 10; //%
constexpr int VOL_MAX = 50; //%
constexpr float VOL_PER_SEC = ((float)(VOL_MAX-VOL_MIN))/
((float)(WAKEUP_DURATION - WAKEUP_MUSIC_ON));

//FIXME new
constexpr int BRI_PER_Ks = 1000*254/WAKEUP_DURATION; //Ks kiloSecond
constexpr int CT_PER_Ks = (1000*(CT_MAX-CT_MIN)) /WAKEUP_DURATION;
constexpr int VOL_PER_Ks = (1000*(VOL_MAX-VOL_MIN)) /(WAKEUP_DURATION - WAKEUP_MUSIC_ON);





namespace lght {//lightvalues
	constexpr int BED = 0;
	constexpr int KITCHEN = 1;
	constexpr int DOOR = 2;
	constexpr int LEN = 3;
}

namespace mov {//movement sensors
	constexpr int DOOR = 0;
	constexpr int KITCHEN = 1;
	constexpr int BED_l = 2;
	constexpr int BED_r = 3;
	constexpr int RADIATOR = 4;
	constexpr int MIDDLEROOM = 5;
	constexpr int BATHROOM_WC = 6;
	constexpr int BATHROOM_SHOWER = 7;
	constexpr int LEN = 8;
}

namespace temp {//temp sensors
	constexpr const char* NAMES[]{"below bed\n", "in bathroom\n", "above door\n"};
	constexpr int BED = 0;
	constexpr int BATHROOM = 1;
	constexpr int DOOR = 2;
	constexpr int LEN = 3;
}

namespace hum {//humidity sensors
	constexpr const char* NAMES[]{"below bed\n", "in bathroom\n", "above door\n"};
	constexpr int BED = 0;
	constexpr int BATHROOM = 1;
	constexpr int DOOR = 2;
	constexpr int LEN = 3;
}

namespace lmp {//lamps
	constexpr uint8_t DOOR = 0;
	constexpr uint8_t KITCHEN = 1;
	constexpr uint8_t CEILING = 2;
	constexpr uint8_t BATHROOM = 3;
	constexpr uint8_t RADIATOR = 4;
	constexpr uint8_t BUREAU = 5;
	constexpr uint8_t LEN = 6;
}

namespace plnt {//plants
	constexpr const char* NAMES[]{"plantA", "plantB", "plantC"};
	constexpr int ALERT_HUMIDITY_BELOW[]{1, 2, 3};
	constexpr int NUMB_OF_PLANT_SENSORS = 3;//TODO check if needed
	constexpr int LEN = 3;
}


enum plotables{
  MOVEMENTSENSOR0,
  MOVEMENTSENSOR1,
  MOVEMENTSENSOR2,
  MOVEMENTSENSOR3,
  MOVEMENTSENSOR4,
  MOVEMENTSENSOR5,
  MOVEMENTSENSOR6,
  MOVEMENTSENSOR7,

  TEMP_BED,
  TEMP_BATHROOM,
  TEMP_DOORHIGH,

  HUMIDITY_BED,
  HUMIDITY_BATHROOM,
  HUMIDITY_DOORHIGH,

  CO2PPM,
	PRESSURE,

  BRIGHTNESS_BED,
  BRIGHTNESS_BEYONDCURTAINS,
  BRIGHTNESS_KITCHEN,
  BRIGHTNESS_DOORHIGH
};

#endif
