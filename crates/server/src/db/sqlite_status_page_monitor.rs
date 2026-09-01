use async_trait::async_trait;
use chrono::Utc;
use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use shared::api::AddStatusPageMonitorRequest;
use shared::models::StatusPageMonitor;

use crate::db::StatusPageMonitorRepository;
use crate::error::ApiError;

pub struct SqliteStatusPageMonitorRepository {
    pub pool: SqlitePool,
}

#[async_trait]
impl StatusPageMonitorRepository for SqliteStatusPageMonitorRepository {
    async fn add(
        &self,
        status_page_id: Uuid,
        req: AddStatusPageMonitorRequest,
    ) -> Result<StatusPageMonitor, ApiError> {
        let now = Utc::now();
        let status_page_id_str = status_page_id.to_string();
        let monitor_id_str = req.monitor_id.to_string();
        let now_str = now.to_rfc3339();

        sqlx::query(
            "INSERT INTO status_page_monitors (status_page_id, monitor_id, created_at)
             VALUES (?, ?, ?)",
        )
        .bind(&status_page_id_str)
        .bind(&monitor_id_str)
        .bind(&now_str)
        .execute(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::Database(ref db_err) if db_err.is_unique_violation() => {
                ApiError::BadRequest(format!(
                    "monitor {} is already on this status page",
                    req.monitor_id
                ))
            }
            other => ApiError::from(other),
        })?;

        Ok(StatusPageMonitor {
            status_page_id,
            monitor_id: req.monitor_id,
            created_at: now,
        })
    }

    async fn list_for_page(
        &self,
        status_page_id: Uuid,
    ) -> Result<Vec<StatusPageMonitor>, ApiError> {
        let status_page_id_str = status_page_id.to_string();

        let rows = sqlx::query(
            "SELECT status_page_id, monitor_id, created_at
             FROM status_page_monitors WHERE status_page_id = ? ORDER BY created_at ASC",
        )
        .bind(&status_page_id_str)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|row| {
                let status_page_id_str: String = row.try_get("status_page_id")?;
                let monitor_id_str: String = row.try_get("monitor_id")?;
                let created_at_str: String = row.try_get("created_at")?;

                Ok(StatusPageMonitor {
                    status_page_id: Uuid::parse_str(&status_page_id_str)?,
                    monitor_id: Uuid::parse_str(&monitor_id_str)?,
                    created_at: chrono::DateTime::parse_from_rfc3339(&created_at_str)?
                        .with_timezone(&Utc),
                })
            })
            .collect()
    }

    async fn remove(&self, status_page_id: Uuid, monitor_id: Uuid) -> Result<(), ApiError> {
        let status_page_id_str = status_page_id.to_string();
        let monitor_id_str = monitor_id.to_string();

        let result = sqlx::query(
            "DELETE FROM status_page_monitors WHERE status_page_id = ? AND monitor_id = ?",
        )
        .bind(&status_page_id_str)
        .bind(&monitor_id_str)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(ApiError::NotFound(format!(
                "monitor {monitor_id} is not on status page {status_page_id}"
            )));
        }

        Ok(())
    }
}
