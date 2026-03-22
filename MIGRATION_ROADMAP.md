# Roadmap: Migración Frontend Slint → Tauri + Svelte

## Contexto

| | Actual | Objetivo |
|---|---|---|
| Shell | Slint | Tauri |
| UI | Slint (.slint) | Svelte 5 + TypeScript |
| Gráficos | plotters (PNG estático) | Apache ECharts |
| Package mgr | — | Deno |
| IPC | callbacks Slint | Tauri commands (serde JSON) |

**Lo que NO cambia:** todo `src/features/`, `src/controller/`, `src/services/` (excepto charts.rs), `db/`, `locales/`.

## Documentación de referencia

Estos documentos mapean la UI y el IPC actuales. Son la fuente de verdad para las fases 2, 3 y 4:

- `docs/slint-ui-map.md` — mapa completo de las 6 páginas, 17 modales, componentes y patrones de UI
- `docs/slint-callback-inventory.md` — inventario de los 121 callbacks (IPC boundary), agrupados por dominio con inputs/outputs y notas de migración

---

## Fase 0 — Preparación (sin tocar código existente)

**Objetivo:** dejar el entorno listo antes de modificar nada.

- [ ] Verificar que `flake.nix` nuevo compila correctamente (`nix develop`)
- [ ] Decidir estructura de directorios: `src-tauri/` (backend Rust) + `ui-svelte/` (frontend)
- [ ] Definir convención de nombres para Tauri commands (snake_case)
- [ ] Listar todos los tipos de datos que cruzan la frontera IPC (modelos Slint actuales)

---

## Fase 1 — Scaffold Tauri

**Objetivo:** Tauri arranca con una pantalla en blanco. Rust compila. Nada roto aún.

- [ ] Agregar dependencia `tauri` a `Cargo.toml`, mantener `slint` temporalmente
- [ ] Crear `src-tauri/tauri.conf.json` con configuración base (sin CSP estricto aún)
- [ ] Adaptar `src/main.rs` para arrancar Tauri en lugar de Slint
- [ ] Crear `ui-svelte/` con scaffold Svelte 5 + Vite + Deno (`deno.json`)
- [ ] Verificar que `nix develop -c cargo check` pasa
- [ ] Verificar que el WebView carga la pantalla en blanco sin errores

---

## Fase 2 — Capa de datos IPC (la más crítica)

**Objetivo:** definir todos los DTOs serializables antes de escribir un solo comando.

Esta fase es la base de todo. Si se hace mal, hay que rehacer comandos y componentes Svelte.

- [ ] Crear `src/ui/dto/` — structs con `#[derive(Serialize, Deserialize)]` por dominio:
  - `dto/finance.rs` — cuentas, transacciones, categorías, metas
  - `dto/crypto.rs` — wallets, transacciones, portfolio, tax
  - `dto/habits.rs` — hábitos, rewards, heatmap data
  - `dto/dashboard.rs` — métricas, resúmenes
  - `dto/ingestion.rs` — preview, resultados, configuración
  - `dto/settings.rs` — configuración general
  - `dto/charts.rs` — series de datos para ECharts (reemplaza plotters)
- [ ] Cada DTO debe mapear 1:1 con lo que el frontend necesita (no exponer internals)
- [ ] Agregar `serde` + `serde_json` a `Cargo.toml` si no están

---

## Fase 3 — Tauri Commands por dominio

**Objetivo:** migrar `src/ui/callbacks/` a `src/ui/commands/` domain por domain.

Orden recomendado (de menor a mayor complejidad):

1. **Vault** (`vault.rs` — 166 líneas) — login, unlock, create vault
2. **Settings** (`settings.rs` — 411 líneas) — configuración general
3. **Dashboard** (`dashboard.rs` — 288 líneas) — métricas home
4. **Finance** (`finance.rs` — 810 líneas) — cuentas y transacciones
5. **Habits** (`habits/` — ~2200 líneas) — hábitos + rewards
6. **Ingestion** (`ingestion.rs` — 1040 líneas) — imports CSV + on-chain
7. **Crypto** (`crypto/` — ~4000 líneas) — lo más complejo, al final

Para cada dominio:
- [ ] Crear `src/ui/commands/{dominio}.rs` con funciones `#[tauri::command]`
- [ ] Registrar commands en `main.rs` (`invoke_handler`)
- [ ] Mantener callbacks Slint existentes intactos hasta que el dominio esté completamente migrado
- [ ] Testear cada command con `tauri-plugin-devtools` o logs antes de conectar UI

---

## Fase 4 — Frontend Svelte

**Objetivo:** construir la UI página por página, conectada a los commands reales.

Orden recomendado (mismo que fase 3):

1. **Login / Vault unlock**
2. **Settings**
3. **Dashboard**
4. **Finances** (página + modales)
5. **Habits** (página + modales + heatmap)
6. **Ingestion** (flujo completo: selección → preview → resultados)
7. **Crypto** (página + modales + tax)

Para cada página:
- [ ] Layout base con sidebar navegación
- [ ] Conectar a Tauri commands via `invoke()`
- [ ] Manejo de errores y estados de carga
- [ ] NO estilizar hasta tener funcionalidad completa

### Componentes compartidos (hacer primero):
- [ ] Sistema de design tokens (colores, espaciado, tipografía)
- [ ] Sidebar + navegación
- [ ] Componente de notificaciones (reemplaza `NotificationAdapter`)
- [ ] Modales base reutilizables
- [ ] Tabla genérica con virtualización (para listas largas)

---

## Fase 5 — Gráficos con ECharts

**Objetivo:** reemplazar plotters por ECharts en todos los gráficos.

- [ ] Instalar Apache ECharts via Deno
- [ ] Crear componente Svelte `<Chart>` wrapper genérico
- [ ] Migrar por tipo de gráfico:
  - [ ] Línea / área (portfolio value, finance balance)
  - [ ] Barras (ingresos vs gastos por mes)
  - [ ] Dona / pie (breakdown por categoría)
  - [ ] Heatmap (habits calendar)
  - [ ] Candlestick (crypto — si aplica)
- [ ] Eliminar `src/services/charts.rs` (plotters) cuando todos los gráficos estén migrados
- [ ] Verificar rendimiento con datasets grandes (>1000 puntos)

---

## Fase 6 — i18n

**Objetivo:** mantener las traducciones Fluent existentes, exponerlas al frontend.

Estrategia recomendada: **un solo command `get_translations()` que devuelve todas las strings como JSON map**. El frontend no necesita saber que son Fluent.

- [ ] Crear `src/ui/commands/translations.rs` — command que serializa todas las keys
- [ ] Crear store Svelte `translations.ts` con fallback al inglés
- [ ] Reemplazar usos de `Translations.*` en Slint por el store Svelte
- [ ] Mantener `locales/*.ftl` sin cambios — siguen siendo la fuente de verdad

---

## Fase 7 — Limpieza Slint

**Objetivo:** remover todo lo de Slint una vez que el frontend Svelte esté completo.

- [ ] Remover dependencia `slint` de `Cargo.toml`
- [ ] Eliminar `ui/` (directorio Slint completo)
- [ ] Eliminar `src/ui/callbacks/` (reemplazado por `src/ui/commands/`)
- [ ] Actualizar `AGENTS.md` — nueva estructura de directorios
- [ ] `cargo check` limpio sin warnings

---

## Fase 8 — Seguridad y hardening

**Objetivo:** configurar Tauri correctamente para una app de seguridad/privacidad.

- [ ] CSP estricto en `tauri.conf.json` (sin `unsafe-inline`, sin CDN externos)
- [ ] Allowlist de commands explícita (solo los registrados)
- [ ] Verificar que ningún asset se carga de internet en runtime
- [ ] Auditar IPC: ningún command expone datos sensibles innecesariamente
- [ ] `cargo audit` pasa sin vulnerabilidades conocidas
- [ ] Revisar permisos de Tauri (filesystem, network, etc.)

---

## Fase 9 — Polish UI

**Objetivo:** hacer que la UI se vea bien. Solo después de que todo funciona.

- [ ] Design system completo (tokens, dark mode)
- [ ] Animaciones y transiciones (solo las que aporten)
- [ ] Iconos SVG (migrar `ui/assets/icons/` al frontend Svelte)
- [ ] Crypto icons (migrar `ui/assets/crypto-icons/`)
- [ ] Responsive (si aplica para desktop)
- [ ] Accesibilidad básica (labels, focus, keyboard nav)

---

## Reglas durante la migración

1. **Ignorar el legacy de Tauri** — el proyecto era otro entonces, sin arquitectura feature-sliced ni los dominios actuales. Usarlo como referencia generaría errores y decisiones inconsistentes. La fuente de verdad es el código Slint actual.
2. **Una fase a la vez** — no empezar Fase 3 sin terminar Fase 2
2. **Slint funciona hasta el final** — no romper nada existente hasta Fase 7
3. **DTOs primero** — si un command necesita un tipo nuevo, definirlo en `dto/` antes de escribir el command
4. **Sin estilo hasta Fase 9** — funcionalidad antes que apariencia
5. **Un dominio completo** — terminar vault antes de empezar finance, etc.
6. **`cargo check` debe pasar siempre** — nunca commitear código que no compila

---

## Estimación de complejidad por fase

| Fase | Complejidad | Dependencias |
|---|---|---|
| 0 — Preparación | Baja | — |
| 1 — Scaffold Tauri | Media | Fase 0 |
| 2 — DTOs IPC | Alta | Fase 1 |
| 3 — Commands | Alta | Fase 2 |
| 4 — Frontend Svelte | Alta | Fase 3 |
| 5 — ECharts | Media | Fase 4 |
| 6 — i18n | Baja | Fase 4 |
| 7 — Limpieza Slint | Baja | Fases 3-6 completas |
| 8 — Seguridad | Media | Fase 7 |
| 9 — Polish | Media | Fase 8 |
