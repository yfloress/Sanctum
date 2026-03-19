# Frontend Migration Plan

## Current Stack
- Slint (native UI framework)
- plotters (chart rendering — images estáticas)

## Target Stack
- **Tauri** — shell nativo
- **Svelte 5 + TypeScript** — compila a vanilla JS, runtime mínimo
- **Vite** — build tool (default de Tauri)
- **Deno** — package manager sin node_modules, verificación de integridad nativa
- **Apache ECharts** — gráficos financieros de calidad

## Motivación
- Slint tiene techo bajo para UIs ricas y gráficos complejos
- plotters renderizando imágenes estáticas es el techo de Slint para charts
- Tauri fue el stack original del proyecto (existe branch legacy)
- Deno es coherente con la filosofía de seguridad/privacidad de Sanctum
- ECharts tiene excelente soporte para gráficos financieros (candlestick, área, línea)

## Lo que NO cambia
Todo el backend Rust permanece intacto:
- `src/features/` — toda la lógica de negocio
- `src/controller/` — orquestación
- `src/services/` — servicios transversales
- `db/` — migraciones y SQL
- `locales/` — i18n

## Lo que cambia
- `src/ui/callbacks/` → se reescribe como comandos Tauri (`#[tauri::command]`)
- `ui/` (Slint) → se reemplaza por frontend Svelte
- `src/main.rs` → bootstrap de Tauri en vez de Slint

## Cuándo migrar
**No antes de cerrar el MVP.** Primero terminar y pulir las features actuales,
luego migrar la capa UI desde una base estable.
