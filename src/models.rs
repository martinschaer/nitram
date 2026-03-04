use chrono::{DateTime, Utc};
use rpc_router::RpcResource;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

pub type UserSessionId = String;

/// **User session** that is stored in the database
#[derive(Clone, Serialize, Deserialize)]
pub struct UserSession {
    pub id: UserSessionId,
    pub user_id: String,
    pub expires_at: DateTime<Utc>,
}

#[derive(Clone, RpcResource)]
pub struct Store {
    pub kv: Arc<Mutex<HashMap<String, Value>>>,
}

impl Store {
    pub fn new() -> Self {
        Self {
            kv: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    pub async fn get<T: DeserializeOwned>(&self, key: &str) -> Option<T> {
        let store = self.kv.lock().await;
        match store.get(key).cloned() {
            Some(x) => serde_json::from_value(x).ok(),
            None => None,
        }
    }
    pub async fn insert(&mut self, key: &str, value: Value) -> Option<Value> {
        let mut store = self.kv.lock().await;
        store.insert(key.to_string(), value)
    }
}

pub struct UserPayload {
    pub user_session: UserSession,
    pub store: Store,
}
