use std::fmt::Display;

use serde::Deserialize;

use super::{ResourceList, TerraformCloud};

type WorkspaceVariableList = ResourceList<WorkspaceVariable>;

#[derive(Debug, Deserialize)]
pub struct WorkspaceVariable {
    pub id: String,
    pub r#type: String,

    pub attributes: Option<WorkspaceVariableAttributes>,

    #[serde(flatten)]
    pub extra: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct WorkspaceVariableAttributes {
    pub key: String,
    pub value: Option<String>,
    pub category: String,
    pub sensitive: bool,
    pub hcl: bool,

    #[serde(flatten)]
    pub extra: serde_json::Value,
}

impl TerraformCloud {
    pub async fn list_workspace_variables<S: Display>(
        &self,
        workspace_id: S,
    ) -> anyhow::Result<Vec<WorkspaceVariable>> {
        let workspace_variables = self
            .client
            .get(format!(
                "https://app.terraform.io/api/v2/workspaces/{}/vars",
                workspace_id
            ))
            .bearer_auth(&self.token)
            .header(reqwest::header::CONTENT_TYPE, "application/vnd.api+json")
            .send()
            .await?
            .json::<WorkspaceVariableList>()
            .await?;

        Ok(workspace_variables.data)
    }
}
