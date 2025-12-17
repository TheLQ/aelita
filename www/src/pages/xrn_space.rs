use crate::controllers::handlebars::HandlebarsPage;
use crate::controllers::sqlcontroller::SqlState;
use crate::err::{WebError, WebResult};
use crate::server::convert_xrn::XrnFromUrl;
use aelita_stor_diesel::api_journal::storapi_journal_get_created;
use aelita_stor_diesel::api_space::storapi_space_get;
use aelita_stor_diesel::id_types::ModelSpaceId;
use aelita_xrn::defs::space_xrn::{ProjectTypeXrn, SpaceXrn};
use axum::body::Body;
use axum::extract::State;
use serde::Serialize;
use std::sync::LazyLock;

pub async fn handle_xrn_space(
    State(state): State<SqlState>,
    XrnFromUrl(xrn): XrnFromUrl<SpaceXrn>,
) -> WebResult<Body> {
    render_html(state, xrn).await
}

async fn render_html(state: SqlState, xrn: SpaceXrn) -> WebResult<Body> {
    match xrn.ptype() {
        ProjectTypeXrn::Simple => render_simple(state, xrn).await,
        ptype => Err(WebError::unsupported_xrn_route(ptype.as_ref())),
    }
}

async fn render_simple(state: SqlState, xrn: SpaceXrn) -> WebResult<Body> {
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
    let tpl = get_template();
    tpl.render(props)
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

fn get_template() -> &'static HandlebarsPage {
    const TEMPLATE: &str = include_str!("../../html/xrn_space.hbs");
    static INSTANCE: LazyLock<HandlebarsPage> =
        LazyLock::new(|| HandlebarsPage::from_template(TEMPLATE));
    &INSTANCE
}
