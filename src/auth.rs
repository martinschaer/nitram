use base64::prelude::*;
use chrono::{DateTime, Utc};
use rpc_router::RpcResource;
use std::fmt;
use std::time::Duration;
use uuid::Uuid;

use crate::{
    error::{Error, Result},
    models::{AuthStrategy, ParsedToken, Session},
};

#[derive(Clone, RpcResource)]
pub struct SessionAnonymResource {
    pub session_id: Uuid,
}

#[derive(Clone, RpcResource)]
pub struct SessionAuthedResource {
    pub user_id: String,
}

#[derive(Clone)]
pub enum ConciergeSession {
    Anonymous,
    Authenticated(Session),
}

impl fmt::Debug for ConciergeSession {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ConciergeSession::Anonymous => write!(f, "Anonymous"),
            ConciergeSession::Authenticated(session) => {
                write!(f, "Authenticated({})", session.id)
            }
        }
    }
}

pub fn generate_token(
    user_id: impl Into<String>,
    strategy: &AuthStrategy,
) -> Result<(Uuid, DateTime<Utc>, String)> {
    let uuid = Uuid::new_v4();
    let now = Utc::now();
    let expires_at = now + Duration::new(7 * 24 * 60 * 60, 0);
    let token = match strategy {
        AuthStrategy::EmailLink => serde_json::to_string(&ParsedToken {
            expires_at: expires_at.into(),
            session_id: uuid.clone(),
            user_id: user_id.into(),
        }),
    }
    .map_err(|e| Error::TokenError(e.to_string()))?;
    let encoded_token = BASE64_STANDARD.encode(token);
    Ok((uuid, expires_at.into(), encoded_token))
}

pub fn parse_token(token: impl Into<String>) -> Result<ParsedToken> {
    let token = BASE64_STANDARD.decode(token.into().as_bytes());
    let token = token.map_err(|e| Error::TokenError(e.to_string()))?;
    serde_json::from_slice(&token).map_err(|e| Error::TokenError(e.to_string()))
}
