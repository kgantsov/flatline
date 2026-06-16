use async_trait::async_trait;
use chrono::Utc;
use shared::models::User;
use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use crate::db::UserRepository;
use crate::error::ApiError;

pub struct SqliteUserRepository {
    pub pool: SqlitePool,
}

#[async_trait]
impl UserRepository for SqliteUserRepository {
    async fn find_by_sub(&self, sub: &str) -> Result<Option<User>, ApiError> {
        let row = sqlx::query(
            "SELECT id, sub, email, name, created_at FROM users WHERE sub = ? LIMIT 1",
        )
        .bind(sub)
        .fetch_optional(&self.pool)
        .await?;

        row.map(|r| parse_user_row(&r)).transpose()
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, ApiError> {
        let id_str = id.to_string();
        let row = sqlx::query(
            "SELECT id, sub, email, name, created_at FROM users WHERE id = ? LIMIT 1",
        )
        .bind(&id_str)
        .fetch_optional(&self.pool)
        .await?;

        row.map(|r| parse_user_row(&r)).transpose()
    }

    async fn count(&self) -> Result<i64, ApiError> {
        let row = sqlx::query("SELECT COUNT(*) as count FROM users")
            .fetch_one(&self.pool)
            .await?;
        Ok(row.try_get::<i64, _>("count")?)
    }

    async fn create(&self, user: &User) -> Result<(), ApiError> {
        let id_str = user.id.to_string();
        let created_at_str = user.created_at.to_rfc3339();
        sqlx::query(
            "INSERT INTO users (id, sub, email, name, created_at) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(&id_str)
        .bind(&user.sub)
        .bind(&user.email)
        .bind(&user.name)
        .bind(&created_at_str)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

fn parse_user_row(row: &sqlx::sqlite::SqliteRow) -> Result<User, ApiError> {
    let id: String = row.try_get("id")?;
    let sub: String = row.try_get("sub")?;
    let email: Option<String> = row.try_get("email")?;
    let name: Option<String> = row.try_get("name")?;
    let created_at_str: String = row.try_get("created_at")?;
    Ok(User {
        id: Uuid::parse_str(&id)?,
        sub,
        email,
        name,
        created_at: chrono::DateTime::parse_from_rfc3339(&created_at_str)?.with_timezone(&Utc),
    })
}
