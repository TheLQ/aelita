use crate::err::WebResult;
use aelita_stor_diesel::load_db_url_from_env;
use aelita_stor_diesel::{PermaStore, StorTransaction, establish_connection};
use aelita_stor_diesel::{StorDieselResult, apply_stor_instrument};
use deadpool_diesel::mysql::{Hook, Manager, Pool};
use std::sync::Arc;
use std::time::SystemTime;
use xana_commons_rs::tracing_re::info;

#[derive(Clone)]
pub struct SqlState {
    pub sqlfs: Arc<SqlController>,
}

impl SqlState {
    pub fn new(store: PermaStore) -> Self {
        Self {
            sqlfs: Arc::new(SqlController::new(store)),
        }
    }
}

pub struct SqlController {
    pool: Pool,
}

impl SqlController {
    pub fn new(store: PermaStore) -> Self {
        info!("building sql pool");

        let db_url = load_db_url_from_env(store);
        let manager = Manager::new(db_url, deadpool_diesel::Runtime::Tokio1);
        let pool = Pool::builder(manager)
            .post_create(Hook::sync_fn(|conn, _metrics| {
                let mut conn = conn.lock().unwrap();
                apply_stor_instrument(&mut conn);
                Ok(())
            }))
            .build()
            .unwrap();

        Self { pool }
    }

    pub async fn transact<'a, F, R>(&self, inner: F) -> WebResult<R>
    where
        F: FnOnce(&mut StorTransaction) -> StorDieselResult<R> + Send + 'static,
        R: Send + 'static,
    {
        let conn = self.pool.get().await?;
        let result = conn
            .interact(|conn| {
                //
                StorTransaction::new_transaction("www", conn, inner)
            })
            .await??;
        Ok(result)
    }
}

pub fn basic_cause(input: &str) -> String {
    format!(
        "{input}-{}",
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    )
}
