use async_trait::async_trait;
use chrono::Utc;
use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use shared::api::{CreateStatusPageRequest, UpdateStatusPageRequest};
use shared::models::StatusPage;

use crate::db::StatusPageRepository;
use crate::error::ApiError;

pub struct SqliteStatusPageRepository {
    pub pool: SqlitePool,
}

#[async_trait]
impl StatusPageRepository for SqliteStatusPageRepository {
    async fn create(&self, input: CreateStatusPageRequest) -> Result<StatusPage, ApiError> {
        let id = Uuid::now_v7();
        let now = Utc::now();
        let id_str = id.to_string();
        let created_at_str = now.to_rfc3339();
        let refresh_interval = input.refresh_interval.unwrap_or(60) as i64;

        sqlx::query(
            "INSERT INTO status_pages (
                id, name, description, slug, refresh_interval, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id_str)
        .bind(&input.name)
        .bind(&input.description)
        .bind(&input.slug)
        .bind(refresh_interval)
        .bind(&created_at_str)
        .bind(&created_at_str)
        .execute(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::Database(ref db_err) if db_err.is_unique_violation() => {
                ApiError::BadRequest(format!("slug '{}' is already taken", input.slug))
            }
            other => ApiError::from(other),
        })?;

        Ok(StatusPage {
            id,
            name: input.name,
            description: input.description,
            slug: input.slug,
            refresh_interval: refresh_interval as u32,
            created_at: now,
            updated_at: now,
        })
    }

    async fn list(&self) -> Result<Vec<StatusPage>, ApiError> {
        let rows = sqlx::query(
            "SELECT id, name, description, slug, refresh_interval, created_at, updated_at
             FROM status_pages ORDER BY created_at DESC LIMIT 100",
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|row| {
                let id_str: String = row.try_get("id")?;
                let created_at_str: String = row.try_get("created_at")?;
                let updated_at_str: String = row.try_get("updated_at")?;
                let refresh_interval: i64 = row.try_get("refresh_interval")?;

                Ok(StatusPage {
                    id: Uuid::parse_str(&id_str)?,
                    name: row.try_get("name")?,
                    description: row.try_get("description")?,
                    slug: row.try_get("slug")?,
                    refresh_interval: refresh_interval as u32,
                    created_at: chrono::DateTime::parse_from_rfc3339(&created_at_str)?
                        .with_timezone(&Utc),
                    updated_at: chrono::DateTime::parse_from_rfc3339(&updated_at_str)?
                        .with_timezone(&Utc),
                })
            })
            .collect()
    }

    async fn get(&self, id: Uuid) -> Result<StatusPage, ApiError> {
        let id_str = id.to_string();
        let row = sqlx::query(
            "SELECT id, name, description, slug, refresh_interval, created_at, updated_at
             FROM status_pages WHERE id = ? LIMIT 1",
        )
        .bind(&id_str)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("status page {id} not found")))?;

        let refresh_interval: i64 = row.try_get("refresh_interval")?;
        let created_at_str: String = row.try_get("created_at")?;
        let updated_at_str: String = row.try_get("updated_at")?;

        Ok(StatusPage {
            id: Uuid::parse_str(&row.try_get::<String, _>("id")?)?,
            name: row.try_get("name")?,
            description: row.try_get("description")?,
            slug: row.try_get("slug")?,
            refresh_interval: refresh_interval as u32,
            created_at: chrono::DateTime::parse_from_rfc3339(&created_at_str)?.with_timezone(&Utc),
            updated_at: chrono::DateTime::parse_from_rfc3339(&updated_at_str)?.with_timezone(&Utc),
        })
    }

    async fn get_by_slug(&self, slug: &str) -> Result<StatusPage, ApiError> {
        let row = sqlx::query(
            "SELECT id, name, description, slug, refresh_interval, created_at, updated_at
             FROM status_pages WHERE slug = ? LIMIT 1",
        )
        .bind(slug)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("status page '{slug}' not found")))?;

        let refresh_interval: i64 = row.try_get("refresh_interval")?;
        let created_at_str: String = row.try_get("created_at")?;
        let updated_at_str: String = row.try_get("updated_at")?;

        Ok(StatusPage {
            id: Uuid::parse_str(&row.try_get::<String, _>("id")?)?,
            name: row.try_get("name")?,
            description: row.try_get("description")?,
            slug: row.try_get("slug")?,
            refresh_interval: refresh_interval as u32,
            created_at: chrono::DateTime::parse_from_rfc3339(&created_at_str)?.with_timezone(&Utc),
            updated_at: chrono::DateTime::parse_from_rfc3339(&updated_at_str)?.with_timezone(&Utc),
        })
    }

    async fn update(&self, id: Uuid, input: UpdateStatusPageRequest) -> Result<StatusPage, ApiError> {
        let existing = self.get(id).await?;
        let now = Utc::now();
        let id_str = id.to_string();
        let updated_at_str = now.to_rfc3339();

        let name = input.name.unwrap_or(existing.name);
        let description = match input.description {
            Some(d) if d.is_empty() => None, // empty string = clear the description
            Some(d) => Some(d),
            None => existing.description,     // field absent = keep existing
        };
        let slug = input.slug.unwrap_or(existing.slug);
        let refresh_interval = input.refresh_interval.unwrap_or(existing.refresh_interval);

        sqlx::query(
            "UPDATE status_pages SET name = ?, description = ?, slug = ?, refresh_interval = ?, updated_at = ? WHERE id = ?",
        )
        .bind(&name)
        .bind(&description)
        .bind(&slug)
        .bind(refresh_interval as i64)
        .bind(&updated_at_str)
        .bind(&id_str)
        .execute(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::Database(ref db_err) if db_err.is_unique_violation() => {
                ApiError::BadRequest(format!("slug '{slug}' is already taken"))
            }
            other => ApiError::from(other),
        })?;

        Ok(StatusPage {
            id,
            name,
            description,
            slug,
            refresh_interval,
            created_at: existing.created_at,
            updated_at: now,
        })
    }

    async fn delete(&self, id: Uuid) -> Result<(), ApiError> {
        let id_str = id.to_string();
        sqlx::query("DELETE FROM status_pages WHERE id = ?")
            .bind(&id_str)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}
