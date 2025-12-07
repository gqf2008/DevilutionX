//! Audio System for DevilutionX-RS
//!
//! Handles sound effects and music playback.
//! References: Source/effects.cpp, Source/sound.cpp

use std::collections::HashMap;

/// Sound effect categories (from effects.h)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SoundCategory {
    /// Player sounds
    Player,
    /// Monster sounds
    Monster,
    /// Item sounds
    Item,
    /// Environment/ambient sounds
    Environment,
    /// UI sounds
    UI,
    /// Spell/magic sounds
    Spell,
}

/// Sound effect IDs (based on effects.h sfx_id enum)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SoundEffect {
    // Player sounds
    PlayerHit,
    PlayerDeath,
    PlayerLevelUp,
    PlayerBlock,
    PlayerDrinkPotion,
    PlayerCast,

    // Footsteps
    FootstepStone,
    FootstepWood,
    FootstepDirt,
    FootstepWater,

    // Weapon sounds
    SwingSword,
    SwingAxe,
    SwingBlunt,
    SwingBow,
    ArrowHit,
    ArrowMiss,

    // Monster sounds
    MonsterHit,
    MonsterDeath,
    MonsterAttack,
    MonsterAlert,
    MonsterIdle,

    // Specific monster sounds
    SkeletonHit,
    SkeletonDeath,
    ZombieGroan,
    ZombieDeath,
    ScavengerHit,
    ScavengerDeath,
    FallenHit,
    FallenDeath,
    GargoyleWing,
    GolemStep,
    SuccubusLaugh,
    DiabloRoar,

    // Item sounds
    ItemPickup,
    ItemDrop,
    ItemEquip,
    ItemUnequip,
    GoldPickup,
    GoldDrop,
    PotionUse,
    ScrollUse,
    BookRead,

    // Door/Chest
    DoorOpen,
    DoorClose,
    ChestOpen,
    ChestLocked,
    LeverPull,

    // Environment
    TorchBurning,
    FireCrackle,
    WaterDrip,
    WindHowl,
    ThunderRumble,
    Earthquake,

    // Spell sounds
    SpellFire,
    SpellLightning,
    SpellHoly,
    SpellHeal,
    SpellTeleport,
    SpellStonecurse,
    SpellApocalypse,
    SpellNova,
    SpellGolem,
    SpellBloodStar,
    SpellBoneSpirit,

    // UI sounds
    UIClick,
    UIOpen,
    UIClose,
    UIError,
    UIQuest,
    UIMessage,

    // Misc
    Shrine,
    Fountain,
    Cauldron,
    Portal,
    StairsDown,
    StairsUp,
}

impl SoundEffect {
    /// Get the category of this sound effect
    pub fn category(&self) -> SoundCategory {
        use SoundEffect::*;
        match self {
            PlayerHit | PlayerDeath | PlayerLevelUp | PlayerBlock |
            PlayerDrinkPotion | PlayerCast |
            FootstepStone | FootstepWood | FootstepDirt | FootstepWater => SoundCategory::Player,

            SwingSword | SwingAxe | SwingBlunt | SwingBow | ArrowHit | ArrowMiss => SoundCategory::Player,

            MonsterHit | MonsterDeath | MonsterAttack | MonsterAlert | MonsterIdle |
            SkeletonHit | SkeletonDeath | ZombieGroan | ZombieDeath |
            ScavengerHit | ScavengerDeath | FallenHit | FallenDeath |
            GargoyleWing | GolemStep | SuccubusLaugh | DiabloRoar => SoundCategory::Monster,

            ItemPickup | ItemDrop | ItemEquip | ItemUnequip |
            GoldPickup | GoldDrop | PotionUse | ScrollUse | BookRead => SoundCategory::Item,

            DoorOpen | DoorClose | ChestOpen | ChestLocked | LeverPull |
            TorchBurning | FireCrackle | WaterDrip | WindHowl |
            ThunderRumble | Earthquake => SoundCategory::Environment,

            SpellFire | SpellLightning | SpellHoly | SpellHeal | SpellTeleport |
            SpellStonecurse | SpellApocalypse | SpellNova | SpellGolem |
            SpellBloodStar | SpellBoneSpirit => SoundCategory::Spell,

            UIClick | UIOpen | UIClose | UIError | UIQuest | UIMessage => SoundCategory::UI,

            Shrine | Fountain | Cauldron | Portal | StairsDown | StairsUp => SoundCategory::Environment,
        }
    }

    /// Get the default file name for this sound
    pub fn default_filename(&self) -> &'static str {
        use SoundEffect::*;
        match self {
            PlayerHit => "sfx/player/hit.wav",
            PlayerDeath => "sfx/player/death.wav",
            PlayerLevelUp => "sfx/player/levelup.wav",
            PlayerBlock => "sfx/player/block.wav",
            PlayerDrinkPotion => "sfx/player/drink.wav",
            PlayerCast => "sfx/player/cast.wav",

            FootstepStone => "sfx/player/step_stone.wav",
            FootstepWood => "sfx/player/step_wood.wav",
            FootstepDirt => "sfx/player/step_dirt.wav",
            FootstepWater => "sfx/player/step_water.wav",

            SwingSword => "sfx/weapon/sword.wav",
            SwingAxe => "sfx/weapon/axe.wav",
            SwingBlunt => "sfx/weapon/blunt.wav",
            SwingBow => "sfx/weapon/bow.wav",
            ArrowHit => "sfx/weapon/arrow_hit.wav",
            ArrowMiss => "sfx/weapon/arrow_miss.wav",

            MonsterHit => "sfx/monster/hit.wav",
            MonsterDeath => "sfx/monster/death.wav",
            MonsterAttack => "sfx/monster/attack.wav",
            MonsterAlert => "sfx/monster/alert.wav",
            MonsterIdle => "sfx/monster/idle.wav",

            SkeletonHit => "sfx/monster/skeleton_hit.wav",
            SkeletonDeath => "sfx/monster/skeleton_death.wav",
            ZombieGroan => "sfx/monster/zombie_groan.wav",
            ZombieDeath => "sfx/monster/zombie_death.wav",
            ScavengerHit => "sfx/monster/scavenger_hit.wav",
            ScavengerDeath => "sfx/monster/scavenger_death.wav",
            FallenHit => "sfx/monster/fallen_hit.wav",
            FallenDeath => "sfx/monster/fallen_death.wav",
            GargoyleWing => "sfx/monster/gargoyle_wing.wav",
            GolemStep => "sfx/monster/golem_step.wav",
            SuccubusLaugh => "sfx/monster/succubus_laugh.wav",
            DiabloRoar => "sfx/monster/diablo_roar.wav",

            ItemPickup => "sfx/item/pickup.wav",
            ItemDrop => "sfx/item/drop.wav",
            ItemEquip => "sfx/item/equip.wav",
            ItemUnequip => "sfx/item/unequip.wav",
            GoldPickup => "sfx/item/gold_pickup.wav",
            GoldDrop => "sfx/item/gold_drop.wav",
            PotionUse => "sfx/item/potion.wav",
            ScrollUse => "sfx/item/scroll.wav",
            BookRead => "sfx/item/book.wav",

            DoorOpen => "sfx/env/door_open.wav",
            DoorClose => "sfx/env/door_close.wav",
            ChestOpen => "sfx/env/chest_open.wav",
            ChestLocked => "sfx/env/chest_locked.wav",
            LeverPull => "sfx/env/lever.wav",

            TorchBurning => "sfx/env/torch.wav",
            FireCrackle => "sfx/env/fire.wav",
            WaterDrip => "sfx/env/water_drip.wav",
            WindHowl => "sfx/env/wind.wav",
            ThunderRumble => "sfx/env/thunder.wav",
            Earthquake => "sfx/env/earthquake.wav",

            SpellFire => "sfx/spell/fire.wav",
            SpellLightning => "sfx/spell/lightning.wav",
            SpellHoly => "sfx/spell/holy.wav",
            SpellHeal => "sfx/spell/heal.wav",
            SpellTeleport => "sfx/spell/teleport.wav",
            SpellStonecurse => "sfx/spell/stonecurse.wav",
            SpellApocalypse => "sfx/spell/apocalypse.wav",
            SpellNova => "sfx/spell/nova.wav",
            SpellGolem => "sfx/spell/golem.wav",
            SpellBloodStar => "sfx/spell/bloodstar.wav",
            SpellBoneSpirit => "sfx/spell/bonespirit.wav",

            UIClick => "sfx/ui/click.wav",
            UIOpen => "sfx/ui/open.wav",
            UIClose => "sfx/ui/close.wav",
            UIError => "sfx/ui/error.wav",
            UIQuest => "sfx/ui/quest.wav",
            UIMessage => "sfx/ui/message.wav",

            Shrine => "sfx/env/shrine.wav",
            Fountain => "sfx/env/fountain.wav",
            Cauldron => "sfx/env/cauldron.wav",
            Portal => "sfx/env/portal.wav",
            StairsDown => "sfx/env/stairs_down.wav",
            StairsUp => "sfx/env/stairs_up.wav",
        }
    }

    /// Get base volume for this sound (0.0 - 1.0)
    pub fn base_volume(&self) -> f32 {
        use SoundEffect::*;
        match self {
            // Louder sounds
            PlayerDeath | DiabloRoar | ThunderRumble | Earthquake => 1.0,
            PlayerLevelUp | SpellApocalypse => 0.9,

            // Normal volume
            PlayerHit | MonsterHit | MonsterDeath | SwingSword | SwingAxe => 0.7,

            // Quieter sounds
            FootstepStone | FootstepWood | FootstepDirt | FootstepWater => 0.4,
            MonsterIdle | TorchBurning | WaterDrip => 0.3,

            // UI sounds
            UIClick => 0.5,

            _ => 0.6,
        }
    }

    /// Whether this sound should loop
    pub fn is_looping(&self) -> bool {
        use SoundEffect::*;
        matches!(self, TorchBurning | FireCrackle | WaterDrip | WindHowl | Fountain)
    }
}

/// Music tracks (based on music IDs in the game)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MusicTrack {
    /// Title screen
    Title,
    /// Town (Tristram)
    Town,
    /// Cathedral levels 1-4
    Cathedral,
    /// Catacombs levels 5-8
    Catacombs,
    /// Caves levels 9-12
    Caves,
    /// Hell levels 13-16
    Hell,
    /// Boss fight
    Boss,
    /// Victory
    Victory,
    /// Defeat
    Defeat,
    /// Intro cinematic
    Intro,
    /// Credits
    Credits,
}

impl MusicTrack {
    /// Get the default file name for this music track
    pub fn default_filename(&self) -> &'static str {
        match self {
            MusicTrack::Title => "music/title.mp3",
            MusicTrack::Town => "music/town.mp3",
            MusicTrack::Cathedral => "music/cathedral.mp3",
            MusicTrack::Catacombs => "music/catacombs.mp3",
            MusicTrack::Caves => "music/caves.mp3",
            MusicTrack::Hell => "music/hell.mp3",
            MusicTrack::Boss => "music/boss.mp3",
            MusicTrack::Victory => "music/victory.mp3",
            MusicTrack::Defeat => "music/defeat.mp3",
            MusicTrack::Intro => "music/intro.mp3",
            MusicTrack::Credits => "music/credits.mp3",
        }
    }

    /// Get music track for dungeon level
    pub fn for_dungeon_level(level: u8) -> Self {
        match level {
            0 => MusicTrack::Town,
            1..=4 => MusicTrack::Cathedral,
            5..=8 => MusicTrack::Catacombs,
            9..=12 => MusicTrack::Caves,
            13..=16 => MusicTrack::Hell,
            _ => MusicTrack::Hell,
        }
    }
}

/// A sound instance that can be played
#[derive(Debug, Clone)]
pub struct Sound {
    pub effect: SoundEffect,
    pub volume: f32,
    pub pitch: f32,
    pub pan: f32,  // -1.0 (left) to 1.0 (right)
    pub looping: bool,
}

impl Sound {
    pub fn new(effect: SoundEffect) -> Self {
        Self {
            effect,
            volume: effect.base_volume(),
            pitch: 1.0,
            pan: 0.0,
            looping: effect.is_looping(),
        }
    }

    pub fn with_volume(mut self, volume: f32) -> Self {
        self.volume = volume.clamp(0.0, 1.0);
        self
    }

    pub fn with_pitch(mut self, pitch: f32) -> Self {
        self.pitch = pitch.clamp(0.5, 2.0);
        self
    }

    pub fn with_pan(mut self, pan: f32) -> Self {
        self.pan = pan.clamp(-1.0, 1.0);
        self
    }

    pub fn looping(mut self, looping: bool) -> Self {
        self.looping = looping;
        self
    }
}

/// 3D positioned sound
#[derive(Debug, Clone)]
pub struct PositionalSound {
    pub sound: Sound,
    pub x: i32,
    pub y: i32,
    pub max_distance: f32,
}

impl PositionalSound {
    pub fn new(effect: SoundEffect, x: i32, y: i32) -> Self {
        Self {
            sound: Sound::new(effect),
            x,
            y,
            max_distance: 15.0,  // Tiles
        }
    }

    /// Calculate volume and pan based on listener position
    pub fn calculate_spatial(&self, listener_x: i32, listener_y: i32) -> (f32, f32) {
        let dx = (self.x - listener_x) as f32;
        let dy = (self.y - listener_y) as f32;
        let distance = (dx * dx + dy * dy).sqrt();

        // Volume falloff
        let volume = if distance >= self.max_distance {
            0.0
        } else {
            self.sound.volume * (1.0 - distance / self.max_distance)
        };

        // Pan based on x difference
        let pan = if distance > 0.0 {
            (dx / distance).clamp(-1.0, 1.0)
        } else {
            0.0
        };

        (volume, pan)
    }
}

/// Handle for a playing sound
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SoundHandle(pub u32);

/// Audio system configuration
#[derive(Debug, Clone)]
pub struct AudioConfig {
    pub master_volume: f32,
    pub music_volume: f32,
    pub sfx_volume: f32,
    pub ambient_volume: f32,
    pub muted: bool,
    pub music_enabled: bool,
    pub sfx_enabled: bool,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            master_volume: 1.0,
            music_volume: 0.7,
            sfx_volume: 0.8,
            ambient_volume: 0.5,
            muted: false,
            music_enabled: true,
            sfx_enabled: true,
        }
    }
}

/// Audio manager - handles all sound playback
/// Note: This is a stub implementation. Real audio would use SDL2_mixer or similar.
pub struct AudioManager {
    config: AudioConfig,
    current_music: Option<MusicTrack>,
    next_handle: u32,
    active_sounds: HashMap<SoundHandle, PlayingSound>,
    sound_queue: Vec<Sound>,
    positional_queue: Vec<(PositionalSound, i32, i32)>,  // (sound, listener_x, listener_y)
}

#[derive(Debug)]
struct PlayingSound {
    effect: SoundEffect,
    volume: f32,
    remaining_ms: u32,
    looping: bool,
}

impl Default for AudioManager {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioManager {
    pub fn new() -> Self {
        Self {
            config: AudioConfig::default(),
            current_music: None,
            next_handle: 1,
            active_sounds: HashMap::new(),
            sound_queue: Vec::new(),
            positional_queue: Vec::new(),
        }
    }

    /// Initialize audio system
    pub fn init(&mut self) -> Result<(), String> {
        // In a real implementation, this would initialize SDL2_mixer
        // sdl2::mixer::open_audio(44100, AUDIO_S16LSB, 2, 1024)?;
        // sdl2::mixer::allocate_channels(16);
        Ok(())
    }

    /// Shutdown audio system
    pub fn shutdown(&mut self) {
        self.stop_all();
        // sdl2::mixer::close_audio();
    }

    /// Update audio system (call each frame)
    pub fn update(&mut self, delta_ms: u32) {
        // Collect sounds to play
        let queued: Vec<_> = self.sound_queue.drain(..).collect();
        let positional: Vec<_> = self.positional_queue.drain(..).collect();

        // Process sound queue
        for sound in queued {
            self.play_internal(sound);
        }

        // Process positional sounds
        for (pos_sound, lx, ly) in positional {
            let (volume, pan) = pos_sound.calculate_spatial(lx, ly);
            if volume > 0.01 {
                let sound = pos_sound.sound.with_volume(volume).with_pan(pan);
                self.play_internal(sound);
            }
        }

        // Update active sounds
        let mut finished = Vec::new();
        for (handle, playing) in &mut self.active_sounds {
            if !playing.looping {
                if playing.remaining_ms <= delta_ms {
                    finished.push(*handle);
                } else {
                    playing.remaining_ms -= delta_ms;
                }
            }
        }

        for handle in finished {
            self.active_sounds.remove(&handle);
        }
    }

    /// Play a sound effect
    pub fn play(&mut self, effect: SoundEffect) -> SoundHandle {
        self.play_sound(Sound::new(effect))
    }

    /// Play a configured sound
    pub fn play_sound(&mut self, sound: Sound) -> SoundHandle {
        self.sound_queue.push(sound);
        let handle = SoundHandle(self.next_handle);
        self.next_handle += 1;
        handle
    }

    /// Play a positional sound
    pub fn play_at(&mut self, effect: SoundEffect, x: i32, y: i32, listener_x: i32, listener_y: i32) -> SoundHandle {
        let pos_sound = PositionalSound::new(effect, x, y);
        self.positional_queue.push((pos_sound, listener_x, listener_y));
        let handle = SoundHandle(self.next_handle);
        self.next_handle += 1;
        handle
    }

    fn play_internal(&mut self, sound: Sound) -> SoundHandle {
        if self.config.muted || !self.config.sfx_enabled {
            return SoundHandle(0);
        }

        let final_volume = sound.volume * self.config.sfx_volume * self.config.master_volume;

        // In a real implementation:
        // let chunk = self.load_sound(sound.effect)?;
        // let channel = sdl2::mixer::Channel::all().play(&chunk, if sound.looping { -1 } else { 0 })?;
        // channel.set_volume((final_volume * 128.0) as i32);

        // Stub: just track the sound
        let handle = SoundHandle(self.next_handle);
        self.next_handle += 1;

        let playing = PlayingSound {
            effect: sound.effect,
            volume: final_volume,
            remaining_ms: 500,  // Assume 500ms duration
            looping: sound.looping,
        };

        self.active_sounds.insert(handle, playing);
        handle
    }

    /// Stop a specific sound
    pub fn stop(&mut self, handle: SoundHandle) {
        self.active_sounds.remove(&handle);
    }

    /// Stop all sounds
    pub fn stop_all(&mut self) {
        self.active_sounds.clear();
        // sdl2::mixer::Channel::all().halt();
    }

    /// Stop all sounds of a category
    pub fn stop_category(&mut self, category: SoundCategory) {
        self.active_sounds.retain(|_, playing| {
            playing.effect.category() != category
        });
    }

    /// Play music track
    pub fn play_music(&mut self, track: MusicTrack) {
        if !self.config.music_enabled || self.config.muted {
            self.current_music = Some(track);
            return;
        }

        // In a real implementation:
        // let music = sdl2::mixer::Music::from_file(track.default_filename())?;
        // music.play(-1)?;
        // sdl2::mixer::Music::set_volume((self.config.music_volume * self.config.master_volume * 128.0) as i32);

        self.current_music = Some(track);
    }

    /// Stop music
    pub fn stop_music(&mut self) {
        // sdl2::mixer::Music::halt();
        self.current_music = None;
    }

    /// Pause music
    pub fn pause_music(&mut self) {
        // sdl2::mixer::Music::pause();
    }

    /// Resume music
    pub fn resume_music(&mut self) {
        // sdl2::mixer::Music::resume();
    }

    /// Fade out current music
    pub fn fade_out_music(&mut self, _duration_ms: u32) {
        // sdl2::mixer::Music::fade_out(duration_ms as i32);
    }

    /// Crossfade to new music
    pub fn crossfade_music(&mut self, track: MusicTrack, _duration_ms: u32) {
        // Fade out current, then play new
        self.play_music(track);
    }

    /// Set master volume
    pub fn set_master_volume(&mut self, volume: f32) {
        self.config.master_volume = volume.clamp(0.0, 1.0);
    }

    /// Set music volume
    pub fn set_music_volume(&mut self, volume: f32) {
        self.config.music_volume = volume.clamp(0.0, 1.0);
        // Update playing music volume
        // sdl2::mixer::Music::set_volume((self.config.music_volume * self.config.master_volume * 128.0) as i32);
    }

    /// Set SFX volume
    pub fn set_sfx_volume(&mut self, volume: f32) {
        self.config.sfx_volume = volume.clamp(0.0, 1.0);
    }

    /// Get music volume
    pub fn music_volume(&self) -> f32 {
        self.config.music_volume
    }

    /// Get SFX volume
    pub fn sfx_volume(&self) -> f32 {
        self.config.sfx_volume
    }

    /// Toggle mute
    pub fn toggle_mute(&mut self) {
        self.config.muted = !self.config.muted;
        if self.config.muted {
            self.stop_all();
            self.stop_music();
        } else if let Some(track) = self.current_music {
            self.play_music(track);
        }
    }

    /// Check if muted
    pub fn is_muted(&self) -> bool {
        self.config.muted
    }

    /// Get current music track
    pub fn current_music(&self) -> Option<MusicTrack> {
        self.current_music
    }

    /// Get config
    pub fn config(&self) -> &AudioConfig {
        &self.config
    }

    /// Get mutable config
    pub fn config_mut(&mut self) -> &mut AudioConfig {
        &mut self.config
    }
}

/// Ambient sound manager for environment
pub struct AmbientManager {
    sounds: Vec<(SoundEffect, f32)>,  // (effect, interval_seconds)
    timers: Vec<f32>,
    enabled: bool,
}

impl Default for AmbientManager {
    fn default() -> Self {
        Self::new()
    }
}

impl AmbientManager {
    pub fn new() -> Self {
        Self {
            sounds: Vec::new(),
            timers: Vec::new(),
            enabled: true,
        }
    }

    /// Set ambient sounds for dungeon type
    pub fn set_dungeon_ambient(&mut self, level: u8) {
        self.sounds.clear();
        self.timers.clear();

        match level {
            0 => {
                // Town - birds, wind
                self.add_ambient(SoundEffect::WindHowl, 15.0);
            }
            1..=4 => {
                // Cathedral - drips, creaks
                self.add_ambient(SoundEffect::WaterDrip, 8.0);
                self.add_ambient(SoundEffect::TorchBurning, 5.0);
            }
            5..=8 => {
                // Catacombs - echoes, bones
                self.add_ambient(SoundEffect::WaterDrip, 10.0);
            }
            9..=12 => {
                // Caves - water, wind
                self.add_ambient(SoundEffect::WaterDrip, 6.0);
                self.add_ambient(SoundEffect::WindHowl, 12.0);
            }
            13..=16 => {
                // Hell - fire, screams
                self.add_ambient(SoundEffect::FireCrackle, 4.0);
            }
            _ => {}
        }
    }

    fn add_ambient(&mut self, effect: SoundEffect, interval: f32) {
        self.sounds.push((effect, interval));
        self.timers.push(interval * 0.5);  // Start halfway through
    }

    /// Update ambient sounds, returns sounds to play
    pub fn update(&mut self, dt: f32) -> Vec<SoundEffect> {
        if !self.enabled {
            return Vec::new();
        }

        let mut to_play = Vec::new();

        for (i, timer) in self.timers.iter_mut().enumerate() {
            *timer -= dt;
            if *timer <= 0.0 {
                let (effect, interval) = self.sounds[i];
                to_play.push(effect);
                *timer = interval + rand::random::<f32>() * interval * 0.5;
            }
        }

        to_play
    }

    /// Enable/disable ambient sounds
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}

/// Footstep sound manager
pub struct FootstepManager {
    step_timer: f32,
    step_interval: f32,
}

impl Default for FootstepManager {
    fn default() -> Self {
        Self::new()
    }
}

impl FootstepManager {
    pub fn new() -> Self {
        Self {
            step_timer: 0.0,
            step_interval: 0.3,  // 300ms between steps
        }
    }

    /// Update footsteps, returns sound to play if step occurred
    pub fn update(&mut self, dt: f32, is_moving: bool, tile_type: u8) -> Option<SoundEffect> {
        if !is_moving {
            self.step_timer = 0.0;
            return None;
        }

        self.step_timer += dt;
        if self.step_timer >= self.step_interval {
            self.step_timer = 0.0;

            // Return footstep sound based on tile type
            Some(match tile_type {
                0 => SoundEffect::FootstepStone,  // Floor
                1 => SoundEffect::FootstepWood,   // Wood
                2 => SoundEffect::FootstepDirt,   // Dirt
                3 => SoundEffect::FootstepWater,  // Water
                _ => SoundEffect::FootstepStone,
            })
        } else {
            None
        }
    }
}

// Keep old API for compatibility
pub struct AudioSystem {
    manager: AudioManager,
}

impl AudioSystem {
    pub fn new() -> anyhow::Result<Self> {
        let mut manager = AudioManager::new();
        manager.init().map_err(|e| anyhow::anyhow!(e))?;
        Ok(Self { manager })
    }

    pub fn play_music(&mut self, track: &str) {
        // Map string to track
        let track = match track {
            "title" => MusicTrack::Title,
            "town" => MusicTrack::Town,
            "cathedral" => MusicTrack::Cathedral,
            "catacombs" => MusicTrack::Catacombs,
            "caves" => MusicTrack::Caves,
            "hell" => MusicTrack::Hell,
            _ => return,
        };
        self.manager.play_music(track);
    }

    pub fn play_sound(&mut self, sound: &str) {
        // Map string to effect
        let effect = match sound {
            "hit" => SoundEffect::PlayerHit,
            "death" => SoundEffect::PlayerDeath,
            "pickup" => SoundEffect::ItemPickup,
            "gold" => SoundEffect::GoldPickup,
            _ => return,
        };
        self.manager.play(effect);
    }

    /// Get the underlying audio manager
    pub fn manager(&mut self) -> &mut AudioManager {
        &mut self.manager
    }
}

impl Default for AudioSystem {
    fn default() -> Self {
        Self::new().expect("Failed to create audio system")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sound_effect_category() {
        assert_eq!(SoundEffect::PlayerHit.category(), SoundCategory::Player);
        assert_eq!(SoundEffect::MonsterDeath.category(), SoundCategory::Monster);
        assert_eq!(SoundEffect::ItemPickup.category(), SoundCategory::Item);
    }

    #[test]
    fn test_positional_sound() {
        let pos_sound = PositionalSound::new(SoundEffect::MonsterAttack, 10, 10);

        // Close listener - full volume
        let (vol, pan) = pos_sound.calculate_spatial(10, 10);
        assert!(vol > 0.5);
        assert!(pan.abs() < 0.1);

        // Far listener - low volume
        let (vol, _) = pos_sound.calculate_spatial(30, 30);
        assert!(vol < 0.1);
    }

    #[test]
    fn test_music_track_for_level() {
        assert_eq!(MusicTrack::for_dungeon_level(0), MusicTrack::Town);
        assert_eq!(MusicTrack::for_dungeon_level(3), MusicTrack::Cathedral);
        assert_eq!(MusicTrack::for_dungeon_level(7), MusicTrack::Catacombs);
        assert_eq!(MusicTrack::for_dungeon_level(10), MusicTrack::Caves);
        assert_eq!(MusicTrack::for_dungeon_level(15), MusicTrack::Hell);
    }
}
