use crate::err::{StorImportErrorKind, StorImportResult};
use crate::integ_test::migration_sql_caller::MigrationModel;
use aelita_commons::log_init;
use aelita_stor_diesel::{
    ChangeOp, HdAddPath, HdAddSymlink, ModelJournalTypeName, NewModelJournalImmutable, PermaStore,
    RawDieselBytes, StorTransaction, bootstrap_enum_hd_roots, bootstrap_enum_journal,
    encode_compressed_paths, establish_connection, storapi_hd_get_path_by_path,
    storapi_journal_immutable_push_single,
};
use chrono::NaiveDateTime;
use xana_commons_rs::tracing_re::info;
use xana_commons_rs::{CrashErrKind, PrettyUnwrap, ResultXanaMap};
use xana_fs_indexer_rs::{CompressedPaths, ScanFileTypeWithPath, ScanStat};

// #[test]
pub fn sim_lyoko_main() {
    log_init();
    sim_lyoko().pretty_unwrap();
}

pub fn sim_lyoko() -> StorImportResult<()> {
    let conn = &mut establish_connection(PermaStore::AelitaInteg)
        .map_err(|(m, e)| StorImportErrorKind::DieselFailed.build_err_message(e, m))?;
    StorTransaction::new_transaction("sym-lyko", conn, simulate)
}

fn simulate(conn: &mut StorTransaction) -> StorImportResult<()> {
    let tables = [
        (MigrationModel::Journal, "journal_immutable"),
        (MigrationModel::Journal, "journal_immutable_data"),
        (MigrationModel::Hd, "hd1_files_components"),
    ];

    for (_model, table) in tables.iter().rev() {
        drop_table(conn, table)?;
    }
    for (model, table) in &tables {
        model.create_table(conn, table)?;
    }
    bootstrap_enum_journal(conn)?;

    journal_paths_backup(conn)?;
    journal_paths_active(conn)?;
    test_paths(conn)?;

    for (_model, table) in tables.iter().rev() {
        drop_table(conn, table)?;
    }

    Ok(())
}

fn journal_paths_backup(conn: &mut StorTransaction) -> StorImportResult<()> {
    let stat_dummy_usable = stat_dummy_usable();
    let compressed = CompressedPaths::from_scan(vec![
        (
            ScanFileTypeWithPath::Dir {
                path: "/backup".into(),
            },
            stat_dummy_usable.clone(),
        ),
        (
            ScanFileTypeWithPath::Dir {
                path: "/backup/active".into(),
            },
            stat_dummy_usable.clone(),
        ),
        (
            ScanFileTypeWithPath::Dir {
                path: "/backup/active/important_empty".into(),
            },
            stat_dummy_usable.clone(),
        ),
        (
            ScanFileTypeWithPath::File {
                path: "/backup/active/magic.rs".into(),
            },
            stat_dummy_usable.clone(),
        ),
    ])
    .map_err(StorImportErrorKind::DieselFailed.xana_map())?;
    let encoded = encode_compressed_paths(&compressed, None)
        .map_err(StorImportErrorKind::DieselFailed.xana_map())?;

    storapi_journal_immutable_push_single(
        conn,
        NewModelJournalImmutable {
            journal_type: ModelJournalTypeName::NData1,
            data: RawDieselBytes(encoded),
            metadata: None,
            cause_description: "simulated".into(),
            cause_xrn: None,
        },
    )?;
    Ok(())
}

fn journal_paths_active(conn: &mut StorTransaction) -> StorImportResult<()> {
    let stat_dummy_usable = stat_dummy_usable();
    let mut changes = Vec::new();
    changes.push(ChangeOp::HdAddPath(HdAddPath {
        paths: vec![
            (
                ScanFileTypeWithPath::File {
                    path: "/backup/active/more".into(),
                },
                stat_dummy_usable.clone(),
            ),
            (
                ScanFileTypeWithPath::Dir {
                    path: "/active".into(),
                },
                stat_dummy_usable.clone(),
            ),
        ],
    }));
    changes.push(ChangeOp::HdAddSymlink(HdAddSymlink {
        at: vec![b"active".to_vec()],
        target: vec![b"backup".to_vec(), b"active".to_vec()],
    }));

    let data = RawDieselBytes::serialize_json(changes)
        .map_err(StorImportErrorKind::DieselFailed.err_map())?;
    storapi_journal_immutable_push_single(
        conn,
        NewModelJournalImmutable {
            journal_type: ModelJournalTypeName::ChangeOp1,
            data,
            metadata: None,
            cause_description: "simulated".into(),
            cause_xrn: None,
        },
    )?;

    Ok(())
}

fn stat_dummy_usable() -> ScanStat {
    let some_date =
        NaiveDateTime::parse_from_str("2020-01-01T00:11:00.123456+00:00", "%+").unwrap();
    ScanStat {
        created: some_date.clone(),
        modified: some_date.clone(),
        size: 100,
        hard_links: 100,
        group_id: 100,
        user_id: 100,
    }
}

fn test_paths(conn: &mut StorTransaction) -> StorImportResult<()> {
    for tree_id in storapi_hd_get_path_by_path(conn, &[b"backup", b"active"])? {
        info!("found id {tree_id}")
    }
    Ok(())
}

fn drop_table(conn: &mut StorTransaction, table: &str) -> StorImportResult<()> {
    conn.raw_sql_execute(&format!("DROP TABLE `{}`", table))
        .xana_err(StorImportErrorKind::DieselFailed)?;
    Ok(())
}
