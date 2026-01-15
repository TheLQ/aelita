extern crate core;

use aelita_commons::log_init;
use aelita_stor_diesel::err::StorDieselErrorKind;
use aelita_stor_diesel::{
    ModelHdRoot, ModelJournalTypeName, PermaStore, StorDieselError, StorDieselResult,
    StorTransaction, bootstrap_enum_hd_roots, bootstrap_enum_journal, bootstrap_enum_space_owned,
    establish_connection,
};
use aelita_xrn::defs::address::XrnType;
use aelita_xrn::defs::path_xrn::PathXrnType;
use aelita_xrn::defs::space_xrn::SpaceXrnType;
use diesel::RunQueryDsl;
use std::process::ExitCode;
use strum::VariantArray;
use xana_commons_rs::{CommaJoiner, CrashErrKind, pretty_main};

fn main() -> ExitCode {
    log_init();

    pretty_main(run)
}

pub fn run() -> StorDieselResult<()> {
    let conn = &mut establish_connection(PermaStore::AelitaNull).map_err(|(db_url, e)| {
        StorDieselErrorKind::DatabaseConnectionFailed.build_err_message(e, db_url)
    })?;

    StorTransaction::new_transaction("build", conn, |conn| {
        bootstrap_enum_space_owned(conn)?;
        bootstrap_enum_journal(conn)?;
        bootstrap_enum_hd_roots(conn)?;

        Ok::<(), Box<StorDieselError>>(())
    })?;

    Ok(())
}
