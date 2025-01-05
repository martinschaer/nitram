use serde::{Deserialize, Serialize};
use serde_json::Value;
use ts_rs::TS;

#[derive(Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ConciergeRequest {
    pub id: String,
    pub method: String,
    pub params: Value,
}

#[derive(Serialize, TS)]
#[ts(export)]
pub struct ConciergeResponse {
    pub id: String,
    pub method: String,
    pub response: Value,
    pub ok: bool,
}

impl ConciergeResponse {
    pub fn error(s: impl Into<String>) -> Self {
        Self {
            id: "_err".to_string(),
            method: "_err".to_string(),
            response: serde_json::Value::String(s.into()),
            ok: false,
        }
    }
}

#[derive(Serialize, TS)]
#[ts(export)]
pub struct ConciergeSignal {
    pub signal: String,
    pub payload: Value,
}
