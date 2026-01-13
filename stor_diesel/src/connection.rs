use diesel::connection::{Instrumentation, InstrumentationEvent};
use diesel::{Connection, QueryResult, RunQueryDsl};
use diesel::{ConnectionError, MysqlConnection};
use std::collections::HashMap;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use url::Url;
use xana_commons_rs::LOCALE;
use xana_commons_rs::num_format_re::ToFormattedString;
use xana_commons_rs::tracing_re::{Level, debug, info, span};
use xana_fs_indexer_rs::read_file_better;

#[derive(strum::AsRefStr)]
pub enum PermaStore {
    AelitaNull,
    Edition1,
    Lyoko1,
    AelitaInteg,
}

pub fn load_db_url_from_env(perma: PermaStore) -> String {
    let perma_name = match perma {
        PermaStore::AelitaNull => "aelita_null",
        PermaStore::Edition1 => "edition1",
        PermaStore::Lyoko1 => "lyoko_aelita",
        PermaStore::AelitaInteg => "aelita_integ",
    };

    let mut env_path = Path::new(".env");
    if !env_path.exists() {
        env_path = Path::new("../.env");
    }
    if !env_path.exists() {
        panic!(
            "unable to load path {} workdir {}",
            env_path.display(),
            std::env::current_dir()
                .map(|p| p.to_str().unwrap().to_string())
                .unwrap_or("BAD_CWD??".into())
        )
    }
    info!(
        "Loading environment {}",
        env_path.canonicalize().unwrap().display()
    );
    let env_map_raw = read_file_better(env_path)
        .expect("missing .env file")
        .unwrap();
    let env_map = str::from_utf8(&env_map_raw)
        .unwrap()
        .split("\n")
        .filter_map(|line| {
            if line.is_empty() || line.starts_with('#') {
                None
            } else if let Some((key, value)) = line.split_once("=") {
                Some((key.trim(), value.trim()))
            } else {
                panic!("Line '{line}' missing equals")
            }
        })
        .collect::<HashMap<_, _>>();

    let name = perma.as_ref();
    let url_raw = env_map
        .get(name)
        .unwrap_or_else(|| panic!("{name} not set in {}", env_path.display()))
        .to_string();
    let url = Url::parse(&url_raw).unwrap();
    let url_path = url.path();
    if !url_path.ends_with(&format!("/{perma_name}")) {
        panic!("expected {perma_name} but env is {url_raw}")
    }

    // let last_slash = url_raw
    //     .bytes()
    //     .into_iter()
    //     .rposition(|v| v == b'/')
    //     .unwrap()
    //     + /*slash*/1;
    // url_raw.replace_range(last_slash.., perma_name);
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
    establish_connection_url(database_url)
}

fn establish_connection_url(
    database_url: String,
) -> Result<StorConnection, (String, ConnectionError)> {
    info!("Connecting to {database_url}");
    let mut conn = MysqlConnection::establish(&database_url).map_err(|e| (database_url, e))?;
    apply_stor_instrument(&mut conn);
    Ok(conn)
}

pub fn establish_connection_perma_or_panic(perma: PermaStore) -> StorConnection {
    match establish_connection(perma) {
        Ok(conn) => conn,
        Err((url, e)) => panic!("Failed to connect to {url}: {e}"),
    }
}

pub fn apply_stor_instrument(conn: &mut StorConnection) {
    conn.set_instrumentation(StorInstrument::default());
}

#[derive(Default)]
pub struct StorInstrument {}

impl Instrumentation for StorInstrument {
    fn on_connection_event(&mut self, event: InstrumentationEvent<'_>) {
        match event {
            InstrumentationEvent::StartQuery { query, .. } => {
                if QUIET_LOG_SPAM.load(Ordering::Relaxed) {
                    return;
                }

                let query_str = query.to_string();
                let prefix = 1000;
                let suffix = 500;
                if query_str.len() > prefix + suffix {
                    // :-( no extraction possible yet
                    if let Some(binds) = query_str.find("-- binds: ")
                        && binds < prefix + suffix
                    {
                        let query_str = &query_str[..binds];
                        debug!("{query_str} -- ...binds truncate...");
                    } else {
                        let small_prefix = &query_str[0..prefix];
                        let small_suffix = &query_str[(query_str.len() - suffix)..];
                        let removed = query_str.len() - prefix - suffix;
                        debug!(
                            "{small_prefix}...truncate {} chars...{small_suffix}",
                            removed.to_formatted_string(&LOCALE)
                        );
                    }
                } else {
                    debug!("{}", query_str);
                }
            }
            _ => (),
        }
    }
}

static QUIET_LOG_SPAM: AtomicBool = AtomicBool::new(false);

pub fn with_quiet_sql_log_spam<R>(callback: impl FnOnce() -> R) -> R {
    QUIET_LOG_SPAM.store(true, Ordering::Relaxed);
    let res = callback();
    QUIET_LOG_SPAM.store(false, Ordering::Relaxed);
    res
}

pub struct StorTransaction<'s>(&'s mut StorConnection);

impl<'s> StorTransaction<'s> {
    pub fn new_transaction<T, E>(
        name: &str,
        conn: &mut StorConnection,
        callback: impl FnOnce(&mut StorTransaction) -> Result<T, E>,
    ) -> Result<T, E>
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        conn.transaction(|conn_raw| {
            let mut wrapped = StorTransaction(conn_raw);
            span!(Level::INFO, "q", name).in_scope(|| {
                callback(&mut wrapped)
                    .map_err(|e| diesel::result::Error::QueryBuilderError(Box::new(e)))
            })
        })
        .map_err(|e| match e {
            // only possible error
            diesel::result::Error::QueryBuilderError(e) => *e.downcast().unwrap(),
            e => unreachable!("unreachable {e}"),
        })
    }

    // pub async fn new_transaction_async<T, E, Callback, CallbackResult>(
    //     name: impl Into<String>,
    //     conn: &'s mut StorConnection,
    //     callback: Callback,
    // ) -> Result<T, E>
    // where
    //     E: std::error::Error + Send + Sync + 'static,
    //     // spawn_blocking only needs Send
    //     T: Send + 'static,
    //     Callback: FnOnce(&mut StorTransaction) -> CallbackResult + Send + Sync + 'static,
    //     CallbackResult: Future<Output = Result<T, E>>,
    // {
    //     let name = name.into();
    //     let conn_arc = Arc::new(conn);
    //
    //     tokio::task::spawn_blocking(move || {
    //         let mut conn_arc = conn_arc.clone();
    //         conn_arc
    //             .transaction(move |conn_raw| {
    //                 let mut wrapped = StorTransaction(conn_raw);
    //                 let _span = span!(Level::INFO, "q", name).entered();
    //
    //                 let rt = tokio::runtime::Handle::current();
    //                 rt.block_on(async { callback(&mut wrapped).await })
    //                     .map_err(|e| diesel::result::Error::QueryBuilderError(Box::new(e)))
    //             })
    //             .map_err(|e| match e {
    //                 // only possible error
    //                 diesel::result::Error::QueryBuilderError(e) => *e.downcast().unwrap(),
    //                 _ => unreachable!(),
    //             })
    //     })
    //     .await
    //     .unwrap()
    // }

    pub fn inner(&mut self) -> &mut StorConnection {
        self.0
    }

    pub fn raw_sql_execute<I: Into<String>>(&mut self, input: I) -> QueryResult<usize> {
        diesel::sql_query(input).execute(self.0)
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
