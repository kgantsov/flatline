#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use chrono::Utc;
    use http_body_util::BodyExt;
    use server::db::{CheckRepository, IncidentRepository, MonitorRepository};
    use server::error::ApiError;
    use server::{AppState, build_router};
    use shared::api::{CreateMonitorCheckRequest, CreateMonitorRequest, UpdateMonitorRequest};
    use shared::models::{Monitor, MonitorCheck};
    use std::sync::Arc;
    use tower::ServiceExt; // for `oneshot`
    use uuid::Uuid;

    use mockall::mock;

    mock! {
        pub MonitorRepo {}

        #[async_trait::async_trait]
        impl MonitorRepository for MonitorRepo {
            async fn create(&self, input: CreateMonitorRequest) -> Result<Monitor, ApiError>;
            async fn list(&self) -> Result<Vec<Monitor>, ApiError>;
            async fn get(&self, id: Uuid) -> Result<Monitor, ApiError>;
            async fn update(&self, id: Uuid, input: UpdateMonitorRequest) -> Result<Monitor, ApiError>;
            async fn delete(&self, id: Uuid) -> Result<(), ApiError>;
        }
    }

    mock! {
        pub CheckRepo {}

        #[async_trait::async_trait]
        impl CheckRepository for CheckRepo {
            async fn create(&self, check: CreateMonitorCheckRequest) -> Result<MonitorCheck, ApiError>;
            async fn list_for_monitor(&self, monitor_id: Uuid, limit: i64, before: Option<chrono::DateTime<Utc>>) -> Result<Vec<MonitorCheck>, ApiError>;
        }
    }

    mock! {
        pub IncidentRepo {}

        #[async_trait::async_trait]
        impl IncidentRepository for IncidentRepo {
            async fn open(&self, monitor_id: Uuid, started_at: chrono::DateTime<Utc>) -> Result<shared::models::Incident, ApiError>;
            async fn resolve(&self, id: Uuid, resolved_at: chrono::DateTime<Utc>) -> Result<shared::models::Incident, ApiError>;
            async fn get_open_for_monitor(&self, monitor_id: Uuid) -> Result<Option<shared::models::Incident>, ApiError>;
            async fn list_for_monitor(&self, monitor_id: Uuid, limit: i64, before: Option<chrono::DateTime<Utc>>) -> Result<Vec<shared::models::Incident>, ApiError>;
        }
    }

    fn test_app(
        monitors_mock: MockMonitorRepo,
        checks_mock: MockCheckRepo,
        incidents_mock: MockIncidentRepo,
    ) -> axum::Router {
        build_router(AppState {
            monitors: Arc::new(monitors_mock),
            checks: Arc::new(checks_mock),
            incidents: Arc::new(incidents_mock),
            engine: server::monitor::engine::EngineHandle::new(),
        })
    }

    #[tokio::test]
    async fn create_monitor_returns_201() {
        let mut mock = MockMonitorRepo::new();

        mock.expect_create().once().returning(|input| {
            let now = Utc::now();
            Ok(Monitor {
                id: Uuid::now_v7(),
                name: input.name,
                config: input.config,
                interval: input.interval,
                timeout: input.timeout,
                enabled: input.enabled.unwrap_or(true),
                created_at: now,
                updated_at: now,
            })
        });

        let mut checks_mock = MockCheckRepo::new();
        checks_mock
            .expect_list_for_monitor()
            .returning(|_, _, _| Ok(vec![]));

        let app = test_app(mock, checks_mock, MockIncidentRepo::new());

        let body = serde_json::json!({
            "name": "My Site",
            "config": { "type": "http", "url": "https://example.com" },
            "interval": 60,
            "timeout": 10,
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/monitors")
                    .header("content-type", "application/json")
                    .body(Body::from(body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);

        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let monitor: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(monitor["name"], "My Site");
        assert_eq!(monitor["config"]["type"], "http");
        assert_eq!(monitor["config"]["url"], "https://example.com");
    }

    #[tokio::test]
    async fn create_monitor_rejects_missing_fields() {
        let mock = MockMonitorRepo::new(); // no expectations — repo won't be called

        let body = serde_json::json!({
            "name": "My Site",
            "config": { "type": "http", "url": "https://example.com" },
            // missing interval and timeout
        });

        let response = test_app(mock, MockCheckRepo::new(), MockIncidentRepo::new())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/monitors")
                    .header("content-type", "application/json")
                    .body(Body::from(body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    // helper to build a Monitor with sensible defaults
    fn make_monitor(id: Uuid, name: &str) -> Monitor {
        use shared::models::MonitorConfig;
        let now = Utc::now();
        Monitor {
            id,
            name: name.to_string(),
            config: MonitorConfig::Http {
                url: "https://example.com".to_string(),
                method: None,
                expected_status: None,
            },
            interval: 60,
            timeout: 10,
            enabled: true,
            created_at: now,
            updated_at: now,
        }
    }

    // --- list ---

    #[tokio::test]
    async fn list_monitors_returns_200() {
        let mut mock = MockMonitorRepo::new();
        let monitors = vec![
            make_monitor(Uuid::now_v7(), "Site A"),
            make_monitor(Uuid::now_v7(), "Site B"),
        ];
        mock.expect_list()
            .once()
            .returning(move || Ok(monitors.clone()));

        let response = test_app(mock, MockCheckRepo::new(), MockIncidentRepo::new())
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/api/v1/monitors")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(body.as_array().unwrap().len(), 2);
        assert_eq!(body[0]["name"], "Site A");
        assert_eq!(body[1]["name"], "Site B");
    }

    // --- get ---

    #[tokio::test]
    async fn get_monitor_returns_200() {
        let id = Uuid::now_v7();
        let monitor = make_monitor(id, "My Site");

        let mut mock = MockMonitorRepo::new();
        mock.expect_get()
            .withf(move |arg_id| *arg_id == id)
            .once()
            .returning(move |_| Ok(monitor.clone()));

        let response = test_app(mock, MockCheckRepo::new(), MockIncidentRepo::new())
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!("/api/v1/monitors/{id}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(body["name"], "My Site");
        assert_eq!(body["id"], id.to_string());
    }

    #[tokio::test]
    async fn get_monitor_returns_404_when_not_found() {
        let id = Uuid::now_v7();

        let mut mock = MockMonitorRepo::new();
        mock.expect_get()
            .once()
            .returning(move |_| Err(ApiError::NotFound(format!("monitor {id} not found"))));

        let response = test_app(mock, MockCheckRepo::new(), MockIncidentRepo::new())
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!("/api/v1/monitors/{id}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    // --- update ---

    #[tokio::test]
    async fn update_monitor_returns_200() {
        let id = Uuid::now_v7();
        let updated = make_monitor(id, "Updated Name");

        let mut mock = MockMonitorRepo::new();
        mock.expect_update()
            .withf(move |arg_id, _| *arg_id == id)
            .once()
            .returning(move |_, _| Ok(updated.clone()));

        let mut checks_mock = MockCheckRepo::new();
        checks_mock
            .expect_list_for_monitor()
            .returning(|_, _, _| Ok(vec![]));

        let body = serde_json::json!({ "name": "Updated Name" });

        let response = test_app(mock, checks_mock, MockIncidentRepo::new())
            .oneshot(
                Request::builder()
                    .method("PATCH")
                    .uri(format!("/api/v1/monitors/{id}"))
                    .header("content-type", "application/json")
                    .body(Body::from(body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(body["name"], "Updated Name");
    }

    #[tokio::test]
    async fn update_monitor_returns_404_when_not_found() {
        let id = Uuid::now_v7();

        let mut mock = MockMonitorRepo::new();
        mock.expect_update()
            .once()
            .returning(move |_, _| Err(ApiError::NotFound(format!("monitor {id} not found"))));

        let body = serde_json::json!({ "name": "Updated Name" });

        let response = test_app(mock, MockCheckRepo::new(), MockIncidentRepo::new())
            .oneshot(
                Request::builder()
                    .method("PATCH")
                    .uri(format!("/api/v1/monitors/{id}"))
                    .header("content-type", "application/json")
                    .body(Body::from(body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    // --- checks ---

    fn make_check(monitor_id: Uuid) -> MonitorCheck {
        use shared::models::MonitorCheckStatus;
        MonitorCheck {
            id: Uuid::now_v7(),
            monitor_id,
            status: MonitorCheckStatus::Up,
            status_code: Some(200),
            response_time_ms: 42,
            error_message: None,
            checked_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn get_monitor_checks_returns_200_with_results() {
        let monitor_id = Uuid::now_v7();
        let checks = vec![make_check(monitor_id), make_check(monitor_id)];

        let mut checks_mock = MockCheckRepo::new();
        checks_mock
            .expect_list_for_monitor()
            .withf(move |id, limit, before| *id == monitor_id && *limit == 25 && before.is_none())
            .once()
            .returning(move |_, _, _| Ok(checks.clone()));

        let response = test_app(MockMonitorRepo::new(), checks_mock, MockIncidentRepo::new())
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!("/api/v1/monitors/{monitor_id}/checks"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(body.as_array().unwrap().len(), 2);
        assert_eq!(body[0]["monitor_id"], monitor_id.to_string());
    }

    #[tokio::test]
    async fn get_monitor_checks_passes_limit_param() {
        let monitor_id = Uuid::now_v7();

        let mut checks_mock = MockCheckRepo::new();
        checks_mock
            .expect_list_for_monitor()
            .withf(move |id, limit, before| *id == monitor_id && *limit == 10 && before.is_none())
            .once()
            .returning(|_, _, _| Ok(vec![]));

        let response = test_app(MockMonitorRepo::new(), checks_mock, MockIncidentRepo::new())
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!("/api/v1/monitors/{monitor_id}/checks?limit=10"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn get_monitor_checks_passes_before_param() {
        let monitor_id = Uuid::now_v7();
        let before_ts = "2026-06-04T12:00:00Z";
        let before_dt = chrono::DateTime::parse_from_rfc3339(before_ts)
            .unwrap()
            .with_timezone(&Utc);

        let mut checks_mock = MockCheckRepo::new();
        checks_mock
            .expect_list_for_monitor()
            .withf(move |id, _, before| *id == monitor_id && *before == Some(before_dt))
            .once()
            .returning(|_, _, _| Ok(vec![]));

        let response = test_app(MockMonitorRepo::new(), checks_mock, MockIncidentRepo::new())
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!(
                        "/api/v1/monitors/{monitor_id}/checks?before={before_ts}"
                    ))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn get_monitor_checks_clamps_limit_to_100() {
        let monitor_id = Uuid::now_v7();

        let mut checks_mock = MockCheckRepo::new();
        checks_mock
            .expect_list_for_monitor()
            .withf(move |id, limit, before| *id == monitor_id && *limit == 100 && before.is_none())
            .once()
            .returning(|_, _, _| Ok(vec![]));

        let response = test_app(MockMonitorRepo::new(), checks_mock, MockIncidentRepo::new())
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!("/api/v1/monitors/{monitor_id}/checks?limit=500"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    // --- incidents ---

    fn make_incident(monitor_id: Uuid) -> shared::models::Incident {
        shared::models::Incident {
            id: Uuid::now_v7(),
            monitor_id,
            started_at: Utc::now(),
            resolved_at: None,
        }
    }

    #[tokio::test]
    async fn get_monitor_incidents_returns_200_with_results() {
        let monitor_id = Uuid::now_v7();
        let incidents = vec![make_incident(monitor_id), make_incident(monitor_id)];

        let mut incidents_mock = MockIncidentRepo::new();
        incidents_mock
            .expect_list_for_monitor()
            .withf(move |id, limit, before| *id == monitor_id && *limit == 25 && before.is_none())
            .once()
            .returning(move |_, _, _| Ok(incidents.clone()));

        let response = test_app(MockMonitorRepo::new(), MockCheckRepo::new(), incidents_mock)
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!("/api/v1/monitors/{monitor_id}/incidents"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(body.as_array().unwrap().len(), 2);
        assert_eq!(body[0]["monitor_id"], monitor_id.to_string());
    }

    #[tokio::test]
    async fn get_monitor_incidents_returns_empty_list() {
        let monitor_id = Uuid::now_v7();

        let mut incidents_mock = MockIncidentRepo::new();
        incidents_mock
            .expect_list_for_monitor()
            .once()
            .returning(|_, _, _| Ok(vec![]));

        let response = test_app(MockMonitorRepo::new(), MockCheckRepo::new(), incidents_mock)
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!("/api/v1/monitors/{monitor_id}/incidents"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(body.as_array().unwrap().len(), 0);
    }

    #[tokio::test]
    async fn get_monitor_incidents_passes_limit_param() {
        let monitor_id = Uuid::now_v7();

        let mut incidents_mock = MockIncidentRepo::new();
        incidents_mock
            .expect_list_for_monitor()
            .withf(move |id, limit, before| *id == monitor_id && *limit == 10 && before.is_none())
            .once()
            .returning(|_, _, _| Ok(vec![]));

        let response = test_app(MockMonitorRepo::new(), MockCheckRepo::new(), incidents_mock)
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!("/api/v1/monitors/{monitor_id}/incidents?limit=10"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn get_monitor_incidents_clamps_limit_to_100() {
        let monitor_id = Uuid::now_v7();

        let mut incidents_mock = MockIncidentRepo::new();
        incidents_mock
            .expect_list_for_monitor()
            .withf(move |id, limit, before| *id == monitor_id && *limit == 100 && before.is_none())
            .once()
            .returning(|_, _, _| Ok(vec![]));

        let response = test_app(MockMonitorRepo::new(), MockCheckRepo::new(), incidents_mock)
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!("/api/v1/monitors/{monitor_id}/incidents?limit=500"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn get_monitor_incidents_passes_before_param() {
        let monitor_id = Uuid::now_v7();
        let before_ts = "2026-06-04T12:00:00Z";
        let before_dt = chrono::DateTime::parse_from_rfc3339(before_ts)
            .unwrap()
            .with_timezone(&Utc);

        let mut incidents_mock = MockIncidentRepo::new();
        incidents_mock
            .expect_list_for_monitor()
            .withf(move |id, _, before| *id == monitor_id && *before == Some(before_dt))
            .once()
            .returning(|_, _, _| Ok(vec![]));

        let response = test_app(MockMonitorRepo::new(), MockCheckRepo::new(), incidents_mock)
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!(
                        "/api/v1/monitors/{monitor_id}/incidents?before={before_ts}"
                    ))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    // --- delete ---

    #[tokio::test]
    async fn delete_monitor_returns_204() {
        let id = Uuid::now_v7();

        let mut mock = MockMonitorRepo::new();
        mock.expect_delete()
            .withf(move |arg_id| *arg_id == id)
            .once()
            .returning(|_| Ok(()));

        let response = test_app(mock, MockCheckRepo::new(), MockIncidentRepo::new())
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri(format!("/api/v1/monitors/{id}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn delete_monitor_returns_404_when_not_found() {
        let id = Uuid::now_v7();

        let mut mock = MockMonitorRepo::new();
        mock.expect_delete()
            .once()
            .returning(move |_| Err(ApiError::NotFound(format!("monitor {id} not found"))));

        let response = test_app(mock, MockCheckRepo::new(), MockIncidentRepo::new())
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri(format!("/api/v1/monitors/{id}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
