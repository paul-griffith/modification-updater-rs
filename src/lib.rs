use chrono::{DateTime, Utc};
use hex_fmt::HexFmt;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::Path;

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
    #[serde(rename = "CD")]
    ClientDesigner = 0b110,
    #[serde(rename = "CG")]
    ClientGateway = 0b101,
    #[serde(rename = "DG")]
    DesignerGateway = 0b011,
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
    pub attributes: Attributes,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Attributes {
    #[serde(rename = "lastModification")]
    pub last_modification: LastModification,
    #[serde(
        rename = "lastModificationSignature",
        skip_serializing_if = "Option::is_none"
    )]
    pub last_modification_signature: Option<String>,

    #[serde(flatten)]
    pub attributes: BTreeMap<String, Value>,
}

impl Attributes {
    fn sorted_entries(&self) -> Vec<String> {
        let mut keys = self.attributes.keys().cloned().collect_vec();
        keys.push(LAST_MODIFICATION.to_string());
        keys.push(LAST_MODIFICATION_SIGNATURE.to_string());
        keys.sort();

        let mut ret: Vec<String> = vec![];
        for key in keys {
            if key == LAST_MODIFICATION {
                ret.push(LAST_MODIFICATION.to_string());
                ret.push(serde_json::to_string(&self.last_modification).unwrap())
            } else if key == LAST_MODIFICATION_SIGNATURE {
                if let Some(signature) = &self.last_modification_signature {
                    ret.push(LAST_MODIFICATION_SIGNATURE.to_string());
                    ret.push(signature.to_string())
                }
            } else {
                ret.push(key.to_string());
                ret.push(serde_json::to_string(&self.attributes[&key]).unwrap());
            }
        }

        ret
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct LastModification {
    pub actor: String,
    #[serde(with = "ignition_date_format")]
    pub timestamp: DateTime<Utc>,
}

fn skip_serializing_false(field: &bool) -> bool {
    !*field
}

mod ignition_date_format {
    use chrono::{DateTime, SecondsFormat, Utc};
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&date.to_rfc3339_opts(SecondsFormat::Secs, true))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        s.parse::<DateTime<Utc>>().map_err(serde::de::Error::custom)
    }
}

pub struct ProjectResource {
    pub manifest: ResourceManifest,
    pub data: HashMap<String, Vec<u8>>,
}

pub const LAST_MODIFICATION: &str = "lastModification";
pub const LAST_MODIFICATION_SIGNATURE: &str = "lastModificationSignature";

impl ProjectResource {
    pub fn get_signature(self) -> String {
        let to_sign = Attributes {
            last_modification: self.manifest.attributes.last_modification,
            last_modification_signature: None,
            attributes: self.manifest.attributes.attributes,
        };
        let without_last_modification = ResourceManifest {
            attributes: to_sign,
            ..self.manifest
        };
        calculate_content_digest(without_last_modification, self.data)
    }

    pub fn update(self, modification: LastModification) -> ProjectResource {
        let to_sign = Attributes {
            last_modification: modification.clone(),
            last_modification_signature: None,
            attributes: self.manifest.attributes.attributes.clone(),
        };
        let intermediate_manifest = ResourceManifest {
            attributes: to_sign.clone(),
            ..self.manifest.clone()
        };
        let new_signature = calculate_content_digest(intermediate_manifest, self.data.clone());

        ProjectResource {
            manifest: ResourceManifest {
                attributes: Attributes {
                    last_modification: modification,
                    last_modification_signature: Some(new_signature),
                    ..to_sign
                },
                ..self.manifest
            },
            data: self.data,
        }
    }

    pub fn from_path(path: &Path) -> Result<ProjectResource, Box<dyn std::error::Error>> {
        assert!(
            path.is_dir(),
            "Supplied path {} was not a directory",
            path.display()
        );
        let resource_path = path.join("resource.json");
        let resource_file = fs::read(resource_path)?;
        let manifest: ResourceManifest = serde_json::from_slice(&resource_file)?;
        let data: HashMap<String, Vec<u8>> = manifest
            .files
            .iter()
            .map(|data_key| (data_key.clone(), fs::read(path.join(data_key)).unwrap()))
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

    for attribute in manifest.attributes.sorted_entries() {
        dbg!(attribute.clone());
        hasher.update(attribute.as_bytes());
    }

    let result = hasher.finalize();

    format!("{}", HexFmt(&result[..]))
}
