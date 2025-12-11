use actix_files::NamedFile;
use actix_web::{middleware::Logger, web, App, HttpServer};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;
use ts_rs::TS;
use uuid::Uuid;

use nitram::{
    auth::{parse_token, WSSessionAnonymResource, WSSessionAuthedResource},
    error::{MethodError, MethodResult},
    models::{AuthStrategy, DBSession, ParsedToken, Store},
    nitram_handler, ws, AuthenticateParams, FromResources, IdParams, IntoParams, NitramBuilder,
    NitramInner,
};

#[derive(Clone, Deserialize, Serialize, TS)]
struct User {
    id: String,
    name: String,
}
impl User {
    fn new(name: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name,
        }
    }
}

#[derive(Clone, Default)]
struct MockDB {
    users: HashMap<String, User>,
    messages: HashMap<String, Vec<String>>,
}
impl MockDB {
    fn insert_user(&mut self, user: User) -> String {
        let user_id = user.id.clone();
        self.users.insert(user_id.clone(), user);
        user_id
    }
    fn insert_message(&mut self, channel: &str, message: String, user_id: &str) -> () {
        let user = self.users.get(user_id);
        match user {
            None => {}
            Some(user) => {
                let list = self.messages.get_mut(channel);
                match list {
                    Some(list) => list.push(format!("{}: {}", user.name, message)),
                    None => {
                        self.messages.insert(
                            channel.to_string(),
                            vec![format!("{}: {}", user.name, message)],
                        );
                    }
                }
            }
        }
    }
}

#[derive(Clone)]
struct NitramResource {
    db: Arc<Mutex<MockDB>>,
    nitram_inner: Arc<Mutex<NitramInner>>,
}
impl FromResources for NitramResource {}
impl NitramResource {
    fn new(nitram_inner: Arc<Mutex<NitramInner>>) -> Self {
        let db = Arc::new(Mutex::new(MockDB::default()));
        Self { db, nitram_inner }
    }
}

// =============================================================================
// Handlers
// =============================================================================

// We are taking a shortcut here for the sake of the example. In a real-world
// application, we would send the user an email with a link that would contain
// an id to look for a token in the DB.
async fn get_token_handler(
    resource: NitramResource,
    params: GetTokenParams,
) -> MethodResult<String> {
    let mut db = resource.db.lock().await;
    let user_id = db.insert_user(User::new(params.user_name));
    let db_session =
        DBSession::new(&user_id, AuthStrategy::EmailLink).map_err(|_| MethodError::Server)?;
    let qty = db.users.len();
    tracing::debug!("Users: {:?}", qty);
    Ok(db_session.token)
}
nitram_handler!(
    GetTokenAPI,    // Method name
    GetTokenParams, // Params type
    String,         // Return type
    // Params
    user_name: String
);

async fn send_message_handler(
    resource: NitramResource,
    session: WSSessionAuthedResource,
    mut store: Store,
    params: SendMessageParams,
) -> MethodResult<Vec<String>> {
    let count = store.get::<i32>("count").await.unwrap_or_default();
    let now = Utc::now();
    store.insert("last", json!(now)).await;
    store.insert("count", json!(count + 1)).await;
    store.insert("notify", json!(true)).await;
    let mut db = resource.db.lock().await;
    db.insert_message(&params.channel, params.message, &session.user_id);
    Ok(db.messages.get(&params.channel).unwrap().clone())
}
nitram_handler!(
    SendMessageAPI,    // Method name
    SendMessageParams, // Params type
    Vec<String>,       // Return type
    // Params
    message: String,
    channel: String
);

async fn authenticate_handler(
    resource: NitramResource,
    anonym_session: WSSessionAnonymResource,
    params: AuthenticateParams,
) -> MethodResult<String> {
    let db = resource.db.lock().await;
    let token = params.token.clone();
    let parsed_token: ParsedToken = parse_token(&token).map_err(|_| MethodError::Server)?;
    tracing::debug!("Parsed token: {:?}", parsed_token);
    match db.users.get(&parsed_token.user_id) {
        Some(user) => {
            let user_id = user.id.clone();

            // authenticate nitram session
            let mut nitram = resource.nitram_inner.lock().await;
            let db_session = DBSession {
                id: parsed_token.db_session_id,
                user_id: user_id.clone(),
                strategy: AuthStrategy::EmailLink,
                token,
                expires_at: parsed_token.expires_at,
            };
            nitram.auth_ws_session(anonym_session.ws_session_id, db_session);

            Ok(user_id)
        }
        None => {
            tracing::debug!("Invalid token. Not surprising since the DB is not persistent. Front-end should clear the token.");
            Err(MethodError::NotAuthenticated)
        }
    }
}

#[derive(Clone, Deserialize, Serialize, TS)]
struct MessagesOutput {
    messages: Vec<String>,
    last: Option<DateTime<Utc>>,
    count: i32,
}

async fn messages_handler(
    resource: NitramResource,
    mut store: Store,
    params: MessagesParams,
) -> MethodResult<MessagesOutput> {
    let last = store.get::<DateTime<Utc>>("last").await;
    let count = store.get::<i32>("count").await;
    let notify = store.get::<bool>("notify").await.unwrap_or(true);
    if !notify {
        return Err(MethodError::NoResponse);
    }

    store.insert("notify", json!(false)).await;
    let db = resource.db.lock().await;
    Ok(MessagesOutput {
        messages: db
            .messages
            .get(&params.channel)
            .cloned()
            .unwrap_or_default(),
        last,
        count: count.unwrap_or_default(),
    })
}
nitram_handler!(
    MessagesAPI,    // Method name
    MessagesParams, // Params type
    MessagesOutput, // Return type
    // Params
    channel: String
);

async fn get_user_handler(resource: NitramResource, params: IdParams) -> MethodResult<User> {
    let db = resource.db.lock().await;
    match db.users.get(&params.id) {
        Some(user) => Ok(user.clone()),
        None => Err(MethodError::NotFound),
    }
}
nitram_handler!(
    GetUserAPI, // Method name
    IdParams,   // Params type
    User        // Return type
);

// =============================================================================
// Server
// =============================================================================
async fn index() -> actix_web::Result<NamedFile> {
    Ok(NamedFile::open("examples/main/web-app/dist/index.html")?)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt()
        .with_ansi(true)
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .init();
    let inner = NitramInner::default();
    let inner_arc = Arc::new(Mutex::new(inner));
    let resource = NitramResource::new(inner_arc.clone());
    let cb = NitramBuilder::default()
        .add_resource(resource)
        .add_public_handler("Authenticate", authenticate_handler)
        .add_public_handler("GetToken", get_token_handler)
        .add_private_handler("SendMessage", send_message_handler)
        .add_private_handler("GetUser", get_user_handler)
        .add_server_message_handler("Messages", messages_handler);
    let nitram = cb.build(inner_arc);
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(web::Data::new(nitram.clone()))
            .route("/ws", web::get().to(ws::handler))
            .route("/", web::get().to(index))
            .service(actix_files::Files::new("/", "examples/main/web-app/dist"))
    })
    .bind(("0.0.0.0", 8000))?
    .run()
    .await
}

// =============================================================================
// Tests are required to generate the TS bindings
// =============================================================================
#[cfg(test)]
mod tests {
    #[test]
    fn test_to_generate_bindings() {
        assert!(true);
    }
}
