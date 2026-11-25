use std::{env, time::Duration};

use anyhow::{bail, Context, Result};

const DEFAULT_INTERVAL_SECS: u64 = 60;
const DEFAULT_MODELS: &str = "2,3,4";
const DEFAULT_SUBSIDIARY: &str = "FR";

pub struct Config {
    pub webhook_url: String,
    pub check_interval: Duration,
    pub models: Vec<u8>,
    pub subsidiary: String,
    pub send_startup_message: bool,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let webhook_url =
            env::var("DISCORD_WEBHOOK_URL").context("DISCORD_WEBHOOK_URL is not set")?;

        let interval_secs = match env::var("CHECK_INTERVAL_SECONDS") {
            Ok(value) => value
                .parse::<u64>()
                .context("CHECK_INTERVAL_SECONDS must be a positive integer")?,
            Err(_) => DEFAULT_INTERVAL_SECS,
        };
        if interval_secs == 0 {
            bail!("CHECK_INTERVAL_SECONDS must be greater than zero");
        }

        let models = env::var("MODELS")
            .unwrap_or_else(|_| DEFAULT_MODELS.to_owned())
            .split(',')
            .map(|model| model.trim().parse::<u8>())
            .collect::<Result<Vec<_>, _>>()
            .context("MODELS must be a comma-separated list of numbers")?;

        let subsidiary =
            env::var("OVH_SUBSIDIARY").unwrap_or_else(|_| DEFAULT_SUBSIDIARY.to_owned());

        let send_startup_message = env::var("SEND_STARTUP_MESSAGE")
            .map(|value| !value.eq_ignore_ascii_case("false"))
            .unwrap_or(true);

        Ok(Self {
            webhook_url,
            check_interval: Duration::from_secs(interval_secs),
            models,
            subsidiary,
            send_startup_message,
        })
    }
}
