use async_trait::async_trait;
use chrono::Utc;
use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use shared::api::CreateMonitorNotificationRequest;
use shared::models::MonitorNotification;

use crate::db::MonitorNotificationRepository;
use crate::error::ApiError;

pub struct SqliteMonitorNotificationRepository {
    pub pool: SqlitePool,
}

#[async_trait]
impl MonitorNotificationRepository for SqliteMonitorNotificationRepository {
    async fn create(
        &self,
        monitor_id: Uuid,
        input: CreateMonitorNotificationRequest,
    ) -> Result<MonitorNotification, ApiError> {
        let id = Uuid::now_v7();
        let now = Utc::now();
        let id_str = id.to_string();
        let monitor_id_str = monitor_id.to_string();
        let channel_id_str = input.channel_id.to_string();
        let on_recovery = input.on_recovery.unwrap_or(true);
        let on_recovery_int = on_recovery as i64;
        let now_str = now.to_rfc3339();

        sqlx::query(
            "INSERT INTO monitor_notifications (id, monitor_id, channel_id, on_recovery, created_at)
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(&id_str)
        .bind(&monitor_id_str)
        .bind(&channel_id_str)
        .bind(on_recovery_int)
        .bind(&now_str)
        .execute(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::Database(ref db_err) if db_err.is_unique_violation() => {
                ApiError::BadRequest(format!(
                    "monitor {monitor_id} is already linked to channel {}",
                    input.channel_id
                ))
            }
            other => ApiError::from(other),
        })?;

        Ok(MonitorNotification {
            id,
            monitor_id,
            channel_id: input.channel_id,
            on_recovery,
            created_at: now,
        })
    }

    async fn list_for_monitor(
        &self,
        monitor_id: Uuid,
    ) -> Result<Vec<MonitorNotification>, ApiError> {
        let monitor_id_str = monitor_id.to_string();

        let rows = sqlx::query(
            "SELECT id, monitor_id, channel_id, on_recovery, created_at
             FROM monitor_notifications WHERE monitor_id = ? ORDER BY created_at ASC",
        )
        .bind(&monitor_id_str)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|row| {
                let id_str: String = row.try_get("id")?;
                let monitor_id_str: String = row.try_get("monitor_id")?;
                let channel_id_str: String = row.try_get("channel_id")?;
                let on_recovery_int: i64 = row.try_get("on_recovery")?;
                let created_at_str: String = row.try_get("created_at")?;

                Ok(MonitorNotification {
                    id: Uuid::parse_str(&id_str)?,
                    monitor_id: Uuid::parse_str(&monitor_id_str)?,
                    channel_id: Uuid::parse_str(&channel_id_str)?,
                    on_recovery: on_recovery_int != 0,
                    created_at: chrono::DateTime::parse_from_rfc3339(&created_at_str)?
                        .with_timezone(&Utc),
                })
            })
            .collect()
    }

    async fn delete(&self, monitor_id: Uuid, channel_id: Uuid) -> Result<(), ApiError> {
        let monitor_id_str = monitor_id.to_string();
        let channel_id_str = channel_id.to_string();

        let result = sqlx::query(
            "DELETE FROM monitor_notifications WHERE monitor_id = ? AND channel_id = ?",
        )
        .bind(&monitor_id_str)
        .bind(&channel_id_str)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(ApiError::NotFound(format!(
                "no notification link between monitor {monitor_id} and channel {channel_id}"
            )));
        }

        Ok(())
    }
}
