use std::fmt::Display;

use serde::Deserialize;

use super::{ResourceList, TerraformCloud};

type VarsetList = ResourceList<Varset>;

#[derive(Debug, Deserialize)]
pub struct Varset {
    pub id: String,
    pub r#type: String,

    pub attributes: VarsetAttributes,
    pub relationships: VarsetRelationships,

    #[serde(flatten)]
    pub extra: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct VarsetAttributes {
    pub name: String,
    pub global: bool,

    #[serde(flatten)]
    pub extra: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct VarsetRelationships {
    pub vars: ResourceList<VarsetVariable>,

    #[serde(flatten)]
    pub extra: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct VarsetVariable {
    pub id: String,
    pub r#type: String,

    pub attributes: Option<VarsetVariableAttributes>,
}

#[derive(Debug, Deserialize)]
pub struct VarsetVariableAttributes {
    pub key: String,
    pub value: Option<String>,

    #[serde(flatten)]
    pub extra: serde_json::Value,
}

impl TerraformCloud {
    pub async fn list_variable_sets_for_workspace<S: Display>(
        &self,
        workspace_id: S,
    ) -> anyhow::Result<Vec<Varset>> {
        let varsets = self
            .client
            .get(format!(
                "https://app.terraform.io/api/v2/workspaces/{workspace_id}/varsets"
            ))
            .bearer_auth(&self.token)
            .header(reqwest::header::CONTENT_TYPE, "application/vnd.api+json")
            .send()
            .await?
            .json::<VarsetList>()
            .await?;

        Ok(varsets.data)
    }

    pub async fn list_variables_for_variable_set<S: Display>(
        &self,
        varset_id: S,
    ) -> anyhow::Result<Vec<VarsetVariable>> {
        let variables = self
            .client
            .get(format!(
                "https://app.terraform.io/api/v2/varsets/{varset_id}/relationships/vars"
            ))
            .bearer_auth(&self.token)
            .header(reqwest::header::CONTENT_TYPE, "application/vnd.api+json")
            .send()
            .await?
            .json::<ResourceList<VarsetVariable>>()
            .await?;

        Ok(variables.data)
    }
}
