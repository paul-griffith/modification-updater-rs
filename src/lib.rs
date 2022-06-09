use chrono::{DateTime, SecondsFormat, Utc};
use hex_fmt::HexFmt;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[repr(i32)]
pub enum ApplicationScope {
    #[serde(rename = "N")]
    None = 0b000,
    #[serde(rename = "G")]
    Gateway = 0b001,
    #[serde(rename = "D")]
    Designer = 0b010,
    #[serde(rename = "C")]
    Client = 0b100,
    #[serde(rename = "A")]
    All = 0b111,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct ResourceManifest {
    pub scope: ApplicationScope,
    pub version: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub documentation: Option<String>,
    #[serde(default, skip_serializing_if = "skip_serializing_false")]
    pub locked: bool,
    #[serde(default)]
    pub restricted: bool,
    #[serde(default)]
    pub overridable: bool,
    pub files: Vec<String>,
    pub attributes: BTreeMap<String, Value>,
}

fn skip_serializing_false(field: &bool) -> bool {
    !*field
}

pub struct ProjectResource {
    pub manifest: ResourceManifest,
    pub data: HashMap<String, Vec<u8>>,
}

const LAST_MODIFICATION: &str = "lastModification";
const LAST_MODIFICATION_SIGNATURE: &str = "lastModificationSignature";

impl ProjectResource {
    pub fn get_signature(self) -> String {
        let mut modified_attributes = self.manifest.attributes.clone();
        modified_attributes.remove(LAST_MODIFICATION_SIGNATURE);
        let without_last_modification = ResourceManifest {
            attributes: modified_attributes,
            ..self.manifest
        };
        calculate_content_digest(without_last_modification, self.data)
    }

    pub fn update(self, actor: &str, timestamp: DateTime<Utc>) -> ProjectResource {
        let mut to_sign = self.manifest.attributes.clone();
        to_sign.remove(LAST_MODIFICATION_SIGNATURE);
        to_sign.insert(
            String::from(LAST_MODIFICATION),
            json!({
                "actor": actor,
                "timestamp": timestamp.to_rfc3339_opts(SecondsFormat::Secs, true)
            }),
        );
        let intermediate_manifest = ResourceManifest {
            attributes: to_sign.clone(),
            ..self.manifest.clone()
        };
        let new_signature = calculate_content_digest(intermediate_manifest, self.data.clone());
        to_sign.insert(
            String::from(LAST_MODIFICATION_SIGNATURE),
            json!(new_signature),
        );

        ProjectResource {
            manifest: ResourceManifest {
                attributes: to_sign,
                ..self.manifest
            },
            data: self.data,
        }
    }

    pub fn from_path(path: &str) -> Result<ProjectResource, Box<dyn std::error::Error>> {
        let root_path = PathBuf::from(path);
        let resource_path = root_path.join(Path::new("resource.json"));
        let resource_file = fs::read(resource_path)?;
        let manifest: ResourceManifest = serde_json::from_slice(&resource_file)?;
        let data: HashMap<String, Vec<u8>> = manifest
            .files
            .iter()
            .map(|data_key| {
                (
                    data_key.clone(),
                    fs::read(root_path.join(data_key)).unwrap(),
                )
            })
            .collect();
        Ok(ProjectResource { manifest, data })
    }
}

fn calculate_content_digest(manifest: ResourceManifest, data: HashMap<String, Vec<u8>>) -> String {
    let mut hasher = Sha256::new();

    hasher.update(&(manifest.scope as i32).to_be_bytes());

    if let Some(documentation) = manifest.documentation {
        hasher.update(documentation.as_bytes());
    }

    hasher.update(&manifest.version.to_be_bytes());
    hasher.update(&(if manifest.locked { 1i8 } else { 0i8 }).to_be_bytes());
    hasher.update(&(if manifest.restricted { 1i8 } else { 0i8 }).to_be_bytes());
    hasher.update(&(if manifest.overridable { 1i8 } else { 0i8 }).to_be_bytes());

    let files = manifest.files.iter().sorted();
    for key in files {
        hasher.update(key.as_bytes());
        let data: &Vec<u8> = data.get(key).unwrap(); // TODO empty data should be allowed
        hasher.update(data)
    }

    let attribute_keys = manifest.attributes.keys().sorted();
    for attribute_key in attribute_keys {
        hasher.update(attribute_key.as_bytes());
        let entry = manifest.attributes.get(attribute_key).unwrap();
        let value = serde_json::to_string(entry).unwrap();
        hasher.update(value.as_bytes());
    }

    let result = hasher.finalize();

    format!("{}", HexFmt(&result[..]))
}
