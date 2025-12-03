use crate::err::StorImportResult;
use aelita_stor_diesel::StorTransaction;
use aelita_stor_diesel::api_journal::{
    storapi_journal_commit_new, storapi_journal_immutable_push,
    storapi_journal_immutable_push_single,
};
use aelita_stor_diesel::api_tor::storapi_tor_host_get;
use aelita_stor_diesel::id_types::ModelJournalTypeName;
use aelita_stor_diesel::model_journal::NewModelJournalDataImmutable;
use aelita_stor_diesel::model_tor::ModelQbHosts;
use bytes::Bytes;
use tokio::runtime::Handle;
use tokio::task::JoinSet;
use xana_commons_rs::SimpleIoMap;
use xana_commons_rs::qbittorrent_re::QBittorrentClientBuilder;
use xana_commons_rs::tracing_re::{Level, info, span};

pub fn storfetch_journal_torrents<'t>(conn: &mut StorTransaction<'t>) -> StorImportResult<()> {
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

    let res = fetch_async_start(hosts)?;
    storapi_journal_immutable_push(
        conn,
        res.into_iter()
            .map(|(model, v)| NewModelJournalDataImmutable {
                journal_type: ModelJournalTypeName::Journal1,
                data: v.into(),
                cause_description: format!("stor {hosts_num} qb hosts"),
                cause_xrn: None,
            }),
    )?;

    Ok(())
}

fn fetch_async_start(hosts: Vec<ModelQbHosts>) -> StorImportResult<Vec<(ModelQbHosts, Bytes)>> {
    tokio::task::block_in_place(move || Handle::current().block_on(_fetch_async(hosts)))
}

async fn _fetch_async(hosts: Vec<ModelQbHosts>) -> StorImportResult<Vec<(ModelQbHosts, Bytes)>> {
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

async fn fetch_async_host(host: ModelQbHosts) -> StorImportResult<(ModelQbHosts, Bytes)> {
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
