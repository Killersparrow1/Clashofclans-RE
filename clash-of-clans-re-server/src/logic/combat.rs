//! Combat Simulation
//! 
//! Handles attack simulation, troop deployment, pathfinding, and battle resolution.

use super::game_data::*;
use super::{GridPos, VillageGrid, BuildingInstance};
use super::resources::ResourceManager;
use serde::{Serialize, Deserialize};
use std::collections::{HashMap, VecDeque};
use std::cmp::Ordering;

/// Troop instance during battle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployedTroop {
    pub id: i64,
    pub character_data_id: i32,
    pub level: i32,
    pub position: Position,
    pub target: Option<Target>,
    pub hitpoints: f32,
    pub max_hitpoints: f32,
    pub state: TroopState,
    pub last_attack_time: f32,
    pub deploy_time: f32,
    pub path: Vec<GridPos>,
    pub path_index: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TroopState {
    Deploying,
    Moving,
    Attacking,
    Retargeting,
    Dead,
}

/// Target for troop attacks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Target {
    Building(i64),      // Building instance ID
    Wall(GridPos),      // Wall segment position
    None,
}

/// 2D position with sub-tile precision
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

impl Position {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
    
    pub fn from_grid(pos: GridPos) -> Self {
        Self::new(pos.x as f32 + 0.5, pos.y as f32 + 0.5)
    }
    
    pub fn to_grid(&self) -> GridPos {
        GridPos::new(self.x.floor() as i32, self.y.floor() as i32)
    }
    
    pub fn distance_to(&self, other: &Position) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }
    
    pub fn move_towards(&mut self, target: &Position, speed: f32, dt: f32) -> bool {
        let dx = target.x - self.x;
        let dy = target.y - self.y;
        let dist = (dx * dx + dy * dy).sqrt();
        
        if dist <= speed * dt {
            self.x = target.x;
            self.y = target.y;
            true
        } else {
            self.x += (dx / dist) * speed * dt;
            self.y += (dy / dist) * speed * dt;
            false
        }
    }
}

/// Battle state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Battle {
    pub attacker_id: i64,
    pub defender_id: i64,
    pub attacker_troops: Vec<DeployedTroop>,
    pub defender_buildings: HashMap<i64, BuildingInstance>,
    pub defender_resources: ResourceManager,
    pub start_time: f32,
    pub current_time: f32,
    pub duration_limit: f32, // 3 minutes = 180 seconds
    pub state: BattleState,
    pub destruction_percent: f32,
    pub stars_earned: u8,
    pub loot_earned: ResourceCost,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BattleState {
    Preparing,
    InProgress,
    Victory,
    Defeat,
    Timeout,
}

impl Battle {
    pub fn new(attacker_id: i64, defender_id: i64, defender_grid: &VillageGrid, defender_resources: ResourceManager, game_data: &GameData) -> Self {
        let defender_buildings: HashMap<i64, BuildingInstance> = defender_grid.buildings
            .iter()
            .map(|(k, v)| (*k, v.clone()))
            .collect();
        
        Self {
            attacker_id,
            defender_id,
            attacker_troops: Vec::new(),
            defender_buildings,
            defender_resources,
            start_time: 0.0,
            current_time: 0.0,
            duration_limit: 180.0, // 3 minutes
            state: BattleState::Preparing,
            destruction_percent: 0.0,
            stars_earned: 0,
            loot_earned: ResourceCost::default(),
        }
    }
    
    pub fn deploy_troop(&mut self, character_data_id: i32, level: i32, position: Position, game_data: &GameData) -> Result<i64, String> {
        let character = game_data.get_character(character_data_id)
            .ok_or("Character data not found")?;
        
        if level > character.max_level || level < 1 {
            return Err("Invalid troop level".to_string());
        }
        
        let troop_id = self.attacker_troops.len() as i64 + 1;
        
        let character_data = character.get_data_at_level(level);
        
        let troop = DeployedTroop {
            id: troop_id,
            character_data_id,
            level,
            position,
            target: None,
            hitpoints: character_data.hitpoints as f32,
            max_hitpoints: character_data.hitpoints as f32,
            state: TroopState::Deploying,
            last_attack_time: 0.0,
            deploy_time: 0.0,
            path: Vec::new(),
            path_index: 0,
        };
        
        self.attacker_troops.push(troop);
        Ok(troop_id)
    }
    
    pub fn update(&mut self, dt: f32, game_data: &GameData) {
        if self.state != BattleState::InProgress {
            return;
        }
        
        self.current_time += dt;
        
        if self.current_time >= self.duration_limit {
            self.state = BattleState::Timeout;
            self.calculate_loot();
            return;
        }
        
        // Update troops - use index-based loop to avoid borrow issues
        let len = self.attacker_troops.len();
        for i in 0..len {
            if self.attacker_troops[i].state == TroopState::Dead {
                continue;
            }
            
            self.update_troop_by_index(i, dt, game_data);
        }
        
        // Check win conditions
        self.check_victory();
    }
    
fn update_troop_by_index(&mut self, index: usize, dt: f32, _game_data: &GameData) {
        // Handle deploying state
        if self.attacker_troops[index].state == TroopState::Deploying {
            self.attacker_troops[index].deploy_time += dt;
            if self.attacker_troops[index].deploy_time >= 1.0 {
                self.attacker_troops[index].state = TroopState::Moving;
                self.find_target_by_index(index);
            }
            return;
        }
        
        // Find target if none
        if self.attacker_troops[index].target.is_none() {
            self.find_target_by_index(index);
        }
        
        // Move towards target
        if let Some(target) = &self.attacker_troops[index].target {
            let target_pos = self.get_target_position(target);
            if let Some(target_pos) = target_pos {
                let character_data_id = self.attacker_troops[index].character_data_id;
                let character = self.get_character_data(character_data_id);
                if let Some(character) = character {
                    let speed = character.movement_speed;
                    let arrived = self.attacker_troops[index].position.move_towards(&target_pos, speed, dt);
                    
                    if arrived {
                        self.attacker_troops[index].state = TroopState::Attacking;
                    } else {
                        self.attacker_troops[index].state = TroopState::Moving;
                    }
                }
            }
        }
        
        // Attack if in range
        if self.attacker_troops[index].state == TroopState::Attacking {
            if let Some(target) = &self.attacker_troops[index].target {
                let target_pos = self.get_target_position(target);
                if let Some(target_pos) = target_pos {
                    let character_data_id = self.attacker_troops[index].character_data_id;
                    let character = self.get_character_data(character_data_id);
                    
                    if let Some(character) = character {
                        let dist = self.attacker_troops[index].position.distance_to(&target_pos);
                        let range = character.attack_range;
                        if dist <= range {
                            self.attack_by_index(index, &target_pos, dt);
                        } else {
                            self.attacker_troops[index].state = TroopState::Moving;
                        }
                    }
                }
            }
        }
    }
    
    fn find_target_by_index(&mut self, index: usize) {
        let character_data_id = self.attacker_troops[index].character_data_id;
        let character = self.get_character_data(character_data_id);
        if character.is_none() {
            return;
        }
        
        let character = character.unwrap();
        
        // Find valid targets based on troop preference
        let mut candidates = Vec::new();
        
        for (id, building) in &self.defender_buildings {
            if building.state == BuildingState::Destroyed {
                continue;
            }
            
            if let Some(data) = building.get_data(&crate::logic::game_data::GameData::new()) {
                let is_preferred = match character.favorite_target.as_str() {
                    "Any" => true,
                    "Defense" => data.building_type == BuildingType::Defense,
                    "Resource" => data.building_type == BuildingType::ResourceStorage || data.building_type == BuildingType::ResourceCollector,
                    "Wall" => data.building_type == BuildingType::Wall,
                    "Ground" => data.building_type != BuildingType::Wall, // Simplified
                    "Air" => false, // Air buildings don't exist in this simplified model
                    _ => true,
                };
                
                if is_preferred {
                    let center_x = building.position.x as f32 + data.width as f32 / 2.0;
                    let center_y = building.position.y as f32 + data.height as f32 / 2.0;
                    let troop_pos = self.attacker_troops[index].position;
                    let dist = ((troop_pos.x - center_x).powi(2) + (troop_pos.y - center_y).powi(2)).sqrt();
                    candidates.push((dist, building.id));
                }
            }
        }
        
        // Sort by distance
        candidates.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(Ordering::Equal));
        
        if let Some((_, target_id)) = candidates.first() {
            self.attacker_troops[index].target = Some(Target::Building(*target_id));
        }
    }
    
    fn get_target_position(&self, target: &Target) -> Option<Position> {
        match target {
            Target::Building(id) => {
                if let Some(building) = self.defender_buildings.get(id) {
                    if let Some(data) = building.get_data(&crate::logic::game_data::GameData::new()) {
                        let center_x = building.position.x as f32 + data.width as f32 / 2.0;
                        let center_y = building.position.y as f32 + data.height as f32 / 2.0;
                        Some(Position::new(center_x, center_y))
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            Target::Wall(pos) => Some(Position::new(pos.x as f32 + 0.5, pos.y as f32 + 0.5)),
            Target::None => None,
        }
    }
    
    fn attack(&mut self, troop: &mut DeployedTroop, target_pos: &Position, dt: f32) {
        let character = self.get_character_data(troop.character_data_id);
        if character.is_none() {
            return;
        }
        
        let character = character.unwrap();
        let now = self.current_time;
        
        if now - troop.last_attack_time >= 1.0 / character.attack_speed {
            // Find target building
            if let Some(Target::Building(target_id)) = &troop.target {
                if let Some(building) = self.defender_buildings.get_mut(target_id) {
                    let dps = character.damage_per_second as f32;
                    let damage = dps * dt;
                    
                    // Apply damage
                    building.hitpoints = (building.hitpoints as f32 - damage).max(0.0) as i32;
                    
                    if building.hitpoints <= 0 {
                        building.state = BuildingState::Destroyed;
                    }
                    
                    troop.last_attack_time = now;
                }
            }
        }
    }
    
    fn attack_by_index(&mut self, index: usize, target_pos: &Position, dt: f32) {
        let character_data_id = self.attacker_troops[index].character_data_id;
        let character = self.get_character_data(character_data_id);
        if character.is_none() {
            return;
        }
        
        let character = character.unwrap();
        let now = self.current_time;
        
        if self.current_time - self.attacker_troops[index].last_attack_time >= 1.0 / character.attack_speed {
            // Find target building
            if let Some(Target::Building(target_id)) = &self.attacker_troops[index].target {
                if let Some(building) = self.defender_buildings.get_mut(target_id) {
                    let dps = character.damage_per_second as f32;
                    let damage = dps * dt;
                    
                    // Apply damage
                    building.hitpoints = (building.hitpoints as f32 - damage).max(0.0) as i32;
                    
                    if building.hitpoints <= 0 {
                        building.state = BuildingState::Destroyed;
                    }
                    
                    self.attacker_troops[index].last_attack_time = now;
                }
            }
        }
    }
    
    fn check_victory(&mut self) {
        let total_buildings = self.defender_buildings.len();
        let destroyed = self.defender_buildings.values().filter(|b| b.state == BuildingState::Destroyed).count();
        
        self.destruction_percent = if total_buildings > 0 {
            destroyed as f32 / total_buildings as f32 * 100.0
        } else {
            100.0
        };
        
        // Check for town hall destruction
        let townhall_destroyed = self.defender_buildings.values()
            .any(|b| b.get_data(&crate::logic::game_data::GameData::new())
                .map(|d| d.building_type == BuildingType::TownHall && b.state == BuildingState::Destroyed)
                .unwrap_or(false));
        
        // Calculate stars
        if townhall_destroyed && self.destruction_percent >= 50.0 {
            self.stars_earned = 3;
            self.state = BattleState::Victory;
        } else if townhall_destroyed {
            self.stars_earned = 1;
            self.state = BattleState::Victory;
        } else if self.destruction_percent >= 50.0 {
            self.stars_earned = 1;
            self.state = BattleState::Victory;
        } else if self.state == BattleState::Timeout {
            self.stars_earned = 0;
            self.state = BattleState::Defeat;
        }
        
        if self.state == BattleState::Victory {
            self.calculate_loot();
        }
    }
    
    fn calculate_loot(&mut self) {
        // Calculate loot based on remaining resources
        // Simplified
        self.loot_earned = ResourceCost::default();
    }
    
    fn get_character_data(&self, character_data_id: i32) -> Option<CharacterDataSnapshot> {
        // Would need access to GameData
        None
    }
}

/// Helper struct for character data at specific level
#[derive(Debug, Clone)]
pub struct CharacterDataSnapshot {
    pub hitpoints: i32,
    pub damage_per_second: i32,
    pub movement_speed: f32,
    pub attack_speed: f32,
    pub range: f32,
    pub attack_range: f32,
    pub favorite_target: String,
    pub damage_type: String,
    pub is_super_troop: bool,
}

/// Pathfinding for troop movement (A*)
pub fn find_path(start: GridPos, goal: GridPos, grid: &VillageGrid, _game_data: &GameData) -> Vec<GridPos> {
    // A* pathfinding
    let mut open_set = VecDeque::new();
    let mut came_from = HashMap::new();
    let mut g_score = HashMap::new();
    let mut f_score = HashMap::new();
    
    open_set.push_back(start);
    g_score.insert(start, 0);
    f_score.insert(start, heuristic(start, goal));
    
    while let Some(current) = open_set.pop_front() {
        if current == goal {
            return reconstruct_path(came_from, current);
        }
        
        for neighbor in get_neighbors(current, grid) {
            let tentative_g = g_score.get(&current).unwrap_or(&0) + 1;
            
            if tentative_g < *g_score.get(&neighbor).unwrap_or(&i32::MAX) {
                came_from.insert(neighbor, current);
                g_score.insert(neighbor, tentative_g);
                f_score.insert(neighbor, tentative_g + heuristic(neighbor, goal));
                
                if !open_set.contains(&neighbor) {
                    open_set.push_back(neighbor);
                }
            }
        }
    }
    
    // No path found
    Vec::new()
}
    
    fn heuristic(a: GridPos, b: GridPos) -> i32 {
        (a.x - b.x).abs() + (a.y - b.y).abs()
    }
    
    fn get_neighbors(pos: GridPos, grid: &VillageGrid) -> Vec<GridPos> {
        let mut neighbors = Vec::new();
        let dirs = [(0, 1), (1, 0), (0, -1), (-1, 0)];
        
        for (dx, dy) in dirs {
            let nx = pos.x + dx;
            let ny = pos.y + dy;
            
            if nx >= 0 && nx < grid.width && ny >= 0 && ny < grid.height {
                let pos = GridPos::new(nx, ny);
                if !grid.tiles.get(&pos).map(|t| t.occupied).unwrap_or(false) {
                    neighbors.push(pos);
                }
            }
        }
        neighbors
    }
    
    fn reconstruct_path(came_from: HashMap<GridPos, GridPos>, current: GridPos) -> Vec<GridPos> {
        let mut path = vec![current];
        let mut current = current;
        
        while let Some(prev) = came_from.get(&current) {
            path.push(*prev);
            current = *prev;
        }
        
        path.reverse();
        path
    }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logic::game_data::*;
    
    #[test]
    fn test_position_movement() {
        let mut pos = Position::new(0.0, 0.0);
        let target = Position::new(10.0, 0.0);
        
        let arrived = pos.move_towards(&target, 5.0, 1.0);
        assert!(!arrived);
        assert!((pos.x - 5.0).abs() < 0.01);
        
        let arrived = pos.move_towards(&target, 5.0, 1.0);
        assert!(arrived);
        assert!((pos.x - 10.0).abs() < 0.01);
    }
}