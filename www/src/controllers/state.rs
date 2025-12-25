use crate::controllers::handlebars::HandlebarsController;
use crate::controllers::sqlcontroller::SqlController;
use crate::err::WebResult;
use aelita_stor_diesel::PermaStore;
use std::sync::Arc;

#[derive(Clone)]
pub struct WState {
    pub sqlfs: Arc<SqlController>,
    /*
    todo: this shouldn't abuse static
    but this is referenced in every web handler
    which breaks #[axum::debug_handler]
    */
    pub handlebars: Arc<HandlebarsController<'static>>,
}

impl WState {
    pub fn new(store: PermaStore) -> WebResult<Self> {
        Ok(Self {
            sqlfs: Arc::new(SqlController::new(store)),
            handlebars: Arc::new(HandlebarsController::new()?),
        })
    }
}
