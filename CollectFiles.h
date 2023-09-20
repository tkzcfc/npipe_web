#pragma once

#include <filesystem>

enum class PathType
{
    File,
    Dir,
    Both,
    MatchFile,
};

struct IgnoreAttribute
{
    std::string base;
    std::string path;
    // ÊÇ·ñµÝ¹é
    bool recursive;
    PathType type;
};

class CollectFiles
{
public:

    CollectFiles(const std::string& path);

    ~CollectFiles();

    const std::vector<std::filesystem::path>& Files();

    std::filesystem::path GetRelativePath(const std::filesystem::path& path);

    std::filesystem::path GetRootPath();

private:

    void WalkFiles(const std::filesystem::path& path);

    bool Ignore(const std::filesystem::path& path);

    void ParseGitIgnore(const std::filesystem::path& path);

    std::string FmtPath(const std::string& path);

protected:
    std::string m_rootPath;
    std::vector<IgnoreAttribute> m_ignores;
    std::vector<std::filesystem::path> m_files;
    std::string m_preferredSeparator;
};

