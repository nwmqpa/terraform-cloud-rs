pub mod organizations;
pub use organizations::*;

pub mod varsets;
pub mod workspaces;
use serde::Deserialize;
pub use varsets::*;
pub use workspaces::*;
pub mod workspace_variables;
pub use workspace_variables::*;

pub mod state_versions;
pub use state_versions::*;

#[derive(Debug, Deserialize)]
pub struct ResourceList<T> {
    data: Vec<T>,
}

#[derive(Debug, Deserialize)]
pub struct WrappedResource<T> {
    data: T,
}

#[derive(Debug, Clone)]
pub struct TerraformCloud {
    token: String,
    client: reqwest::Client,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum TerraformResult<T> {
    #[serde(rename = "errors")]
    Error {
        errors: serde_json::Value,
    },
    Success(TerraformData<T>),
}

impl<T> TerraformResult<T> {
    fn to_data(self) -> anyhow::Result<TerraformData<T>> {
        match self {
            TerraformResult::Error { errors } => {
                return Err(anyhow::anyhow!("Error fetching resource: {:?}", errors))
            }
            TerraformResult::Success(data) => return Ok(data),
        }
    }

    pub fn to_unique(self) -> anyhow::Result<T> {
        match self.to_data()? {
            TerraformData::ResourceList(_) => {
                return Err(anyhow::anyhow!("Expected a single resource, got a list"))
            }
            TerraformData::UniqueResource(resource) => return Ok(resource.data),
        }
    }

    pub fn to_list(self) -> anyhow::Result<Vec<T>> {
        match self.to_data()? {
            TerraformData::ResourceList(list) => return Ok(list.data),
            TerraformData::UniqueResource(_) => {
                return Err(anyhow::anyhow!("Expected a list, got a single resource"))
            }
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum TerraformData<T> {
    ResourceList(ResourceList<T>),
    UniqueResource(WrappedResource<T>),
}

impl TerraformCloud {
    pub fn new<S: Into<String>>(token: S) -> Self {
        Self {
            token: token.into(),
            client: reqwest::Client::new(),
        }
    }
}
