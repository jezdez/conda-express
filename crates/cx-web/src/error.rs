use std::fmt;

use wasm_bindgen::prelude::*;

#[derive(Debug)]
pub enum CxWebError {
    LockfileParse(String),
    PlatformUnknown(String),
    NoDefaultEnvironment,
    NoRecordsForPlatform(String),
    FetchFailed(String),
    ExtractFailed(String),
    SerializeFailed(String),
    UnknownPackageFormat(String),
}

impl fmt::Display for CxWebError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LockfileParse(e) => write!(f, "failed to parse lockfile: {e}"),
            Self::PlatformUnknown(p) => write!(f, "unknown platform: {p}"),
            Self::NoDefaultEnvironment => write!(f, "no default environment in lockfile"),
            Self::NoRecordsForPlatform(p) => write!(f, "no records for platform {p}"),
            Self::FetchFailed(e) => write!(f, "fetch failed: {e}"),
            Self::ExtractFailed(e) => write!(f, "extraction failed: {e}"),
            Self::SerializeFailed(e) => write!(f, "serialization failed: {e}"),
            Self::UnknownPackageFormat(url) => write!(f, "unknown package format: {url}"),
        }
    }
}

impl From<CxWebError> for JsValue {
    fn from(err: CxWebError) -> Self {
        js_sys::Error::new(&err.to_string()).into()
    }
}
