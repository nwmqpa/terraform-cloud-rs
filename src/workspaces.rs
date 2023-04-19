use std::fmt::Display;

use serde::Deserialize;

use super::{ResourceList, TerraformCloud};

type WorkspacesList = ResourceList<Workspace>;

#[derive(Debug, Deserialize, Clone)]
pub struct Workspace {
    pub id: String,
    pub r#type: String,

    pub attributes: WorkspaceAttributes,

    #[serde(flatten)]
    pub extra: serde_json::Value,
}

#[derive(Debug, Deserialize, Clone)]
pub struct WorkspaceAttributes {
    pub name: String,

    #[serde(flatten)]
    pub extra: serde_json::Value,
}
    
impl Display for Workspace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.attributes.name, self.id)
    }
}

impl TerraformCloud {
    pub async fn list_workspaces<S: Display>(
        &self,
        organization_name: S,
    ) -> anyhow::Result<Vec<Workspace>> {
        let workspaces = self
            .client
            .get(format!(
                "https://app.terraform.io/api/v2/organizations/{organization_name}/workspaces"
            ))
            .bearer_auth(&self.token)
            .header(reqwest::header::CONTENT_TYPE, "application/vnd.api+json")
            .send()
            .await?
            .json::<WorkspacesList>()
            .await?;

        Ok(workspaces.data)
    }

    pub async fn lock_workspace<S: Display>(
        &self,
        workspace_id: S,
        reason: impl ToString,
    ) -> anyhow::Result<()> {
        self.client
            .post(format!(
                "https://app.terraform.io/api/v2/workspaces/{workspace_id}/actions/lock"
            ))
            .bearer_auth(&self.token)
            .header(reqwest::header::CONTENT_TYPE, "application/vnd.api+json")
            .body(
                serde_json::json!({
                    "reason": reason.to_string(),
                })
                .to_string(),
            )
            .send()
            .await?;

        Ok(())
    }

    pub async fn unlock_workspace<S: Display>(&self, workspace_id: S) -> anyhow::Result<()> {
        self.client
            .post(format!(
                "https://app.terraform.io/api/v2/workspaces/{workspace_id}/actions/unlock"
            ))
            .bearer_auth(&self.token)
            .header(reqwest::header::CONTENT_TYPE, "application/vnd.api+json")
            .send()
            .await?;

        Ok(())
    }
}
