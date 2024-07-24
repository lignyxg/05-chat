use argon2::password_hash::{rand_core::OsRng, SaltString};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use sqlx::{query_as, PgPool};
use tracing::info;

use crate::error::AppError;
use crate::models::{CreateUser, CreateWorkspace, User, Workspace};

impl User {
    pub async fn create(create_user: CreateUser, pool: &PgPool) -> Result<Self, AppError> {
        if let Some(user) = Self::find_user_by_email(&create_user.email, pool).await? {
            return Err(AppError::EmailAlreadyExists(user.email));
        }
        let ws = match Workspace::find_workspace_by_name(&create_user.ws_name, pool).await? {
            Some(ws) => ws,
            None => {
                let ws = Workspace::create(
                    CreateWorkspace {
                        name: create_user.ws_name,
                        owner_id: 0,
                    },
                    pool,
                )
                .await?;
                info!("workspace {} created", ws.name);
                ws
            }
        };

        let password_hash = Self::hash_password(&create_user.password)?;
        let user: User = sqlx::query_as(
            r#"
            INSERT INTO users (ws_id, fullname, email, password_hash)
            VALUES ($1, $2, $3, $4)
            RETURNING *
            "#,
        )
        .bind(ws.id)
        .bind(create_user.fullname)
        .bind(create_user.email)
        .bind(password_hash)
        .fetch_one(pool)
        .await?;

        if ws.owner_id == 0 {
            Workspace::update_owner(&ws.name, &user.email, pool).await?;
            info!(
                "workspace {} owner updated([super admin] => [{}])",
                ws.name, user.fullname
            );
        }

        Ok(user)
    }

    fn hash_password(pwd: &str) -> Result<String, AppError> {
        let salt = SaltString::generate(&mut OsRng);

        // Argon2 with default params (Argon2id v19)
        let argon2 = Argon2::default();

        // Hash password to PHC string ($argon2id$v=19$...)
        let password_hash = argon2.hash_password(pwd.as_ref(), &salt)?.to_string();

        Ok(password_hash)
    }

    pub async fn update_password(id: i64, password: &str, pool: &PgPool) -> Result<Self, AppError> {
        let password_hash = Self::hash_password(password)?;
        let user = sqlx::query_as(
            r#"
            UPDATE users
            SET password_hash = $1
            WHERE id = $2
            RETURNING *
            "#,
        )
        .bind(password_hash)
        .bind(id)
        .fetch_one(pool)
        .await?;

        Ok(user)
    }

    pub async fn find_user_by_email(email: &str, pool: &PgPool) -> Result<Option<Self>, AppError> {
        let user: Option<User> = query_as(
            r#"
            SELECT *
            FROM users
            WHERE email = $1
            "#,
        )
        .bind(email)
        .fetch_optional(pool)
        .await?;

        if let Some(mut user) = user {
            user.password_hash.take();
            Ok(Some(user))
        } else {
            Ok(None)
        }
    }

    pub async fn verify_password(
        email: &str,
        password: &str,
        pool: &PgPool,
    ) -> Result<Option<Self>, AppError> {
        let user: Option<User> = query_as(
            r#"
            SELECT *
            FROM users
            WHERE email = $1
        "#,
        )
        .bind(email)
        .fetch_optional(pool)
        .await?;

        if let Some(user) = user {
            let argon2 = Argon2::default();
            let parsed_hash = PasswordHash::new(user.password_hash.as_ref().unwrap())?;
            match argon2.verify_password(password.as_ref(), &parsed_hash) {
                Ok(_) => Ok(Some(user)),
                Err(_) => Err(AppError::Unauthorized("password not match".to_string())),
            }
        } else {
            Ok(None)
        }
    }

    pub async fn list_users_by_workspace(ws_id: i64, pool: &PgPool) -> Result<Vec<Self>, AppError> {
        let users = query_as(
            r#"
            SELECT *
            FROM users
            WHERE ws_id = $1
            "#,
        )
        .bind(ws_id)
        .fetch_all(pool)
        .await?;

        Ok(users)
    }

    pub async fn find_user_by_ids(ids: &[i64], pool: &PgPool) -> Result<Vec<Self>, AppError> {
        let users = query_as(
            r#"
            SELECT *
            FROM users
            WHERE id = ANY($1)
            "#,
        )
        .bind(ids)
        .fetch_all(pool)
        .await?;

        Ok(users)
    }
}

#[cfg(test)]
impl User {
    pub fn new(id: i64, ws_id: i64, fullname: String, email: String) -> Self {
        Self {
            id,
            ws_id,
            fullname,
            email,
            password_hash: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::test_util::get_test_pool;

    use super::*;

    #[tokio::test]
    async fn test_create_get_user() {
        let (pool, _tdb) = get_test_pool(None).await;

        let name = "lign";
        let email = "testlign@gmail.com";
        let pwd = "password123";
        let ws_name = "test_ws";

        let create_user = CreateUser {
            ws_name: ws_name.to_string(),
            fullname: name.to_string(),
            email: email.to_string(),
            password: pwd.to_string(),
        };
        let user = User::create(create_user, &pool).await.unwrap();

        assert_eq!(
            User::verify_password(email, pwd, &pool).await.unwrap(),
            Some(user.clone())
        );
        let user_get = User::find_user_by_email(email, &pool)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(user_get.fullname, user.fullname);
        assert_eq!(user_get.email, user.email);
        assert_eq!(user_get.password_hash, None);
    }
}
