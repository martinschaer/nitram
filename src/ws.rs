use actix_web::{web, HttpRequest, HttpResponse};
use actix_ws::AggregatedMessage;
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::Mutex;

use crate::Nitram;

pub async fn handler(
    req: HttpRequest,
    body: web::Payload,
    nitram: web::Data<Nitram>,
) -> std::result::Result<HttpResponse, actix_web::Error> {
    let (response, mut session, stream) = actix_ws::handle(&req, body)?;

    // TODO: is there a better frame size?
    // increase the maximum allowed frame size to 128KiB and aggregate continuation frames
    let mut stream = stream.max_frame_size(128 * 1024).aggregate_continuations();

    let session_id = nitram.insert().await;

    let alive = Arc::new(Mutex::new(Instant::now()));
    let alive2 = alive.clone();
    let mut session2 = session.clone();
    let nitram_for_loop = nitram.clone();
    actix_web::rt::spawn(async move {
        let mut interval = actix_web::rt::time::interval(Duration::from_secs(5));

        loop {
            interval.tick().await;
            if session2.ping(b"").await.is_err() {
                tracing::debug!(
                    sess = session_id.to_string(),
                    "Breaking loop because ping failed"
                );
                break;
            }

            if Instant::now().duration_since(*alive2.lock().await) > Duration::from_secs(10) {
                let _ = session2.close(None).await;
                tracing::debug!(
                    sess = session_id.to_string(),
                    "Breaking loop because of timeout"
                );
                break;
            }

            // -- Session signals
            let signals = nitram_for_loop.get_signals_for_session(&session_id).await;
            match serde_json::to_string(&signals) {
                Ok(json) => {
                    let _ = session2.text(json).await;
                }
                _ => {}
            }
        }

        tracing::debug!(sess = session_id.to_string(), "Loop ended");
        nitram_for_loop.remove(&session_id).await;
    });

    actix_web::rt::spawn(async move {
        while let Some(Ok(msg)) = stream.recv().await {
            match msg {
                AggregatedMessage::Ping(bytes) => {
                    if session.pong(&bytes).await.is_err() {
                        return;
                    }
                }

                AggregatedMessage::Text(string) => {
                    tracing::debug!(sess = session_id.to_string(), "Relaying text: {}", string);
                    let res = nitram.send(string, &session_id).await;
                    let _ = session.text(res).await;
                }

                AggregatedMessage::Close(reason) => {
                    let _ = session.close(reason).await;
                    tracing::debug!(sess = session_id.to_string(), "Got close, bailing");
                    return;
                }

                AggregatedMessage::Pong(_) => {
                    *alive.lock().await = Instant::now();
                }

                _ => (),
            };
        }
        let _ = session.close(None).await;
        nitram.remove(&session_id).await;
    });

    tracing::info!(sess = session_id.to_string(), "Spawned");

    Ok(response)
}
