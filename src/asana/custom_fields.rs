//! Custom field validation and building logic.
//!
//! This module contains shared logic for validating and building custom field
//! values for both task creation and updates.

use crate::asana::CustomField;
use crate::state::CustomFieldValue;
use anyhow::Result;
use log::*;
use std::collections::{HashMap, HashSet};

/// Mode for building custom fields (Create vs Update).
#[derive(Debug, Clone, Copy)]
pub enum BuildMode {
    Create,
    Update,
}

/// Builder for custom field values.
pub struct CustomFieldBuilder;

impl CustomFieldBuilder {
    /// Validate and build custom fields for task creation or update.
    ///
    /// # Arguments
    /// * `custom_fields` - Map of custom field GID to value
    /// * `project_custom_fields` - Available custom fields for the project
    /// * `mode` - Whether building for create or update
    ///
    /// # Returns
    /// A JSON object map ready to be sent to the Asana API, or an error if validation fails.
    pub fn validate_and_build(
        custom_fields: &HashMap<String, CustomFieldValue>,
        project_custom_fields: &[CustomField],
        mode: BuildMode,
    ) -> Result<serde_json::Map<String, serde_json::Value>> {
        if custom_fields.is_empty() {
            return Ok(serde_json::Map::new());
        }

        // Build set of valid custom field GIDs
        let valid_cf_gids: HashSet<String> = project_custom_fields
            .iter()
            .filter(|cf| {
                // Filter out custom_id fields - they cannot be manually set
                let is_custom_id = cf
                    .representation_type
                    .as_ref()
                    .map(|s| s == "custom_id")
                    .unwrap_or(false)
                    || cf.id_prefix.is_some();
                !is_custom_id
            })
            .map(|cf| cf.gid.clone())
            .collect();

        // Build map of custom field GID -> enum option GIDs for validation
        let mut cf_enum_options: HashMap<String, HashSet<String>> = HashMap::new();
        for cf in project_custom_fields {
            if cf.resource_subtype == "enum" || cf.resource_subtype == "multi_enum" {
                let enum_gids: HashSet<String> =
                    cf.enum_options.iter().map(|eo| eo.gid.clone()).collect();
                cf_enum_options.insert(cf.gid.clone(), enum_gids);
            }
        }

        let mut custom_fields_obj = serde_json::Map::new();

        for (gid, value) in custom_fields {
            // Skip invalid GIDs
            if gid.is_empty() || gid == "0" {
                warn!("Skipping invalid custom field GID (empty or '0'): {}", gid);
                continue;
            }

            // For create mode, validate against project fields
            if matches!(mode, BuildMode::Create) && !valid_cf_gids.contains(gid.as_str()) {
                warn!(
                    "Skipping custom field GID not in project: {} (valid GIDs: {:?})",
                    gid, valid_cf_gids
                );
                continue;
            }

            // Build the value based on type
            let cf_value = match value {
                CustomFieldValue::Text(s) if !s.trim().is_empty() => {
                    Some(serde_json::Value::String(s.trim().to_string()))
                }
                CustomFieldValue::Number(Some(n)) => {
                    if let Some(num) = serde_json::Number::from_f64(*n) {
                        Some(serde_json::Value::Number(num))
                    } else {
                        warn!("Invalid number value for custom field {}, skipping", gid);
                        None
                    }
                }
                CustomFieldValue::Date(Some(d)) if !d.trim().is_empty() => {
                    Some(serde_json::Value::String(d.trim().to_string()))
                }
                CustomFieldValue::Enum(Some(enum_gid)) => {
                    if enum_gid.is_empty() || enum_gid == "0" {
                        warn!(
                            "Skipping invalid enum option GID for custom field {}: {}",
                            gid, enum_gid
                        );
                        None
                    } else {
                        // For create mode, validate enum option
                        if matches!(mode, BuildMode::Create) {
                            if let Some(valid_enum_gids) = cf_enum_options.get(gid) {
                                if !valid_enum_gids.contains(enum_gid.as_str()) {
                                    warn!(
                                        "Skipping invalid enum option GID '{}' for custom field {} (valid options: {:?})",
                                        enum_gid, gid, valid_enum_gids
                                    );
                                    None
                                } else {
                                    Some(serde_json::Value::String(enum_gid.clone()))
                                }
                            } else {
                                warn!(
                                    "Custom field {} is not an enum field or has no enum options",
                                    gid
                                );
                                None
                            }
                        } else {
                            Some(serde_json::Value::String(enum_gid.clone()))
                        }
                    }
                }
                CustomFieldValue::MultiEnum(gids) if !gids.is_empty() => {
                    let valid_enum_gids = cf_enum_options.get(gid).cloned().unwrap_or_default();
                    let valid_gids: Vec<_> = gids
                        .iter()
                        .filter(|enum_gid| {
                            !enum_gid.is_empty()
                                && *enum_gid != "0"
                                && (matches!(mode, BuildMode::Update)
                                    || valid_enum_gids.contains(enum_gid.as_str()))
                        })
                        .cloned()
                        .collect();
                    if valid_gids.is_empty() {
                        warn!(
                            "Skipping multi-enum with no valid option GIDs for custom field {}",
                            gid
                        );
                        None
                    } else {
                        Some(serde_json::Value::Array(
                            valid_gids
                                .iter()
                                .map(|gid| serde_json::Value::String(gid.clone()))
                                .collect(),
                        ))
                    }
                }
                CustomFieldValue::People(gids) if !gids.is_empty() => {
                    let valid_gids: Vec<_> = gids
                        .iter()
                        .filter(|gid| !gid.is_empty() && *gid != "0")
                        .cloned()
                        .collect();
                    if valid_gids.is_empty() {
                        warn!("Skipping people field with no valid user GIDs");
                        None
                    } else {
                        Some(serde_json::Value::Array(
                            valid_gids
                                .iter()
                                .map(|gid| serde_json::Value::String(gid.clone()))
                                .collect(),
                        ))
                    }
                }
                _ => None, // Skip empty values
            };

            if let Some(cf_val) = cf_value {
                // Final validation: ensure no invalid GIDs in the value
                let mut should_skip = false;

                if let Some(enum_gid_str) = cf_val.as_str() {
                    if enum_gid_str == "0" || enum_gid_str.is_empty() {
                        error!(
                            "ERROR: Attempted to add custom field with invalid enum GID '{}' - skipping!",
                            enum_gid_str
                        );
                        should_skip = true;
                    }
                }

                if let Some(arr) = cf_val.as_array() {
                    for item in arr {
                        if let Some(gid_str) = item.as_str() {
                            if gid_str == "0" || gid_str.is_empty() {
                                error!(
                                    "ERROR: Attempted to add custom field with invalid GID '{}' in array - skipping entire field!",
                                    gid_str
                                );
                                should_skip = true;
                                break;
                            }
                        }
                    }
                }

                if !should_skip {
                    custom_fields_obj.insert(gid.clone(), cf_val);
                }
            }
        }

        Ok(custom_fields_obj)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::asana::EnumOption;
    use crate::state::CustomFieldValue;
    use fake::{Fake, Faker};
    use std::collections::HashMap;
    use uuid::Uuid;

    fn create_test_custom_field(
        gid: String,
        resource_subtype: String,
        enum_options: Vec<EnumOption>,
    ) -> CustomField {
        CustomField {
            gid,
            name: Faker.fake(),
            resource_subtype,
            representation_type: None,
            id_prefix: None,
            enum_options,
            text_value: None,
            number_value: None,
            date_value: None,
            enum_value: None,
            multi_enum_values: vec![],
            people_value: vec![],
            enabled: true,
        }
    }

    fn create_test_enum_option(gid: String, name: String) -> EnumOption {
        EnumOption {
            gid,
            name,
            enabled: true,
            color: None,
        }
    }

    #[test]
    fn test_validate_and_build_empty() {
        let custom_fields = HashMap::new();
        let project_custom_fields = vec![];
        let result = CustomFieldBuilder::validate_and_build(
            &custom_fields,
            &project_custom_fields,
            BuildMode::Create,
        );
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_validate_and_build_text_field() {
        let mut custom_fields = HashMap::new();
        custom_fields.insert(
            "123456".to_string(),
            CustomFieldValue::Text("Test value".to_string()),
        );

        let project_custom_fields = vec![create_test_custom_field(
            "123456".to_string(),
            "text".to_string(),
            vec![],
        )];

        let result = CustomFieldBuilder::validate_and_build(
            &custom_fields,
            &project_custom_fields,
            BuildMode::Create,
        );
        assert!(result.is_ok());
        let obj = result.unwrap();
        assert_eq!(obj.len(), 1);
        assert_eq!(
            obj.get("123456").and_then(|v| v.as_str()),
            Some("Test value")
        );
    }

    #[test]
    fn test_validate_and_build_text_field_empty_skipped() {
        let mut custom_fields = HashMap::new();
        custom_fields.insert(
            "123456".to_string(),
            CustomFieldValue::Text("   ".to_string()),
        );

        let project_custom_fields = vec![create_test_custom_field(
            "123456".to_string(),
            "text".to_string(),
            vec![],
        )];

        let result = CustomFieldBuilder::validate_and_build(
            &custom_fields,
            &project_custom_fields,
            BuildMode::Create,
        );
        assert!(result.is_ok());
        let obj = result.unwrap();
        assert!(obj.is_empty());
    }

    #[test]
    fn test_validate_and_build_number_field() {
        let mut custom_fields = HashMap::new();
        custom_fields.insert("123456".to_string(), CustomFieldValue::Number(Some(42.5)));

        let project_custom_fields = vec![create_test_custom_field(
            "123456".to_string(),
            "number".to_string(),
            vec![],
        )];

        let result = CustomFieldBuilder::validate_and_build(
            &custom_fields,
            &project_custom_fields,
            BuildMode::Create,
        );
        assert!(result.is_ok());
        let obj = result.unwrap();
        assert_eq!(obj.len(), 1);
        assert_eq!(obj.get("123456").and_then(|v| v.as_f64()), Some(42.5));
    }

    #[test]
    fn test_validate_and_build_enum_field() {
        let enum_option_gid: String = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440010")
            .expect("Hardcoded test UUID should be valid")
            .to_string();
        let mut custom_fields = HashMap::new();
        custom_fields.insert(
            "123456".to_string(),
            CustomFieldValue::Enum(Some(enum_option_gid.clone())),
        );

        let enum_option = create_test_enum_option(enum_option_gid.clone(), "Option 1".to_string());
        let project_custom_fields = vec![create_test_custom_field(
            "123456".to_string(),
            "enum".to_string(),
            vec![enum_option],
        )];

        let result = CustomFieldBuilder::validate_and_build(
            &custom_fields,
            &project_custom_fields,
            BuildMode::Create,
        );
        assert!(result.is_ok());
        let obj = result.unwrap();
        assert_eq!(obj.len(), 1);
        assert_eq!(
            obj.get("123456").and_then(|v| v.as_str()),
            Some(enum_option_gid.as_str())
        );
    }

    #[test]
    fn test_validate_and_build_enum_field_invalid_option_skipped() {
        let valid_gid: String = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440011")
            .expect("Hardcoded test UUID should be valid")
            .to_string();
        let invalid_gid: String = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440012")
            .expect("Hardcoded test UUID should be valid")
            .to_string();
        let mut custom_fields = HashMap::new();
        custom_fields.insert(
            "123456".to_string(),
            CustomFieldValue::Enum(Some(invalid_gid.clone())),
        );

        let enum_option = create_test_enum_option(valid_gid, "Option 1".to_string());
        let project_custom_fields = vec![create_test_custom_field(
            "123456".to_string(),
            "enum".to_string(),
            vec![enum_option],
        )];

        let result = CustomFieldBuilder::validate_and_build(
            &custom_fields,
            &project_custom_fields,
            BuildMode::Create,
        );
        assert!(result.is_ok());
        let obj = result.unwrap();
        assert!(obj.is_empty());
    }

    #[test]
    fn test_validate_and_build_multi_enum_field() {
        let option1_gid: String = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440020")
            .expect("Hardcoded test UUID should be valid")
            .to_string();
        let option2_gid: String = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440021")
            .expect("Hardcoded test UUID should be valid")
            .to_string();
        let mut custom_fields = HashMap::new();
        custom_fields.insert(
            "123456".to_string(),
            CustomFieldValue::MultiEnum(vec![option1_gid.clone(), option2_gid.clone()]),
        );

        let enum_options = vec![
            create_test_enum_option(option1_gid.clone(), "Option 1".to_string()),
            create_test_enum_option(option2_gid.clone(), "Option 2".to_string()),
        ];
        let project_custom_fields = vec![create_test_custom_field(
            "123456".to_string(),
            "multi_enum".to_string(),
            enum_options,
        )];

        let result = CustomFieldBuilder::validate_and_build(
            &custom_fields,
            &project_custom_fields,
            BuildMode::Create,
        );
        assert!(result.is_ok());
        let obj = result.unwrap();
        assert_eq!(obj.len(), 1);
        if let Some(serde_json::Value::Array(arr)) = obj.get("123456") {
            assert_eq!(arr.len(), 2);
        } else {
            panic!("Expected array value");
        }
    }

    #[test]
    fn test_validate_and_build_people_field() {
        let user1_gid: String = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440030")
            .expect("Hardcoded test UUID should be valid")
            .to_string();
        let user2_gid: String = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440031")
            .expect("Hardcoded test UUID should be valid")
            .to_string();
        let mut custom_fields = HashMap::new();
        custom_fields.insert(
            "123456".to_string(),
            CustomFieldValue::People(vec![user1_gid.clone(), user2_gid.clone()]),
        );

        let project_custom_fields = vec![create_test_custom_field(
            "123456".to_string(),
            "people".to_string(),
            vec![],
        )];

        let result = CustomFieldBuilder::validate_and_build(
            &custom_fields,
            &project_custom_fields,
            BuildMode::Create,
        );
        assert!(result.is_ok());
        let obj = result.unwrap();
        assert_eq!(obj.len(), 1);
        if let Some(serde_json::Value::Array(arr)) = obj.get("123456") {
            assert_eq!(arr.len(), 2);
        } else {
            panic!("Expected array value");
        }
    }

    #[test]
    fn test_validate_and_build_invalid_gid_skipped() {
        let mut custom_fields = HashMap::new();
        custom_fields.insert("0".to_string(), CustomFieldValue::Text("Test".to_string()));
        custom_fields.insert("".to_string(), CustomFieldValue::Text("Test".to_string()));

        let project_custom_fields = vec![];

        let result = CustomFieldBuilder::validate_and_build(
            &custom_fields,
            &project_custom_fields,
            BuildMode::Create,
        );
        assert!(result.is_ok());
        let obj = result.unwrap();
        assert!(obj.is_empty());
    }

    #[test]
    fn test_validate_and_build_create_mode_filters_invalid_fields() {
        let mut custom_fields = HashMap::new();
        custom_fields.insert(
            "123456".to_string(),
            CustomFieldValue::Text("Valid".to_string()),
        );
        custom_fields.insert(
            "999999".to_string(),
            CustomFieldValue::Text("Invalid".to_string()),
        );

        let project_custom_fields = vec![create_test_custom_field(
            "123456".to_string(),
            "text".to_string(),
            vec![],
        )];

        let result = CustomFieldBuilder::validate_and_build(
            &custom_fields,
            &project_custom_fields,
            BuildMode::Create,
        );
        assert!(result.is_ok());
        let obj = result.unwrap();
        assert_eq!(obj.len(), 1);
        assert!(obj.contains_key("123456"));
        assert!(!obj.contains_key("999999"));
    }

    #[test]
    fn test_validate_and_build_update_mode_allows_unknown_fields() {
        let mut custom_fields = HashMap::new();
        custom_fields.insert(
            "999999".to_string(),
            CustomFieldValue::Text("Valid".to_string()),
        );

        let project_custom_fields = vec![];

        let result = CustomFieldBuilder::validate_and_build(
            &custom_fields,
            &project_custom_fields,
            BuildMode::Update,
        );
        assert!(result.is_ok());
        let obj = result.unwrap();
        assert_eq!(obj.len(), 1);
        assert!(obj.contains_key("999999"));
    }
}
