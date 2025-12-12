use crate::err::{StorImportError, StorImportResult};
use crate::importers::qb_get_tor_json_v1::defs::ImportQbMetadata;
use aelita_stor_diesel::StorTransaction;
use aelita_stor_diesel::api_journal::storapi_journal_immutable_push_single;
use aelita_stor_diesel::api_tor::storapi_tor_host_get;
use aelita_stor_diesel::id_types::ModelJournalTypeName;
use aelita_stor_diesel::model_journal::NewModelJournalImmutable;
use aelita_stor_diesel::model_tor::ModelQbHost;
use aelita_stor_diesel::util_types::RawDieselBytes;
use bytes::Bytes;
use tokio::runtime::Handle;
use tokio::task::JoinSet;
use xana_commons_rs::BasicWatch;
use xana_commons_rs::qbittorrent_re::QBittorrentClientBuilder;
use xana_commons_rs::tracing_re::{Level, info, span};

pub fn storfetch_torrents(conn: &mut StorTransaction<'_>) -> StorImportResult<()> {
    let hosts = storapi_tor_host_get(conn)?;
    info!(
        "connecting to {} hosts: {}",
        hosts.len(),
        hosts
            .iter()
            .map(|v| v.gui_name())
            .collect::<Vec<_>>()
            .join(",")
    );
    let hosts_num = hosts.len();

    let fetch_results = fetch_async_start(hosts)?;
    for (model, data) in fetch_results {
        storapi_journal_immutable_push_single(
            conn,
            NewModelJournalImmutable {
                journal_type: ModelJournalTypeName::QbGetTorJson1,
                data: RawDieselBytes::new(data.into()),
                metadata: Some(RawDieselBytes::serialize_json(ImportQbMetadata {
                    qb_host_id: model.qb_host_id,
                })?),
                cause_description: format!("stor {hosts_num} qb hosts"),
                cause_xrn: None,
            },
        )?;
    }

    Ok(())
}

fn fetch_async_start(hosts: Vec<ModelQbHost>) -> StorImportResult<Vec<(ModelQbHost, Bytes)>> {
    let hosts_len = hosts.len();
    let watch = BasicWatch::start();
    let res = tokio::task::block_in_place(move || Handle::current().block_on(_fetch_async(hosts)));
    info!("Fetched {hosts_len} hosts in {watch}");
    res
}

async fn _fetch_async(hosts: Vec<ModelQbHost>) -> StorImportResult<Vec<(ModelQbHost, Bytes)>> {
    let mut queries = JoinSet::new();
    for host in hosts {
        queries.spawn(async {
            // let span = span!(Level::INFO, "fetch_async", name = host.name);
            // span.in_scope(|| fetch_async_host(host)).await
            fetch_async_host(host).await
        });
    }
    queries.join_all().await.into_iter().try_collect::<Vec<_>>()
}

async fn fetch_async_host(host: ModelQbHost) -> StorImportResult<(ModelQbHost, Bytes)> {
    let span = span!(Level::ERROR, "fetch_async", name = host.name);
    span.in_scope(async || {
        let client = QBittorrentClientBuilder {
            fqdn: host.address.clone(),
            is_https: false,
        }
        .build()?;
        info!("get_torrents {}", host.gui_name());

        client.auth().await?;

        client
            .get_torrents_raw(None)
            .await
            .map(|bytes| (host, bytes))
            .map_err(Into::into)
    })
    .await
}
