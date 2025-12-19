use crate::controllers::handlebars::HandlebarsController;
use crate::controllers::sqlcontroller::SqlController;
use crate::err::WebResult;
use aelita_stor_diesel::PermaStore;
use handlebars::Handlebars;
use std::sync::Arc;

#[derive(Clone)]
pub struct WState<'h> {
    pub sqlfs: Arc<SqlController>,
    // todo 'static may leak but a lifetime causes type chaos
    pub handlebars: Arc<HandlebarsController<'h>>,
}

impl<'h> WState<'h> {
    pub fn new(store: PermaStore) -> WebResult<Self> {
        Ok(Self {
            sqlfs: Arc::new(SqlController::new(store)),
            handlebars: Arc::new(HandlebarsController::new()?),
        })
    }
}
