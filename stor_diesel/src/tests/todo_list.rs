use crate::api::api_project::{
    storapi_project_names_list, storapi_project_names_list_range, storapi_project_names_push,
};
use crate::api::api_registry_ids::{storapi_registry_ids_push, storapi_registry_ids_reset};
use crate::date_wrapper::StorDate;
use crate::err::StorDieselResult;
use crate::models::{NewModelProject, NewModelRegistryId};
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

pub fn synthesize(
    conn: &mut MysqlConnection,
    current_time: StorDate,
    model: Model,
) -> StorDieselResult<()> {
    let added_rows = storapi_registry_ids_reset(conn)?;
    info!("reset registry of {} rows", added_rows);

    storapi_registry_ids_push(conn, model.registry_ids)?;
    info!("push registry");

    Ok(())
}

#[derive(Default)]
struct Model {
    registry_ids: Vec<NewModelRegistryId>,
    project_names: Vec<NewModelProject>,
}

impl Model {
    pub fn synthesize(conn: &mut MysqlConnection, current_time: StorDate) -> StorDieselResult<()> {
        let mut model = Self::default();

        model.project_names.push(NewModelProject {
            title: "alpha".into(),
            description: "what what??".into(),
            published: current_time.clone(),
            publish_cause: "todo_list init".into(),
        });
        model.project_names.push(NewModelProject {
            title: "beta".into(),
            description: "hell yea brother??".into(),
            published: current_time.clone(),
            publish_cause: "todo_list init".into(),
        });
        model.project_names.push(NewModelProject {
            title: "gamma".into(),
            description: "yeaoo??".into(),
            published: current_time.clone(),
            publish_cause: "todo_list init".into(),
        });

        let output_projects_ids = storapi_project_names_push(conn, model.project_names)?;
        let output_projects = storapi_project_names_list_range(conn, output_projects_ids)?;
        for project in output_projects {
            info!("project {:?}", project);
        }

        // let project_alpha_xrn = ProjectTypeXrn::Paper.into_xrn(1);
        // let project_beta_xrn = ProjectTypeXrn::Paper.into_xrn(2);
        // let project_gamma_xrn = ProjectTypeXrn::Paper.into_xrn(3);
        // let all_projects = [project_alpha_xrn, project_beta_xrn, project_gamma_xrn].clone();
        //
        // for project in all_projects {
        //     model.registry_ids.push(NewModelRegistryId {
        //         xrn: project.into_addr(),
        //         published: current_time.clone(),
        //         publish_cause: "todo_list init".into(),
        //     });
        // }
        //

        Ok(())
    }
}
