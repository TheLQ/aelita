use aelita_commons::tracing_re::trace;
use diesel::Connection;
use diesel::MysqlConnection;
use diesel::connection::{Instrumentation, InstrumentationEvent, LoadConnection};
use diesel::mysql::Mysql;
use dotenvy::dotenv;
use std::env;

pub fn load_db_url_from_env() -> String {
    dotenv().ok();
    env::var("DATABASE_URL").expect("DATABASE_URL must be set")
}

/// todo: How to do this with "impl Trait"?
// pub type StorConnection = InstrumentedMysqlConnection;
pub type StorConnection = impl Connection<Backend = Mysql> + LoadConnection;

pub fn establish_connection() -> StorConnection {
    let database_url = load_db_url_from_env();
    // InstrumentedMysqlConnection::establish(&database_url)
    let mut conn = MysqlConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url));
    conn.set_instrumentation(StorInstrument {});
    conn
}

struct StorInstrument;

impl Instrumentation for StorInstrument {
    fn on_connection_event(&mut self, event: InstrumentationEvent<'_>) {
        if let InstrumentationEvent::StartQuery { query, .. } = event {
            trace!("{}", query.to_string());
        }
    }
}
