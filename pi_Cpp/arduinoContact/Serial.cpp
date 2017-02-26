#include "Serial.h"



Serial::Serial(const std::string& port, const unsigned int& baud_rate)
: _io(), _serial(_io,port){
  
  std::cout << "Opening serial port : " << port << "\n";
  _serial.set_option(boost::asio::serial_port_base::baud_rate(baud_rate));

  //wait till the arduino sends its done with initialising
  while (readHeader() != SETUP_DONE){ ;}
  std::cout << "Done initialising\n";
}


//Send message to Arduino
void Serial::writeString(const std::string& s) {
    boost::asio::write(_serial,boost::asio::buffer(s.c_str(),s.size()));
}

//Read from Arduino
std::string Serial::readLine() {

    bool end = false;
    std::string result;
    while (!end) {
        char c;
        boost::asio::read(_serial, boost::asio::buffer(&c,1));
        if(c == '\n')
            end = true;

        else if(c != '\r')
            result += c;
    }
    return result;
}

//Read from Arduino
unsigned char Serial::readHeader() {
  unsigned char c;
  boost::asio::read(_serial, boost::asio::buffer(&c,1));

  //std::cout << c << +c;
  return c;
}

void Serial::readMessage(unsigned char message[], unsigned char messageLen) {  
  boost::asio::read(_serial, boost::asio::buffer(message, messageLen));
}

