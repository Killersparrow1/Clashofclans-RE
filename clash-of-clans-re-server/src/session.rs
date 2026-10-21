use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;
use clash_of_clans_re_protocol::{Packet, PacketBuffer};
use crate::error::{ServerError, Result};

pub struct Session {
    pub id: String,
    pub addr: SocketAddr,
    pub buffer: PacketBuffer,
    pub connected_at: Instant,
    pub last_activity: Instant,
    pub is_authenticated: bool,
    pub device_id: Option<String>,
    pub avatar_id: Option<i64>,
}

impl Session {
    pub fn new(id: String, addr: SocketAddr) -> Self {
        let now = Instant::now();
        Self {
            id,
            addr,
            buffer: PacketBuffer::new(),
            connected_at: now,
            last_activity: now,
            is_authenticated: false,
            device_id: None,
            avatar_id: None,
        }
    }

    pub fn push_data(&mut self, data: &[u8]) {
        self.buffer.push(data);
        self.last_activity = Instant::now();
    }

    pub fn try_pop_packet(&mut self) -> Result<Option<Packet>> {
        self.buffer.try_pop()
            .map_err(ServerError::Protocol)
    }

    pub fn authenticate(&mut self, device_id: String, avatar_id: i64) {
        self.is_authenticated = true;
        self.device_id = Some(device_id);
        self.avatar_id = Some(avatar_id);
    }
}

pub type SessionRef = Arc<Mutex<Session>>;

pub struct SessionManager {
    sessions: dashmap::DashMap<String, SessionRef>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: dashmap::DashMap::new(),
        }
    }

    pub fn create_session(&self, id: String, addr: SocketAddr) -> SessionRef {
        let session = Arc::new(Mutex::new(Session::new(id.clone(), addr)));
        self.sessions.insert(id, session.clone());
        session
    }

    pub fn get_session(&self, id: &str) -> Option<SessionRef> {
        self.sessions.get(id).map(|s| s.clone())
    }

    pub fn remove_session(&self, id: &str) -> bool {
        self.sessions.remove(id).is_some()
    }

    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }

    pub fn cleanup_stale_sessions(&self, max_age: std::time::Duration) {
        let now = Instant::now();
        self.sessions.retain(|_, session| {
            match session.try_lock() {
                Ok(s) => now.duration_since(s.last_activity) < max_age,
                Err(_) => true,
            }
        });
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    #[tokio::test]
    async fn test_session_creation() {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 9339);
        let manager = SessionManager::new();
        let session = manager.create_session("test-1".to_string(), addr);
        
        let session_guard = session.lock().await;
        assert_eq!(session_guard.id, "test-1");
        assert_eq!(session_guard.addr, addr);
        assert!(!session_guard.is_authenticated);
    }

    #[tokio::test]
    async fn test_session_manager() {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 9339);
        let manager = SessionManager::new();
        let _session = manager.create_session("test-1".to_string(), addr);
        
        assert_eq!(manager.session_count(), 1);
        assert!(manager.get_session("test-1").is_some());
        assert!(manager.get_session("test-2").is_none());
        
        manager.remove_session("test-1");
        assert_eq!(manager.session_count(), 0);
    }
}