// Sanctum — a privacy-first personal finance and crypto vault.
// Copyright (C) 2026  yfloress
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/agpl-3.0.html>.
//

use super::{IngestionError, IngestionService};
use crate::db::Database;
use crate::features::crypto::api::{ProxyConfig, create_secure_client, download_response_body};
use crate::features::crypto::{SETTING_CRYPTO_PROXY_ENABLED, SETTING_CRYPTO_PROXY_URL};
use crate::features::ingestion::MAX_FILE_SIZE;
use crate::features::ingestion::parsers::{ExchangeSource, parser_for};
use crate::services::i18n::t;
use crate::ui::dto::kraken_sync::{KrakenSyncResult, KrakenSyncStatus};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use jiff::Timestamp;
use reqwest::Client;
use reqwest::header::HeaderValue;
use ring::{digest, hmac};
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use tokio::sync::Mutex;
use zeroize::{Zeroize, Zeroizing};

const SOURCE: &str = "kraken_ledger_source_v1";
const LEDGERS: &str = "/0/private/Ledgers";
const KEY_INFO: &str = "/0/private/GetApiKeyInfo";
const OVERLAP_SECS: i64 = 86_400;
static BUSY: Mutex<()> = Mutex::const_new(());
static NONCE: AtomicU64 = AtomicU64::new(0);

fn invalid(message: &str) -> IngestionError {
    IngestionError::Validation(message.into())
}

#[derive(Serialize, Deserialize)]
struct Source {
    generation: String,
    wallet_id: String,
    key: String,
    secret: String,
    on_unlock: bool,
    synced_through: Option<i64>,
}

impl Drop for Source {
    fn drop(&mut self) {
        self.key.zeroize();
        self.secret.zeroize();
    }
}

fn load(db: &Database) -> Result<Option<Source>, IngestionError> {
    let value = Zeroizing::new(db.get_setting(SOURCE)?.unwrap_or_default());
    if value.is_empty() {
        return Ok(None);
    }
    serde_json::from_str(&value)
        .map(Some)
        .map_err(|_| invalid("Invalid Kraken source configuration"))
}

fn save(db: &Database, source: &Source) -> Result<(), IngestionError> {
    let json = Zeroizing::new(
        serde_json::to_string(source)
            .map_err(|_| invalid("Invalid Kraken source configuration"))?,
    );
    Ok(db.set_setting(SOURCE, &json)?)
}

#[derive(Deserialize)]
struct Reply<T> {
    error: Vec<String>,
    result: Option<T>,
}

#[derive(Deserialize)]
struct Info {
    permissions: Vec<String>,
}

#[derive(Deserialize)]
struct Page {
    ledger: BTreeMap<String, Entry>,
    count: usize,
}

#[derive(Deserialize)]
struct Entry {
    refid: String,
    time: f64,
    #[serde(rename = "type")]
    kind: String,
    #[serde(default)]
    subtype: String,
    asset: String,
    amount: String,
    fee: String,
}

async fn request<T: DeserializeOwned>(
    client: &Client,
    source: &Source,
    path: &str,
    fields: &str,
) -> Result<T, IngestionError> {
    let secret = Zeroizing::new(
        STANDARD
            .decode(&source.secret)
            .map_err(|_| invalid("Invalid Kraken secret"))?,
    );
    let signer = hmac::Key::new(hmac::HMAC_SHA512, &secret);
    let mut api_key =
        HeaderValue::from_str(&source.key).map_err(|_| invalid("Invalid Kraken API key"))?;
    api_key.set_sensitive(true);
    for attempt in 0..3 {
        let clock = u64::try_from(Timestamp::now().as_millisecond()).unwrap_or_default();
        let previous = NONCE
            .try_update(Ordering::SeqCst, Ordering::SeqCst, |n| {
                Some(clock.max(n + 1))
            })
            .expect("nonce always advances");
        let nonce = clock.max(previous + 1);
        let body = format!("nonce={nonce}{fields}");
        let hash = digest::digest(&digest::SHA256, format!("{nonce}{body}").as_bytes());
        let mut signed = hmac::Context::with_key(&signer);
        signed.update(path.as_bytes());
        signed.update(hash.as_ref());
        let mut signature = HeaderValue::from_str(&STANDARD.encode(signed.sign().as_ref()))
            .map_err(|_| invalid("Invalid Kraken signature"))?;
        signature.set_sensitive(true);
        let response = client
            .post(format!("https://api.kraken.com{path}"))
            .header("API-Key", api_key.clone())
            .header("API-Sign", signature)
            .header("content-type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .map_err(|_| invalid("Kraken connection failed. Retry synchronization."))?;
        if response.status().as_u16() != 429 {
            if !response.status().is_success() {
                return Err(invalid(
                    "Kraken rejected the request. Verify the API key and retry.",
                ));
            }
            let bytes = Zeroizing::new(
                download_response_body(response)
                    .await
                    .map_err(|_| invalid("Invalid Kraken response"))?,
            );
            let reply: Reply<T> =
                serde_json::from_slice(&bytes).map_err(|_| invalid("Invalid Kraken response"))?;
            if reply.error.is_empty() {
                return reply
                    .result
                    .ok_or_else(|| invalid("Missing Kraken response"));
            }
            if !reply
                .error
                .iter()
                .any(|error| error.contains("Rate limit") || error.contains("Throttled"))
            {
                return Err(invalid(
                    "Kraken rejected the request. Verify the API key, clock and permissions.",
                ));
            }
        }
        if attempt < 2 {
            tokio::time::sleep(Duration::from_secs(4 << attempt)).await;
        }
    }
    Err(invalid("Kraken rate limit reached. Retry later."))
}

impl IngestionService {
    pub fn kraken_status(&self) -> Result<KrakenSyncStatus, IngestionError> {
        self.with_db_readonly(|db| {
            let source = load(db)?;
            Ok(KrakenSyncStatus {
                wallet_id: source.as_ref().map(|s| s.wallet_id.clone()),
                on_unlock: source.as_ref().is_some_and(|s| s.on_unlock),
                synced_through: source.and_then(|s| s.synced_through),
            })
        })
    }

    pub fn configure_kraken(
        &self,
        wallet_id: String,
        key: SecretString,
        secret: SecretString,
        on_unlock: bool,
    ) -> Result<(), IngestionError> {
        let _gate = BUSY
            .try_lock()
            .map_err(|_| invalid("Kraken sync is already running"))?;
        let key = key.expose_secret().trim();
        let secret = secret.expose_secret().trim();
        if key.is_empty()
            || secret.is_empty()
            || key.len() > 512
            || secret.len() > 512
            || HeaderValue::from_str(key).is_err()
            || STANDARD.decode(secret).map(Zeroizing::new).is_err()
        {
            return Err(invalid("Invalid Kraken credentials"));
        }
        self.with_db(|db| {
            db.get_wallet(&wallet_id)?
                .ok_or_else(|| invalid("Select an existing wallet"))?;
            save(
                db,
                &Source {
                    generation: uuid::Uuid::new_v4().to_string(),
                    wallet_id,
                    key: key.into(),
                    secret: secret.into(),
                    on_unlock,
                    synced_through: None,
                },
            )
        })
    }

    pub fn disconnect_kraken(&self) -> Result<(), IngestionError> {
        let _gate = BUSY
            .try_lock()
            .map_err(|_| invalid("Kraken sync is already running"))?;
        self.with_db(|db| Ok(db.set_setting(SOURCE, "")?))
    }

    pub async fn sync_kraken(&self, automatic: bool) -> Result<KrakenSyncResult, IngestionError> {
        let _gate = BUSY
            .try_lock()
            .map_err(|_| invalid("Kraken sync is already running"))?;
        let (mut source, proxy) = self.with_db_readonly(|db| {
            let source = load(db)?.ok_or_else(|| invalid("Configure a Kraken source first"))?;
            if automatic && !source.on_unlock {
                return Err(invalid("Automatic Kraken synchronization is disabled"));
            }
            let proxy = if db.get_setting(SETTING_CRYPTO_PROXY_ENABLED)?.as_deref() == Some("true")
            {
                Some(ProxyConfig {
                    url: db
                        .get_setting(SETTING_CRYPTO_PROXY_URL)?
                        .ok_or_else(|| invalid("Configure the enabled proxy first"))?,
                })
            } else {
                None
            };
            Ok((source, proxy))
        })?;
        let client = create_secure_client(proxy.as_ref())
            .map_err(|_| invalid("Could not create secure Kraken client"))?;
        let info: Info = request(&client, &source, KEY_INFO, "").await?;
        if info.permissions != ["query-ledger"] {
            return Err(invalid(
                "Use a dedicated Kraken key with only Query ledger entries enabled",
            ));
        }
        let end = Timestamp::now().as_second();
        let start = source
            .synced_through
            .unwrap_or(0)
            .saturating_sub(OVERLAP_SECS)
            .max(0);
        if source.synced_through.is_some_and(|cursor| cursor > end) {
            return Err(invalid("Check your clock before synchronizing Kraken"));
        }
        let mut csv = csv::Writer::from_writer(Vec::new());
        csv.write_record([
            "txid", "refid", "time", "type", "subtype", "asset", "amount", "fee",
        ])
        .map_err(|_| invalid("Could not read Kraken ledger"))?;
        let mut ids = std::collections::HashSet::new();
        let mut expected = None;
        loop {
            self.with_db_readonly(|db| ensure_current(db, &source))?;
            let fields = format!("&start={start}&end={end}&ofs={}", ids.len());
            let page: Page = request(&client, &source, LEDGERS, &fields).await?;
            if expected.is_some_and(|n| n != page.count)
                || (page.ledger.is_empty() && ids.len() < page.count)
            {
                return Err(invalid(
                    "Kraken ledger changed during synchronization. Retry.",
                ));
            }
            expected = Some(page.count);
            for (id, row) in page.ledger {
                if !ids.insert(id.clone()) {
                    return Err(invalid("Kraken ledger pagination changed. Retry."));
                }
                let time = Timestamp::from_second(row.time.floor() as i64)
                    .map_err(|_| invalid("Invalid Kraken ledger date"))?;
                let stamp = time.strftime("%Y-%m-%d %H:%M:%S").to_string();
                csv.write_record([
                    id,
                    row.refid,
                    stamp,
                    row.kind,
                    row.subtype,
                    row.asset,
                    row.amount,
                    row.fee,
                ])
                .map_err(|_| invalid("Invalid Kraken ledger row"))?;
                if csv.get_ref().len() > MAX_FILE_SIZE {
                    return Err(invalid("Kraken ledger exceeds the import size limit"));
                }
            }
            if ids.len() > page.count {
                return Err(invalid("Inconsistent Kraken ledger count"));
            }
            if ids.len() == page.count {
                break;
            }
            tokio::time::sleep(Duration::from_secs(3)).await;
        }
        let data = csv
            .into_inner()
            .map_err(|_| invalid("Invalid Kraken ledger"))?;
        let csv = std::str::from_utf8(&data).map_err(|_| invalid("Invalid Kraken ledger"))?;
        self.with_db_session(
            |db| {
                ensure_current(db, &source)?;
                let wallet = db
                    .get_wallet(&source.wallet_id)?
                    .ok_or_else(|| invalid("The configured Kraken wallet no longer exists"))?;
                let names =
                    crate::features::ingestion::IngestionRepository::build_wallet_lookup(db)?;
                if names
                    .get(&wallet.name.trim().to_lowercase())
                    .is_none_or(|w| w.id != wallet.id)
                {
                    return Err(invalid("The Kraken wallet name is ambiguous"));
                }
                let format = ExchangeSource::KrakenLedger;
                let parsed = parser_for(format)
                    .parse(csv, &wallet.name)
                    .map_err(|_| invalid("Could not parse Kraken ledger"))?;
                if !parsed.errors.is_empty() {
                    return Err(invalid(
                        "Kraken ledger has unsupported or invalid rows. Review a CSV export.",
                    ));
                }
                let preview = self.process_crypto_transactions_with_db(
                    db,
                    parsed.items.clone(),
                    format.label(),
                    true,
                    true,
                )?;
                let duplicates = t("import-skipped-duplicate-crypto");
                if preview.errors > 0
                    || preview
                        .skipped_reasons
                        .iter()
                        .any(|reason| reason != &duplicates)
                {
                    return Err(invalid(
                        "Some Kraken rows need review. Check wallet assets before retrying.",
                    ));
                }
                let result = self.process_crypto_transactions_with_db(
                    db,
                    parsed.items,
                    format.label(),
                    false,
                    true,
                )?;
                if result.errors > 0
                    || result
                        .skipped_reasons
                        .iter()
                        .any(|reason| reason != &duplicates)
                {
                    return Err(invalid(
                        "Kraken import was incomplete. The cursor was preserved for retry.",
                    ));
                }
                source.synced_through = Some(end);
                save(db, &source)?;
                Ok(KrakenSyncResult {
                    inserted: result.inserted,
                    duplicates: result.skipped,
                    synced_through: end,
                })
            },
            !automatic,
        )
    }
}

fn ensure_current(db: &Database, source: &Source) -> Result<(), IngestionError> {
    if !load(db)?.is_some_and(|other| other.generation == source.generation) {
        return Err(invalid(
            "The Kraken source changed. Synchronization stopped.",
        ));
    }
    Ok(())
}
