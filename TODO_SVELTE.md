# Svelte UI — Pendientes (vs Slint)

Lista de funcionalidades presentes en el frontend Slint que faltan en el frontend Svelte.
Marcar con [x] al completar.

---

## CryptoPage

- [x] **Add/Edit crypto transaction** — formulario completo para agregar/editar transacciones (buy, sell, swap, income, transfer, fee)
- [x] **Ticker bar config** — modal para seleccionar qué tickers aparecen en la barra, con reordenamiento ↑↓
- [x] **Coin catalog** — UI para browse/agregar coins custom/eliminar custom. Integrada como tab en la tuerca del ticker
- [x] **Refresh prices / last updated** — botón sync y display de última actualización
- [x] **IPC CSV import** — importar historial de precios. API: `importIpcCsv`, `getIpcSummary`
- [ ] **On-chain wallet import** — requiere backend (no hay Tauri commands para on-chain import)
- [x] **Tax: wallet exclusion list** — toggles para excluir wallets del cálculo de impuestos
- [ ] **Tax: sync missing prices** — no existe comando backend para esto (`syncTaxMissingPrices` no implementado)
- [x] **Tax: IPC summary display** — resumen de datos IPC en tab Tax
- [x] **Wallet icon edit** — UI para cambiar ícono de wallet desde panel de detalle

---

## FinancesPage

- [x] **Edit account** — modal para editar nombre, tipo, currency, balance inicial
- [x] **Account icon editing** — picker de íconos en panel de detalle. API: `updateAccountIcon`
- [x] **Edit transfer** — editar transferencia existente desde la lista de actividad. API: `updateTransfer`
- [~] **Account archive/unarchive** — `delete_account` hace soft-delete. Falta toggle para ver/restaurar cuentas archivadas (requiere investigar si hay comando de unarchive)

---

## HabitsPage

- [x] **Edit streak reward** — modal para editar reward existente
- [x] **Edit goal** — modal para editar goal incluyendo checkpoints
- [x] **Goal archive** — botón para archivar/completar un goal. API: `completeGoal`, `archiveGoal`
- [x] **Heatmap year navigation** — botones prev/next para navegar entre años

---

## SettingsPage

- [ ] **Login wallpaper** — requiere backend (Tauri command no expuesto)
- [ ] **On-chain custom endpoints** — requiere backend (no hay Tauri commands ni API)

---

## LoginPage

- [x] **Restore from backup** — link "Restore from backup" disponible en login

---

## Notas

- Items sin backend (on-chain import, login wallpaper, on-chain endpoints) quedan pendientes hasta que se implemente el backend
- Account archive/unarchive: verificar si existe `unarchive_account` command en el backend antes de implementar
