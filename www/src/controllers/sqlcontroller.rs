use crate::err::{WebError, WebResult};
use aelita_commons::tracing_re::info;
use aelita_stor_diesel::connection::load_db_url_from_env;
use aelita_stor_diesel::diesel_re::dsl::insert_into;
use aelita_stor_diesel::diesel_re::prelude::*;
use aelita_stor_diesel::models::projects_model::ModelProject;
use aelita_stor_diesel::models::*;
use aelita_stor_diesel::schema::xrn_registry;
use deadpool_diesel::mysql::{Manager, Pool};
use std::backtrace::Backtrace;
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

    pub async fn xrns_list(&self) -> WebResult<Vec<XrnExtraction>> {
        let conn = self.pool.get().await?;
        let result = conn
            .interact(|conn| {
                xrn_registry::table
                    .select(XrnExtraction::as_select())
                    .load(conn)
            })
            .await??;
        Ok(result)
    }

    pub async fn xrns_push(&self, new: Vec<NewXrnExtraction>) -> WebResult<()> {
        let conn = self.pool.get().await?;
        conn.interact(
            |conn| match insert_into(xrn_registry::table).values(new).execute(conn) {
                Ok(affected_rows) if affected_rows == 0 => {
                    Err(WebError::XrnRegistry_IsEmpty(Backtrace::capture()))
                }
                Ok(_affected_rows) => Ok(()),
                Err(err) => Err(WebError::Diesel(err, Backtrace::capture())),
            },
        )
        .await??;
        Ok(())
    }

    pub async fn projects_list(&self) -> WebResult<Vec<ModelProject>> {
        let conn = self.pool.get().await?;
        let result = conn
            .interact(|conn| {
                xrn_registry::table
                    .select(ModelProject::as_select())
                    .load(conn)
            })
            .await??;
        Ok(result)
    }
}
