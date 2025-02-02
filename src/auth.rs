use base64::prelude::*;
use chrono::{DateTime, Utc};
use rpc_router::RpcResource;
use std::fmt;
use std::time::Duration;
use uuid::Uuid;

use crate::{
    error::{Error, Result},
    models::{AuthStrategy, DBSession, DBSessionId, ParsedToken},
};

#[derive(Clone, RpcResource)]
pub struct WSSessionAnonymResource {
    pub ws_session_id: Uuid,
}

#[derive(Clone, RpcResource)]
pub struct WSSessionAuthedResource {
    pub user_id: String,
    // db_session_id: Uuid,
}

#[derive(Clone)]
pub enum NitramSession {
    Anonymous,
    Authenticated(DBSession),
}

impl fmt::Debug for NitramSession {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            NitramSession::Anonymous => write!(f, "Anonymous"),
            NitramSession::Authenticated(db_session) => {
                write!(f, "Authenticated({})", db_session.id)
            }
        }
    }
}

pub fn generate_token(
    user_id: impl Into<String>,
    strategy: &AuthStrategy,
) -> Result<(DBSessionId, DateTime<Utc>, String)> {
    // TODO(6cd5): use new method when implemented
    // None => DBSessionId::new(),
    let id = Uuid::new_v4().to_string();
    let now = Utc::now();
    let expires_at = now + Duration::new(7 * 24 * 60 * 60, 0);
    let token = match strategy {
        AuthStrategy::EmailLink => serde_json::to_string(&ParsedToken {
            expires_at: expires_at.into(),
            db_session_id: id.clone(),
            user_id: user_id.into(),
        }),
    }
    .map_err(|e| Error::TokenError(e.to_string()))?;
    let encoded_token = BASE64_STANDARD.encode(token);
    Ok((id, expires_at.into(), encoded_token))
}

pub fn parse_token(token: impl Into<String>) -> Result<ParsedToken> {
    let token = BASE64_STANDARD.decode(token.into().as_bytes());
    let token = token.map_err(|e| Error::TokenError(e.to_string()))?;
    serde_json::from_slice(&token).map_err(|e| Error::TokenError(e.to_string()))
}
