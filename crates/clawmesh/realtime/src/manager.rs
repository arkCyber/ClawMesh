//! Connection and room management

use crate::messages::ServerMessage;
use crate::{ConnectionId, PresenceStatus, RoomId, UserPresence};
use anyhow::Result;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use tracing::{debug, info, warn};

/// Connection manager for tracking active connections
#[derive(Clone)]
pub struct ConnectionManager {
    /// Map of connection ID to user ID
    connections: Arc<RwLock<HashMap<ConnectionId, i32>>>,
    /// Map of user ID to connection IDs
    user_connections: Arc<RwLock<HashMap<i32, HashSet<ConnectionId>>>>,
    /// User presence information
    presence: Arc<RwLock<HashMap<i32, UserPresence>>>,
}

impl ConnectionManager {
    /// Create a new connection manager
    #[must_use]
    pub fn new() -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            user_connections: Arc::new(RwLock::new(HashMap::new())),
            presence: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a new connection
    ///
    /// # Errors
    /// Returns error if locks are poisoned
    pub fn register(&self, conn_id: ConnectionId, user_id: i32) -> Result<()> {
        let mut connections = self.connections.write().map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
        let mut user_connections = self.user_connections.write().map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
        let mut presence = self.presence.write().map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;

        connections.insert(conn_id, user_id);
        user_connections.entry(user_id).or_insert_with(HashSet::new).insert(conn_id);

        let user_presence = presence.entry(user_id).or_insert_with(|| UserPresence {
            user_id,
            status: PresenceStatus::Online,
            last_seen: chrono::Utc::now(),
            connections: Vec::new(),
        });
        user_presence.connections.push(conn_id);
        user_presence.status = PresenceStatus::Online;
        user_presence.last_seen = chrono::Utc::now();

        info!("Registered connection {} for user {}", conn_id, user_id);
        Ok(())
    }

    /// Unregister a connection
    ///
    /// # Errors
    /// Returns error if locks are poisoned
    pub fn unregister(&self, conn_id: ConnectionId) -> Result<Option<i32>> {
        let mut connections = self.connections.write().map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
        let mut user_connections = self.user_connections.write().map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
        let mut presence = self.presence.write().map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;

        if let Some(user_id) = connections.remove(&conn_id) {
            if let Some(conns) = user_connections.get_mut(&user_id) {
                conns.remove(&conn_id);
                
                if let Some(user_presence) = presence.get_mut(&user_id) {
                    user_presence.connections.retain(|&id| id != conn_id);
                    if user_presence.connections.is_empty() {
                        user_presence.status = PresenceStatus::Offline;
                    }
                    user_presence.last_seen = chrono::Utc::now();
                }
            }
            info!("Unregistered connection {} for user {}", conn_id, user_id);
            Ok(Some(user_id))
        } else {
            Ok(None)
        }
    }

    /// Get user presence
    ///
    /// # Errors
    /// Returns error if locks are poisoned
    pub fn get_presence(&self, user_id: i32) -> Result<Option<UserPresence>> {
        let presence = self.presence.read().map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
        Ok(presence.get(&user_id).cloned())
    }

    /// Update user presence status
    ///
    /// # Errors
    /// Returns error if locks are poisoned
    pub fn update_presence(&self, user_id: i32, status: PresenceStatus) -> Result<()> {
        let mut presence = self.presence.write().map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
        if let Some(user_presence) = presence.get_mut(&user_id) {
            user_presence.status = status;
            user_presence.last_seen = chrono::Utc::now();
        }
        Ok(())
    }

    /// Get all online users
    ///
    /// # Errors
    /// Returns error if locks are poisoned
    pub fn get_online_users(&self) -> Result<Vec<i32>> {
        let presence = self.presence.read().map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
        Ok(presence
            .values()
            .filter(|p| p.status == PresenceStatus::Online)
            .map(|p| p.user_id)
            .collect())
    }
}

impl Default for ConnectionManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Room manager for managing chat rooms
#[derive(Clone)]
pub struct RoomManager {
    /// Map of room ID to member connection IDs
    rooms: Arc<RwLock<HashMap<RoomId, HashSet<ConnectionId>>>>,
    /// Map of connection ID to joined rooms
    connection_rooms: Arc<RwLock<HashMap<ConnectionId, HashSet<RoomId>>>>,
}

impl RoomManager {
    /// Create a new room manager
    #[must_use]
    pub fn new() -> Self {
        Self {
            rooms: Arc::new(RwLock::new(HashMap::new())),
            connection_rooms: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Join a room
    ///
    /// # Errors
    /// Returns error if locks are poisoned
    pub fn join(&self, conn_id: ConnectionId, room_id: RoomId) -> Result<()> {
        let mut rooms = self.rooms.write().map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
        let mut connection_rooms = self.connection_rooms.write().map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;

        rooms.entry(room_id.clone()).or_insert_with(HashSet::new).insert(conn_id);
        connection_rooms.entry(conn_id).or_insert_with(HashSet::new).insert(room_id.clone());

        info!("Connection {} joined room {}", conn_id, room_id);
        Ok(())
    }

    /// Leave a room
    ///
    /// # Errors
    /// Returns error if locks are poisoned
    pub fn leave(&self, conn_id: ConnectionId, room_id: &RoomId) -> Result<()> {
        let mut rooms = self.rooms.write().map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
        let mut connection_rooms = self.connection_rooms.write().map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;

        if let Some(members) = rooms.get_mut(room_id) {
            members.remove(&conn_id);
        }
        if let Some(user_rooms) = connection_rooms.get_mut(&conn_id) {
            user_rooms.remove(room_id);
        }

        info!("Connection {} left room {}", conn_id, room_id);
        Ok(())
    }

    /// Leave all rooms for a connection
    ///
    /// # Errors
    /// Returns error if locks are poisoned
    pub fn leave_all(&self, conn_id: ConnectionId) -> Result<()> {
        let mut rooms = self.rooms.write().map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
        let mut connection_rooms = self.connection_rooms.write().map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;

        if let Some(user_rooms) = connection_rooms.remove(&conn_id) {
            for room_id in user_rooms {
                if let Some(members) = rooms.get_mut(&room_id) {
                    members.remove(&conn_id);
                }
            }
        }

        info!("Connection {} left all rooms", conn_id);
        Ok(())
    }

    /// Get room members
    ///
    /// # Errors
    /// Returns error if locks are poisoned
    pub fn get_members(&self, room_id: &RoomId) -> Result<Vec<ConnectionId>> {
        let rooms = self.rooms.read().map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
        Ok(rooms.get(room_id).map_or_else(Vec::new, |members| members.iter().copied().collect()))
    }

    /// Get rooms for a connection
    ///
    /// # Errors
    /// Returns error if locks are poisoned
    pub fn get_rooms(&self, conn_id: ConnectionId) -> Result<Vec<RoomId>> {
        let connection_rooms = self.connection_rooms.read().map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
        Ok(connection_rooms.get(&conn_id).map_or_else(Vec::new, |rooms| rooms.iter().cloned().collect()))
    }
}

impl Default for RoomManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_manager() {
        let manager = ConnectionManager::new();
        let conn_id = uuid::Uuid::new_v4();
        let user_id = 1;

        manager.register(conn_id, user_id).expect("Failed to register");
        let presence = manager.get_presence(user_id).expect("Failed to get presence");
        assert!(presence.is_some());
        assert_eq!(presence.unwrap().status, PresenceStatus::Online);

        manager.unregister(conn_id).expect("Failed to unregister");
    }

    #[test]
    fn test_room_manager() {
        let manager = RoomManager::new();
        let conn_id = uuid::Uuid::new_v4();
        let room_id = "test_room".to_string();

        manager.join(conn_id, room_id.clone()).expect("Failed to join room");
        let members = manager.get_members(&room_id).expect("Failed to get members");
        assert_eq!(members.len(), 1);

        manager.leave(conn_id, &room_id).expect("Failed to leave room");
        let members = manager.get_members(&room_id).expect("Failed to get members");
        assert_eq!(members.len(), 0);
    }
}
