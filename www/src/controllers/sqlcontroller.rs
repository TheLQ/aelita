use crate::err::WebResult;
use aelita_commons::tracing_re::info;
use aelita_stor_diesel::connection::load_db_url_from_env;
use aelita_stor_diesel::diesel_re::MysqlConnection;
use aelita_stor_diesel::err::StorDieselResult;
use deadpool_diesel::mysql::{Manager, Pool};
use std::sync::Arc;

#[derive(Clone)]
pub struct SqlState {
    pub sqlfs: Arc<SqlController>,
}

impl SqlState {
    pub fn new() -> Self {
        Self {
            sqlfs: Arc::new(SqlController::new()),
        }
    }
}

pub struct SqlController {
    pool: Pool,
}

impl SqlController {
    pub fn new() -> Self {
        info!("building sql pool");

        let db_url = load_db_url_from_env();
        let manager = Manager::new(db_url, deadpool_diesel::Runtime::Tokio1);
        let pool = Pool::builder(manager).build().unwrap();

        Self { pool }
    }

    pub async fn query_stor<'a, F, R>(&self, inner: F) -> WebResult<R>
    where
        F: FnOnce(&mut MysqlConnection) -> StorDieselResult<R> + Send + 'static,
        R: Send + 'static,
    {
        let conn = self.pool.get().await?;
        let result = conn.interact(inner).await??;
        Ok(result)
    }
}
