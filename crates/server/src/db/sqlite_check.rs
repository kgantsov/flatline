use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use shared::api::CreateMonitorCheckRequest;
use shared::models::{MonitorCheck, MonitorCheckStatus};

use crate::db::CheckRepository;
use crate::error::ApiError;

pub struct SqliteCheckRepository {
    pub pool: SqlitePool,
}

#[async_trait]
impl CheckRepository for SqliteCheckRepository {
    async fn create(&self, check: CreateMonitorCheckRequest) -> Result<MonitorCheck, ApiError> {
        let id = Uuid::now_v7();
        let now = Utc::now();
        let id_str = id.to_string();
        let monitor_id_str = check.monitor_id.to_string();
        let checked_at_str = now.to_rfc3339();
        let status_str = check.status.to_string();
        let status_code = check.status_code.map(|v| v as i64);
        let response_time_ms = check.response_time_ms as i64;
        let error_message = check.error_message.clone();

        sqlx::query(
            "INSERT INTO monitor_checks (id, monitor_id, status, status_code, response_time_ms, error_message, checked_at)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id_str)
        .bind(&monitor_id_str)
        .bind(&status_str)
        .bind(status_code)
        .bind(response_time_ms)
        .bind(&error_message)
        .bind(&checked_at_str)
        .execute(&self.pool)
        .await
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

        Ok(MonitorCheck {
            id,
            monitor_id: check.monitor_id,
            status: check.status,
            status_code: check.status_code,
            response_time_ms: check.response_time_ms,
            error_message: check.error_message,
            checked_at: now,
        })
    }

    async fn list_for_monitor(&self, monitor_id: Uuid, limit: i64) -> Result<Vec<MonitorCheck>, ApiError> {
        let monitor_id_str = monitor_id.to_string();

        let rows = sqlx::query(
            "SELECT id, monitor_id, status, status_code, response_time_ms, error_message, checked_at
             FROM monitor_checks
             WHERE monitor_id = ?
             ORDER BY checked_at DESC
             LIMIT ?",
        )
        .bind(&monitor_id_str)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

        rows.into_iter()
            .map(|row| {
                let id_str: String = row.try_get("id").map_err(|e| ApiError::InternalServerError(e.to_string()))?;
                let monitor_id_str: String = row.try_get("monitor_id").map_err(|e| ApiError::InternalServerError(e.to_string()))?;
                let status_str: String = row.try_get("status").map_err(|e| ApiError::InternalServerError(e.to_string()))?;
                let status_code: Option<i64> = row.try_get("status_code").map_err(|e| ApiError::InternalServerError(e.to_string()))?;
                let response_time_ms: i64 = row.try_get("response_time_ms").map_err(|e| ApiError::InternalServerError(e.to_string()))?;
                let error_message: Option<String> = row.try_get("error_message").map_err(|e| ApiError::InternalServerError(e.to_string()))?;
                let checked_at_str: String = row.try_get("checked_at").map_err(|e| ApiError::InternalServerError(e.to_string()))?;

                let id = Uuid::parse_str(&id_str).map_err(|e| ApiError::InternalServerError(e.to_string()))?;
                let monitor_id = Uuid::parse_str(&monitor_id_str).map_err(|e| ApiError::InternalServerError(e.to_string()))?;
                let status = match status_str.as_str() {
                    "up" => MonitorCheckStatus::Up,
                    "down" => MonitorCheckStatus::Down,
                    other => return Err(ApiError::InternalServerError(format!("unknown status: {other}"))),
                };
                let checked_at = DateTime::parse_from_rfc3339(&checked_at_str)
                    .map_err(|e| ApiError::InternalServerError(e.to_string()))?
                    .with_timezone(&Utc);

                Ok(MonitorCheck {
                    id,
                    monitor_id,
                    status,
                    status_code: status_code.map(|v| v as u16),
                    response_time_ms: response_time_ms as u64,
                    error_message,
                    checked_at,
                })
            })
            .collect()
    }
}
