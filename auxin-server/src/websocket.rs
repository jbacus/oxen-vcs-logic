use actix_web::{web, HttpRequest, HttpResponse};
use actix_ws::Message;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tracing::{error, info};

use crate::error::AppResult;
use crate::extensions::ActivityType;

/// WebSocket message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum WsMessage {
    /// Activity occurred in repository
    Activity {
        activity_type: String,
        user: String,
        message: String,
        timestamp: String,
    },
    /// Lock status changed
    LockAcquired {
        user: String,
        lock_id: String,
    },
    /// Lock released
    LockReleased {
        lock_id: String,
    },
    /// New commit
    Commit {
        commit_id: String,
        message: String,
        user: String,
    },
    /// Branch created
    BranchCreated {
        branch_name: String,
        user: String,
    },
    /// Ping/Pong for keepalive
    Ping,
    Pong,
}

/// Repository-specific broadcast channel
type RepoChannel = broadcast::Sender<String>;

/// WebSocket hub managing all repository connections
#[derive(Debug, Clone)]
pub struct WsHub {
    /// Map of repository key (namespace/name) to broadcast channel
    channels: Arc<RwLock<HashMap<String, RepoChannel>>>,
}

impl Default for WsHub {
    fn default() -> Self {
        Self::new()
    }
}

impl WsHub {
    pub fn new() -> Self {
        Self {
            channels: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get or create a broadcast channel for a repository
    async fn get_or_create_channel(&self, repo_key: &str) -> RepoChannel {
        let mut channels = self.channels.write().await;

        if let Some(sender) = channels.get(repo_key) {
            sender.clone()
        } else {
            // Create new channel with buffer size 100
            let (sender, _) = broadcast::channel(100);
            channels.insert(repo_key.to_string(), sender.clone());
            sender
        }
    }

    /// Subscribe to a repository's broadcast channel
    pub async fn subscribe(&self, repo_key: &str) -> broadcast::Receiver<String> {
        let sender = self.get_or_create_channel(repo_key).await;
        sender.subscribe()
    }

    /// Broadcast a message to all subscribers of a repository
    pub async fn broadcast(&self, repo_key: &str, message: WsMessage) -> AppResult<()> {
        let channels = self.channels.read().await;

        if let Some(sender) = channels.get(repo_key) {
            let json = serde_json::to_string(&message)
                .map_err(|e| crate::error::AppError::Internal(format!("JSON error: {}", e)))?;

            // It's ok if there are no receivers
            let _ = sender.send(json);
            info!("Broadcast to {}: {:?}", repo_key, message);
        }

        Ok(())
    }

    /// Broadcast an activity event
    pub async fn broadcast_activity(
        &self,
        namespace: &str,
        repo_name: &str,
        activity_type: ActivityType,
        user: &str,
        message: &str,
    ) -> AppResult<()> {
        let repo_key = format!("{}/{}", namespace, repo_name);

        let ws_message = WsMessage::Activity {
            activity_type: format!("{:?}", activity_type).to_lowercase(),
            user: user.to_string(),
            message: message.to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        self.broadcast(&repo_key, ws_message).await
    }

    /// Broadcast lock acquired event
    pub async fn broadcast_lock_acquired(
        &self,
        namespace: &str,
        repo_name: &str,
        user: &str,
        lock_id: &str,
    ) -> AppResult<()> {
        let repo_key = format!("{}/{}", namespace, repo_name);

        let ws_message = WsMessage::LockAcquired {
            user: user.to_string(),
            lock_id: lock_id.to_string(),
        };

        self.broadcast(&repo_key, ws_message).await
    }

    /// Broadcast lock released event
    pub async fn broadcast_lock_released(
        &self,
        namespace: &str,
        repo_name: &str,
        lock_id: &str,
    ) -> AppResult<()> {
        let repo_key = format!("{}/{}", namespace, repo_name);

        let ws_message = WsMessage::LockReleased {
            lock_id: lock_id.to_string(),
        };

        self.broadcast(&repo_key, ws_message).await
    }

    /// Broadcast commit event
    pub async fn broadcast_commit(
        &self,
        namespace: &str,
        repo_name: &str,
        commit_id: &str,
        message: &str,
        user: &str,
    ) -> AppResult<()> {
        let repo_key = format!("{}/{}", namespace, repo_name);

        let ws_message = WsMessage::Commit {
            commit_id: commit_id.to_string(),
            message: message.to_string(),
            user: user.to_string(),
        };

        self.broadcast(&repo_key, ws_message).await
    }
}

/// WebSocket handler for repository notifications
pub async fn ws_handler(
    req: HttpRequest,
    stream: web::Payload,
    path: web::Path<(String, String)>,
    hub: web::Data<WsHub>,
) -> Result<HttpResponse, actix_web::Error> {
    let (namespace, repo_name) = path.into_inner();
    let repo_key = format!("{}/{}", namespace, repo_name);

    info!("WebSocket connection request for: {}", repo_key);

    // Upgrade to WebSocket
    let (response, mut session, mut msg_stream) = actix_ws::handle(&req, stream)?;

    // Subscribe to repository channel
    let mut receiver = hub.subscribe(&repo_key).await;

    // Spawn task to handle the WebSocket connection
    actix_rt::spawn(async move {
        info!("WebSocket connected for: {}", repo_key);

        loop {
            tokio::select! {
                // Handle incoming WebSocket messages
                Some(msg) = msg_stream.next() => {
                    match msg {
                        Ok(Message::Ping(bytes)) => {
                            if session.pong(&bytes).await.is_err() {
                                break;
                            }
                        }
                        Ok(Message::Text(text)) => {
                            // Handle incoming text messages (e.g., ping)
                            if text.trim() == "ping" {
                                let _ = session.text("pong").await;
                            }
                        }
                        Ok(Message::Close(_)) => {
                            info!("WebSocket closed for: {}", repo_key);
                            break;
                        }
                        Err(e) => {
                            error!("WebSocket error for {}: {}", repo_key, e);
                            break;
                        }
                        _ => {}
                    }
                }
                // Handle broadcast messages
                Ok(msg) = receiver.recv() => {
                    if session.text(msg).await.is_err() {
                        break;
                    }
                }
            }
        }

        info!("WebSocket disconnected for: {}", repo_key);
        let _ = session.close(None).await;
    });

    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_hub_subscribe_and_broadcast() {
        let hub = WsHub::new();
        let repo_key = "test/repo";

        // Subscribe
        let mut receiver = hub.subscribe(repo_key).await;

        // Broadcast
        let message = WsMessage::Commit {
            commit_id: "abc123".to_string(),
            message: "Test commit".to_string(),
            user: "testuser".to_string(),
        };

        hub.broadcast(repo_key, message).await.unwrap();

        // Receive
        let received = receiver.recv().await.unwrap();
        assert!(received.contains("abc123"));
        assert!(received.contains("testuser"));
    }

    #[tokio::test]
    async fn test_broadcast_to_nonexistent_repo() {
        let hub = WsHub::new();

        // Should not error even if no subscribers
        let result = hub
            .broadcast(
                "nonexistent/repo",
                WsMessage::Ping,
            )
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_broadcast_activity() {
        let hub = WsHub::new();
        let mut receiver = hub.subscribe("test/repo").await;

        hub.broadcast_activity("test", "repo", ActivityType::Commit, "user", "Test")
            .await
            .unwrap();

        let received = receiver.recv().await.unwrap();
        assert!(received.contains("commit"));
        assert!(received.contains("user"));
    }

    #[tokio::test]
    async fn test_broadcast_lock_acquired() {
        let hub = WsHub::new();
        let mut receiver = hub.subscribe("test/repo").await;

        hub.broadcast_lock_acquired("test", "repo", "user", "lock-123")
            .await
            .unwrap();

        let received = receiver.recv().await.unwrap();
        assert!(received.contains("lock-123"));
        assert!(received.contains("user"));
    }

    #[tokio::test]
    async fn test_multiple_subscribers() {
        let hub = WsHub::new();
        let repo_key = "test/repo";

        let mut receiver1 = hub.subscribe(repo_key).await;
        let mut receiver2 = hub.subscribe(repo_key).await;

        hub.broadcast(repo_key, WsMessage::Ping).await.unwrap();

        // Both should receive the message
        let msg1 = receiver1.recv().await.unwrap();
        let msg2 = receiver2.recv().await.unwrap();

        assert_eq!(msg1, msg2);
    }
}
