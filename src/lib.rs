extern crate proc_macro;

use serde::Deserialize;
use ts_rs::TS;

pub use rpc_router::FromResources;
pub use rpc_router::IntoParams;

mod builder;
mod messages;
mod nitram;

pub mod auth;
pub mod error;
pub mod models;
pub mod nice;
pub mod ws;
pub use nitram::*;

pub use builder::NitramBuilder;

#[derive(TS)]
#[ts(export, export_to = "Nitram.ts")]
pub struct EmptyParams;

#[derive(Deserialize, TS)]
#[ts(export, export_to = "Nitram.ts")]
pub struct IdParams {
    pub id: String,
}
impl IntoParams for IdParams {}

#[macro_export]
macro_rules! nitram_api {
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
macro_rules! nitram_handler {
    (
        $name:ident,
        $params_ty:ident,
        $output_ty:ty,
        $( $param_name:ident : $param_ty:ty ),*
    ) => {
        #[derive(Deserialize, Clone, TS)]
        #[ts(export, export_to = "API/Params.ts")]
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
        $params_ty:ty,
        $output_ty:ty
    ) =>
    {
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
