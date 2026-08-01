#pragma once
#include <memory>
namespace devilution {
class ClxSpriteList {};
using OptionalOwnedClxSpriteList = std::unique_ptr<ClxSpriteList>;
}
