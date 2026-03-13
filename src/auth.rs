use chrono::{DateTime, Utc};
use rpc_router::{IntoParams, RpcResource};
use serde::Deserialize;
use serde_json::Value;
use std::sync::Arc;
use std::{collections::HashMap, fmt};
use tokio::sync::Mutex;
use ts_rs::TS;
use uuid::Uuid;

use crate::models::{Store, UserSession};
use crate::{nitram_handler, NitramState};

#[derive(Clone, RpcResource)]
pub struct WSSessionAnonymResource {
    pub ws_session_id: Uuid,
    pub nitram_state: Arc<Mutex<NitramState>>,
}

impl WSSessionAnonymResource {
    pub async fn auth(&self, user_id: &str, expires_at: DateTime<Utc>) -> () {
        let user_session = UserSession {
            id: self.ws_session_id.clone(),
            user_id: user_id.to_string(),
            expires_at,
        };
        let mut state = self.nitram_state.lock().await;
        state.auth_ws_session(self.ws_session_id, user_session);
    }
}

#[derive(Clone, RpcResource)]
pub struct WSSessionAuthedResource {
    pub user_id: String,
}

#[derive(Clone)]
pub enum NitramSession {
    Anonymous,
    Authenticated {
        user_session: UserSession,
        topics_registered: HashMap<String, Value>,
        store: Store,
    },
}

impl NitramSession {
    pub fn new_auth(user_session: UserSession) -> Self {
        NitramSession::Authenticated {
            user_session,
            topics_registered: HashMap::new(),
            store: Store::new(),
        }
    }
}

impl fmt::Debug for NitramSession {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            NitramSession::Anonymous => write!(f, "Anonymous"),
            NitramSession::Authenticated {
                user_session,
                topics_registered,
                store: _,
            } => {
                write!(
                    f,
                    "Authenticated({},topics={:?})",
                    user_session.id,
                    topics_registered.keys().collect::<Vec<&String>>()
                )
            }
        }
    }
}

nitram_handler!(
    AuthenticateAPI,    // Method name
    AuthenticateParams, // Params type
    String,             // Return type
    // Params
    token: String
);
