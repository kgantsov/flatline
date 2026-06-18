use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use shared::models::Incident;
use shared::models::LatencyPercentiles;

use crate::db::IncidentRepository;
use crate::error::ApiError;

pub struct SqliteIncidentRepository {
    pub pool: SqlitePool,
}

#[async_trait]
impl IncidentRepository for SqliteIncidentRepository {
    async fn open(
        &self,
        monitor_id: Uuid,
        started_at: DateTime<Utc>,
    ) -> Result<Incident, ApiError> {
        let id = Uuid::now_v7();
        let id_str = id.to_string();
        let monitor_id_str = monitor_id.to_string();
        let started_at_str = started_at.to_rfc3339();

        sqlx::query(
            "INSERT INTO incidents (id, monitor_id, started_at)
             VALUES (?, ?, ?)",
        )
        .bind(&id_str)
        .bind(&monitor_id_str)
        .bind(&started_at_str)
        .execute(&self.pool)
        .await?;

        Ok(Incident {
            id,
            monitor_id,
            started_at,
            resolved_at: None,
        })
    }

    async fn list_for_monitor(
        &self,
        monitor_id: Uuid,
        limit: i64,
        before: Option<DateTime<Utc>>,
    ) -> Result<Vec<Incident>, ApiError> {
        let monitor_id_str = monitor_id.to_string();

        let mut qb = sqlx::QueryBuilder::new(
            "SELECT id, monitor_id, started_at, resolved_at FROM incidents WHERE monitor_id = ",
        );
        qb.push_bind(&monitor_id_str);
        if let Some(before) = before {
            qb.push(" AND started_at < ");
            qb.push_bind(before.to_rfc3339());
        }
        qb.push(" ORDER BY started_at DESC LIMIT ");
        qb.push_bind(limit);

        let rows = qb.build().fetch_all(&self.pool).await?;

        rows.into_iter()
            .map(|row| {
                let id_str: String = row.try_get("id")?;
                let monitor_id_str: String = row.try_get("monitor_id")?;
                let started_at_str: String = row.try_get("started_at")?;
                let resolved_at_str: Option<String> = row.try_get("resolved_at")?;

                let id = Uuid::parse_str(&id_str)?;
                let monitor_id = Uuid::parse_str(&monitor_id_str)?;
                let started_at = DateTime::parse_from_rfc3339(&started_at_str)?.with_timezone(&Utc);
                let resolved_at = resolved_at_str
                    .map(|s| DateTime::parse_from_rfc3339(&s).map(|dt| dt.with_timezone(&Utc)))
                    .transpose()?;

                Ok(Incident {
                    id,
                    monitor_id,
                    started_at,
                    resolved_at,
                })
            })
            .collect()
    }

    async fn resolve(&self, id: Uuid, resolved_at: DateTime<Utc>) -> Result<Incident, ApiError> {
        let id_str = id.to_string();
        let resolved_at_str = resolved_at.to_rfc3339();

        let row = sqlx::query(
            "UPDATE incidents SET resolved_at = ? WHERE id = ?
             RETURNING id, monitor_id, started_at",
        )
        .bind(&resolved_at_str)
        .bind(&id_str)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("incident {id} not found")))?;

        let monitor_id_str: String = row.try_get("monitor_id")?;
        let started_at_str: String = row.try_get("started_at")?;

        Ok(Incident {
            id,
            monitor_id: Uuid::parse_str(&monitor_id_str)?,
            started_at: DateTime::parse_from_rfc3339(&started_at_str)?.with_timezone(&Utc),
            resolved_at: Some(resolved_at),
        })
    }

    async fn uptime_percentage(
        &self,
        monitor_id: Uuid,
        monitor_created_at: DateTime<Utc>,
        window_start: DateTime<Utc>,
    ) -> Result<Option<(f64, u64)>, ApiError> {
        // Clamp the window start to when the monitor actually existed.
        let effective_start = window_start.max(monitor_created_at);
        let effective_start_str = effective_start.to_rfc3339();
        let monitor_id_str = monitor_id.to_string();

        let row = sqlx::query(
            "SELECT
               (unixepoch('now') - unixepoch(?)) AS monitored_seconds,
               COALESCE(SUM(
                 MIN(
                   unixepoch(COALESCE(resolved_at, datetime('now'))),
                   unixepoch('now')
                 ) - MAX(
                   unixepoch(started_at),
                   unixepoch(?)
                 )
               ), 0) AS downtime_seconds
             FROM incidents
             WHERE monitor_id = ?
               AND unixepoch(started_at) < unixepoch('now')
               AND (resolved_at IS NULL OR resolved_at > ?)",
        )
        .bind(&effective_start_str)
        .bind(&effective_start_str)
        .bind(&monitor_id_str)
        .bind(&effective_start_str)
        .fetch_one(&self.pool)
        .await?;

        let monitored_seconds: i64 = row.try_get("monitored_seconds")?;
        if monitored_seconds <= 0 {
            return Ok(None);
        }

        let downtime_seconds: i64 = row.try_get("downtime_seconds")?;
        let downtime_seconds = downtime_seconds.max(0) as u64;
        let uptime =
            (monitored_seconds - downtime_seconds as i64) as f64 / monitored_seconds as f64;
        Ok(Some((uptime.clamp(0.0, 1.0), downtime_seconds)))
    }

    async fn latency_percentiles(
        &self,
        monitor_id: Uuid,
        window_start: DateTime<Utc>,
    ) -> Result<Option<LatencyPercentiles>, ApiError> {
        // Clamp the window start to when the monitor actually existed.
        // let effective_start = window_start.max(monitor_created_at);
        let effective_start_str = window_start.to_rfc3339();
        let monitor_id_str = monitor_id.to_string();

        let row = sqlx::query(
            "WITH ordered AS (
               SELECT response_time_ms,
                      ROW_NUMBER() OVER (ORDER BY response_time_ms) AS rn,
                      COUNT(*) OVER ()                               AS total
               FROM monitor_checks
               WHERE monitor_id = ? AND checked_at > ?
             )
             SELECT
               MAX(CASE WHEN rn * 100 <= total * 50 THEN response_time_ms END) AS p50,
               MAX(CASE WHEN rn * 100 <= total * 95 THEN response_time_ms END) AS p95
             FROM ordered",
        )
        .bind(&monitor_id_str)
        .bind(&effective_start_str)
        .fetch_one(&self.pool)
        .await?;

        let p50: Option<i64> = row.try_get("p50")?;
        let p95: Option<i64> = row.try_get("p95")?;

        match (p50, p95) {
            (Some(p50), Some(p95)) => Ok(Some(LatencyPercentiles {
                p50_ms: p50 as u64,
                p95_ms: p95 as u64,
            })),
            _ => Ok(None),
        }
    }

    async fn get_open_for_monitor(&self, monitor_id: Uuid) -> Result<Option<Incident>, ApiError> {
        let monitor_id_str = monitor_id.to_string();

        let Some(row) = sqlx::query(
            "SELECT id, started_at
             FROM incidents
             WHERE monitor_id = ? AND resolved_at IS NULL
             LIMIT 1",
        )
        .bind(&monitor_id_str)
        .fetch_optional(&self.pool)
        .await?
        else {
            return Ok(None);
        };

        let id_str: String = row.try_get("id")?;
        let started_at_str: String = row.try_get("started_at")?;

        Ok(Some(Incident {
            id: Uuid::parse_str(&id_str)?,
            monitor_id,
            started_at: DateTime::parse_from_rfc3339(&started_at_str)?.with_timezone(&Utc),
            resolved_at: None,
        }))
    }
}
