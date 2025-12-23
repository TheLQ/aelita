use crate::controllers::handlebars::HbsPage;
use crate::controllers::state::WState;
use crate::err::{WebErrorKind, WebResult};
use crate::server::convert_xrn::XrnFromUrl;
use crate::server::util::BasicResponse;
use aelita_stor_diesel::ModelSpaceId;
use aelita_stor_diesel::storapi_journal_get_created;
use aelita_stor_diesel::storapi_space_get;
use aelita_xrn::defs::space_xrn::{SpaceXrn, SpaceXrnType};
use axum::extract::State;
use serde::Serialize;

pub async fn handle_xrn_space(
    State(state): State<WState<'_>>,
    XrnFromUrl(xrn): XrnFromUrl<SpaceXrn>,
) -> WebResult<BasicResponse> {
    render_html(state, xrn).await
}

async fn render_html(state: WState<'_>, xrn: SpaceXrn) -> WebResult<BasicResponse> {
    #[allow(unreachable_patterns)]
    match xrn.stype() {
        SpaceXrnType::Simple => render_simple(state, xrn).await,
        ptype => Err(WebErrorKind::UnsupportedXrnRoute.build_message(ptype.as_ref())),
    }
}

async fn render_simple(state: WState<'_>, xrn: SpaceXrn) -> WebResult<BasicResponse> {
    let space_id = ModelSpaceId::from_project_xrn(&xrn);
    let space_id_trans = space_id.clone();
    let (space, created) = state
        .sqlfs
        .transact(move |conn| {
            let space = storapi_space_get(conn, space_id_trans)?;
            let created = storapi_journal_get_created(conn, space.journal_id)?;
            Ok((space, created))
        })
        .await?;
    #[derive(Serialize)]
    struct Spaceentry {
        xrn: String,
        published: String,
        title: String,
    }
    #[derive(Serialize)]
    struct HtmlProps {
        dash_name: String,
        space: Spaceentry,
    }

    let journal_id = space.journal_id;
    let props = HtmlProps {
        dash_name: xrn.to_string(),
        space: Spaceentry {
            xrn: space.space_id.to_string(), // todo actual xrn
            published: format!("Journal {journal_id} on {created}"),
            title: space.description,
        },
    };
    state.render_page(HbsPage::Xrn_Space, props)
}

// #[derive(Deserialize)]
// pub struct PostData {
//     pub project_type: String,
//     pub title: String,
//     pub description: String,
// }

// pub async fn handle_project_post(
//     State(state): State<SqlState>,
//     XrnFromUrl(xrn): XrnFromUrl<SpaceXrn>,
//     Form(PostData {
//         project_type,
//         title,
//         description,
//     }): Form<PostData>,
// ) -> WebResult<Body> {
//     let project_type = ProjectTypeXrn::from_str(&project_type)?;
//     // match project_type {
//     //     ProjectTypeXrn::Simple => render_html(state, xrn).await,
//     //     _ => return Err(WebError::UnsupportedXrnRoute(project_type.as_ref().into())),
//     // }
//
//     todo!("this doesn't actually work");
//     // state.sqlfs.transact(|conn| {
//     //     storapi_space_new(conn, NewModelSpaceName {
//     //         journal_id: todo!(),
//     //         description,
//     //         space_name: title,
//     //     })
//     //     let cause = "frontend-form";
//     // });
//     //
//     // render_html(state, xrn).await
// }
