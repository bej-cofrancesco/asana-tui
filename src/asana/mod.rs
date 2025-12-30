mod client;
mod models;
mod resource;

pub use resource::*;

use crate::model;
use anyhow::Result;
use chrono::prelude::*;
use client::Client;
use log::*;
use models::Wrapper;

/// Responsible for asynchronous interaction with the Asana API including
/// transformation of response data into explicitly-defined types.
///
pub struct Asana {
    client: Client,
}

impl Asana {
    /// Returns a new instance for the given access token.
    ///
    pub fn new(access_token: &str) -> Asana {
        debug!(
            "Initializing Asana client with personal access token {}...",
            access_token
        );
        Asana {
            client: Client::new(access_token, "https://app.asana.com/api/1.0"),
        }
    }

    /// Returns a tuple containing the current user and the workspaces to which
    /// they have access.
    ///
    pub async fn me(&mut self) -> Result<(User, Vec<Workspace>)> {
        debug!("Requesting authenticated user details...");

        model!(WorkspaceModel "workspaces" { name: String });
        model!(UserModel "users" {
            email: String,
            name: String,
            workspaces: Vec<WorkspaceModel>,
        } WorkspaceModel);

        let data = self.client.get::<UserModel>("me").await?;

        Ok((
            User {
                gid: data.gid,
                name: data.name,
                email: data.email,
            },
            data.workspaces
                .into_iter()
                .map(|w| Workspace {
                    gid: w.gid,
                    name: w.name,
                })
                .collect(),
        ))
    }

    /// Returns a vector of projects for the workspace.
    ///
    pub async fn projects(&mut self, workspace_gid: &str) -> Result<Vec<Project>> {
        debug!(
            "Requesting projects for workspace GID {} (with pagination)...",
            workspace_gid
        );

        model!(ProjectModel "projects" { name: String });

        // Use pagination to handle workspaces with many projects
        let data: Vec<ProjectModel> = self
            .client
            .list_paginated::<ProjectModel>(Some(vec![("workspace", workspace_gid)]), Some(100))
            .await?;

        debug!(
            "Retrieved {} projects for workspace GID {}",
            data.len(),
            workspace_gid
        );

        Ok(data
            .into_iter()
            .map(|p| Project {
                gid: p.gid,
                name: p.name,
                archived: false,
                color: String::new(),
                notes: String::new(),
            })
            .collect())
    }

    /// Returns a vector of tasks for the project.
    /// By default, only fetches incomplete tasks for efficiency.
    ///
    pub async fn tasks(
        &mut self,
        project_gid: &str,
        _workspace_gid: Option<&str>,
        include_completed: bool,
    ) -> Result<Vec<Task>> {
        debug!(
            "Requesting tasks for project GID {} (with pagination, include_completed: {})...",
            project_gid, include_completed
        );

        model!(TaskModel "tasks" { name: String, completed: bool });

        // Build query parameters
        // According to Asana API: "Must specify exactly one of project, tag, section, user task list, or assignee + workspace"
        // So we should NOT include workspace when we have project - they're mutually exclusive
        // The workspace_gid parameter is kept for API compatibility but not used
        let mut params: Vec<(&str, &str)> = vec![("project", project_gid)];

        // Add opt_fields to request section and assignee information via memberships
        // This is needed for kanban board to group tasks by section and show assignees
        // Note: This will override the model's opt_fields, so we need to include all fields we want
        let opt_fields = "resource_type,name,completed,memberships.section.gid,memberships.section.name,assignee.name,assignee.email";
        params.push(("opt_fields", opt_fields));

        // Only filter to incomplete tasks if we don't want completed tasks
        // This is more efficient for large projects when we only need incomplete tasks
        // Store the string in a variable that lives long enough
        let completed_since_str = if !include_completed {
            Some(Utc::now().format("%Y-%m-%dT%H:%M:%S%.fZ").to_string())
        } else {
            None
        };

        if let Some(ref completed_since) = completed_since_str {
            params.push(("completed_since", completed_since.as_str()));
        }

        // Use pagination to handle large result sets
        let data: Vec<TaskModel> = self
            .client
            .list_paginated::<TaskModel>(Some(params), Some(100))
            .await?;

        debug!(
            "Retrieved {} tasks for project GID {}",
            data.len(),
            project_gid
        );

        Ok(data
            .into_iter()
            .map(|t| {
                // Extract section from memberships in extra fields
                let section = if let Some(memberships) =
                    t.extra.get("memberships").and_then(|m| m.as_array())
                {
                    memberships.iter().find_map(|m| {
                        m.get("section").and_then(|s| s.as_object()).and_then(|s| {
                            Some(Section {
                                gid: s.get("gid")?.as_str()?.to_string(),
                                name: s.get("name")?.as_str()?.to_string(),
                            })
                        })
                    })
                } else {
                    None
                };

                // Extract assignee from extra fields
                let assignee = if let Some(assignee_val) = t.extra.get("assignee") {
                    if let Some(assignee_obj) = assignee_val.as_object() {
                        if let (Some(gid_val), Some(name_val)) = (
                            assignee_obj.get("gid").and_then(|v| v.as_str()),
                            assignee_obj.get("name").and_then(|v| v.as_str()),
                        ) {
                            Some(User {
                                gid: gid_val.to_string(),
                                name: name_val.to_string(),
                                email: assignee_obj
                                    .get("email")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("")
                                    .to_string(),
                            })
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                };

                Task {
                    gid: t.gid,
                    name: t.name,
                    completed: t.completed,
                    notes: None,
                    assignee,
                    due_date: None,
                    due_on: None,
                    start_on: None,
                    section,
                    tags: vec![],
                    custom_fields: vec![],
                    created_at: None,
                    modified_at: None,
                    num_subtasks: 0,
                    num_comments: 0,
                }
            })
            .collect())
    }

    /// Returns a vector of incomplete tasks assigned to the user.
    ///
    pub async fn my_tasks(&mut self, user_gid: &str, workspace_gid: &str) -> Result<Vec<Task>> {
        debug!(
            "Requesting tasks for user GID {} and workspace GID {} (with pagination)...",
            user_gid, workspace_gid
        );

        model!(TaskModel "tasks" { name: String, completed: bool });

        // Use pagination to handle large result sets
        let data: Vec<TaskModel> = self
            .client
            .list_paginated::<TaskModel>(
                Some(vec![
                    ("assignee", user_gid),
                    ("workspace", workspace_gid),
                    (
                        "completed_since",
                        &Utc::now().format("%Y-%m-%dT%H:%M:%S%.fZ").to_string(),
                    ),
                ]),
                Some(100),
            )
            .await?;

        debug!("Retrieved {} tasks for user GID {}", data.len(), user_gid);

        Ok(data
            .into_iter()
            .map(|t| Task {
                gid: t.gid,
                name: t.name,
                completed: t.completed,
                notes: None,
                assignee: None,
                due_date: None,
                due_on: None,
                start_on: None,
                section: None,
                tags: vec![],
                custom_fields: vec![],
                created_at: None,
                modified_at: None,
                num_subtasks: 0,
                num_comments: 0,
            })
            .collect())
    }

    /// Parse custom fields from task data JSON.
    ///
    fn parse_custom_fields_from_task_data(
        extra: &std::collections::HashMap<String, serde_json::Value>,
    ) -> Vec<CustomField> {
        let mut custom_fields = Vec::new();

        if let Some(custom_fields_array) = extra.get("custom_fields").and_then(|v| v.as_array()) {
            for cf_val in custom_fields_array {
                if let Some(cf_obj) = cf_val.as_object() {
                    let gid = cf_obj
                        .get("gid")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let name = cf_obj
                        .get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let resource_subtype = cf_obj
                        .get("resource_subtype")
                        .and_then(|v| v.as_str())
                        .unwrap_or("text")
                        .to_string();
                    let enabled = cf_obj
                        .get("enabled")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(true);

                    // Parse enum options
                    let mut enum_options = Vec::new();
                    if let Some(enum_opts_array) =
                        cf_obj.get("enum_options").and_then(|v| v.as_array())
                    {
                        for eo_val in enum_opts_array {
                            if let Some(eo_obj) = eo_val.as_object() {
                                enum_options.push(EnumOption {
                                    gid: eo_obj
                                        .get("gid")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("")
                                        .to_string(),
                                    name: eo_obj
                                        .get("name")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("")
                                        .to_string(),
                                    enabled: eo_obj
                                        .get("enabled")
                                        .and_then(|v| v.as_bool())
                                        .unwrap_or(true),
                                    color: eo_obj
                                        .get("color")
                                        .and_then(|v| v.as_str())
                                        .map(|s| s.to_string()),
                                });
                            }
                        }
                    }

                    // Parse values based on resource_subtype
                    let mut text_value = None;
                    let mut number_value = None;
                    let mut date_value = None;
                    let mut enum_value = None;
                    let mut multi_enum_values = Vec::new();
                    let mut people_value = Vec::new();

                    match resource_subtype.as_str() {
                        "text" => {
                            text_value = cf_obj
                                .get("text_value")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string());
                        }
                        "number" => {
                            number_value = cf_obj.get("number_value").and_then(|v| v.as_f64());
                        }
                        "date" => {
                            if let Some(date_obj) =
                                cf_obj.get("date_value").and_then(|v| v.as_object())
                            {
                                date_value = date_obj
                                    .get("date")
                                    .and_then(|v| v.as_str())
                                    .map(|s| s.to_string());
                            }
                        }
                        "enum" => {
                            if let Some(enum_obj) =
                                cf_obj.get("enum_value").and_then(|v| v.as_object())
                            {
                                enum_value = Some(EnumOption {
                                    gid: enum_obj
                                        .get("gid")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("")
                                        .to_string(),
                                    name: enum_obj
                                        .get("name")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("")
                                        .to_string(),
                                    enabled: enum_obj
                                        .get("enabled")
                                        .and_then(|v| v.as_bool())
                                        .unwrap_or(true),
                                    color: enum_obj
                                        .get("color")
                                        .and_then(|v| v.as_str())
                                        .map(|s| s.to_string()),
                                });
                            }
                        }
                        "multi_enum" => {
                            if let Some(multi_enum_array) =
                                cf_obj.get("multi_enum_values").and_then(|v| v.as_array())
                            {
                                for me_val in multi_enum_array {
                                    if let Some(me_obj) = me_val.as_object() {
                                        multi_enum_values.push(EnumOption {
                                            gid: me_obj
                                                .get("gid")
                                                .and_then(|v| v.as_str())
                                                .unwrap_or("")
                                                .to_string(),
                                            name: me_obj
                                                .get("name")
                                                .and_then(|v| v.as_str())
                                                .unwrap_or("")
                                                .to_string(),
                                            enabled: me_obj
                                                .get("enabled")
                                                .and_then(|v| v.as_bool())
                                                .unwrap_or(true),
                                            color: me_obj
                                                .get("color")
                                                .and_then(|v| v.as_str())
                                                .map(|s| s.to_string()),
                                        });
                                    }
                                }
                            }
                        }
                        "people" => {
                            if let Some(people_array) =
                                cf_obj.get("people_value").and_then(|v| v.as_array())
                            {
                                for p_val in people_array {
                                    if let Some(p_obj) = p_val.as_object() {
                                        people_value.push(User {
                                            gid: p_obj
                                                .get("gid")
                                                .and_then(|v| v.as_str())
                                                .unwrap_or("")
                                                .to_string(),
                                            name: p_obj
                                                .get("name")
                                                .and_then(|v| v.as_str())
                                                .unwrap_or("")
                                                .to_string(),
                                            email: p_obj
                                                .get("email")
                                                .and_then(|v| v.as_str())
                                                .unwrap_or("")
                                                .to_string(),
                                        });
                                    }
                                }
                            }
                        }
                        _ => {}
                    }

                    custom_fields.push(CustomField {
                        gid,
                        name,
                        resource_subtype,
                        representation_type: None, // Not available in task data
                        id_prefix: None,           // Not available in task data
                        enum_options,
                        text_value,
                        number_value,
                        date_value,
                        enum_value,
                        multi_enum_values,
                        people_value,
                        enabled,
                    });
                }
            }
        }

        custom_fields
    }

    /// Get a single task with full details.
    ///
    pub async fn get_task(&mut self, task_gid: &str) -> Result<Task> {
        debug!("Fetching full details for task GID {}...", task_gid);

        // Use a simple model that doesn't require nested objects
        // The API returns nested objects as partial (gid + resource_type) unless we request specific fields
        // We'll fetch the basic task first, then get nested objects separately
        model!(TaskModelSimple "tasks" {
            name: String,
            completed: bool,
            notes: Option<String>,
            due_date: Option<String>,
            due_on: Option<String>,
            start_on: Option<String>,
            created_at: Option<String>,
            modified_at: Option<String>,
        });

        // Request task with assignee, section, tags, and custom fields as nested fields
        // For GET /tasks/{task_gid}, we pass opt_fields but NO other params (no project, workspace, etc.)
        // The API returns nested objects as partial (gid + resource_type) unless we request specific fields
        // IMPORTANT: Always include resource_type in opt_fields as it's required by the model
        let opt_fields = "resource_type,name,completed,notes,due_on,start_on,created_at,modified_at,assignee.name,assignee.email,memberships.section.name,tags.name,custom_fields.gid,custom_fields.name,custom_fields.resource_subtype,custom_fields.enum_options.gid,custom_fields.enum_options.name,custom_fields.enum_options.enabled,custom_fields.enum_options.color,custom_fields.text_value,custom_fields.number_value,custom_fields.date_value.date,custom_fields.enum_value.gid,custom_fields.enum_value.name,custom_fields.multi_enum_values.gid,custom_fields.multi_enum_values.name,custom_fields.people_value.gid,custom_fields.people_value.name,custom_fields.enabled";

        // Build URL manually to avoid client adding conflicting params
        let uri = format!("tasks/{}?opt_fields={}", task_gid, opt_fields);
        let request_url = format!("{}/{}", "https://app.asana.com/api/1.0", uri);

        let response = self
            .client
            .http_client
            .get(&request_url)
            .header(
                "Authorization",
                format!("Bearer {}", self.client.access_token),
            )
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let response_text = response
                .text()
                .await
                .unwrap_or_else(|_| String::from("Unable to read response"));
            anyhow::bail!(
                "API request failed with status {}: {}",
                status,
                response_text
            );
        }

        let model: Wrapper<TaskModelSimple> = response.json().await?;
        let task_data = model.data;

        // Extract assignee from extra fields
        let assignee = if let Some(assignee_val) = task_data.extra.get("assignee") {
            if let Some(assignee_obj) = assignee_val.as_object() {
                if let (Some(gid_val), Some(name_val), Some(email_val)) = (
                    assignee_obj.get("gid").and_then(|v| v.as_str()),
                    assignee_obj.get("name").and_then(|v| v.as_str()),
                    assignee_obj.get("email").and_then(|v| v.as_str()),
                ) {
                    Some(User {
                        gid: gid_val.to_string(),
                        name: name_val.to_string(),
                        email: email_val.to_string(),
                    })
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        // Extract section from memberships
        let section = if let Some(memberships) = task_data
            .extra
            .get("memberships")
            .and_then(|v| v.as_array())
            .and_then(|arr| arr.first())
            .and_then(|m| m.get("section"))
            .and_then(|s| s.as_object())
        {
            if let (Some(gid_val), Some(name_val)) = (
                memberships.get("gid").and_then(|v| v.as_str()),
                memberships.get("name").and_then(|v| v.as_str()),
            ) {
                Some(Section {
                    gid: gid_val.to_string(),
                    name: name_val.to_string(),
                })
            } else {
                None
            }
        } else {
            None
        };

        // Extract tags
        let tags = if let Some(tags_array) = task_data.extra.get("tags").and_then(|v| v.as_array())
        {
            tags_array
                .iter()
                .filter_map(|tag_val| {
                    if let Some(tag_obj) = tag_val.as_object() {
                        if let (Some(gid_val), Some(name_val)) = (
                            tag_obj.get("gid").and_then(|v| v.as_str()),
                            tag_obj.get("name").and_then(|v| v.as_str()),
                        ) {
                            Some(Tag {
                                gid: gid_val.to_string(),
                                name: name_val.to_string(),
                            })
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect()
        } else {
            vec![]
        };

        // Get subtasks count - for now, just return 0 to avoid API errors
        // TODO: Implement proper subtasks fetching
        let subtasks: Vec<String> = vec![];

        // Get stories/comments
        let stories = match self.get_task_stories(task_gid).await {
            Ok(s) => s,
            Err(e) => {
                warn!("Failed to fetch stories for task {}: {}", task_gid, e);
                vec![]
            }
        };

        // Extract custom fields
        let custom_fields = Self::parse_custom_fields_from_task_data(&task_data.extra);

        Ok(Task {
            gid: task_data.gid,
            name: task_data.name,
            completed: task_data.completed,
            notes: task_data.notes,
            assignee,
            due_date: task_data.due_date,
            due_on: task_data.due_on,
            start_on: task_data.start_on,
            section,
            tags,
            custom_fields,
            created_at: task_data.created_at,
            modified_at: task_data.modified_at,
            num_subtasks: subtasks.len(),
            num_comments: stories
                .iter()
                .filter(|s| match &s.resource_subtype {
                    Some(subtype) => subtype == "comment_added",
                    None => s.created_by.is_some(),
                })
                .count(),
        })
    }

    /// Update a task (e.g., mark as complete/incomplete).
    ///
    pub async fn update_task(&mut self, task_gid: &str, completed: Option<bool>) -> Result<Task> {
        debug!("Updating task GID {}...", task_gid);

        model!(TaskModel "tasks" { name: String, completed: bool });

        let body = if let Some(completed) = completed {
            serde_json::json!({
                "data": {
                    "completed": completed
                }
            })
        } else {
            serde_json::json!({})
        };

        let model: Wrapper<TaskModel> = self
            .client
            .call_with_body::<TaskModel>(reqwest::Method::PUT, Some(task_gid), None, Some(body))
            .await?
            .json()
            .await?;

        Ok(Task {
            gid: model.data.gid,
            name: model.data.name,
            completed: model.data.completed,
            notes: None,
            assignee: None,
            due_date: None,
            due_on: None,
            start_on: None,
            section: None,
            tags: vec![],
            custom_fields: vec![],
            created_at: None,
            modified_at: None,
            num_subtasks: 0,
            num_comments: 0,
        })
    }

    /// Get all sections for a project (for kanban board).
    ///
    pub async fn get_project_sections(&mut self, project_gid: &str) -> Result<Vec<Section>> {
        debug!("Fetching sections for project GID {}...", project_gid);

        model!(ProjectModel "projects" { name: String });
        model!(SectionModel "sections" { name: String });

        // Use the relational endpoint: GET /projects/{project_gid}/sections
        let data: Vec<SectionModel> = self
            .client
            .from::<ProjectModel>(project_gid)
            .list_paginated::<SectionModel>(None, Some(100))
            .await?;

        Ok(data
            .into_iter()
            .map(|s| Section {
                gid: s.gid,
                name: s.name,
            })
            .collect())
    }

    /// Get custom fields for a project.
    ///
    pub async fn get_project_custom_fields(
        &mut self,
        project_gid: &str,
    ) -> Result<Vec<CustomField>> {
        debug!("Fetching custom fields for project GID {}...", project_gid);

        // Use GET /projects/{project_gid}/custom_field_settings
        // Then fetch each custom field definition
        let uri = format!("projects/{}/custom_field_settings?opt_fields=custom_field.gid,custom_field.name,custom_field.resource_subtype,custom_field.representation_type,custom_field.id_prefix,custom_field.enum_options.gid,custom_field.enum_options.name,custom_field.enum_options.enabled,custom_field.enum_options.color", project_gid);
        let request_url = format!("{}/{}", "https://app.asana.com/api/1.0", uri);

        let response = self
            .client
            .http_client
            .get(&request_url)
            .header(
                "Authorization",
                format!("Bearer {}", self.client.access_token),
            )
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let response_text = response
                .text()
                .await
                .unwrap_or_else(|_| String::from("Unable to read response"));
            warn!(
                "Failed to fetch custom fields: {} - {}",
                status, response_text
            );
            return Ok(vec![]); // Return empty vec on error (custom fields are optional)
        }

        let json: serde_json::Value = response.json().await?;
        let mut custom_fields = Vec::new();

        if let Some(data_array) = json.get("data").and_then(|v| v.as_array()) {
            for item in data_array {
                if let Some(cf_obj) = item.get("custom_field").and_then(|v| v.as_object()) {
                    let gid = cf_obj
                        .get("gid")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let name = cf_obj
                        .get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let resource_subtype = cf_obj
                        .get("resource_subtype")
                        .and_then(|v| v.as_str())
                        .unwrap_or("text")
                        .to_string();
                    let representation_type = cf_obj
                        .get("representation_type")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());
                    let id_prefix = cf_obj
                        .get("id_prefix")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());

                    // Parse enum options
                    let mut enum_options = Vec::new();
                    if let Some(enum_opts_array) =
                        cf_obj.get("enum_options").and_then(|v| v.as_array())
                    {
                        for eo_val in enum_opts_array {
                            if let Some(eo_obj) = eo_val.as_object() {
                                enum_options.push(EnumOption {
                                    gid: eo_obj
                                        .get("gid")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("")
                                        .to_string(),
                                    name: eo_obj
                                        .get("name")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("")
                                        .to_string(),
                                    enabled: eo_obj
                                        .get("enabled")
                                        .and_then(|v| v.as_bool())
                                        .unwrap_or(true),
                                    color: eo_obj
                                        .get("color")
                                        .and_then(|v| v.as_str())
                                        .map(|s| s.to_string()),
                                });
                            }
                        }
                    }

                    custom_fields.push(CustomField {
                        gid,
                        name,
                        resource_subtype,
                        representation_type,
                        id_prefix,
                        enum_options,
                        text_value: None,
                        number_value: None,
                        date_value: None,
                        enum_value: None,
                        multi_enum_values: vec![],
                        people_value: vec![],
                        enabled: true,
                    });
                }
            }
        }

        Ok(custom_fields)
    }

    /// Get all stories/comments for a task.
    ///
    pub async fn get_task_stories(&mut self, task_gid: &str) -> Result<Vec<Story>> {
        debug!("Fetching stories/comments for task GID {}...", task_gid);

        // In story responses, created_by only has gid and resource_type (no name/email)
        model!(UserModel "users" { name: Option<String>, email: Option<String> });
        model!(StoryModel "stories" {
            text: Option<String>,  // Some story types (reminders, reactions) don't have text
            created_at: Option<String>,
            created_by: Option<UserModel>,
            resource_subtype: Option<String>,
        } UserModel);

        // Use relational endpoint: GET /tasks/{task_gid}/stories
        let response = self
            .client
            .http_client
            .get(format!(
                "{}/tasks/{}/stories",
                &self.client.base_url, task_gid
            ))
            .bearer_auth(&self.client.access_token)
            .send()
            .await?;

        // Log the raw response
        let response_text = response.text().await?;
        debug!("Stories API response: {}", &response_text);

        // Parse the response
        let wrapper: Wrapper<Vec<StoryModel>> =
            serde_json::from_str(&response_text).map_err(|e| {
                error!(
                    "Failed to deserialize stories response: {}. Response body: {}",
                    e, &response_text
                );
                e
            })?;

        Ok(wrapper
            .data
            .into_iter()
            .filter_map(|s| {
                // Skip stories without text (reminders, reactions, etc.)
                let text = s.text?;

                debug!(
                    "Story: gid={}, created_by={:?}, text={}",
                    &s.gid, &s.created_by, &text
                );

                Some(Story {
                    gid: s.gid.clone(),
                    text,
                    created_at: s.created_at.clone(),
                    created_by: s.created_by.map(|u| {
                        debug!(
                            "  User: gid={}, name={:?}, email={:?}",
                            &u.gid, &u.name, &u.email
                        );
                        User {
                            gid: u.gid,
                            name: u.name.unwrap_or_else(|| "Unknown User".to_string()),
                            email: u.email.unwrap_or_default(),
                        }
                    }),
                    resource_subtype: s.resource_subtype.clone(),
                })
            })
            .collect())
    }

    /// Create a story/comment on a task.
    ///
    pub async fn create_story(&mut self, task_gid: &str, text: &str) -> Result<Story> {
        debug!("Creating story/comment on task GID {}...", task_gid);

        // In story responses, created_by only has gid and resource_type (no name/email)
        model!(UserModel "users" { name: Option<String>, email: Option<String> });
        model!(StoryModel "stories" {
            text: String,
            created_at: Option<String>,
            created_by: Option<UserModel>,
        } UserModel);
        model!(TaskModel "tasks" { name: String });

        let body = serde_json::json!({
            "data": {
                "text": text
            }
        });

        // Post to /tasks/{task_gid}/stories
        let response = self
            .client
            .from::<TaskModel>(task_gid)
            .call_with_body::<StoryModel>(reqwest::Method::POST, None, None, Some(body))
            .await?;

        // Story creation returns { "data": { ... } } wrapper
        let model: Wrapper<StoryModel> = response.json().await?;

        Ok(Story {
            gid: model.data.gid,
            text: model.data.text,
            created_at: model.data.created_at,
            created_by: model.data.created_by.map(|u| User {
                gid: u.gid,
                name: u.name.unwrap_or_else(|| "Unknown User".to_string()),
                email: u.email.unwrap_or_default(),
            }),
            resource_subtype: None, // Not included in create response
        })
    }

    /// Get all users in a workspace.
    ///
    pub async fn get_workspace_users(&mut self, workspace_gid: &str) -> Result<Vec<User>> {
        debug!("Fetching users for workspace GID {}...", workspace_gid);

        model!(UserModel "users" { name: String, email: String });

        let data: Vec<UserModel> = self
            .client
            .list_paginated::<UserModel>(Some(vec![("workspace", workspace_gid)]), Some(100))
            .await?;

        Ok(data
            .into_iter()
            .map(|u| User {
                gid: u.gid,
                name: u.name,
                email: u.email,
            })
            .collect())
    }

    /// Create a new task.
    ///
    pub async fn create_task(
        &mut self,
        project_gid: &str,
        name: &str,
        notes: Option<&str>,
        assignee: Option<&str>,
        due_on: Option<&str>,
        section: Option<&str>,
        custom_fields: &std::collections::HashMap<String, crate::state::CustomFieldValue>,
    ) -> Result<Task> {
        debug!("Creating new task in project GID {}...", project_gid);

        model!(TaskModel "tasks" { name: String, completed: bool });

        let mut data = serde_json::json!({
            "name": name,
            "projects": [project_gid]
        });

        if let Some(notes_val) = notes {
            data["notes"] = serde_json::Value::String(notes_val.to_string());
        }
        if let Some(assignee_val) = assignee {
            data["assignee"] = serde_json::Value::String(assignee_val.to_string());
        }
        if let Some(due_on_val) = due_on {
            data["due_on"] = serde_json::Value::String(due_on_val.to_string());
        }

        // Add custom fields - validate GIDs and skip invalid ones
        if !custom_fields.is_empty() {
            // Get valid custom field GIDs for this project
            // Get full custom field definitions (not just GIDs) so we can validate enum options
            let project_custom_fields: Vec<CustomField> =
                match self.get_project_custom_fields(project_gid).await {
                    Ok(cfs) => {
                        // Filter out custom_id fields - they cannot be manually set
                        cfs.iter()
                            .filter(|cf| {
                                let is_custom_id = cf
                                    .representation_type
                                    .as_ref()
                                    .map(|s| s == "custom_id")
                                    .unwrap_or(false)
                                    || cf.id_prefix.is_some();
                                !is_custom_id
                            })
                            .cloned()
                            .collect()
                    }
                    Err(e) => {
                        warn!(
                            "Failed to fetch project custom fields for validation: {}",
                            e
                        );
                        Vec::new()
                    }
                };

            let valid_cf_gids: std::collections::HashSet<String> = project_custom_fields
                .iter()
                .map(|cf| cf.gid.clone())
                .collect();

            // Build a map of custom field GID -> enum option GIDs for validation
            let mut cf_enum_options: std::collections::HashMap<
                String,
                std::collections::HashSet<String>,
            > = std::collections::HashMap::new();
            for cf in &project_custom_fields {
                if cf.resource_subtype == "enum" || cf.resource_subtype == "multi_enum" {
                    let enum_gids: std::collections::HashSet<String> =
                        cf.enum_options.iter().map(|eo| eo.gid.clone()).collect();
                    cf_enum_options.insert(cf.gid.clone(), enum_gids);
                }
            }

            info!("Valid custom field GIDs for project: {:?}", valid_cf_gids);
            // For CREATE, custom_fields must be an object/map, NOT an array
            // Format: { "custom_field_gid": value } where value is:
            //   - enum: just the enum GID string
            //   - text: just the text string
            //   - number: just the number
            //   - date: date string (YYYY-MM-DD)
            //   - multi_enum: array of enum GID strings
            //   - people: array of user GID strings
            let mut custom_fields_obj = serde_json::Map::new();
            for (gid, value) in custom_fields {
                info!("Checking custom field: gid={}, value={:?}", gid, value);
                // Skip invalid GIDs (empty, "0", or not in project's custom fields)
                if gid.is_empty() || gid == "0" {
                    warn!("Skipping invalid custom field GID (empty or '0'): {}", gid);
                    continue;
                }
                if !valid_cf_gids.contains(gid.as_str()) {
                    warn!(
                        "Skipping custom field GID not in project: {} (valid GIDs: {:?})",
                        gid, valid_cf_gids
                    );
                    info!(
                        "Custom field {} is NOT in the project's custom fields list",
                        gid
                    );
                    continue;
                }
                info!("Custom field {} is valid, processing...", gid);

                info!(
                    "Processing custom field for create: gid={}, value={:?}",
                    gid, value
                );

                // For CREATE, format is: { "custom_field_gid": direct_value }
                let cf_value: Option<serde_json::Value> = match value {
                    crate::state::CustomFieldValue::Text(s) if !s.trim().is_empty() => {
                        Some(serde_json::Value::String(s.trim().to_string()))
                    }
                    crate::state::CustomFieldValue::Number(Some(n)) => {
                        if let Some(num) = serde_json::Number::from_f64(*n) {
                            Some(serde_json::Value::Number(num))
                        } else {
                            warn!("Invalid number value for custom field {}, skipping", gid);
                            None
                        }
                    }
                    crate::state::CustomFieldValue::Date(Some(d)) if !d.trim().is_empty() => {
                        // For date, use format "YYYY-MM-DD" as a string
                        Some(serde_json::Value::String(d.trim().to_string()))
                    }
                    crate::state::CustomFieldValue::Enum(Some(enum_gid)) => {
                        // Validate enum option GID - for create, send just the enum GID string directly
                        if enum_gid.is_empty() || enum_gid == "0" {
                            warn!(
                                "Skipping invalid enum option GID for custom field {}: {}",
                                gid, enum_gid
                            );
                            None
                        } else {
                            // Check if this enum option is valid for this custom field
                            if let Some(valid_enum_gids) = cf_enum_options.get(gid) {
                                if !valid_enum_gids.contains(enum_gid.as_str()) {
                                    warn!(
                                        "Skipping invalid enum option GID '{}' for custom field {} (valid options: {:?})",
                                        enum_gid, gid, valid_enum_gids
                                    );
                                    None
                                } else {
                                    info!(
                                        "Adding enum custom field: gid={}, enum_gid={}",
                                        gid, enum_gid
                                    );
                                    // For create, enum value is just the enum GID string
                                    Some(serde_json::Value::String(enum_gid.clone()))
                                }
                            } else {
                                // Custom field doesn't have enum options, but we're trying to set one
                                warn!(
                                    "Custom field {} is not an enum field or has no enum options",
                                    gid
                                );
                                None
                            }
                        }
                    }
                    crate::state::CustomFieldValue::MultiEnum(gids) if !gids.is_empty() => {
                        // Filter out invalid enum option GIDs and validate against custom field's enum options
                        let valid_enum_gids = cf_enum_options.get(gid).cloned().unwrap_or_default();
                        let valid_gids: Vec<_> = gids
                            .iter()
                            .filter(|enum_gid| {
                                !enum_gid.is_empty()
                                    && *enum_gid != "0"
                                    && valid_enum_gids.contains(enum_gid.as_str())
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
                            info!(
                                "Adding multi-enum custom field: gid={}, enum_gids={:?}",
                                gid, valid_gids
                            );
                            // For create, multi-enum is an array of enum GID strings
                            Some(serde_json::Value::Array(
                                valid_gids
                                    .iter()
                                    .map(|gid| serde_json::Value::String(gid.clone()))
                                    .collect(),
                            ))
                        }
                    }
                    crate::state::CustomFieldValue::People(gids) if !gids.is_empty() => {
                        // Filter out invalid people GIDs
                        let valid_gids: Vec<_> = gids
                            .iter()
                            .filter(|gid| !gid.is_empty() && *gid != "0")
                            .cloned()
                            .collect();
                        if valid_gids.is_empty() {
                            warn!("Skipping people field with no valid user GIDs");
                            None
                        } else {
                            // For create, people is an array of user GID strings
                            Some(serde_json::Value::Array(
                                valid_gids
                                    .iter()
                                    .map(|gid| serde_json::Value::String(gid.clone()))
                                    .collect(),
                            ))
                        }
                    }
                    _ => {
                        // Skip empty values (Text(""), Enum(None), Number(None), Date(None), empty arrays)
                        None
                    }
                };

                // Only add if we have valid data
                if let Some(cf_val) = cf_value {
                    info!(
                        "Adding custom field to object: gid={}, value={:?}",
                        gid, cf_val
                    );
                    let mut should_skip = false;

                    // For create, we're using direct values, so check if enum value is a string and validate it
                    if let Some(enum_gid_str) = cf_val.as_str() {
                        // This is an enum field - validate the enum GID
                        if enum_gid_str == "0" || enum_gid_str.is_empty() {
                            error!("ERROR: Attempted to add custom field with invalid enum GID '{}' - skipping!", enum_gid_str);
                            should_skip = true;
                        }
                    }

                    // Check if it's an array (multi_enum or people) and validate GIDs
                    if !should_skip {
                        if let Some(arr) = cf_val.as_array() {
                            for item in arr {
                                if let Some(gid_str) = item.as_str() {
                                    if gid_str == "0" || gid_str.is_empty() {
                                        error!("ERROR: Attempted to add custom field with invalid GID '{}' in array - skipping entire field!", gid_str);
                                        should_skip = true;
                                        break;
                                    }
                                }
                            }
                        }
                    }

                    if !should_skip {
                        custom_fields_obj.insert(gid.clone(), cf_val);
                    }
                }
            }
            if !custom_fields_obj.is_empty() {
                info!(
                    "Adding {} custom fields to create task request (as object)",
                    custom_fields_obj.len()
                );
                info!("Custom fields object: {:?}", custom_fields_obj);
                data["custom_fields"] = serde_json::Value::Object(custom_fields_obj);
            } else {
                info!("No valid custom fields to add to create task request");
            }
        }

        let body = serde_json::json!({
            "data": data
        });

        info!(
            "Create task request body: {}",
            serde_json::to_string_pretty(&body).unwrap_or_default()
        );
        let response = self
            .client
            .call_with_body::<TaskModel>(reqwest::Method::POST, None, None, Some(body))
            .await?;

        // Check response status before trying to deserialize
        if !response.status().is_success() {
            let error_text = response.text().await?;
            error!("Failed to create task: {}", error_text);
            anyhow::bail!("Failed to create task: {}", error_text);
        }

        let model: Wrapper<TaskModel> = response.json().await?;

        // If section is specified, add task to that section
        if let Some(section_gid) = section {
            self.add_task_to_section(&model.data.gid, section_gid)
                .await?;
        }

        Ok(Task {
            gid: model.data.gid,
            name: model.data.name,
            completed: model.data.completed,
            notes: None,
            assignee: None,
            due_date: None,
            due_on: None,
            start_on: None,
            section: None,
            tags: vec![],
            custom_fields: vec![],
            created_at: None,
            modified_at: None,
            num_subtasks: 0,
            num_comments: 0,
        })
    }

    /// Update task fields.
    ///
    pub async fn update_task_fields(
        &mut self,
        task_gid: &str,
        name: Option<&str>,
        notes: Option<&str>,
        assignee: Option<&str>,
        due_on: Option<&str>,
        section: Option<&str>,
        completed: Option<bool>,
        custom_fields: &std::collections::HashMap<String, crate::state::CustomFieldValue>,
    ) -> Result<Task> {
        debug!("Updating task fields for GID {}...", task_gid);

        // Include all fields we might send in the update
        // Note: section is NOT included here because it's handled via a separate endpoint
        model!(TaskModel "tasks" {
            name: String,
            completed: bool,
            notes: Option<String>,
            assignee: Option<String>,
            due_on: Option<String>,
        });

        let mut data = serde_json::json!({});

        // Only add fields that are Some and non-empty (after trimming)
        // This ensures we never send empty strings to the API
        if let Some(name_val) = name {
            let trimmed = name_val.trim();
            if !trimmed.is_empty() {
                data["name"] = serde_json::Value::String(trimmed.to_string());
            } else {
                warn!("Skipping empty name field");
            }
        }
        if let Some(notes_val) = notes {
            let trimmed = notes_val.trim();
            if !trimmed.is_empty() {
                data["notes"] = serde_json::Value::String(trimmed.to_string());
            } else {
                warn!(
                    "Skipping empty notes field (original length: {})",
                    notes_val.len()
                );
            }
        }
        if let Some(assignee_val) = assignee {
            let trimmed = assignee_val.trim();
            if !trimmed.is_empty() {
                data["assignee"] = serde_json::Value::String(trimmed.to_string());
            } else {
                warn!("Skipping empty assignee field");
            }
        }
        if let Some(due_on_val) = due_on {
            let trimmed = due_on_val.trim();
            if !trimmed.is_empty() {
                data["due_on"] = serde_json::Value::String(trimmed.to_string());
            } else {
                warn!("Skipping empty due_on field");
            }
        }
        if let Some(completed_val) = completed {
            data["completed"] = serde_json::Value::Bool(completed_val);
        }

        // Add custom fields - validate GIDs and skip invalid ones
        // NOTE: For updates, custom_fields must be an object/map, NOT an array
        // Format: { "custom_field_gid": value } where value is:
        //   - enum: just the enum GID string
        //   - text: just the text string
        //   - number: just the number
        //   - date: date string (YYYY-MM-DD)
        //   - multi_enum: array of enum GID strings
        //   - people: array of user GID strings
        if !custom_fields.is_empty() {
            // Check each custom field to see if it's a custom_id field (which cannot be manually updated)
            let mut custom_id_gids = std::collections::HashSet::new();
            for gid in custom_fields.keys() {
                // Fetch the custom field definition to check if it's a custom_id field
                let cf_uri = format!(
                    "custom_fields/{}?opt_fields=representation_type,id_prefix",
                    gid
                );
                let cf_url = format!("{}/{}", "https://app.asana.com/api/1.0", cf_uri);
                let cf_response = self
                    .client
                    .http_client
                    .get(&cf_url)
                    .header(
                        "Authorization",
                        format!("Bearer {}", self.client.access_token),
                    )
                    .send()
                    .await;

                if let Ok(resp) = cf_response {
                    if resp.status().is_success() {
                        if let Ok(json) = resp.json::<serde_json::Value>().await {
                            // Check if this is a custom_id field
                            let is_custom_id = json
                                .get("data")
                                .map(|d| {
                                    // Check representation_type == "custom_id" or if id_prefix is not null
                                    let rep_type = d
                                        .get("representation_type")
                                        .and_then(|v| v.as_str())
                                        .map(|s| s == "custom_id")
                                        .unwrap_or(false);
                                    let has_id_prefix =
                                        d.get("id_prefix").map(|v| !v.is_null()).unwrap_or(false);
                                    rep_type || has_id_prefix
                                })
                                .unwrap_or(false);

                            if is_custom_id {
                                warn!(
                                    "Skipping custom_id field {} - cannot be manually updated",
                                    gid
                                );
                                custom_id_gids.insert(gid.clone());
                            }
                        }
                    }
                }
            }

            // For updates, we can't easily validate against project fields, so we'll just check for obviously invalid GIDs
            let mut custom_fields_obj = serde_json::Map::new();
            for (gid, value) in custom_fields {
                // Skip custom_id fields (they cannot be manually updated)
                if custom_id_gids.contains(gid) {
                    continue;
                }
                // Skip invalid GIDs (empty or "0")
                if gid.is_empty() || gid == "0" {
                    warn!("Skipping invalid custom field GID: {}", gid);
                    info!(
                        "Skipped custom field with GID '{}' because it's empty or '0'",
                        gid
                    );
                    continue;
                }
                info!("Processing custom field with GID: {}", gid);

                // For updates, format is: { "custom_field_gid": direct_value }
                let cf_value: Option<serde_json::Value> = match value {
                    crate::state::CustomFieldValue::Text(s) if !s.trim().is_empty() => {
                        Some(serde_json::Value::String(s.trim().to_string()))
                    }
                    crate::state::CustomFieldValue::Number(Some(n)) => {
                        if let Some(num) = serde_json::Number::from_f64(*n) {
                            Some(serde_json::Value::Number(num))
                        } else {
                            warn!("Invalid number value for custom field {}, skipping", gid);
                            None
                        }
                    }
                    crate::state::CustomFieldValue::Date(Some(d)) if !d.trim().is_empty() => {
                        // For date, use format "YYYY-MM-DD" as a string
                        Some(serde_json::Value::String(d.trim().to_string()))
                    }
                    crate::state::CustomFieldValue::Enum(Some(enum_gid)) => {
                        // Validate enum option GID - for updates, send just the enum GID string directly
                        if enum_gid.is_empty() || enum_gid == "0" {
                            warn!(
                                "Skipping invalid enum option GID for custom field {}: {}",
                                gid, enum_gid
                            );
                            None
                        } else {
                            info!(
                                "Adding enum custom field: gid={}, enum_gid={}",
                                gid, enum_gid
                            );
                            // For updates, enum value is just the enum GID string
                            Some(serde_json::Value::String(enum_gid.clone()))
                        }
                    }
                    crate::state::CustomFieldValue::MultiEnum(gids) if !gids.is_empty() => {
                        // Filter out invalid enum option GIDs
                        let valid_gids: Vec<_> = gids
                            .iter()
                            .filter(|gid| !gid.is_empty() && *gid != "0")
                            .cloned()
                            .collect();
                        if valid_gids.is_empty() {
                            warn!("Skipping multi-enum with no valid option GIDs");
                            None
                        } else {
                            // For updates, multi-enum is an array of enum GID strings
                            Some(serde_json::Value::Array(
                                valid_gids
                                    .iter()
                                    .map(|gid| serde_json::Value::String(gid.clone()))
                                    .collect(),
                            ))
                        }
                    }
                    crate::state::CustomFieldValue::People(gids) if !gids.is_empty() => {
                        // Filter out invalid people GIDs
                        let valid_gids: Vec<_> = gids
                            .iter()
                            .filter(|gid| !gid.is_empty() && *gid != "0")
                            .cloned()
                            .collect();
                        if valid_gids.is_empty() {
                            warn!("Skipping people field with no valid user GIDs");
                            None
                        } else {
                            // For updates, people is an array of user GID strings
                            Some(serde_json::Value::Array(
                                valid_gids
                                    .iter()
                                    .map(|gid| serde_json::Value::String(gid.clone()))
                                    .collect(),
                            ))
                        }
                    }
                    _ => {
                        // Skip empty values (Text(""), Enum(None), Number(None), Date(None), empty arrays)
                        None
                    }
                };

                // Only add if we have valid data
                if let Some(cf_val) = cf_value {
                    info!(
                        "Adding custom field to object: gid={}, value={:?}",
                        gid, cf_val
                    );
                    custom_fields_obj.insert(gid.clone(), cf_val);
                }
            }
            if !custom_fields_obj.is_empty() {
                info!(
                    "Adding {} custom fields to update task request (as object)",
                    custom_fields_obj.len()
                );
                info!("Custom fields object: {:?}", custom_fields_obj);
                data["custom_fields"] = serde_json::Value::Object(custom_fields_obj);
            } else {
                info!("No valid custom fields to add to update task request");
            }
        }

        // If only section changed, skip the PUT request and just move the section
        let has_section_only = section.is_some()
            && name.is_none()
            && notes.is_none()
            && assignee.is_none()
            && due_on.is_none()
            && completed.is_none()
            && custom_fields.is_empty();

        if has_section_only {
            // Just move the section, no PUT request needed
            if let Some(section_gid) = section {
                self.add_task_to_section(task_gid, section_gid).await?;
            }
            return self.get_task(task_gid).await;
        }

        // Ensure we have at least one field to update
        if let Some(obj) = data.as_object() {
            if obj.is_empty() {
                warn!("⚠️ No fields to update, skipping API call");
                // If section is specified, handle it separately
                if let Some(section_gid) = section {
                    self.add_task_to_section(task_gid, section_gid).await?;
                }
                // Return the current task without making an API call
                return self.get_task(task_gid).await;
            }
        }

        // Final validation: ensure no empty string values in data
        if let Some(obj) = data.as_object_mut() {
            let mut removed_fields = Vec::new();
            obj.retain(|key, value| match value {
                serde_json::Value::String(s) => {
                    if s.trim().is_empty() {
                        removed_fields.push(key.clone());
                        error!("⚠️ REMOVING EMPTY STRING FIELD: {} (value: '{:?}')", key, s);
                        false
                    } else {
                        true
                    }
                }
                serde_json::Value::Null => {
                    removed_fields.push(key.clone());
                    error!("⚠️ REMOVING NULL FIELD: {}", key);
                    false
                }
                _ => true,
            });
            if !removed_fields.is_empty() {
                error!(
                    "❌ Removed {} empty/null fields: {:?}",
                    removed_fields.len(),
                    removed_fields
                );
            }
        }

        let body = serde_json::json!({
            "data": data
        });

        info!(
            "Update task request body: {}",
            serde_json::to_string_pretty(&body).unwrap_or_default()
        );
        let response = self
            .client
            .call_with_body::<TaskModel>(reqwest::Method::PUT, Some(task_gid), None, Some(body))
            .await?;

        // Check response status
        if !response.status().is_success() {
            let error_text = response.text().await?;
            anyhow::bail!("Failed to update task: {}", error_text);
        }

        // If section is specified, move task to that section
        if let Some(section_gid) = section {
            self.add_task_to_section(task_gid, section_gid).await?;
        }

        // Return updated task by fetching it again (ignore the PUT response)
        self.get_task(task_gid).await
    }

    /// Add a task to a section (move task in kanban).
    ///
    pub async fn add_task_to_section(&mut self, task_gid: &str, section_gid: &str) -> Result<()> {
        debug!("Moving task {} to section {}...", task_gid, section_gid);

        // Get the project GID from the task's memberships

        // Get the project GID from the task's memberships
        // We need to fetch the task with memberships.project included
        let opt_fields = "resource_type,memberships.project.gid,memberships.section.gid";
        let uri = format!("tasks/{}?opt_fields={}", task_gid, opt_fields);
        let request_url = format!("{}/{}", self.client.base_url, uri);

        let response = self
            .client
            .http_client
            .get(&request_url)
            .header(
                "Authorization",
                format!("Bearer {}", self.client.access_token),
            )
            .send()
            .await?;

        let task_json: serde_json::Value = response.json().await?;
        let task_data = task_json
            .get("data")
            .ok_or_else(|| anyhow::anyhow!("No data in response"))?;

        // Extract project GID from memberships
        let project_gid = task_data
            .get("memberships")
            .and_then(|m| m.as_array())
            .and_then(|arr| arr.first())
            .and_then(|m| m.get("project"))
            .and_then(|p| p.as_object())
            .and_then(|p| p.get("gid"))
            .and_then(|g| g.as_str());

        let current_section_gid = task_data
            .get("memberships")
            .and_then(|m| m.as_array())
            .and_then(|arr| arr.first())
            .and_then(|m| m.get("section"))
            .and_then(|s| s.as_object())
            .and_then(|s| s.get("gid"))
            .and_then(|g| g.as_str());

        info!(
            "Current section: {:?}, Target section: {}",
            current_section_gid, section_gid
        );

        // If the task is already in the target section, no need to move
        if current_section_gid == Some(section_gid) {
            info!("Task is already in the target section, no move needed");
            return Ok(());
        }

        // Use POST /tasks/{task_gid}/addProject with section specified
        // This is the recommended way to move a task to a different section
        // According to Asana API docs, this will move the task to the new section
        let project_gid = project_gid.ok_or_else(|| anyhow::anyhow!("Task is not in a project"))?;

        let body = serde_json::json!({
            "data": {
                "project": project_gid,
                "section": section_gid
            }
        });

        let url = format!("{}/tasks/{}/addProject", self.client.base_url, task_gid);

        let response = self
            .client
            .http_client
            .post(&url)
            .header(
                "Authorization",
                format!("Bearer {}", self.client.access_token),
            )
            .json(&body)
            .send()
            .await?;

        let status = response.status();

        if !status.is_success() {
            let error_text = response.text().await?;
            error!("Failed to move task to section: {}", error_text);
            anyhow::bail!("Failed to move task to section: {}", error_text);
        }

        Ok(())
    }

    /// Delete a task.
    ///
    pub async fn delete_task(&mut self, task_gid: &str) -> Result<()> {
        debug!("Deleting task GID {}...", task_gid);

        model!(TaskModel "tasks" { name: String, completed: bool });

        self.client
            .call_with_body::<TaskModel>(reqwest::Method::DELETE, Some(task_gid), None, None)
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fake::uuid::UUIDv4;
    use fake::{Fake, Faker};
    use httpmock::MockServer;
    use serde_json::json;
    use uuid::Uuid;

    #[tokio::test]
    async fn me_success() -> Result<()> {
        let token: Uuid = UUIDv4.fake();
        let user: User = Faker.fake();
        let workspaces: [Workspace; 2] = [Faker.fake(), Faker.fake()];

        let server = MockServer::start();
        let mock = server.mock_async(|when, then| {
            when.method("GET")
                .path("/users/me")
                .header("Authorization", &format!("Bearer {}", &token));
            then.status(200).json_body(json!({
                "data": {
                    "gid": user.gid,
                    "name": user.name,
                    "email": user.email,
                    "resource_type": "user",
                    "workspaces": [
                        { "gid": workspaces[0].gid, "resource_type": "workspace", "name": workspaces[0].name },
                        { "gid": workspaces[1].gid, "resource_type": "workspace", "name": workspaces[1].name },
                    ]
                }
            }));
        }).await;

        let mut asana = Asana {
            client: Client::new(&token.to_string(), &server.base_url()),
        };
        asana.me().await?;
        mock.assert_async().await;
        Ok(())
    }

    #[tokio::test]
    async fn me_unauthorized() {
        let server = MockServer::start();
        let mock = server
            .mock_async(|when, then| {
                when.method("GET").path("/users/me");
                then.status(401);
            })
            .await;

        let mut asana = Asana {
            client: Client::new("", &server.base_url()),
        };
        assert!(asana.me().await.is_err());
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn projects_success() -> Result<()> {
        let token: Uuid = UUIDv4.fake_with_rng(&mut rand::thread_rng());
        let workspace: Workspace = Faker.fake();
        let projects: [Project; 2] = Faker.fake();

        let server = MockServer::start();
        let mock = server
            .mock_async(|when, then| {
                when.method("GET")
                    .path("/projects/")
                    .header("Authorization", &format!("Bearer {}", &token))
                    .query_param("workspace", &workspace.gid);
                then.status(200).json_body(json!({
                    "data": [
                        {
                            "gid": projects[0].gid,
                            "resource_type": "task",
                            "name": projects[0].name,
                        },
                        {
                            "gid": projects[1].gid,
                            "resource_type": "task",
                            "name": projects[1].name,
                        }
                    ]
                }));
            })
            .await;

        let mut asana = Asana {
            client: Client::new(&token.to_string(), &server.base_url()),
        };
        asana.projects(&workspace.gid).await?;
        mock.assert_async().await;
        Ok(())
    }

    #[tokio::test]
    async fn tasks_success() -> Result<()> {
        let token: Uuid = UUIDv4.fake();
        let project: Project = Faker.fake();
        let tasks: [Task; 2] = Faker.fake();

        let server = MockServer::start();
        let mock = server
            .mock_async(|when, then| {
                when.method("GET")
                    .path("/tasks/")
                    .header("Authorization", &format!("Bearer {}", &token))
                    .query_param("project", &project.gid);
                then.status(200).json_body(json!({
                    "data": [
                        {
                            "gid": tasks[0].gid,
                            "resource_type": "task",
                            "name": tasks[0].name,
                        },
                        {
                            "gid": tasks[1].gid,
                            "resource_type": "task",
                            "name": tasks[1].name,
                        }
                    ]
                }));
            })
            .await;

        let mut asana = Asana {
            client: Client::new(&token.to_string(), &server.base_url()),
        };
        asana.tasks(&project.gid, None, false).await?;
        mock.assert_async().await;
        Ok(())
    }

    #[tokio::test]
    async fn my_tasks_success() -> Result<()> {
        let token: Uuid = UUIDv4.fake();
        let user: User = Faker.fake();
        let workspace: Workspace = Faker.fake();
        let tasks: [Task; 2] = Faker.fake();

        let server = MockServer::start();
        let mock = server
            .mock_async(|when, then| {
                when.method("GET")
                    .path("/tasks/")
                    .header("Authorization", &format!("Bearer {}", &token))
                    .query_param("assignee", &user.gid)
                    .query_param("workspace", &workspace.gid)
                    .query_param_exists("completed_since");
                then.status(200).json_body(json!({
                    "data": [
                        {
                            "gid": tasks[0].gid,
                            "resource_type": "task",
                            "name": tasks[0].name,
                        },
                        {
                            "gid": tasks[1].gid,
                            "resource_type": "task",
                            "name": tasks[1].name,
                        }
                    ]
                }));
            })
            .await;

        let mut asana = Asana {
            client: Client::new(&token.to_string(), &server.base_url()),
        };
        asana.my_tasks(&user.gid, &workspace.gid).await?;
        mock.assert_async().await;
        Ok(())
    }
}
