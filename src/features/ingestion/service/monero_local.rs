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
use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;
use serde::de::DeserializeOwned;
use std::io::Write;
use std::process::{Command, Stdio};
use zeroize::Zeroizing;

fn error(msg: &str) -> IngestionError {
    IngestionError::Validation(msg.into())
}

fn rpc<T: DeserializeOwned>(
    user: &str,
    password: &SecretString,
    method: &str,
    params: serde_json::Value,
) -> Result<T, IngestionError> {
    // Accept printable credentials only; curl configuration is supplied over stdin.
    let pass = password.expose_secret();
    if user.is_empty()
        || user.contains(':')
        || pass.is_empty()
        || user.len() + pass.len() > 1024
        || user
            .chars()
            .chain(pass.chars())
            .any(|c| c.is_control() || c == '"' || c == '\\')
    {
        return Err(error("Invalid local RPC credentials"));
    }
    let body = serde_json::json!({"jsonrpc":"2.0","id":"sanctum","method":method,"params":params})
        .to_string();
    let mut child = Command::new("curl")
        .args([
            "--disable",
            "--noproxy",
            "*",
            "--silent",
            "--fail",
            "--digest",
            "--max-time",
            "30",
            "--max-filesize",
            "10485760",
            "--header",
            "Content-Type: application/json",
            "--data",
            &body,
            "--config",
            "-",
            "http://127.0.0.1:18083/json_rpc",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|_| error("Local wallet connection requires curl"))?;
    let config = Zeroizing::new(format!("user = \"{user}:{pass}\"\n"));
    let wrote = child
        .stdin
        .take()
        .and_then(|mut pipe| pipe.write_all(config.as_bytes()).ok());
    let reply = child
        .wait_with_output()
        .map_err(|_| error("Local wallet unavailable"))?;
    if wrote.is_none() || !reply.status.success() {
        return Err(error("Check local wallet RPC and credentials"));
    }
    #[derive(Deserialize)]
    struct Response<R> {
        result: R,
    }
    serde_json::from_slice::<Response<T>>(&reply.stdout)
        .map(|r| r.result)
        .map_err(|_| error("Local wallet returned an incomplete response"))
}

impl IngestionService {
    pub fn read_monero_history(
        &self,
        address: &str,
        user: &str,
        password: &SecretString,
    ) -> Result<String, IngestionError> {
        self.with_db_readonly(|_| Ok(()))?;
        #[derive(Deserialize)]
        struct Address {
            address: String,
        }
        #[derive(Deserialize)]
        struct Transfer {
            timestamp: i64,
            height: u64,
            amount: u64,
            fee: u64,
            txid: String,
            confirmations: u64,
        }
        #[derive(Deserialize)]
        struct History {
            #[serde(default, rename = "in")]
            incoming: Vec<Transfer>,
            #[serde(default, rename = "out")]
            outgoing: Vec<Transfer>,
        }
        let check_address = || -> Result<(), IngestionError> {
            let actual: Address = rpc(
                user,
                password,
                "get_address",
                serde_json::json!({"account_index":0}),
            )?;
            if actual.address != address.trim() {
                return Err(error("Monero primary account address does not match"));
            }
            Ok(())
        };
        check_address()?;
        let rows: History = rpc(
            user,
            password,
            "get_transfers",
            serde_json::json!({"in":true,"out":true,"account_index":0}),
        )?;
        check_address()?;
        self.with_db_readonly(|_| Ok(()))?;
        let mut csv = csv::Writer::from_writer(Vec::new());
        let invalid = || error("Invalid wallet history");
        csv.write_record([
            "blockheight",
            "epoch",
            "date",
            "direction",
            "amount",
            "fee",
            "txid",
            "address",
            "description",
            "paymentid",
        ])
        .map_err(|_| invalid())?;
        let xmr = |v: u64| format!("{}.{:012}", v / 1_000_000_000_000, v % 1_000_000_000_000);
        for (direction, transfers) in [("in", rows.incoming), ("out", rows.outgoing)] {
            for tx in transfers {
                if tx.confirmations < 10 {
                    continue;
                }
                if tx.txid.len() != 64 || !tx.txid.bytes().all(|b| b.is_ascii_hexdigit()) {
                    return Err(invalid());
                }
                let date = jiff::Timestamp::from_second(tx.timestamp)
                    .map_err(|_| invalid())?
                    .strftime("%Y-%m-%d %H:%M:%S")
                    .to_string();
                csv.write_record([
                    tx.height.to_string(),
                    tx.timestamp.to_string(),
                    date,
                    direction.into(),
                    xmr(tx.amount),
                    xmr(tx.fee),
                    tx.txid,
                    String::new(),
                    String::new(),
                    String::new(),
                ])
                .map_err(|_| invalid())?;
            }
        }
        String::from_utf8(csv.into_inner().map_err(|_| invalid())?).map_err(|_| invalid())
    }
}
