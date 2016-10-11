#ifndef STOREDATA_H
#define STOREDATA_H
#include <iostream>
#include <stdio.h>
#include <signal.h>
#include <cstring> //memcopy
#include <ctime> //time
#include <climits> //int max etc
#include <cstdint> //uint16_t

#include <sys/stat.h> //mkdir
#include <sys/time.h>

/*
Pir saving format, normal packages with sometimes a timestamp package in front

NORMAL PIR PACKAGE:
total length 4 bytes, time short contains the lower part of the 4 byte unix time
  ----------------------------------------------------------------------------
  - time low 16 bit | pir confirmed ones 8 bit | pir confirmed zeros 8 bit -
  ----------------------------------------------------------------------------

TIMESTAMP PIR PACKAGE:
total length 4 bytes, used to store the full unixtime just in front of a normal 
pir package that crosses a treshold for putting in the full time again 
  --------------------------------------
  - time high 16 bit | time low 16 bit -
  --------------------------------------
time is in front so we can have 2 messages with the same low time part
 after eachother


=> test if timestamp package:
this is what would be read, so test 
  -----------------
  - n | m | x | x - data block a
  ----------------- 

  -----------------
  - c | d | a | b - data block b0
  -----------------
    0   1   2   3
  -----------------
  - a | b | x | x - data block b1
  -----------------   

 a | b0 | b1 | c | d

*/

#define HALFDAYSEC = 43200; //numb of sec in half a day
#define PIR_DT = 1000; //number of milliseconds to bin the pir data to
#define KB = 1000

//keeps track of data and cache
class StoreData
{
  public:
	  StoreData();//
	  ~StoreData();
	  
	  void write_pir(unsigned char data[4]);
	  void write_atmospheric(unsigned char data[18]);
	  void write_plants(unsigned char data[]);
	  
	  void read_pir(unsigned char& data[4]);
	  void read_atmospheric(unsigned char& data[18]);
	  void read_plants(unsigned char& data[]);
    
    FILE* sensDatFile;
    FILE* pirDatFile;
  private:
    unsigned char cache_pir[4*KB];//caches data and keeps track of the data
    unsigned char cache_atmospheric[18*KB];//caches data and keeps track of the data
    unsigned char cache_plants[KB];//caches data and keeps track of the data
};

//processes the data and converts to the format for storing
class PirData
{
  public:    
    uint32_t getClosestTimeStamp(int lineNumber);
    void process(unsigned char data[2]);

  private:
    unsigned char compress(unsigned char data);
    unsigned char prevData[2];    
    unsigned char Record[2];

    long long t_begin;

    bool TimeStampSet_first;
    bool TimeStampSet_second;    

    long long GetMilliSec();
    struct timeval tp;//TODO cant this be in the function?
    long int unix_timestamp();  

    bool isTimeStampPackage(unsigned char susp_time[4],  unsigned char susp_data[4]);

    bool isNotSame(unsigned char data[2]);

    void convertNotation(unsigned char B[2]);
    void combine(unsigned char B[2]);
    void binData(unsigned char data[2]);

    void write(unsigned char data[2]);  
    void writeTimestamp(long int timestamp);        
}

#endif // DATASTORE_H
