use thiserror::Error;

use crate::{
    internal::{conversion::IntoBindgen, wit},
    prelude::EntityId,
};

#[derive(Error, Debug)]
/// Errors that can occur when resolving an asset URL.
pub enum UrlError {
    #[error("Invalid URL: {0}")]
    /// The URL is invalid.
    InvalidUrl(String),
}
impl From<wit::asset::UrlError> for UrlError {
    fn from(value: wit::asset::UrlError) -> Self {
        match value {
            wit::asset::UrlError::InvalidUrl(err) => UrlError::InvalidUrl(err),
        }
    }
}

/// Resolves a asset path for an Ambient asset to an absolute URL.
#[doc(hidden)]
pub fn url_for_package_asset(package_id: EntityId, path: &str) -> Result<String, UrlError> {
    Ok(wit::asset::url(package_id.into_bindgen(), path)?)
}
