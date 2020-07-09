/// The main error type of this crate
#[derive(Debug, Clone)]
pub enum Error {
    Simple(StringError),
    IO(IOErrorWrapper),
    NFD(NFDErrorWrapper),
    Parse(std::num::ParseIntError),
    Reqwest(ReqwestErrorWrapper),
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::IO(IOErrorWrapper::from(error))
    }
}

impl From<StringError> for Error {
    fn from(error: StringError) -> Self {
        Self::Simple(error)
    }
}

impl From<nfd2::error::NFDError> for Error {
    fn from(error: nfd2::error::NFDError) -> Self {
        Self::NFD(NFDErrorWrapper::from(error))
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(error: std::num::ParseIntError) -> Self {
        Self::Parse(error)
    }
}

impl From<reqwest::Error> for Error {
    fn from(error: reqwest::Error) -> Self {
        Self::Reqwest(ReqwestErrorWrapper::from(error))
    }
}

// We need to wrap some errors because we require cloneable errors to properly work with iced
// (Possible alternative: use Arc?)

#[derive(Debug, Clone)]
pub enum NFDErrorWrapper {
    NulError(std::ffi::NulError),
    Utf8Error(core::str::Utf8Error),
    Error(String),
}

impl From<nfd2::error::NFDError> for NFDErrorWrapper {
    fn from(error: nfd2::error::NFDError) -> Self {
        match error {
            nfd2::error::NFDError::NulError(e) => Self::NulError(e),
            nfd2::error::NFDError::Utf8Error(e) => Self::Utf8Error(e),
            nfd2::error::NFDError::Error(s) => Self::Error(s),
        }
    }
}

#[derive(Debug, Clone)]
pub enum IOErrorWrapper {
    Simple(std::io::ErrorKind)
}

// This conversion loses some information in case of a custom error or an Os Error that is not represented by an `ErrorKind`
impl From<std::io::Error> for IOErrorWrapper {
    fn from(error: std::io::Error) -> Self {
        Self::Simple(error.kind())
    }
}

#[derive(Debug, Clone)]
pub struct ReqwestErrorWrapper {
    url: Option<reqwest::Url>,
    is_builder: bool,
    is_redirect: bool,
    is_status: bool,
    is_timeout: bool,
    status: Option<reqwest::StatusCode>,
}

impl From<reqwest::Error> for ReqwestErrorWrapper {
      fn from(error: reqwest::Error) -> Self {
        let owned_url = {
            if let Some(url) = error.url(){
                Some(url.to_owned())
            } else {
                None
            }
        };
        Self {
            url: owned_url,
            is_builder: error.is_builder(),
            is_redirect: error.is_redirect(),
            is_status: error.is_status(),
            is_timeout: error.is_timeout(),
            status: error.status(),
        }
    }
}

/// StringError type for development, wraps everything we don't have a proper error enum for
#[derive(Debug, Clone)]
pub struct StringError {
    pub desc: &'static str
}

impl StringError {
    pub fn new(desc: &'static str) -> Self {
        Self {
            desc,
        }
    }
}