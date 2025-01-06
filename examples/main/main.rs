use actix_files::NamedFile;
use actix_web::{middleware::Logger, web, App, HttpServer};
use nitram::EmptyParams;
use nitram::FromResources;
use nitram::IntoParams;
use nitram::NitramInner;
use nitram::{auth::SessionAuthedResource, error::MethodResult, nitram_handler, ws, NitramBuilder};
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;
use ts_rs::TS;

#[derive(Clone)]
struct MockDB {}
impl MockDB {
    fn get(&self) -> String {
        "hello".to_string()
    }
}

#[derive(Clone)]
struct NitramResource {
    db: MockDB,
}
impl FromResources for NitramResource {}
impl NitramResource {
    fn new() -> Self {
        Self { db: MockDB {} }
    }
}

// =============================================================================
// Handlers
// =============================================================================

async fn hello_handler(resource: NitramResource) -> MethodResult<String> {
    Ok(resource.db.get())
}
nitram_handler!(HelloAPI, String);

async fn echo_handler(session: SessionAuthedResource, params: EchoParams) -> MethodResult<String> {
    Ok(format!("Hello {}: {}", session.user_id, params.msg))
}
nitram_handler!(EchoAPI, EchoParams, String, msg:String);

async fn authenticate_handler(
    _resource: NitramResource,
    _params: AuthenticateParams,
) -> MethodResult<bool> {
    Ok(true)
}
nitram_handler!(AuthenticateAPI, AuthenticateParams, bool, token: String);

async fn signal_handler(session: SessionAuthedResource) -> MethodResult<String> {
    Ok(format!("Hello {}", session.user_id))
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
        .add_public_handler("Hello", hello_handler)
        .add_public_handler("Authenticate", authenticate_handler)
        .add_private_handler("Echo", echo_handler)
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
