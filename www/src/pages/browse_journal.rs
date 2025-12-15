use crate::controllers::handlebars::HandlebarsPage;
use crate::controllers::sqlcontroller::SqlState;
use crate::err::WebResult;
use aelita_stor_diesel::api_journal::storapi_journal_list;
use aelita_stor_diesel::model_journal::ModelJournalImmutableDiesel;
use axum::body::Body;
use axum::extract::{Path, State};
use serde::Serialize;
use std::sync::LazyLock;

pub async fn handle_browse_journal(State(state): State<SqlState>) -> WebResult<Body> {
    let journals = state
        .sqlfs
        .transact(|conn| storapi_journal_list(conn))
        .await?;

    render_html_list(journals)
}

fn render_html_list(journals: Vec<ModelJournalImmutableDiesel>) -> WebResult<Body> {
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
    let tpl = get_template();
    tpl.render(props)
}

fn get_template() -> &'static HandlebarsPage {
    const TEMPLATE: &str = include_str!("../../html/browse_journal.hbs");
    static INSTANCE: LazyLock<HandlebarsPage> =
        LazyLock::new(|| HandlebarsPage::from_template(TEMPLATE));
    &INSTANCE
}
