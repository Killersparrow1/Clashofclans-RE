//! Resource Management
//! 
//! Handles resource production, storage, spending, and looting.

use super::game_data::*;
use super::buildings::BuildingInstance;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// Resource types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceType {
    Gold,
    Elixir,
    DarkElixir,
    BuilderGold,
    BuilderElixir,
    Gems,
    StarBonus,
}

/// Resource storage with capacity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceStorage {
    pub amount: i64,
    pub capacity: i64,
}

impl ResourceStorage {
    pub fn new(capacity: i64) -> Self {
        Self { amount: 0, capacity }
    }
    
    pub fn add(&mut self, amount: i64) -> i64 {
        let added = (self.amount + amount).min(self.capacity) - self.amount;
        self.amount += added;
        added
    }
    
    pub fn spend(&mut self, amount: i64) -> bool {
        if self.amount >= amount {
            self.amount -= amount;
            true
        } else {
            false
        }
    }
    
    pub fn can_afford(&self, amount: i64) -> bool {
        self.amount >= amount
    }
    
    pub fn fill_ratio(&self) -> f64 {
        if self.capacity == 0 {
            0.0
        } else {
            self.amount as f64 / self.capacity as f64
        }
    }
}

/// Complete resource manager for a village
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceManager {
    pub gold: ResourceStorage,
    pub elixir: ResourceStorage,
    pub dark_elixir: ResourceStorage,
    pub builder_gold: ResourceStorage,
    pub builder_elixir: ResourceStorage,
    pub gems: i32,
    pub star_bonus: i32,
    
    // Production rates (per second)
    pub gold_rate: f64,
    pub elixir_rate: f64,
    pub dark_elixir_rate: f64,
    pub builder_gold_rate: f64,
    pub builder_elixir_rate: f64,
    
    // Last update timestamp
    pub last_update: i64,
}

impl ResourceManager {
    pub fn new() -> Self {
        Self {
            gold: ResourceStorage::new(1000),
            elixir: ResourceStorage::new(1000),
            dark_elixir: ResourceStorage::new(0),
            builder_gold: ResourceStorage::new(0),
            builder_elixir: ResourceStorage::new(0),
            gems: 0,
            star_bonus: 0,
            gold_rate: 0.0,
            elixir_rate: 0.0,
            dark_elixir_rate: 0.0,
            builder_gold_rate: 0.0,
            builder_elixir_rate: 0.0,
            last_update: chrono::Utc::now().timestamp(),
        }
    }
    
    pub fn update(&mut self, current_time: i64) {
        let dt = (current_time - self.last_update) as f64;
        
        self.gold.amount = (self.gold.amount as f64 + self.gold_rate * dt).min(self.gold.capacity as f64) as i64;
        self.elixir.amount = (self.elixir.amount as f64 + self.elixir_rate * dt).min(self.elixir.capacity as f64) as i64;
        self.dark_elixir.amount = (self.dark_elixir.amount as f64 + self.dark_elixir_rate * dt).min(self.dark_elixir.capacity as f64) as i64;
        self.builder_gold.amount = (self.builder_gold.amount as f64 + self.builder_gold_rate * dt).min(self.builder_gold.capacity as f64) as i64;
        self.builder_elixir.amount = (self.builder_elixir.amount as f64 + self.builder_elixir_rate * dt).min(self.builder_elixir.capacity as f64) as i64;
        
        self.last_update = chrono::Utc::now().timestamp();
    }
    
    pub fn add_gold(&mut self, amount: i64) -> i64 {
        self.gold.add(amount)
    }
    
    pub fn add_elixir(&mut self, amount: i64) -> i64 {
        self.elixir.add(amount)
    }
    
    pub fn add_dark_elixir(&mut self, amount: i64) -> i64 {
        self.dark_elixir.add(amount)
    }
    
    pub fn add_builder_gold(&mut self, amount: i64) -> i64 {
        self.builder_gold.add(amount)
    }
    
    pub fn add_builder_elixir(&mut self, amount: i64) -> i64 {
        self.builder_elixir.add(amount)
    }
    
    pub fn add_gems(&mut self, amount: i32) {
        self.gems += amount;
    }
    
    pub fn spend_gold(&mut self, amount: i64) -> bool {
        self.gold.spend(amount)
    }
    
    pub fn spend_elixir(&mut self, amount: i64) -> bool {
        self.elixir.spend(amount)
    }
    
    pub fn spend_dark_elixir(&mut self, amount: i64) -> bool {
        self.dark_elixir.spend(amount)
    }
    
    pub fn spend_builder_gold(&mut self, amount: i64) -> bool {
        self.builder_gold.spend(amount)
    }
    
    pub fn spend_builder_elixir(&mut self, amount: i64) -> bool {
        self.builder_elixir.spend(amount)
    }
    
    pub fn spend_gems(&mut self, amount: i32) -> bool {
        if self.gems >= amount {
            self.gems -= amount;
            true
        } else {
            false
        }
    }
    
    pub fn can_afford(&self, cost: &ResourceCost) -> bool {
        self.gold.can_afford(cost.gold) &&
        self.elixir.can_afford(cost.elixir) &&
        self.dark_elixir.can_afford(cost.dark_elixir) &&
        self.builder_gold.can_afford(cost.builder_gold) &&
        self.builder_elixir.can_afford(cost.builder_elixir)
    }
    
    pub fn spend_cost(&mut self, cost: &ResourceCost) -> bool {
        if !self.can_afford(cost) {
            return false;
        }
        self.spend_gold(cost.gold);
        self.spend_elixir(cost.elixir);
        self.spend_dark_elixir(cost.dark_elixir);
        self.spend_builder_gold(cost.builder_gold);
        self.spend_builder_elixir(cost.builder_elixir);
        true
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
                        let rate = data.production_rate[building.level as usize - 1] as f64;
                        // Determine resource type from sub_type or building_data_id
                        // This is simplified - real implementation would check sub_type
                        gold_rate += rate;
                    }
                }
            }
        }
        
        self.gold_rate = gold_rate;
        self.elixir_rate = elixir_rate;
        self.dark_elixir_rate = dark_elixir_rate;
    }
    
    /// Calculate loot available for raiding (percentage of stored resources)
    pub fn calculate_loot(&self, townhall_level: i32) -> ResourceCost {
        let loot_percent = self.get_loot_percentage(townhall_level);
        
        ResourceCost {
            gold: (self.gold.amount as f64 * loot_percent) as i64,
            elixir: (self.elixir.amount as f64 * loot_percent) as i64,
            dark_elixir: (self.dark_elixir.amount as f64 * loot_percent) as i64,
            builder_gold: (self.builder_gold.amount as f64 * loot_percent) as i64,
            builder_elixir: (self.builder_elixir.amount as f64 * loot_percent) as i64,
            time_seconds: 0,
        }
    }
    
    /// Loot percentage based on townhall level difference (CoC formula)
    fn get_loot_percentage(&self, attacker_th: i32) -> f64 {
        // Simplified - real CoC uses complex formula based on TH difference
        // This is a placeholder
        0.2
    }
    
    /// Get resource for loot calculation
    pub fn get_resource(&self, resource_type: ResourceType) -> &ResourceStorage {
        match resource_type {
            ResourceType::Gold => &self.gold,
            ResourceType::Elixir => &self.elixir,
            ResourceType::DarkElixir => &self.dark_elixir,
            ResourceType::BuilderGold => &self.builder_gold,
            ResourceType::BuilderElixir => &self.builder_elixir,
            ResourceType::Gems => panic!("Gems not stored in ResourceStorage"),
            ResourceType::StarBonus => panic!("Star bonus not stored in ResourceStorage"),
        }
    }
    
    pub fn get_resource_mut(&mut self, resource_type: ResourceType) -> &mut ResourceStorage {
        match resource_type {
            ResourceType::Gold => &mut self.gold,
            ResourceType::Elixir => &mut self.elixir,
            ResourceType::DarkElixir => &mut self.dark_elixir,
            ResourceType::BuilderGold => &mut self.builder_gold,
            ResourceType::BuilderElixir => &mut self.builder_elixir,
            ResourceType::Gems => panic!("Gems not stored in ResourceStorage"),
            ResourceType::StarBonus => panic!("Star bonus not stored in ResourceStorage"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_resource_storage() {
        let mut storage = ResourceStorage::new(100);
        assert_eq!(storage.add(50), 50);
        assert_eq!(storage.amount, 50);
        assert_eq!(storage.add(60), 50); // Capped at capacity
        assert_eq!(storage.amount, 100);
        assert!(storage.spend(30));
        assert_eq!(storage.amount, 70);
        assert!(!storage.spend(100));
    }
    
    #[test]
    fn test_resource_manager() {
        let mut mgr = ResourceManager::new();
        mgr.gold.capacity = 1000;
        mgr.gold.amount = 500;
        mgr.elixir.capacity = 1000;
        mgr.elixir.amount = 500;
        
        let cost = ResourceCost { gold: 100, elixir: 100, dark_elixir: 0, builder_gold: 0, builder_elixir: 0, time_seconds: 0 };
        assert!(mgr.can_afford(&cost));
        assert!(mgr.spend_cost(&cost));
        assert_eq!(mgr.gold.amount, 400);
    }
}

pub use super::game_data::ResourceCost;