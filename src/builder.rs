use rpc_router::{FromResources, Handler, RouterBuilder};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::{Nitram, NitramInner};

pub struct NitramBuilder {
    rpc_router_builder_public: RouterBuilder,
    rpc_router_builder_private: RouterBuilder,
    rpc_router_builder_server_messages: RouterBuilder,
    registered_public_handlers: Vec<String>,
    registered_private_handlers: Vec<String>,
    registered_server_messages_handlers: Vec<String>,
    ping_interval_in_seconds: Option<u64>,
    server_messages_interval_in_millis: Option<u64>,
    timeout_in_seconds: Option<u64>,
    max_frame_size: Option<usize>,
}

impl Default for NitramBuilder {
    fn default() -> Self {
        NitramBuilder {
            rpc_router_builder_public: RouterBuilder::default(),
            rpc_router_builder_private: RouterBuilder::default(),
            rpc_router_builder_server_messages: RouterBuilder::default(),
            registered_public_handlers: vec![],
            registered_private_handlers: vec![],
            registered_server_messages_handlers: vec![],
            ping_interval_in_seconds: None,
            server_messages_interval_in_millis: None,
            timeout_in_seconds: None,
            max_frame_size: None,
        }
    }
}

impl NitramBuilder {
    pub fn set_server_messages_interval(mut self, interval_in_millis: u64) -> Self {
        self.server_messages_interval_in_millis = Some(interval_in_millis);
        self
    }

    pub fn add_resource(
        mut self,
        resource: impl FromResources + Clone + Send + Sync + 'static,
    ) -> Self {
        self.rpc_router_builder_public = self
            .rpc_router_builder_public
            .append_resource(resource.clone());
        self.rpc_router_builder_private = self
            .rpc_router_builder_private
            .append_resource(resource.clone());
        self.rpc_router_builder_server_messages = self
            .rpc_router_builder_server_messages
            .append_resource(resource.clone());
        self
    }

    pub fn add_public_handler<H, T, P, R>(mut self, name: &'static str, handler: H) -> Self
    where
        H: Handler<T, P, R> + Clone + Send + Sync + 'static,
        T: Send + Sync + 'static,
        P: Send + Sync + 'static,
        R: Send + Sync + 'static,
    {
        self.registered_public_handlers.push(name.to_string());
        self.rpc_router_builder_public = self
            .rpc_router_builder_public
            .append_dyn(name, handler.into_dyn());
        self
    }

    pub fn add_private_handler<H, T, P, R>(mut self, name: &'static str, handler: H) -> Self
    where
        H: Handler<T, P, R> + Clone + Send + Sync + 'static,
        T: Send + Sync + 'static,
        P: Send + Sync + 'static,
        R: Send + Sync + 'static,
    {
        self.registered_private_handlers.push(name.to_string());
        self.rpc_router_builder_private = self
            .rpc_router_builder_private
            .append_dyn(name, handler.into_dyn());
        self
    }

    pub fn add_server_message_handler<H, T, P, R>(mut self, name: &'static str, handler: H) -> Self
    where
        H: Handler<T, P, R> + Clone + Send + Sync + 'static,
        T: Send + Sync + 'static,
        P: Send + Sync + 'static,
        R: Send + Sync + 'static,
    {
        self.registered_server_messages_handlers
            .push(name.to_string());
        self.rpc_router_builder_server_messages = self
            .rpc_router_builder_server_messages
            .append_dyn(name, handler.into_dyn());
        self
    }

    pub fn build(self, inner: Arc<Mutex<NitramInner>>) -> Nitram {
        tracing::debug!(
            "Registered public handlers: {:?}",
            self.registered_public_handlers
        );
        tracing::debug!(
            "Registered private handlers: {:?}",
            self.registered_private_handlers
        );
        tracing::debug!(
            "Registered server message handlers: {:?}",
            self.registered_server_messages_handlers
        );
        Nitram::new(
            inner,
            self.rpc_router_builder_public.build(),
            self.rpc_router_builder_private.build(),
            self.rpc_router_builder_server_messages.build(),
            self.registered_public_handlers,
            self.registered_private_handlers,
            self.registered_server_messages_handlers,
            self.ping_interval_in_seconds,
            self.server_messages_interval_in_millis,
            self.timeout_in_seconds,
            self.max_frame_size,
        )
    }
}
