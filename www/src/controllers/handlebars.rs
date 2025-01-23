use crate::err::WResult;
use axum::body::Body;
use handlebars::Handlebars;
use serde::Serialize;

const VIEWER_TEMPLATE_NAME: &str = "tpl_viewer";

pub struct HandlebarsPage {
    template: Handlebars<'static>,
}

impl HandlebarsPage {
    pub fn from_template(template: &str) -> Self {
        let mut hbs = Handlebars::new();
        hbs.set_strict_mode(true);

        // const HEADER_COMMON: &str = include_str!("../../html/header_common.html");
        // hbs.register_template_string("header_common", HEADER_COMMON)
        //     .unwrap();
        //
        // const MENU_COMMON: &str = include_str!("../../html/menu_common.html");
        // hbs.register_template_string("menu_common", MENU_COMMON)
        //     .unwrap();

        hbs.register_template_string(VIEWER_TEMPLATE_NAME, template)
            .unwrap();
        HandlebarsPage { template: hbs }
    }

    pub fn render(&self, params: impl Serialize) -> WResult<Body> {
        let html = self.template.render(VIEWER_TEMPLATE_NAME, &params)?;
        Ok(Body::new(html))
    }
}
