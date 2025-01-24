use crate::api::api_project::{
    storapi_project_names_list, storapi_project_names_list_range, storapi_project_names_push,
    storapi_project_names_reset,
};
use crate::api::api_registry_ids::{storapi_registry_ids_push, storapi_registry_ids_reset};
use crate::date_wrapper::StorDate;
use crate::err::StorDieselResult;
use crate::models::{ModelProject, NewModelProject, NewModelRegistryId};
use crate::schema::registry_ids::dsl::registry_ids;
use aelita_commons::tracing_re::info;
use aelita_xrn::defs::address::{XrnAddr, XrnAddrType};
use aelita_xrn::defs::project_xrn::{ProjectTypeXrn, ProjectXrn};
use diesel::prelude::*;
use diesel::{MysqlConnection, RunQueryDsl};

pub fn create_todo_list(conn: &mut MysqlConnection) -> StorDieselResult<()> {
    info!("TheWhiteBoard");

    let current_time = StorDate::now();
    let model = Model::synthesize(conn, current_time.clone())?;
    Ok(())
}

#[derive(Default)]
struct Model {
    projects: Vec<ModelProject>,
    current_time: StorDate,
}

impl Model {
    pub fn synthesize(conn: &mut MysqlConnection, current_time: StorDate) -> StorDieselResult<()> {
        let mut model = Self::default();
        model.current_time = current_time.clone();

        model.reset(conn)?;

        model.projects_initial_1(conn)?;
        model.projects_initial_2(conn)?;

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
        let mut project_names: Vec<NewModelProject> = Vec::new();
        project_names.push(NewModelProject {
            title: "alpha".into(),
            description: "what what??".into(),
            published: self.current_time.clone(),
            publish_cause: "todo_list init".into(),
        });

        let output_projects_ids = storapi_project_names_push(conn, project_names)?;
        let mut output_projects =
            storapi_project_names_list_range(conn, output_projects_ids.clone())?;
        for project in &output_projects {
            info!("Inserted Project {:?}", project);
        }
        self.projects.append(&mut output_projects);
        Ok(())
    }

    fn projects_initial_2(&mut self, conn: &mut MysqlConnection) -> StorDieselResult<()> {
        info!("start push2");
        let mut project_names = Vec::new();
        project_names.push(NewModelProject {
            title: "beta".into(),
            description: "hell yea brother??".into(),
            published: self.current_time.clone(),
            publish_cause: "todo_list init".into(),
        });
        project_names.push(NewModelProject {
            title: "gamma".into(),
            description: "yeaoo??".into(),
            published: self.current_time.clone(),
            publish_cause: "todo_list init".into(),
        });

        let output_projects_ids = storapi_project_names_push(conn, project_names)?;
        let mut output_projects = storapi_project_names_list_range(conn, output_projects_ids)?;
        for project in &output_projects {
            info!("project {:?}", project);
        }
        self.projects.append(&mut output_projects);
        Ok(())
    }
}
