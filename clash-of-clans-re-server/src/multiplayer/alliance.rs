//! Alliance System
//! 
//! Handles clan/alliance management including chat, donations, and wars.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;
use crate::logic::game_data::CharacterType;
use crate::logic::resources::{ResourceManager, ResourceType};

/// Alliance role hierarchy
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum AllianceRole {
    Member = 0,
    Elder = 1,
    CoLeader = 2,
    Leader = 3,
}

impl AllianceRole {
    pub fn can_kick(&self, target: AllianceRole) -> bool {
        *self > target
    }
    
    pub fn can_promote(&self, target: AllianceRole) -> bool {
        *self > target
    }
    
    pub fn can_demote(&self, target: AllianceRole) -> bool {
        *self > target
    }
    
    pub fn can_start_war(&self) -> bool {
        matches!(self, AllianceRole::Leader | AllianceRole::CoLeader)
    }
    
    pub fn can_manage_settings(&self) -> bool {
        matches!(self, AllianceRole::Leader | AllianceRole::CoLeader)
    }
    
    pub fn can_accept_members(&self) -> bool {
        matches!(self, AllianceRole::Leader | AllianceRole::CoLeader | AllianceRole::Elder)
    }
    
    pub fn name(&self) -> &'static str {
        match self {
            AllianceRole::Member => "Member",
            AllianceRole::Elder => "Elder",
            AllianceRole::CoLeader => "Co-Leader",
            AllianceRole::Leader => "Leader",
        }
    }
}

/// Alliance member
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllianceMember {
    pub player_id: i64,
    pub name: String,
    pub role: AllianceRole,
    pub townhall_level: i32,
    pub trophies: i32,
    pub donations_given: i64,
    pub donations_received: i64,
    pub war_stars: i32,
    pub joined_at: i64,
    pub last_online: i64,
}

impl AllianceMember {
    pub fn new(player_id: i64, name: String, townhall_level: i32, trophies: i32) -> Self {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        Self {
            player_id,
            name,
            role: AllianceRole::Member,
            townhall_level,
            trophies,
            donations_given: 0,
            donations_received: 0,
            war_stars: 0,
            joined_at: now as i64,
            last_online: now as i64,
        }
    }
    
    pub fn can_donate(&self) -> bool {
        true // All members can donate
    }
    
    pub fn can_request(&self) -> bool {
        true // All members can request
    }
    
    pub fn get_donation_limit(&self) -> i32 {
        match self.role {
            AllianceRole::Leader | AllianceRole::CoLeader => 10,
            AllianceRole::Elder => 8,
            AllianceRole::Member => 6,
        }
    }
}

/// Alliance chat message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllianceChatMessage {
    pub id: String,
    pub player_id: i64,
    pub player_name: String,
    pub player_role: AllianceRole,
    pub message: String,
    pub timestamp: i64,
    pub message_type: ChatMessageType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChatMessageType {
    Normal,
    DonationRequest,
    DonationReceived,
    WarDeclaration,
    WarResult,
    Promotion,
    Demotion,
    Kick,
    Join,
    Leave,
    System,
}

/// Alliance donation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllianceDonation {
    pub id: String,
    pub requester_id: i64,
    pub requester_name: String,
    pub troop_type: CharacterType,
    pub troop_level: i32,
    pub count: i32,
    pub timestamp: i64,
    pub fulfilled: bool,
    pub fulfiller_id: Option<i64>,
    pub fulfiller_name: Option<String>,
}

impl AllianceDonation {
    pub fn new(requester_id: i64, requester_name: String, troop_type: CharacterType, troop_level: i32, count: i32) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            requester_id,
            requester_name,
            troop_type,
            troop_level,
            count,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64,
            fulfilled: false,
            fulfiller_id: None,
            fulfiller_name: None,
        }
    }
}

/// Alliance war state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AllianceWarState {
    NotInWar,
    Preparation,
    InWar,
    WarEnded,
}

/// War participant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WarParticipant {
    pub player_id: i64,
    pub player_name: String,
    pub townhall_level: i32,
    pub map_position: i32,
    pub stars_earned: u32,
    pub attacks_used: u8,
    pub attacks_available: u8,
    pub destruction_percent: f32,
    pub best_attack_destruction: f32,
}

impl WarParticipant {
    pub fn new(player_id: i64, name: String, townhall_level: i32, position: i32) -> Self {
        Self {
            player_id,
            player_name: name,
            townhall_level,
            map_position: position,
            stars_earned: 0,
            attacks_used: 0,
            attacks_available: 2,
            destruction_percent: 0.0,
            best_attack_destruction: 0.0,
        }
    }
}

/// Alliance war
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllianceWar {
    pub id: String,
    pub attacker_alliance_id: i64,
    pub defender_alliance_id: i64,
    pub state: AllianceWarState,
    pub preparation_start: i64,
    pub war_start: i64,
    pub war_end: i64,
    pub attacker_participants: Vec<WarParticipant>,
    pub defender_participants: Vec<WarParticipant>,
    pub attacker_stars: u32,
    pub defender_stars: u32,
    pub attacker_destruction: f32,
    pub defender_destruction: f32,
    pub attacker_attacks: u32,
    pub defender_attacks: u32,
}

/// Alliance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alliance {
    pub id: i64,
    pub name: String,
    pub tag: String,
    pub description: String,
    pub badge_id: i32,
    pub required_trophies: i32,
    pub war_frequency: WarFrequency,
    pub location_id: i32,
    pub is_open: bool,
    pub min_townhall_level: i32,
    pub members: HashMap<i64, AllianceMember>,
    pub chat: Vec<AllianceChatMessage>,
    pub donation_requests: Vec<AllianceDonation>,
    pub war_log: Vec<AllianceWar>,
    pub current_war: Option<AllianceWar>,
    pub total_donations: i64,
    pub total_members: i32,
    pub total_trophies: i64,
    pub war_wins: u32,
    pub war_losses: u32,
    pub war_draws: u32,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WarFrequency {
    Always,
    OnceAWeek,
    TwiceAWeek,
    Never,
}

impl Alliance {
    pub fn new(id: i64, name: String, tag: String, leader_id: i64, leader_name: String, leader_th: i32, leader_trophies: i32) -> Self {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;
        let mut members = HashMap::new();
        let leader = AllianceMember::new(leader_id, leader_name, 1, 0);
        let mut leader = leader;
        leader.role = AllianceRole::Leader;
        leader.townhall_level = leader_th;
        leader.trophies = leader_trophies;
        members.insert(leader_id, leader);
        
        Self {
            id: 0, // Will be set by database
            name,
            tag: tag.to_uppercase(),
            description: String::new(),
            badge_id: 0,
            required_trophies: 0,
            war_frequency: WarFrequency::Always,
            location_id: 0,
            is_open: true,
            min_townhall_level: 1,
            members,
            chat: Vec::new(),
            donation_requests: Vec::new(),
            war_log: Vec::new(),
            current_war: None,
            total_donations: 0,
            total_members: 1,
            total_trophies: leader_trophies as i64,
            war_wins: 0,
            war_losses: 0,
            war_draws: 0,
            created_at: now,
            updated_at: now,
        }
    }
    
    pub fn add_member(&mut self, member: AllianceMember) -> Result<(), String> {
        if self.members.len() >= 50 {
            return Err("Alliance is full".to_string());
        }
        if self.members.contains_key(&member.player_id) {
            return Err("Player already in alliance".to_string());
        }
        if member.trophies < self.required_trophies {
            return Err(format!("Requires {} trophies", self.required_trophies));
        }
        if member.townhall_level < self.min_townhall_level {
            return Err(format!("Requires Town Hall level {}", self.min_townhall_level));
        }
        
        self.total_trophies += member.trophies as i64;
        self.total_members += 1;
        self.members.insert(member.player_id, member);
        self.updated_at = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;
        Ok(())
    }
    
    pub fn remove_member(&mut self, player_id: i64, requester_role: AllianceRole) -> Result<AllianceMember, String> {
        let member = self.members.remove(&player_id)
            .ok_or("Member not found")?;
        
        if requester_role <= member.role && player_id != requester_role as i64 {
            return Err("Insufficient permissions".to_string());
        }
        
        self.total_trophies -= member.trophies as i64;
        self.total_members -= 1;
        self.updated_at = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;
        Ok(member)
    }
    
    pub fn promote_member(&mut self, player_id: i64, new_role: AllianceRole, requester_role: AllianceRole) -> Result<(), String> {
        let member = self.members.get_mut(&player_id)
            .ok_or("Member not found")?;
        
        if !requester_role.can_promote(member.role) {
            return Err("Insufficient permissions".to_string());
        }
        if !requester_role.can_promote(new_role) {
            return Err("Cannot promote to that role".to_string());
        }
        if new_role <= member.role {
            return Err("Must promote to higher role".to_string());
        }
        
        member.role = new_role;
        self.updated_at = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;
        Ok(())
    }
    
    pub fn demote_member(&mut self, player_id: i64, new_role: AllianceRole, requester_role: AllianceRole) -> Result<(), String> {
        let member = self.members.get_mut(&player_id)
            .ok_or("Member not found")?;
        
        if !requester_role.can_demote(member.role) {
            return Err("Insufficient permissions".to_string());
        }
        if new_role >= member.role {
            return Err("Must demote to lower role".to_string());
        }
        
        member.role = new_role;
        self.updated_at = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;
        Ok(())
    }
    
    pub fn send_chat(&mut self, player_id: i64, message: String, msg_type: ChatMessageType) -> Result<(), String> {
        let member = self.members.get(&player_id)
            .ok_or("Player not in alliance")?;
        
        if message.len() > 200 {
            return Err("Message too long".to_string());
        }
        
        let msg = AllianceChatMessage {
            id: Uuid::new_v4().to_string(),
            player_id,
            player_name: member.name.clone(),
            player_role: member.role,
            message,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64,
            message_type: msg_type,
        };
        
        self.chat.push(msg);
        if self.chat.len() > 100 {
            self.chat.remove(0);
        }
        
        Ok(())
    }
    
    pub fn request_donation(&mut self, requester_id: i64, troop_type: CharacterType, troop_level: i32, count: i32) -> Result<(), String> {
        let member = self.members.get(&requester_id)
            .ok_or("Player not in alliance")?;
        
        if count > member.get_donation_limit() {
            return Err(format!("Can request max {} troops", member.get_donation_limit()));
        }
        
        let donation = AllianceDonation::new(requester_id, member.name.clone(), troop_type, troop_level, count);
        self.donation_requests.push(donation);
        Ok(())
    }
    
    pub fn fulfill_donation(&mut self, donation_id: &str, fulfiller_id: i64) -> Result<(), String> {
        let donation = self.donation_requests.iter_mut()
            .find(|d| d.id == donation_id)
            .ok_or("Donation request not found")?;
        
        if donation.fulfilled {
            return Err("Already fulfilled".to_string());
        }
        
        let fulfiller = self.members.get(&fulfiller_id)
            .ok_or("Fulfiller not in alliance")?;
        
        if fulfiller_id == donation.requester_id {
            return Err("Cannot donate to yourself".to_string());
        }
        
        donation.fulfilled = true;
        donation.fulfiller_id = Some(fulfiller_id);
        donation.fulfiller_name = Some(fulfiller.name.clone());
        
        // Update stats
        if let Some(requester) = self.members.get_mut(&donation.requester_id) {
            requester.donations_received += donation.count as i64;
        }
        if let Some(fulfiller) = self.members.get_mut(&fulfiller_id) {
            fulfiller.donations_given += donation.count as i64;
        }
        
        self.total_donations += 1;
        Ok(())
    }
    
    pub fn start_war(&mut self, opponent_id: i64, requester_role: AllianceRole) -> Result<AllianceWar, String> {
        if !requester_role.can_start_war() {
            return Err("Insufficient permissions".to_string());
        }
        
        if self.current_war.is_some() {
            return Err("Already in a war".to_string());
        }
        
        // In a real implementation, this would matchmake with another alliance
        let war = AllianceWar {
            id: Uuid::new_v4().to_string(),
            attacker_alliance_id: self.id,
            defender_alliance_id: 0, // Would be set during matchmaking
            state: AllianceWarState::Preparation,
            preparation_start: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64,
            war_start: 0,
            war_end: 0,
            attacker_participants: Vec::new(),
            defender_participants: Vec::new(),
            attacker_stars: 0,
            defender_stars: 0,
            attacker_destruction: 0.0,
            defender_destruction: 0.0,
            attacker_attacks: 0,
            defender_attacks: 0,
        };
        
        self.current_war = Some(war.clone());
        Ok(war)
    }
    
    pub fn add_war_attack(&mut self, attacker_id: i64, defender_id: i64, stars: u8, destruction: f32) -> Result<(), String> {
        let war = self.current_war.as_mut()
            .ok_or("No active war")?;
        
        if war.state != AllianceWarState::InWar {
            return Err("War not in progress".to_string());
        }
        
        // Find attacker participant
        let attacker = war.attacker_participants.iter_mut()
            .find(|p| p.player_id == 0) // Would be attacker's player_id
            .ok_or("Attacker not in war")?;
        
        if attacker.attacks_used >= attacker.attacks_available {
            return Err("No attacks remaining".to_string());
        }
        
        attacker.attacks_used += 1;
        attacker.stars_earned += stars as u32;
        attacker.destruction_percent = destruction.max(attacker.destruction_percent);
        attacker.best_attack_destruction = destruction.max(attacker.best_attack_destruction);
        
        // Update defender
        if let Some(defender) = war.defender_participants.iter_mut().find(|p| p.player_id == 0) {
            defender.destruction_percent = destruction.max(defender.destruction_percent);
        }
        
        // Update war stats
        // (Would update stars and destruction totals)
        
        Ok(())
    }
    
    pub fn get_member(&self, player_id: i64) -> Option<&AllianceMember> {
        self.members.get(&player_id)
    }
    
    pub fn get_member_mut(&mut self, player_id: i64) -> Option<&mut AllianceMember> {
        self.members.get_mut(&player_id)
    }
    
    pub fn get_online_members(&self) -> Vec<&AllianceMember> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;
        self.members.values()
            .filter(|m| now - m.last_online < 300) // Online within 5 minutes
            .collect()
    }
    
    pub fn get_donation_requests(&self) -> &[AllianceDonation] {
        &self.donation_requests
    }
    
    pub fn get_chat_history(&self, limit: usize) -> &[AllianceChatMessage] {
        let start = self.chat.len().saturating_sub(limit);
        &self.chat[start..]
    }
}