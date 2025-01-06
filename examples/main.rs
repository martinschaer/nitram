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

async fn hello_handler(resource: NitramResource) -> MethodResult<String> {
    Ok(resource.db.get())
}
nitram_handler!(HelloAPI, String);

async fn echo_handler(session: SessionAuthedResource, params: EchoParams) -> MethodResult<String> {
    Ok(format!("Hello {}: {}", session.user_id, params.msg))
}
nitram_handler!(EchoAPI, EchoParams,String, msg:String);

async fn signal_handler(session: SessionAuthedResource) -> MethodResult<String> {
    Ok(format!("Hello {}", session.user_id))
}

async fn index() -> actix_web::Result<NamedFile> {
    Ok(NamedFile::open("examples/web-app/dist/index.html")?)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let inner = NitramInner::default();
    let inner_arc = Arc::new(Mutex::new(inner));
    let resource = NitramResource::new();
    let cb = NitramBuilder::default()
        .add_resource(resource)
        .add_public_handler("Hello", hello_handler)
        .add_private_handler("Echo", echo_handler)
        .add_signal_handler("Signal", signal_handler);
    let nitram = cb.build(inner_arc);
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(web::Data::new(nitram.clone()))
            .route("/", web::get().to(index))
            .service(actix_files::Files::new("/", "examples/web-app/dist"))
            .route("/ws", web::get().to(ws::handler))
    })
    .bind(("0.0.0.0", 8000))?
    .run()
    .await
}
