// Game Logic Data Structures
// Core data structures for Clash of Clans game logic.

pub mod game_data;
pub mod buildings;
pub mod resources;
pub mod combat;

pub use game_data::{
    BuildingData, BuildingType, BuildingState,
    CharacterData, CharacterType,
    SpellData, SpellType,
    HeroData,
    TrapData, TrapType,
    DecorationData,
    ObstacleData,
    TownHallLevelData,
    ResourceCost,
    GameData,
    ExtractedKeys,
};

pub use buildings::{
    GridPos, GridTile, VillageGrid, BuildingInstance, BuildingDataSnapshot,
};

pub use resources::{
    ResourceType, ResourceStorage, ResourceManager,
};

pub use combat::{
    Position, DeployedTroop, TroopState, Target,
    Battle, BattleState, SpellState, HeroState,
    DeployedSpell, DeployedHero, CharacterDataSnapshot,
    find_path,
};