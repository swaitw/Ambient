use std::path::Path;

use thiserror::Error;
use url::Url;

#[derive(Error, Debug)]
pub enum RetrieveError {
    #[error("file reading is not supported on web")]
    FileReadingNotSupportedOnWeb,
    #[error("failed to read path {file_path:?}")]
    FailedToReadPath { file_path: String },
    #[error("failed to get URL {url:?}")]
    FailedToGetUrl { url: Url },
    #[error("invalid file path for URL {url:?}")]
    InvalidFilePathForUrl { url: Url },
    #[error("failed to get text from URL {url:?}")]
    FailedToGetTextFromUrl { url: Url },
}

pub fn retrieve_file(path: &Path) -> Result<String, RetrieveError> {
    #[cfg(target_os = "unknown")]
    return Err(RetrieveError::FileReadingNotSupportedOnWeb);

    #[cfg(not(target_os = "unknown"))]
    return std::fs::read_to_string(path).map_err(|_| RetrieveError::FailedToReadPath {
        file_path: path.to_string_lossy().to_string(),
    });
}

pub async fn retrieve_url(url: &Url) -> Result<String, RetrieveError> {
    if url.scheme() == "file" {
        #[cfg(target_os = "unknown")]
        return Err(RetrieveError::FileReadingNotSupportedOnWeb);

        #[cfg(not(target_os = "unknown"))]
        return retrieve_file(
            &url.to_file_path()
                .map_err(|_| RetrieveError::InvalidFilePathForUrl { url: url.clone() })?,
        );
    }

    reqwest::get(url.clone())
        .await
        .and_then(|r| r.error_for_status())
        .map_err(|_| RetrieveError::FailedToGetUrl { url: url.clone() })?
        .text()
        .await
        .map_err(|_| RetrieveError::FailedToGetTextFromUrl { url: url.clone() })
}
