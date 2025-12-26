use aelita_commons::log_init;
use aelita_stor_diesel::{
    ModelJournalId, ModelSpaceOwned, NewModelSpaceName, PermaStore, StorDieselResult, StorIdType,
    StorTransaction, XrnAsOwnedTable, establish_connection, establish_connection_or_panic,
    storapi_reset_space, storapi_space_new, storapi_space_owned_new,
};
use aelita_xrn::defs::path_xrn::{PathXrn, PathXrnType};
use std::process::ExitCode;
use xana_commons_rs::pretty_main;

fn main() -> ExitCode {
    log_init();
    pretty_main(run)
}

pub fn run() -> StorDieselResult<()> {
    let conn = &mut establish_connection_or_panic(PermaStore::AelitaNull);

    StorTransaction::new_transaction("csfd", conn, generate)?;
    Ok(())
}

fn generate(conn: &mut StorTransaction) -> StorDieselResult<()> {
    storapi_reset_space(conn)?;

    let space_id = storapi_space_new(
        conn,
        NewModelSpaceName {
            journal_id: ModelJournalId::new(1),
            space_name: "test".to_string(),
            description: "test".to_string(),
        },
    )?;

    let some_path_xrn = PathXrn::new_id(PathXrnType::Fs, 88);
    storapi_space_owned_new(
        conn,
        [(
            ModelSpaceOwned {
                journal_id: ModelJournalId::new(1),
                space_id,
                description: Some("fdf".into()),
            },
            XrnAsOwnedTable::try_from(some_path_xrn)?,
        )],
    )?;

    Ok(())
}
