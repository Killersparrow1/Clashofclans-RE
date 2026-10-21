//! CSV to Game Data Parser
//! 
//! Converts parsed CSV tables into typed game data structures.

use super::game_data::*;
use crate::assets::csv::{CsvParser, CsvTable, find_table, table_names};
use crate::assets::AssetError;
use std::collections::HashMap;

pub struct CsvGameDataParser {
    parser: CsvParser,
}

impl CsvGameDataParser {
    pub fn new() -> Self {
        Self {
            parser: CsvParser::new(),
        }
    }
    
    pub fn with_key(mut self, key: [u8; 16], iv: [u8; 16]) -> Self {
        self.parser = self.parser.with_key(key, iv);
        self
    }
    
    pub fn parse_all(&self, csv_files: &HashMap<String, Vec<u8>>) -> Result<GameData, AssetError> {
        let mut game_data = GameData::new();
        
        // Parse each CSV file
        for (name, data) in csv_files {
            let tables = self.parser.parse_file(data)?;
            
            // Find the main table in the file
            if let Some(table) = find_table(&tables, name) {
                self.parse_table(&mut game_data, name, table)?;
            }
        }
        
        Ok(game_data)
    }
    
    fn parse_table(&self, game_data: &mut GameData, name: &str, table: &crate::assets::csv::CsvTable) -> Result<(), AssetError> {
        match name {
            table_names::BUILDINGS => self.parse_buildings_table(game_data, table)?,
            table_names::CHARACTERS => self.parse_characters_table(game_data, table)?,
            table_names::SPELLS => self.parse_spells_table(game_data, table)?,
            table_names::TRAPS => self.parse_traps_table(game_data, table)?,
            table_names::DECORATIONS => self.parse_decorations_table(game_data, table)?,
            table_names::OBSTACLES => self.parse_obstacles_table(game_data, table)?,
            table_names::TOWNHALL_LEVELS => self.parse_townhall_levels_table(game_data, table)?,
            _ => {
                // Unknown table, skip
            }
        }
        Ok(())
    }
    
    fn parse_buildings_table(&self, game_data: &mut GameData, table: &crate::assets::csv::CsvTable) -> Result<(), AssetError> {
        for row in &table.rows {
            let building = BuildingData {
                id: row.get_int("id").unwrap_or(0) as i32,
                name: row.get("name").cloned().unwrap_or_default(),
                building_type: self.parse_building_type(row.get("type").cloned().unwrap_or_default()),
                width: row.get_int("width").unwrap_or(0) as i32,
                height: row.get_int("height").unwrap_or(0) as i32,
                max_level: row.get_int("maxLevel").unwrap_or(1) as i32,
                townhall_level_required: row.get_int("townHallLevelRequired").unwrap_or(1) as i32,
                build_time: self.parse_int_vec(row, "buildTime"),
                build_cost: self.parse_resource_cost_vec(row, "buildCost"),
                gold_storage: self.parse_int64_vec(row, "goldStorage"),
                elixir_storage: self.parse_int64_vec(row, "elixirStorage"),
                dark_elixir_storage: self.parse_int64_vec(row, "darkElixirStorage"),
                production_rate: self.parse_int64_vec(row, "productionRate"),
                capacity: self.parse_int64_vec(row, "capacity"),
                hitpoints: self.parse_int_vec(row, "hitpoints"),
                damage_per_second: self.parse_int_vec(row, "damagePerSecond"),
                range: self.parse_int_vec(row, "range"),
                favorite_target: self.parse_string_vec(row, "favoriteTarget"),
                damage_type: self.parse_string_vec(row, "damageType"),
                hit_speed: self.parse_float_vec(row, "hitSpeed"),
                splash_radius: self.parse_int_vec(row, "splashRadius"),
                housing_space: self.parse_int_vec(row, "housingSpace"),
                training_time: self.parse_int_vec(row, "trainingTime"),
                research_cost: self.parse_resource_cost_vec(row, "researchCost"),
                research_time: self.parse_int_vec(row, "researchTime"),
                laboratory_level_required: self.parse_int_vec(row, "laboratoryLevelRequired"),
                sub_type: row.get("subType").cloned().unwrap_or_default(),
                description: row.get("description").cloned().unwrap_or_default(),
            };
            game_data.buildings.insert(building.id, building);
        }
        Ok(())
    }
    
    fn parse_characters_table(&self, game_data: &mut GameData, table: &crate::assets::csv::CsvTable) -> Result<(), AssetError> {
        for row in &table.rows {
            let character = CharacterData {
                id: row.get_int("id").unwrap_or(0) as i32,
                name: row.get("name").cloned().unwrap_or_default(),
                character_type: self.parse_character_type(row.get("type").cloned().unwrap_or_default()),
                housing_space: row.get_int("housingSpace").unwrap_or(1) as i32,
                training_time: row.get_int("trainingTime").unwrap_or(0) as i32,
                training_cost: self.parse_resource_cost_vec(row, "trainingCost"),
                max_level: row.get_int("maxLevel").unwrap_or(1) as i32,
                townhall_level_required: row.get_int("townHallLevelRequired").unwrap_or(1) as i32,
                barracks_level_required: self.parse_int_vec(row, "barracksLevelRequired"),
                dark_barracks_level_required: self.parse_int_vec(row, "darkBarracksLevelRequired"),
                hitpoints: self.parse_int_vec(row, "hitpoints"),
                damage_per_second: self.parse_int_vec(row, "damagePerSecond"),
                damage_per_attack: self.parse_int_vec(row, "damagePerAttack"),
                hitpoints_heal: self.parse_int_vec(row, "hitpointsHeal"),
                speed: self.parse_float_vec(row, "speed"),
                attack_speed: self.parse_float_vec(row, "attackSpeed"),
                range: self.parse_float_vec(row, "range"),
                sight_range: self.parse_float_vec(row, "sightRange"),
                movement_speed: self.parse_float_vec(row, "movementSpeed"),
                attack_range: self.parse_float_vec(row, "attackRange"),
                favorite_target: self.parse_string_vec(row, "favoriteTarget"),
                damage_type: self.parse_string_vec(row, "damageType"),
                can_heal: self.parse_bool_vec(row, "canHeal"),
                can_jump_walls: self.parse_bool_vec(row, "canJumpWalls"),
                can_fly: self.parse_bool_vec(row, "canFly"),
                is_super_troop: row.get_bool("isSuperTroop").unwrap_or(false),
                super_troop_base: row.get_int("superTroopBase").map(|v| v as i32),
                description: row.get("description").cloned().unwrap_or_default(),
            };
            game_data.characters.insert(character.id, character);
        }
        Ok(())
    }
    
    fn parse_spells_table(&self, game_data: &mut GameData, table: &crate::assets::csv::CsvTable) -> Result<(), AssetError> {
        for row in &table.rows {
            let spell = SpellData {
                id: row.get_int("id").unwrap_or(0) as i32,
                name: row.get("name").cloned().unwrap_or_default(),
                spell_type: self.parse_spell_type(row.get("type").cloned().unwrap_or_default()),
                housing_space: row.get_int("housingSpace").unwrap_or(1) as i32,
                brewing_time: row.get_int("brewingTime").unwrap_or(0) as i32,
                brewing_cost: self.parse_resource_cost_vec(row, "brewingCost"),
                max_level: row.get_int("maxLevel").unwrap_or(1) as i32,
                factory_level_required: self.parse_int_vec(row, "factoryLevelRequired"),
                dark_factory_level_required: self.parse_int_vec(row, "darkFactoryLevelRequired"),
                radius: self.parse_float_vec(row, "radius"),
                duration: self.parse_float_vec(row, "duration"),
                damage: self.parse_int_vec(row, "damage"),
                heal_amount: self.parse_int_vec(row, "healAmount"),
                speed_multiplier: self.parse_float_vec(row, "speedMultiplier"),
                damage_per_second: self.parse_int_vec(row, "damagePerSecond"),
                freeze_duration: self.parse_float_vec(row, "freezeDuration"),
                clone_housing_limit: self.parse_int_vec(row, "cloneHousingLimit"),
                description: row.get("description").cloned().unwrap_or_default(),
            };
            game_data.spells.insert(spell.id, spell);
        }
        Ok(())
    }
    
    fn parse_traps_table(&self, game_data: &mut GameData, table: &crate::assets::csv::CsvTable) -> Result<(), AssetError> {
        for row in &table.rows {
            let trap = TrapData {
                id: row.get_int("id").unwrap_or(0) as i32,
                name: row.get("name").cloned().unwrap_or_default(),
                trap_type: self.parse_trap_type(row.get("type").cloned().unwrap_or_default()),
                max_level: row.get_int("maxLevel").unwrap_or(1) as i32,
                townhall_level_required: row.get_int("townHallLevelRequired").unwrap_or(1) as i32,
                build_time: self.parse_int_vec(row, "buildTime"),
                build_cost: self.parse_resource_cost_vec(row, "buildCost"),
                damage: self.parse_int_vec(row, "damage"),
                trigger_radius: self.parse_float_vec(row, "triggerRadius"),
                trigger_type: self.parse_string_vec(row, "triggerType"),
                reset_time: self.parse_int_vec(row, "resetTime"),
                description: row.get("description").cloned().unwrap_or_default(),
            };
            game_data.traps.insert(trap.id, trap);
        }
        Ok(())
    }
    
    fn parse_decorations_table(&self, game_data: &mut GameData, table: &crate::assets::csv::CsvTable) -> Result<(), AssetError> {
        for row in &table.rows {
            let decoration = DecorationData {
                id: row.get_int("id").unwrap_or(0) as i32,
                name: row.get("name").cloned().unwrap_or_default(),
                width: row.get_int("width").unwrap_or(0) as i32,
                height: row.get_int("height").unwrap_or(0) as i32,
                cost: self.parse_resource_cost_vec(row, "cost"),
                description: row.get("description").cloned().unwrap_or_default(),
            };
            game_data.decorations.insert(decoration.id, decoration);
        }
        Ok(())
    }
    
    fn parse_obstacles_table(&self, game_data: &mut GameData, table: &crate::assets::csv::CsvTable) -> Result<(), AssetError> {
        for row in &table.rows {
            let obstacle = ObstacleData {
                id: row.get_int("id").unwrap_or(0) as i32,
                name: row.get("name").cloned().unwrap_or_default(),
                width: row.get_int("width").unwrap_or(0) as i32,
                height: row.get_int("height").unwrap_or(0) as i32,
                clear_time: row.get_int("clearTime").unwrap_or(0) as i32,
                clear_cost: self.parse_resource_cost_vec(row, "clearCost"),
                loot: self.parse_resource_cost_vec(row, "loot"),
                description: row.get("description").cloned().unwrap_or_default(),
            };
            game_data.obstacles.insert(obstacle.id, obstacle);
        }
        Ok(())
    }
    
    fn parse_townhall_levels_table(&self, game_data: &mut GameData, table: &crate::assets::csv::CsvTable) -> Result<(), AssetError> {
        for row in &table.rows {
            let level = TownHallLevelData {
                level: row.get_int("level").unwrap_or(1) as i32,
                hitpoints: row.get_int("hitpoints").unwrap_or(1000) as i32,
                max_buildings: self.parse_building_limits(row),
                max_walls: row.get_int("maxWalls").unwrap_or(0) as i32,
                max_traps: row.get_int("maxTraps").unwrap_or(0) as i32,
                max_decorations: row.get_int("maxDecorations").unwrap_or(0) as i32,
                gold_storage: row.get_int("goldStorage").unwrap_or(0) as i64,
                elixir_storage: row.get_int("elixirStorage").unwrap_or(0) as i64,
                dark_elixir_storage: row.get_int("darkElixirStorage").unwrap_or(0) as i64,
                builder_huts: row.get_int("builderHuts").unwrap_or(1) as i32,
                spell_factory: row.get_bool("hasSpellFactory").unwrap_or(false),
                dark_spell_factory: row.get_bool("hasDarkSpellFactory").unwrap_or(false),
                workshop: row.get_bool("hasWorkshop").unwrap_or(false),
                pet_house: row.get_bool("hasPetHouse").unwrap_or(false),
            };
            game_data.townhall_levels.insert(level.level, level);
        }
        Ok(())
    }
    
    // Helper parsing functions
    
    fn parse_building_type(&self, s: String) -> BuildingType {
        match s.to_lowercase().as_str() {
            "resourcecollector" => BuildingType::ResourceCollector,
            "resourcestorage" => BuildingType::ResourceStorage,
            "defense" => BuildingType::Defense,
            "army" => BuildingType::Army,
            "trap" => BuildingType::Trap,
            "decoration" => BuildingType::Decoration,
            "obstacle" => BuildingType::Obstacle,
            "special" => BuildingType::Special,
            "wall" => BuildingType::Wall,
            "townhall" => BuildingType::TownHall,
            "clancastle" => BuildingType::ClanCastle,
            "builderhut" => BuildingType::BuilderHut,
            "laboratory" => BuildingType::Laboratory,
            "spellfactory" => BuildingType::SpellFactory,
            "barracks" => BuildingType::Barracks,
            "darkbarracks" => BuildingType::DarkBarracks,
            "darkspellfactory" => BuildingType::DarkSpellFactory,
            "workshop" => BuildingType::Workshop,
            "heroaltar" => BuildingType::HeroAltar,
            "siegemachine" => BuildingType::SiegeMachine,
            "pethouse" => BuildingType::PetHouse,
            "supertroop" => BuildingType::SuperTroop,
            _ => BuildingType::Unknown,
        }
    }
    
    fn parse_character_type(&self, s: String) -> CharacterType {
        match s.to_lowercase().as_str() {
            "normal" => CharacterType::Normal,
            "dark" => CharacterType::Dark,
            "super" => CharacterType::Super,
            "hero" => CharacterType::Hero,
            "siege" => CharacterType::Siege,
            "pet" => CharacterType::Pet,
            _ => CharacterType::Normal,
        }
    }
    
    fn parse_spell_type(&self, s: String) -> SpellType {
        match s.to_lowercase().as_str() {
            "normal" => SpellType::Normal,
            "dark" => SpellType::Dark,
            "siege" => SpellType::Siege,
            _ => SpellType::Normal,
        }
    }
    
    fn parse_trap_type(&self, s: String) -> TrapType {
        match s.to_lowercase().as_str() {
            "normal" => TrapType::Normal,
            "seekingairmine" => TrapType::SeekingAirMine,
            "airbomb" => TrapType::AirBomb,
            "giantbomb" => TrapType::GiantBomb,
            "skeletontrap" => TrapType::SkeletonTrap,
            "tornadotrap" => TrapType::TornadoTrap,
            _ => TrapType::Normal,
        }
    }
    
    fn parse_int_vec(&self, row: &crate::assets::csv::CsvRow, field: &str) -> Vec<i32> {
        row.get(field)
            .and_then(|s| {
                s.split(',')
                    .filter_map(|v| v.trim().parse::<i32>().ok())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    }
    
    fn parse_int64_vec(&self, row: &crate::assets::csv::CsvRow, field: &str) -> Vec<i64> {
        row.get(field)
            .and_then(|s| {
                s.split(',')
                    .filter_map(|v| v.trim().parse::<i64>().ok())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    }
    
    fn parse_float_vec(&self, row: &crate::assets::csv::CsvRow, field: &str) -> Vec<f32> {
        row.get(field)
            .and_then(|s| {
                s.split(',')
                    .filter_map(|v| v.trim().parse::<f32>().ok())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    }
    
    fn parse_string_vec(&self, row: &crate::assets::csv::CsvRow, field: &str) -> Vec<String> {
        row.get(field)
            .and_then(|s| {
                Some(s.split(',').map(|v| v.trim().to_string()).collect())
            })
            .unwrap_or_default()
    }
    
    fn parse_bool_vec(&self, row: &crate::assets::csv::CsvRow, field: &str) -> Vec<bool> {
        row.get(field)
            .and_then(|s| {
                Some(s.split(',')
                    .map(|v| v.trim() == "1" || v.trim().eq_ignore_ascii_case("true"))
                    .collect())
            })
            .unwrap_or_default()
    }
    
    fn parse_resource_cost_vec(&self, row: &crate::assets::csv::CsvRow, field: &str) -> Vec<ResourceCost> {
        // Format: "gold,elixir,dark_elixir,builder_gold,builder_elixir,time"
        row.get(field)
            .and_then(|s| {
                let costs: Vec<ResourceCost> = s.split(';')
                    .filter_map(|cost_str| {
                        let parts: Vec<&str> = cost_str.split(',').collect();
                        if parts.len() >= 6 {
                            Some(ResourceCost {
                                gold: parts[0].parse().ok()?,
                                elixir: parts[1].parse().ok()?,
                                dark_elixir: parts[2].parse().ok()?,
                                builder_gold: parts[3].parse().ok()?,
                                builder_elixir: parts[4].parse().ok()?,
                                time_seconds: parts[5].parse().ok()?,
                            })
                        } else {
                            None
                        }
                    })
                    .collect();
                if costs.is_empty() { None } else { Some(costs) }
            })
            .unwrap_or_default()
    }
    
    fn parse_building_limits(&self, row: &crate::assets::csv::CsvRow) -> HashMap<String, i32> {
        // Format: "building_type:count;building_type:count"
        row.get("maxBuildings")
            .and_then(|s| {
                let mut map = HashMap::new();
                for pair in s.split(';') {
                    let parts: Vec<&str> = pair.split(':').collect();
                    if parts.len() == 2 {
                        if let Ok(count) = parts[1].parse::<i32>() {
                            map.insert(parts[0].to_string(), count);
                        }
                    }
                }
                Some(map)
            })
            .unwrap_or_default()
    }
}

impl Default for CsvGameDataParser {
    fn default() -> Self {
        Self::new()
    }
}