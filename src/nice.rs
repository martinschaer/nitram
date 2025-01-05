use serde_json::Value;

pub enum NiceMessage {
    ServerError,
    NotFound,
    NotAuthorized,
    NotAuthenticated,
    BadRequest,
}

impl core::fmt::Display for NiceMessage {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        write!(
            fmt,
            "{}",
            match self {
                NiceMessage::ServerError => "server error".to_string(),
                NiceMessage::NotFound => "not found".to_string(),
                NiceMessage::NotAuthorized => "not authorized".to_string(),
                NiceMessage::NotAuthenticated => "not authenticated".to_string(),
                NiceMessage::BadRequest => "bad request".to_string(),
            }
        )
    }
}

pub struct Nice {
    msg: NiceMessage,
    data: Value,
}

impl From<NiceMessage> for Nice {
    fn from(msg: NiceMessage) -> Self {
        Self {
            msg,
            data: Value::Null,
        }
    }
}

impl Nice {
    pub fn with_data(msg: NiceMessage, data: Value) -> Self {
        Self { msg, data }
    }
}

impl ToString for Nice {
    fn to_string(&self) -> String {
        match self.data {
            Value::Null => format!("(~ {} ~)", self.msg),
            _ => format!(
                "(~ {} ~~ {} ~)",
                self.msg,
                serde_json::to_string(&self.data).unwrap_or_default()
            ),
        }
    }
}

impl Into<serde_json::Value> for Nice {
    fn into(self) -> serde_json::Value {
        self.to_string().into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_without_data() {
        let nice = Nice::from(NiceMessage::ServerError);
        assert_eq!(nice.to_string(), "(~ server error ~)");
    }

    #[test]
    fn test_with_data() {
        let nice = Nice::with_data(
            NiceMessage::ServerError,
            serde_json::json!({ "key": "value" }),
        );
        assert_eq!(
            nice.to_string(),
            "(~ server error ~~ {\"key\":\"value\"} ~)"
        );
    }
}
