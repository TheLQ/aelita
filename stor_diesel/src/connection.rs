use diesel::Connection;
use diesel::MysqlConnection;
use diesel::connection::{Instrumentation, InstrumentationEvent};
use dotenvy::dotenv;
use std::env;
use xana_commons_rs::tracing_re::trace;

pub enum PermaStore {
    AelitaNull,
    Edition1,
}

fn load_db_url_from_env(perma: PermaStore) -> String {
    let perma_name = match perma {
        PermaStore::AelitaNull => "aelita_null",
        PermaStore::Edition1 => "edition1",
    };

    dotenv().ok();
    let mut url_raw = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let last_slash = url_raw
        .bytes()
        .into_iter()
        .rposition(|v| v == b'/')
        .unwrap()
        + /*slash*/1;
    url_raw.replace_range(last_slash.., perma_name);
    url_raw
}

/*
todo: type_alias_impl_trait
 when it doesn't need the #[define_opaque(Alias)]
*/
pub type StorConnection = MysqlConnection;

pub fn establish_connection(perma: PermaStore) -> StorConnection {
    let database_url = load_db_url_from_env(perma);
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
