use chrono::{DateTime, Utc};
use image::codecs::jpeg::JpegDecoder;
use reqwest::{
    Client, Proxy,
    header::{ACCEPT, ACCEPT_LANGUAGE, AUTHORIZATION, HeaderMap},
};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::io::Cursor;

use db_wallet::{ContextIdType, JournalRefType};

pub const CORPORATION_ID: i64 = 98762057;

pub struct QueryDevice {
    client: Client,
    token_str: Option<String>,
}

impl QueryDevice {
    pub fn new(https_proxy: Option<String>, token_str: Option<String>) -> QueryDevice {
        let mut client_builder = Client::builder();
        if let Some(proxy_str) = https_proxy {
            client_builder = client_builder.proxy(Proxy::https(proxy_str).unwrap());
        }
        let client = client_builder.build().unwrap();
        QueryDevice { client, token_str }
    }

    pub async fn get_corporation_wallet_journal(
        &self,
        corporation_id: i64,
        division: i64,
        page: i32,
    ) -> Result<Option<ResCorporationWalletJournal>, String> {
        let url = format!(
            "https://esi.evetech.net/corporations/{corporation_id}/wallets/{division}/journal?page={page}",
        );
        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, "application/json".parse().unwrap());
        headers.insert(ACCEPT_LANGUAGE, "en".parse().unwrap());
        headers.insert(
            AUTHORIZATION,
            self.token_str.clone().unwrap().parse().unwrap(),
        );
        headers.insert("X-Compatibility-Date", "2025-09-30".parse().unwrap());

        let res = self
            .client
            .get(url)
            .headers(headers)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if res.status().is_success() {
            let r = res
                .json::<ResCorporationWalletJournal>()
                .await
                .map_err(|e| e.to_string())?;
            Ok(Some(r))
        } else if res.status().as_u16() == 404 {
            Ok(None)
        } else {
            let s = res.text().await.unwrap_or_else(|e| e.to_string());
            Err(s)
        }
    }

    pub async fn get_character_public_information(
        &self,
        character_id: i64,
    ) -> Result<Option<ResCharacterPublicInformation>, String> {
        let url = format!("https://esi.evetech.net/characters/{character_id}");
        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, "application/json".parse().unwrap());
        headers.insert(ACCEPT_LANGUAGE, "en".parse().unwrap());
        headers.insert("X-Compatibility-Date", "2025-09-30".parse().unwrap());

        let res = self
            .client
            .get(url)
            .headers(headers)
            .send()
            .await
            .map_err(|e| format!("get_character_public_information: {}", e.to_string()))?;

        if res.status().is_success() {
            let r = res
                .json::<ResCharacterPublicInformation>()
                .await
                .map_err(|e| e.to_string())?;
            Ok(Some(r))
        } else if res.status().as_u16() == 404 {
            Ok(None)
        } else {
            let s = res.text().await.unwrap_or_else(|e| e.to_string());
            Err(format!(
                "get_character_public_information({}): {}",
                character_id, s
            ))
        }
    }

    pub async fn get_corporation_information(
        &self,
        corporation_id: i64,
    ) -> Result<Option<ResCorporationInformation>, String> {
        println!("get_corporation_information id: {}", corporation_id);
        let url = format!("https://esi.evetech.net/corporations/{corporation_id}");
        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, "application/json".parse().unwrap());
        headers.insert(ACCEPT_LANGUAGE, "en".parse().unwrap());
        headers.insert("X-Compatibility-Date", "2025-09-30".parse().unwrap());

        let res = self
            .client
            .get(url)
            .headers(headers)
            .send()
            .await
            .map_err(|e| format!("get_corporation_information: {}", e.to_string()))?;

        if res.status().is_success() {
            let r = res
                .json::<ResCorporationInformation>()
                .await
                .map_err(|e| e.to_string())?;
            Ok(Some(r))
        } else if res.status().as_u16() == 404 {
            Ok(None)
        } else {
            let s = res.text().await.unwrap_or_else(|e| e.to_string());
            Err(format!(
                "get_corporation_information({}): {}",
                corporation_id, s
            ))
        }
    }

    pub async fn get_character_portraits(
        &self,
        character_id: i64,
    ) -> Result<ResCharacterPortraits, String> {
        let url = format!("https://esi.evetech.net/characters/{character_id}/portrait");
        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, "application/json".parse().unwrap());
        headers.insert(ACCEPT_LANGUAGE, "en".parse().unwrap());
        headers.insert("X-Compatibility-Date", "2025-09-30".parse().unwrap());

        let res = self
            .client
            .get(url)
            .headers(headers)
            .send()
            .await
            .map_err(|e| format!("get_character_portraits: {}", e.to_string()))?;

        if res.status().is_success() {
            let r = res
                .json::<ResCharacterPortraits>()
                .await
                .map_err(|e| e.to_string())?;
            Ok(r)
        } else {
            let s = res.text().await.unwrap_or_else(|e| e.to_string());
            Err(s)
        }
    }

    pub async fn get_image(&self, url: &str) -> Result<Vec<u8>, String> {
        let res = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| format!("get_image: {}", e.to_string()))?;
        if res.status().is_success() == false {
            let s = res.text().await.unwrap_or_else(|e| e.to_string());
            return Err(s);
        }
        let data = res.bytes().await.map_err(|e| e.to_string())?;
        match JpegDecoder::new(Cursor::new(data.as_ref())) {
            Ok(_) => Ok(data.to_vec()),
            Err(e) => Err(e.to_string()),
        }
    }

    pub async fn get_portraits(&self, urls: &ResCharacterPortraits) -> Result<Portraits, String> {
        let portrait64 = self.get_image(urls.px64x64.as_str()).await?;
        let portrait128 = self.get_image(urls.px128x128.as_str()).await?;
        let portrait256 = self.get_image(urls.px256x256.as_str()).await?;
        let portrait512 = self.get_image(urls.px512x512.as_str()).await?;
        Ok(Portraits {
            portrait64,
            portrait128,
            portrait256,
            portrait512,
        })
    }
}

pub struct Portraits {
    pub portrait64: Vec<u8>,
    pub portrait128: Vec<u8>,
    pub portrait256: Vec<u8>,
    pub portrait512: Vec<u8>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ResCorporationWalletJournal(pub Vec<ResCorporationWalletJournalItem>);

#[derive(Serialize, Deserialize, Clone)]
pub struct ResCorporationWalletJournalItem {
    pub id: i64,
    pub date: DateTime<Utc>,
    pub ref_type: JournalRefType,
    pub description: String,
    pub amount: Option<Decimal>,
    pub balance: Option<Decimal>,
    pub context_id: Option<i64>,
    pub context_id_type: Option<ContextIdType>,
    pub reason: Option<String>,
    pub first_party_id: Option<i64>,
    pub second_party_id: Option<i64>,
    pub tax: Option<Decimal>,
    pub tax_receiver_id: Option<i64>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ResCharacterPublicInformation {
    pub name: String,
    pub birthday: DateTime<Utc>,
    pub corporation_id: i64,
    pub bloodline_id: i64,
    pub race_id: i64,
    pub gender: String,
    pub alliance_id: Option<i64>,
    pub description: Option<String>,
    pub security_status: Option<f64>,
    pub title: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ResCharacterPortraits {
    pub px64x64: String,
    pub px128x128: String,
    pub px256x256: String,
    pub px512x512: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ResCorporationInformation {
    pub name: String,
    pub ticker: String,
    pub date_founded: Option<DateTime<Utc>>,
    pub description: Option<String>,
    pub alliance_id: Option<i64>,
    pub ceo_id: i64,
    pub faction_id: Option<i64>,
    pub home_station_id: Option<i64>,
    pub member_count: i64,
    pub shares: Option<i64>,
    pub tax_rate: f64,
    pub url: Option<String>,
    pub war_eligible: Option<bool>,
}
