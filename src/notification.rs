use serde_json::json;
use tracing::{info, error};

pub struct NotificationHub {
    slack_webhook: Option<String>,
    pagerduty_key: Option<String>,
}

impl NotificationHub {
    pub fn new() -> Self {
        Self {
            slack_webhook: std::env::var("SLACK_WEBHOOK_URL").ok(),
            pagerduty_key: std::env::var("PAGERDUTY_ROUTING_KEY").ok(),
        }
    }

    pub async fn alert(&self, title: &str, message: &str, severity: &str) {
        info!("🔔 [ALERT] {}: {} ({})", severity, title, message);

        if let Some(url) = &self.slack_webhook {
            let payload = json!({
                "text": format!("🚨 *FluxGate Alert [{}]*\n*{}*\n{}", severity, title, message)
            });
            let client = reqwest::Client::new();
            if let Err(e) = client.post(url).json(&payload).send().await {
                error!("Failed to send Slack alert: {}", e);
            }
        }

        if let Some(key) = &self.pagerduty_key {
            let payload = json!({
                "routing_key": key,
                "event_action": "trigger",
                "payload": {
                    "summary": format!("{}: {}", title, message),
                    "source": "fluxgate-sovereign-01",
                    "severity": match severity {
                        "critical" => "critical",
                        "warning" => "warning",
                        _ => "info"
                    }
                }
            });
            let client = reqwest::Client::new();
            if let Err(e) = client.post("https://events.pagerduty.com/v2/enqueue").json(&payload).send().await {
                error!("Failed to send PagerDuty alert: {}", e);
            }
        }
    }
}
