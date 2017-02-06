#ifndef MAINSTATE
#define MAINSTATE

	/* An automatic update will be triggerd whenever there is new data
	 * (from sensors). It then can send out a command. Commands can cause
	 * a change of state that can trigger an automatic update issueing
	 * a command again.
	 * 
	 * To minimise the needed computations new data is always combined 
	 * with an integer that by means of bitwise operations (1 for true 
	 * 0 for false) indicates which values have changed. This to 
	 * facilitate switching in a case statement.
	 * 
	 * Data is also not always updated, in case of a slight variation 
	 * to the origional data updating is ignored. The definition of a
	 * slight variation is left to the function providing the data.
	 * 
	 * Automatic updates consist of 2 phases, a pre scan phase that
	 * determins which states could be affected and an update function
	 * that updates the possibly affected states update function.
	 * 
	 * Example: there is a change in the brightness value for one of the
	 * lamps. If the change is large enough the function reading the value
	 * will wake up the pre scan thread of this class.
	 *
	 * Data races are prevented by the functions of this class, the class
	 * can safely be copied
	 */
	 
	 
	/* States are devided into two groups, the mutually exculusive 
	 * major states and the non exclusive minor states. A major state
	 * might be sleeping or default. In the sleeping state there should 
	 * be a diffrent response to many changing values then in the default
	 * state. The minor states can refine responses, think off showering
	 * going to the toilet etc.
	 * 
	 */

#include <ctime> //time()
#include <thread>

#include <mutex>
#include <memory> //for shared_ptr
#include <array>
#include <string.h> //strcmp
#include <iostream> //cout

#include "../config.h"

enum Command {LIGHTS_ALLON, LIGHTS_ALLOFF, MS_SLEEPING, MOVIEMODE};

enum MajorStates {AWAY, DEFAULT, ALMOSTSLEEPING, SLEEPING};

struct MinorStates {
	bool alarmDisarm;
	bool authorisedClose;
	bool listenToAudioBook;
	bool wakingUp;
  bool inBathroom;
  bool showering;
  bool inKitchenArea;
  bool movieMode;
};

	 
class MainState{
		
	public:
		//creates shared objects
		MainState();
			
		//gets data in the form of url's transformes it to commands or
		//state changes and if the state changed executes an update ran
		//in the httpd thread
		void httpSwitcher(const char* raw_url);
		
		//send commands to the right threads/functions
		void parseCommand(Command toParse);
		
		//is waken and then executes pre_scan();
		void thread_watchForUpdate();
	
	private:
		std::mutex alarmProcedureStarted;
		uint32_t currentTime;
		uint32_t lastBedMovement;
		
		//sensorValues
		std::array<int, 5> lightValues;
		bool lightValues_updated; 		
		std::array<int, 5> tempValues;
		bool tempValues_updated; 		
		std::array<int, 5> humidityValues;
		bool humidityValues_updated; 		
		std::array<int, 5> soilHumidityValues;
		bool soilHumidity_updated; 		
		std::array<uint32_t, 5> movement;
		int CO2ppm;
		
		//stateBookKeeping
		MinorStates minorState;
		MajorStates majorState;		
		std::array<bool, 6> lampOn;
		
		//4 mutually exclusive paths for checking which conditions should
		//be checked by the updating functions
		void update_away();			
		void update_sleeping();
		void update_default();
		void update_almostSleeping();
		
		//functions that should be ran when changed into this state
		void init_away();
		void init_sleeping();
		void init_default();
		void init_almostSleeping(MajorStates fromState);

		//state mutually exclusive state transition functions, they are
		//ran on every check.
		void transitions_away();
		void transitions_sleeping();
		void transitions_default();
		void transitions_almostSleeping();

		//away functions in away.cpp
		void away_intruder_alarm();
		void check_Plants();

		//default functions in default.cpp
		void def_lampcheck_Door();
		void def_lampCheck_Kitchen();
		void def_lampCheck_CeilingAndRadiator();
		void def_lampCheck_Bureau();
		void lampCheck_Bathroom(); //used in other major states too
		void environmental_alarm();

		//almostSleeping functions in almostSleeping.cpp
		void almostSleeping_lampCheck();
		
		//sleeping functions in sleeping.cpp
		void night_alarm();
		
		
		//functions that change states
		 //turn lamps on
		 
		 //turn lamps off
		 
		 //movie mode
		 
		//general support functions that need access to this class
		inline bool recent(uint32_t time, unsigned int threshold);
		inline bool anyRecent(std::array<uint32_t, 5> times, unsigned int threshold);
};
	
//general support functions
inline void sleep(int seconds);


#endif // MAINSTATE
