extern crate core;

use aelita_commons::log_init;
use aelita_stor_diesel::{
    PermaStore, StorDieselError, StorDieselResult, StorTransaction, establish_connection,
    show_create_table,
};
use aelita_xrn::defs::address::XrnType;
use aelita_xrn::defs::path_xrn::PathXrnType;
use aelita_xrn::defs::space_xrn::SpaceXrnType;
use diesel::RunQueryDsl;
use postcard::to_vec;
use std::process::ExitCode;
use xana_commons_rs::tracing_re::info;
use xana_commons_rs::{CommaJoiner, pretty_main};

fn main() -> ExitCode {
    log_init();

    pretty_main(run)
}

pub fn run() -> StorDieselResult<()> {
    let conn = &mut establish_connection(PermaStore::AelitaNull).unwrap();

    // StorTransaction::new_transaction("build", conn, |conn| {
    //     let create = show_create_table(conn.inner(), "space_owned")?;
    //     info!("create {create}");
    //     Ok::<(), StorDieselError>(())
    // })?;
    StorTransaction::new_transaction("build", conn, |conn| {
        let primary_keys = type_names(<XrnType as strum::VariantNames>::VARIANTS);
        diesel::sql_query(alter_enum_query("space_owned", "child_type1", primary_keys))
            .execute(conn.inner())?;

        let secondary_keys = type_names(
            [
                <SpaceXrnType as strum::VariantNames>::VARIANTS,
                <PathXrnType as strum::VariantNames>::VARIANTS,
            ]
            .into_iter()
            .flatten(),
        );
        diesel::sql_query(alter_enum_query(
            "space_owned",
            "child_type2",
            secondary_keys,
        ))
        .execute(conn.inner())?;

        // let create = show_create_table(conn.inner(), "space_owned")?;
        // info!("create {create}");

        Ok::<(), StorDieselError>(())
    })?;

    Ok(())
}

fn type_names<'a>(from: impl IntoIterator<Item = impl AsRef<str>>) -> String {
    from.into_iter()
        .map(|v| format!("'{}'", v.as_ref()))
        .collect::<CommaJoiner>()
        .value()
}

fn alter_enum_query(table: &str, key: &str, values: String) -> String {
    format!(
        "ALTER TABLE `{table}` MODIFY \
        {key} ENUM ( {values} ) NOT NULL"
    )
}
