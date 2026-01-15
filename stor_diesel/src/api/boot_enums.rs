use crate::{ModelHdRoot, ModelJournalTypeName, StorDieselResult, StorTransaction};
use aelita_xrn::defs::address::XrnType;
use aelita_xrn::defs::path_xrn::PathXrnType;
use aelita_xrn::defs::space_xrn::SpaceXrnType;
use diesel::RunQueryDsl;
use strum::VariantArray;
use xana_commons_rs::CommaJoiner;

pub fn bootstrap_enum_space_owned(conn: &mut StorTransaction) -> StorDieselResult<()> {
    let primary_keys = type_names::<XrnType>();
    diesel::sql_query(alter_enum_query("space_owned", "child_type1", primary_keys))
        .execute(conn.inner())?;

    let mut secondary_keys = String::new();
    secondary_keys.push_str(&type_names::<SpaceXrnType>());
    secondary_keys.push_str(", ");
    secondary_keys.push_str(&type_names::<PathXrnType>());
    diesel::sql_query(alter_enum_query(
        "space_owned",
        "child_type2",
        secondary_keys,
    ))
    .execute(conn.inner())?;
    Ok(())
}

pub fn bootstrap_enum_journal(conn: &mut StorTransaction) -> StorDieselResult<()> {
    let journal_keys = type_names::<ModelJournalTypeName>();
    diesel::sql_query(alter_enum_query(
        "journal_immutable",
        "journal_type",
        journal_keys,
    ))
    .execute(conn.inner())?;
    Ok(())
}

pub fn bootstrap_enum_hd_roots(conn: &mut StorTransaction) -> StorDieselResult<()> {
    diesel::sql_query(alter_enum_query(
        "hd1_roots",
        "rtype",
        type_names::<ModelHdRoot>(),
    ))
    .execute(conn.inner())?;
    Ok(())
}

fn type_names<'a, V: VariantArray + AsRef<str>>() -> String {
    V::VARIANTS
        .into_iter()
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
