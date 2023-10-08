#pragma once

#include <vector>
#include <string>

class Config
{
public:

	static Config& instance();

	Config();

	std::vector<std::string> srcIgnores;
	std::vector<std::string> dstIgnores;

	// 禁用文件删除
	bool disableFileDeletion;
	// 线程池线程数量
	int threadNum;
};

