//! Building Placement and Upgrade Logic
//! 
//! Handles building placement validation, collision detection, and upgrade management.

use super::game_data::*;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// Grid position (tile coordinates)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GridPos {
    pub x: i32,
    pub y: i32,
}

impl GridPos {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
    
    pub fn distance_to(&self, other: &GridPos) -> f32 {
        let dx = (self.x - other.x) as f32;
        let dy = (self.y - other.y) as f32;
        (dx * dx + dy * dy).sqrt()
    }
}

/// Building instance in the village
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildingInstance {
    pub id: i64,                    // Unique instance ID
    pub building_data_id: i32,      // Reference to BuildingData
    pub position: GridPos,
    pub level: i32,
    pub state: BuildingState,
    pub upgrade_finish_time: Option<i64>, // Unix timestamp
    pub construction_finish_time: Option<i64>, // Unix timestamp
    pub rotation: i32,              // 0, 90, 180, 270 degrees
    pub hitpoints: i32,             // Current HP (for damaged buildings)
}

impl BuildingInstance {
    pub fn new(instance_id: i64, data_id: i32, position: GridPos) -> Self {
        Self {
            id: instance_id,
            building_data_id: data_id,
            position,
            level: 1,
            state: BuildingState::Constructing,
            upgrade_finish_time: None,
            construction_finish_time: None,
            rotation: 0,
            hitpoints: 0,
        }
    }
    
    pub fn get_data<'a>(&self, game_data: &'a GameData) -> Option<&'a BuildingData> {
        game_data.get_building(self.building_data_id)
    }
    
    pub fn get_current_data<'a>(&self, game_data: &'a GameData) -> Option<BuildingDataSnapshot> {
        self.get_data(game_data).map(|data| {
            let level_idx = (self.level - 1) as usize;
            BuildingDataSnapshot {
                building_type: data.building_type,
                width: data.width,
                height: data.height,
                level: self.level,
                max_level: data.max_level,
                hitpoints: data.hitpoints.get(level_idx).copied().unwrap_or(0),
                damage_per_second: data.damage_per_second.get(level_idx).copied().unwrap_or(0),
                range: data.range.get(level_idx).copied().unwrap_or(0),
                damage_per_second_value: data.damage_per_second.get(level_idx).copied().unwrap_or(0),
                build_time: data.build_time.get(level_idx).copied().unwrap_or(0),
                build_cost: data.build_cost.get(level_idx).cloned().unwrap_or_default(),
                townhall_level_required: data.townhall_level_required,
            }
        })
    }
    
    pub fn is_max_level(&self, game_data: &GameData) -> bool {
        if let Some(data) = self.get_data(game_data) {
            self.level >= data.max_level
        } else {
            true
        }
    }
    
    pub fn can_upgrade(&self, game_data: &GameData, townhall_level: i32) -> bool {
        if self.is_max_level(game_data) {
            return false;
        }
        if self.state != BuildingState::Active {
            return false;
        }
        if let Some(data) = self.get_data(game_data) {
            let next_level = self.level as usize;
            if next_level >= data.build_time.len() {
                return false;
            }
            if townhall_level < data.townhall_level_required {
                return false;
            }
            // Check if already upgrading/constructing
            if self.state == BuildingState::Upgrading || self.state == BuildingState::Constructing {
                return false;
            }
            true
        } else {
            false
        }
    }
    
    pub fn start_upgrade(&mut self, finish_time: i64) {
        self.state = BuildingState::Upgrading;
        self.upgrade_finish_time = Some(finish_time);
    }
    
    pub fn complete_upgrade(&mut self) {
        self.level += 1;
        self.state = BuildingState::Active;
        self.upgrade_finish_time = None;
    }
    
    pub fn complete_construction(&mut self) {
        self.state = BuildingState::Active;
        self.construction_finish_time = None;
    }
}

/// Snapshot of building data at current level
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildingDataSnapshot {
    pub building_type: BuildingType,
    pub width: i32,
    pub height: i32,
    pub level: i32,
    pub max_level: i32,
    pub hitpoints: i32,
    pub damage_per_second: i32,
    pub range: i32,
    pub damage_per_second_value: i32,
    pub build_time: i32,
    pub build_cost: ResourceCost,
    pub townhall_level_required: i32,
}

/// Village grid (44x44 tiles for main village)
pub struct VillageGrid {
    pub width: i32,
    pub height: i32,
    pub tiles: HashMap<GridPos, GridTile>,
    pub buildings: HashMap<i64, BuildingInstance>,
    pub next_instance_id: i64,
}

impl VillageGrid {
    pub fn new() -> Self {
        Self {
            width: 44,
            height: 44,
            tiles: HashMap::new(),
            buildings: HashMap::new(),
            next_instance_id: 1,
        }
    }
    
    pub fn place_building(&mut self, building_data_id: i32, position: GridPos, game_data: &GameData) -> Result<i64, String> {
        let data = game_data.get_building(building_data_id)
            .ok_or("Building data not found")?;
        
        // Check bounds
        if position.x < 0 || position.y < 0 ||
           position.x + data.width > self.width ||
           position.y + data.height > self.height {
            return Err("Building out of bounds".to_string());
        }
        
        // Check collision
        for x in position.x..position.x + data.width {
            for y in position.y..position.y + data.height {
                let pos = GridPos::new(x, y);
                if self.tiles.contains_key(&pos) {
                    return Err(format!("Collision at {}, {}", x, y));
                }
            }
        }
        
        // Place building
        let instance_id = self.next_instance_id;
        self.next_instance_id += 1;
        
        let mut building = BuildingInstance::new(instance_id, building_data_id, position);
        
        // Mark tiles as occupied
        for x in position.x..position.x + data.width {
            for y in position.y..position.y + data.height {
                self.tiles.insert(GridPos::new(x, y), GridTile {
                    occupied: true,
                    building_id: Some(instance_id),
                });
            }
        }
        
        self.buildings.insert(instance_id, building);
        Ok(instance_id)
    }
    
    pub fn remove_building(&mut self, instance_id: i64) -> Result<(), String> {
        let building = self.buildings.remove(&instance_id)
            .ok_or("Building not found")?;
        
        // Get building dimensions before removing
        let (width, height) = if let Some(data) = self.get_building_data(building.building_data_id) {
            (data.width, data.height)
        } else {
            return Ok(());
        };
        
        for x in building.position.x..building.position.x + width {
            for y in building.position.y..building.position.y + height {
                self.tiles.remove(&GridPos::new(x, y));
            }
        }
        Ok(())
    }
    
    pub fn move_building(&mut self, instance_id: i64, new_position: GridPos) -> Result<(), String> {
        // Get building data first (before any mutable operations)
        let (data_width, data_height, building_position) = {
            let building = self.buildings.get(&instance_id)
                .ok_or("Building not found")?;
            
            let data = self.get_building_data(building.building_data_id)
                .ok_or("Building data not found")?;
            
            (data.width, data.height, building.position)
        };
        
        // Check bounds
        if new_position.x < 0 || new_position.y < 0 ||
           new_position.x + data_width > self.width ||
           new_position.y + data_height > self.height {
            return Err("Building out of bounds".to_string());
        }
        
        // Check collision (excluding self)
        for x in new_position.x..new_position.x + data_width {
            for y in new_position.y..new_position.y + data_height {
                let pos = GridPos::new(x, y);
                if let Some(tile) = self.tiles.get(&pos) {
                    if tile.building_id != Some(instance_id) {
                        return Err(format!("Collision at {}, {}", x, y));
                    }
                }
            }
        }
        
        // Remove from old position
        for x in building_position.x..building_position.x + data_width {
            for y in building_position.y..building_position.y + data_height {
                self.tiles.remove(&GridPos::new(x, y));
            }
        }
        
        // Place at new position
        for x in new_position.x..new_position.x + data_width {
            for y in new_position.y..new_position.y + data_height {
                self.tiles.insert(GridPos::new(x, y), GridTile {
                    occupied: true,
                    building_id: Some(instance_id),
                });
            }
        }
        
        // Update building position
        if let Some(building) = self.buildings.get_mut(&instance_id) {
            building.position = new_position;
        }
        
        Ok(())
    }
    
    pub fn get_building_data(&self, building_data_id: i32) -> Option<&BuildingData> {
        // This would need access to GameData - simplified for now
        None
    }
    
    pub fn get_building(&self, instance_id: i64) -> Option<&BuildingInstance> {
        self.buildings.get(&instance_id)
    }
    
    pub fn get_building_mut(&mut self, instance_id: i64) -> Option<&mut BuildingInstance> {
        self.buildings.get_mut(&instance_id)
    }
    
    pub fn get_all_buildings(&self) -> Vec<&BuildingInstance> {
        self.buildings.values().collect()
    }
    
    pub fn get_buildings_by_type(&self, building_type: BuildingType, game_data: &GameData) -> Vec<&BuildingInstance> {
        self.buildings.values()
            .filter(|b| {
                if let Some(data) = game_data.get_building(b.building_data_id) {
                    data.building_type == building_type
                } else {
                    false
                }
            })
            .collect()
    }
}

/// Grid tile representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridTile {
    pub occupied: bool,
    pub building_id: Option<i64>,
}

/// Resource manager for village
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceManager {
    pub gold: i64,
    pub elixir: i64,
    pub dark_elixir: i64,
    pub builder_gold: i64,
    pub builder_elixir: i64,
    pub gems: i32,
    pub star_bonus: i32,
    
    // Production rates (per second)
    pub gold_rate: f64,
    pub elixir_rate: f64,
    pub dark_elixir_rate: f64,
    
    // Storage capacities
    pub gold_capacity: i64,
    pub elixir_capacity: i64,
    pub dark_elixir_capacity: i64,
    
    pub last_update: i64, // Unix timestamp
}

impl ResourceManager {
    pub fn new() -> Self {
        Self {
            gold: 0,
            elixir: 0,
            dark_elixir: 0,
            builder_gold: 0,
            builder_elixir: 0,
            gems: 0,
            star_bonus: 0,
            gold_rate: 0.0,
            elixir_rate: 0.0,
            dark_elixir_rate: 0.0,
            gold_capacity: 1000,
            elixir_capacity: 1000,
            dark_elixir_capacity: 0,
            last_update: chrono::Utc::now().timestamp(),
        }
    }
    
    pub fn update_resources(&mut self, current_time: i64) {
        let dt = (current_time - self.last_update) as f64;
        
        self.gold = (self.gold as f64 + self.gold_rate * dt).min(self.gold_capacity as f64) as i64;
        self.elixir = (self.elixir as f64 + self.elixir_rate * dt).min(self.elixir_capacity as f64) as i64;
        self.dark_elixir = (self.dark_elixir as f64 + self.dark_elixir_rate * dt).min(self.dark_elixir_capacity as f64) as i64;
        
        self.last_update = current_time;
    }
    
    pub fn add_resources(&mut self, gold: i64, elixir: i64, dark_elixir: i64) {
        self.gold = (self.gold + gold).min(self.gold_capacity);
        self.elixir = (self.elixir + elixir).min(self.elixir_capacity);
        self.dark_elixir = (self.dark_elixir + dark_elixir).min(self.dark_elixir_capacity);
    }
    
    pub fn spend_resources(&mut self, gold: i64, elixir: i64, dark_elixir: i64) -> bool {
        if self.gold < gold || self.elixir < elixir || self.dark_elixir < dark_elixir {
            return false;
        }
        self.gold -= gold;
        self.elixir -= elixir;
        self.dark_elixir -= dark_elixir;
        true
    }
    
    pub fn can_afford(&self, cost: &ResourceCost) -> bool {
        self.gold >= cost.gold &&
        self.elixir >= cost.elixir &&
        self.dark_elixir >= cost.dark_elixir
    }
    
    pub fn recalculate_rates(&mut self, buildings: &[&BuildingInstance], game_data: &GameData) {
        let mut gold_rate = 0.0;
        let mut elixir_rate = 0.0;
        let mut dark_elixir_rate = 0.0;
        
        for building in buildings {
            if let Some(data) = building.get_data(game_data) {
                if data.building_type == BuildingType::ResourceCollector {
                    let level_idx = (building.level - 1) as usize;
                    if level_idx < data.production_rate.len() {
                        match data.building_type {
                            BuildingType::ResourceCollector => {
                                // Would need to know which resource this collector produces
                                // Simplified for now
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        
        self.gold_rate = gold_rate;
        self.elixir_rate = elixir_rate;
        self.dark_elixir_rate = dark_elixir_rate;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logic::game_data::*;
    
    #[test]
    fn test_grid_pos() {
        let pos1 = GridPos::new(0, 0);
        let pos2 = GridPos::new(3, 4);
        assert!((pos1.distance_to(&pos2) - 5.0).abs() < 0.001);
    }
    
    #[test]
    fn test_village_grid_placement() {
        let mut grid = VillageGrid::new();
        
        // Create mock building data
        let building_data = BuildingData {
            id: 1,
            name: "Cannon".to_string(),
            building_type: BuildingType::Defense,
            width: 3,
            height: 3,
            max_level: 21,
            townhall_level_required: 1,
            build_time: vec![10],
            build_cost: vec![ResourceCost { gold: 200, elixir: 0, dark_elixir: 0, builder_gold: 0, builder_elixir: 0, time_seconds: 10 }],
            gold_storage: vec![],
            elixir_storage: vec![],
            dark_elixir_storage: vec![],
            production_rate: vec![],
            capacity: vec![],
            hitpoints: vec![300],
            damage_per_second: vec![11],
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
            description: "Test".to_string(),
        };
        
        let mut game_data = GameData::new();
        game_data.buildings.insert(1, building_data);
        
        // Place building
        let result = grid.place_building(1, GridPos::new(10, 10), &game_data);
        assert!(result.is_ok());
        
        // Try to place overlapping
        let result2 = grid.place_building(1, GridPos::new(11, 10), &game_data);
        assert!(result2.is_err());
    }
}