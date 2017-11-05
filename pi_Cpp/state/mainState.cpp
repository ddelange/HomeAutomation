#include "mainState.h"

bool State::updateOnHttp(){
	data->httpState->updated = false;
	bool updateState = true;

	std::string url = data->httpState->url;
	data->httpState->m.unlock();//unlock to indicate url has been read

	if(url == "/|lamps/evening"){
		if(data->stateName != MINIMAL_S){data->newState = MINIMAL_S;}
		data->setAll_ctBri(254, 320, 10);
	}
	else if(url == "/|lamps/night"){
		if(data->stateName != MINIMAL_S){data->newState = MINIMAL_S;}
		data->setAll_ctBri(220, 500, 10);
	}
	else if(url == "/|lamps/bedlight"){
		if(data->stateName != MINIMAL_S){data->newState = MINIMAL_S;}
		data->setAll_ctBri(1, 500, 10);
	}
	else if(url == "/|lamps/normal"){
		if(data->stateName != MINIMAL_S){data->newState = MINIMAL_S;}
		data->setAll_ctBri(254, 220, 10);
	}
	else if(url == "/|lamps/alloff"){
		if(data->stateName != MINIMAL_S){data->newState = MINIMAL_S;}
		std::cout<<"should be turning lamps off\n";
		data->Lamps::off();
	}
	else if(url == "/|lamps/allon"){
		if(data->stateName != MINIMAL_S){data->newState = MINIMAL_S;}
		data->Lamps::on();
	}
	else if(url == "/|lamps/toggle"){
		if(data->stateName != MINIMAL_S){data->newState = MINIMAL_S;}
		if(data->avgOn())
			data->Lamps::off();
		else
			data->Lamps::on();
	}
	else if(url == "/|lamps/flicker"){
		if(data->stateName != MINIMAL_S){data->newState = MINIMAL_S;}
		for(int i=0; i<20; i++){
			std::cout<<"test0\n";
			data->Lamps::on(lmp::BUREAU);
			std::cout<<"test1\n";
			data->Lamps::off(lmp::BUREAU);
		}
	}

//	else if(url == "/|state/away"){
//		if(stateName != AWAY){data->newState = AWAY;}
//		else{updateState=false;}
//	}
	else if(url == "/|state/default"){
		if(data->stateName != DEFAULT_S){data->newState = DEFAULT_S;}
		else{updateState=false;}
	}
//	else if(url == "/|state/goingToSleep"){
//		if(stateName != GOINGTOSLEEP_S){data->newState = GOINGTOSLEEP_S;}
//		else{updateState=false;}
//	}
//	else if(url == "/|state/sleeping"){
//		if(stateName != SLEEPING){data->newState = SLEEPING;}
//		else{updateState=false;}
//	}
	else if(url == "/|state/minimal"){
		if(data->stateName != MINIMAL_S){data->newState = MINIMAL_S;}
		else{updateState=false;}
	}
	else if(url == "/|state/wakeup"){
		if(data->stateName != WAKEUP_S){data->newState = WAKEUP_S;}
		else{updateState=false;}
	}

	//if string /|set/alarm in url
	else if(url.size()>11 && url.substr(0, 11) == "/|set/alarm"){
		std::cout<<"HI\n";
		int nMinutes = std::stoi(url.substr(11, url.size()-11));
		setAlarm(nMinutes);
		updateState=false;
	}
	else if(url == "/|minorState/windows"){
		data->computerState->windows = true; updateState=false; }
	else if(url == "/|minorState/linux"){
		data->computerState->windows = true; updateState=false; }
	else if(url == "/|minorState/off"){
		data->computerState->windows = true; updateState=false; }
	else
		updateState=false;

	std::cout<<"updateState is returning: "<<updateState<<"\n";
	return updateState;
}
////////////////////////GENERAL MEMBER FUNCT/////////////////////////////// /
void State::lampCheck_Bathroom(){
	std::atomic<std::int32_t>* movement = data->sensorState->movement;
	//std::cout<<"updating "<<movement[mov::BATHROOM_WC]<<", "<<movement[mov::BATHROOM_SHOWER]<<"\n";
	if(recent(movement[mov::BATHROOM_WC],10) or recent(movement[mov::BATHROOM_SHOWER],10)){
		//std::cout<<"recent\n";
		if(!data->isOn[lmp::BATHROOM]){
			ds("Turning bathroom lamps on\n")
			data->on(lmp::BATHROOM);
		}
	}
	else{
		//std::cout<<data->isOn[lmp::BATHROOM]<<"\n";
		if(data->isOn[lmp::BATHROOM]){
			ds("Turning bathroom lamps off\n")
			data->off(lmp::BATHROOM);
		}
	}
}

////////////////////////GENERAL FUNCT///////////////////////////////////////
inline void setAlarm(int nMinutes){
	std::string syscall = "at now +"+std::to_string(nMinutes)+
	                      " minutes <<< \"curl 192.168.1.10:8080/Scene/evening\"";
	//std::cout<<syscall<<"\n";
	system(syscall.c_str() );
}

inline void sleep(int seconds){
	std::this_thread::sleep_for(std::chrono::seconds(seconds));
}

inline bool State::recent(uint32_t time, unsigned int threshold){
	if(data->currentTime-time < threshold){return true; }
	else{return false; }
}

inline bool State::anyRecent(uint32_t times[],
unsigned int threshold){
	bool recent = false;
	for(int i=0; i<EncFastFile::N_ENCODED; i++)
		if(data->currentTime - times[i] < threshold){recent = true; }
	return recent;
}

inline std::string toTime(uint32_t seconds){
	if(seconds<3600){return std::to_string(seconds/60)+"minutes"; }
	else if(seconds<24*3600){return std::to_string(seconds/3600)+"hours"; }
	else{return std::to_string(seconds/(24*3600))+"days"; }
}
