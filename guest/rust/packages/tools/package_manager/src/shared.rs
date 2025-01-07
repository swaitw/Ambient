use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PackageJson {
    // TODO: don't let people submit their own custom URLs to the backend
    pub url: String,
    pub id: String,
    pub name: String,
    pub version: String,
    pub authors: Vec<String>,
    pub description: Option<String>,
}
