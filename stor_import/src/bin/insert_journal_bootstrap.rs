use aelita_commons::log_init;
use aelita_stor_diesel::{
    ChangeOp, HdAddPath, HdAddSymlink, ModelJournalTypeName, NewModelJournalImmutable, PermaStore,
    RawDieselBytes, StorTransaction, convert_path_to_comps, convert_path_to_comps_owned,
    establish_connection, storapi_journal_immutable_push_single,
};
use aelita_stor_diesel::{HdAddRoot, ModelHdRoot};
use aelita_stor_import::err::{StorImportErrorKind, StorImportResult};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use xana_commons_rs::{CrashErrKind, pretty_main};
use xana_fs_indexer_rs::{ScanFileTypeWithPath, ScanStat};

fn main() -> ExitCode {
    log_init();
    pretty_main(run)
}

fn run() -> StorImportResult<()> {
    let conn = &mut establish_connection(PermaStore::AelitaNull)
        .map_err(|(m, e)| StorImportErrorKind::DieselFailed.build_err_message(e, m))?;
    StorTransaction::new_transaction("ins-jnl", conn, |conn| {
        let mut changes = Vec::new();
        push_symlinked_zfs_roots(&mut changes, Path::new("/big18"), Path::new("/dup18/big18"))?;
        push_symlinked_zfs_roots(&mut changes, Path::new("/hug24"), Path::new("/che12"))?;

        let encoded = RawDieselBytes::serialize_json(changes)
            .map_err(StorImportErrorKind::DieselFailed.err_map())?;
        storapi_journal_immutable_push_single(
            conn,
            NewModelJournalImmutable {
                journal_type: ModelJournalTypeName::ChangeOp1,
                data: encoded,
                metadata: None,
                cause_xrn: None,
                cause_description: "xana-bootstrap".to_string(),
            },
        )
        .map_err(StorImportErrorKind::DieselFailed.xana_map())
    })?;

    Ok(())
}

fn push_symlinked_zfs_roots(
    changes: &mut Vec<ChangeOp>,
    active: &Path,
    backup: &Path,
) -> StorImportResult<()> {
    let name = active.file_name().unwrap().to_str().unwrap();

    changes.push(ChangeOp::HdAddRoot(HdAddRoot {
        source: convert_path_to_comps_owned(active)
            .map_err(StorImportErrorKind::DieselFailed.xana_map())?,
        description: "ZFS Root Active".to_string(),
        space_name: format!("zfs-{name}"),
        root_type: ModelHdRoot::ZfsDataset,
    }));
    changes.push(ChangeOp::HdAddRoot(HdAddRoot {
        source: convert_path_to_comps_owned(backup)
            .map_err(StorImportErrorKind::DieselFailed.xana_map())?,
        description: "ZFS Root Backup".to_string(),
        space_name: format!("zfs-{name}"),
        root_type: ModelHdRoot::ZfsDataset,
    }));

    // todo: this should fail as out of order from the hd add root?
    changes.push(add_path_from_fs(active.to_path_buf())?);
    changes.push(ChangeOp::HdAddSymlink(HdAddSymlink {
        at: convert_path_to_comps_owned(active)
            .map_err(StorImportErrorKind::DieselFailed.xana_map())?,
        target: convert_path_to_comps_owned(backup)
            .map_err(StorImportErrorKind::DieselFailed.xana_map())?,
    }));

    Ok(())
}

fn add_path_from_fs(path: PathBuf) -> StorImportResult<ChangeOp> {
    let stat = ScanStat::new(&path).map_err(StorImportErrorKind::DieselFailed.err_map())?;
    Ok(ChangeOp::HdAddPath(HdAddPath {
        paths: vec![(ScanFileTypeWithPath::Dir { path }, stat)],
    }))
}
