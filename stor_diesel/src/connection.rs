use aelita_commons::tracing_re::trace;
use diesel::Connection;
use diesel::MysqlConnection;
use diesel::connection::{Instrumentation, InstrumentationEvent};
use dotenvy::dotenv;
use std::env;

pub fn load_db_url_from_env() -> String {
    dotenv().ok();
    env::var("DATABASE_URL").expect("DATABASE_URL must be set")
}

/*
todo: type_alias_impl_trait
 when it doesn't need the #[define_opaque(Alias)]
*/
pub type StorConnection = MysqlConnection;

pub fn establish_connection() -> StorConnection {
    let database_url = load_db_url_from_env();
    // InstrumentedMysqlConnection::establish(&database_url)
    let mut conn = MysqlConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url));
    conn.set_instrumentation(StorInstrument::default());
    conn
}

#[derive(Default)]
struct StorInstrument {
    inside_tx: bool,
}

impl Instrumentation for StorInstrument {
    fn on_connection_event(&mut self, event: InstrumentationEvent<'_>) {
        match event {
            InstrumentationEvent::StartQuery { query, .. } => {
                let query_str = query.to_string();
                if !self.inside_tx && query_str != "COMMIT" {
                    trace!("---s");
                }
                trace!("{}", query_str);
            }
            InstrumentationEvent::BeginTransaction { .. } => {
                self.inside_tx = true;
                trace!("---");
            }
            InstrumentationEvent::CommitTransaction { .. }
            | InstrumentationEvent::RollbackTransaction { .. } => {
                self.inside_tx = false;
            }
            _ => {}
        }
    }
}
