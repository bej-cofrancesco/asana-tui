use fake::Dummy;

/// Defines user data structure.
///
#[derive(Clone, Debug, Dummy, PartialEq, Eq)]
pub struct User {
    pub gid: String,
    pub name: String,
    pub email: String,
}

/// Defines workspace data structure.
///
#[derive(Clone, Debug, Dummy, PartialEq, Eq)]
pub struct Workspace {
    pub gid: String,
    pub name: String,
}

/// Defines section data structure (for kanban boards).
///
#[derive(Clone, Debug, Dummy, PartialEq, Eq)]
pub struct Section {
    pub gid: String,
    pub name: String,
}

/// Defines tag data structure.
///
#[derive(Clone, Debug, Dummy, PartialEq, Eq)]
pub struct Tag {
    pub gid: String,
    pub name: String,
}

/// Defines enum option for custom fields.
///
#[derive(Clone, Debug, Dummy, PartialEq, Eq)]
pub struct EnumOption {
    pub gid: String,
    pub name: String,
    pub enabled: bool,
    pub color: Option<String>,
}

/// Defines custom field data structure.
///
#[derive(Clone, Debug, Dummy, PartialEq)]
pub struct CustomField {
    pub gid: String,
    pub name: String,
    pub resource_subtype: String, // text, number, date, enum, multi_enum, people, reference
    pub representation_type: Option<String>, // text, enum, multi_enum, number, date, people, formula, custom_id, reference
    pub id_prefix: Option<String>, // Custom ID prefix (only set for custom_id fields)
    pub enum_options: Vec<EnumOption>, // For enum and multi_enum types
    // Values (only one will be set based on resource_subtype)
    pub text_value: Option<String>,
    pub number_value: Option<f64>,
    pub date_value: Option<String>,         // ISO date string
    pub enum_value: Option<EnumOption>,     // For single enum
    pub multi_enum_values: Vec<EnumOption>, // For multi_enum
    pub people_value: Vec<User>,            // For people type
    pub enabled: bool,                      // Whether the custom field is enabled on this task
}

/// Defines task data structure.
///
#[derive(Clone, Debug, Dummy, PartialEq)]
pub struct Task {
    pub gid: String,
    pub name: String,
    pub completed: bool,
    pub notes: Option<String>,
    pub assignee: Option<User>,
    pub due_date: Option<String>,
    pub due_on: Option<String>,
    pub start_on: Option<String>,
    pub section: Option<Section>,
    pub tags: Vec<Tag>,
    pub custom_fields: Vec<CustomField>, // Custom fields on this task
    pub created_at: Option<String>,
    pub modified_at: Option<String>,
    pub num_subtasks: usize,
    pub num_comments: usize,
}

/// Defines story/comment data structure.
///
#[derive(Clone, Debug, Dummy, PartialEq, Eq)]
pub struct Story {
    pub gid: String,
    pub text: String,
    pub created_at: Option<String>,
    pub created_by: Option<User>,
    pub resource_subtype: Option<String>, // "comment_added" for comments, system activity otherwise
}

/// Defines project data structure.
///
#[derive(Clone, Debug, Dummy, PartialEq)]
pub struct Project {
    pub gid: String,
    pub name: String,
    pub archived: bool,
    pub color: String,
    pub notes: String,
}
