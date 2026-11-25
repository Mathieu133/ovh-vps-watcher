use std::{collections::HashMap, time::Duration};

use tokio::time::{self, MissedTickBehavior};
use tracing::{error, info, warn};

use crate::{
    discord::DiscordWebhook,
    ovh::{Datacenter, OvhClient},
};

const ORDER_URL: &str = "https://www.ovhcloud.com/fr/vps/";

struct StockChange {
    model: u8,
    datacenter: Datacenter,
}

pub struct Watcher {
    ovh: OvhClient,
    discord: DiscordWebhook,
    models: Vec<u8>,
    stock: HashMap<(u8, String), bool>,
}

impl Watcher {
    pub fn new(ovh: OvhClient, discord: DiscordWebhook, models: Vec<u8>) -> Self {
        Self {
            ovh,
            discord,
            models,
            stock: HashMap::new(),
        }
    }

    pub async fn run(mut self, check_interval: Duration) {
        let mut ticker = time::interval(check_interval);
        ticker.set_missed_tick_behavior(MissedTickBehavior::Delay);

        loop {
            ticker.tick().await;
            self.check().await;
        }
    }

    async fn check(&mut self) {
        let mut restocked = Vec::new();
        let mut sold_out = Vec::new();

        for &model in &self.models {
            let datacenters = match self.ovh.datacenters(model).await {
                Ok(datacenters) => datacenters,
                Err(err) => {
                    warn!(model, error = %err, "failed to fetch availability");
                    continue;
                }
            };

            for datacenter in datacenters.into_iter().filter(Datacenter::is_european) {
                let available = datacenter.is_available();
                let was_available = self
                    .stock
                    .insert((model, datacenter.code.clone()), available)
                    .unwrap_or(false);

                match (was_available, available) {
                    (false, true) => restocked.push(StockChange { model, datacenter }),
                    (true, false) => sold_out.push(StockChange { model, datacenter }),
                    _ => {}
                }
            }
        }

        if !restocked.is_empty() {
            self.notify(&restock_message(&restocked)).await;
        }
        if !sold_out.is_empty() {
            self.notify(&sold_out_message(&sold_out)).await;
        }

        let available_count = self.stock.values().filter(|&&available| available).count();
        info!(available = available_count, "check completed");
    }

    async fn notify(&self, content: &str) {
        if let Err(err) = self.discord.send(content).await {
            error!(error = %err, "failed to send Discord notification");
        }
    }
}

fn restock_message(changes: &[StockChange]) -> String {
    let lines: Vec<String> = changes
        .iter()
        .map(|change| {
            format!(
                "• **VPS-{}** → **{}** (Linux : {}, Windows : {})",
                change.model,
                change.datacenter.location(),
                change.datacenter.linux_status,
                change.datacenter.windows_status,
            )
        })
        .collect();

    format!(
        "🟢 **VPS OVH disponible !**\n{}\n👉 {ORDER_URL}",
        lines.join("\n")
    )
}

fn sold_out_message(changes: &[StockChange]) -> String {
    let lines: Vec<String> = changes
        .iter()
        .map(|change| format!("• VPS-{} → {}", change.model, change.datacenter.location()))
        .collect();

    format!("🔴 Plus en stock :\n{}", lines.join("\n"))
}
