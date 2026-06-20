mod channel_modal;
mod discord;
mod slack;
mod telegram;
mod webhook;
mod page;

pub use page::NotificationsPage;

use crate::api::NotificationChannelConfig;

pub(super) fn is_valid_url(s: &str) -> bool {
    let s = s.trim();
    (s.starts_with("http://") || s.starts_with("https://")) && s.len() > 8
}

fn channel_url(config: &NotificationChannelConfig) -> &str {
    match config {
        NotificationChannelConfig::Webhook { url } => url.as_str(),
        NotificationChannelConfig::Slack { url } => url.as_str(),
        NotificationChannelConfig::Telegram { url, .. } => url.as_str(),
        NotificationChannelConfig::Discord { url } => url.as_str(),
    }
}

fn channel_type_key(config: &NotificationChannelConfig) -> &'static str {
    match config {
        NotificationChannelConfig::Webhook { .. } => "webhook",
        NotificationChannelConfig::Slack { .. } => "slack",
        NotificationChannelConfig::Telegram { .. } => "telegram",
        NotificationChannelConfig::Discord { .. } => "discord",
    }
}

fn channel_type_label(config: &NotificationChannelConfig) -> &'static str {
    match config {
        NotificationChannelConfig::Webhook { .. } => "Webhook",
        NotificationChannelConfig::Slack { .. } => "Slack",
        NotificationChannelConfig::Telegram { .. } => "Telegram",
        NotificationChannelConfig::Discord { .. } => "Discord",
    }
}
