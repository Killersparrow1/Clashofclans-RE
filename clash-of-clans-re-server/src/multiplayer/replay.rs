//! Replay System
//! 
//! Handles battle recording, playback, and sharing.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;
use bincode;
use crate::logic::game_data::*;
use crate::logic::{Battle, BattleState, DeployedTroop, Position, DeployedSpell, DeployedHero, Target, SpellState, HeroState, TroopState};
use crate::logic::buildings::GridPos;

/// Replay metadata
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ReplayMetadata {
    pub replay_id: String,
    pub version: String,
    pub game_version: String,
    pub timestamp: i64,
    pub attacker_id: i64,
    pub defender_id: i64,
    pub attacker_name: String,
    pub defender_name: String,
    pub attacker_trophies: i32,
    defender_trophies: i32,
    attacker_townhall: i32,
    defender_townhall: i32,
    attacker_th: i32,
    defender_th: i32,
    map_name: String,
    duration: f32,
    stars_earned: u8,
    destruction_percent: f32,
    result: BattleResult,
    file_size: u64,
    checksum: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum BattleResult {
    #[default]
    Victory3Stars,
    Victory2Stars,
    Victory1Star,
    Defeat0Stars,
    Draw,
}

/// Replay frame (snapshot at a specific time)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayFrame {
    pub timestamp: f32, // seconds from start
    pub attacker_troops: Vec<TroopSnapshot>,
    pub defender_buildings: Vec<BuildingSnapshot>,
    pub spells: Vec<SpellSnapshot>,
    pub heroes: Vec<HeroSnapshot>,
    pub projectiles: Vec<ProjectileSnapshot>,
    pub destruction_percent: f32,
    pub stars_earned: u8,
}

/// Troop snapshot at a specific frame
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TroopSnapshot {
    pub id: i64,
    pub character_data_id: i32,
    pub level: i32,
    pub position: Position,
    pub target: Option<Target>,
    pub hitpoints: f32,
    pub max_hitpoints: f32,
    pub state: TroopState,
    pub path_index: usize,
}

/// Building snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildingSnapshot {
    pub instance_id: i64,
    pub building_data_id: i32,
    pub position: crate::logic::buildings::GridPos,
    pub level: i32,
    pub state: BuildingState,
    pub hitpoints: i32,
    pub max_hitpoints: i32,
}

/// Spell snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpellSnapshot {
    pub id: i64,
    pub spell_data_id: i32,
    pub level: i32,
    pub position: Position,
    pub radius: f32,
    pub remaining_time: f32,
    pub state: SpellState,
}

/// Hero snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeroSnapshot {
    pub id: i64,
    pub hero_data_id: i32,
    pub level: i32,
    pub position: Position,
    pub hitpoints: f32,
    pub max_hitpoints: f32,
    pub state: HeroState,
    pub ability_active: bool,
    pub ability_remaining: f32,
}

/// Projectile snapshot (for visual effects)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectileSnapshot {
    pub id: i64,
    pub projectile_type: ProjectileType,
    pub start: Position,
    pub target: Position,
    pub progress: f32, // 0.0 to 1.0
    pub damage: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProjectileType {
    Arrow,
    Cannonball,
    Fireball,
    Lightning,
    Rocket,
    MortarShell,
    IceShard,
}

/// Complete replay
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Replay {
    pub metadata: ReplayMetadata,
    pub frames: Vec<ReplayFrame>,
    pub events: Vec<ReplayEvent>,
    pub initial_state: BattleInitialState,
}

/// Battle initial state for replay
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BattleInitialState {
    pub attacker_troops: Vec<DeployedTroopSnapshot>,
    pub attacker_spells: Vec<DeployedSpellSnapshot>,
    pub attacker_heroes: Vec<DeployedHeroSnapshot>,
    pub defender_buildings: Vec<BuildingSnapshot>,
    pub defender_clan_castle: Vec<DeployedTroopSnapshot>,
    pub defender_resources: crate::logic::resources::ResourceManager,
}

/// Troop initial state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployedTroopSnapshot {
    pub character_data_id: i32,
    pub level: i32,
    pub position: Position,
    pub hitpoints: f32,
    pub max_hitpoints: f32,
}

/// Spell initial state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployedSpellSnapshot {
    pub spell_data_id: i32,
    pub level: i32,
    pub position: Position,
    pub radius: f32,
    pub duration: f32,
}

/// Hero initial state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployedHeroSnapshot {
    pub hero_data_id: i32,
    pub level: i32,
    pub position: Position,
    pub hitpoints: f32,
    pub max_hitpoints: f32,
}

/// Replay event (discrete events during battle)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayEvent {
    pub timestamp: f32,
    pub event_type: ReplayEventType,
    pub data: ReplayEventData,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReplayEventType {
    TroopDeployed,
    SpellCast,
    HeroAbility,
    BuildingDestroyed,
    BuildingDamaged,
    TroopDeath,
    StarEarned,
    TroopTargetChanged,
    SpellExpired,
    HeroAbilityUsed,
    BattleStart,
    BattleEnd,
    StarEarned3,
    StarEarned2,
    StarEarned1,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReplayEventData {
    TroopDeployed {
        troop_id: i64,
        character_id: i32,
        position: Position,
    },
    SpellCast {
        spell_id: i64,
        spell_id_data: i32,
        position: Position,
    },
    HeroAbility {
        hero_id: i64,
        ability_name: String,
    },
    BuildingDestroyed {
        building_id: i64,
        destruction_percent: f32,
    },
    BuildingDamaged {
        building_id: i64,
        damage: i32,
        remaining_hp: i32,
    },
    TroopDeath {
        troop_id: i64,
    },
    StarEarned {
        stars: u8,
    },
    TroopTargetChanged {
        troop_id: i64,
        target: Option<Target>,
    },
    SpellExpired {
        spell_id: i64,
    },
    HeroAbilityUsed {
        hero_id: i64,
        ability_name: String,
    },
    BattleStart,
    BattleEnd {
        result: BattleResult,
        stars: u8,
        destruction: f32,
    },
}

/// Replay manager for recording and playback
pub struct ReplayRecorder {
    frames: Vec<ReplayFrame>,
    events: Vec<ReplayEvent>,
    initial_state: BattleInitialState,
    start_time: f32,
    current_time: f32,
    frame_interval: f32, // Record frame every N seconds
    last_frame_time: f32,
    metadata: ReplayMetadata,
}

impl ReplayRecorder {
    pub fn new(metadata: ReplayMetadata, frame_interval: f32) -> Self {
        Self {
            frames: Vec::new(),
            events: Vec::new(),
            initial_state: BattleInitialState::default(),
            start_time: 0.0,
            current_time: 0.0,
            frame_interval,
            last_frame_time: 0.0,
            metadata: ReplayMetadata::default(),
        }
    }
    
    pub fn start_recording(&mut self, initial_state: BattleInitialState) {
        self.initial_state = initial_state;
        self.start_time = 0.0;
        self.current_time = 0.0;
        self.frames.clear();
        self.events.clear();
        self.record_frame();
    }
    
    pub fn update(&mut self, dt: f32, battle: &crate::logic::combat::Battle) {
        self.current_time += dt;
        
        // Record events
        // (In a real implementation, we'd track events during battle update)
        
        // Record frame at interval
        if self.current_time - self.last_frame_time >= self.frame_interval {
            self.record_frame();
        }
    }
    
    fn record_frame(&mut self) {
        // Capture current state
        // This would capture all troop positions, building states, etc.
        self.last_frame_time = self.current_time;
    }
    
    pub fn add_event(&mut self, event: ReplayEvent) {
        self.events.push(event);
    }
    
    pub fn finish(&mut self, result: BattleResult, _destruction: f32, _stars: u8) -> Replay {
        self.events.push(ReplayEvent {
            timestamp: self.current_time,
            event_type: ReplayEventType::BattleEnd,
            data: ReplayEventData::BattleEnd {
                result,
                stars: self.metadata.stars_earned,
                destruction: self.metadata.destruction_percent,
            },
        });
        
        Replay {
            metadata: self.metadata.clone(),
            frames: self.frames.clone(),
            events: self.events.clone(),
            initial_state: self.initial_state.clone(),
        }
    }
    
    pub fn export_json(&self) -> Result<String, serde_json::Error> {
        let replay = Replay {
            metadata: self.metadata.clone(),
            frames: self.frames.clone(),
            events: self.events.clone(),
            initial_state: self.initial_state.clone(),
        };
        serde_json::to_string_pretty(&replay)
    }
    
    pub fn export_binary(&self) -> Result<Vec<u8>, bincode::Error> {
        let replay = Replay {
            metadata: self.metadata.clone(),
            frames: self.frames.clone(),
            events: self.events.clone(),
            initial_state: self.initial_state.clone(),
        };
        bincode::serialize(&replay)
    }
}

impl Default for BattleInitialState {
    fn default() -> Self {
        Self {
            attacker_troops: Vec::new(),
            attacker_spells: Vec::new(),
            attacker_heroes: Vec::new(),
            defender_buildings: Vec::new(),
            defender_clan_castle: Vec::new(),
            defender_resources: crate::logic::resources::ResourceManager::new(),
        }
    }
}

/// Replay player for playback
pub struct ReplayPlayer {
    replay: Replay,
    current_frame: usize,
    current_time: f32,
    current_event_index: usize,
    playback_speed: f32,
    paused: bool,
    on_event: Option<Box<dyn Fn(&ReplayEvent)>>,
    on_frame: Option<Box<dyn Fn(&ReplayFrame)>>,
}

impl ReplayPlayer {
    pub fn new(replay: Replay) -> Self {
        Self {
            replay,
            current_frame: 0,
            current_time: 0.0,
            current_event_index: 0,
            playback_speed: 1.0,
            paused: true,
            on_event: None,
            on_frame: None,
        }
    }
    
    pub fn play(&mut self) {
        self.paused = false;
    }
    
    pub fn pause(&mut self) {
        self.paused = true;
    }
    
    pub fn set_speed(&mut self, speed: f32) {
        self.playback_speed = speed.max(0.1).min(10.0);
    }
    
    pub fn seek(&mut self, time: f32) {
        self.current_time = time.clamp(0.0, self.replay.metadata.duration);
        // Find closest frame
        self.current_frame = self.replay.frames
            .iter()
            .position(|f| f.timestamp >= self.current_time)
            .unwrap_or(self.replay.frames.len().saturating_sub(1));
    }
    
    pub fn set_event_callback(&mut self, callback: Box<dyn Fn(&ReplayEvent)>) {
        self.on_event = Some(callback);
    }
    
    pub fn set_frame_callback(&mut self, callback: Box<dyn Fn(&ReplayFrame)>) {
        self.on_frame = Some(callback);
    }
    
    pub fn update(&mut self, dt: f32) -> bool {
        if self.paused {
            return false;
        }
        
        let dt = dt * self.playback_speed;
        self.current_time += dt;
        
        // Process events
        while let Some(event) = self.replay.events.get(self.current_event_index) {
            if event.timestamp <= self.current_time {
                if let Some(cb) = &self.on_event {
                    cb(event);
                }
                self.current_event_index += 1;
            } else {
                break;
            }
        }
        
        // Update frame
        while self.current_frame + 1 < self.replay.frames.len() 
            && self.replay.frames[self.current_frame + 1].timestamp <= self.current_time {
            self.current_frame += 1;
            if let Some(cb) = &self.on_frame {
                cb(&self.replay.frames[self.current_frame]);
            }
        }
        
        self.current_time += dt / self.playback_speed;
        
        self.current_time >= self.replay.metadata.duration
    }
    
    pub fn get_current_frame(&self) -> Option<&ReplayFrame> {
        self.replay.frames.get(self.current_frame)
    }
    
    pub fn get_progress(&self) -> f32 {
        if self.replay.metadata.duration > 0.0 {
            self.current_time / self.replay.metadata.duration
        } else {
            0.0
        }
    }
    
    pub fn is_finished(&self) -> bool {
        self.current_time >= self.replay.metadata.duration
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_replay_metadata() {
        let metadata = ReplayMetadata {
            replay_id: "test".to_string(),
            version: "1.0".to_string(),
            game_version: "1.0".to_string(),
            timestamp: 0,
            attacker_id: 1,
            defender_id: 2,
            attacker_name: "Attacker".to_string(),
            defender_name: "Defender".to_string(),
            attacker_trophies: 1500,
            defender_trophies: 1400,
            attacker_townhall: 10,
            defender_townhall: 10,
            attacker_th: 10,
            defender_th: 10,
            map_name: "Test Map".to_string(),
            duration: 180.0,
            stars_earned: 3,
            destruction_percent: 100.0,
            result: BattleResult::Victory3Stars,
            file_size: 1024,
            checksum: "abc123".to_string(),
        };
        
        assert_eq!(metadata.result, BattleResult::Victory3Stars);
    }
}