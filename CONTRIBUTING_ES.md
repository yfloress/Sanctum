# CONTRIBUTING.md

> Guía técnica para contribuidores de Sanctum.
>
> **[Read in English](CONTRIBUTING.md)**

---

## 1. Inicio Rápido

```bash
# Entrar al entorno de desarrollo
nix develop  # o direnv allow

# Ejecutar en desarrollo
cargo tauri dev

# Linting
cargo clippy && deno task check

# Formatear
cargo fmt && deno fmt

# Build de producción
cargo tauri build
```

---

## 2. Filosofía del Proyecto

1. **Local-First:** 100% funcional sin internet. Los datos viven en el dispositivo.
2. **Privacidad por Diseño:** Sin telemetría ni servidores externos (excepto CoinGecko bajo demanda).
3. **Cero Dependencias Ocultas:** Sin librerías que requieran servidores propietarios.

---

## 3. Arquitectura

### Sistema de Monedas
- **Moneda base:** USD (todos los cálculos de patrimonio)
- **Soportadas:** USD, CLP (Peso Chileno)
- **Tasa de cambio:** Obtenida de CoinGecko, cacheada en SQLCipher para uso offline

```
┌─────────────────────────────────────────────────────────────────┐
│                        FRONTEND (React)                         │
│  Components (View) ◄── Stores (Zustand) ◄── Tauri IPC Bridge    │
└──────────────────────────────────────────────────────────────────┘
                                   │
┌──────────────────────────────────│───────────────────────────────┐
│                        BACKEND (Rust)                            │
│  db.rs (Model) ◄── commands.rs (Controller) ◄── #[tauri::command]│
│       │                                                          │
│  SQLite + SQLCipher (Encrypted)                                  │
└──────────────────────────────────────────────────────────────────┘
```

### Mapeo a MVC

| Capa       | Ubicación             | Responsabilidad                        |
| :--------- | :-------------------- | :------------------------------------- |
| Model      | `src-tauri/src/db.rs` | Persistencia, SQL, migraciones         |
| Controller | `commands.rs`         | Validación, sanitización, coordinación |
| ViewModel  | `src/stores/*.ts`     | Estado, lógica de negocio, IPC         |
| View       | `src/features/**`     | Renderizado puro, eventos UI           |

---

## 4. Estructura de Directorios

```
src/
├── features/           # Vistas por funcionalidad
│   ├── accounts/       #   Cuentas FIAT
│   ├── analytics/      #   Reportes y gráficos
│   ├── auth/           #   Login, creación de vault
│   ├── crypto/         #   Portfolio crypto, wallets
│   ├── dashboard/      #   Vista principal
│   ├── habits/         #   Tracker de hábitos
│   └── transactions/   #   Transacciones FIAT
│
├── stores/             # Estado global (Zustand)
│   ├── accountStore.ts #   Cuentas FIAT y balances
│   ├── authStore.ts    #   Autenticación, vault, kill switch
│   ├── cryptoStore.ts  #   Wallets, portfolio, precios
│   ├── financialStore.ts   # Transacciones FIAT
│   ├── habitStore.ts   #   Hábitos y logs
│   └── toastStore.ts   #   Notificaciones UI
│
├── components/         # Componentes reutilizables
├── types/              # Interfaces TypeScript
└── utils/              # Funciones puras

src-tauri/src/
├── lib.rs              # Entry point, registro de comandos
├── commands.rs         # Comandos IPC (validación)
├── db.rs               # Acceso a datos, migraciones, SQLCipher
├── models.rs           # Structs de dominio
├── crypto.rs           # Cliente HTTP para CoinGecko + tasas de cambio
└── security_log.rs     # Logging de seguridad
```

---

## 5. Estándares de Código

### Frontend

**Regla Cardinal:** La lógica de negocio va en los Stores, NO en los componentes.

```typescript
//  CORRECTO
function TransactionsView() {
  const transactions = useTransactions(); // Solo lee del store
  const { deleteTransaction } = useFinancialStore();
  // Componente solo renderiza
}

// ❌ INCORRECTO
function TransactionsView() {
  const [transactions, setTransactions] = useState([]);
  useEffect(() => {
    invoke("get_transactions").then(setTransactions); // NO
  }, []);
}
```

**Reglas:**
- Usar selectores específicos para evitar re-renders
- NO usar `persist` middleware (datos sensibles)
- Llamadas a `invoke()` solo dentro de Stores
- Cada store debe tener método `reset()` para el kill switch

### Backend

**Regla Cardinal:** La DB es la única fuente de verdad. Nunca cachear estado.

```rust
// Contraseñas: SIEMPRE SecretString
fn open_db(password: SecretString) -> Result<(), DbError> {
    let key = password.expose_secret();
}

// Entrada del usuario: SIEMPRE validar
fn add_transaction(amount: i64, category: &str) -> Result<(), DbError> {
    validate_amount(amount)?;
    let safe = sanitize_string(category, MAX_LENGTH)?;
}
```

**Reglas:**
- Prepared statements para todas las queries
- Validación en `commands.rs`, persistencia en `db.rs`
- Errores genéricos al usuario, detallados en logs internos

---

## 6. Seguridad SQLCipher

La base de datos usa SQLCipher con parámetros endurecidos:

- **Cipher:** AES-256-CBC
- **KDF:** PBKDF2-HMAC-SHA512 con 600,000 iteraciones
- **HMAC:** HMAC-SHA512
- **Memory Security:** Habilitado

**IMPORTANTE:** Los parámetros de cifrado deben aplicarse en CADA apertura de la DB,
no solo en la creación. Ver `apply_sqlcipher_hardening()` en `db.rs`.

---

## 7. Agregar Nueva Feature

1. **Backend - Migración:** `db.rs` → `run_migrations()`
2. **Backend - Modelo:** `models.rs` → struct con Serialize/Deserialize
3. **Backend - Comando:** `commands.rs` → validación + llamada a db
4. **Backend - Registro:** `lib.rs` → agregar a `invoke_handler!`
5. **Frontend - Tipos:** `types/index.ts` → interface TypeScript
6. **Frontend - Store:** `stores/` → nuevo store con `reset()`
7. **Frontend - Vista:** `features/` → componente que consume store
8. **Integración:** Agregar al Sidebar, App.tsx, y kill switch en authStore

---

## 8. Checklist de PR

**Backend:**
- [ ] `cargo clippy` sin warnings
- [ ] Datos sensibles usan `SecretString`
- [ ] Comandos validados en `commands.rs`
- [ ] Comandos registrados en `lib.rs`

**Frontend:**
- [ ] `deno task check` pasa
- [ ] Sin `console.log` (solo `console.error` para errores)
- [ ] Store tiene `reset()` para kill switch
- [ ] Store exportado en `stores/index.ts`

**General:**
- [ ] Sin secrets hardcodeados
- [ ] Tipos sincronizados entre Rust y TypeScript

---

## 9. Commits

Seguimos Conventional Commits:

```
feat(crypto): add swap transaction support
fix(auth): handle rate limit edge case
refactor(stores): migrate from hooks to Zustand
docs: update contributing guide
```

---

## 10. Contacto

Dudas antes de codificar → abre un Issue con tag `question`.
