use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::auth::generate_token;
use crate::error::Result;

#[derive(Clone, Serialize, Deserialize)]
pub enum AuthStrategy {
    EmailLink,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ParsedToken {
    pub expires_at: DateTime<Utc>,
    pub db_session_id: DBSessionId,
    pub user_id: String,
}

// TODO(6cd5): make DBSessionId a struct with inner types that can be String or
// Uuid, with a new() method, and implementing Serialize and Deserialize
pub type DBSessionId = String;

#[derive(Clone, Serialize, Deserialize)]
pub struct DBSession {
    pub id: DBSessionId,
    pub user_id: String,
    pub strategy: AuthStrategy,
    pub token: String,
    pub expires_at: DateTime<Utc>,
}

impl DBSession {
    pub fn new(user_id: impl Into<String>, strategy: AuthStrategy) -> Result<Self> {
        let user_id = user_id.into();
        let (id, expires_at, encoded_token) = generate_token(&user_id, &strategy)?;
        Ok(Self {
            id,
            user_id,
            strategy,
            token: encoded_token,
            expires_at,
        })
    }
}
