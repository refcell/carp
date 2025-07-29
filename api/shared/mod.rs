pub mod auth;

pub use auth::{
    authenticate_request, check_scope, AuthenticatedUser, ApiError, AuthResult, 
    unauthorized_error, forbidden_error,
};