# Plan: Universal Data Ingestion (Issue #18)

## Objetivo
Importar transacciones y habit logs desde archivos externos (JSON, CSV, TXT) sin llamadas de red.

---

## Formatos soportados

### 1. JSON v1 (Sanctum Web)
```json
{
  "version": "1.0",
  "exported_at": "2024-01-15T10:30:00Z",
  "transactions": [
    {
      "date": "2024-01-15",
      "account": "Banco Chile",
      "type": "expense",
      "amount": 45.50,
      "currency": "CLP",
      "category": "Comida",
      "description": "Supermercado",
      "transfer_to_account": null
    }
  ],
  "habit_logs": [
    {
      "habit": "Meditar",
      "date": "2024-01-15",
      "completed": true
    }
  ]
}
```

### 2. CSV (Excel/Sheets)
**transactions.csv:**
```csv
date,account,type,amount,currency,category,description,transfer_to_account
2024-01-15,Banco Chile,expense,45.50,CLP,Comida,Supermercado,
2024-01-14,Banco Chile,transfer,500.00,CLP,,Ahorro mensual,Cuenta Ahorro
```

**habit_logs.csv:**
```csv
habit,date,completed
Meditar,2024-01-15,true
Ejercicio,2024-01-15,false
```

### 3. Plain Text (semicolon-separated)
**Transactions:**
```
# Transactions (date;account;type;amount;currency;category;description;transfer_to_account)
2024-01-15;Banco Chile;expense;45.50;CLP;Comida;Supermercado;
2024-01-14;Banco Chile;transfer;500.00;CLP;;Ahorro mensual;Cuenta Ahorro
```

**Habit Logs:**
```
# Habit Logs (habit;date;completed)
Meditar;2024-01-15;true
```

---

## Estructura de archivos
```
src/
  features/
    ingestion/
      mod.rs
      service.rs
      repository.rs
      validation.rs
      types.rs
      parsers/
        mod.rs
        json.rs
        csv.rs
        text.rs
  controller/
    ingestion.rs
```

---

## Tipos de datos
```rust
pub struct ImportTransaction {
    pub date: String,              // YYYY-MM-DD
    pub account: String,
    pub transaction_type: String,  // income/expense/transfer
    pub amount: f64,
    pub currency: String,
    pub category: String,
    pub description: String,
    pub transfer_to_account: Option<String>,
}

pub struct ImportHabitLog {
    pub habit: String,
    pub date: String,
    pub completed: bool,
}

pub struct ImportSummary {
    pub format: String,
    pub data_type: String,
    pub total_processed: usize,
    pub inserted: usize,
    pub skipped: usize,
    pub errors: usize,
    pub error_details: Vec<RowError>,
    pub skipped_reasons: Vec<String>,
}
```

---

## Reglas de negocio

| Aspecto | Regla |
|---------|-------|
| Cuentas | Lookup case-insensitive, trimmed |
| Categorias | Filtrar por type (expense/income) |
| Habitos | Lookup case-insensitive |
| Transfers | Requiere `transfer_to_account` |
| Categoria en transfer | Opcional; si viene vacia se asigna categoria del sistema al guardar |
| Habit completed | Solo insertar si `true` |
| Duplicados TX | date + account + transfer_to_account + currency + amount + type + description |
| Duplicados Habit | habit_id + date |
| Errores | Parcial (best effort), acumula `errors[]` |
| Límite archivo | 10MB |
| CSV parsing | Usar crate `csv` (manejo de comillas/commas) |
| Raw data | Mantener solo en memoria (nunca log) |

---

## Controller Methods
```rust
// src/controller/ingestion.rs
impl AppController {
    pub fn import_data(
        &self,
        content: String,
        filename: String,
    ) -> Result<ImportSummary, ControllerError>;

    pub fn max_import_file_size(&self) -> usize;
}
```

---

## Modificaciones requeridas

### src/controller/mod.rs
```rust
mod ingestion;

pub ingestion_service: IngestionService,
```

### src/features/mod.rs
```rust
pub mod ingestion;
```

---

## Verificación
```bash
nix develop -c cargo check
nix develop -c cargo clippy
nix develop -c cargo test
```
