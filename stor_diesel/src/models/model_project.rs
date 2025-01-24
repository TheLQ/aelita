use crate::date_wrapper::StorDate;
use crate::err::StorDieselError;
use aelita_xrn::defs::project_xrn::{ProjectTypeXrn, ProjectXrn};
use diesel::{Insertable, Queryable, Selectable};
use serde::Deserialize;
use std::str::FromStr;

#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = crate::schema::aproject_names)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct ModelProjectSql {
    xrn_project_id: i32,
    title: String,
    published: String,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = crate::schema::aproject_names)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct NewModelProjectSql {
    title: String,
    published: String,
}

#[derive(Deserialize)]
pub struct ModelProject {
    xrn_project_id: u32,
    pub title: String,
    pub published: StorDate,
}

impl ModelProject {
    fn maybe_xrn(&self) -> ProjectXrn {
        ProjectXrn::new(ProjectTypeXrn::Paper, self.xrn_project_id)
    }
}

impl TryFrom<ModelProjectSql> for ModelProject {
    type Error = StorDieselError;
    fn try_from(
        ModelProjectSql {
            xrn_project_id,
            title,
            published,
        }: ModelProjectSql,
    ) -> Result<Self, Self::Error> {
        let xrn_project_id: u32 = xrn_project_id.try_into()?;
        Ok(ModelProject {
            xrn_project_id,
            title,
            published: StorDate::from_str(&published)?,
        })
    }
}

impl From<ModelProject> for ModelProjectSql {
    fn from(
        ModelProject {
            xrn_project_id,
            title,
            published,
        }: ModelProject,
    ) -> Self {
        let published = published.to_stor_string();
        assert!(
            published.len() <= 25,
            "ModelProject_len {} for {}",
            published.len(),
            published
        );
        ModelProjectSql {
            xrn_project_id: xrn.id(),
            title,
            published,
        }
    }
}
