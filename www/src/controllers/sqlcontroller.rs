use crate::err::WebResult;
use aelita_commons::err_utils::xbt;
use aelita_commons::tracing_re::info;
use aelita_stor_diesel::api::api_xrn_registry::storapi_xrns_push;
use aelita_stor_diesel::connection::load_db_url_from_env;
use aelita_stor_diesel::diesel_re::dsl::insert_into;
use aelita_stor_diesel::diesel_re::prelude::*;
use aelita_stor_diesel::err::{StorDieselError, StorDieselResult};
use aelita_stor_diesel::models::*;
use aelita_stor_diesel::schema::*;
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

    // pub async fn xrns_list(&self) -> WebResult<Vec<XrnExtraction>> {
    //     // let conn = self.pool.get().await?;
    //     // let res = conn.interact(|conn| storapi_xrns_list(conn)).await??;
    //     // Ok(res)
    //     self.query_stor(storapi_xrns_list).await
    // }

    pub async fn xrns_push(&self, new: Vec<NewXrnExtraction>) -> WebResult<()> {
        let conn = self.pool.get().await?;
        conn.interact(|conn| storapi_xrns_push(conn, new)).await??;
        Ok(())
    }

    pub async fn project_names(&self) -> WebResult<Vec<ModelProject>> {
        // let conn = self.pool.get().await?;
        // let query = conn
        //     .interact(|conn| {
        //         aproject_names::table
        //             .select(ModelProjectSql::as_select())
        //             .load(conn)
        //     })
        //     .await??;
        // let result: Vec<ModelProject> = query.into_iter().map(|e| e.try_into()).try_collect()?;
        // Ok(result)
        todo!()
    }

    pub async fn project_names_push(&self, new: Vec<ModelProject>) -> WebResult<()> {
        // let new: Vec<ModelProjectSql> = new.into_iter().map(|v| v.try_into()).try_collect()?;
        //
        // let conn = self.pool.get().await?;
        // conn.interact(|conn| {
        //     check_insert_num_rows(
        //         new.len(),
        //         insert_into(aproject_names::table).values(new).execute(conn),
        //     )
        // })
        // .await??;
        // Ok(())
        todo!()
    }
}
