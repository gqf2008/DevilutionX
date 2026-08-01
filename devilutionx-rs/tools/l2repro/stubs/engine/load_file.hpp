#pragma once
#include <cstdint>
#include <memory>
namespace devilution {
template <typename T>
std::unique_ptr<T[]> LoadFileInMem(const char *path);
}
