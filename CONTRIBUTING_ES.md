# CONTRIBUTING.md

> Guia tecnica para contribuidores de Sanctum.
>
> **[Read in English](CONTRIBUTING.md)**

Este documento establece las reglas de ingenieria para mantener el codigo
limpio, predecible y extensible. No es burocracia, es arquitectura.

---

## 1. Inicio Rapido

Antes de contribuir, asegurate de tener el entorno de desarrollo configurado.
Consulta las instrucciones detalladas en **[INSTALL.md](INSTALL.md)**.

### Cheatsheet de Comandos

| Accion               | Comando                             |
| :------------------- | :---------------------------------- |
| **Iniciar Entorno**  | `nix develop` (o configurar manual) |
| **Correr App (Dev)** | `cargo tauri dev`                   |
| **Linting (Rust)**   | `cargo clippy`                      |
| **Linting (TS)**     | `deno task check`                   |
| **Formatear Codigo** | `cargo fmt && deno fmt`             |
| **Compilar Release** | `cargo tauri build`                 |

---

## 2. Filosofia del Proyecto (Requisitos No Funcionales)

Cualquier cambio de codigo debe respetar estos tres pilares:

1. **Local-First:** La app debe ser 100% funcional sin internet. Los datos viven
   en el dispositivo del usuario.
2. **Privacidad por Diseno:** No hay telemetria, ni analiticas, ni "ping" a
   servidores externos (excepto CoinGecko bajo demanda explicita del usuario).
3. **Cero Dependencias Ocultas:** No usar librerias que requieran servidores
   propietarios (ej: Firebase).

---

## 3. Arquitectura del Proyecto

Sanctum sigue una arquitectura en capas con separación estricta de
responsabilidades:

```
┌─────────────────────────────────────────────────────────────────┐
│                        FRONTEND (React)                         │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────────────┐  │
│  │  Components │◄───│   Stores    │◄───│  Tauri IPC Bridge   │  │
│  │  (View)     │    │  (Zustand)  │    │  invoke()           │  │
│  └─────────────┘    └─────────────┘    └──────────┬──────────┘  │
└──────────────────────────────────────────────────│──────────────┘
                                                   │
┌──────────────────────────────────────────────────│──────────────┐
│                        BACKEND (Rust)            │              │
│  ┌─────────────┐    ┌─────────────┐    ┌────────▼──────────┐   │
│  │   db.rs     │◄───│  commands.rs│◄───│  #[tauri::command] │   │
│  │  (Model)    │    │ (Controller)│    │  (Entry Points)    │   │
│  └──────┬──────┘    └─────────────┘    └───────────────────┘   │
│         │                                                       │
│  ┌──────▼──────┐                                                │
│  │   SQLite    │                                                │
│  │  (SQLCipher)│                                                │
│  └─────────────┘                                                │
└─────────────────────────────────────────────────────────────────┘
```

### Flujo de Datos

1. **SQLite (SQLCipher)**: Almacenamiento encriptado. Única fuente de verdad.
2. **db.rs**: Capa de acceso a datos. CRUD, migraciones, queries.
3. **commands.rs**: Validación de entrada, sanitización, orquestación.
4. **Tauri IPC**: Puente entre Rust y JavaScript via `invoke()`.
5. **Zustand Stores**: Estado global del frontend. Contiene toda la lógica de
   negocio.
6. **React Components**: Renderizado puro. Sin lógica, solo presentación.

### Mapeo a MVC

| Capa       | Ubicación                   | Responsabilidad                         |
| :--------- | :-------------------------- | :-------------------------------------- |
| Model      | `src-tauri/src/db.rs`       | Persistencia, queries SQL, migraciones  |
| Controller | `src-tauri/src/commands.rs` | Validación, sanitización, coordinación  |
| ViewModel  | `src/stores/*.ts`           | Estado, lógica de negocio, llamadas IPC |
| View       | `src/features/**/*.tsx`     | Renderizado, eventos UI                 |

---

## 4. Estandares de Codigo

### Frontend (TypeScript/React)

**Regla Cardinal:** La logica de negocio va en los Stores, NO en los
componentes.

```typescript
// INCORRECTO: Logica en el componente
function TransactionsView() {
  const [transactions, setTransactions] = useState([]);

  useEffect(() => {
    invoke("get_transactions").then(setTransactions); // NO
  }, []);

  const handleDelete = async (id: string) => {
    await invoke("delete_transaction", { id }); // NO
    setTransactions((prev) => prev.filter((t) => t.id !== id));
  };
}

// CORRECTO: Componente consume el Store
function TransactionsView() {
  const transactions = useTransactions(); // Solo lee
  const { deleteTransaction } = useFinancialStore(); // Solo acciones

  return (
    <ul>
      {transactions.map((t) => (
        <li
          key={t.id}
          onClick={() => deleteTransaction(t.id)}
        >
          {t.description}
        </li>
      ))}
    </ul>
  );
}
```

**Otras reglas:**

- Usar selectores especificos para evitar re-renders innecesarios
- No usar `persist` middleware en Zustand (datos sensibles)
- Llamadas a `invoke()` solo dentro de los Stores
- Componentes en `features/` son vistas completas, en `components/` son
  reutilizables

### Backend (Rust)

**Regla Cardinal:** La base de datos es la unica fuente de verdad. Nunca cachear
estado derivado.

```rust
// INCORRECTO: Guardar estado derivado
struct AppState {
    balance: i64,  // NO - se desincroniza
}

// CORRECTO: Calcular siempre desde la DB
pub fn get_balance(conn: &Connection) -> Result<BalanceSummary, DbError> {
    // Siempre consulta la DB
}
```

**Seguridad obligatoria:**

```rust
// Contrasenas: SIEMPRE SecretString
use secrecy::{SecretString, ExposeSecret};

fn open_db(password: SecretString) -> Result<(), DbError> {
    let key = password.expose_secret();  // Solo en punto de uso
    // ...
}

// Entrada del usuario: SIEMPRE validar
fn add_transaction(amount: i64, category: &str) -> Result<(), DbError> {
    validate_amount(amount)?;
    let safe_category = sanitize_string(category, MAX_CATEGORY_LENGTH)?;
    // ...
}
```

**Otras reglas:**

- Prepared statements para todas las queries (anti SQL injection)
- Validacion en `commands.rs`, persistencia en `db.rs`
- Errores genericos al usuario, detallados en logs internos
- `PRAGMA foreign_keys = ON` siempre activo

### Gestion de Ventana Tauri

**Regla Cardinal:** La ventana principal inicia oculta y se muestra solo despues
de que React hidrate.

Esto previene el "flash blanco" al iniciar. El patron esta implementado en:

1. `tauri.conf.json`: Ventana configurada con `"visible": false`
2. `main.tsx`: Llama `getCurrentWindow().show()` via callback `onReady`
3. `App.tsx`: Dispara `onReady` en `useLayoutEffect` despues del primer render

```typescript
// main.tsx
import { getCurrentWindow } from "@tauri-apps/api/window";

const showWindow = () => {
  getCurrentWindow().show().catch(console.error);
};

// App.tsx
useLayoutEffect(() => {
  if (onReady) {
    onReady();
  }
}, [onReady]);
```

**Permiso requerido** en `src-tauri/capabilities/default.json`:

```json
"permissions": ["core:default", "core:window:allow-show", "opener:default"]
```

**NO hacer:**

- Configurar `"visible": true` en `tauri.conf.json`
- Llamar `show()` antes de que React haya montado
- Bloquear el hilo principal con operaciones sincronas durante el arranque

---

## 5. Patrones de Diseno Utilizados

### Observer Pattern (Zustand)

Los Stores implementan el patron Observer. Los componentes se suscriben a slices
del estado y React re-renderiza automaticamente cuando cambian.

```typescript
// El store es el Subject
const useFinancialStore = create<FinancialState>((set, get) => ({
  transactions: [],
  addTransaction: async (data) => {
    await invoke("add_transaction", data);
    set({ transactions: await invoke("get_transactions") });
  },
}));

// Los componentes son Observers
function TransactionList() {
  // Se suscribe solo a `transactions`
  const transactions = useFinancialStore(state => state.transactions);
  return <>{transactions.map(...)}</>;
}
```

### Command Pattern (Tauri IPC)

Cada operacion del backend se expone como un comando discreto. El frontend no
conoce la implementacion, solo el contrato.

```rust
// Backend: Define el comando
#[tauri::command]
pub fn add_transaction(amount: i64, category: String) -> Result<String, String> {
    // Implementacion encapsulada
}

// Frontend: Invoca el comando
await invoke("add_transaction", { amount: 1000, category: "Food" });
```

### Repository Pattern (db.rs)

`db.rs` actua como repositorio. Abstrae SQLite detras de funciones de dominio.

```rust
// El resto del codigo no sabe que usamos SQL
pub fn get_transactions(conn: &Connection) -> Result<Vec<Transaction>, DbError>;
pub fn add_wallet(conn: &Connection, wallet: &CryptoWallet) -> Result<(), DbError>;
```

---

## 6. Estructura de Directorios

```
src/
├── features/           # Vistas por funcionalidad (una carpeta = una seccion de la app)
│   ├── auth/           #   LoginScreen.tsx
│   ├── dashboard/      #   Dashboard.tsx
│   ├── transactions/   #   TransactionsView.tsx
│   ├── crypto/         #   CryptoView.tsx
│   └── habits/         #   HabitsView.tsx
│
├── stores/             # Estado global (Zustand)
│   ├── authStore.ts    #   Autenticacion, vault, kill switch
│   ├── financialStore.ts   # Transacciones FIAT
│   ├── cryptoStore.ts  #   Wallets, portfolio, precios
│   ├── habitStore.ts   #   Habitos y logs
│   └── index.ts        #   Re-exportaciones
│
├── components/         # Componentes reutilizables (sin logica de negocio)
│   ├── layout/         #   Sidebar, Header
│   ├── modals/         #   DeleteConfirmModal
│   └── ui/             #   Toast, Button, Input
│
├── types/              # Interfaces TypeScript
│   └── index.ts        #   Transaction, CryptoWallet, etc.
│
└── utils/              # Funciones puras (sin side effects)
    └── index.ts        #   formatAmount, formatDate, etc.

src-tauri/src/
├── lib.rs              # Entry point, registro de comandos
├── main.rs             # Bootstrap de Tauri
├── commands.rs         # Comandos IPC (validacion, orquestacion)
├── db.rs               # Acceso a datos, migraciones, SQLCipher
├── models.rs           # Structs y Enums de dominio
├── crypto.rs           # Cliente HTTP para CoinGecko
└── security_log.rs     # Logging de eventos de seguridad
```

### Donde va cada cosa

| Necesitas...             | Ubicacion                         |
| :----------------------- | :-------------------------------- |
| Nueva pantalla/seccion   | `src/features/nueva-feature/`     |
| Estado global compartido | `src/stores/nuevoStore.ts`        |
| Componente reutilizable  | `src/components/categoria/`       |
| Tipo/Interface           | `src/types/index.ts`              |
| Funcion helper pura      | `src/utils/index.ts`              |
| Nueva tabla SQL          | `src-tauri/src/db.rs` (migracion) |
| Nuevo endpoint IPC       | `src-tauri/src/commands.rs`       |
| Struct de datos          | `src-tauri/src/models.rs`         |

> **Nota:** El proyecto NO usa una carpeta `hooks/`. Toda la logica de estado
> esta en los Zustand stores.

---

## 7. Flujo de Trabajo para Nuevas Features

Para agregar una nueva funcionalidad (ejemplo: modulo de Metas Financieras):

### Paso 1: Backend - Modelo de Datos

Edita `src-tauri/src/db.rs`:

```rust
// 1. Agrega la migracion
fn run_migrations(conn: &Connection) -> Result<(), DbError> {
    // ... migraciones existentes ...
    
    // Nueva migracion
    conn.execute(
        "CREATE TABLE IF NOT EXISTS goals (
            id TEXT PRIMARY KEY NOT NULL,
            name TEXT NOT NULL,
            target_amount INTEGER NOT NULL,
            current_amount INTEGER NOT NULL DEFAULT 0,
            deadline TEXT,
            created_at TEXT NOT NULL
        )",
        [],
    )?;
    
    Ok(())
}

// 2. Agrega funciones CRUD
pub fn add_goal(conn: &Connection, goal: &Goal) -> Result<(), DbError> { ... }
pub fn get_goals(conn: &Connection) -> Result<Vec<Goal>, DbError> { ... }
pub fn update_goal(conn: &Connection, id: &str, amount: i64) -> Result<(), DbError> { ... }
pub fn delete_goal(conn: &Connection, id: &str) -> Result<(), DbError> { ... }
```

### Paso 2: Backend - Modelo de Dominio

Edita `src-tauri/src/models.rs`:

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct Goal {
    pub id: String,
    pub name: String,
    pub target_amount: i64,
    pub current_amount: i64,
    pub deadline: Option<String>,
    pub created_at: String,
}
```

### Paso 3: Backend - Comandos IPC

Edita `src-tauri/src/commands.rs`:

```rust
#[tauri::command]
pub fn add_goal(name: String, target_amount: i64, deadline: Option<String>) -> Result<String, String> {
    // Validacion
    let name = sanitize_string(&name, MAX_NAME_LENGTH)?;
    validate_amount(target_amount)?;
    
    // Persistencia
    let goal = Goal { id: uuid(), name, target_amount, ... };
    db::add_goal(&get_connection()?, &goal)?;
    
    Ok(goal.id)
}

// ... otros comandos ...
```

Registra en `src-tauri/src/lib.rs`:

```rust
.invoke_handler(tauri::generate_handler![
    // ... comandos existentes ...
    commands::add_goal,
    commands::get_goals,
    commands::update_goal,
    commands::delete_goal,
])
```

### Paso 4: Frontend - Tipos

Edita `src/types/index.ts`:

```typescript
export interface Goal {
  id: string;
  name: string;
  target_amount: number;
  current_amount: number;
  deadline: string | null;
  created_at: string;
}
```

### Paso 5: Frontend - Store

Crea `src/stores/goalStore.ts`:

```typescript
import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import type { Goal } from "../types";

interface GoalState {
  goals: Goal[];
  isLoading: boolean;
  error: string | null;
  successMessage: string | null;

  loadGoals: () => Promise<void>;
  addGoal: (name: string, targetAmount: number) => Promise<boolean>;
  // ... otras acciones ...

  // Mensajes
  setError: (error: string | null) => void;
  setSuccess: (message: string | null) => void;
  clearMessages: () => void;

  // Seguridad: Limpieza de RAM
  reset: () => void;
}

export const useGoalStore = create<GoalState>((set, get) => ({
  goals: [],
  isLoading: false,
  error: null,
  successMessage: null,

  loadGoals: async () => {
    set({ isLoading: true });
    try {
      const goals = await invoke<Goal[]>("get_goals");
      set({ goals, error: null });
    } catch (e) {
      set({ error: String(e) });
    } finally {
      set({ isLoading: false });
    }
  },

  addGoal: async (name, targetAmount) => {
    set({ isLoading: true, error: null });
    try {
      await invoke("add_goal", { name, targetAmount });
      await get().loadGoals();
      set({ successMessage: "Goal created successfully" });
      return true;
    } catch (e) {
      set({ error: String(e) });
      return false;
    } finally {
      set({ isLoading: false });
    }
  },

  setError: (error) => set({ error }),
  setSuccess: (message) => set({ successMessage: message }),
  clearMessages: () => set({ error: null, successMessage: null }),

  // IMPORTANTE: Limpiar datos sensibles de RAM al cerrar vault
  reset: () =>
    set({
      goals: [],
      isLoading: false,
      error: null,
      successMessage: null,
    }),
}));

// Selectores optimizados (evitan re-renders innecesarios)
export const useGoals = () => useGoalStore((s) => s.goals);
export const useGoalLoading = () => useGoalStore((s) => s.isLoading);
export const useGoalError = () => useGoalStore((s) => s.error);
export const useGoalSuccess = () => useGoalStore((s) => s.successMessage);
```

### Paso 6: Frontend - Vista

Crea `src/features/goals/GoalsView.tsx`:

```typescript
import { useEffect } from "react";
import { useGoalLoading, useGoals, useGoalStore } from "../../stores/goalStore";

export function GoalsView() {
  // Selectores especificos (optimiza re-renders)
  const goals = useGoals();
  const isLoading = useGoalLoading();

  // Acciones del store
  const loadGoals = useGoalStore((s) => s.loadGoals);
  const addGoal = useGoalStore((s) => s.addGoal);

  useEffect(() => {
    loadGoals();
  }, [loadGoals]);

  if (isLoading) {
    return <div className="loader">Loading goals...</div>;
  }

  return (
    <div className="goals-view">
      <h1>Financial Goals</h1>
      <div className="goals-grid">
        {goals.map((goal) => <GoalCard key={goal.id} goal={goal} />)}
      </div>
    </div>
  );
}
```

### Paso 7: Integracion

1. Agrega el tab en `src/App.tsx`:
   - Importa `GoalsView` desde `./features/goals/GoalsView`
   - Agrega el case en el render condicional de tabs

2. Agrega el item en `src/components/layout/Sidebar.tsx`:
   - Agrega el boton con el icono y label correspondiente

3. Agrega el store al kill switch en `src/stores/authStore.ts`:
   ```typescript
   // En _clearAllStores:
   useGoalStore.getState().reset();
   ```

4. Exporta el store en `src/stores/index.ts`:
   ```typescript
   export {
     useGoalLoading,
     useGoals,
     useGoalStore,
     // ...
   } from "./goalStore";
   ```

5. Agrega el tipo de tab en `src/types/index.ts`:
   ```typescript
   export type TabType = "dashboard" | "transactions" | ... | "goals";
   ```

---

## 8. Checklist de Pull Request

Antes de enviar un PR, verifica:

**Backend (Rust):**

- [ ] `cargo clippy` pasa sin warnings
- [ ] `cargo test` pasa (si hay tests)
- [ ] Datos sensibles usan `SecretString`
- [ ] Nuevos comandos tienen validacion de entrada en `commands.rs`
- [ ] Nuevos comandos estan registrados en `lib.rs`

**Frontend (TypeScript):**

- [ ] `deno task check` pasa (TypeScript)
- [ ] No hay `console.log` en el codigo final (excepto en desarrollo)
- [ ] Nuevos stores tienen metodo `reset()` para el kill switch
- [ ] Store exportado en `stores/index.ts`
- [ ] Selectores especificos para evitar re-renders

**General:**

- [ ] No hay secrets hardcodeados
- [ ] La documentacion en `ARCHITECT.md` esta actualizada si cambia la
      estructura
- [ ] Tipos sincronizados entre Rust (`models.rs`) y TypeScript
      (`types/index.ts`)

---

## 9. Convenciones de Commits

Seguimos Conventional Commits:

```
<type>(<scope>): <description>

feat(crypto): add swap transaction support
fix(auth): handle rate limit edge case
refactor(stores): migrate from hooks to Zustand
docs(contributing): add development workflow
```

Tipos validos: `feat`, `fix`, `refactor`, `docs`, `test`, `chore`, `perf`,
`security`

---

## 10. Contacto

Si tienes dudas sobre la arquitectura antes de empezar a codificar, abre un
Issue con el tag `question`. Es mejor preguntar que reescribir.
