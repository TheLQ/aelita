use crate::controllers::handlebars::HbsPage;
use crate::controllers::state::WState;
use crate::err::WebResult;
use crate::server::util::BasicResponse;
use aelita_stor_diesel::ModelJournalImmutableDiesel;
use aelita_stor_diesel::storapi_journal_list;
use axum::extract::State;
use serde::Serialize;

pub async fn handle_browse_journal(State(state): State<WState<'_>>) -> WebResult<BasicResponse> {
    let journals = state
        .sqlfs
        .transact(|conn| storapi_journal_list(conn))
        .await?;

    render_html_list(state, journals)
}

fn render_html_list(
    state: WState<'_>,
    journals: Vec<ModelJournalImmutableDiesel>,
) -> WebResult<BasicResponse> {
    #[derive(Serialize)]
    struct JournalEntry {
        xrn: String,
        at: String,
        cause_description: String,
    }
    #[derive(Serialize)]
    struct HtmlProps {
        journals: Vec<JournalEntry>,
    }
    let props = HtmlProps {
        journals: journals
            .into_iter()
            .map(
                |ModelJournalImmutableDiesel {
                     journal_id,
                     at,
                     cause_description,
                     ..
                 }| JournalEntry {
                    xrn: journal_id.to_string(), // todo real xrn
                    at: at.to_string(),
                    cause_description,
                },
            )
            .collect(),
    };
    state.render_page(HbsPage::Browse_Journal, props)
}
