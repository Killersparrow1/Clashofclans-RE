//! Multiplayer & Social Module
//! 
//! Handles alliance system, matchmaking, and replay system.

pub mod alliance;
pub mod matchmaking;
pub mod replay;

pub use alliance::{
    Alliance, AllianceMember, AllianceRole, AllianceWar, AllianceChatMessage,
    AllianceDonation, AllianceWarState, WarParticipant,
};
pub use matchmaking::{
    Matchmaker, MatchmakingPool, MatchmakingCriteria, MatchResult,
};
pub use replay::{
    Replay, ReplayFrame, ReplayEvent, ReplayMetadata,
};