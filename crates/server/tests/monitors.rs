#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use chrono::Utc;
    use http_body_util::BodyExt;
    use server::db::MonitorRepository;
    use server::error::ApiError;
    use server::{AppState, build_router};
    use shared::api::{CreateMonitorRequest, UpdateMonitorRequest};
    use shared::models::Monitor;
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

    fn test_app(mock: MockMonitorRepo) -> axum::Router {
        build_router(AppState {
            monitors: Arc::new(mock),
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

        let app = test_app(mock);

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

        let response = test_app(mock)
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
        mock.expect_list().once().returning(move || Ok(monitors.clone()));

        let response = test_app(mock)
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

        let response = test_app(mock)
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

        let response = test_app(mock)
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

        let body = serde_json::json!({ "name": "Updated Name" });

        let response = test_app(mock)
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

        let response = test_app(mock)
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

    // --- delete ---

    #[tokio::test]
    async fn delete_monitor_returns_204() {
        let id = Uuid::now_v7();

        let mut mock = MockMonitorRepo::new();
        mock.expect_delete()
            .withf(move |arg_id| *arg_id == id)
            .once()
            .returning(|_| Ok(()));

        let response = test_app(mock)
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

        let response = test_app(mock)
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
