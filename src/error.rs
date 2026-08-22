use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum ShftError {
    #[error("invalid arguments: {0}")]
    InvalidArguments(String),
    #[error("unsupported image format for `{0}`")]
    UnsupportedFormat(PathBuf),
    #[error("failed to discover images at `{path}`")]
    Discovery {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to read `{path}`")]
    Read {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to decode `{path}`: {detail}")]
    Decode { path: PathBuf, detail: String },
    #[error("failed to encode `{path}` as {format}: {detail}")]
    Encode {
        path: PathBuf,
        format: String,
        detail: String,
    },
    #[error("output `{0}` already exists; use --overwrite or choose another destination")]
    OutputExists(PathBuf),
    #[error("failed to read the custom output name")]
    Prompt(#[source] std::io::Error),
    #[error("failed to write output `{path}`")]
    Write {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("transparent image cannot be encoded as JPEG without --background <HEX>")]
    TransparencyRequiresBackground,
    #[error("no supported images found at `{0}`")]
    NoImages(PathBuf),
    #[error("thread pool initialization failed: {0}")]
    ThreadPool(String),
    #[error("failed to serialize JSON report: {0}")]
    Json(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, ShftError>;
