use sqlx::{query_as, PgPool};

use crate::error::AppError;
use crate::models::{CreateWorkspace, User, Workspace};

impl Workspace {
    pub(crate) async fn create(
        create_ws: CreateWorkspace,
        pool: &PgPool,
    ) -> Result<Self, AppError> {
        let ws = query_as(
            r#"
            INSERT INTO workspaces (name, owner_id)
            VALUES ($1, $2)
            RETURNING *
            "#,
        )
        .bind(create_ws.name)
        .bind(create_ws.owner_id)
        .fetch_one(pool)
        .await?;

        Ok(ws)
    }

    pub(crate) async fn find_workspace_by_name(
        name: &str,
        pool: &PgPool,
    ) -> Result<Option<Self>, AppError> {
        let workspace: Option<Workspace> = query_as(
            r#"
            SELECT *
            FROM workspaces
            WHERE name = $1
            "#,
        )
        .bind(name)
        .fetch_optional(pool)
        .await?;

        Ok(workspace)
    }

    pub(crate) async fn update_owner(
        name: &str,
        email: &str,
        pool: &PgPool,
    ) -> Result<Self, AppError> {
        let user = User::find_user_by_email(email, pool).await?;
        if user.is_none() {
            return Err(AppError::NotFound("user not found".to_string()));
        }
        let owner_id = user.unwrap().id;
        let workspace = query_as(
            r#"
            UPDATE workspaces
            SET owner_id = $1
            WHERE name = $2
            RETURNING *
            "#,
        )
        .bind(owner_id)
        .bind(name)
        .fetch_one(pool)
        .await?;

        Ok(workspace)
    }

    pub(crate) async fn list_workspaces(pool: &PgPool) -> Result<Vec<Self>, AppError> {
        let workspaces = query_as(
            r#"
            SELECT *
            FROM workspaces
            "#,
        )
        .fetch_all(pool)
        .await?;

        Ok(workspaces)
    }
}
