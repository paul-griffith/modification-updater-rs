use modification_updater::{ApplicationScope, ProjectResource, ResourceManifest};
use serde_json::json;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

fn create_script_manifest() -> ResourceManifest {
    ResourceManifest {
        scope: ApplicationScope::All,
        version: 1,
        documentation: None,
        locked: false,
        restricted: false,
        overridable: true,
        files: vec![String::from("code.py")],
        attributes: BTreeMap::from([
            (
                String::from("lastModification"),
                json!({
                    "actor": "qq",
                    "timestamp": "2022-05-26T23:20:28Z",
                }),
            ),
            (
                String::from("lastModificationSignature"),
                json!("7ea951abc0ddc97f549f41a5670b06aa513b30e189050159f40e207cfe502b02"),
            ),
        ]),
    }
}

#[test]
fn test_deserialization() {
    let expected = create_script_manifest();
    let file = fs::read("tests/script/resource.json").unwrap();
    let actual: ResourceManifest = serde_json::from_slice(&file).unwrap();
    assert_eq!(expected, actual);
}

#[test]
fn test_serialization() {
    let expected = create_script_manifest();
    let resource_as_string = serde_json::to_string_pretty(&expected).unwrap();
    assert_eq!(
        resource_as_string,
        fs::read_to_string(Path::new("tests/script/resource.json")).unwrap()
    );
}

#[test]
fn test_signature_calculation() {
    let resource = ProjectResource::from_path("tests/script").unwrap();
    let signature = resource.get_signature();
    assert_eq!(
        signature,
        "7ea951abc0ddc97f549f41a5670b06aa513b30e189050159f40e207cfe502b02"
    )
}
