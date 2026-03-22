# Frontend Migration Plan

## Current Stack
- Slint (native UI framework)
- plotters (chart rendering — images estáticas)

## Target Stack
- **Tauri** — shell nativo
- **Svelte 5 + TypeScript** — compila a vanilla JS, runtime mínimo
- **Vite** — build tool (default de Tauri)
- **pnpm** — package manager con lockfile estricto, sin phantom dependencies, `ignore-scripts=true` por defecto (`.npmrc`)
- **Apache ECharts** — gráficos financieros de calidad

## Motivación
- Slint tiene techo bajo para UIs ricas y gráficos complejos
- plotters renderizando imágenes estáticas es el techo de Slint para charts
- Tauri fue el stack original del proyecto (existe branch legacy)
- pnpm con `ignore-scripts=true` mitiga riesgos de supply chain (post-install scripts maliciosos)
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
