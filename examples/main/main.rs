use actix_files::NamedFile;
use actix_web::{middleware::Logger, web, App, HttpServer};
use nitram::auth::parse_token;
use nitram::auth::SessionAnonymResource;
use nitram::error::MethodError;
use nitram::models::AuthStrategy;
use nitram::models::ParsedToken;
use nitram::models::Session;
use nitram::FromResources;
use nitram::IntoParams;
use nitram::NitramInner;
use nitram::{auth::SessionAuthedResource, error::MethodResult, nitram_handler, ws, NitramBuilder};
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;
use ts_rs::TS;
use uuid::Uuid;

#[derive(Clone)]
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
    messages: Vec<String>,
}
impl MockDB {
    fn insert_user(&mut self, user: User) -> String {
        let user_id = user.id.clone();
        self.users.insert(user_id.clone(), user);
        user_id
    }
    fn insert_message(&mut self, message: String, user_id: &str) -> () {
        let user = self.users.get(user_id);
        match user {
            None => {}
            Some(user) => {
                self.messages.push(format!("{}: {}", user.name, message));
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
    fn new() -> Self {
        let db = Arc::new(Mutex::new(MockDB::default()));
        let nitram_inner = Arc::new(Mutex::new(NitramInner::default()));
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
    let session = Session::new(user_id, AuthStrategy::EmailLink)
        .map_err(|_| MethodError::NotAuthenticated)?;
    let qty = db.users.len();
    tracing::debug!("Users: {:?}", qty);
    Ok(session.token)
}
nitram_handler!(GetTokenAPI, GetTokenParams, String, user_name: String);

async fn send_message_handler(
    resource: NitramResource,
    session: SessionAuthedResource,
    params: SendMessageParams,
) -> MethodResult<()> {
    let mut db = resource.db.lock().await;
    db.insert_message(params.message, &session.user_id);
    Ok(())
}
nitram_handler!(SendMessageAPI, SendMessageParams, (), message: String);

async fn authenticate_handler(
    resource: NitramResource,
    session: SessionAnonymResource,
    params: AuthenticateParams,
) -> MethodResult<bool> {
    let db = resource.db.lock().await;
    let parsed_token: ParsedToken = parse_token(params.token).map_err(|_| MethodError::Server)?;
    tracing::debug!("Parsed token: {:?}", parsed_token);
    match db.users.get(&parsed_token.user_id) {
        Some(user) => {
            let user_id = user.id.clone();

            // authenticate nitram session
            let mut concierge = resource.nitram_inner.lock().await;
            let new_session = Session::new(user_id, AuthStrategy::EmailLink)
                .map_err(|_| MethodError::NotAuthenticated)?;
            let session_id = concierge.add_auth_session(session.session_id, new_session);
            tracing::debug!(sess = session_id.to_string(), "authenticate handler finish");

            Ok(true)
        }
        None => {
            tracing::debug!("Invalid token. Not surprising since the DB is not persistent. Front-end should clear the token.");
            Err(MethodError::NotAuthenticated)
        }
    }
}
nitram_handler!(AuthenticateAPI, AuthenticateParams, bool, token: String);

async fn signal_handler(resource: NitramResource) -> MethodResult<Vec<String>> {
    let db = resource.db.lock().await;
    tracing::debug!("Messages: {:?}", db.messages);
    Ok(db.messages.clone())
}
// We don't need to export signal types to the front-end, that's why we don't
// call nitram_handler!(...) here.

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
    let resource = NitramResource::new();
    let cb = NitramBuilder::default()
        .add_resource(resource)
        .add_public_handler("Authenticate", authenticate_handler)
        .add_public_handler("GetToken", get_token_handler)
        .add_private_handler("SendMessage", send_message_handler)
        .add_signal_handler("Signal", signal_handler);
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
