use modification_updater::{ApplicationScope, Attributes, LastModification, ProjectResource, ResourceManifest};
use serde_json::json;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use chrono::{DateTime, Utc};
use rstest::rstest;

#[test]
fn script_deserialization() {
    let file = fs::read("tests/script/resource.json").unwrap();
    let actual: ResourceManifest = serde_json::from_slice(&file).unwrap();
    assert_eq!(actual.scope, ApplicationScope::All);
    assert_eq!(actual.version, 1);
    assert_eq!(actual.documentation, None);
    assert_eq!(actual.locked, false);
    assert_eq!(actual.restricted, false);
    assert_eq!(actual.overridable, true);
    assert_eq!(actual.files, vec!["code.py"]);
    assert_eq!(
        actual.attributes.last_modification_signature.unwrap(),
        "7ea951abc0ddc97f549f41a5670b06aa513b30e189050159f40e207cfe502b02"
    )
}

#[test]
fn script_serialization() {
    let expected = ResourceManifest {
        scope: ApplicationScope::All,
        version: 1,
        documentation: None,
        locked: false,
        restricted: false,
        overridable: true,
        files: vec![String::from("code.py")],
        attributes: Attributes {
            last_modification: LastModification {
                actor: "qq".to_string(),
                timestamp: "2022-05-26T23:20:28Z".parse::<DateTime<Utc>>().unwrap()
            },
            last_modification_signature: Some("7ea951abc0ddc97f549f41a5670b06aa513b30e189050159f40e207cfe502b02".to_string()),
            attributes: BTreeMap::new()
        }
    };
    let resource_as_string = serde_json::to_string_pretty(&expected).unwrap();
    assert_eq!(
        resource_as_string,
        fs::read_to_string(Path::new("tests/script/resource.json")).unwrap()
    );
}

#[test]
fn view_serialization() {
    let expected = ResourceManifest {
        scope: ApplicationScope::Gateway,
        version: 1,
        documentation: None,
        locked: false,
        restricted: false,
        overridable: true,
        files: vec![String::from("view.json"), String::from("thumbnail.png")],
        attributes: Attributes {
            last_modification: LastModification {
                actor: "qq".to_string(),
                timestamp: "2022-05-21T00:01:56Z".parse::<DateTime<Utc>>().unwrap()
            },
            last_modification_signature: Some("1f2e193ab0b2be15cef750b100bf5c6906b7a92fbb5e7c4f8fb7b68e83b4eb89".to_string()),
            attributes: BTreeMap::new()
        }
    };
    let resource_as_string = serde_json::to_string_pretty(&expected).unwrap();
    assert_eq!(
        resource_as_string,
        fs::read_to_string(Path::new("tests/view/resource.json")).unwrap()
    );
}

#[rstest]
#[case::script("script", "7ea951abc0ddc97f549f41a5670b06aa513b30e189050159f40e207cfe502b02")]
#[case::script2("script2", "aa5f6ff86772d32ddad86da18914f835769ccd49e3603e8aea63f5b2fcaf7b08")]
#[case::view("view", "1f2e193ab0b2be15cef750b100bf5c6906b7a92fbb5e7c4f8fb7b68e83b4eb89")]
#[case::group("group", "bd5f1e421d6fe1e3e5fabfd90bcf3cde685faaf3f7fcdbb61c0857f0f6bbf8cd")]
#[case::window("window", "0aeb36c0376059d3a4f1c30d124a8341ade82303c9faf97c733f6f2cd949770f")]
#[case::report("report", "1b580972bdd0e5fb515849b7a0d3a3df224a6d8fd6eed06ab1c762e78eedfa10")]
#[case::named_query("named_query", "834eca8162b3942191a05f3d53412af34ea77280f61a38a47c5ff9a6340cb909")]
#[case::python("python", "0cddcfe5f55db6c853459c5d71a96090e34191a04c5a8beb3b4d85a411b66ded")]
fn signature_tests(#[case] file: &str, #[case] signature: &str) {
    let resource = ProjectResource::from_path(&Path::new("tests/").join(file)).unwrap();
    let actual = resource.get_signature();
    assert_eq!(
        actual,
        signature
    );
}
