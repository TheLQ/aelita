use crate::api::api_journal::{
    storapi_journal_immutable_push, storapi_journal_publish_push, storapi_reset_journal,
};
use crate::api::api_space::{storapi_reset_space, storapi_space_new, storapi_space_owned_new};
use crate::api::assert_test_database;
use crate::connection::{StorConnection, StorTransaction};
use crate::err::StorDieselResult;
use crate::models::date::StorDate;
use crate::models::id_types::ModelJournalTypeName;
use crate::models::model_journal::{
    ModelJournalDataImmutable, NewModelJournalDataImmutable, NewModelPublishLog,
};
use crate::models::model_space::{ModelSpaceOwned, NewModelSpaceNames};
use xana_commons_rs::tracing_re::info;

pub fn create_todo_list(conn: &mut StorConnection) -> StorDieselResult<()> {
    info!("TheWhiteBoard");

    let current_time = StorDate::now();
    Model::synthesize(conn, current_time.clone())?;
    Ok(())
}

#[derive(Default)]
struct Model {
    projects: Vec<ModelJournalDataImmutable>,
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
        model.space_create_1(conn)?;
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

    fn space_create_1(&mut self, conn: &mut StorConnection) -> StorDieselResult<()> {
        info!("start space 1");
        let (publish_id, journal_ids) =
            StorTransaction::new_transaction("new-journal", conn, |conn| {
                let publish_id = storapi_journal_publish_push(
                    conn,
                    NewModelPublishLog {
                        cause_description: "space 1 create".into(),
                        cause_xrn: None,
                    },
                )?;
                let journal_ids = storapi_journal_immutable_push(
                    conn,
                    [NewModelJournalDataImmutable {
                        publish_id,
                        journal_type: ModelJournalTypeName::Space1,
                        data: "hello_world".as_bytes().to_vec(),
                    }],
                )?;
                Ok((publish_id, journal_ids))
            })?;

        info!("synth space");
        StorTransaction::new_transaction("new-space", conn, |conn| {
            let space_id = storapi_space_new(
                conn,
                NewModelSpaceNames {
                    publish_id,
                    space_name: "space1".into(),
                    description: "some space".into(),
                },
            )?;
            storapi_space_owned_new(
                conn,
                &journal_ids
                    .into_iter()
                    .map(|journal_id| ModelSpaceOwned {
                        publish_id,
                        space_id,
                        description: "test".into(),
                        child_xrn: format!("xrn:import:{journal_id}"),
                    })
                    .collect::<Vec<_>>(),
            )
        })?;

        // let mut project_names: Vec<NewModelProjectName> = Vec::new();
        // project_names.push(NewModelProjectName {
        //     title: "alpha".into(),
        //     description: "what what??".into(),
        //     published: self.current_time.clone(),
        //     publish_cause: "todo_list init".into(),
        // });
        //
        // let mut output_projects = storapi_project_names_push_and_get(conn, project_names)?;
        // for project in &output_projects {
        //     info!("Inserted Project {:?}", project);
        // }
        // self.projects.append(&mut output_projects);
        Ok(())
    }

    // fn projects_initial_2(&mut self, conn: &mut StorConnection) -> StorDieselResult<()> {
    //     info!("start push2");
    //     let mut project_names = Vec::new();
    //     project_names.push(NewModelProjectName {
    //         title: "beta".into(),
    //         description: "hell yea brother??".into(),
    //         published: self.current_time.clone(),
    //         publish_cause: "todo_list init".into(),
    //     });
    //     project_names.push(NewModelProjectName {
    //         title: "gamma".into(),
    //         description: "yeaoo??".into(),
    //         published: self.current_time.clone(),
    //         publish_cause: "todo_list init".into(),
    //     });
    //
    //     let mut output_projects = storapi_project_names_push_and_get(conn, project_names)?;
    //     for project in &output_projects {
    //         info!("project {:?}", project);
    //     }
    //     self.projects.append(&mut output_projects);
    //     Ok(())
    // }
    //
    // fn register_project_xrn(&mut self, conn: &mut StorConnection) -> StorDieselResult<()> {
    //     todo!()
    //     // let new: Vec<ModelRegistryId> = self
    //     //     .projects
    //     //     .iter()
    //     //     .map(|v| ModelRegistryId {
    //     //         xrn: v.xrn().into_addr(),
    //     //         published: self.current_time.clone(),
    //     //         publish_cause: "todo_list init".into(),
    //     //     })
    //     //     .collect();
    //     // let new_len = new.len();
    //     // storapi_registry_ids_push(conn, new)?;
    //     // info!("registry new ids {}", new_len);
    //     //
    //     // Ok(())
    // }
    //
    // fn task_initial_1(&mut self, conn: &mut StorConnection) -> StorDieselResult<()> {
    //     let mut laser_names = Vec::new();
    //     let laser_names_bases = vec!["nissan", "mazda", "ford", "chevy"];
    //     let level_1_max = 5;
    //     let level_2_max = 3;
    //     for base in laser_names_bases {
    //         for level_1 in 0..level_1_max {
    //             for level_2 in 0..level_2_max {
    //                 laser_names.push(format!("{}-{}.{}", base, level_1, level_2));
    //             }
    //         }
    //     }
    //
    //     let mut lasers = Vec::new();
    //     for laser_name in laser_names {
    //         lasers.push(NewModelProjectLaserSql {
    //             published: self.current_time.clone(),
    //             publish_cause: "init".into(),
    //             title: "title".into(),
    //             description: "somedwz".into(),
    //         });
    //     }
    //
    //     Ok(())
    // }
}
