use anyhow::{Context, Result};
use google_sheets4::{
    hyper_rustls::{
        HttpsConnector, 
        HttpsConnectorBuilder
    }, 
    hyper_util::{
        self, 
        client::legacy::connect::HttpConnector, 
        rt::TokioExecutor
    }, 
    yup_oauth2::{
        parse_service_account_key, 
        ServiceAccountAuthenticator,
        ServiceAccountKey
    }, 
    Sheets
};

use crate::settings::Settings;

pub struct Client {
    hub: Sheets<HttpsConnector<HttpConnector>>,
    spreadsheet_id: String
}

impl Client {
    pub async fn new(settings: Settings) -> Result<Self> {
        let spreadsheet_id = settings.spreadsheet_id();

        let sa_key: ServiceAccountKey = parse_service_account_key(settings.service_account_json().as_bytes())
            .context("Failed to parse service account JSON")?;

        let auth = ServiceAccountAuthenticator::builder(sa_key)
            .build()
            .await
            .context("Failed to build service account authenticator")?;

        let client = hyper_util::client::legacy::Client::builder(TokioExecutor::new())
            .build(
                HttpsConnectorBuilder::new()
                    .with_native_roots()
                    .context("Failed to build connector")?
                    .https_or_http()
                    .enable_http1()
                    .build()
            );
        let hub = Sheets::new(client, auth);

        Ok(Self {
            hub, 
            spreadsheet_id
        })
    }

    pub fn spreadsheet_id(&self) -> &str {
        &self.spreadsheet_id
    }

    pub async fn fetch_sheet(&self, range: &str) -> Result<Vec<Vec<String>>> {
        let (_, value_range) = self.hub
            .spreadsheets()
            .values_get(self.spreadsheet_id(), range)
            .doit()
            .await
            .context("Failed to fetch sheet")?;

        Ok(value_range.values
            .unwrap_or_default()
            .into_iter()
            .map(|row| row
                .into_iter()
                .map(|v| v.to_string())
                .collect::<Vec<String>>()
            )
            .collect::<Vec<Vec<String>>>()
        )
    }
}

