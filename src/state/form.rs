//! Form editing state types.
//!
//! This module contains types related to form editing, including form fields,
//! custom field values, and form state management.

/// Custom field value for form editing.
///
#[derive(Clone, Debug, PartialEq)]
pub enum CustomFieldValue {
    Text(String),
    Number(Option<f64>),
    Date(Option<String>),
    Enum(Option<String>),   // GID of selected enum option
    MultiEnum(Vec<String>), // GIDs of selected enum options
    People(Vec<String>),    // GIDs of selected users
}

/// Specifying edit form field state.
///
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum EditFormState {
    Name,
    Notes,
    Assignee,
    DueDate,
    Section,
    CustomField(usize), // Index into custom_fields array
}

/// Specifying task filter options.
///
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum TaskFilter {
    All,
    Incomplete,
    Completed,
    #[allow(dead_code)]
    Assignee(Option<String>),
}

/// Get the base shortcuts list.
///
pub fn base_shortcuts() -> Vec<String> {
    vec![]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_custom_field_value_text() {
        let value = CustomFieldValue::Text("Test".to_string());
        assert!(matches!(value, CustomFieldValue::Text(_)));
        if let CustomFieldValue::Text(s) = value {
            assert_eq!(s, "Test");
        }
    }

    #[test]
    fn test_custom_field_value_number() {
        let value = CustomFieldValue::Number(Some(42.5));
        assert!(matches!(value, CustomFieldValue::Number(_)));
        if let CustomFieldValue::Number(Some(n)) = value {
            assert_eq!(n, 42.5);
        }

        let value = CustomFieldValue::Number(None);
        assert!(matches!(value, CustomFieldValue::Number(None)));
    }

    #[test]
    fn test_custom_field_value_date() {
        let value = CustomFieldValue::Date(Some("2024-01-01".to_string()));
        assert!(matches!(value, CustomFieldValue::Date(_)));
        if let CustomFieldValue::Date(Some(d)) = value {
            assert_eq!(d, "2024-01-01");
        }
    }

    #[test]
    fn test_custom_field_value_enum() {
        let value = CustomFieldValue::Enum(Some("123456".to_string()));
        assert!(matches!(value, CustomFieldValue::Enum(_)));
        if let CustomFieldValue::Enum(Some(gid)) = value {
            assert_eq!(gid, "123456");
        }
    }

    #[test]
    fn test_custom_field_value_multi_enum() {
        let value = CustomFieldValue::MultiEnum(vec!["123".to_string(), "456".to_string()]);
        assert!(matches!(value, CustomFieldValue::MultiEnum(_)));
        if let CustomFieldValue::MultiEnum(gids) = value {
            assert_eq!(gids.len(), 2);
            assert_eq!(gids[0], "123");
            assert_eq!(gids[1], "456");
        }
    }

    #[test]
    fn test_custom_field_value_people() {
        let value = CustomFieldValue::People(vec!["user1".to_string(), "user2".to_string()]);
        assert!(matches!(value, CustomFieldValue::People(_)));
        if let CustomFieldValue::People(gids) = value {
            assert_eq!(gids.len(), 2);
            assert_eq!(gids[0], "user1");
            assert_eq!(gids[1], "user2");
        }
    }

    #[test]
    fn test_edit_form_state() {
        assert_eq!(EditFormState::Name, EditFormState::Name);
        assert_eq!(EditFormState::Notes, EditFormState::Notes);
        assert_eq!(EditFormState::Assignee, EditFormState::Assignee);
        assert_eq!(EditFormState::DueDate, EditFormState::DueDate);
        assert_eq!(EditFormState::Section, EditFormState::Section);
        assert_eq!(EditFormState::CustomField(0), EditFormState::CustomField(0));
        assert_ne!(EditFormState::CustomField(0), EditFormState::CustomField(1));
    }

    #[test]
    fn test_task_filter() {
        assert_eq!(TaskFilter::All, TaskFilter::All);
        assert_eq!(TaskFilter::Incomplete, TaskFilter::Incomplete);
        assert_eq!(TaskFilter::Completed, TaskFilter::Completed);
        assert_eq!(
            TaskFilter::Assignee(Some("123".to_string())),
            TaskFilter::Assignee(Some("123".to_string()))
        );
        assert_eq!(TaskFilter::Assignee(None), TaskFilter::Assignee(None));
    }

    #[test]
    fn test_base_shortcuts() {
        let shortcuts = base_shortcuts();
        assert!(shortcuts.is_empty());
    }
}
