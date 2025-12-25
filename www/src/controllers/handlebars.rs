use crate::controllers::state::WState;
use crate::err::{WebErrorKind, WebResult};
use crate::server::util::BasicResponse;
use axum::body::Body;
use axum::http::StatusCode;
use handlebars::{
    Context, Handlebars, Helper, HelperDef, HelperResult, Output, RenderContext, RenderErrorReason,
};
use serde::Serialize;
use std::path::PathBuf;
use strum::VariantArray;
use xana_commons_rs::ResultXanaMap;

impl WState {
    fn render_page_string(&self, page: HbsPage, data: impl Serialize) -> WebResult<String> {
        self.handlebars
            .backend
            .render(page.as_ref(), &data)
            .xana_err(WebErrorKind::HandlebarsRenderFailed)
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
            backend
                .register_template_file(page.as_ref(), page.to_path())
                .xana_err(WebErrorKind::HandlebarsInitFailed)?;
        }
        backend.register_helper("percent", Box::new(PercentHelper));

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
    PathBuf::from("www/src/pages")
}

impl HbsPage {
    fn to_path(&self) -> PathBuf {
        html_dir_path().join(format!("{}.{}", self.as_ref(), self.extension()))
    }

    fn extension(&self) -> &'static str {
        match self {
            // Self::Browse_Tor => "html",
            _ => "hbs",
        }
    }
}

/// avoid annoying string-ifying in Rust with our typesafe structs
#[derive(Clone, Copy)]
struct PercentHelper;

impl HelperDef for PercentHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper,
        _: &Handlebars,
        _: &Context,
        _rc: &mut RenderContext,
        out: &mut dyn Output,
    ) -> HelperResult {
        let param_raw = h.param(0).unwrap();
        let Some(param) = param_raw.value().as_f64() else {
            return Err(RenderErrorReason::Other("PercentHelper param is not float".into()).into());
        };
        out.write_fmt(format_args!("%{param:.1}"))?;
        Ok(())
    }
}
