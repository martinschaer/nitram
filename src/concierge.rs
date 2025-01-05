use bytestring::ByteString;
use rpc_router::{Request, Resources, Router};
use serde_json::{json, Value};
use std::{collections::BTreeMap, sync::Arc};
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::auth::{ConciergeSession, SessionAnonymResource, SessionAuthedResource};
use crate::error::{Error, MethodError, Result};
use crate::messages::{ConciergeRequest, ConciergeResponse, ConciergeSignal};
use crate::models::Session;
use crate::nice::{Nice, NiceMessage};

pub struct ConciergeInner {
    sessions: BTreeMap<Uuid, ConciergeSession>,
}

impl Default for ConciergeInner {
    fn default() -> Self {
        ConciergeInner {
            sessions: BTreeMap::new(),
        }
    }
}

impl ConciergeInner {
    pub fn add_anonym_session(&mut self) -> Uuid {
        let id = Uuid::new_v4();
        self.sessions.insert(id, ConciergeSession::Anonymous);
        id
    }
    pub fn add_auth_session(&mut self, session_id: Uuid, session: Session) -> Uuid {
        self.sessions
            .insert(session_id, ConciergeSession::Authenticated(session));
        session_id
    }
}

#[derive(Clone)]
pub struct Concierge {
    pub inner: Arc<Mutex<ConciergeInner>>,
    rpc_router_public: Router,
    rpc_router_private: Router,
    rpc_router_signals: Router,
    registered_public_handlers: Vec<String>,
    registered_private_handlers: Vec<String>,
    registered_signal_handlers: Vec<String>,
}

impl Concierge {
    pub fn new(
        rpc_router_public: Router,
        rpc_router_private: Router,
        rpc_router_signals: Router,
        registered_public_handlers: Vec<String>,
        registered_private_handlers: Vec<String>,
        registered_signal_handlers: Vec<String>,
        inner: Arc<Mutex<ConciergeInner>>,
    ) -> Self {
        // TODO: spawn a tokio task to read from live query streams
        Concierge {
            inner,
            rpc_router_public,
            rpc_router_private,
            rpc_router_signals,
            registered_public_handlers,
            registered_private_handlers,
            registered_signal_handlers,
        }
    }

    pub async fn insert(&self) -> (Uuid, usize) {
        let uuid = Uuid::new_v4();
        let mut sessions = self.inner.lock().await;
        sessions.sessions.insert(uuid, ConciergeSession::Anonymous);
        let count = sessions.sessions.len();
        (uuid, count)
    }

    pub async fn remove(&self, session_id: &Uuid) {
        let mut sessions = self.inner.lock().await;
        let removed = sessions.sessions.remove(session_id);
        let count = sessions.sessions.len();
        tracing::debug!(
            sess = format!("{:?}", removed),
            remaining = count,
            "Removed"
        );
    }

    async fn is_auth(&self, session_id: &Uuid) -> Result<crate::models::Session> {
        let inner = self.inner.lock().await;
        match inner.sessions.get(session_id) {
            Some(ConciergeSession::Authenticated(session)) => Ok(session.clone()),
            Some(ConciergeSession::Anonymous) => Err(Error::NotAuthorized),
            _ => Err(Error::NotAuthenticated),
        }
    }

    async fn handle(
        &self,
        session_id: &Uuid,
        msg: impl Into<String>,
        params: Value,
    ) -> Result<Value> {
        let msg: String = msg.into();
        tracing::debug!("Handling message: {}, with params: {}", msg, params);
        let is_public = self.registered_public_handlers.contains(&msg);
        let is_private = self.registered_private_handlers.contains(&msg);
        let rpc_request: Request = json!({
            "jsonrpc": "2.0",
            "id": null,
            "method": msg,
            "params": Some(params),
        })
        .try_into()?;

        let result = if is_public {
            let session_resource = SessionAnonymResource {
                session_id: session_id.clone(),
            };
            let rpc_resources = Resources::builder().append(session_resource).build();
            self.rpc_router_public
                .call_with_resources(rpc_request, rpc_resources)
                .await
                .map(|r| r.value)
                .map_err(|e| e.into())
        } else if is_private {
            let session = self.is_auth(session_id).await?;
            let session_resource = SessionAuthedResource {
                user_id: session.user_id,
            };
            let rpc_resources = Resources::builder().append(session_resource).build();
            self.rpc_router_private
                .call_with_resources(rpc_request, rpc_resources)
                .await
                .map(|r| r.value)
                .map_err(|e| {
                    tracing::debug!("Error in private rpc router: {:?}", e.error);
                    e.into()
                })
        } else {
            Err(Error::MethodNotFound)
        };
        result
    }

    pub async fn send(&self, payload: impl Into<ByteString>, session_id: &Uuid) -> String {
        let parsed = serde_json::from_str::<ConciergeRequest>(&payload.into());
        let response = match parsed {
            Ok(req) => {
                let id = req.id;
                let method = req.method;
                let params = req.params;
                let res = match self.handle(session_id, &method, params).await {
                    Ok(res) => ConciergeResponse {
                        id,
                        response: res,
                        ok: true,
                        method,
                    },
                    Err(Error::NotAuthorized) => ConciergeResponse {
                        id,
                        response: Nice::from(NiceMessage::NotAuthorized).into(),
                        ok: false,
                        method,
                    },
                    Err(Error::NotAuthenticated) => ConciergeResponse {
                        id,
                        response: Nice::from(NiceMessage::NotAuthenticated).into(),
                        ok: false,
                        method,
                    },
                    Err(Error::RpcCallError(e)) => match e.error {
                        rpc_router::Error::Handler(e) => ConciergeResponse {
                            id,
                            response: serde_json::to_value(e.get::<MethodError>())
                                .unwrap_or_default(),
                            ok: false,
                            method,
                        },
                        rpc_router::Error::ParamsParsing(_)
                        | rpc_router::Error::ParamsMissingButRequested => ConciergeResponse {
                            id,
                            response: Nice::from(NiceMessage::BadRequest).into(),
                            ok: false,
                            method,
                        },
                        rpc_router::Error::MethodUnknown => ConciergeResponse {
                            id,
                            response: Nice::from(NiceMessage::BadRequest).into(),
                            ok: false,
                            method,
                        },
                        _ => ConciergeResponse {
                            id,
                            response: Nice::from(NiceMessage::ServerError).into(),
                            ok: false,
                            method,
                        },
                    },
                    Err(Error::MethodNotFound) => ConciergeResponse {
                        id,
                        response: Nice::from(NiceMessage::BadRequest).into(),
                        ok: false,
                        method,
                    },
                    Err(e) => {
                        tracing::error!("Concierge unknown error: {}", e);
                        ConciergeResponse {
                            id,
                            response: Nice::from(NiceMessage::ServerError).into(),
                            ok: false,
                            method,
                        }
                    }
                };
                res
            }
            Err(_) => {
                let res = ConciergeResponse::error("Invalid message, check API");
                res
            }
        };

        serde_json::to_string(&response).unwrap_or_default()
    }

    pub async fn get_signals_for_session(&self, session_id: &Uuid) -> Vec<ConciergeSignal> {
        let mut signals: Vec<ConciergeSignal> = vec![];
        let inner = self.inner.lock().await;
        let session = inner.sessions.get(session_id);
        if let Some(session) = session {
            if let ConciergeSession::Authenticated(session) = session {
                // Call registered signal handlers
                for signal_handler_name in &self.registered_signal_handlers {
                    let rpc_request = Request {
                        id: "fake".into(),
                        method: signal_handler_name.clone(),
                        params: None,
                    };
                    let session_resource = SessionAuthedResource {
                        user_id: session.user_id.clone(),
                    };
                    let rpc_resources = Resources::builder().append(session_resource).build();

                    let result: Result<Value> = self
                        .rpc_router_signals
                        .call_with_resources(rpc_request, rpc_resources)
                        .await
                        .map(|r| r.value)
                        .map_err(|e| e.into());

                    match result {
                        Ok(result) => {
                            signals.push(ConciergeSignal {
                                signal: signal_handler_name.clone(),
                                payload: result,
                            });
                        }
                        Err(e) => {
                            tracing::error!("Error calling signal handler: {}", e);
                        }
                    }
                }
            }
        }
        signals
    }
}
