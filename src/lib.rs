extern crate proc_macro;

use ts_rs::TS;

pub use rpc_router::FromResources;
pub use rpc_router::IntoParams;

mod builder;
mod concierge;
mod messages;

pub mod auth;
pub mod error;
pub mod models;
pub mod nice;
pub mod ws;
pub use concierge::*;

pub use builder::ConciergeBuilder;

#[derive(TS)]
#[ts(export, export_to = "Concierge.ts")]
pub struct EmptyParams;

#[macro_export]
macro_rules! concierge_api {
    ( $name:ident, $params_ty:ty, $output_ty:ty ) => {
        #[derive(TS)]
        #[ts(export, export_to = "API/index.ts")]
        struct $name {
            i: $params_ty,
            o: $output_ty,
        }
    };
}

#[macro_export]
macro_rules! concierge_handler {
    (
        $name:ident,
        $params_ty:ident,
        $output_ty:ty,
        $( $param_name:ident : $param_ty:ty ),*
    ) => {
        #[derive(Deserialize, Clone, TS)]
        #[ts(export, export_to = "API/Jobs.ts")]
        pub struct $params_ty {
            $(
                $param_name: $param_ty,
            )*
        }

        impl IntoParams for $params_ty {}


        #[derive(TS)]
        #[ts(export, export_to = "API/index.ts")]
        struct $name {
            i: $params_ty,
            o: $output_ty,
        }
    };
    (
        $name:ident,
        $output_ty:ty
    ) => {
        #[derive(TS)]
        #[ts(export, export_to = "API/index.ts")]
        struct $name {
            i: EmptyParams,
            o: $output_ty,
        }
    }
}
