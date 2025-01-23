use std::backtrace::Backtrace;
use std::error::{Error, request_ref};

/// Make a message and a back trace look pretty
pub fn pretty_error(e: impl Error) -> String {
    let btraw = request_ref::<Backtrace>(&e);
    if let Some(bt) = btraw {
        format!("Panic {} bt\n{}", e, bt)
    } else {
        format!("Panic {}", e)
    }
}

pub fn xbt() -> Backtrace {
    Backtrace::capture()
}
