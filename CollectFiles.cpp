#include "CollectFiles.h"
#include <fstream>
#include <iostream>

CollectFiles::CollectFiles(const std::string& path)
{
    m_preferredSeparator = std::filesystem::path::preferred_separator;

    m_rootPath = FmtPath(path);

    while (!m_rootPath.empty() && (m_rootPath.back() == '/' || m_rootPath.back() == '\\'))
    {
        m_rootPath.pop_back();
    }

    IgnoreAttribute attri;
    attri.base = m_rootPath;
    attri.path = ".git";
    attri.recursive = true;
    attri.type = PathType::Dir;
    m_ignores.push_back(attri);

    WalkFiles(m_rootPath);
}

CollectFiles::~CollectFiles()
{}

const std::vector<std::filesystem::path>& CollectFiles::Files()
{
    return m_files;
}

std::filesystem::path CollectFiles::GetRelativePath(const std::filesystem::path& path)
{
    if (path.string().size() >= m_rootPath.size() && path.string().substr(0, m_rootPath.size()) == m_rootPath)
    {
        return path.string().substr(m_rootPath.size() + 1);
    }
    return "";
}

std::filesystem::path CollectFiles::GetRootPath()
{
    return m_rootPath;
}

void CollectFiles::WalkFiles(const std::filesystem::path& path)
{
    std::vector<std::filesystem::path> dirs;
    std::vector<std::filesystem::path> files;
    for (const auto& entry : std::filesystem::directory_iterator(path)) 
    {
        if (std::filesystem::is_directory(entry.path())) 
        {
            dirs.push_back(entry.path());
        }
        else
        {
            files.push_back(entry.path());
        }
    }

    for (auto& file : files)
    {
        if (file.filename().string() == ".gitignore")
        {
            ParseGitIgnore(file);
        }
    }

    for (auto& file : files)
    {
        if (!Ignore(file))
        {
            m_files.push_back(file);
        }
    }

    for (auto& dir : dirs) 
    {
        if (!Ignore(dir))
        {
            WalkFiles(dir);
        }
    }
}

bool CollectFiles::Ignore(const std::filesystem::path& path)
{
    bool isDirectory = std::filesystem::is_directory(path);
    auto pathStr = path.string();
    auto fileName = path.filename().string();

    for (auto& ignore : m_ignores)
    {
        // 有效路径
        if (!pathStr.starts_with(ignore.base))
        {
            continue;
        }

        // 不递归则判断是否在有效路径内 d:\new
        if (!ignore.recursive && path.parent_path().string() != ignore.base) 
        {
            continue;
        }

        // 文件后缀通配
        if (ignore.type == PathType::MatchFile)
        {
            if (path.filename().string() == ignore.path)
                return true;
            if (path.extension().string() == ignore.path)
                return true;
        }

        if (isDirectory)
        {
            switch (ignore.type)
            {
            case PathType::Dir:
            case PathType::Both:
            {
                if (path.filename().string() == ignore.path)
                {
                    return true;
                }
            }break;
            default:
                break;
            }
        }
        else
        {
            switch (ignore.type)
            {
            case PathType::File:
            case PathType::Both:
            {
                if (path.filename().string() == ignore.path)
                {
                    return true;
                }
            }break;
            default:
                break;
            }
        }
    }
    return false;
}

inline std::string trim(const std::string& str, const std::string& whitespace = " \t\n\r") 
{
    size_t start = str.find_first_not_of(whitespace);
    if (start == std::string::npos)
        return ""; // no content except whitespace
    size_t end = str.find_last_not_of(whitespace);
    return str.substr(start, end - start + 1);
}

void CollectFiles::ParseGitIgnore(const std::filesystem::path& path)
{
    std::ifstream file(path);
    if (!file.is_open()) 
    {
        std::cerr << "Unable to open file\n";
        return;
    }

    std::string line;
    while (std::getline(file, line)) 
    {
        line = trim(line);
        if (line.empty())
            continue;
        if (line[0] == '#')
            continue;

        for (size_t i = 0; i < line.size(); ++i)
        {
            if (line[i] == '\\')
                line[i] = '/';
        }

        IgnoreAttribute attri;
        attri.base = path.parent_path().string();
        attri.recursive = line[0] != '/';

        if (line[0] == '/')
        {
            line = line.substr(1);
        }

        line = trim(line);
        if (line.empty())
            continue;

        if (line[line.size() - 1] == '/')
        {
            attri.type = PathType::Dir;
            line = line.substr(0, line.find_last_of('/'));
        }
        else
        {
            if (line[0] == '*')
            {
                line = line.substr(1);
                attri.type = PathType::MatchFile;
            }
            else
            {
                attri.type = PathType::Both;
            }
        }

        if (line.empty())
            continue;

        auto separatorPos = line.find_last_of('/');
        if (separatorPos != std::string::npos && separatorPos > 0)
        {
            attri.base = attri.base + m_preferredSeparator + line.substr(0, separatorPos);
            line = line.substr(separatorPos + 1);
        }

        attri.base = FmtPath(attri.base);
        attri.path = line;
        m_ignores.push_back(attri);
    }

    file.close();
}

std::string CollectFiles::FmtPath(const std::string& path)
{
    std::string fmt = path;
    if(m_preferredSeparator == "\\")
    {
        for (size_t i = 0; i < fmt.size(); ++i)
        {
            if (fmt[i] == '/')
            {
                fmt[i] = '\\';
            }
        }
    }
    else
    {
        for (size_t i = 0; i < fmt.size(); ++i)
        {
            if (fmt[i] == '\\')
            {
                fmt[i] = '/';
            }
        }
    }
    return fmt;
}
