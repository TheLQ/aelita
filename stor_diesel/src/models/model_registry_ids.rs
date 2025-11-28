use crate::models::date::StorDate;
use diesel::prelude::*;

#[derive(Queryable, Selectable, Insertable, Debug)]
#[diesel(table_name = crate::schema::registry_ids)]
pub struct ModelRegistryId {
    pub xrn: String,
    pub published: StorDate,
    pub publish_cause: String,
}
