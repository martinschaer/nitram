use derive_more::From;
use rpc_router::RpcHandlerError;
use serde::Serialize;

use crate::nice::{Nice, NiceMessage};

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, From)]
pub enum Error {
    MethodNotFound,
    NotAuthenticated,
    NotAuthorized,
    RpcRequestError(String),
    TokenError(String),

    // -- RPC
    #[from]
    RpcCallError(rpc_router::CallError),

    #[from]
    RpcRequestParsingError(rpc_router::RequestParsingError),
}

impl core::fmt::Display for Error {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}

impl std::error::Error for Error {}

pub type MethodResult<T> = core::result::Result<T, MethodError>;

#[derive(Debug, RpcHandlerError)]
pub enum MethodError {
    Server,
    NotFound,
    NotAuthorized,
    NotAuthenticated,
}

impl Serialize for MethodError {
    fn serialize<S>(&self, serializer: S) -> core::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            MethodError::NotFound => {
                serializer.serialize_str(&Nice::from(NiceMessage::NotFound).to_string())
            }
            MethodError::Server => {
                serializer.serialize_str(&Nice::from(NiceMessage::ServerError).to_string())
            }
            MethodError::NotAuthorized => {
                serializer.serialize_str(&Nice::from(NiceMessage::NotAuthorized).to_string())
            }
            MethodError::NotAuthenticated => {
                serializer.serialize_str(&Nice::from(NiceMessage::NotAuthenticated).to_string())
            }
        }
    }
}
