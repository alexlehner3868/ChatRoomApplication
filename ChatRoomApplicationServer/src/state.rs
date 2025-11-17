use sqlx::PgPool;
use std::sync::Arc;

use crate::AppState;

#[derive(Clone)]
pub struct GlobalState {
    pub db_pool: PgPool,
    pub app_state: Arc<AppState>,
}
