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

//! Database migration system
//!
//! Uses `PRAGMA user_version` for schema version tracking.
//! Migrations are applied incrementally and sequentially.
//!
//! ## Adding New Migrations
//! 1. Create `vXXX_description.rs` with `pub fn up(conn: &Connection) -> Result<(), DbError>`
//! 2. Add module declaration and Migration entry to `get_migrations()`
//! 3. Increment `SCHEMA_VERSION`

mod v001_initial_schema;

use super::DbError;
use rusqlite::Connection;

/// Current schema version - increment when adding new migrations
pub const SCHEMA_VERSION: i64 = 1;

/// Represents a single database migration
pub struct Migration {
    pub version: i64,
    pub name: &'static str,
    pub up: fn(&Connection) -> Result<(), DbError>,
}

/// Returns all migrations in version order
pub fn get_migrations() -> Vec<Migration> {
    vec![
        Migration {
            version: 1,
            name: "initial_schema",
            up: v001_initial_schema::up,
        },
    ]
}

/// Gets the current schema version from database
pub fn get_current_version(conn: &Connection) -> Result<i64, DbError> {
    conn.pragma_query_value(None, "user_version", |row| row.get(0))
        .map_err(DbError::Sqlite)
}

/// Runs all pending migrations from `from_version` to `to_version`
pub fn run_pending(conn: &Connection, from_version: i64, to_version: i64) -> Result<(), DbError> {
    for migration in get_migrations() {
        if migration.version > from_version && migration.version <= to_version {
            log::info!(
                "[Migration] Applying v{}: {}",
                migration.version,
                migration.name
            );
            (migration.up)(conn)?;
            conn.pragma_update(None, "user_version", migration.version)?;
            log::info!("[Migration] Completed v{}", migration.version);
        }
    }
    Ok(())
}
