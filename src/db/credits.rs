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

//! Credit database operations
//!
//! Fixed-term debts and their schedules. A payment is an ordinary row in
//! `transactions`, so budgets, charts and the activity list see it without
//! knowing credits exist; the installment only keeps a reference to it.

use super::{Database, DbError};
use crate::models::{Credit, CreditInstallment, Transaction};
use rusqlite::{Connection, Error as RusqliteError, Row, params};
use std::collections::HashMap;

impl Database {
    /// Writes a credit together with its whole schedule.
    ///
    /// One transaction: a credit without installments would show as a plan with
    /// nothing to pay, which no later action could repair.
    pub fn create_credit(
        &self,
        credit: &Credit,
        installments: &[CreditInstallment],
    ) -> Result<(), DbError> {
        self.with_transaction(|conn| {
            conn.execute(
                "INSERT INTO credits
                    (id, account_id, name, category, kind, down_payment, installment_amount,
                     installment_count, first_due_date, cash_price, principal, monthly_rate_ppm,
                     created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
                params![
                    credit.id,
                    credit.account_id,
                    credit.name,
                    credit.category,
                    credit.kind,
                    credit.down_payment,
                    credit.installment_amount,
                    credit.installment_count,
                    credit.first_due_date,
                    credit.cash_price,
                    credit.principal,
                    credit.monthly_rate_ppm,
                    credit.created_at,
                ],
            )?;

            for installment in installments {
                Self::insert_installment_on(conn, &credit.id, installment)?;
            }

            Ok(())
        })
    }

    fn insert_installment_on(
        conn: &Connection,
        credit_id: &str,
        installment: &CreditInstallment,
    ) -> Result<(), DbError> {
        conn.execute(
            "INSERT INTO credit_installments
                (id, credit_id, kind, number, amount, due_date, note, transaction_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, NULL)",
            params![
                installment.id,
                credit_id,
                installment.kind,
                installment.number,
                installment.amount,
                installment.due_date,
                installment.note,
            ],
        )?;
        Ok(())
    }

    /// Every credit, newest first.
    pub fn get_credits(&self) -> Result<Vec<Credit>, DbError> {
        self.read(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, account_id, name, category, kind, down_payment, installment_amount,
                        installment_count, first_due_date, cash_price, principal,
                        monthly_rate_ppm, created_at
                 FROM credits
                 ORDER BY created_at DESC",
            )?;
            let credits = stmt
                .query_map([], row_to_credit)?
                .collect::<Result<Vec<_>, _>>()?;
            Ok(credits)
        })
    }

    /// A single credit by id.
    pub fn get_credit(&self, id: &str) -> Result<Credit, DbError> {
        self.read(|conn| Self::get_credit_on(conn, id))
    }

    fn get_credit_on(conn: &Connection, id: &str) -> Result<Credit, DbError> {
        conn.query_row(
            "SELECT id, account_id, name, category, kind, down_payment, installment_amount,
                    installment_count, first_due_date, cash_price, principal,
                    monthly_rate_ppm, created_at
             FROM credits WHERE id = ?1",
            params![id],
            row_to_credit,
        )
        .map_err(|e| match e {
            RusqliteError::QueryReturnedNoRows => DbError::CreditNotFound,
            other => DbError::Sqlite(other),
        })
    }

    /// Every installment of every credit, keyed by credit id and in order.
    ///
    /// Read in one pass rather than per credit: the screen always draws the
    /// whole list, and a query per card is the same data in N round trips.
    pub fn get_credit_installments(
        &self,
    ) -> Result<HashMap<String, Vec<CreditInstallment>>, DbError> {
        self.read(|conn| {
            let mut stmt = conn.prepare(
                "SELECT i.id, i.credit_id, i.kind, i.number, i.amount, i.due_date, i.note,
                        i.transaction_id, t.date
                 FROM credit_installments i
                 LEFT JOIN transactions t ON t.id = i.transaction_id
                 ORDER BY i.credit_id, i.due_date ASC, i.number ASC",
            )?;
            let rows = stmt
                .query_map([], row_to_installment)?
                .collect::<Result<Vec<_>, _>>()?;

            let mut grouped: HashMap<String, Vec<CreditInstallment>> = HashMap::new();
            for row in rows {
                grouped.entry(row.credit_id.clone()).or_default().push(row);
            }
            Ok(grouped)
        })
    }

    /// A single installment by id.
    pub fn get_credit_installment(&self, id: &str) -> Result<CreditInstallment, DbError> {
        self.read(|conn| {
            conn.query_row(
                "SELECT i.id, i.credit_id, i.kind, i.number, i.amount, i.due_date, i.note,
                        i.transaction_id, t.date
                 FROM credit_installments i
                 LEFT JOIN transactions t ON t.id = i.transaction_id
                 WHERE i.id = ?1",
                params![id],
                row_to_installment,
            )
            .map_err(|e| match e {
                RusqliteError::QueryReturnedNoRows => DbError::InstallmentNotFound,
                other => DbError::Sqlite(other),
            })
        })
    }

    /// Deletes a credit and its schedule.
    ///
    /// Payments already made stay in the ledger: the money did leave the
    /// account, and dropping the plan is not a claim that it never did.
    pub fn delete_credit(&self, id: &str) -> Result<(), DbError> {
        self.write(|conn| {
            let changed = conn.execute("DELETE FROM credits WHERE id = ?1", params![id])?;
            if changed == 0 {
                return Err(DbError::CreditNotFound);
            }
            Ok(())
        })
    }

    /// Records `payment` as the payment of one installment.
    ///
    /// Both halves in one transaction: a payment the installment does not point
    /// at would be an untraceable expense, and a link to a row that was never
    /// written would show as paid with nothing behind it.
    pub fn pay_credit_installment(
        &self,
        installment_id: &str,
        payment: &Transaction,
    ) -> Result<(), DbError> {
        self.with_transaction(|conn| {
            let existing: Option<String> = conn
                .query_row(
                    "SELECT transaction_id FROM credit_installments WHERE id = ?1",
                    params![installment_id],
                    |row| row.get(0),
                )
                .map_err(|e| match e {
                    RusqliteError::QueryReturnedNoRows => DbError::InstallmentNotFound,
                    other => DbError::Sqlite(other),
                })?;

            if existing.is_some() {
                return Err(DbError::InstallmentAlreadyPaid);
            }

            conn.execute(
                "INSERT INTO transactions
                    (id, account_id, amount, category, description, date, type, transfer_account_id)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'expense', NULL)",
                params![
                    payment.id,
                    payment.account_id,
                    payment.amount,
                    payment.category,
                    payment.description,
                    payment.date,
                ],
            )?;

            conn.execute(
                "UPDATE credit_installments SET transaction_id = ?1 WHERE id = ?2",
                params![payment.id, installment_id],
            )?;

            Ok(())
        })
    }

    /// Undoes a payment, taking the transaction it created with it.
    ///
    /// The link is cleared before the delete so the outcome does not depend on
    /// foreign keys being enforced on this connection.
    pub fn unpay_credit_installment(&self, installment_id: &str) -> Result<(), DbError> {
        self.with_transaction(|conn| {
            let transaction_id: Option<String> = conn
                .query_row(
                    "SELECT transaction_id FROM credit_installments WHERE id = ?1",
                    params![installment_id],
                    |row| row.get(0),
                )
                .map_err(|e| match e {
                    RusqliteError::QueryReturnedNoRows => DbError::InstallmentNotFound,
                    other => DbError::Sqlite(other),
                })?;

            let transaction_id = transaction_id.ok_or(DbError::InstallmentNotPaid)?;

            conn.execute(
                "UPDATE credit_installments SET transaction_id = NULL WHERE id = ?1",
                params![installment_id],
            )?;
            conn.execute(
                "DELETE FROM transactions WHERE id = ?1",
                params![transaction_id],
            )?;

            Ok(())
        })
    }
}

impl Database {
    /// Corrects one unpaid row's amount and date.
    ///
    /// Only unpaid rows: once a row is paid the transaction is the amount, and
    /// letting the two say different things would put the schedule at odds with
    /// the ledger. Correcting a paid one means undoing the payment first.
    pub fn update_credit_installment(
        &self,
        installment_id: &str,
        amount: i64,
        due_date: &str,
    ) -> Result<(), DbError> {
        self.with_transaction(|conn| {
            let paid: Option<String> = conn
                .query_row(
                    "SELECT transaction_id FROM credit_installments WHERE id = ?1",
                    params![installment_id],
                    |row| row.get(0),
                )
                .map_err(|e| match e {
                    RusqliteError::QueryReturnedNoRows => DbError::InstallmentNotFound,
                    other => DbError::Sqlite(other),
                })?;

            if paid.is_some() {
                return Err(DbError::InstallmentAlreadyPaid);
            }

            conn.execute(
                "UPDATE credit_installments SET amount = ?1, due_date = ?2 WHERE id = ?3",
                params![amount, due_date, installment_id],
            )?;
            Ok(())
        })
    }

    /// Adds a fee the lender charged on top of the plan.
    ///
    /// Numbered within its own kind, so charges never collide with the
    /// installments they were charged over.
    pub fn add_credit_charge(&self, charge: &CreditInstallment) -> Result<(), DbError> {
        self.with_transaction(|conn| {
            // Fails here rather than on the foreign key, which would surface as
            // an opaque database error.
            let exists: i64 = conn.query_row(
                "SELECT COUNT(*) FROM credits WHERE id = ?1",
                params![charge.credit_id],
                |row| row.get(0),
            )?;
            if exists == 0 {
                return Err(DbError::CreditNotFound);
            }

            let next: i32 = conn.query_row(
                "SELECT COALESCE(MAX(number), 0) + 1 FROM credit_installments
                 WHERE credit_id = ?1 AND kind = 'charge'",
                params![charge.credit_id],
                |row| row.get(0),
            )?;

            let numbered = CreditInstallment {
                number: next,
                ..charge.clone()
            };
            Self::insert_installment_on(conn, &charge.credit_id, &numbered)
        })
    }

    /// Removes a charge. Only charges: the plan's own rows are not optional.
    pub fn delete_credit_charge(&self, installment_id: &str) -> Result<(), DbError> {
        self.write(|conn| {
            let changed = conn.execute(
                "DELETE FROM credit_installments
                 WHERE id = ?1 AND kind = 'charge' AND transaction_id IS NULL",
                params![installment_id],
            )?;
            if changed == 0 {
                return Err(DbError::InstallmentNotFound);
            }
            Ok(())
        })
    }
}

fn row_to_credit(row: &Row<'_>) -> rusqlite::Result<Credit> {
    Ok(Credit {
        id: row.get(0)?,
        account_id: row.get(1)?,
        name: row.get(2)?,
        category: row.get(3)?,
        kind: row.get(4)?,
        down_payment: row.get(5)?,
        installment_amount: row.get(6)?,
        installment_count: row.get(7)?,
        first_due_date: row.get(8)?,
        cash_price: row.get(9)?,
        principal: row.get(10)?,
        monthly_rate_ppm: row.get(11)?,
        created_at: row.get(12)?,
    })
}

fn row_to_installment(row: &Row<'_>) -> rusqlite::Result<CreditInstallment> {
    Ok(CreditInstallment {
        id: row.get(0)?,
        credit_id: row.get(1)?,
        kind: row.get(2)?,
        number: row.get(3)?,
        amount: row.get(4)?,
        due_date: row.get(5)?,
        note: row.get(6)?,
        transaction_id: row.get(7)?,
        paid_date: row.get(8)?,
    })
}
