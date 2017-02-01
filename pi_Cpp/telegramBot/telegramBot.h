#ifndef TELEGRAMBOT
#define TELEGRAMBOT

#include <stdio.h> //TODO needed?
#include <curl/curl.h>
#include <iostream> //cout
#include <string.h> //strcmp
#include <cstring> //std::memcpy

#include "httpGetPostPut.h"

class TelegramBot : public HttpGetPostPut
{
	public:
		TelegramBot();
		void processMessage();
		
	private:
		bool authorised();
		
};


#endif // MAINSTATE