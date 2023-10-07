
#include <iostream>
#include "ThreadPool.h"
#include "CollectFiles.h"
#include <thread>
#include "args.hxx"
#include <set>
#include <fstream>
#include <chrono>
#include "spdlog/spdlog.h"
#include "spdlog/fmt/fmt.h"
#include "spdlog/fmt/ostr.h"

using nbsdx::concurrent::ThreadPool;

bool CompareFile(const std::filesystem::path& srcFile, const std::filesystem::path& dstFile)
{
    if (!std::filesystem::exists(dstFile))
    {
        return false;
    }

    std::ifstream src(srcFile, std::ifstream::ate | std::ifstream::binary);
    std::ifstream dst(dstFile, std::ifstream::ate | std::ifstream::binary);

    if (!src.is_open())
    {
        std::cerr << "File open failed: " << srcFile;
        return false;
    }
    if (!dst.is_open())
    {
        std::cerr << "File open failed: " << dstFile;
        return false;
    }

    auto srcg = src.tellg();
    auto dstg = dst.tellg();
    if (srcg != dstg)
    {
        return false;
    }
    src.seekg(0);
    dst.seekg(0);

    const int FILE_SIZE_THRESHOLD = 1024 * 8;

    if (srcg < FILE_SIZE_THRESHOLD && dstg < FILE_SIZE_THRESHOLD)
    {
        return std::equal(
            std::istreambuf_iterator<char>(src.rdbuf()),
            std::istreambuf_iterator<char>(),
            std::istreambuf_iterator<char>(dst.rdbuf())
        );
    }

    const int BUFFER_SIZE = 1024;

    std::vector<char> buffer1(BUFFER_SIZE, '\0');
    std::vector<char> buffer2(BUFFER_SIZE, '\0');

    do {
        src.read(&buffer1[0], BUFFER_SIZE);
        dst.read(&buffer2[0], BUFFER_SIZE);

        if (src.gcount() != dst.gcount())
            return false;

        if (!std::equal(buffer1.begin(), buffer1.end(), buffer2.begin()))
            return false;
    } while (src.good() || dst.good());

    return true;
}

int Sync(const std::string& src, const std::string& dst)
{
    if (!std::filesystem::exists(src) || !std::filesystem::is_directory(src))
    {
        spdlog::error("Source directory does not exist : {0}", src);
        return -1;
    }

    if (!std::filesystem::exists(dst) || !std::filesystem::is_directory(dst))
    {
        if (!std::filesystem::create_directory(dst))
        {
            spdlog::error("Failed to create directory: {0}", dst);
            return -1;
        }
    }

    CollectFiles srcFiles(src);
    CollectFiles dstFiles(dst);

    const unsigned int n = std::thread::hardware_concurrency();
    //std::cout << "Number of CPU cores: " << n << std::endl;
    ThreadPool pool(n);

    std::set<std::filesystem::path> srcRelativeFileSet;
    for (auto& file : srcFiles.Files())
    {
        srcRelativeFileSet.insert(srcFiles.GetRelativePath(file));
    }

    // 删除多余的文件
    for (auto& file : dstFiles.Files())
    {
        if (!srcRelativeFileSet.contains(dstFiles.GetRelativePath(file)))
        {
            pool.AddJob([file, &dstFiles]() {
                try
                {
                    if (!std::filesystem::remove(file))
                        spdlog::error("remove failed: {0}", file);
                    else
                        spdlog::info("remove file: {0}", dstFiles.GetRelativePath(file));
                }
                catch (const std::filesystem::filesystem_error& e)
                {
                    spdlog::error(e.what());
                }
                catch (const std::exception& e)
                {
                    spdlog::error(e.what());
                }
            });
        }
    }

    for (auto& file : srcFiles.Files())
    {
        auto dstFile = dstFiles.GetRootPath() / srcFiles.GetRelativePath(file);
        pool.AddJob([file, dstFile, &srcFiles]() {
            try
            {
                if (!CompareFile(file, dstFile))
                {
                    std::filesystem::create_directories(dstFile.parent_path());
                    std::filesystem::copy_file(file, dstFile, std::filesystem::copy_options::overwrite_existing);
                    spdlog::info("copy file: {0}", srcFiles.GetRelativePath(file));
                }
            }
            catch (const std::filesystem::filesystem_error& e)
            {
                spdlog::error(e.what());
            }
            catch (const std::exception& e)
            {
                spdlog::error(e.what());
            }
        });
    }

    pool.JoinAll();
    return 0;
}

int main(int argc, char** argv)
{
    spdlog::set_level(spdlog::level::level_enum::warn);

    args::ArgumentParser p("file sync");
    args::Group commands(p, "commands");
    args::Command sync(commands, "sync", "synchronize folder content", [&](args::Subparser& parser)
    {
        args::ValueFlag<std::string> srcDir(parser, "ptah", "The source directory", { 's', "src" });
        args::ValueFlag<std::string> dstDir(parser, "ptah", "The destination directory", { 'd', "dst" });
        args::ValueFlag<int> logLevel(parser, "level", "The log level", { "log_level" });
        parser.Parse();

        if (logLevel)
        {
            if(logLevel.Get() >= 0 || logLevel.Get() <= spdlog::level::level_enum::n_levels)
            {
                spdlog::set_level((spdlog::level::level_enum)logLevel.Get());
            }
        }

        if (srcDir && dstDir)
        {
            auto start = std::chrono::high_resolution_clock::now();
            auto code = Sync(srcDir.Get(), dstDir.Get());
            auto finish = std::chrono::high_resolution_clock::now();
            std::chrono::duration<double, std::milli> elapsed = finish - start;
            spdlog::debug("sync time: {0}ms, code:{1}", elapsed.count(), code);
        }
        else
        {
            std::cout << p << std::endl;
        }
    });

    args::Group arguments("arguments");
    args::HelpFlag h(arguments, "help", "help", { 'h', "help" });
    args::GlobalOptions globals(p, arguments);


    try
    {
        p.ParseCLI(argc, argv);
    }
    catch (args::Help)
    {
        std::cout << p << std::endl;
    }
    catch (args::Error& e)
    {
        spdlog::error("{0}", e.what());
        //spdlog::error("{0}\n{1}", e.what(), fmt::streamed(p));
        std::cout << p << std::endl;
        return 1;
    }
    return 0;
}

