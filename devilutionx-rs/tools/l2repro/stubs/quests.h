#pragma once
#include <cstdint>
namespace devilution {
enum quest_id : uint8_t {
	Q_BLOOD,
	Q_SCHAMB,
	Q_BLIND,
	Q_ANVIL,
	Q_WARLORD,
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
	bool IsAvailable() const;
};
struct QuestData {
	bool isSinglePlayerOnly;
};
extern Quest Quests[MAXQUESTS];
extern const QuestData QuestsData[MAXQUESTS];
}
