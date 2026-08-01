//! 网络消息系统 - 移植自 Source/msg.cpp
//!
//! 实现网络游戏中的消息发送和接收功能
//!
//! **C++ 源文件**: `Source/msg.cpp` (~3491行), `Source/msg.h` (~766行)

use crate::game::types::Point;

/// 最大发送字符串长度
pub const MAX_SEND_STR_LEN: usize = 80;

/// 最大玩家数量
pub const MAX_PLRS: usize = 4;

/// 命令ID枚举
///
/// **C++ Reference**: `Source/msg.h` - `_cmd_id`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum CmdId {
    // Values 0-41 match the C++ `_cmd_id` enum (msg.h) exactly.
    Stand = 0,
    WalkXY = 1,
    AckPlrInfo = 2,
    AddStr = 3,
    AddMag = 4,
    AddDex = 5,
    AddVit = 6,
    GetItem = 7,
    AGetItem = 8,
    PutItem = 9,
    SpawnItem = 10,
    RAttackXY = 11,
    SpellXY = 12,
    OpObjXY = 13,
    DisarmXY = 14,
    AttackId = 15,
    AttackPid = 16,
    RAttackId = 17,
    RAttackPid = 18,
    SpellId = 19,
    SpellPid = 20,
    Resurrect = 21,
    OpObjT = 22,
    Knockback = 23,
    TalkXY = 24,
    NewLvl = 25,
    Warp = 26,
    CheatExperience = 27,
    ChangeSpellLevel = 28,
    Debug = 29,
    SyncData = 30,
    MonstDeath = 31,
    MonstDamage = 32,
    PlrDead = 33,
    PlrAlive = 34, // CMD_PLRALIVE (upstream msg.h)
    RequestGItem = 35,
    RequestAGItem = 36,
    GotoGetItem = 37,
    GotoAGetItem = 38,
    OpenDoor = 39,
    CloseDoor = 40,
    OperateObj = 41,
    // Values 42-80: C++ `_cmd_id` (previously misnumbered in Rust).
    BreakObj = 42,
    ChangePlrItems = 43,
    DelPlrItems = 44,
    ChangeInvItems = 45,
    DelInvItems = 46,
    ChangeBeltItems = 47,
    DelBeltItems = 48,
    PlrDamage = 49,
    PlrLevel = 50,
    DropItem = 51,
    PlayerJoinLevel = 52,
    SendPlrInfo = 53,
    SAttackXY = 54,
    ActivatePortal = 55,
    DeactivatePortal = 56,
    DLevel = 57,
    DLevelJunk = 58,
    DLevelEnd = 59,
    HealOther = 60,
    String = 61,
    FriendlyMode = 62,
    SetStr = 63,
    SetMag = 64,
    SetDex = 65,
    SetVit = 66,
    Retown = 67,
    SpellXYD = 68,
    ItemExtra = 69,
    SyncPutItem = 70,
    SyncQuest = 71,
    RequestSpawnGolem = 72,
    SetShield = 73,
    RemShield = 74,
    SetReflect = 75,
    Nakrul = 76,
    OpenHive = 77,
    OpenGrave = 78,
    SpawnMonster = 79,
    Invalid = 80,
    // Values 81+: Rust-port-only commands (no C++ `_cmd_id` counterpart;
    // NOT wire compatible with the original game).
    PlrOp = 81,
    Disconnect = 82,
    SetPlrAttr = 83,
    StartGame = 84,
    JoinGame = 85,
    LeaveGame = 86,
    Chat = 87,
    Pause = 88,
    SyncReq = 89,
    UseItem = 90,
    EquipItem = 91,
    UnequipItem = 92,
    BuyItem = 93,
    SellItem = 94,
    RepairItem = 95,
    IdentifyItem = 96,
    RechargeItem = 97,
    SetQuest = 98,
    Ping = 99,
    Pong = 100,
}

impl CmdId {
    /// 从 u8 转换
    pub fn from_u8(value: u8) -> Option<Self> {
        if value <= 100 {
            Some(unsafe { std::mem::transmute(value) })
        } else {
            None
        }
    }

    /// 转换为 u8
    pub fn to_u8(self) -> u8 {
        self as u8
    }

    /// 检查是否是移动命令
    pub fn is_movement(&self) -> bool {
        matches!(self, CmdId::WalkXY | CmdId::GotoGetItem | CmdId::GotoAGetItem)
    }

    /// 检查是否是攻击命令
    pub fn is_attack(&self) -> bool {
        matches!(
            self,
            CmdId::RAttackXY
                | CmdId::AttackId
                | CmdId::AttackPid
                | CmdId::RAttackId
                | CmdId::RAttackPid
        )
    }

    /// 检查是否是法术命令
    pub fn is_spell(&self) -> bool {
        matches!(self, CmdId::SpellXY | CmdId::SpellId | CmdId::SpellPid)
    }
}

/// 基础命令结构
///
/// **C++ Reference**: `Source/msg.h` - `TCmd`
#[derive(Debug, Clone, Copy, Default)]
#[repr(C, packed)]
pub struct TCmd {
    /// 命令ID
    pub cmd: u8,
}

impl TCmd {
    pub fn new(cmd: CmdId) -> Self {
        Self { cmd: cmd.to_u8() }
    }
}

/// 位置命令结构
///
/// **C++ Reference**: `Source/msg.h` - `TCmdLoc`
#[derive(Debug, Clone, Copy, Default)]
#[repr(C, packed)]
pub struct TCmdLoc {
    /// 命令ID
    pub cmd: u8,
    /// X坐标
    pub x: i8,
    /// Y坐标
    pub y: i8,
}

impl TCmdLoc {
    pub fn new(cmd: CmdId, x: i8, y: i8) -> Self {
        Self {
            cmd: cmd.to_u8(),
            x,
            y,
        }
    }

    pub fn position(&self) -> Point {
        Point::new(self.x as i32, self.y as i32)
    }
}

/// 带一个参数的命令
///
/// **C++ Reference**: `Source/msg.h` - `TCmdParam1`
#[derive(Debug, Clone, Copy, Default)]
#[repr(C, packed)]
pub struct TCmdParam1 {
    /// 命令ID
    pub cmd: u8,
    /// 参数1
    pub param1: i16,
}

impl TCmdParam1 {
    pub fn new(cmd: CmdId, param1: i16) -> Self {
        Self {
            cmd: cmd.to_u8(),
            param1,
        }
    }
}

/// 带两个参数的命令
///
/// **C++ Reference**: `Source/msg.h` - `TCmdParam2`
#[derive(Debug, Clone, Copy, Default)]
#[repr(C, packed)]
pub struct TCmdParam2 {
    /// 命令ID
    pub cmd: u8,
    /// 参数1
    pub param1: i16,
    /// 参数2
    pub param2: i16,
}

impl TCmdParam2 {
    pub fn new(cmd: CmdId, param1: i16, param2: i16) -> Self {
        Self {
            cmd: cmd.to_u8(),
            param1,
            param2,
        }
    }
}

/// 带位置和一个参数的命令
///
/// **C++ Reference**: `Source/msg.h` - `TCmdLocParam1`
#[derive(Debug, Clone, Copy, Default)]
#[repr(C, packed)]
pub struct TCmdLocParam1 {
    /// 命令ID
    pub cmd: u8,
    /// X坐标
    pub x: i8,
    /// Y坐标
    pub y: i8,
    /// 参数1
    pub param1: i16,
}

impl TCmdLocParam1 {
    pub fn new(cmd: CmdId, x: i8, y: i8, param1: i16) -> Self {
        Self {
            cmd: cmd.to_u8(),
            x,
            y,
            param1,
        }
    }

    pub fn position(&self) -> Point {
        Point::new(self.x as i32, self.y as i32)
    }
}

/// 带位置和四个参数的命令（用于法术）
///
/// **C++ Reference**: `Source/msg.h` - `TCmdLocParam4`
#[derive(Debug, Clone, Copy, Default)]
#[repr(C, packed)]
pub struct TCmdLocParam4 {
    /// 命令ID
    pub cmd: u8,
    /// X坐标
    pub x: i8,
    /// Y坐标
    pub y: i8,
    /// 参数1 (spell_id)
    pub param1: i16,
    /// 参数2 (spell_type)
    pub param2: i16,
    /// 参数3 (spell_level)
    pub param3: i16,
    /// 参数4 (spell_from)
    pub param4: i16,
}

impl TCmdLocParam4 {
    pub fn new(cmd: CmdId, x: i8, y: i8, spell_id: i16, spell_type: i16, spell_level: i16, spell_from: i16) -> Self {
        Self {
            cmd: cmd.to_u8(),
            x,
            y,
            param1: spell_id,
            param2: spell_type,
            param3: spell_level,
            param4: spell_from,
        }
    }

    pub fn position(&self) -> Point {
        Point::new(self.x as i32, self.y as i32)
    }
}

/// 带五个参数的命令（用于法术）
///
/// **C++ Reference**: `Source/msg.h` - `TCmdParam5`
#[derive(Debug, Clone, Copy, Default)]
#[repr(C, packed)]
pub struct TCmdParam5 {
    /// 命令ID
    pub cmd: u8,
    /// 参数1 (target_id)
    pub param1: i16,
    /// 参数2 (spell_id)
    pub param2: i16,
    /// 参数3 (spell_type)
    pub param3: i16,
    /// 参数4 (spell_level)
    pub param4: i16,
    /// 参数5 (spell_from)
    pub param5: i16,
}

/// 物品拾取命令
///
/// **C++ Reference**: `Source/msg.h` - `TCmdGItem`
#[derive(Debug, Clone, Copy, Default)]
#[repr(C, packed)]
pub struct TCmdGItem {
    /// 命令ID
    pub cmd: u8,
    /// 玩家ID
    pub pnum: u8,
    /// 物品ID
    pub item_id: u8,
    /// X坐标
    pub x: i8,
    /// Y坐标
    pub y: i8,
    /// 物品种子
    pub item_seed: u32,
    /// 物品索引
    pub item_idx: u16,
    /// 物品创建信息
    pub item_cf: u16,
}

/// 物品放置命令
///
/// **C++ Reference**: `Source/msg.h` - `TCmdPItem`
#[derive(Debug, Clone, Copy, Default)]
#[repr(C, packed)]
pub struct TCmdPItem {
    /// 命令ID
    pub cmd: u8,
    /// X坐标
    pub x: i8,
    /// Y坐标
    pub y: i8,
    /// 物品ID
    pub item_id: u8,
    /// 物品种子
    pub item_seed: u32,
    /// 物品索引
    pub item_idx: u16,
    /// 物品创建信息
    pub item_cf: u16,
    /// 物品数据 (序列化)
    pub item_data: [u8; 32],
}

/// 聊天消息命令
///
/// **C++ Reference**: `Source/msg.h` - `TCmdChat`
#[derive(Debug, Clone)]
pub struct TCmdChat {
    /// 命令ID
    pub cmd: u8,
    /// 发送者ID
    pub sender: u8,
    /// 消息内容
    pub message: String,
}

impl TCmdChat {
    pub fn new(sender: u8, message: String) -> Self {
        Self {
            cmd: CmdId::Chat.to_u8(),
            sender,
            message,
        }
    }
}

/// 同步头结构
///
/// **C++ Reference**: `Source/msg.h` - `TSyncHeader`
#[derive(Debug, Clone, Copy, Default)]
#[repr(C, packed)]
pub struct TSyncHeader {
    /// 命令ID
    pub cmd: u8,
    /// 关卡号
    pub level: u8,
    /// 怪物数量
    pub num_monsters: u16,
    /// 物品数量
    pub num_items: u16,
    /// 对象数量
    pub num_objects: u16,
}

/// 同步怪物数据
///
/// **C++ Reference**: `Source/msg.h` - `TSyncMonster`
#[derive(Debug, Clone, Copy, Default)]
#[repr(C, packed)]
pub struct TSyncMonster {
    /// 怪物ID
    pub monster_id: u16,
    /// X坐标
    pub x: i8,
    /// Y坐标
    pub y: i8,
    /// HP
    pub hp: i32,
    /// 怪物模式
    pub mode: u8,
    /// 朝向
    pub direction: u8,
}

/// 玩家信息头
///
/// **C++ Reference**: `Source/msg.h` - `TCmdPlrInfoHdr`
#[derive(Debug, Clone, Copy, Default)]
#[repr(C, packed)]
pub struct TCmdPlrInfoHdr {
    /// 命令ID
    pub cmd: u8,
    /// 玩家ID
    pub pnum: u8,
    /// 数据长度
    pub length: u16,
}

/// 消息处理结果
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MsgResult {
    /// 成功处理
    Ok,
    /// 需要更多数据
    NeedMoreData,
    /// 无效消息
    Invalid,
    /// 玩家验证失败
    ValidationFailed,
}

/// 消息缓冲区
pub struct MsgBuffer {
    /// 数据缓冲区
    data: Vec<u8>,
    /// 读取位置
    read_pos: usize,
    /// 写入位置
    write_pos: usize,
}

impl MsgBuffer {
    /// 创建新的消息缓冲区
    pub fn new(capacity: usize) -> Self {
        Self {
            data: vec![0; capacity],
            read_pos: 0,
            write_pos: 0,
        }
    }

    /// 创建默认容量的缓冲区
    pub fn default_capacity() -> Self {
        Self::new(4096)
    }

    /// 写入数据
    pub fn write(&mut self, data: &[u8]) -> bool {
        if self.write_pos + data.len() > self.data.len() {
            return false;
        }
        self.data[self.write_pos..self.write_pos + data.len()].copy_from_slice(data);
        self.write_pos += data.len();
        true
    }

    /// 读取数据
    pub fn read(&mut self, len: usize) -> Option<&[u8]> {
        if self.read_pos + len > self.write_pos {
            return None;
        }
        let data = &self.data[self.read_pos..self.read_pos + len];
        self.read_pos += len;
        Some(data)
    }

    /// 查看数据但不移动读取位置
    pub fn peek(&self, len: usize) -> Option<&[u8]> {
        if self.read_pos + len > self.write_pos {
            return None;
        }
        Some(&self.data[self.read_pos..self.read_pos + len])
    }

    /// 可读数据长度
    pub fn available(&self) -> usize {
        self.write_pos - self.read_pos
    }

    /// 重置缓冲区
    pub fn reset(&mut self) {
        self.read_pos = 0;
        self.write_pos = 0;
    }

    /// 压缩缓冲区
    pub fn compact(&mut self) {
        if self.read_pos > 0 {
            let available = self.available();
            self.data.copy_within(self.read_pos..self.write_pos, 0);
            self.read_pos = 0;
            self.write_pos = available;
        }
    }
}

impl Default for MsgBuffer {
    fn default() -> Self {
        Self::default_capacity()
    }
}

/// 网络消息处理器
pub struct MsgHandler {
    /// 本地玩家ID
    pub local_player: u8,
    /// 是否是主机
    pub is_host: bool,
    /// 接收缓冲区
    pub recv_buffer: MsgBuffer,
    /// 发送缓冲区
    pub send_buffer: MsgBuffer,
    /// 待处理消息数
    pub pending_count: u32,
}

impl MsgHandler {
    /// 创建新的消息处理器
    pub fn new(local_player: u8, is_host: bool) -> Self {
        Self {
            local_player,
            is_host,
            recv_buffer: MsgBuffer::default_capacity(),
            send_buffer: MsgBuffer::default_capacity(),
            pending_count: 0,
        }
    }

    /// 发送站立命令
    pub fn send_stand(&mut self) -> bool {
        let cmd = TCmd::new(CmdId::Stand);
        self.send_buffer.write(&[cmd.cmd])
    }

    /// 发送移动命令
    pub fn send_walk(&mut self, x: i8, y: i8) -> bool {
        let cmd = TCmdLoc::new(CmdId::WalkXY, x, y);
        let data = unsafe {
            std::slice::from_raw_parts(
                &cmd as *const _ as *const u8,
                std::mem::size_of::<TCmdLoc>(),
            )
        };
        self.send_buffer.write(data)
    }


    /// 发送攻击怪物命令
    pub fn send_attack_id(&mut self, monster_id: i16) -> bool {
        let cmd = TCmdParam1::new(CmdId::AttackId, monster_id);
        let data = unsafe {
            std::slice::from_raw_parts(
                &cmd as *const _ as *const u8,
                std::mem::size_of::<TCmdParam1>(),
            )
        };
        self.send_buffer.write(data)
    }

    /// 发送施法命令
    pub fn send_spell_xy(&mut self, x: i8, y: i8, spell_id: i16, spell_type: i16, spell_level: i16, spell_from: i16) -> bool {
        let cmd = TCmdLocParam4::new(CmdId::SpellXY, x, y, spell_id, spell_type, spell_level, spell_from);
        let data = unsafe {
            std::slice::from_raw_parts(
                &cmd as *const _ as *const u8,
                std::mem::size_of::<TCmdLocParam4>(),
            )
        };
        self.send_buffer.write(data)
    }

    /// 发送聊天消息
    pub fn send_chat(&mut self, message: &str) -> bool {
        let truncated = if message.len() > MAX_SEND_STR_LEN {
            &message[..MAX_SEND_STR_LEN]
        } else {
            message
        };

        let mut data = vec![CmdId::Chat.to_u8(), self.local_player];
        data.extend(truncated.as_bytes());
        data.push(0); // null terminator

        self.send_buffer.write(&data)
    }

    /// 发送进入新关卡命令
    pub fn send_new_level(&mut self, trigger_msg: i16, level: i16) -> bool {
        let cmd = TCmdParam2::new(CmdId::NewLvl, trigger_msg, level);
        let data = unsafe {
            std::slice::from_raw_parts(
                &cmd as *const _ as *const u8,
                std::mem::size_of::<TCmdParam2>(),
            )
        };
        self.send_buffer.write(data)
    }

    /// 发送传送门命令
    pub fn send_warp(&mut self, portal_id: i16) -> bool {
        let cmd = TCmdParam1::new(CmdId::Warp, portal_id);
        let data = unsafe {
            std::slice::from_raw_parts(
                &cmd as *const _ as *const u8,
                std::mem::size_of::<TCmdParam1>(),
            )
        };
        self.send_buffer.write(data)
    }

    /// 发送玩家死亡命令
    pub fn send_player_dead(&mut self, ear_flag: bool) -> bool {
        let cmd = TCmdParam1::new(CmdId::PlrDead, if ear_flag { 1 } else { 0 });
        let data = unsafe {
            std::slice::from_raw_parts(
                &cmd as *const _ as *const u8,
                std::mem::size_of::<TCmdParam1>(),
            )
        };
        self.send_buffer.write(data)
    }

    /// 解析接收到的命令
    pub fn parse_command(&mut self) -> Option<(CmdId, &[u8])> {
        if self.recv_buffer.available() == 0 {
            return None;
        }

        let cmd_byte = self.recv_buffer.peek(1)?[0];
        let cmd = CmdId::from_u8(cmd_byte)?;

        let cmd_size = Self::get_command_size(cmd);
        if self.recv_buffer.available() < cmd_size {
            return None;
        }

        let data = self.recv_buffer.read(cmd_size)?;
        Some((cmd, data))
    }

    /// 获取命令大小
    fn get_command_size(cmd: CmdId) -> usize {
        match cmd {
            CmdId::Stand | CmdId::CheatExperience | CmdId::Debug
            | CmdId::PlrAlive => 1,
            CmdId::WalkXY | CmdId::RAttackXY | CmdId::OpObjXY
            | CmdId::DisarmXY | CmdId::OpObjT | CmdId::OpenDoor | CmdId::CloseDoor
            | CmdId::OperateObj => std::mem::size_of::<TCmdLoc>(),
            CmdId::AddStr | CmdId::AddMag | CmdId::AddDex | CmdId::AddVit
            | CmdId::AttackId | CmdId::AttackPid | CmdId::RAttackId | CmdId::RAttackPid
            | CmdId::Resurrect | CmdId::Knockback | CmdId::Warp | CmdId::PlrDead => {
                std::mem::size_of::<TCmdParam1>()
            }
            CmdId::NewLvl | CmdId::ChangeSpellLevel | CmdId::MonstDamage => {
                std::mem::size_of::<TCmdParam2>()
            }
            CmdId::TalkXY | CmdId::MonstDeath | CmdId::GotoGetItem | CmdId::GotoAGetItem => {
                std::mem::size_of::<TCmdLocParam1>()
            }
            CmdId::SpellXY => std::mem::size_of::<TCmdLocParam4>(),
            CmdId::SpellId | CmdId::SpellPid => std::mem::size_of::<TCmdParam5>(),
            // 可变长度命令
            _ => 1, // 需要进一步解析
        }
    }

    /// 处理接收到的数据
    pub fn receive(&mut self, data: &[u8]) -> bool {
        self.recv_buffer.write(data)
    }

    /// 获取待发送数据
    pub fn get_send_data(&mut self) -> Option<Vec<u8>> {
        let len = self.send_buffer.available();
        if len == 0 {
            return None;
        }

        let data = self.send_buffer.read(len)?.to_vec();
        self.send_buffer.reset();
        Some(data)
    }

    /// 清空所有缓冲区
    pub fn clear(&mut self) {
        self.recv_buffer.reset();
        self.send_buffer.reset();
        self.pending_count = 0;
    }
}

impl Default for MsgHandler {
    fn default() -> Self {
        Self::new(0, false)
    }
}

/// 验证玩家数据包
///
/// **C++ Reference**: `Source/msg.cpp` - `ValidateField`
pub fn validate_packet_field<T: std::fmt::Display>(
    player_name: &str,
    field_name: &str,
    value: T,
    condition: bool,
) -> MsgResult {
    if !condition {
        log_failed_packet(player_name, field_name, &value.to_string());
        MsgResult::ValidationFailed
    } else {
        MsgResult::Ok
    }
}

/// 记录失败的数据包
fn log_failed_packet(player_name: &str, field_name: &str, value: &str) {
    eprintln!(
        "Remote player '{}' packet validation failed: {} = {}",
        player_name, field_name, value
    );
}

// ============================================================================
// Delta 同步数据结构 - 严格对应 Source/msg.cpp 中的内部结构
//
// 这些结构用于多人游戏关卡增量数据的导入/导出（DeltaExport/DeltaImport）
// ============================================================================

/// 最大物品数
///
/// **C++ Reference**: `items.h` - `MAXITEMS = 127`
pub const MAXITEMS: usize = 127;

/// 最大怪物数
///
/// **C++ Reference**: `monster.h` - `MaxMonsters = 200`
pub const MAX_MONSTERS: usize = 200;

/// 最大对象数
///
/// **C++ Reference**: `objects.h` - `MAXOBJECTS = 127`
pub const MAXOBJECTS: usize = 127;

/// 最大任务数
///
/// **C++ Reference**: `quests.h` - `MAXQUESTS = 24`
pub const MAXQUESTS: usize = 24;

/// 最大传送门数
///
/// **C++ Reference**: `portal.h` - `MAXPORTAL = 4`
pub const MAXPORTAL: usize = 4;

/// 怪物增量同步数据
///
/// **C++ Reference**: `Source/msg.cpp` - `DMonsterStr` (pragma pack push 1)
///
/// ```cpp
/// struct DMonsterStr {
///     WorldTilePosition position;
///     uint8_t menemy;
///     uint8_t mactive;
///     int32_t hitPoints;
///     int8_t mWhoHit;
/// };
/// ```
#[derive(Debug, Clone, Copy, Default)]
#[repr(C, packed)]
pub struct DMonsterStr {
    /// 怪物位置 X
    pub x: u8,
    /// 怪物位置 Y
    pub y: u8,
    /// 敌人ID
    pub menemy: u8,
    /// 是否活跃
    pub mactive: u8,
    /// 生命值
    pub hit_points: i32,
    /// 击中者
    pub m_who_hit: i8,
}

impl DMonsterStr {
    /// 创建表示"无效"（空槽）的怪物增量，对应 C++ 中 0xFF 填充
    pub fn invalid() -> Self {
        Self {
            x: 0xFF,
            y: 0xFF,
            menemy: 0xFF,
            mactive: 0xFF,
            hit_points: -1,
            m_who_hit: -1,
        }
    }

    /// 是否为有效的怪物增量数据
    ///
    /// **C++ Reference**: `Source/msg.cpp` - `IsMonsterDeltaValid()`
    pub fn is_valid(&self) -> bool {
        // C++: InDungeonBounds(position) && hitPoints >= 0
        self.x < 112 && self.y < 112 && self.hit_points >= 0
    }
}

/// 对象增量同步数据
///
/// **C++ Reference**: `Source/msg.cpp` - `DObjectStr`
///
/// ```cpp
/// struct DObjectStr {
///     _cmd_id bCmd;
/// };
/// ```
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DObjectStr {
    /// 命令ID（对象状态变化）
    pub cmd: u8,
}

/// 生成怪物增量数据
///
/// **C++ Reference**: `Source/msg.cpp` - `DSpawnedMonster`
#[derive(Debug, Clone, Copy, Default)]
pub struct DSpawnedMonster {
    /// 类型索引
    pub type_index: u16,
    /// 种子
    pub seed: u32,
    /// 魔像所有者玩家ID
    pub golem_owner_player_id: u8,
    /// 魔像法术等级
    pub golem_spell_level: i16,
}

/// 传送门增量数据
///
/// **C++ Reference**: `Source/msg.cpp` - `DPortal` (pragma pack push 1)
///
/// ```cpp
/// struct DPortal {
///     uint8_t x;
///     uint8_t y;
///     uint8_t level;
///     uint8_t ltype;
///     uint8_t setlvl;
/// };
/// ```
#[derive(Debug, Clone, Copy, Default)]
#[repr(C, packed)]
pub struct DPortal {
    /// X坐标
    pub x: u8,
    /// Y坐标
    pub y: u8,
    /// 关卡
    pub level: u8,
    /// 关卡类型
    pub ltype: u8,
    /// 是否设置关卡
    pub setlvl: u8,
}

impl DPortal {
    /// 创建无效（空槽）的传送门，对应 C++ 中 memset 0xFF
    pub fn invalid() -> Self {
        Self {
            x: 0xFF,
            y: 0xFF,
            level: 0xFF,
            ltype: 0xFF,
            setlvl: 0xFF,
        }
    }
}

/// 任务状态枚举（与 C++ quest_state 匹配）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum QuestState {
    /// 未激活
    QuestInvalid = 0,
    /// 未开始
    QuestNotStarted = 1,
    /// 进行中
    QuestActive = 2,
    /// 已完成
    QuestDone = 3,
}

impl Default for QuestState {
    fn default() -> Self {
        Self::QuestInvalid
    }
}

impl QuestState {
    /// 从 u8 转换
    pub fn from_u8(v: u8) -> Self {
        match v {
            1 => Self::QuestNotStarted,
            2 => Self::QuestActive,
            3 => Self::QuestDone,
            _ => Self::QuestInvalid,
        }
    }
}

/// 多人任务增量数据
///
/// **C++ Reference**: `Source/msg.cpp` - `MultiQuests` (pragma pack push 1)
///
/// ```cpp
/// struct MultiQuests {
///     quest_state qstate;
///     uint8_t qlog;
///     uint8_t qvar1;
///     uint8_t qvar2;
///     int16_t qmsg;
/// };
/// ```
#[derive(Debug, Clone, Copy, Default)]
#[repr(C, packed)]
pub struct MultiQuests {
    /// 任务状态
    pub qstate: u8,
    /// 是否记录到日志
    pub qlog: u8,
    /// 任务变量1
    pub qvar1: u8,
    /// 任务变量2
    pub qvar2: u8,
    /// 任务消息ID
    pub qmsg: i16,
}

// ============================================================================
// DeltaExport / DeltaImport - 关卡增量数据的二进制序列化
//
// 这些函数严格匹配 Source/msg.cpp 中 DeltaExportItem/DeltaImportItem 等函数
// 的二进制布局，用于多人游戏关卡状态同步
// ============================================================================

/// 序列化物品增量数组到字节缓冲区
///
/// **C++ Reference**: `Source/msg.cpp` - `DeltaExportItem()`
///
/// ```cpp
/// std::byte *DeltaExportItem(std::byte *dst, const TCmdPItem *src)
/// {
///     for (int i = 0; i < MAXITEMS; i++, src++) {
///         if (src->bCmd == CMD_INVALID) {
///             *dst++ = std::byte { 0xFF };
///         } else {
///             memcpy(dst, src, sizeof(TCmdPItem));
///             dst += sizeof(TCmdPItem);
///         }
///     }
///     return dst;
/// }
/// ```
///
/// 空槽位写入单字节 0xFF，非空槽位写入完整的 TCmdPItem（cmd 字段非 0xFF 表示有效）。
/// 返回写入的字节数。
pub fn delta_export_item(dst: &mut Vec<u8>, src: &[TCmdPItem]) -> usize {
    let start = dst.len();
    for item in src.iter().take(MAXITEMS) {
        if item.cmd == 0xFF {
            dst.push(0xFF);
        } else {
            // 序列化整个 TCmdPItem
            unsafe {
                let bytes = std::slice::from_raw_parts(
                    item as *const TCmdPItem as *const u8,
                    std::mem::size_of::<TCmdPItem>(),
                );
                dst.extend_from_slice(bytes);
            }
        }
    }
    dst.len() - start
}

/// 从字节缓冲区反序列化物品增量数组
///
/// **C++ Reference**: `Source/msg.cpp` - `DeltaImportItem()`
///
/// 返回导入的字节数（Some）或解析失败（None）。
pub fn delta_import_item(src: &[u8], dst: &mut [TCmdPItem]) -> Option<usize> {
    let mut offset = 0usize;
    let item_size = std::mem::size_of::<TCmdPItem>();

    for slot in dst.iter_mut().take(MAXITEMS) {
        if offset >= src.len() {
            return None;
        }
        if src[offset] == 0xFF {
            // 空槽 - 填充 0xFF
            unsafe {
                std::ptr::write_bytes(slot as *mut TCmdPItem as *mut u8, 0xFF, item_size);
            }
            offset += 1;
        } else {
            if offset + item_size > src.len() {
                return None;
            }
            unsafe {
                std::ptr::copy_nonoverlapping(
                    src.as_ptr().add(offset) as *const TCmdPItem,
                    slot as *mut TCmdPItem,
                    1,
                );
            }
            // 如果物品无效，则填充 0xFF（对应 C++ IsItemDeltaValid 检查）
            if slot.cmd != 0xFF && !is_item_delta_valid(slot) {
                unsafe {
                    std::ptr::write_bytes(slot as *mut TCmdPItem as *mut u8, 0xFF, item_size);
                }
            }
            offset += item_size;
        }
    }
    Some(offset)
}

/// 检查物品增量是否有效
///
/// **C++ Reference**: `Source/msg.cpp` - `IsItemDeltaValid()`
fn is_item_delta_valid(item: &TCmdPItem) -> bool {
    // 简化检查：cmd 不为 INVALID 且坐标在地牢范围内
    item.cmd != 0xFF && item.x >= 0 && item.y >= 0
}

/// 序列化对象增量映射到字节缓冲区
///
/// **C++ Reference**: `Source/msg.cpp` - `DeltaExportObject()`
///
/// ```cpp
/// std::byte *DeltaExportObject(std::byte *dst, const map<WorldTilePosition, DObjectStr> &src)
/// {
///     *dst++ = static_cast<std::byte>(src.size());
///     for (const auto &[position, obj] : src) {
///         *dst++ = static_cast<std::byte>(position.x);
///         *dst++ = static_cast<std::byte>(position.y);
///         *dst++ = static_cast<std::byte>(obj.bCmd);
///     }
///     return dst;
/// }
/// ```
pub fn delta_export_object(dst: &mut Vec<u8>, src: &[(u8, u8, DObjectStr)]) -> usize {
    let start = dst.len();
    let count = src.len().min(MAXOBJECTS);
    dst.push(count as u8);
    for &(x, y, ref obj) in src.iter().take(count) {
        dst.push(x);
        dst.push(y);
        dst.push(obj.cmd);
    }
    dst.len() - start
}

/// 从字节缓冲区反序列化对象增量映射
///
/// **C++ Reference**: `Source/msg.cpp` - `DeltaImportObjects()`
pub fn delta_import_object(src: &[u8], dst: &mut Vec<(u8, u8, DObjectStr)>) -> Option<usize> {
    if src.is_empty() {
        return None;
    }
    let count = src[0] as usize;
    if count > MAXOBJECTS {
        return None;
    }
    let needed = 1 + 3 * count;
    if src.len() < needed {
        return None;
    }
    dst.clear();
    dst.reserve(count);
    let mut offset = 1;
    for _ in 0..count {
        let x = src[offset];
        let y = src[offset + 1];
        let cmd = src[offset + 2];
        dst.push((x, y, DObjectStr { cmd }));
        offset += 3;
    }
    Some(offset)
}

/// 序列化怪物增量数组到字节缓冲区
///
/// **C++ Reference**: `Source/msg.cpp` - `DeltaExportMonster()`
///
/// ```cpp
/// std::byte *DeltaExportMonster(std::byte *dst, const DMonsterStr *src)
/// {
///     for (size_t i = 0; i < MaxMonsters; i++, src++) {
///         if (src->position.x == 0xFF) {
///             *dst++ = std::byte { 0xFF };
///         } else {
///             memcpy(dst, src, sizeof(DMonsterStr));
///             dst += sizeof(DMonsterStr);
///         }
///     }
///     return dst;
/// }
/// ```
pub fn delta_export_monster(dst: &mut Vec<u8>, src: &[DMonsterStr]) -> usize {
    let start = dst.len();
    let mon_size = std::mem::size_of::<DMonsterStr>();
    for m in src.iter().take(MAX_MONSTERS) {
        if m.x == 0xFF {
            dst.push(0xFF);
        } else {
            unsafe {
                let bytes = std::slice::from_raw_parts(m as *const DMonsterStr as *const u8, mon_size);
                dst.extend_from_slice(bytes);
            }
        }
    }
    dst.len() - start
}

/// 从字节缓冲区反序列化怪物增量数组
///
/// **C++ Reference**: `Source/msg.cpp` - `DeltaImportMonster()`
pub fn delta_import_monster(src: &[u8], dst: &mut [DMonsterStr]) -> Option<usize> {
    let mut offset = 0usize;
    let mon_size = std::mem::size_of::<DMonsterStr>();
    for slot in dst.iter_mut().take(MAX_MONSTERS) {
        if offset >= src.len() {
            return None;
        }
        if src[offset] == 0xFF {
            *slot = DMonsterStr::invalid();
            offset += 1;
        } else {
            if offset + mon_size > src.len() {
                return None;
            }
            unsafe {
                std::ptr::copy_nonoverlapping(
                    src.as_ptr().add(offset) as *const DMonsterStr,
                    slot as *mut DMonsterStr,
                    1,
                );
            }
            offset += mon_size;
        }
    }
    Some(offset)
}

/// 序列化传送门和任务增量数据到字节缓冲区
///
/// **C++ Reference**: `Source/msg.cpp` - `DeltaExportJunk()`
///
/// 传送门部分：每个 portal.x==0xFF 写入单字节 0xFF，否则写入完整 DPortal。
/// 任务部分：逐个写入 MultiQuests。
pub fn delta_export_junk(
    dst: &mut Vec<u8>,
    portals: &[DPortal],
    quests: &[MultiQuests],
) -> usize {
    let start = dst.len();
    let portal_size = std::mem::size_of::<DPortal>();
    let quest_size = std::mem::size_of::<MultiQuests>();

    // 传送门
    for portal in portals.iter().take(MAXPORTAL) {
        if portal.x == 0xFF {
            dst.push(0xFF);
        } else {
            unsafe {
                let bytes = std::slice::from_raw_parts(portal as *const DPortal as *const u8, portal_size);
                dst.extend_from_slice(bytes);
            }
        }
    }

    // 任务
    for quest in quests.iter().take(MAXQUESTS) {
        unsafe {
            let bytes = std::slice::from_raw_parts(quest as *const MultiQuests as *const u8, quest_size);
            dst.extend_from_slice(bytes);
        }
    }

    dst.len() - start
}

/// 从字节缓冲区反序列化传送门和任务增量数据
///
/// **C++ Reference**: `Source/msg.cpp` - `DeltaImportJunk()`
pub fn delta_import_junk(
    src: &[u8],
    portals: &mut [DPortal],
    quests: &mut [MultiQuests],
) -> Option<usize> {
    let mut offset = 0usize;
    let portal_size = std::mem::size_of::<DPortal>();
    let quest_size = std::mem::size_of::<MultiQuests>();

    // 传送门
    for portal in portals.iter_mut().take(MAXPORTAL) {
        if offset >= src.len() {
            return None;
        }
        if src[offset] == 0xFF {
            *portal = DPortal::invalid();
            offset += 1;
        } else {
            if offset + portal_size > src.len() {
                return None;
            }
            unsafe {
                std::ptr::copy_nonoverlapping(
                    src.as_ptr().add(offset) as *const DPortal,
                    portal as *mut DPortal,
                    1,
                );
            }
            offset += portal_size;
        }
    }

    // 任务
    for quest in quests.iter_mut().take(MAXQUESTS) {
        if offset + quest_size > src.len() {
            return None;
        }
        unsafe {
            std::ptr::copy_nonoverlapping(
                src.as_ptr().add(offset) as *const MultiQuests,
                quest as *mut MultiQuests,
                1,
            );
        }
        offset += quest_size;
    }

    Some(offset)
}

// ============================================================================
// NetSendCmd 系列 - 命令发送函数（对应 Source/msg.cpp 的 NetSendCmd* 系列）
//
// C++ 中这些函数最终通过 NetSendHiPri/NetSendLoPri 发送。
// 这里提供构造命令字节数组并写入发送缓冲区的逻辑。
// ============================================================================

impl MsgHandler {
    /// 发送通用命令（仅命令ID）
    ///
    /// **C++ Reference**: `Source/msg.cpp` - `NetSendCmd(bool bHiPri, _cmd_id bCmd)`
    pub fn net_send_cmd(&mut self, hi_pri: bool, cmd: CmdId) -> bool {
        let _ = hi_pri;
        let cmd = TCmd::new(cmd);
        let data = unsafe {
            std::slice::from_raw_parts(&cmd as *const _ as *const u8, std::mem::size_of::<TCmd>())
        };
        self.send_buffer.write(data)
    }

    /// 发送带位置的命令
    ///
    /// **C++ Reference**: `Source/msg.cpp` - `NetSendCmdLoc()`
    pub fn net_send_cmd_loc(
        &mut self,
        player_id: u8,
        hi_pri: bool,
        cmd: CmdId,
        x: i8,
        y: i8,
    ) -> bool {
        let _ = (hi_pri, player_id);
        let cmd = TCmdLoc::new(cmd, x, y);
        let data = unsafe {
            std::slice::from_raw_parts(&cmd as *const _ as *const u8, std::mem::size_of::<TCmdLoc>())
        };
        self.send_buffer.write(data)
    }

    /// 发送带位置和1个参数的命令
    ///
    /// **C++ Reference**: `Source/msg.cpp` - `NetSendCmdLocParam1()`
    pub fn net_send_cmd_loc_param1(
        &mut self,
        hi_pri: bool,
        cmd: CmdId,
        x: i8,
        y: i8,
        param1: i16,
    ) -> bool {
        let _ = hi_pri;
        let cmd = TCmdLocParam1::new(cmd, x, y, param1);
        let data = unsafe {
            std::slice::from_raw_parts(
                &cmd as *const _ as *const u8,
                std::mem::size_of::<TCmdLocParam1>(),
            )
        };
        self.send_buffer.write(data)
    }

    /// 发送带位置和2个参数的命令
    ///
    /// **C++ Reference**: `Source/msg.cpp` - `NetSendCmdLocParam2()`
    pub fn net_send_cmd_loc_param2(
        &mut self,
        hi_pri: bool,
        cmd: CmdId,
        x: i8,
        y: i8,
        param1: i16,
        param2: i16,
    ) -> bool {
        let _ = hi_pri;
        // TCmdLocParam2 在文件中未定义，这里手动构造
        #[repr(C, packed)]
        struct LocParam2 {
            cmd: u8,
            x: i8,
            y: i8,
            param1: i16,
            param2: i16,
        }
        let cmd = LocParam2 {
            cmd: cmd.to_u8(),
            x,
            y,
            param1,
            param2,
        };
        let data = unsafe {
            std::slice::from_raw_parts(&cmd as *const _ as *const u8, std::mem::size_of::<LocParam2>())
        };
        self.send_buffer.write(data)
    }

    /// 发送带位置和3个参数的命令
    ///
    /// **C++ Reference**: `Source/msg.cpp` - `NetSendCmdLocParam3()`
    pub fn net_send_cmd_loc_param3(
        &mut self,
        hi_pri: bool,
        cmd: CmdId,
        x: i8,
        y: i8,
        param1: i16,
        param2: i16,
        param3: i16,
    ) -> bool {
        let _ = hi_pri;
        #[repr(C, packed)]
        struct LocParam3 {
            cmd: u8,
            x: i8,
            y: i8,
            param1: i16,
            param2: i16,
            param3: i16,
        }
        let cmd = LocParam3 {
            cmd: cmd.to_u8(),
            x,
            y,
            param1,
            param2,
            param3,
        };
        let data = unsafe {
            std::slice::from_raw_parts(&cmd as *const _ as *const u8, std::mem::size_of::<LocParam3>())
        };
        self.send_buffer.write(data)
    }

    /// 发送带位置和4个参数的命令（法术）
    ///
    /// **C++ Reference**: `Source/msg.cpp` - `NetSendCmdLocParam4()`
    pub fn net_send_cmd_loc_param4(
        &mut self,
        hi_pri: bool,
        cmd: CmdId,
        x: i8,
        y: i8,
        spell_id: i16,
        spell_type: i16,
        spell_level: i16,
        spell_from: i16,
    ) -> bool {
        let _ = hi_pri;
        let cmd =
            TCmdLocParam4::new(cmd, x, y, spell_id, spell_type, spell_level, spell_from);
        let data = unsafe {
            std::slice::from_raw_parts(
                &cmd as *const _ as *const u8,
                std::mem::size_of::<TCmdLocParam4>(),
            )
        };
        self.send_buffer.write(data)
    }

    /// 发送带1个参数的命令
    ///
    /// **C++ Reference**: `Source/msg.cpp` - `NetSendCmdParam1()`
    pub fn net_send_cmd_param1(&mut self, hi_pri: bool, cmd: CmdId, param1: i16) -> bool {
        let _ = hi_pri;
        let cmd = TCmdParam1::new(cmd, param1);
        let data = unsafe {
            std::slice::from_raw_parts(
                &cmd as *const _ as *const u8,
                std::mem::size_of::<TCmdParam1>(),
            )
        };
        self.send_buffer.write(data)
    }

    /// 发送带2个参数的命令
    ///
    /// **C++ Reference**: `Source/msg.cpp` - `NetSendCmdParam2()`
    pub fn net_send_cmd_param2(
        &mut self,
        hi_pri: bool,
        cmd: CmdId,
        param1: i16,
        param2: i16,
    ) -> bool {
        let _ = hi_pri;
        let cmd = TCmdParam2::new(cmd, param1, param2);
        let data = unsafe {
            std::slice::from_raw_parts(
                &cmd as *const _ as *const u8,
                std::mem::size_of::<TCmdParam2>(),
            )
        };
        self.send_buffer.write(data)
    }

    /// 发送带4个参数的命令
    ///
    /// **C++ Reference**: `Source/msg.cpp` - `NetSendCmdParam4()`
    pub fn net_send_cmd_param4(
        &mut self,
        hi_pri: bool,
        cmd: CmdId,
        param1: i16,
        param2: i16,
        param3: i16,
        param4: i16,
    ) -> bool {
        let _ = hi_pri;
        #[repr(C, packed)]
        struct Param4 {
            cmd: u8,
            param1: i16,
            param2: i16,
            param3: i16,
            param4: i16,
        }
        let cmd = Param4 {
            cmd: cmd.to_u8(),
            param1,
            param2,
            param3,
            param4,
        };
        let data = unsafe {
            std::slice::from_raw_parts(&cmd as *const _ as *const u8, std::mem::size_of::<Param4>())
        };
        self.send_buffer.write(data)
    }

    /// 发送任务命令
    ///
    /// **C++ Reference**: `Source/msg.cpp` - `NetSendCmdQuest()`
    pub fn net_send_cmd_quest(
        &mut self,
        hi_pri: bool,
        quest_idx: u8,
        qstate: u8,
        qlog: u8,
        qvar1: u8,
        qvar2: u8,
        qmsg: i16,
    ) -> bool {
        let _ = hi_pri;
        // TCmdQuest 结构
        #[repr(C, packed)]
        struct TCmdQuest {
            cmd: u8,
            q: u8,
            qstate: u8,
            qlog: u8,
            qvar1: u8,
            qvar2: u8,
            qmsg: i16,
        }
        let cmd = TCmdQuest {
            cmd: CmdId::SyncQuest.to_u8(),
            q: quest_idx,
            qstate,
            qlog,
            qvar1,
            qvar2,
            qmsg,
        };
        let data = unsafe {
            std::slice::from_raw_parts(
                &cmd as *const _ as *const u8,
                std::mem::size_of::<TCmdQuest>(),
            )
        };
        self.send_buffer.write(data)
    }

    /// 发送字符串命令
    ///
    /// **C++ Reference**: `Source/msg.cpp` - `NetSendCmdString()`: the packet is
    /// `[CMD_STRING][UTF-8 text][NUL]` — no player mask in the body (pmask is
    /// the addressing mask passed to `multi_send_msg_packet`).
    pub fn net_send_cmd_string(&mut self, pmask: u32, text: &str) -> bool {
        let _ = pmask;
        // C++ `CMD_STRING` = 61 (msg.h). NOTE: the Rust `CmdId` enum numbering
        // diverges from C++ `_cmd_id` at value 42+, so the raw C++ value is used
        // here for wire compatibility until the enum is renumbered.
        const CMD_STRING: u8 = 61;
        let mut data = vec![CMD_STRING];
        let truncated = if text.len() > MAX_SEND_STR_LEN {
            &text[..MAX_SEND_STR_LEN]
        } else {
            text
        };
        data.extend_from_slice(truncated.as_bytes());
        data.push(0);
        self.send_buffer.write(&data)
    }
}

//
// TESTS
//

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cmd_id_from_u8() {
        assert_eq!(CmdId::from_u8(0), Some(CmdId::Stand));
        assert_eq!(CmdId::from_u8(1), Some(CmdId::WalkXY));
        assert_eq!(CmdId::from_u8(255), None);
    }

    #[test]
    fn test_cmd_id_to_u8() {
        assert_eq!(CmdId::Stand.to_u8(), 0);
        assert_eq!(CmdId::WalkXY.to_u8(), 1);
    }

    #[test]
    fn test_cmd_is_movement() {
        assert!(CmdId::WalkXY.is_movement());
        assert!(CmdId::GotoGetItem.is_movement());
        assert!(!CmdId::RAttackXY.is_movement());
    }

    #[test]
    fn test_cmd_is_attack() {
        assert!(CmdId::RAttackXY.is_attack());
        assert!(CmdId::AttackId.is_attack());
        assert!(!CmdId::WalkXY.is_attack());
    }

    #[test]
    fn test_cmd_is_spell() {
        assert!(CmdId::SpellXY.is_spell());
        assert!(CmdId::SpellId.is_spell());
        assert!(!CmdId::RAttackXY.is_spell());
    }

    #[test]
    fn test_tcmd_loc() {
        let cmd = TCmdLoc::new(CmdId::WalkXY, 10, 20);
        assert_eq!(cmd.cmd, 1);
        assert_eq!(cmd.x, 10);
        assert_eq!(cmd.y, 20);
        assert_eq!(cmd.position(), Point::new(10, 20));
    }

    #[test]
    fn test_tcmd_param1() {
        let cmd = TCmdParam1::new(CmdId::AttackId, 42);
        assert_eq!(cmd.cmd, CmdId::AttackId.to_u8());
        let param1 = { cmd.param1 }; // 复制避免对齐问题
        assert_eq!(param1, 42);
    }

    #[test]
    fn test_tcmd_param2() {
        let cmd = TCmdParam2::new(CmdId::NewLvl, 1, 5);
        assert_eq!(cmd.cmd, CmdId::NewLvl.to_u8());
        let param1 = { cmd.param1 }; // 复制避免对齐问题
        let param2 = { cmd.param2 };
        assert_eq!(param1, 1);
        assert_eq!(param2, 5);
    }

    #[test]
    fn test_msg_buffer_write_read() {
        let mut buffer = MsgBuffer::new(64);

        assert!(buffer.write(&[1, 2, 3, 4]));
        assert_eq!(buffer.available(), 4);

        let data = buffer.read(2).unwrap();
        assert_eq!(data, &[1, 2]);
        assert_eq!(buffer.available(), 2);
    }

    #[test]
    fn test_msg_buffer_peek() {
        let mut buffer = MsgBuffer::new(64);
        buffer.write(&[1, 2, 3, 4]);

        let data = buffer.peek(2).unwrap();
        assert_eq!(data, &[1, 2]);
        assert_eq!(buffer.available(), 4); // 不变
    }

    #[test]
    fn test_msg_buffer_compact() {
        let mut buffer = MsgBuffer::new(64);
        buffer.write(&[1, 2, 3, 4]);
        buffer.read(2);

        buffer.compact();
        assert_eq!(buffer.available(), 2);

        let data = buffer.read(2).unwrap();
        assert_eq!(data, &[3, 4]);
    }

    #[test]
    fn test_msg_handler_send_stand() {
        let mut handler = MsgHandler::new(0, false);
        assert!(handler.send_stand());

        let data = handler.get_send_data().unwrap();
        assert_eq!(data[0], CmdId::Stand.to_u8());
    }

    #[test]
    fn test_msg_handler_send_walk() {
        let mut handler = MsgHandler::new(0, false);
        assert!(handler.send_walk(10, 20));

        let data = handler.get_send_data().unwrap();
        assert_eq!(data[0], CmdId::WalkXY.to_u8());
        assert_eq!(data[1], 10);
        assert_eq!(data[2], 20);
    }

    #[test]
    fn test_msg_handler_send_chat() {
        let mut handler = MsgHandler::new(1, false);
        assert!(handler.send_chat("Hello"));

        let data = handler.get_send_data().unwrap();
        assert_eq!(data[0], CmdId::Chat.to_u8());
        assert_eq!(data[1], 1); // player id
        assert_eq!(&data[2..7], b"Hello");
    }

    #[test]
    fn test_msg_handler_send_chat_truncate() {
        let mut handler = MsgHandler::new(0, false);
        let long_msg = "A".repeat(100);
        assert!(handler.send_chat(&long_msg));

        let data = handler.get_send_data().unwrap();
        // 应该被截断到 MAX_SEND_STR_LEN
        assert!(data.len() <= MAX_SEND_STR_LEN + 3); // cmd + player + null
    }

    #[test]
    fn test_msg_handler_receive_parse() {
        let mut handler = MsgHandler::new(0, false);

        // 模拟接收 WalkXY 命令
        let cmd = TCmdLoc::new(CmdId::WalkXY, 5, 10);
        let data = unsafe {
            std::slice::from_raw_parts(
                &cmd as *const _ as *const u8,
                std::mem::size_of::<TCmdLoc>(),
            )
        };

        assert!(handler.receive(data));

        let (parsed_cmd, parsed_data) = handler.parse_command().unwrap();
        assert_eq!(parsed_cmd, CmdId::WalkXY);
        assert_eq!(parsed_data[1], 5);
        assert_eq!(parsed_data[2], 10);
    }

    #[test]
    fn test_validate_packet_field() {
        let result = validate_packet_field("Player1", "hp", 100, 100 > 0);
        assert_eq!(result, MsgResult::Ok);

        let result = validate_packet_field("Player1", "hp", -1, -1 > 0);
        assert_eq!(result, MsgResult::ValidationFailed);
    }

    #[test]
    fn test_get_command_size() {
        assert_eq!(MsgHandler::get_command_size(CmdId::Stand), 1);
        assert_eq!(MsgHandler::get_command_size(CmdId::WalkXY), std::mem::size_of::<TCmdLoc>());
        assert_eq!(MsgHandler::get_command_size(CmdId::AttackId), std::mem::size_of::<TCmdParam1>());
    }

    // ========================================================================
    // Delta 结构测试
    // ========================================================================

    #[test]
    fn test_d_monster_str_default_and_invalid() {
        let m = DMonsterStr::default();
        assert_eq!(m.x, 0);
        assert_eq!(m.y, 0);
        // 默认: x=0,y=0 在地牢范围内，hit_points=0 >= 0 -> 有效
        assert!(m.is_valid());

        let inv = DMonsterStr::invalid();
        assert_eq!(inv.x, 0xFF);
        assert!(!inv.is_valid()); // x=0xFF 超出地牢范围
    }

    #[test]
    fn test_d_monster_str_valid() {
        let m = DMonsterStr {
            x: 50,
            y: 60,
            menemy: 1,
            mactive: 1,
            hit_points: 100,
            m_who_hit: 0,
        };
        assert!(m.is_valid());
    }

    #[test]
    fn test_d_portal_invalid() {
        let p = DPortal::invalid();
        assert_eq!(p.x, 0xFF);
        assert_eq!(p.level, 0xFF);
    }

    #[test]
    fn test_quest_state_from_u8() {
        assert_eq!(QuestState::from_u8(0), QuestState::QuestInvalid);
        assert_eq!(QuestState::from_u8(1), QuestState::QuestNotStarted);
        assert_eq!(QuestState::from_u8(2), QuestState::QuestActive);
        assert_eq!(QuestState::from_u8(3), QuestState::QuestDone);
        assert_eq!(QuestState::from_u8(99), QuestState::QuestInvalid);
    }

    #[test]
    fn test_delta_constants() {
        assert_eq!(MAXITEMS, 127);
        assert_eq!(MAXOBJECTS, 127);
        assert_eq!(MAXQUESTS, 24);
        assert_eq!(MAXPORTAL, 4);
        assert_eq!(MAX_MONSTERS, 200);
    }

    // ========================================================================
    // DeltaExport / DeltaImport 往返测试
    // ========================================================================

    #[test]
    fn test_delta_export_import_monster_roundtrip() {
        // 构造 MAX_MONSTERS 大小的数组，部分有效、部分空槽（x=0xFF）
        let mut src = [DMonsterStr::default(); MAX_MONSTERS];
        src[0] = DMonsterStr {
            x: 10,
            y: 20,
            menemy: 1,
            mactive: 1,
            hit_points: 100,
            m_who_hit: 0,
        };
        src[1] = DMonsterStr::invalid(); // 空槽
        src[2] = DMonsterStr {
            x: 30,
            y: 40,
            menemy: 5,
            mactive: 1,
            hit_points: 50,
            m_who_hit: 1,
        };
        // 其余保持 default（x=0 -> 有效）

        let mut buf = Vec::new();
        let written = delta_export_monster(&mut buf, &src);
        assert!(written > 0);

        let mut dst = [DMonsterStr::default(); MAX_MONSTERS];
        let read = delta_import_monster(&buf, &mut dst).expect("import should succeed");

        assert_eq!(written, read);
        assert_eq!(dst[0].x, 10);
        assert_eq!(dst[0].y, 20);
        // packed 结构 - 通过拷贝读取 i32 字段
        let hp = { dst[0].hit_points };
        assert_eq!(hp, 100);
        // 空槽导入后应为 invalid
        assert_eq!(dst[1].x, 0xFF);
        assert_eq!(dst[2].x, 30);
        let mwho = { dst[2].m_who_hit };
        assert_eq!(mwho, 1);
    }

    #[test]
    fn test_delta_export_monster_all_empty() {
        // 全部空槽 - 每个只写 1 字节
        let src = [DMonsterStr::invalid(); MAX_MONSTERS];
        let mut buf = Vec::new();
        let written = delta_export_monster(&mut buf, &src);
        assert_eq!(written, MAX_MONSTERS); // 每槽 1 字节
    }

    #[test]
    fn test_delta_import_monster_truncated_fails() {
        let buf = [0xFFu8; 5]; // 远不足以容纳 MAX_MONSTERS 个槽
        let mut dst = [DMonsterStr::default(); MAX_MONSTERS];
        let result = delta_import_monster(&buf, &mut dst);
        assert!(result.is_none());
    }

    #[test]
    fn test_delta_export_import_object_roundtrip() {
        let src = vec![
            (10, 20, DObjectStr { cmd: 0x41 }),
            (30, 40, DObjectStr { cmd: 0x42 }),
            (50, 60, DObjectStr { cmd: 0x43 }),
        ];
        let mut buf = Vec::new();
        let written = delta_export_object(&mut buf, &src);
        assert_eq!(written, 1 + 3 * 3); // count + 3 bytes per entry

        let mut dst = Vec::new();
        let read = delta_import_object(&buf, &mut dst).expect("import should succeed");
        assert_eq!(read, written);
        assert_eq!(dst.len(), 3);
        assert_eq!(dst[0].0, 10);
        assert_eq!(dst[0].1, 20);
        assert_eq!(dst[0].2.cmd, 0x41);
        assert_eq!(dst[2].2.cmd, 0x43);
    }

    #[test]
    fn test_delta_import_object_too_many() {
        // count 字节声称有 200 个对象（超过 MAXOBJECTS=127）
        let mut buf = vec![200u8];
        buf.extend(std::iter::repeat(0).take(3 * 200));
        let mut dst = Vec::new();
        let result = delta_import_object(&buf, &mut dst);
        assert!(result.is_none());
    }

    #[test]
    fn test_delta_export_import_junk_roundtrip() {
        let mut portals = [DPortal::default(); MAXPORTAL];
        portals[0] = DPortal {
            x: 50,
            y: 60,
            level: 5,
            ltype: 1,
            setlvl: 0,
        };
        portals[1] = DPortal::invalid(); // 空槽

        let mut quests = [MultiQuests::default(); MAXQUESTS];
        quests[0] = MultiQuests {
            qstate: 2,
            qlog: 1,
            qvar1: 10,
            qvar2: 20,
            qmsg: 5,
        };

        let mut buf = Vec::new();
        let written = delta_export_junk(&mut buf, &portals, &quests);
        assert!(written > 0);

        let mut dst_portals = [DPortal::default(); MAXPORTAL];
        let mut dst_quests = [MultiQuests::default(); MAXQUESTS];
        let read =
            delta_import_junk(&buf, &mut dst_portals, &mut dst_quests).expect("import should work");
        assert_eq!(read, written);

        assert_eq!(dst_portals[0].x, 50);
        assert_eq!(dst_portals[0].level, 5);
        assert_eq!(dst_portals[1].x, 0xFF); // 空槽
        assert_eq!(dst_quests[0].qstate, 2);
        assert_eq!(dst_quests[0].qvar1, 10);
        let qmsg = { dst_quests[0].qmsg };
        assert_eq!(qmsg, 5);
    }

    #[test]
    fn test_delta_export_item_mixed() {
        // 混合有效和空槽。空槽用 cmd=0xFF（对应 C++ CMD_INVALID / memset 0xFF）
        let item_size = std::mem::size_of::<TCmdPItem>();

        // 创建全部为空槽（0xFF）的数组
        let mut src: [TCmdPItem; MAXITEMS] = [TCmdPItem {
            cmd: 0xFF,
            x: -1,
            y: -1,
            item_id: 0xFF,
            item_seed: 0xFFFF_FFFF,
            item_idx: 0xFFFF,
            item_cf: 0xFFFF,
            item_data: [0xFF; 32],
        }; MAXITEMS];

        // 第 0 个为有效物品
        src[0] = TCmdPItem {
            cmd: 0x10,
            x: 10,
            y: 20,
            item_id: 1,
            item_seed: 12345,
            item_idx: 5,
            item_cf: 0,
            item_data: [0; 32],
        };

        let mut buf = Vec::new();
        let written = delta_export_item(&mut buf, &src);
        // 第 0 个写完整结构，其余 126 个空槽每个写 1 字节
        let expected = item_size + (MAXITEMS - 1);
        assert_eq!(written, expected);

        let mut dst: [TCmdPItem; MAXITEMS] =
            [TCmdPItem::default(); MAXITEMS];
        let read = delta_import_item(&buf, &mut dst).expect("import should succeed");
        assert_eq!(read, written);
        assert_eq!(dst[0].cmd, 0x10);
        assert_eq!(dst[0].x, 10);
        assert_eq!(dst[1].cmd, 0xFF); // 空槽
    }

    // ========================================================================
    // NetSendCmd 系列测试
    // ========================================================================

    #[test]
    fn test_net_send_cmd_basic() {
        let mut h = MsgHandler::new(0, false);
        assert!(h.net_send_cmd(true, CmdId::Stand));
        let data = h.get_send_data().unwrap();
        assert_eq!(data.len(), std::mem::size_of::<TCmd>());
        assert_eq!(data[0], CmdId::Stand.to_u8());
    }

    #[test]
    fn test_net_send_cmd_loc() {
        let mut h = MsgHandler::new(0, false);
        assert!(h.net_send_cmd_loc(0, true, CmdId::WalkXY, 10, 20));
        let data = h.get_send_data().unwrap();
        assert_eq!(data.len(), std::mem::size_of::<TCmdLoc>());
        assert_eq!(data[0], CmdId::WalkXY.to_u8());
        assert_eq!(data[1], 10);
        assert_eq!(data[2], 20);
    }

    #[test]
    fn test_net_send_cmd_loc_param1() {
        let mut h = MsgHandler::new(0, false);
        assert!(h.net_send_cmd_loc_param1(true, CmdId::GotoGetItem, 5, 6, 42));
        let data = h.get_send_data().unwrap();
        assert_eq!(data[0], CmdId::GotoGetItem.to_u8());
        assert_eq!(data[1], 5);
        assert_eq!(data[2], 6);
    }

    #[test]
    fn test_net_send_cmd_loc_param2() {
        let mut h = MsgHandler::new(0, false);
        assert!(h.net_send_cmd_loc_param2(true, CmdId::RAttackXY, 1, 2, 100, 200));
        let data = h.get_send_data().unwrap();
        assert_eq!(data[0], CmdId::RAttackXY.to_u8());
        assert_eq!(data.len(), 1 + 2 + 2 * 2); // cmd + xy + 2 params
    }

    #[test]
    fn test_net_send_cmd_loc_param3() {
        let mut h = MsgHandler::new(0, false);
        assert!(h.net_send_cmd_loc_param3(true, CmdId::RAttackXY, 1, 2, 10, 20, 30));
        let data = h.get_send_data().unwrap();
        assert_eq!(data.len(), 1 + 2 + 3 * 2);
    }

    #[test]
    fn test_net_send_cmd_loc_param4() {
        let mut h = MsgHandler::new(0, false);
        assert!(h.net_send_cmd_loc_param4(
            true, CmdId::SpellXY, 5, 6, 1, 2, 3, 4
        ));
        let data = h.get_send_data().unwrap();
        assert_eq!(data[0], CmdId::SpellXY.to_u8());
        assert_eq!(data.len(), std::mem::size_of::<TCmdLocParam4>());
    }

    #[test]
    fn test_net_send_cmd_param1() {
        let mut h = MsgHandler::new(0, false);
        assert!(h.net_send_cmd_param1(true, CmdId::AttackId, 42));
        let data = h.get_send_data().unwrap();
        assert_eq!(data[0], CmdId::AttackId.to_u8());
        assert_eq!(data.len(), std::mem::size_of::<TCmdParam1>());
    }

    #[test]
    fn test_net_send_cmd_param2() {
        let mut h = MsgHandler::new(0, false);
        assert!(h.net_send_cmd_param2(true, CmdId::NewLvl, 1, 5));
        let data = h.get_send_data().unwrap();
        assert_eq!(data[0], CmdId::NewLvl.to_u8());
        assert_eq!(data.len(), std::mem::size_of::<TCmdParam2>());
    }

    #[test]
    fn test_net_send_cmd_param4() {
        let mut h = MsgHandler::new(0, false);
        assert!(h.net_send_cmd_param4(true, CmdId::Stand, 1, 2, 3, 4));
        let data = h.get_send_data().unwrap();
        // Param4 = 1 (cmd) + 4*2 (params) = 9 字节
        assert_eq!(data.len(), 9);
    }

    #[test]
    fn test_net_send_cmd_quest() {
        let mut h = MsgHandler::new(0, false);
        assert!(h.net_send_cmd_quest(true, 2, 1, 1, 10, 20, 5));
        let data = h.get_send_data().unwrap();
        assert_eq!(data[0], CmdId::SyncQuest.to_u8());
        // TCmdQuest: cmd(1) + q(1) + qstate(1) + qlog(1) + qvar1(1) + qvar2(1) + qmsg(2) = 8
        assert_eq!(data.len(), 8);
    }

    #[test]
    fn test_net_send_cmd_string() {
        let mut h = MsgHandler::new(0, false);
        assert!(h.net_send_cmd_string(0xFFFF_FFFF, "Hello"));
        let data = h.get_send_data().unwrap();
        // C++ NetSendCmdString: [CMD_STRING=61][utf8][NUL]; the pmask is the
        // addressing mask, not part of the packet body.
        assert_eq!(data[0], 61);
        assert_eq!(&data[1..6], b"Hello");
        assert_eq!(data[6], 0);
        assert_eq!(data.len(), 7);
    }

    #[test]
    fn test_net_send_cmd_string_truncates() {
        let mut h = MsgHandler::new(0, false);
        let long = "A".repeat(200);
        assert!(h.net_send_cmd_string(0, &long));
        let data = h.get_send_data().unwrap();
        // 应截断到 MAX_SEND_STR_LEN
        // cmd(1) + pmask(4) + truncated(MAX_SEND_STR_LEN) + null(1)
        assert!(data.len() <= 1 + 4 + MAX_SEND_STR_LEN + 1);
    }
    #[test]
    fn test_net_send_cmd_string_cpp_layout() {
        let mut handler = MsgHandler::new(0, false);
        assert!(handler.net_send_cmd_string(0, "hi"));
        let data = handler.send_buffer.read(4).expect("4 bytes written");
        // C++ NetSendCmdString: [CMD_STRING=61][utf8][NUL], no mask in body.
        assert_eq!(data[0], 61);
        assert_eq!(&data[1..3], b"hi");
        assert_eq!(data[3], 0);
    }


    #[test]
    fn test_cmd_id_values_match_cpp() {
        use super::CmdId;
        // The first 42 values are shared and must equal C++ `_cmd_id` (msg.h).
        assert_eq!(CmdId::Stand as u8, 0);
        assert_eq!(CmdId::WalkXY as u8, 1);
        assert_eq!(CmdId::AckPlrInfo as u8, 2);
        // Upstream removed CMD_ATTACKXY (old 11) and added CMD_PLRALIVE at 34,
        // shifting CMD_RATTACKXY..CMD_PLRDEAD down by one (msg.h @ 4b2e6c74).
        assert_eq!(CmdId::RAttackXY as u8, 11);
        assert_eq!(CmdId::Warp as u8, 26);
        assert_eq!(CmdId::PlrDead as u8, 33);
        assert_eq!(CmdId::PlrAlive as u8, 34);
        assert_eq!(CmdId::RequestGItem as u8, 35);
        assert_eq!(CmdId::OperateObj as u8, 41);
        // Renumbered C++ values (previously misaligned in Rust).
        assert_eq!(CmdId::BreakObj as u8, 42);
        assert_eq!(CmdId::DropItem as u8, 51);
        assert_eq!(CmdId::SendPlrInfo as u8, 53);
        assert_eq!(CmdId::ActivatePortal as u8, 55);
        assert_eq!(CmdId::String as u8, 61);
        assert_eq!(CmdId::SyncQuest as u8, 71);
        assert_eq!(CmdId::Invalid as u8, 80);
        // Rust-port-only commands live above the C++ range.
        assert_eq!(CmdId::Chat as u8, 87);
        assert_eq!(CmdId::Pong as u8, 100);
    }


}
