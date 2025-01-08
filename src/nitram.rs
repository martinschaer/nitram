use bytestring::ByteString;
use rpc_router::{Request, Resources, Router};
use serde_json::{json, Value};
use std::{collections::BTreeMap, sync::Arc};
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::auth::{NitramSession, WSSessionAnonymResource, WSSessionAuthedResource};
use crate::error::{Error, MethodError, Result};
use crate::messages::{NitramRequest, NitramResponse, NitramSignal};
use crate::models::DBSession;
use crate::nice::{Nice, NiceMessage};

pub struct NitramInner {
    ws_sessions: BTreeMap<Uuid, NitramSession>,
}

impl Default for NitramInner {
    fn default() -> Self {
        NitramInner {
            ws_sessions: BTreeMap::new(),
        }
    }
}

impl NitramInner {
    pub fn add_anonym_ws_session(&mut self) -> Uuid {
        let id = Uuid::new_v4();
        self.ws_sessions.insert(id, NitramSession::Anonymous);
        id
    }
    pub fn auth_ws_session(&mut self, ws_session_id: Uuid, db_session: DBSession) -> () {
        self.ws_sessions
            .insert(ws_session_id, NitramSession::Authenticated(db_session));
        tracing::debug!("auth_ws_session sessions: {:?}", self.ws_sessions);
    }
}

#[derive(Clone)]
pub struct Nitram {
    pub inner: Arc<Mutex<NitramInner>>,
    rpc_router_public: Router,
    rpc_router_private: Router,
    rpc_router_signals: Router,
    registered_public_handlers: Vec<String>,
    registered_private_handlers: Vec<String>,
    registered_signal_handlers: Vec<String>,
}

impl Nitram {
    pub fn new(
        rpc_router_public: Router,
        rpc_router_private: Router,
        rpc_router_signals: Router,
        registered_public_handlers: Vec<String>,
        registered_private_handlers: Vec<String>,
        registered_signal_handlers: Vec<String>,
        inner: Arc<Mutex<NitramInner>>,
    ) -> Self {
        // TODO: spawn a tokio task to read from live query streams
        Nitram {
            inner,
            rpc_router_public,
            rpc_router_private,
            rpc_router_signals,
            registered_public_handlers,
            registered_private_handlers,
            registered_signal_handlers,
        }
    }

    pub async fn insert(&self) -> Uuid {
        let uuid = Uuid::new_v4();
        let mut inner = self.inner.lock().await;
        inner.ws_sessions.insert(uuid, NitramSession::Anonymous);
        let count = inner.ws_sessions.len();
        tracing::info!(sess = uuid.to_string(), count = count, "Inserted session");
        uuid
    }

    pub async fn remove(&self, ws_session_id: &Uuid) {
        let mut inner = self.inner.lock().await;
        let removed = inner.ws_sessions.remove(ws_session_id);
        let count = inner.ws_sessions.len();
        tracing::info!(
            sess = ws_session_id.to_string(),
            kind = format!("{:?}", removed),
            remaining = count,
            "Removed session"
        );
    }

    async fn is_auth(&self, ws_session_id: &Uuid) -> Result<DBSession> {
        let inner = self.inner.lock().await;
        tracing::debug!("WS sessions: {:?}", inner.ws_sessions);
        match inner.ws_sessions.get(ws_session_id) {
            Some(NitramSession::Authenticated(db_session)) => Ok(db_session.clone()),
            Some(NitramSession::Anonymous) => Err(Error::NotAuthorized),
            _ => Err(Error::NotAuthenticated),
        }
    }

    async fn handle(
        &self,
        ws_session_id: &Uuid,
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
            let session_resource = WSSessionAnonymResource {
                ws_session_id: ws_session_id.clone(),
            };
            let rpc_resources = Resources::builder().append(session_resource).build();
            self.rpc_router_public
                .call_with_resources(rpc_request, rpc_resources)
                .await
                .map(|r| r.value)
                .map_err(|e| e.into())
        } else if is_private {
            let db_session = self.is_auth(ws_session_id).await?;
            let session_resource = WSSessionAuthedResource {
                user_id: db_session.user_id,
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

    pub async fn send(&self, payload: impl Into<ByteString>, ws_session_id: &Uuid) -> String {
        let parsed = serde_json::from_str::<NitramRequest>(&payload.into());
        let response = match parsed {
            Ok(req) => {
                let id = req.id;
                let method = req.method;
                let params = req.params;
                let res = match self.handle(ws_session_id, &method, params).await {
                    Ok(res) => NitramResponse {
                        id,
                        response: res,
                        ok: true,
                        method,
                    },
                    Err(Error::NotAuthorized) => NitramResponse {
                        id,
                        response: Nice::from(NiceMessage::NotAuthorized).into(),
                        ok: false,
                        method,
                    },
                    Err(Error::NotAuthenticated) => NitramResponse {
                        id,
                        response: Nice::from(NiceMessage::NotAuthenticated).into(),
                        ok: false,
                        method,
                    },
                    Err(Error::RpcCallError(e)) => match e.error {
                        rpc_router::Error::Handler(e) => NitramResponse {
                            id,
                            response: serde_json::to_value(e.get::<MethodError>())
                                .unwrap_or_default(),
                            ok: false,
                            method,
                        },
                        rpc_router::Error::ParamsParsing(_)
                        | rpc_router::Error::ParamsMissingButRequested => NitramResponse {
                            id,
                            response: Nice::from(NiceMessage::BadRequest).into(),
                            ok: false,
                            method,
                        },
                        rpc_router::Error::MethodUnknown => NitramResponse {
                            id,
                            response: Nice::from(NiceMessage::BadRequest).into(),
                            ok: false,
                            method,
                        },
                        _ => NitramResponse {
                            id,
                            response: Nice::from(NiceMessage::ServerError).into(),
                            ok: false,
                            method,
                        },
                    },
                    Err(Error::MethodNotFound) => NitramResponse {
                        id,
                        response: Nice::from(NiceMessage::BadRequest).into(),
                        ok: false,
                        method,
                    },
                    Err(e) => {
                        tracing::error!("Nitram unknown error: {}", e);
                        NitramResponse {
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
                let res = NitramResponse::error("Invalid message, check API");
                res
            }
        };

        serde_json::to_string(&response).unwrap_or_default()
    }

    pub async fn get_signals_for_session(&self, ws_session_id: &Uuid) -> Vec<NitramSignal> {
        let mut signals: Vec<NitramSignal> = vec![];
        let inner = self.inner.lock().await;
        let session = inner.ws_sessions.get(ws_session_id);
        if let Some(session) = session {
            if let NitramSession::Authenticated(db_session) = session {
                // Call registered signal handlers
                for signal_handler_name in &self.registered_signal_handlers {
                    let rpc_request = Request {
                        id: "fake".into(),
                        method: signal_handler_name.clone(),
                        params: None,
                    };
                    let session_resource = WSSessionAuthedResource {
                        user_id: db_session.user_id.clone(),
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
                            signals.push(NitramSignal {
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
