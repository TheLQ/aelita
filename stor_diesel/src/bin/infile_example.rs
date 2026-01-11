use aelita_commons::log_init;
use aelita_stor_diesel::StorInstrument;
use diesel::connection::SimpleConnection;
use diesel::mysql::Mysql;
use diesel::query_builder::AsQuery;
use diesel::{Connection, IntoSql, MysqlConnection, QueryDsl, RunQueryDsl, debug_query};
use std::fmt::Display;
use std::fs::{File, OpenOptions};
use std::hash::DefaultHasher;
use std::io::Write;
use std::path::Path;
use xana_commons_rs::tracing_re::info;

fn main() {
    log_init();
    let url = "mysql://root:codelyoko@localhost/aelita_null?\
            unix_socket=/intrasock/aelita-dbmy.socket";
    // &\
    // local_infile=1";
    // let url = "mysql://u:p@localhost/db?\
    //         unix_socket=/mysql.socket&\
    //         local_infile=1";
    let conn = &mut MysqlConnection::establish(url).unwrap();
    conn.set_instrumentation(StorInstrument::default());

    conn.batch_execute(
        "CREATE TEMPORARY TABLE fast_table (\
        id INT NOT NULL PRIMARY KEY,\
        value BLOB NOT NULL\
        )",
    )
    .unwrap();

    let load_path = Path::new("mysql-rows.dat").canonicalize().unwrap();
    // write_rows_file(&load_path);
    // load_rows_file(conn, &load_path);
    select_diesel(conn);
    select_outfile(conn);
}

diesel::table! {
    fast_table (id) {
        id -> Integer,
        value -> Varchar,
    }
}

const ROW_SEP: u8 = 0x1e;
const COL_SEP: u8 = 0x1f;
fn write_rows_file(path: &Path) -> () {
    std::fs::remove_file(&path).unwrap();
    let mut out_file = File::create(path).unwrap();
    for row in 0..10 {
        let something = "something";
        out_file
            .write_fmt(format_args!("{row}{COL_SEP}{something}{ROW_SEP}"))
            .unwrap();
    }
}

fn load_rows_file(conn: &mut MysqlConnection, path: &Path) {
    conn.batch_execute(&format!(
        "LOAD DATA LOCAL INFILE '{}' \
        INTO TABLE `fast_table` \
        FIELDS TERMINATED BY '{COL_SEP}' \
        LINES TERMINATED BY '{ROW_SEP}' \
        (id, value)",
        path.display()
    ))
    .unwrap();
}

fn select_diesel(conn: &mut MysqlConnection) {
    for (id, value) in fast_table::table
        .get_results::<(i32, String)>(conn)
        .unwrap()
    {
        info!("row {id} value '{value}'",);
    }
}

fn select_outfile(conn: &mut MysqlConnection) {
    diesel::sql_query(format!(
        "SELECT * FROM fast_table \
        INTO OUTFILE '/var/lib/mysql-files/select.dat' \
        FIELDS TERMINATED BY '{COL_SEP}' \
        LINES TERMINATED BY '{ROW_SEP}'",
    ))
    .execute(conn)
    .unwrap();
}
