use std::fmt;

use wasm_bindgen::prelude::*;

#[derive(Debug)]
pub enum CxWasmError {
    // Bootstrap / lockfile errors
    LockfileParse(String),
    PlatformUnknown(String),
    NotEmbedded(String),
    NoDefaultEnvironment,
    NoRecordsForPlatform(String),
    FetchFailed(String),
    ExtractFailed(String),
    UnknownPackageFormat(String),
    CallbackFailed(String),

    // Solve errors
    RepodataParse(String),
    SpecParse(String),
    PackageParse(String),
    SolveFailed(String),
    InvalidInput(String),

    // Shared
    SerializeFailed(String),
}

impl fmt::Display for CxWasmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LockfileParse(e) => write!(f, "failed to parse lockfile: {e}"),
            Self::PlatformUnknown(p) => write!(f, "unknown platform: {p}"),
            Self::NotEmbedded(what) => write!(f, "no embedded {what} (build without CX_LOCKFILE_PATH / CX_PLATFORM)"),
            Self::NoDefaultEnvironment => write!(f, "no default environment in lockfile"),
            Self::NoRecordsForPlatform(p) => write!(f, "no records for platform {p}"),
            Self::FetchFailed(e) => write!(f, "fetch failed: {e}"),
            Self::ExtractFailed(e) => write!(f, "extraction failed: {e}"),
            Self::UnknownPackageFormat(url) => write!(f, "unknown package format: {url}"),
            Self::CallbackFailed(e) => write!(f, "JS callback failed: {e}"),
            Self::RepodataParse(e) => write!(f, "failed to parse repodata: {e}"),
            Self::SpecParse(e) => write!(f, "failed to parse match spec: {e}"),
            Self::PackageParse(e) => write!(f, "failed to parse package record: {e}"),
            Self::SolveFailed(e) => write!(f, "solver failed: {e}"),
            Self::InvalidInput(e) => write!(f, "invalid input: {e}"),
            Self::SerializeFailed(e) => write!(f, "serialization failed: {e}"),
        }
    }
}

impl From<CxWasmError> for JsValue {
    fn from(err: CxWasmError) -> Self {
        js_sys::Error::new(&err.to_string()).into()
    }
}
