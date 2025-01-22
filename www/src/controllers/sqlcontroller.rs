use aelita_commons::tracing_re::info;
use aelita_stor_diesel::connection::load_db_url_from_env;
use aelita_stor_diesel::diesel_re::prelude::*;
use aelita_stor_diesel::models::*;
use aelita_stor_diesel::schema::xrn_registry;
use deadpool_diesel::mysql::{Manager, Object, Pool};
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

type PResult<T> = Result<T, deadpool_diesel::PoolError>;

impl SqlController {
    pub fn new() -> Self {
        info!("building sql pool");

        let db_url = load_db_url_from_env();
        let manager = Manager::new(db_url, deadpool_diesel::Runtime::Tokio1);
        let pool = Pool::builder(manager).build().unwrap();

        Self { pool }
    }

    pub async fn xrns_list(&self) -> PResult<Vec<XrnExtraction>> {
        let conn = self.pool.get().await?;
        let res = conn
            .interact(|conn| {
                xrn_registry::table
                    .select(XrnExtraction::as_select())
                    .load(conn)
            })
            .await
            .unwrap();
        Ok(res.unwrap())
    }
}
