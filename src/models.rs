use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::auth::generate_token;
use crate::error::Result;

#[derive(Clone, Serialize, Deserialize)]
pub enum AuthStrategy {
    EmailLink,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ParsedToken {
    pub expires_at: DateTime<Utc>,
    pub session_id: Uuid,
    pub user_id: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: Uuid,
    pub user_id: String,
    pub strategy: AuthStrategy,
    pub token: String,
    pub expires_at: DateTime<Utc>,
}

impl Session {
    pub fn new(user_id: impl Into<String>, strategy: AuthStrategy) -> Result<Self> {
        let user_id = user_id.into();
        let (uuid, expires_at, encoded_token) = generate_token(&user_id, &strategy)?;
        Ok(Self {
            id: uuid,
            user_id,
            strategy,
            token: encoded_token,
            expires_at,
        })
    }
}
