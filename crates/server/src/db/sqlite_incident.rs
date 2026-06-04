use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use shared::models::Incident;

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
