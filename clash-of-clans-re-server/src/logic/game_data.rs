//! Game Logic Data Structures
//! 
//! Core data structures for Clash of Clans game logic.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Building type categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BuildingType {
    /// Resource collectors
    ResourceCollector,
    /// Resource storages
    ResourceStorage,
    /// Defensive buildings
    Defense,
    /// Army buildings
    Army,
    /// Traps
    Trap,
    /// Decorations
    Decoration,
    /// Obstacles
    Obstacle,
    /// Special buildings
    Special,
    /// Walls
    Wall,
    /// Town Hall
    TownHall,
    /// Clan Castle
    ClanCastle,
    /// Builder's Hut
    BuilderHut,
    /// Laboratory
    Laboratory,
    /// Spell Factory
    SpellFactory,
    /// Barracks
    Barracks,
    /// Dark Barracks
    DarkBarracks,
    /// Spell Factory (Dark)
    DarkSpellFactory,
    /// Workshop (Siege Machines)
    Workshop,
    /// Hero Altar
    HeroAltar,
    /// Siege Machine
    SiegeMachine,
    /// Pet House
    PetHouse,
    /// Super Troop
    SuperTroop,
    /// Unknown
    Unknown,
}

/// Building state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BuildingState {
    /// Normal/active
    Active,
    /// Being upgraded
    Upgrading,
    /// Under construction
    Constructing,
    /// Damaged/destroyed
    Destroyed,
    /// Sleeping (traps)
    Sleeping,
    /// Hidden (traps)
    Hidden,
}

/// Building definition from CSV
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildingData {
    pub id: i32,
    pub name: String,
    pub building_type: BuildingType,
    pub width: i32,
    pub height: i32,
    pub max_level: i32,
    pub townhall_level_required: i32,
    pub build_time: Vec<i32>,          // Per level (seconds)
    pub build_cost: Vec<ResourceCost>, // Per level
    pub gold_storage: Vec<i64>,        // Per level (for storages)
    pub elixir_storage: Vec<i64>,
    pub dark_elixir_storage: Vec<i64>,
    pub production_rate: Vec<i64>,     // Per level (for collectors)
    pub capacity: Vec<i64>,            // Per level (for storages/housing)
    pub hitpoints: Vec<i32>,           // Per level
    pub damage_per_second: Vec<i32>,   // Per level (for defenses)
    pub range: Vec<i32>,               // Per level (for defenses)
    pub favorite_target: Vec<String>,  // Per level (for defenses)
    pub damage_type: Vec<String>,      // Per level (for defenses)
    pub hit_speed: Vec<f32>,           // Per level (for defenses)
    pub splash_radius: Vec<i32>,       // Per level (for splash)
    pub housing_space: Vec<i32>,       // Per level (for army buildings)
    pub training_time: Vec<i32>,       // Per level (for troops)
    pub research_cost: Vec<ResourceCost>, // Per level
    pub research_time: Vec<i32>,       // Per level (seconds)
    pub laboratory_level_required: Vec<i32>,
    pub sub_type: String,
    pub description: String,
}

/// Resource cost for building/upgrading
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceCost {
    pub gold: i64,
    pub elixir: i64,
    pub dark_elixir: i64,
    pub builder_gold: i64,
    pub builder_elixir: i64,
    pub time_seconds: i32,
}

/// Character/Troop data from CSV
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterData {
    pub id: i32,
    pub name: String,
    pub character_type: CharacterType,
    pub housing_space: i32,
    pub training_time: i32,        // seconds
    pub training_cost: Vec<ResourceCost>, // per level
    pub max_level: i32,
    pub townhall_level_required: i32,
    pub barracks_level_required: Vec<i32>,
    pub dark_barracks_level_required: Vec<i32>,
    pub hitpoints: Vec<i32>,
    pub damage_per_second: Vec<i32>,
    pub damage_per_attack: Vec<i32>,
    pub hitpoints_heal: Vec<i32>,
    pub speed: Vec<f32>,
    pub attack_speed: Vec<f32>,
    pub range: Vec<f32>,
    pub sight_range: Vec<f32>,
    pub movement_speed: Vec<f32>,
    pub attack_range: Vec<f32>,
    pub favorite_target: Vec<String>,
    pub damage_type: Vec<String>,
    pub can_heal: Vec<bool>,
    pub can_jump_walls: Vec<bool>,
    pub can_fly: Vec<bool>,
    pub is_super_troop: bool,
    pub super_troop_base: Option<i32>,
    pub description: String,
}

/// Character type categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CharacterType {
    Normal,
    Dark,
    Super,
    Hero,
    Siege,
    Pet,
}

/// Spell data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpellData {
    pub id: i32,
    pub name: String,
    pub spell_type: SpellType,
    pub housing_space: i32,
    pub brewing_time: i32,
    pub brewing_cost: Vec<ResourceCost>,
    pub max_level: i32,
    pub factory_level_required: Vec<i32>,
    pub dark_factory_level_required: Vec<i32>,
    pub radius: Vec<f32>,
    pub duration: Vec<f32>,
    pub damage: Vec<i32>,
    pub heal_amount: Vec<i32>,
    pub speed_multiplier: Vec<f32>,
    pub damage_per_second: Vec<i32>,
    pub freeze_duration: Vec<f32>,
    pub clone_housing_limit: Vec<i32>,
    pub description: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpellType {
    Normal,
    Dark,
    Siege,
}

/// Hero data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeroData {
    pub id: i32,
    pub name: String,
    pub max_level: i32,
    pub townhall_level_required: i32,
    pub altar_level_required: Vec<i32>,
    pub hitpoints: Vec<i32>,
    pub damage_per_second: Vec<i32>,
    pub movement_speed: Vec<f32>,
    pub attack_speed: Vec<f32>,
    pub attack_range: Vec<f32>,
    pub ability_name: Vec<String>,
    pub ability_description: Vec<String>,
    pub ability_cooldown: Vec<f32>,
    pub ability_duration: Vec<f32>,
    pub ability_radius: Vec<f32>,
    pub regeneration_time: Vec<i32>,
    pub description: String,
}

/// Trap data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrapData {
    pub id: i32,
    pub name: String,
    pub trap_type: TrapType,
    pub max_level: i32,
    pub townhall_level_required: i32,
    pub build_time: Vec<i32>,
    pub build_cost: Vec<ResourceCost>,
    pub damage: Vec<i32>,
    pub trigger_radius: Vec<f32>,
    pub trigger_type: Vec<String>, // air, ground, both
    pub reset_time: Vec<i32>,
    pub description: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrapType {
    Normal,
    SeekingAirMine,
    AirBomb,
    GiantBomb,
    SkeletonTrap,
    TornadoTrap,
}

/// Decoration data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecorationData {
    pub id: i32,
    pub name: String,
    pub width: i32,
    pub height: i32,
    pub cost: Vec<ResourceCost>,
    pub description: String,
}

/// Obstacle data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObstacleData {
    pub id: i32,
    pub name: String,
    pub width: i32,
    pub height: i32,
    pub clear_time: i32,
    pub clear_cost: Vec<ResourceCost>,
    pub loot: Vec<ResourceCost>,
    pub description: String,
}

/// Town Hall level data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TownHallLevelData {
    pub level: i32,
    pub hitpoints: i32,
    pub max_buildings: HashMap<String, i32>, // building_type -> count
    pub max_walls: i32,
    pub max_traps: i32,
    pub max_decorations: i32,
    pub gold_storage: i64,
    pub elixir_storage: i64,
    pub dark_elixir_storage: i64,
    pub builder_huts: i32,
    pub spell_factory: bool,
    pub dark_spell_factory: bool,
    pub workshop: bool,
    pub pet_house: bool,
}

/// All game data combined
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GameData {
    pub buildings: HashMap<i32, BuildingData>,
    pub characters: HashMap<i32, CharacterData>,
    pub spells: HashMap<i32, SpellData>,
    pub heroes: HashMap<i32, HeroData>,
    pub traps: HashMap<i32, TrapData>,
    pub decorations: HashMap<i32, DecorationData>,
    pub obstacles: HashMap<i32, ObstacleData>,
    pub townhall_levels: HashMap<i32, TownHallLevelData>,
}

impl GameData {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn get_building(&self, id: i32) -> Option<&BuildingData> {
        self.buildings.get(&id)
    }
    
    pub fn get_character(&self, id: i32) -> Option<&CharacterData> {
        self.characters.get(&id)
    }
    
    pub fn get_spell(&self, id: i32) -> Option<&SpellData> {
        self.spells.get(&id)
    }
    
    pub fn get_hero(&self, id: i32) -> Option<&HeroData> {
        self.heroes.get(&id)
    }
    
    pub fn get_trap(&self, id: i32) -> Option<&TrapData> {
        self.traps.get(&id)
    }
    
    pub fn get_townhall_level(&self, level: i32) -> Option<&TownHallLevelData> {
        self.townhall_levels.get(&level)
    }
    
    pub fn buildings_by_type(&self, building_type: BuildingType) -> Vec<&BuildingData> {
        self.buildings.values()
            .filter(|b| b.building_type == building_type)
            .collect()
    }
    
    pub fn characters_by_type(&self, character_type: CharacterType) -> Vec<&CharacterData> {
        self.characters.values()
            .filter(|c| c.character_type == character_type)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_building_data() {
        let building = BuildingData {
            id: 1,
            name: "Cannon".to_string(),
            building_type: BuildingType::Defense,
            width: 3,
            height: 3,
            max_level: 21,
            townhall_level_required: 1,
            build_time: vec![10, 60, 300, 1800],
            build_cost: vec![
                ResourceCost { gold: 200, elixir: 0, dark_elixir: 0, builder_gold: 0, builder_elixir: 0, time_seconds: 10 },
            ],
            gold_storage: vec![],
            elixir_storage: vec![],
            dark_elixir_storage: vec![],
            production_rate: vec![],
            capacity: vec![],
            hitpoints: vec![300, 350, 400],
            damage_per_second: vec![11, 13, 15],
            range: vec![9],
            favorite_target: vec!["Ground".to_string()],
            damage_type: vec!["Single".to_string()],
            hit_speed: vec![0.8],
            splash_radius: vec![],
            housing_space: vec![],
            training_time: vec![],
            research_cost: vec![],
            research_time: vec![],
            laboratory_level_required: vec![],
            sub_type: "Cannon".to_string(),
            description: "Shoots cannonballs at ground targets".to_string(),
        };
        
        assert_eq!(building.id, 1);
        assert_eq!(building.building_type, BuildingType::Defense);
        assert_eq!(building.max_level, 21);
    }
    
    #[test]
    fn test_character_data() {
        let character = CharacterData {
            id: 1,
            name: "Barbarian".to_string(),
            character_type: CharacterType::Normal,
            housing_space: 1,
            training_time: 20,
            training_cost: vec![
                ResourceCost { gold: 0, elixir: 25, dark_elixir: 0, builder_gold: 0, builder_elixir: 0, time_seconds: 20 },
            ],
            max_level: 10,
            townhall_level_required: 1,
            barracks_level_required: vec![1],
            dark_barracks_level_required: vec![],
            hitpoints: vec![45, 54, 66, 78, 96, 110, 127, 143, 164, 185],
            damage_per_second: vec![8, 9, 11, 13, 15, 18, 20, 22, 24, 25],
            damage_per_attack: vec![16, 18, 22, 26, 30, 36, 40, 44, 48, 50],
            hitpoints_heal: vec![],
            speed: vec![16.0, 16.0, 16.0, 16.0, 16.0, 16.0, 16.0, 16.0, 16.0, 16.0],
            attack_speed: vec![1.2, 1.2, 1.2, 1.2, 1.2, 1.2, 1.2, 1.2, 1.2, 1.2],
            range: vec![0.4],
            sight_range: vec![8.0],
            movement_speed: vec![16.0],
            attack_range: vec![0.4],
            favorite_target: vec!["Any".to_string()],
            damage_type: vec!["Normal".to_string()],
            can_heal: vec![false],
            can_jump_walls: vec![false],
            can_fly: vec![false],
            is_super_troop: false,
            super_troop_base: None,
            description: "Close range combat unit".to_string(),
        };
        
        assert_eq!(character.id, 1);
        assert_eq!(character.character_type, CharacterType::Normal);
        assert_eq!(character.max_level, 10);
    }
}