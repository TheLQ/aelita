use crate::date_wrapper::StorDate;
use crate::err::StorDieselError;
use aelita_xrn::defs::project_xrn::ProjectXrn;
use diesel::{Insertable, Queryable, Selectable};
use serde::Deserialize;
use std::str::FromStr;

#[derive(Queryable, Selectable, Insertable, Debug)]
#[diesel(table_name = crate::schema::aproject_names)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct ModelProjectSql {
    xrn: String,
    title: String,
    published: String,
}

#[derive(Deserialize)]
pub struct ModelProject {
    pub xrn: ProjectXrn,
    pub title: String,
    pub published: StorDate,
}

impl TryFrom<ModelProjectSql> for ModelProject {
    type Error = StorDieselError;
    fn try_from(
        ModelProjectSql {
            xrn,
            title,
            published,
        }: ModelProjectSql,
    ) -> Result<Self, Self::Error> {
        Ok(ModelProject {
            xrn: ProjectXrn::from_str(&xrn)?,
            title,
            published: StorDate::from_str(&published)?,
        })
    }
}

impl From<ModelProject> for ModelProjectSql {
    fn from(
        ModelProject {
            xrn,
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
            xrn: xrn.to_string(),
            title,
            published,
        }
    }
}
