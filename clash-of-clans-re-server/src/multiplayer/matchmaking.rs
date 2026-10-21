//! Matchmaking System
//! 
//! Handles trophy-based matchmaking for battles and clan wars.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;
use crate::logic::game_data::*;

/// Matchmaking criteria for finding opponents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchmakingCriteria {
    pub player_id: i64,
    pub trophies: i32,
    pub townhall_level: i32,
    pub min_trophy_diff: i32,
    pub max_trophy_diff: i32,
    pub max_townhall_diff: i32,
    pub max_wait_time: u32, // seconds
    pub allow_higher_th: bool,
    pub allow_lower_th: bool,
}

impl MatchmakingCriteria {
    pub fn new(player_id: i64, trophies: i32, townhall_level: i32) -> Self {
        Self {
            player_id,
            trophies,
            townhall_level,
            min_trophy_diff: -200,
            max_trophy_diff: 200,
            max_townhall_diff: 2,
            max_wait_time: 300, // 5 minutes
            allow_higher_th: true,
            allow_lower_th: true,
        }
    }
    
    pub fn with_custom_range(mut self, min_diff: i32, max_diff: i32, max_th_diff: i32) -> Self {
        self.min_trophy_diff = min_diff;
        self.max_trophy_diff = max_diff;
        self.max_townhall_diff = max_th_diff;
        self
    }
    
    pub fn with_max_wait(mut self, seconds: u32) -> Self {
        self.max_wait_time = seconds;
        self
    }
}

/// Matchmaking pool entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolEntry {
    pub player_id: i64,
    pub trophies: i32,
    pub townhall_level: i32,
    pub criteria: MatchmakingCriteria,
    pub queued_at: i64,
    pub attempts: u32,
}

impl PoolEntry {
    pub fn new(criteria: MatchmakingCriteria) -> Self {
        Self {
            player_id: criteria.player_id,
            trophies: criteria.trophies,
            townhall_level: criteria.townhall_level,
            criteria,
            queued_at: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64,
            attempts: 0,
        }
    }
    
    pub fn is_expired(&self, max_wait: u32) -> bool {
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64;
        (now - self.queued_at) > max_wait as i64
    }
    
    pub fn is_compatible(&self, other: &PoolEntry) -> bool {
        let trophy_diff = (self.trophies - other.trophies).abs();
        let th_diff = (self.townhall_level as i32 - other.townhall_level as i32).abs();
        
        trophy_diff >= self.criteria.min_trophy_diff &&  // min_trophy_diff is negative, so this is always true for positive diff
        trophy_diff <= self.criteria.max_trophy_diff as i32 &&
        th_diff <= self.criteria.max_townhall_diff
    }
}

/// Match result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchResult {
    pub match_id: String,
    pub attacker_id: i64,
    pub defender_id: i64,
    pub attacker_trophies: i32,
    pub defender_trophies: i32,
    pub trophy_change: i32,
    pub timestamp: i64,
}

/// Matchmaking pool for a specific trophy range
pub struct MatchmakingPool {
    trophy_range: (i32, i32),
    queue: VecDeque<PoolEntry>,
    matches: Vec<MatchResult>,
}

impl MatchmakingPool {
    fn new(min_trophies: i32, max_trophies: i32) -> Self {
        Self {
            trophy_range: (min_trophies, max_trophies),
            queue: VecDeque::new(),
            matches: Vec::new(),
        }
    }
    
    fn add_to_queue(&mut self, entry: PoolEntry) {
        self.queue.push_back(entry);
    }
    
    fn find_match(&mut self, entry: &PoolEntry) -> Option<PoolEntry> {
        let mut best_match = None;
        let mut best_score = i32::MAX;
        
        for (i, candidate) in self.queue.iter().enumerate() {
            if candidate.player_id == entry.player_id {
                continue;
            }
            if !entry.is_compatible(candidate) {
                continue;
            }
            
            let score = (entry.trophies - candidate.trophies).abs();
            if score < best_score {
                best_score = score;
                best_match = Some((i, candidate.clone()));
            }
        }
        
        if let Some((index, matched)) = best_match {
            self.queue.remove(index);
            Some(matched)
        } else {
            None
        }
    }
    
    fn cleanup_expired(&mut self, max_wait: u32) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        
        self.queue.retain(|e| (now - e.queued_at) < max_wait as i64);
    }
}

/// Main matchmaker
pub struct Matchmaker {
    pools: HashMap<(i32, i32), MatchmakingPool>,
    pending_matches: HashMap<String, MatchResult>,
    config: MatchmakerConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchmakerConfig {
    pub trophy_brackets: Vec<(i32, i32)>, // (min, max) trophy ranges
    pub max_trophy_diff: i32,
    pub max_townhall_diff: i32,
    pub max_wait_time: u32,
    pub min_trophy_diff: i32,
}

impl Default for MatchmakerConfig {
    fn default() -> Self {
        Self {
            trophy_brackets: vec![
                (0, 400),
                (400, 800),
                (800, 1400),
                (1400, 2000),
                (2000, 3000),
                (3000, 4000),
                (4000, 5000),
                (5000, 6000),
                (6000, 10000),
            ],
            max_trophy_diff: 200,
            max_townhall_diff: 2,
            max_wait_time: 300,
            min_trophy_diff: -200,
        }
    }
}

impl Matchmaker {
    pub fn new(config: MatchmakerConfig) -> Self {
        let mut pools = HashMap::new();
        for (min, max) in &config.trophy_brackets {
            pools.insert((*min, *max), MatchmakingPool::new(*min, *max));
        }
        
        Self {
            pools,
            pending_matches: HashMap::new(),
            config,
        }
    }
    
    pub fn queue_player(&mut self, criteria: MatchmakingCriteria) -> Result<String, String> {
        let entry = PoolEntry::new(criteria.clone());
        
        // Find appropriate pool
        let pool_key = self.find_pool(entry.trophies)
            .ok_or("No pool found for trophy range")?;
        let pool = self.pools.get_mut(&pool_key)
            .ok_or("No pool found for trophy range")?;
        
        // Try to find immediate match
        let entry_copy = PoolEntry::new(criteria.clone());
        
        if let Some(matched) = pool.queue.iter().find(|e| e.is_compatible(&entry_copy)) {
            // Found match!
            let match_id = Uuid::new_v4().to_string();
            let match_result = MatchResult {
                match_id: Uuid::new_v4().to_string(),
                attacker_id: criteria.player_id,
                defender_id: matched.player_id,
                attacker_trophies: criteria.trophies,
                defender_trophies: matched.trophies,
                trophy_change: 0,
                timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64,
            };
            
            // Store match result
            let match_id = Uuid::new_v4().to_string();
            self.pending_matches.insert(match_id.clone(), MatchResult {
                match_id: match_id.clone(),
                attacker_id: criteria.player_id,
                defender_id: matched.player_id,
                attacker_trophies: criteria.trophies,
                defender_trophies: matched.trophies,
                trophy_change: 0,
                timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64,
            });
            
            return Ok(match_id);
        }
        
        // No immediate match, add to queue
        pool.add_to_queue(PoolEntry::new(MatchmakingCriteria::new(
            criteria.player_id,
            criteria.trophies,
            criteria.townhall_level
        )));
        
        // Start background matching process
        // In a real implementation, this would be a background task
        Ok(Uuid::new_v4().to_string())
    }
    
    pub fn cancel_queue(&mut self, player_id: i64) -> Result<(), String> {
        for pool in self.pools.values_mut() {
            if let Some(pos) = pool.queue.iter().position(|e| e.player_id == player_id) {
                pool.queue.remove(pos);
                return Ok(());
            }
        }
        Err("Player not in queue".to_string())
    }
    
    pub fn get_match_result(&self, match_id: &str) -> Option<&MatchResult> {
        self.pending_matches.get(match_id)
    }
    
    pub fn process_queues(&mut self) -> Vec<MatchResult> {
        let mut completed = Vec::new();
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64;
        
        for pool in self.pools.values_mut() {
            pool.cleanup_expired(self.config.max_wait_time);
            
            // Try to match entries
            let mut i = 0;
            while i < pool.queue.len() {
                let entry = pool.queue[i].clone();
                if let Some(matched) = pool.find_match(&entry) {
                    // Create match
                    let match_result = MatchResult {
                        match_id: Uuid::new_v4().to_string(),
                        attacker_id: entry.player_id,
                        defender_id: matched.player_id,
                        attacker_trophies: entry.trophies,
                        defender_trophies: matched.trophies,
                        trophy_change: 0,
                        timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64,
                    };
                    
                    let match_id = Uuid::new_v4().to_string();
                    self.pending_matches.insert(match_id.clone(), MatchResult {
                        match_id: match_id.clone(),
                        attacker_id: entry.player_id,
                        defender_id: matched.player_id,
                        attacker_trophies: entry.trophies,
                        defender_trophies: matched.trophies,
                        trophy_change: 0,
                        timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64,
                    });
                    
                    completed.push(MatchResult {
                        match_id: match_id.clone(),
                        attacker_id: entry.player_id,
                        defender_id: matched.player_id,
                        attacker_trophies: entry.trophies,
                        defender_trophies: matched.trophies,
                        trophy_change: 0,
                        timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64,
                    });
                    
                    // Remove both from queue
                    let idx1 = pool.queue.iter().position(|e| e.player_id == entry.player_id);
                    let idx2 = pool.queue.iter().position(|e| e.player_id == matched.player_id);
                    if let (Some(i1), Some(i2)) = (idx1, idx2) {
                        let max_idx = i1.max(i2);
                        let min_idx = i1.min(i2);
                        pool.queue.remove(max_idx);
                        pool.queue.remove(min_idx);
                    }
                    continue;
                }
                i += 1;
            }
        }
        completed
    }
    
    fn find_pool(&self, trophies: i32) -> Option<(i32, i32)> {
        for (min, max) in &self.config.trophy_brackets {
            if trophies >= *min && trophies <= *max {
                return Some((*min, *max));
            }
        }
        // Fallback to closest bracket
        let mut best = None;
        let mut best_diff = i32::MAX;
        for (min, max) in &self.config.trophy_brackets {
            let mid = (min + max) / 2;
            let diff = (trophies - mid).abs();
            if diff < best_diff {
                best_diff = diff;
                best = Some((*min, *max));
            }
        }
        best
    }
    
    pub fn get_queue_stats(&self) -> HashMap<String, usize> {
        let mut stats = HashMap::new();
        for ((min, max), pool) in &self.pools {
            stats.insert(format!("{}-{}", min, max), pool.queue.len());
        }
        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_matchmaking_criteria() {
        let criteria = MatchmakingCriteria::new(1, 1500, 10);
        assert_eq!(criteria.trophies, 1500);
        assert_eq!(criteria.townhall_level, 10);
    }
    
    #[test]
    fn test_pool_entry_compatibility() {
        let criteria1 = MatchmakingCriteria::new(1, 1500, 10);
        let criteria2 = MatchmakingCriteria::new(2, 1600, 11);
        
        let entry1 = PoolEntry::new(criteria1);
        let entry2 = PoolEntry::new(criteria2);
        
        assert!(entry1.is_compatible(&entry2));
    }
}