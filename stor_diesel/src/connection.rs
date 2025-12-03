use crate::err::StorDieselResult;
use diesel::Connection;
use diesel::connection::{Instrumentation, InstrumentationEvent};
use diesel::{ConnectionError, MysqlConnection};
use std::collections::HashMap;
use std::path::Path;
use xana_commons_rs::read_file_better;
use xana_commons_rs::tracing_re::{Level, info, span, trace};

pub enum PermaStore {
    AelitaNull,
    Edition1,
}

fn load_db_url_from_env(perma: PermaStore) -> String {
    let perma_name = match perma {
        PermaStore::AelitaNull => "aelita_null",
        PermaStore::Edition1 => "edition1",
    };

    let env_path = Path::new(".env");
    let env_map_raw = read_file_better(env_path)
        .expect("missing .env file")
        .unwrap();
    info!(
        "Loaded environment {}",
        env_path.canonicalize().unwrap().display()
    );
    let env_map = str::from_utf8(&env_map_raw)
        .unwrap()
        .split("\n")
        .filter(|line| !line.is_empty())
        .map(|line| {
            line.split_once("=")
                .unwrap_or_else(|| panic!("Line '{line}' missing equals"))
        })
        .collect::<HashMap<_, _>>();

    let mut url_raw = env_map
        .get("DATABASE_URL")
        .expect("DATABASE_URL must be set")
        .to_string();
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

pub fn establish_connection(
    perma: PermaStore,
) -> Result<StorConnection, (String, ConnectionError)> {
    let database_url = load_db_url_from_env(perma);
    info!("Connecting to {database_url}");
    // InstrumentedMysqlConnection::establish(&database_url)
    let mut conn = MysqlConnection::establish(&database_url).map_err(|e| (database_url, e))?;
    conn.set_instrumentation(StorInstrument::default());
    Ok(conn)
}

pub fn establish_connection_or_panic(perma: PermaStore) -> StorConnection {
    match establish_connection(perma) {
        Ok(conn) => conn,
        Err((url, e)) => panic!("Failed to connect to {url}: {e}"),
    }
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
            _ => (),
        }
    }
}

pub struct StorTransaction<'s>(&'s mut StorConnection);

impl<'s> StorTransaction<'s> {
    pub fn new_transaction<T>(
        name: &str,
        conn: &mut StorConnection,
        callback: impl FnOnce(&mut StorTransaction) -> StorDieselResult<T>,
    ) -> StorDieselResult<T> {
        conn.transaction(|conn_raw| {
            let mut wrapped = StorTransaction(conn_raw);
            let _span = span!(Level::INFO, "q", name).entered();
            callback(&mut wrapped)
                .map_err(|e| diesel::result::Error::QueryBuilderError(Box::new(e)))
        })
        .map_err(|e| match e {
            // only possible error
            diesel::result::Error::QueryBuilderError(e) => *e.downcast().unwrap(),
            _ => unreachable!(),
        })
    }

    pub fn inner(&mut self) -> &mut StorConnection {
        self.0
    }
}

// todo:😢
// impl Deref for StorTransaction<'_> {
//     type Target = StorConnection;
//
//     fn deref(&self) -> &Self::Target {
//         self.0
//     }
// }
//
// impl DerefMut for StorTransaction<'_> {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         self.0
//     }
// }
