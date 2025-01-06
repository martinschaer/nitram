#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};
    use serde_json::json;
    use std::sync::Arc;
    use tokio::sync::Mutex;
    use tracing_test::traced_test;

    use nitram::{
        auth::{SessionAnonymResource, SessionAuthedResource},
        error::MethodError,
        models::{AuthStrategy, Session},
        FromResources, IntoParams, Nitram, NitramBuilder,
    };
    use uuid::Uuid;

    #[derive(Clone)]
    pub struct ModelManager {}
    impl FromResources for ModelManager {}

    #[derive(Deserialize, Serialize, Clone)]
    pub struct MockParams {
        code: String,
    }
    impl IntoParams for MockParams {}

    async fn mock_handler(
        _mm: ModelManager,
        _session: SessionAnonymResource,
        params: MockParams,
    ) -> Result<String, MethodError> {
        Ok(params.code)
    }

    async fn mock_private_handler(
        _mm: ModelManager,
        _session: SessionAuthedResource,
        params: MockParams,
    ) -> Result<String, MethodError> {
        if params.code == "return error" {
            return Err(MethodError::Server);
        }
        Ok(params.code.to_uppercase())
    }

    // nitram_api!(MockAPI, MockParams, String);
    // nitram_api!(MockPrivateAPI, MockParams, String);

    struct Context {
        nitram: Nitram,
        anonym: Uuid,
        authed: Uuid,
    }

    async fn prepare() -> Context {
        let inner = nitram::NitramInner::default();
        let inner_arc = Arc::new(Mutex::new(inner));
        let inner_arc_clone = Arc::clone(&inner_arc);
        let mm = ModelManager {};
        let cb = NitramBuilder::default()
            .add_resource(mm)
            .add_public_handler("Mock", mock_handler)
            .add_private_handler("MockPrivate", mock_private_handler);
        let nitram = cb.build(inner_arc);

        let mut nitram_inner = inner_arc_clone.lock().await;
        let anonym = nitram_inner.add_anonym_session();
        let session = Session::new("fake_user", AuthStrategy::EmailLink).unwrap();
        let authed = Uuid::new_v4();
        let authed = nitram_inner.add_auth_session(authed, session);
        Context {
            nitram,
            anonym,
            authed,
        }
    }

    #[tokio::test]
    #[traced_test]
    async fn test_send() -> Result<(), MethodError> {
        let ctx = prepare().await;
        let req = json!({
            "id": "1",
            "method": "Mock",
            "params": {
                "code": "hello"
            },
        });
        let res = json!({
            "id": "1",
            "method": "Mock",
            "response": "hello",
            "ok": true
        });
        let response = ctx.nitram.send(req.to_string(), &ctx.anonym).await;
        let parsed = serde_json::from_str::<serde_json::Value>(&response).unwrap();
        assert_eq!(parsed, res);
        Ok(())
    }

    #[tokio::test]
    #[traced_test]
    async fn test_send_authed() -> Result<(), MethodError> {
        let ctx = prepare().await;
        let req = json!({
            "id": "1",
            "method": "MockPrivate",
            "params": {
                "code": "hello"
            },
        });
        let res = json!({
            "id": "1",
            "method": "MockPrivate",
            "response": "HELLO",
            "ok": true
        });
        let response = ctx.nitram.send(req.to_string(), &ctx.authed).await;
        let parsed = serde_json::from_str::<serde_json::Value>(&response).unwrap();
        assert_eq!(parsed, res);
        Ok(())
    }

    #[tokio::test]
    #[traced_test]
    async fn test_send_not_authorized() -> Result<(), MethodError> {
        let ctx = prepare().await;
        let req = json!({
            "id": "1",
            "method": "MockPrivate",
            "params": {
                "code": "hello"
            },
        });
        let res = json!({
            "id": "1",
            "method": "MockPrivate",
            "response": "(~ not authorized ~)",
            "ok": false
        });
        let response = ctx.nitram.send(req.to_string(), &ctx.anonym).await;
        let parsed = serde_json::from_str::<serde_json::Value>(&response).unwrap();
        assert_eq!(parsed, res);
        Ok(())
    }

    #[tokio::test]
    #[traced_test]
    async fn test_send_error() -> Result<(), MethodError> {
        let ctx = prepare().await;
        let req = json!({
            "id": "1",
            "method": "MockPrivate",
            "params": {
                "code": "return error"
            },
        });
        let res = json!({
            "id": "1",
            "method": "MockPrivate",
            "response": "(~ server error ~)",
            "ok": false
        });
        let response = ctx.nitram.send(req.to_string(), &ctx.authed).await;
        let parsed = serde_json::from_str::<serde_json::Value>(&response).unwrap();
        assert_eq!(parsed, res);
        Ok(())
    }

    #[tokio::test]
    #[traced_test]
    async fn test_send_wrong_params() -> Result<(), MethodError> {
        let ctx = prepare().await;
        let req = json!({
            "id": "1",
            "method": "Mock",
            "params": {
                "wrong": 69
            },
        });
        let res = json!({
            "id": "1",
            "method": "Mock",
            "response": "(~ bad request ~)",
            "ok": false
        });
        let response = ctx.nitram.send(req.to_string(), &ctx.anonym).await;
        let parsed = serde_json::from_str::<serde_json::Value>(&response).unwrap();
        assert_eq!(parsed, res);
        Ok(())
    }
}
