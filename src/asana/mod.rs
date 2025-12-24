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
            .map(|t| Task {
                gid: t.gid,
                name: t.name,
                completed: t.completed,
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
            })
            .collect())
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
        })
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
        let token: Uuid = UUIDv4.fake();
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
