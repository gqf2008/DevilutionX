#pragma once
#include <cstdint>
#include <vector>
namespace devilution {
#include "engine/point.hpp"
enum quest_id : uint8_t {
	Q_BLOOD,
	Q_SCHAMB,
	Q_BLIND,
	Q_ANVIL,
	Q_WARLORD,
	Q_DIABLO,
	Q_BETRAYER,
	Q_BUTCHER,
	Q_PWATER,
	Q_LTBANNER,
	Q_GARBUD,
	Q_ZHAR,
	Q_VEIL,
	Q_SKELKING,
	MAXQUESTS = 24,
};
enum quest_state : uint8_t {
	QUEST_NOTAVAIL,
	QUEST_INIT,
	QUEST_ACTIVE,
	QUEST_DONE,
	QUEST_REWARD,
	QUEST_REWARD_FAILED,
};
struct Quest {
	quest_id _qidx;
	quest_state _qactive;
	uint8_t _qlevel;
	Point position;
	bool IsAvailable() const;
};
struct QuestData {
	bool isSinglePlayerOnly;
};
extern Quest Quests[MAXQUESTS];
extern const QuestData QuestsData[MAXQUESTS];
bool UseMultiplayerQuests();
}
