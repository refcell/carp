use crate::{auth::AuthService, db::Database, utils::Config};
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    pub auth_service: Arc<AuthService>,
    pub config: Arc<Config>,
}