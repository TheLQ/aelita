use crate::models::date::StorDate;
use aelita_xrn::defs::project_xrn::{ProjectTypeXrn, ProjectXrn};
use diesel::{Insertable, Queryable, Selectable};

#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = crate::schema::aproject_names)]
pub struct ModelProjectName {
    xrn_project_id: u32,
    pub title: String,
    pub published: StorDate,
    description: String,
}

impl ModelProjectName {
    pub fn xrn(&self) -> ProjectXrn {
        ProjectXrn::new(ProjectTypeXrn::Paper, self.xrn_project_id)
    }
}

//

#[derive(Insertable, Debug)]
#[diesel(table_name = crate::schema::aproject_names)]
pub struct NewModelProjectName {
    pub title: String,
    pub description: String,
    pub published: StorDate,
    pub publish_cause: String,
}
