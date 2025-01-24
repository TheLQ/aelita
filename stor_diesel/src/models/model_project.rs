use crate::date_wrapper::StorDate;
use crate::err::StorDieselError;
use crate::gen_try_from_converter;
use aelita_xrn::defs::project_xrn::{ProjectTypeXrn, ProjectXrn};
use diesel::{Insertable, Queryable, Selectable};
use serde::Deserialize;

#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = crate::schema::aproject_names)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct ModelProjectSql {
    xrn_project_id: u32,
    title: String,
    published: String,
    description: String,
}

#[derive(Debug, Deserialize)]
pub struct ModelProject {
    pub xrn_project_id: u32,
    pub title: String,
    pub published: StorDate,
    pub description: String,
}

impl ModelProject {
    pub fn xrn(&self) -> ProjectXrn {
        ProjectXrn::new(ProjectTypeXrn::Paper, self.xrn_project_id)
    }
}

gen_try_from_converter!(
    ModelProject,
    ModelProjectSql,
    (title, description, xrn_project_id),
    (published, |v: StorDate| v.to_stor_string()),
);

gen_try_from_converter!(
    ModelProjectSql,
    ModelProject,
    (title, description, xrn_project_id),
    (published, StorDate::from_string),
);

//

#[derive(Insertable, Debug)]
#[diesel(table_name = crate::schema::aproject_names)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct NewModelProjectSql {
    title: String,
    description: String,
    published: String,
    publish_cause: String,
}

#[derive(Deserialize)]
pub struct NewModelProject {
    pub title: String,
    pub description: String,
    pub published: StorDate,
    pub publish_cause: String,
}

gen_try_from_converter!(
    NewModelProject,
    NewModelProjectSql,
    (title, description, publish_cause),
    (published, |v: StorDate| v.to_stor_string()),
);
