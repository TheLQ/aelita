use crate::controllers::state::WState;
use crate::err::WebResult;
use crate::server::util::BasicResponse;
use axum::body::Body;
use axum::http::StatusCode;
use handlebars::Handlebars;
use serde::Serialize;
use std::path::PathBuf;
use strum::VariantArray;

impl<'h> WState<'h> {
    fn render_page_string(&self, page: HbsPage, data: impl Serialize) -> WebResult<String> {
        self.handlebars
            .backend
            .render(page.as_ref(), &data)
            .map_err(Into::into)
    }

    pub fn render_page(&self, page: HbsPage, data: impl Serialize) -> WebResult<BasicResponse> {
        let body = self.render_page_string(page, data)?;
        Ok(BasicResponse(StatusCode::OK, mime::HTML, Body::from(body)))
    }
}
pub struct HandlebarsController<'h> {
    backend: Handlebars<'h>,
}

impl<'h> HandlebarsController<'h> {
    pub(crate) fn new() -> WebResult<Self> {
        let mut backend = Handlebars::new();
        backend.set_strict_mode(true);
        backend.set_dev_mode(true);

        for page in HbsPage::VARIANTS {
            backend.register_template_file(page.as_ref(), page.to_path())?;
        }

        Ok(Self { backend })
    }
}

#[derive(Clone, Copy, strum::AsRefStr, strum::VariantArray)]
#[allow(non_camel_case_types)]
#[strum(serialize_all = "lowercase")]
pub enum HbsPage {
    Base_Html,
    Browse_Journal,
    Browse_Paths,
    Browse_Space,
    Browse_Tor,
    Xrn_Journal,
    Xrn_Path,
    Xrn_Space,
}

fn html_dir_path() -> PathBuf {
    PathBuf::from("www/html")
}

impl HbsPage {
    fn to_path(&self) -> PathBuf {
        html_dir_path().join(format!("{}.{}", self.as_ref(), self.extension()))
    }

    fn extension(&self) -> &'static str {
        match self {
            Self::Browse_Tor => "html",
            _ => "hbs",
        }
    }
}
