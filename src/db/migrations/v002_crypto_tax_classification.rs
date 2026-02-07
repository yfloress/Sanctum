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

//! Migration v002: Add tax classification + overrides to crypto transactions.

use crate::db::DbError;
use rusqlite::Connection;

pub fn up(conn: &Connection) -> Result<(), DbError> {
    conn.execute(
        "ALTER TABLE crypto_transactions ADD COLUMN tax_type TEXT",
        [],
    )?;
    conn.execute(
        "ALTER TABLE crypto_transactions ADD COLUMN tax_subtype TEXT",
        [],
    )?;
    conn.execute(
        "ALTER TABLE crypto_transactions ADD COLUMN override_proceeds REAL",
        [],
    )?;
    conn.execute(
        "ALTER TABLE crypto_transactions ADD COLUMN override_cost_basis REAL",
        [],
    )?;
    Ok(())
}
