use crate::err::{WebError, WebResult};
use aelita_commons::err_utils::xbt;
use aelita_commons::tracing_re::info;
use aelita_stor_diesel::connection::load_db_url_from_env;
use aelita_stor_diesel::diesel_re::dsl::insert_into;
use aelita_stor_diesel::diesel_re::prelude::*;
use aelita_stor_diesel::models::model_project::{ModelProject, ModelProjectSql};
use aelita_stor_diesel::models::*;
use aelita_stor_diesel::schema::{aproject_names, xrn_registry};
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
        conn.interact(|conn| {
            check_insert_num_rows(
                new.len(),
                insert_into(xrn_registry::table).values(new).execute(conn),
            )
        })
        .await??;
        Ok(())
    }

    pub async fn project_names(&self) -> WebResult<Vec<ModelProject>> {
        let conn = self.pool.get().await?;
        let query = conn
            .interact(|conn| {
                aproject_names::table
                    .select(ModelProjectSql::as_select())
                    .load(conn)
            })
            .await??;
        let result: Vec<ModelProject> = query.into_iter().map(|e| e.try_into()).try_collect()?;
        Ok(result)
    }

    pub async fn project_names_push(&self, new: Vec<ModelProject>) -> WebResult<()> {
        let new: Vec<ModelProjectSql> = new.into_iter().map(|v| v.into()).collect();

        let conn = self.pool.get().await?;
        conn.interact(|conn| {
            check_insert_num_rows(
                new.len(),
                insert_into(aproject_names::table).values(new).execute(conn),
            )
        })
        .await??;
        Ok(())
    }
}

fn check_insert_num_rows(len: usize, query: QueryResult<usize>) -> WebResult<()> {
    match query {
        Ok(0) => Err(WebError::XrnRegistry_IsEmpty(xbt())),
        Ok(query_len) if len == query_len => Ok(()),
        Ok(_query_len) => Err(WebError::XrnRegistry_IsEmpty(xbt())),
        Err(err) => Err(WebError::Diesel(err, xbt())),
    }
}
