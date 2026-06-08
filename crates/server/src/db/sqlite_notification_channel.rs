use async_trait::async_trait;
use chrono::Utc;
use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use shared::api::{CreateNotificationChannelRequest, UpdateNotificationChannelRequest};
use shared::models::NotificationChannel;

use crate::db::NotificationChannelRepository;
use crate::error::ApiError;

pub struct SqliteNotificationChannelRepository {
    pub pool: SqlitePool,
}

#[async_trait]
impl NotificationChannelRepository for SqliteNotificationChannelRepository {
    async fn create(
        &self,
        input: CreateNotificationChannelRequest,
    ) -> Result<NotificationChannel, ApiError> {
        let id = Uuid::now_v7();
        let now = Utc::now();
        let id_str = id.to_string();
        let now_str = now.to_rfc3339();
        let config_json = serde_json::to_string(&input.config)?;

        sqlx::query(
            "INSERT INTO notification_channels (id, name, config, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(&id_str)
        .bind(&input.name)
        .bind(&config_json)
        .bind(&now_str)
        .bind(&now_str)
        .execute(&self.pool)
        .await?;

        Ok(NotificationChannel {
            id,
            name: input.name,
            config: input.config,
            created_at: now,
            updated_at: now,
        })
    }

    async fn list(&self) -> Result<Vec<NotificationChannel>, ApiError> {
        let rows = sqlx::query(
            "SELECT id, name, config, created_at, updated_at FROM notification_channels",
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|row| {
                let id_str: String = row.try_get("id")?;
                let name: String = row.try_get("name")?;
                let config_str: String = row.try_get("config")?;
                let created_at_str: String = row.try_get("created_at")?;
                let updated_at_str: String = row.try_get("updated_at")?;

                Ok(NotificationChannel {
                    id: Uuid::parse_str(&id_str)?,
                    name,
                    config: serde_json::from_str(&config_str)?,
                    created_at: chrono::DateTime::parse_from_rfc3339(&created_at_str)?
                        .with_timezone(&Utc),
                    updated_at: chrono::DateTime::parse_from_rfc3339(&updated_at_str)?
                        .with_timezone(&Utc),
                })
            })
            .collect()
    }

    async fn get(&self, id: Uuid) -> Result<NotificationChannel, ApiError> {
        let id_str = id.to_string();
        let row = sqlx::query(
            "SELECT id, name, config, created_at, updated_at
             FROM notification_channels WHERE id = ? LIMIT 1",
        )
        .bind(&id_str)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("notification channel {id} not found")))?;

        let name: String = row.try_get("name")?;
        let config_str: String = row.try_get("config")?;
        let created_at_str: String = row.try_get("created_at")?;
        let updated_at_str: String = row.try_get("updated_at")?;

        Ok(NotificationChannel {
            id,
            name,
            config: serde_json::from_str(&config_str)?,
            created_at: chrono::DateTime::parse_from_rfc3339(&created_at_str)?.with_timezone(&Utc),
            updated_at: chrono::DateTime::parse_from_rfc3339(&updated_at_str)?.with_timezone(&Utc),
        })
    }

    async fn update(
        &self,
        id: Uuid,
        input: UpdateNotificationChannelRequest,
    ) -> Result<NotificationChannel, ApiError> {
        let existing = self.get(id).await?;
        let now = Utc::now();
        let id_str = id.to_string();
        let now_str = now.to_rfc3339();

        let name = input.name.unwrap_or(existing.name);
        let config = input.config.unwrap_or(existing.config);
        let config_json = serde_json::to_string(&config)?;

        sqlx::query(
            "UPDATE notification_channels SET name = ?, config = ?, updated_at = ? WHERE id = ?",
        )
        .bind(&name)
        .bind(&config_json)
        .bind(&now_str)
        .bind(&id_str)
        .execute(&self.pool)
        .await?;

        Ok(NotificationChannel {
            id,
            name,
            config,
            created_at: existing.created_at,
            updated_at: now,
        })
    }

    async fn delete(&self, id: Uuid) -> Result<(), ApiError> {
        let id_str = id.to_string();
        sqlx::query("DELETE FROM notification_channels WHERE id = ?")
            .bind(&id_str)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}
