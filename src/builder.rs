use rpc_router::{FromResources, Handler, RouterBuilder};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::{Nitram, NitramInner};

pub struct NitramBuilder {
    rpc_router_builder_public: RouterBuilder,
    rpc_router_builder_private: RouterBuilder,
    rpc_router_builder_signals: RouterBuilder,
    registered_public_handlers: Vec<String>,
    registered_private_handlers: Vec<String>,
    registered_signal_handlers: Vec<String>,
}

impl Default for NitramBuilder {
    fn default() -> Self {
        NitramBuilder {
            rpc_router_builder_public: RouterBuilder::default(),
            rpc_router_builder_private: RouterBuilder::default(),
            rpc_router_builder_signals: RouterBuilder::default(),
            registered_public_handlers: vec![],
            registered_private_handlers: vec![],
            registered_signal_handlers: vec![],
        }
    }
}

impl NitramBuilder {
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
        self.rpc_router_builder_signals = self
            .rpc_router_builder_signals
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

    pub fn add_signal_handler<H, T, P, R>(mut self, name: &'static str, handler: H) -> Self
    where
        H: Handler<T, P, R> + Clone + Send + Sync + 'static,
        T: Send + Sync + 'static,
        P: Send + Sync + 'static,
        R: Send + Sync + 'static,
    {
        self.registered_signal_handlers.push(name.to_string());
        self.rpc_router_builder_signals = self
            .rpc_router_builder_signals
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
            "Registered signal handlers: {:?}",
            self.registered_signal_handlers
        );
        Nitram::new(
            self.rpc_router_builder_public.build(),
            self.rpc_router_builder_private.build(),
            self.rpc_router_builder_signals.build(),
            self.registered_public_handlers,
            self.registered_private_handlers,
            self.registered_signal_handlers,
            inner,
        )
    }
}
