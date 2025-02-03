use crate::api::api_project::{storapi_project_names_push_and_get, storapi_project_names_reset};
use crate::api::api_registry_ids::{storapi_registry_ids_push, storapi_registry_ids_reset};
use crate::err::StorDieselResult;
use crate::models::date::StorDate;
use crate::models::model_project_laser::{ModelProjectLaserSql, NewModelProjectLaserSql};
use crate::models::{ModelProjectName, ModelRegistryId, NewModelProjectName};
use aelita_commons::tracing_re::info;
use diesel::{MysqlConnection, QueryableByName, RunQueryDsl, Selectable, sql_query};

pub fn create_todo_list(conn: &mut MysqlConnection) -> StorDieselResult<()> {
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
    pub fn synthesize(conn: &mut MysqlConnection, current_time: StorDate) -> StorDieselResult<()> {
        let mut model = Self::default();
        model.current_time = current_time.clone();

        model.reset(conn)?;

        model.projects_initial_1(conn)?;
        model.projects_initial_2(conn)?;
        model.register_project_xrn(conn)?;

        Ok(())
    }

    fn reset(&mut self, conn: &mut MysqlConnection) -> StorDieselResult<()> {
        let added_rows = storapi_registry_ids_reset(conn)?;
        info!("reset registry_ids of {} rows", added_rows);
        let added_rows = storapi_project_names_reset(conn)?;
        info!("reset project_names of {} rows", added_rows);
        Ok(())
    }

    fn projects_initial_1(&mut self, conn: &mut MysqlConnection) -> StorDieselResult<()> {
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

    fn projects_initial_2(&mut self, conn: &mut MysqlConnection) -> StorDieselResult<()> {
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

    fn register_project_xrn(&mut self, conn: &mut MysqlConnection) -> StorDieselResult<()> {
        let new: Vec<ModelRegistryId> = self
            .projects
            .iter()
            .map(|v| ModelRegistryId {
                xrn: v.xrn().into_addr(),
                published: self.current_time.clone(),
                publish_cause: "todo_list init".into(),
            })
            .collect();
        let new_len = new.len();
        storapi_registry_ids_push(conn, new)?;
        info!("registry new ids {}", new_len);

        Ok(())
    }
}
