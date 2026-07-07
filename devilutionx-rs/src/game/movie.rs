//! 视频播放系统
//!
//! 移植自 Source/movie.cpp
//! 负责过场动画和视频播放

use std::time::{Duration, Instant};

/// 视频播放状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MovieState {
    /// 停止
    Stopped,
    /// 播放中
    Playing,
    /// 暂停
    Paused,
    /// 完成
    Finished,
}

/// 视频播放标志
#[derive(Debug, Clone, Copy)]
pub struct MovieFlags {
    /// 用户可以关闭
    pub user_can_close: bool,
    /// 循环播放
    pub loop_movie: bool,
    /// 是否是游戏内视频
    pub in_game: bool,
}

impl Default for MovieFlags {
    fn default() -> Self {
        Self {
            user_can_close: true,
            loop_movie: false,
            in_game: false,
        }
    }
}

/// 视频事件类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MovieEvent {
    /// 无事件
    None,
    /// 按键按下
    KeyDown,
    /// 鼠标按下
    MouseDown,
    /// 鼠标释放
    MouseUp,
    /// ESC键
    Escape,
    /// 窗口失去焦点
    FocusLost,
    /// 窗口获得焦点
    FocusGained,
    /// 退出请求
    Quit,
    /// 控制器按钮
    ControllerButton,
}

/// 视频信息
#[derive(Debug, Clone)]
pub struct MovieInfo {
    /// 文件路径
    pub path: String,
    /// 宽度
    pub width: u32,
    /// 高度
    pub height: u32,
    /// 帧率
    pub fps: f32,
    /// 总帧数
    pub total_frames: u32,
    /// 当前帧
    pub current_frame: u32,
    /// 持续时间
    pub duration: Duration,
}

impl MovieInfo {
    /// 创建新的视频信息
    pub fn new(path: &str) -> Self {
        Self {
            path: path.to_string(),
            width: 640,
            height: 480,
            fps: 15.0,
            total_frames: 0,
            current_frame: 0,
            duration: Duration::ZERO,
        }
    }

    /// 获取进度（0.0 - 1.0）
    pub fn progress(&self) -> f32 {
        if self.total_frames == 0 {
            return 0.0;
        }
        self.current_frame as f32 / self.total_frames as f32
    }

    /// 是否已完成
    ///
    /// A freshly-constructed `MovieInfo` has `total_frames == 0` (the real
    /// frame count is only known once the Smacker stream is opened). Treat
    /// that state as "not finished" rather than "0 >= 0 == finished", so a
    /// movie that hasn't started playing isn't reported as already done.
    pub fn is_finished(&self) -> bool {
        self.total_frames > 0 && self.current_frame >= self.total_frames
    }
}

/// 视频播放器
#[derive(Debug)]
pub struct MoviePlayer {
    /// 当前状态
    state: MovieState,
    /// 播放标志
    flags: MovieFlags,
    /// 当前视频信息
    current_movie: Option<MovieInfo>,
    /// 播放开始时间
    start_time: Option<Instant>,
    /// 暂停时的已播放时间
    paused_elapsed: Duration,
    /// 鼠标位置
    pub mouse_position: (i32, i32),
    /// 是否使用硬件光标
    pub hardware_cursor: bool,
    /// 是否按下焦点丢失暂停
    pub pause_on_focus_loss: bool,
}

impl Default for MoviePlayer {
    fn default() -> Self {
        Self::new()
    }
}

impl MoviePlayer {
    /// 创建新的视频播放器
    pub fn new() -> Self {
        Self {
            state: MovieState::Stopped,
            flags: MovieFlags::default(),
            current_movie: None,
            start_time: None,
            paused_elapsed: Duration::ZERO,
            mouse_position: (0, 0),
            hardware_cursor: false,
            pause_on_focus_loss: true,
        }
    }

    /// 开始播放视频
    pub fn play(&mut self, path: &str, flags: MovieFlags) -> bool {
        // 停止当前播放
        self.stop();

        // 创建视频信息
        let movie = MovieInfo::new(path);

        // 初始化播放状态
        self.current_movie = Some(movie);
        self.flags = flags;
        self.state = MovieState::Playing;
        self.start_time = Some(Instant::now());
        self.paused_elapsed = Duration::ZERO;

        // 隐藏硬件光标
        if self.hardware_cursor {
            // 实际实现会调用SDL隐藏光标
        }

        true
    }

    /// 播放游戏内视频
    pub fn play_in_game(&mut self, path: &str) {
        // 淡出调色板
        self.fade_out(8);

        // 播放视频（不允许用户关闭）
        let flags = MovieFlags {
            user_can_close: false,
            loop_movie: false,
            in_game: true,
        };
        self.play(path, flags);
    }

    /// 停止播放
    pub fn stop(&mut self) {
        if self.state != MovieState::Stopped {
            self.state = MovieState::Stopped;
            self.current_movie = None;
            self.start_time = None;
            self.paused_elapsed = Duration::ZERO;
        }
    }

    /// 暂停播放
    pub fn pause(&mut self) {
        if self.state == MovieState::Playing {
            if let Some(start) = self.start_time {
                self.paused_elapsed += start.elapsed();
            }
            self.state = MovieState::Paused;
            self.start_time = None;
        }
    }

    /// 恢复播放
    pub fn resume(&mut self) {
        if self.state == MovieState::Paused {
            self.state = MovieState::Playing;
            self.start_time = Some(Instant::now());
        }
    }

    /// 处理事件
    pub fn handle_event(&mut self, event: MovieEvent) -> bool {
        if self.state != MovieState::Playing && self.state != MovieState::Paused {
            return false;
        }

        match event {
            MovieEvent::KeyDown | MovieEvent::MouseUp => {
                if self.flags.user_can_close {
                    self.state = MovieState::Finished;
                    return true;
                }
            }
            MovieEvent::Escape => {
                // ESC总是可以关闭
                self.state = MovieState::Finished;
                return true;
            }
            MovieEvent::FocusLost => {
                if self.pause_on_focus_loss {
                    self.pause();
                }
            }
            MovieEvent::FocusGained => {
                if self.pause_on_focus_loss {
                    self.resume();
                }
            }
            MovieEvent::Quit => {
                self.stop();
                return true;
            }
            MovieEvent::ControllerButton => {
                if self.flags.user_can_close {
                    self.state = MovieState::Finished;
                    return true;
                }
            }
            _ => {}
        }

        false
    }

    /// 继续播放（每帧调用）
    pub fn continue_playback(&mut self) -> bool {
        if self.state != MovieState::Playing {
            return false;
        }

        let movie = match &mut self.current_movie {
            Some(m) => m,
            None => return false,
        };

        // 计算当前帧
        let elapsed = if let Some(start) = self.start_time {
            self.paused_elapsed + start.elapsed()
        } else {
            self.paused_elapsed
        };

        let frame = (elapsed.as_secs_f32() * movie.fps) as u32;
        movie.current_frame = frame;

        // 检查是否完成
        if movie.is_finished() {
            if self.flags.loop_movie {
                // 循环播放
                movie.current_frame = 0;
                self.start_time = Some(Instant::now());
                self.paused_elapsed = Duration::ZERO;
            } else {
                self.state = MovieState::Finished;
                return false;
            }
        }

        true
    }

    /// 获取当前状态
    pub fn state(&self) -> MovieState {
        self.state
    }

    /// 是否正在播放
    pub fn is_playing(&self) -> bool {
        self.state == MovieState::Playing
    }

    /// 获取当前视频信息
    pub fn current_movie(&self) -> Option<&MovieInfo> {
        self.current_movie.as_ref()
    }

    /// 获取当前进度
    pub fn progress(&self) -> f32 {
        self.current_movie.as_ref().map(|m| m.progress()).unwrap_or(0.0)
    }

    /// 淡出效果
    fn fade_out(&self, _steps: u32) {
        // 实际实现会调用调色板淡出
    }

    /// 淡入效果
    fn fade_in(&self, _steps: u32) {
        // 实际实现会调用调色板淡入
    }

    /// 完成游戏内视频播放后的清理
    pub fn finish_in_game_movie(&mut self) {
        self.stop();

        // 清除屏幕缓冲区
        // 重绘所有内容
        // 绘制游戏屏幕
        self.fade_in(8);
        // 再次重绘
    }

    /// 更新鼠标位置
    pub fn update_mouse_position(&mut self, x: i32, y: i32) {
        self.mouse_position = (x, y);
    }
}

/// 检查控制器按钮是否跳过视频
pub fn skips_movie(button: u32) -> bool {
    // 简化实现，实际需要检查特定按钮
    // A、B、Start等按钮通常跳过视频
    matches!(button, 0 | 1 | 7) // A, B, Start
}

/// Smacker视频解码器接口（简化）
pub mod smacker {
    use super::*;

    /// Smacker视频帧
    #[derive(Debug)]
    pub struct SmackerFrame {
        /// 帧数据
        pub data: Vec<u8>,
        /// 宽度
        pub width: u32,
        /// 高度
        pub height: u32,
        /// 帧索引
        pub index: u32,
    }

    /// Smacker视频
    #[derive(Debug)]
    pub struct SmackerVideo {
        /// 文件路径
        path: String,
        /// 是否已打开
        opened: bool,
        /// 视频信息
        info: MovieInfo,
        /// 是否循环
        looping: bool,
    }

    impl SmackerVideo {
        /// 打开视频文件
        pub fn open(path: &str, flags: u32) -> Option<Self> {
            let looping = (flags & 0x100) != 0;

            Some(Self {
                path: path.to_string(),
                opened: true,
                info: MovieInfo::new(path),
                looping,
            })
        }

        /// 开始播放
        pub fn play_begin(&mut self) -> bool {
            if !self.opened {
                return false;
            }
            self.info.current_frame = 0;
            true
        }

        /// 继续播放
        pub fn play_continue(&mut self) -> bool {
            if !self.opened {
                return false;
            }

            self.info.current_frame += 1;

            if self.info.is_finished() {
                if self.looping {
                    self.info.current_frame = 0;
                    return true;
                }
                return false;
            }

            true
        }

        /// 结束播放
        pub fn play_end(&mut self) {
            self.info.current_frame = 0;
        }

        /// 获取当前帧
        pub fn get_frame(&self) -> Option<SmackerFrame> {
            if !self.opened {
                return None;
            }

            Some(SmackerFrame {
                data: vec![0; (self.info.width * self.info.height) as usize],
                width: self.info.width,
                height: self.info.height,
                index: self.info.current_frame,
            })
        }

        /// 关闭视频
        pub fn close(&mut self) {
            self.opened = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_movie_state() {
        assert_ne!(MovieState::Playing, MovieState::Stopped);
        assert_ne!(MovieState::Paused, MovieState::Finished);
    }

    #[test]
    fn test_movie_flags_default() {
        let flags = MovieFlags::default();
        assert!(flags.user_can_close);
        assert!(!flags.loop_movie);
        assert!(!flags.in_game);
    }

    #[test]
    fn test_movie_info_new() {
        let info = MovieInfo::new("test.smk");
        assert_eq!(info.path, "test.smk");
        assert_eq!(info.current_frame, 0);
        assert!(!info.is_finished());
    }

    #[test]
    fn test_movie_info_progress() {
        let mut info = MovieInfo::new("test.smk");
        info.total_frames = 100;
        info.current_frame = 50;
        assert!((info.progress() - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_movie_info_finished() {
        let mut info = MovieInfo::new("test.smk");
        info.total_frames = 100;
        info.current_frame = 100;
        assert!(info.is_finished());
    }

    #[test]
    fn test_movie_player_new() {
        let player = MoviePlayer::new();
        assert_eq!(player.state(), MovieState::Stopped);
        assert!(player.current_movie().is_none());
    }

    #[test]
    fn test_movie_player_play() {
        let mut player = MoviePlayer::new();
        let flags = MovieFlags::default();

        assert!(player.play("test.smk", flags));
        assert_eq!(player.state(), MovieState::Playing);
        assert!(player.is_playing());
    }

    #[test]
    fn test_movie_player_stop() {
        let mut player = MoviePlayer::new();
        player.play("test.smk", MovieFlags::default());

        player.stop();
        assert_eq!(player.state(), MovieState::Stopped);
        assert!(player.current_movie().is_none());
    }

    #[test]
    fn test_movie_player_pause_resume() {
        let mut player = MoviePlayer::new();
        player.play("test.smk", MovieFlags::default());

        player.pause();
        assert_eq!(player.state(), MovieState::Paused);

        player.resume();
        assert_eq!(player.state(), MovieState::Playing);
    }

    #[test]
    fn test_movie_player_handle_escape() {
        let mut player = MoviePlayer::new();
        player.play("test.smk", MovieFlags::default());

        assert!(player.handle_event(MovieEvent::Escape));
        assert_eq!(player.state(), MovieState::Finished);
    }

    #[test]
    fn test_movie_player_handle_focus() {
        let mut player = MoviePlayer::new();
        player.play("test.smk", MovieFlags::default());

        player.handle_event(MovieEvent::FocusLost);
        assert_eq!(player.state(), MovieState::Paused);

        player.handle_event(MovieEvent::FocusGained);
        assert_eq!(player.state(), MovieState::Playing);
    }

    #[test]
    fn test_skips_movie() {
        assert!(skips_movie(0)); // A button
        assert!(skips_movie(1)); // B button
        assert!(skips_movie(7)); // Start button
        assert!(!skips_movie(100)); // Unknown
    }

    #[test]
    fn test_smacker_video_open() {
        let video = smacker::SmackerVideo::open("test.smk", 0);
        assert!(video.is_some());
    }

    #[test]
    fn test_smacker_video_play() {
        let mut video = smacker::SmackerVideo::open("test.smk", 0).unwrap();
        assert!(video.play_begin());
        assert!(video.play_continue());
    }
}
