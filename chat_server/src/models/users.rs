use argon2::password_hash::{rand_core::OsRng, SaltString};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use sqlx::{query_as, PgPool};

use crate::error::AppError;
use crate::models::User;

impl User {
    pub async fn create(
        fullname: &str,
        email: &str,
        password: &str,
        pool: &PgPool,
    ) -> Result<Self, AppError> {
        let password_hash = Self::hash_password(password)?;
        let user = sqlx::query_as(
            r#"
            INSERT INTO users (fullname, email, password_hash)
            VALUES ($1, $2, $3)
            RETURNING *
            "#,
        )
        .bind(fullname)
        .bind(email)
        .bind(password_hash)
        .fetch_one(pool)
        .await?;

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
        let user: Option<User> = sqlx::query_as(
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
    ) -> Result<bool, AppError> {
        let user: Option<User> = query_as(
            r#"
            SELECT *
            FROM users
            WHERE email = $1
            "#,
        )
        .bind(&email)
        .fetch_optional(pool)
        .await?;

        if let Some(user) = user {
            let argon2 = Argon2::default();
            let parsed_hash = PasswordHash::new(&user.password_hash.as_ref().unwrap())?;
            Ok(argon2
                .verify_password(password.as_ref(), &parsed_hash)
                .is_ok())
        } else {
            Ok(false)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use sqlx_db_tester::TestPg;

    use super::*;

    #[tokio::test]
    async fn test_create_get_user() {
        let db = TestPg::new(
            "postgres://postgres:postgres@localhost:5432".to_string(),
            Path::new("../migrations"),
        );
        let pool = db.get_pool().await;
        let name = "lign";
        let email = "testlign@gmail.com";
        let pwd = "password123";

        let user = User::create(name, email, pwd, &pool).await.unwrap();
        assert_eq!(
            User::verify_password(email, pwd, &pool).await.unwrap(),
            true
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
