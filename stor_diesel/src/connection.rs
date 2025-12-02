use diesel::MysqlConnection;
use diesel::connection::{Instrumentation, InstrumentationEvent};
use diesel::{Connection, IntoSql};
use dotenvy::dotenv;
use rand::RngCore;
use std::cell::RefCell;
use std::env;
use xana_commons_rs::tracing_re::span::EnteredSpan;
use xana_commons_rs::tracing_re::{Level, span, trace};

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

thread_local! {
    static TX_SPAN: RefCell<Option<EnteredSpan>> = RefCell::new(None);
}

#[derive(Default)]
struct StorInstrument {}

impl Instrumentation for StorInstrument {
    fn on_connection_event(&mut self, event: InstrumentationEvent<'_>) {
        match event {
            InstrumentationEvent::StartQuery { query, .. } => {
                let query_str = query.to_string();
                trace!("{}", query_str);
            }
            InstrumentationEvent::BeginTransaction { .. } => {
                let id_num = rand::rng().next_u32();
                let id_value = format!("{:x}", id_num);
                let span = span!(Level::INFO, "q", tx = id_value);
                let prev = TX_SPAN.replace(Some(span.entered()));
                assert!(prev.is_none());

                // todo: nested transaction
                // let prev = TX_SPAN.take();
                // if let None = prev {
                //     let span = span!(Level::INFO, "q", tx = id_value);
                //     TX_SPAN.set(Some(span.entered()));
                // }
            }
            InstrumentationEvent::FinishQuery { query, .. } => {
                if matches!(query.to_string().as_str(), "COMMIT" | "ROLLBACK") {
                    let prev = TX_SPAN.take();
                    assert!(prev.is_some());
                }
            }
            _ => (),
        }
    }
}

pub fn assert_in_transaction() {
    assert!(TX_SPAN.with_borrow(|v| v.is_some()))
}
