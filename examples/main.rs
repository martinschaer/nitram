use actix_files::NamedFile;
use concierge::EmptyParams;
use concierge::FromResources;
use concierge::IntoParams;
use serde::Deserialize;
use std::sync::Arc;
use ts_rs::TS;

use actix_web::{middleware::Logger, web, App, HttpServer};
use concierge::{
    auth::SessionAuthedResource, concierge_handler, error::MethodResult, ws, ConciergeBuilder,
};
use tokio::sync::Mutex;

#[derive(Clone)]
struct MockDB {}
impl MockDB {
    fn get(&self) -> String {
        "hello".to_string()
    }
}

#[derive(Clone)]
struct ConciergeResource {
    db: MockDB,
}
impl FromResources for ConciergeResource {}
impl ConciergeResource {
    fn new() -> Self {
        Self { db: MockDB {} }
    }
}

async fn hello_handler(resource: ConciergeResource) -> MethodResult<String> {
    Ok(resource.db.get())
}
concierge_handler!(HelloAPI, String);

async fn echo_handler(session: SessionAuthedResource, params: EchoParams) -> MethodResult<String> {
    Ok(format!("Hello {}: {}", session.user_id, params.msg))
}
concierge_handler!(EchoAPI, EchoParams,String, msg:String);

async fn signal_handler(session: SessionAuthedResource) -> MethodResult<String> {
    Ok(format!("Hello {}", session.user_id))
}

async fn index() -> actix_web::Result<NamedFile> {
    Ok(NamedFile::open("examples/web-app/dist/index.html")?)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let inner = concierge::ConciergeInner::default();
    let inner_arc = Arc::new(Mutex::new(inner));
    let resource = ConciergeResource::new();
    let cb = ConciergeBuilder::default()
        .add_resource(resource)
        .add_public_handler("Hello", hello_handler)
        .add_private_handler("Echo", echo_handler)
        .add_signal_handler("Signal", signal_handler);
    let concierge = cb.build(inner_arc);
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(web::Data::new(concierge.clone()))
            .route("/", web::get().to(index))
            .service(actix_files::Files::new("/", "examples/web-app/dist"))
            .route("/ws", web::get().to(ws::handler))
    })
    .bind(("0.0.0.0", 8000))?
    .run()
    .await
}
