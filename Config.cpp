#include "Config.h"
#include <thread>

Config& Config::instance()
{
	static Config ins;
	return ins;
}

Config::Config()
{
	disableFileDeletion = false;
	threadNum = std::thread::hardware_concurrency();
}
