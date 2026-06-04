use async_trait::async_trait;
use chrono::Utc;
use sqlx::SqlitePool;
use uuid::Uuid;

use shared::api::{CreateMonitorRequest, UpdateMonitorRequest};
use shared::models::Monitor;

use crate::db::MonitorRepository;
use crate::error::ApiError;

pub struct SqliteMonitorRepository {
    pub pool: SqlitePool,
}

#[async_trait]
impl MonitorRepository for SqliteMonitorRepository {
    async fn create(&self, input: CreateMonitorRequest) -> Result<Monitor, ApiError> {
        let id = Uuid::now_v7();
        let now = Utc::now();
        let id_str = id.to_string();
        let created_at_str = now.to_rfc3339();
        let enabled = input.enabled.unwrap_or(true);
        let interval = input.interval as i64;
        let timeout = input.timeout as i64;
        let enabled_int = enabled as i64;
        let config_json = serde_json::to_string(&input.config)?;

        sqlx::query(
            "INSERT INTO monitors (id, name, config, interval, timeout, enabled, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id_str)
        .bind(&input.name)
        .bind(&config_json)
        .bind(interval)
        .bind(timeout)
        .bind(enabled_int)
        .bind(&created_at_str)
        .bind(&created_at_str)
        .execute(&self.pool)
        .await?;

        Ok(Monitor {
            id,
            name: input.name,
            config: input.config,
            interval: input.interval,
            timeout: input.timeout,
            enabled,
            created_at: now,
            updated_at: now,
        })
    }

    async fn list(&self) -> Result<Vec<Monitor>, ApiError> {
        let rows = sqlx::query!(
            "SELECT id, name, config, interval, timeout, enabled, created_at, updated_at FROM monitors LIMIT 100"
        )
        .fetch_all(&self.pool)
        .await?;

        let monitors = rows
            .into_iter()
            .filter_map(|row| {
                let id = Uuid::parse_str(&row.id).ok()?;
                let config = serde_json::from_str(&row.config).ok()?;
                let created_at = chrono::DateTime::parse_from_rfc3339(&row.created_at)
                    .ok()?
                    .with_timezone(&Utc);
                let updated_at = chrono::DateTime::parse_from_rfc3339(&row.updated_at)
                    .ok()?
                    .with_timezone(&Utc);

                Some(Monitor {
                    id,
                    name: row.name,
                    config,
                    interval: row.interval as u32,
                    timeout: row.timeout as u32,
                    enabled: row.enabled != 0,
                    created_at,
                    updated_at,
                })
            })
            .collect();

        Ok(monitors)
    }

    async fn get(&self, id: Uuid) -> Result<Monitor, ApiError> {
        let id_str = id.to_string();
        let row = sqlx::query!(
            "SELECT id, name, config, interval, timeout, enabled, created_at, updated_at
             FROM monitors WHERE id = ? LIMIT 1",
            id_str,
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("monitor {id} not found")))?;

        Ok(Monitor {
            id: Uuid::parse_str(&row.id)?,
            name: row.name,
            config: serde_json::from_str(&row.config)?,
            interval: row.interval as u32,
            timeout: row.timeout as u32,
            enabled: row.enabled != 0,
            created_at: chrono::DateTime::parse_from_rfc3339(&row.created_at)?.with_timezone(&Utc),
            updated_at: chrono::DateTime::parse_from_rfc3339(&row.updated_at)?.with_timezone(&Utc),
        })
    }

    async fn update(&self, id: Uuid, input: UpdateMonitorRequest) -> Result<Monitor, ApiError> {
        let existing = self.get(id).await?;
        let now = Utc::now();
        let id_str = id.to_string();
        let updated_at_str = now.to_rfc3339();

        let name = input.name.unwrap_or(existing.name);
        let config = input.config.unwrap_or(existing.config);
        let interval = input.interval.unwrap_or(existing.interval);
        let timeout = input.timeout.unwrap_or(existing.timeout);
        let enabled = input.enabled.unwrap_or(existing.enabled);
        let config_json = serde_json::to_string(&config)?;

        sqlx::query(
            "UPDATE monitors SET name = ?, config = ?, interval = ?, timeout = ?, enabled = ?, updated_at = ? WHERE id = ?",
        )
        .bind(&name)
        .bind(&config_json)
        .bind(interval as i64)
        .bind(timeout as i64)
        .bind(enabled as i64)
        .bind(&updated_at_str)
        .bind(&id_str)
        .execute(&self.pool)
        .await?;

        Ok(Monitor {
            id,
            name,
            config,
            interval,
            timeout,
            enabled,
            created_at: existing.created_at,
            updated_at: now,
        })
    }

    async fn delete(&self, id: Uuid) -> Result<(), ApiError> {
        let id = id.to_string();
        sqlx::query!("DELETE FROM monitors WHERE id = ?", id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}
