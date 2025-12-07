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
    /// 玩家站立
    Stand = 0,
    /// 移动到位置
    WalkXY = 1,
    /// 确认接收玩家信息
    AckPlrInfo = 2,
    /// 增加力量
    AddStr = 3,
    /// 增加魔法
    AddMag = 4,
    /// 增加敏捷
    AddDex = 5,
    /// 增加体力
    AddVit = 6,
    /// 拾取物品到手
    GetItem = 7,
    /// 拾取物品到背包
    AGetItem = 8,
    /// 放下物品
    PutItem = 9,
    /// 生成物品
    SpawnItem = 10,
    /// 攻击位置
    AttackXY = 11,
    /// 远程攻击位置
    RAttackXY = 12,
    /// 施法到位置
    SpellXY = 13,
    /// 操作对象
    OpObjXY = 14,
    /// 解除陷阱
    DisarmXY = 15,
    /// 攻击怪物
    AttackId = 16,
    /// 攻击玩家
    AttackPid = 17,
    /// 远程攻击怪物
    RAttackId = 18,
    /// 远程攻击玩家
    RAttackPid = 19,
    /// 施法到怪物
    SpellId = 20,
    /// 施法到玩家
    SpellPid = 21,
    /// 复活玩家
    Resurrect = 22,
    /// 心灵传动操作对象
    OpObjT = 23,
    /// 击退怪物
    Knockback = 24,
    /// 与NPC交谈
    TalkXY = 25,
    /// 进入新关卡
    NewLvl = 26,
    /// 进入传送门
    Warp = 27,
    /// 作弊：获取经验
    CheatExperience = 28,
    /// 改变法术等级
    ChangeSpellLevel = 29,
    /// 调试命令
    Debug = 30,
    /// 同步数据
    SyncData = 31,
    /// 怪物死亡
    MonstDeath = 32,
    /// 怪物受伤
    MonstDamage = 33,
    /// 玩家死亡
    PlrDead = 34,
    /// 请求拾取物品
    RequestGItem = 35,
    /// 请求拾取物品到背包
    RequestAGItem = 36,
    /// 前往拾取物品
    GotoGetItem = 37,
    /// 前往拾取物品到背包
    GotoAGetItem = 38,
    /// 开门
    OpenDoor = 39,
    /// 关门
    CloseDoor = 40,
    /// 操作对象
    OperateObj = 41,
    /// 玩家操作
    PlrOp = 42,
    /// 断开连接
    Disconnect = 43,
    /// 发送玩家信息
    SendPlrInfo = 44,
    /// 设置玩家属性
    SetPlrAttr = 45,
    /// 开始游戏
    StartGame = 46,
    /// 加入游戏
    JoinGame = 47,
    /// 离开游戏
    LeaveGame = 48,
    /// 聊天消息
    Chat = 49,
    /// 暂停游戏
    Pause = 50,
    /// 同步请求
    SyncReq = 51,
    /// 丢弃物品
    DropItem = 52,
    /// 使用物品
    UseItem = 53,
    /// 装备物品
    EquipItem = 54,
    /// 卸下物品
    UnequipItem = 55,
    /// 购买物品
    BuyItem = 56,
    /// 出售物品
    SellItem = 57,
    /// 修理物品
    RepairItem = 58,
    /// 鉴定物品
    IdentifyItem = 59,
    /// 充能物品
    RechargeItem = 60,
    /// 设置任务状态
    SetQuest = 61,
    /// 同步任务
    SyncQuest = 62,
    /// 激活传送门
    ActivatePortal = 63,
    /// 关闭传送门
    DeactivatePortal = 64,
    /// Ping
    Ping = 65,
    /// Pong
    Pong = 66,
}

impl CmdId {
    /// 从 u8 转换
    pub fn from_u8(value: u8) -> Option<Self> {
        if value <= 66 {
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
            CmdId::AttackXY
                | CmdId::RAttackXY
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

    /// 发送攻击位置命令
    pub fn send_attack_xy(&mut self, x: i8, y: i8) -> bool {
        let cmd = TCmdLoc::new(CmdId::AttackXY, x, y);
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
            CmdId::Stand | CmdId::CheatExperience | CmdId::Debug => 1,
            CmdId::WalkXY | CmdId::AttackXY | CmdId::RAttackXY | CmdId::OpObjXY
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
        assert!(!CmdId::AttackXY.is_movement());
    }

    #[test]
    fn test_cmd_is_attack() {
        assert!(CmdId::AttackXY.is_attack());
        assert!(CmdId::AttackId.is_attack());
        assert!(!CmdId::WalkXY.is_attack());
    }

    #[test]
    fn test_cmd_is_spell() {
        assert!(CmdId::SpellXY.is_spell());
        assert!(CmdId::SpellId.is_spell());
        assert!(!CmdId::AttackXY.is_spell());
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
}
