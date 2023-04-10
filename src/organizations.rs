use std::fmt::Display;

use serde::Deserialize;

use super::{TerraformCloud, TerraformResult};

type OrganizationList = TerraformResult<Organization>;

#[derive(Debug, Deserialize, Clone)]
pub struct Organization {
    pub id: String,
    pub r#type: String,

    pub attributes: OrganizationAttributes,

    #[serde(flatten)]
    pub extra: serde_json::Value,
}

impl Display for Organization {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.attributes.name, self.id)
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct OrganizationAttributes {
    pub name: String,
    pub email: String,

    #[serde(flatten)]
    pub extra: serde_json::Value,
}

impl TerraformCloud {
    pub async fn list_organizations(&self) -> anyhow::Result<Vec<Organization>> {
        let organizations = self
            .client
            .get("https://app.terraform.io/api/v2/organizations")
            .bearer_auth(&self.token)
            .header(reqwest::header::CONTENT_TYPE, "application/vnd.api+json")
            .send()
            .await?
            .json::<OrganizationList>()
            .await?;

        Ok(organizations.to_list()?)
    }
}
