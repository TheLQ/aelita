use crate::controllers::state::WState;
use crate::err::WebResult;
use axum::body::Body;
use axum::extract::{Path, State};

pub async fn _handle_xrn_journal(
    State(_state): State<WState<'_>>,
    _xrn_raw: Option<Path<String>>,
) -> WebResult<Body> {
    todo!()
}

// #[derive(Deserialize)]
// pub struct PagePost {
//     xrn_name: String,
// }

// pub async fn handle_registry_html_post(
//     State(state): State<SqlState>,
//     Form(form): Form<PagePost>,
// ) -> WebResult<Body> {
//     let new = vec![ModelRegistryId {
//         xrn: XrnAddr::from_str(&form.xrn_name)?.to_string(),
//         published: StorDate::now(),
//         publish_cause: basic_cause("frontend-form"),
//     }];
//     state
//         .sqlfs
//         .query_stor(|conn| storapi_registry_ids_push(conn, new))
//         .await?;
//
//     // show same page
//     handle_registry_html(State(state), None).await
// }
