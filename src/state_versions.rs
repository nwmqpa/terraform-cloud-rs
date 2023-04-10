use std::{
    collections::HashMap,
    convert::identity,
    fmt::{Display, Formatter},
    io::Write,
};

use base64::{alphabet, engine, write};
use md5::Digest;
use reqwest::Version;
use serde::{Deserialize, Serialize};

use super::{TerraformCloud, TerraformResult};

type WrappedStateVersion = TerraformResult<StateVersion>;

#[derive(Debug, Deserialize)]
pub struct StateVersion {
    pub id: String,
    pub r#type: String,

    pub attributes: Option<StateVersionAttributes>,

    #[serde(flatten)]
    pub extra: serde_json::Value,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct StateVersionAttributes {
    pub created_at: String,
    pub hosted_json_state_download_url: String,
    pub hosted_state_download_url: String,
}

impl TerraformCloud {
    pub async fn fetch_current_state_version_for_workspace<S: Display>(
        &self,
        workspace_id: S,
    ) -> anyhow::Result<StateVersion> {
        let state_version = self
            .client
            .get(format!(
                "https://app.terraform.io/api/v2/workspaces/{workspace_id}/current-state-version"
            ))
            .bearer_auth(&self.token)
            .header(reqwest::header::CONTENT_TYPE, "application/vnd.api+json")
            .send()
            .await?
            .json::<WrappedStateVersion>()
            .await?;

        Ok(state_version.to_unique()?)
    }

    pub async fn create_state_version<S: Display>(
        &self,
        workspace_id: S,
        input: &CreateStateVersionInput,
    ) -> anyhow::Result<StateVersion> {
        let body = serde_json::to_string(input)?;

        let state_version = self
            .client
            .post(format!(
                "https://app.terraform.io/api/v2/workspaces/{workspace_id}/state-versions",
            ))
            .bearer_auth(&self.token)
            .body(body)
            .header(reqwest::header::CONTENT_TYPE, "application/vnd.api+json")
            .version(Version::HTTP_11)
            .send()
            .await?
            .json::<WrappedStateVersion>()
            .await?;

        Ok(state_version.to_unique()?)
    }
}

#[derive(Debug, Serialize)]
pub struct CreateStateVersionInput {
    data: CreateStateVersionInputData,
}

#[derive(Debug, Serialize)]
pub struct CreateStateVersionInputData {
    r#type: String,
    attributes: CreateStateVersionInputAttributes,
    relationships: Option<CreateStateVersionInputRelationships>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct CreateStateVersionInputAttributes {
    md5: String,
    serial: u64,
    lineage: Option<String>,
    state: String,
    json_state: Option<String>,
    json_state_outputs: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CreateStateVersionInputRelationships {
    run: CreateStateVersionInputRunRelationship,
}

#[derive(Debug, Serialize)]
pub struct CreateStateVersionInputRunRelationship {
    data: CreateStateVersionInputRelationshipData,
}

#[derive(Debug, Serialize)]
pub struct CreateStateVersionInputRelationshipData {
    r#type: String,
    id: String,
}

#[derive(Debug, Serialize, Default)]
pub struct CreateStateVersionInputBuilder {
    lineage: Option<String>,
    md5: Option<String>,
    serial: Option<u64>,
    state: Option<String>,
    json_state: Option<String>,
    json_state_outputs: Option<String>,
    run_id: Option<String>,
}

impl CreateStateVersionInput {
    pub fn builder() -> CreateStateVersionInputBuilder {
        CreateStateVersionInputBuilder::default()
    }
}

impl TryFrom<CreateStateVersionInputBuilder> for CreateStateVersionInput {
    type Error = anyhow::Error;

    fn try_from(builder: CreateStateVersionInputBuilder) -> Result<Self, Self::Error> {
        Ok(Self {
            data: CreateStateVersionInputData {
                r#type: "state-versions".to_string(),
                attributes: CreateStateVersionInputAttributes {
                    md5: builder
                        .md5
                        .ok_or_else(|| anyhow::anyhow!("md5 is required"))?,
                    serial: builder
                        .serial
                        .ok_or_else(|| anyhow::anyhow!("serial is required"))?,
                    state: builder
                        .state
                        .ok_or_else(|| anyhow::anyhow!("state is required"))?,
                    json_state: builder.json_state,
                    lineage: builder.lineage,
                    json_state_outputs: builder.json_state_outputs,
                },
                relationships: builder
                    .run_id
                    .map(|run_id| CreateStateVersionInputRelationships {
                        run: CreateStateVersionInputRunRelationship {
                            data: CreateStateVersionInputRelationshipData {
                                r#type: "runs".to_string(),
                                id: run_id,
                            },
                        },
                    }),
            },
        })
    }
}

impl TryFrom<TerraformState> for CreateStateVersionInput {
    type Error = anyhow::Error;

    fn try_from(state: TerraformState) -> Result<Self, Self::Error> {
        let mut builder = CreateStateVersionInput::builder();

        builder.lineage = Some(state.lineage.clone());
        builder.md5 = Some(state.md5()?);
        builder.serial = Some(state.serial);
        builder.state = Some(state.to_base64_encoded_json()?);

        Ok(builder.try_into()?)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TerraformState {
    pub version: u64,
    pub terraform_version: String,
    pub serial: u64,
    pub lineage: String,
    pub outputs: HashMap<String, TerraformStateOutput>,
    pub resources: Vec<TerraformResource>,
}

impl TerraformState {
    pub fn md5(&self) -> anyhow::Result<String> {
        let mut hasher = md5::Md5::new();
        hasher.update(self.to_json()?);
        Ok(format!("{:x}", hasher.finalize()))
    }

    fn to_json(&self) -> anyhow::Result<String> {
        Ok(serde_json::to_string(self)?)
    }

    fn to_base64_encoded_json(&self) -> anyhow::Result<String> {
        let engine =
            engine::GeneralPurpose::new(&alphabet::STANDARD, engine::general_purpose::NO_PAD);

        let mut buffer = write::EncoderStringWriter::new(&engine);

        buffer.write_all(self.to_json()?.as_bytes())?;

        Ok(buffer.into_inner())
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TerraformStateOutput {
    pub sensitive: Option<bool>,
    pub r#type: serde_json::Value,
    pub value: serde_json::Value,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TerraformResource {
    pub r#type: String,
    pub name: String,
    pub provider: String,
    pub mode: String,
    pub instances: Vec<serde_json::Value>,
    pub module: Option<String>,

    #[serde(flatten)]
    pub extra: serde_json::Value,
}

impl Display for TerraformResource {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        vec![
            if self.mode == "data" {
                Some("data".to_string())
            } else {
                None
            },
            self.module.as_ref().cloned(),
            Some(self.r#type.clone()),
            Some(self.name.clone()),
        ]
        .into_iter()
        .filter_map(identity)
        .collect::<Vec<String>>()
        .join(".")
        .fmt(f)
    }
}
