use std::backtrace::Backtrace;
use std::error::{Error, request_ref};
use std::io;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};

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

/// IO Error Context to always bring path along
/// Integrates well with child error types
pub struct IOEC<E> {
    path: PathBuf,
    _p: PhantomData<E>,
}

pub struct IOECStd {
    pub path: PathBuf,
    pub err: io::Error,
}

pub struct IOECSerde {
    pub path: PathBuf,
    pub err: serde_json::Error,
}

impl<E> IOEC<E> {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            _p: PhantomData::default(),
        }
    }

    pub fn io(&self) -> impl Fn(io::Error) -> E
    where
        E: From<IOECStd>,
    {
        |err: io::Error| {
            IOECStd {
                path: self.path.clone(),
                err,
            }
            .into()
        }
    }

    pub fn serde(&self) -> impl Fn(serde_json::Error) -> E
    where
        E: From<IOECSerde>,
    {
        |err: serde_json::Error| {
            IOECSerde {
                path: self.path.clone(),
                err,
            }
            .into()
        }
    }
}
