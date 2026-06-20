pub use shared::models::{
    Incident, Monitor, MonitorCheck, MonitorCheckStatus, MonitorConfig, MonitorNotification,
    MonitorStats, NotificationChannel, NotificationChannelConfig, SseEvent, User,
};

use serde::Serialize;

// ── Fetch ─────────────────────────────────────────────────────────────────────

pub async fn fetch_me() -> Result<User, ()> {
    let resp = gloo_net::http::Request::get("/auth/me")
        .send()
        .await
        .map_err(|_| ())?;
    if resp.ok() {
        resp.json::<User>().await.map_err(|_| ())
    } else {
        Err(())
    }
}

pub async fn fetch_monitors() -> Result<Vec<Monitor>, String> {
    let resp = gloo_net::http::Request::get("/api/v1/monitors")
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.ok() {
        return Err(format!("HTTP {}", resp.status()));
    }
    resp.json::<Vec<Monitor>>().await.map_err(|e| e.to_string())
}

pub async fn fetch_monitor(id: &str) -> Result<Monitor, String> {
    let resp = gloo_net::http::Request::get(&format!("/api/v1/monitors/{id}"))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.ok() {
        return Err(format!("HTTP {}", resp.status()));
    }
    resp.json::<Monitor>().await.map_err(|e| e.to_string())
}

pub async fn fetch_checks(id: &str, limit: u32) -> Vec<MonitorCheck> {
    let url = format!("/api/v1/monitors/{id}/checks?limit={limit}");
    let Ok(resp) = gloo_net::http::Request::get(&url).send().await else {
        return vec![];
    };
    if !resp.ok() {
        return vec![];
    }
    resp.json::<Vec<MonitorCheck>>().await.unwrap_or_default()
}

pub async fn fetch_incidents(id: &str) -> Vec<Incident> {
    let url = format!("/api/v1/monitors/{id}/incidents?limit=25");
    let Ok(resp) = gloo_net::http::Request::get(&url).send().await else {
        return vec![];
    };
    if !resp.ok() {
        return vec![];
    }
    resp.json::<Vec<Incident>>().await.unwrap_or_default()
}

pub async fn fetch_monitor_notifications(id: &str) -> Vec<MonitorNotification> {
    let url = format!("/api/v1/monitors/{id}/notifications");
    let Ok(resp) = gloo_net::http::Request::get(&url).send().await else {
        return vec![];
    };
    if !resp.ok() {
        return vec![];
    }
    resp.json::<Vec<MonitorNotification>>()
        .await
        .unwrap_or_default()
}

pub async fn fetch_notification_channels() -> Result<Vec<NotificationChannel>, String> {
    let resp = gloo_net::http::Request::get("/api/v1/notification-channels")
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.ok() {
        return Err(format!("HTTP {}", resp.status()));
    }
    resp.json::<Vec<NotificationChannel>>()
        .await
        .map_err(|e| e.to_string())
}

pub async fn fetch_all_channels() -> Vec<NotificationChannel> {
    let Ok(resp) = gloo_net::http::Request::get("/api/v1/notification-channels")
        .send()
        .await
    else {
        return vec![];
    };
    if !resp.ok() {
        return vec![];
    }
    resp.json::<Vec<NotificationChannel>>()
        .await
        .unwrap_or_default()
}

// ── Actions ───────────────────────────────────────────────────────────────────

#[derive(Serialize)]
struct ToggleBody {
    enabled: bool,
}

/// Used for both create (POST) and full update (PATCH).
#[derive(Serialize)]
pub struct MonitorFormData {
    pub name: String,
    pub interval: u32,
    pub timeout: u32,
    pub retries: u32,
    pub enabled: bool,
    pub config: MonitorConfigInput,
}

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum MonitorConfigInput {
    Http {
        url: String,
        method: String,
        #[serde(skip_serializing_if = "Vec::is_empty")]
        expected_status: Vec<u16>,
    },
}

pub async fn toggle_monitor(id: &str, enabled: bool) -> Result<Monitor, String> {
    let resp = gloo_net::http::Request::patch(&format!("/api/v1/monitors/{id}"))
        .json(&ToggleBody { enabled })
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.ok() {
        return Err(format!("HTTP {}", resp.status()));
    }
    resp.json::<Monitor>().await.map_err(|e| e.to_string())
}

pub async fn delete_monitor(id: &str) -> Result<(), String> {
    let resp = gloo_net::http::Request::delete(&format!("/api/v1/monitors/{id}"))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.ok() {
        return Err(format!("HTTP {}", resp.status()));
    }
    Ok(())
}

#[derive(Serialize)]
struct LinkBody {
    channel_id: String,
    on_recovery: bool,
}

pub async fn link_channel(
    monitor_id: &str,
    channel_id: &str,
    on_recovery: bool,
) -> Result<(), String> {
    let resp =
        gloo_net::http::Request::post(&format!("/api/v1/monitors/{monitor_id}/notifications"))
            .json(&LinkBody {
                channel_id: channel_id.to_string(),
                on_recovery,
            })
            .map_err(|e| e.to_string())?
            .send()
            .await
            .map_err(|e| e.to_string())?;
    if !resp.ok() {
        return Err(format!("HTTP {}", resp.status()));
    }
    Ok(())
}

pub async fn unlink_channel(monitor_id: &str, channel_id: &str) -> Result<(), String> {
    let resp = gloo_net::http::Request::delete(&format!(
        "/api/v1/monitors/{monitor_id}/notifications/{channel_id}"
    ))
    .send()
    .await
    .map_err(|e| e.to_string())?;
    if !resp.ok() {
        return Err(format!("HTTP {}", resp.status()));
    }
    Ok(())
}

async fn handle_monitor_response(resp: gloo_net::http::Response) -> Result<Monitor, String> {
    if !resp.ok() {
        let msg = resp
            .json::<serde_json::Value>()
            .await
            .ok()
            .and_then(|v| v["error"].as_str().map(str::to_string))
            .unwrap_or_else(|| format!("HTTP {}", resp.status()));
        return Err(msg);
    }
    resp.json::<Monitor>().await.map_err(|e| e.to_string())
}

pub async fn create_monitor(data: &MonitorFormData) -> Result<Monitor, String> {
    let resp = gloo_net::http::Request::post("/api/v1/monitors")
        .json(data)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;
    handle_monitor_response(resp).await
}

pub async fn update_monitor_full(id: &str, data: &MonitorFormData) -> Result<Monitor, String> {
    let resp = gloo_net::http::Request::patch(&format!("/api/v1/monitors/{id}"))
        .json(data)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;
    handle_monitor_response(resp).await
}

#[derive(Serialize)]
pub struct NotificationChannelFormData {
    pub name: String,
    pub config: NotificationChannelConfigInput,
}

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum NotificationChannelConfigInput {
    Webhook { url: String },
    Slack { webhook_url: String },
    Telegram { url: String, chat_id: String },
}

async fn handle_channel_response(
    resp: gloo_net::http::Response,
) -> Result<NotificationChannel, String> {
    if !resp.ok() {
        let msg = resp
            .json::<serde_json::Value>()
            .await
            .ok()
            .and_then(|v| v["error"].as_str().map(str::to_string))
            .unwrap_or_else(|| format!("HTTP {}", resp.status()));
        return Err(msg);
    }
    resp.json::<NotificationChannel>()
        .await
        .map_err(|e| e.to_string())
}

pub async fn create_channel(
    data: &NotificationChannelFormData,
) -> Result<NotificationChannel, String> {
    let resp = gloo_net::http::Request::post("/api/v1/notification-channels")
        .json(data)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;
    handle_channel_response(resp).await
}

pub async fn update_channel(
    id: &str,
    data: &NotificationChannelFormData,
) -> Result<NotificationChannel, String> {
    let resp = gloo_net::http::Request::patch(&format!("/api/v1/notification-channels/{id}"))
        .json(data)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;
    handle_channel_response(resp).await
}

pub async fn delete_channel(id: &str) -> Result<(), String> {
    let resp = gloo_net::http::Request::delete(&format!("/api/v1/notification-channels/{id}"))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.ok() {
        return Err(format!("HTTP {}", resp.status()));
    }
    Ok(())
}
