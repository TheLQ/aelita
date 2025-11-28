use crate::api::api_project::{storapi_project_names_push_and_get, storapi_project_names_reset};
use crate::api::api_registry_ids::{storapi_registry_ids_push, storapi_registry_ids_reset};
use crate::connection::StorConnection;
use crate::err::StorDieselResult;
use crate::models::date::StorDate;
use crate::models::model_project_laser::{ModelProjectLaserSql, NewModelProjectLaserSql};
use crate::models::{ModelProjectName, ModelRegistryId, NewModelProjectName};
use aelita_commons::tracing_re::info;
use diesel::{QueryableByName, RunQueryDsl, Selectable, sql_query};

pub fn create_todo_list(conn: &mut StorConnection) -> StorDieselResult<()> {
    info!("TheWhiteBoard");

    let current_time = StorDate::now();
    Model::synthesize(conn, current_time.clone())?;
    Ok(())
}

#[derive(Default)]
struct Model {
    projects: Vec<ModelProjectName>,
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

        model.projects_initial_1(conn)?;
        model.projects_initial_2(conn)?;
        model.register_project_xrn(conn)?;

        Ok(())
    }

    fn reset(&mut self, conn: &mut StorConnection) -> StorDieselResult<()> {
        let added_rows = storapi_registry_ids_reset(conn)?;
        info!("reset registry_ids of {} rows", added_rows);
        let added_rows = storapi_project_names_reset(conn)?;
        info!("reset project_names of {} rows", added_rows);
        Ok(())
    }

    fn projects_initial_1(&mut self, conn: &mut StorConnection) -> StorDieselResult<()> {
        info!("start push1");
        let mut project_names: Vec<NewModelProjectName> = Vec::new();
        project_names.push(NewModelProjectName {
            title: "alpha".into(),
            description: "what what??".into(),
            published: self.current_time.clone(),
            publish_cause: "todo_list init".into(),
        });

        let mut output_projects = storapi_project_names_push_and_get(conn, project_names)?;
        for project in &output_projects {
            info!("Inserted Project {:?}", project);
        }
        self.projects.append(&mut output_projects);
        Ok(())
    }

    fn projects_initial_2(&mut self, conn: &mut StorConnection) -> StorDieselResult<()> {
        info!("start push2");
        let mut project_names = Vec::new();
        project_names.push(NewModelProjectName {
            title: "beta".into(),
            description: "hell yea brother??".into(),
            published: self.current_time.clone(),
            publish_cause: "todo_list init".into(),
        });
        project_names.push(NewModelProjectName {
            title: "gamma".into(),
            description: "yeaoo??".into(),
            published: self.current_time.clone(),
            publish_cause: "todo_list init".into(),
        });

        let mut output_projects = storapi_project_names_push_and_get(conn, project_names)?;
        for project in &output_projects {
            info!("project {:?}", project);
        }
        self.projects.append(&mut output_projects);
        Ok(())
    }

    fn register_project_xrn(&mut self, conn: &mut StorConnection) -> StorDieselResult<()> {
        todo!()
        // let new: Vec<ModelRegistryId> = self
        //     .projects
        //     .iter()
        //     .map(|v| ModelRegistryId {
        //         xrn: v.xrn().into_addr(),
        //         published: self.current_time.clone(),
        //         publish_cause: "todo_list init".into(),
        //     })
        //     .collect();
        // let new_len = new.len();
        // storapi_registry_ids_push(conn, new)?;
        // info!("registry new ids {}", new_len);
        //
        // Ok(())
    }

    fn task_initial_1(&mut self, conn: &mut StorConnection) -> StorDieselResult<()> {
        let mut laser_names = Vec::new();
        let laser_names_bases = vec!["nissan", "mazda", "ford", "chevy"];
        let level_1_max = 5;
        let level_2_max = 3;
        for base in laser_names_bases {
            for level_1 in 0..level_1_max {
                for level_2 in 0..level_2_max {
                    laser_names.push(format!("{}-{}.{}", base, level_1, level_2));
                }
            }
        }

        let mut lasers = Vec::new();
        for laser_name in laser_names {
            lasers.push(NewModelProjectLaserSql {
                published: self.current_time.clone(),
                publish_cause: "init".into(),
                title: "title".into(),
                description: "somedwz".into(),
            });
        }

        Ok(())
    }
}
