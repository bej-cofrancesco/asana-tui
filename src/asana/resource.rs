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
