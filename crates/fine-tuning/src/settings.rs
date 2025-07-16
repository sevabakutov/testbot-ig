use anyhow::{Context, Result};
use config::{Config, Environment};
use serde::Deserialize;
use std::fs::File;
use std::io::Read;

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub spreadsheet_id: String,
    pub service_account_json: Option<String>,
    pub service_account_path: Option<String>,
}

impl Settings {
    pub fn new() -> Result<Self> {
        let mut settings: Self = Config::builder()
            .add_source(Environment::with_prefix("GOOGLE"))
            .build()
            .context("Failed to build Config variable")?
            .try_deserialize::<Self>()
            .context("Unable to deserialize GOOGLE_* env vars into Settings")?;

        if let Some(ref path) = settings.service_account_path {
            let mut file = File::open(path).context(format!("Failed to open service account file: {}", path))?;
            let mut json_content = String::new();
            file.read_to_string(&mut json_content).context("Failed to read service account file")?;
            settings.service_account_json = Some(json_content);
        } else if settings.service_account_json.is_none() {
            return Err(anyhow::anyhow!("No service account JSON or path provided in env"));
        }

        Ok(settings)
    }

    pub fn spreadsheet_id(&self) -> String {
        self.spreadsheet_id.clone()
    }

    pub fn service_account_json(&self) -> &str {
        self.service_account_json.as_ref().unwrap()
    }
}