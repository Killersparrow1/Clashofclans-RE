use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use sqlx::Row;
use std::sync::Arc;
use tracing::info;
use crate::error::Result;

pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Arc<Self>> {
        let pool = SqlitePoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await?;
        
        let db = Arc::new(Self { pool });
        db.init_schema().await?;
        info!("Database initialized at {}", database_url);
        Ok(db)
    }

    async fn init_schema(&self) -> Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS players (
                id INTEGER PRIMARY KEY,
                device_id TEXT UNIQUE NOT NULL,
                avatar_name TEXT,
                avatar_id INTEGER UNIQUE,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS player_data (
                player_id INTEGER PRIMARY KEY,
                home_json TEXT NOT NULL,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (player_id) REFERENCES players(id)
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS alliances (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                badge_id INTEGER,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS alliance_members (
                alliance_id INTEGER,
                player_id INTEGER,
                role INTEGER DEFAULT 0,
                joined_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                PRIMARY KEY (alliance_id, player_id),
                FOREIGN KEY (alliance_id) REFERENCES alliances(id),
                FOREIGN KEY (player_id) REFERENCES players(id)
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn create_player(&self, device_id: &str) -> Result<i64> {
        let result = sqlx::query(
            "INSERT INTO players (device_id) VALUES (?) RETURNING id"
        )
        .bind(device_id)
        .fetch_one(&self.pool)
        .await?;
        
        let id: i64 = result.get("id");
        Ok(id)
    }

    pub async fn get_player_by_device(&self, device_id: &str) -> Result<Option<Player>> {
        let row = sqlx::query(
            "SELECT id, device_id, avatar_name, avatar_id FROM players WHERE device_id = ?"
        )
        .bind(device_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| Player {
            id: r.get("id"),
            device_id: r.get("device_id"),
            avatar_name: r.get("avatar_name"),
            avatar_id: r.get("avatar_id"),
        }))
    }

    pub async fn update_player_avatar(&self, player_id: i64, avatar_name: &str, avatar_id: i64) -> Result<()> {
        sqlx::query(
            "UPDATE players SET avatar_name = ?, avatar_id = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?"
        )
        .bind(avatar_name)
        .bind(avatar_id)
        .bind(player_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn save_home_data(&self, player_id: i64, home_json: &str) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO player_data (player_id, home_json) VALUES (?, ?)
            ON CONFLICT(player_id) DO UPDATE SET home_json = ?, updated_at = CURRENT_TIMESTAMP
            "#
        )
        .bind(player_id)
        .bind(home_json)
        .bind(home_json)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn load_home_data(&self, player_id: i64) -> Result<Option<String>> {
        let row = sqlx::query(
            "SELECT home_json FROM player_data WHERE player_id = ?"
        )
        .bind(player_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.get("home_json")))
    }

    pub async fn create_alliance(&self, name: &str, badge_id: i32) -> Result<i64> {
        let result = sqlx::query(
            "INSERT INTO alliances (name, badge_id) VALUES (?, ?) RETURNING id"
        )
        .bind(name)
        .bind(badge_id)
        .fetch_one(&self.pool)
        .await?;
        
        let id: i64 = result.get("id");
        Ok(id)
    }

    pub async fn join_alliance(&self, alliance_id: i64, player_id: i64) -> Result<()> {
        sqlx::query(
            "INSERT INTO alliance_members (alliance_id, player_id) VALUES (?, ?)"
        )
        .bind(alliance_id)
        .bind(player_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn leave_alliance(&self, alliance_id: i64, player_id: i64) -> Result<()> {
        sqlx::query(
            "DELETE FROM alliance_members WHERE alliance_id = ? AND player_id = ?"
        )
        .bind(alliance_id)
        .bind(player_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Player {
    pub id: i64,
    pub device_id: String,
    pub avatar_name: Option<String>,
    pub avatar_id: Option<i64>,
}

pub struct GameState {
    pub database: Arc<Database>,
}

impl GameState {
    pub async fn new(database_url: &str) -> Result<Self> {
        let database = Database::new(database_url).await?;
        Ok(Self { database })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_database() {
        let db = Database::new("sqlite::memory:").await.unwrap();
        let player_id = db.create_player("test-device").await.unwrap();
        
        let player = db.get_player_by_device("test-device").await.unwrap();
        assert!(player.is_some());
        assert_eq!(player.as_ref().unwrap().id, player_id);
        
        db.update_player_avatar(player_id, "TestAvatar", 12345).await.unwrap();
        
        let player = db.get_player_by_device("test-device").await.unwrap();
        assert_eq!(player.as_ref().unwrap().avatar_name, Some("TestAvatar".to_string()));
        assert_eq!(player.as_ref().unwrap().avatar_id, Some(12345));
    }
}