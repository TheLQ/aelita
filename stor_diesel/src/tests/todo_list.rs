use crate::api::api_registry_ids::{storapi_registry_ids_push, storapi_registry_ids_reset};
use crate::date_wrapper::StorDate;
use crate::err::StorDieselResult;
use crate::models::NewModelRegistryId;
use aelita_commons::num_format_re::Locale::vai;
use aelita_commons::tracing_re::info;
use aelita_xrn::defs::address::{XrnAddr, XrnAddrType};
use diesel::prelude::*;
use diesel::{MysqlConnection, RunQueryDsl};

pub fn create_todo_list(conn: &mut MysqlConnection) -> StorDieselResult<()> {
    info!("TheWhiteBoard");

    let current_time = StorDate::now();

    let added_rows = storapi_registry_ids_reset(conn)?;
    info!("reset registry of {} rows", added_rows);

    storapi_registry_ids_push(conn, vec![NewModelRegistryId {
        xrn: XrnAddr::new(XrnAddrType::Project, "paper/1".into()),
        published: current_time.clone(),
        publish_cause: "todo_list init".into(),
    }])?;
    info!("push registry");

    // let new_post = NewXrnExtraction {
    //     xrn: "asdf".into(),
    //     published: "asdf".into(),
    // };
    //
    // diesel::insert_into(xrn_registry)
    //     .values(&new_post)
    //     .execute(conn)
    //     .expect("Error saving new post");
    // println!("inserted post");
    //
    // let results = xrn_registry
    //     .select(XrnExtraction::as_select())
    //     .load(conn)
    //     .expect("Error loading posts");
    //
    // println!("Displaying {} posts", results.len());
    // for post in results {
    //     println!("{}", post.xrn);
    //     println!("-----------\n");
    //     println!("{}", post.published);
    // }

    Ok(())
}
