// Sanctum — a privacy-first personal finance, crypto, and habits vault.
// Copyright (C) 2026  Kyronix
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

//! Tax settings types (per period)

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaxJurisdiction {
    Chile,
    Usa,
}

impl TaxJurisdiction {
    pub fn as_str(&self) -> &'static str {
        match self {
            TaxJurisdiction::Chile => "chile",
            TaxJurisdiction::Usa => "usa",
        }
    }

    pub fn from_str(raw: &str) -> Self {
        match raw.trim().to_lowercase().as_str() {
            "usa" | "us" | "united_states" => TaxJurisdiction::Usa,
            _ => TaxJurisdiction::Chile,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaxMethod {
    Fifo,
    Lifo,
    Hifo,
    Cpp,
}

impl TaxMethod {
    pub fn as_str(&self) -> &'static str {
        match self {
            TaxMethod::Fifo => "fifo",
            TaxMethod::Lifo => "lifo",
            TaxMethod::Hifo => "hifo",
            TaxMethod::Cpp => "cpp",
        }
    }

    pub fn from_str(raw: &str) -> Self {
        match raw.trim().to_lowercase().as_str() {
            "lifo" => TaxMethod::Lifo,
            "hifo" => TaxMethod::Hifo,
            "cpp" | "avg" | "average" => TaxMethod::Cpp,
            _ => TaxMethod::Fifo,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxPeriodSettings {
    pub period_id: String, // Year (YYYY) for now
    pub jurisdiction: String,
    pub method: String,
    pub include_swaps: bool,
    pub include_fee_crypto: bool,
}

impl TaxPeriodSettings {
    pub fn defaults_for(period_id: &str) -> Self {
        Self {
            period_id: period_id.to_string(),
            jurisdiction: TaxJurisdiction::Chile.as_str().to_string(),
            method: TaxMethod::Fifo.as_str().to_string(),
            include_swaps: true,
            include_fee_crypto: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TaxSettingsStore {
    pub periods: Vec<TaxPeriodSettings>,
}

impl TaxSettingsStore {
    pub fn get_for_period(&self, period_id: &str) -> Option<TaxPeriodSettings> {
        self.periods
            .iter()
            .find(|p| p.period_id == period_id)
            .cloned()
    }

    pub fn upsert(&mut self, settings: TaxPeriodSettings) {
        if let Some(existing) = self
            .periods
            .iter_mut()
            .find(|p| p.period_id == settings.period_id)
        {
            *existing = settings;
        } else {
            self.periods.push(settings);
        }
    }
}
