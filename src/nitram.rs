use bytestring::ByteString;
use rpc_router::{Request, Resources, Router};
use serde_json::{json, Value};
use std::{collections::BTreeMap, sync::Arc};
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::auth::{NitramSession, WSSessionAnonymResource, WSSessionAuthedResource};
use crate::error::{Error, MethodError, Result};
use crate::messages::{NitramRequest, NitramResponse, NitramServerMessage};
use crate::models::{DBSession, UserPayload};
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
            .insert(ws_session_id, NitramSession::new(db_session));
        tracing::debug!("auth_ws_session sessions: {:?}", self.ws_sessions);
    }
}

#[derive(Clone)]
pub struct Nitram {
    pub inner: Arc<Mutex<NitramInner>>,
    rpc_router_public: Router,
    rpc_router_private: Router,
    rpc_router_server_messages: Router,
    registered_public_handlers: Vec<String>,
    registered_private_handlers: Vec<String>,
    registered_server_message_handlers: Vec<String>,
}

impl Nitram {
    pub fn new(
        rpc_router_public: Router,
        rpc_router_private: Router,
        rpc_router_server_messages: Router,
        registered_public_handlers: Vec<String>,
        registered_private_handlers: Vec<String>,
        registered_server_message_handlers: Vec<String>,
        inner: Arc<Mutex<NitramInner>>,
    ) -> Self {
        // TODO: spawn a tokio task to read from live query streams
        Nitram {
            inner,
            rpc_router_public,
            rpc_router_private,
            rpc_router_server_messages,
            registered_public_handlers,
            registered_private_handlers,
            registered_server_message_handlers,
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

    async fn is_auth(&self, ws_session_id: &Uuid) -> Result<UserPayload> {
        let inner = self.inner.lock().await;
        tracing::debug!("WS sessions: {:?}", inner.ws_sessions);
        match inner.ws_sessions.get(ws_session_id) {
            Some(NitramSession::Authenticated {
                db_session,
                topics_registered: _,
                store,
            }) => Ok(UserPayload {
                db_session: db_session.clone(),
                store: store.clone(),
            }),
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

        // -- Topic registration
        let is_register = msg == "nitram_topic_register";
        let is_deregister = msg == "nitram_topic_deregister";
        if is_register || is_deregister {
            let topic = params.get("topic").map(|x| x.as_str()).flatten();
            match topic {
                Some(topic) => {
                    let params = match params.get("handler_params") {
                        Some(params) => params.clone(),
                        None => {
                            if is_register {
                                tracing::error!("Missing params for topic registration");
                            }
                            Value::Null
                        }
                    };
                    let mut inner = self.inner.lock().await;
                    tracing::debug!("WS sessions: {:?}", inner.ws_sessions);
                    match inner.ws_sessions.get_mut(ws_session_id) {
                        Some(NitramSession::Authenticated {
                            db_session: _,
                            topics_registered,
                            store: _,
                        }) => {
                            if is_register {
                                topics_registered.insert(topic.to_string(), params);
                            } else if is_deregister {
                                topics_registered.remove(topic);
                            }
                            return Ok(json!(true));
                        }
                        _ => {
                            tracing::error!("Invalid session state for topic registration");
                        }
                    }
                }
                None => {
                    tracing::error!("Missing topic for registration");
                }
            }
        }

        // -- RPC handling
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
            let user_payload = self.is_auth(ws_session_id).await?;
            let session_resource = WSSessionAuthedResource {
                user_id: user_payload.db_session.user_id,
            };
            let rpc_resources = Resources::builder()
                .append(session_resource)
                .append(user_payload.store)
                .build();
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

    pub async fn get_server_messages_for_session(
        &self,
        ws_session_id: &Uuid,
    ) -> Vec<NitramServerMessage> {
        let mut server_messages: Vec<NitramServerMessage> = vec![];
        let inner = self.inner.lock().await;
        let session = inner.ws_sessions.get(ws_session_id);
        if let Some(session) = session {
            if let NitramSession::Authenticated {
                db_session,
                topics_registered,
                store,
            } = session
            {
                // Call registered server message handlers
                for topic in &self.registered_server_message_handlers {
                    match topics_registered.contains_key(topic) {
                        true => {
                            let rpc_request = Request {
                                id: "fake".into(),
                                method: topic.clone(),
                                params: topics_registered.get(topic).cloned(),
                            };
                            let session_resource = WSSessionAuthedResource {
                                user_id: db_session.user_id.clone(),
                            };
                            let rpc_resources = Resources::builder()
                                .append(session_resource)
                                .append(store.clone())
                                .build();

                            let result: Result<Value> = self
                                .rpc_router_server_messages
                                .call_with_resources(rpc_request, rpc_resources)
                                .await
                                .map(|r| r.value)
                                .map_err(|e| e.into());

                            match result {
                                Ok(result) => {
                                    server_messages.push(NitramServerMessage {
                                        topic: topic.clone(),
                                        payload: result,
                                    });
                                }
                                Err(e) => {
                                    tracing::error!("Error calling server message handler: {}", e);
                                }
                            }
                        }
                        false => {
                            // Skip if user is not registered to the topic
                        }
                    }
                }
            }
        }
        server_messages
    }
}
