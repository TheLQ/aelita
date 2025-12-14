use crate::StorDieselError;
use crate::api::api_journal::{storapi_journal_immutable_push_single, storapi_reset_journal};
use crate::api::api_space::{storapi_reset_space, storapi_space_new, storapi_space_owned_new};
use crate::api::assert_test_database;
use crate::connection::{StorConnection, StorTransaction};
use crate::err::StorDieselResult;
use crate::models::date::StorDate;
use crate::models::id_types::ModelJournalTypeName;
use crate::models::model_journal::{ModelJournalImmutable, NewModelJournalImmutable};
use crate::models::model_space::{ModelSpaceOwned, NewModelSpaceNames};
use crate::util_types::RawDieselBytes;
use xana_commons_rs::tracing_re::info;

pub fn create_todo_list(conn: &mut StorConnection) -> StorDieselResult<()> {
    info!("TheWhiteBoard");

    let current_time = StorDate::now();
    Model::synthesize(conn, current_time.clone())?;
    Ok(())
}

#[derive(Default)]
struct Model {
    projects: Vec<ModelJournalImmutable>,
    current_time: StorDate,
}

impl Model {
    pub fn synthesize(conn: &mut StorConnection, current_time: StorDate) -> StorDieselResult<()> {
        // #[derive(QueryableByName)]
        // #[diesel(check_for_backend(diesel::mysql::Mysql))]
        // pub struct CountType {
        //     #[diesel(sql_type = String)]
        //     pub count: String,
        // }
        // let res = sql_query("SELECT @@SESSION.sql_mode").get_result::<CountType>(conn)?;

        let mut model = Self::default();
        model.current_time = current_time.clone();

        model.reset(conn)?;
        model.space_create(conn)?;
        // model.projects_initial_2(conn)?;
        // model.register_project_xrn(conn)?;

        Ok(())
    }

    fn reset(&mut self, conn: &mut StorConnection) -> StorDieselResult<()> {
        StorTransaction::new_transaction("reset", conn, |conn| {
            assert_test_database(conn)?;
            storapi_reset_space(conn)?;
            storapi_reset_journal(conn)
        })
    }

    fn space_create(&mut self, conn: &mut StorConnection) -> StorDieselResult<()> {
        info!("start space 1");
        let journal_id =
            StorTransaction::new_transaction::<_, StorDieselError>("new-journal", conn, |conn| {
                storapi_journal_immutable_push_single(
                    conn,
                    NewModelJournalImmutable {
                        journal_type: ModelJournalTypeName::Space1,
                        data: RawDieselBytes("hello_world".as_bytes().to_vec()),
                        metadata: None,
                        cause_description: "space 1 create".into(),
                        cause_xrn: None,
                    },
                )
                .map_err(Into::into)
            })?;

        info!("synth space");
        StorTransaction::new_transaction("new-space", conn, |conn| {
            let space_id = storapi_space_new(
                conn,
                NewModelSpaceNames {
                    journal_id,
                    space_name: "space1".into(),
                    description: "some space".into(),
                },
            )?;
            storapi_space_owned_new(
                conn,
                &[ModelSpaceOwned {
                    journal_id,
                    space_id,
                    description: "test".into(),
                    child_xrn: format!("xrn:import:{journal_id}"),
                }],
            )
        })?;

        Ok(())
    }

    fn space_torrents_add() {
        todo!()
    }

    fn space_torrents_update() {
        todo!()
    }
}
