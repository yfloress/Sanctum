# Svelte UI — Pendientes (vs Slint)

Lista de funcionalidades presentes en el frontend Slint que faltan en el frontend Svelte.
Marcar con [x] al completar.

---

## CryptoPage

- [x] **Add/Edit crypto transaction** — formulario completo para agregar/editar transacciones (buy, sell, swap, income, transfer, fee). Campos: tipo, coin, wallet, cantidad, precio, fee+fee coin, fecha, notas, subtipo, override proceeds/cost basis. API: `addCryptoTransaction`, `updateCryptoTransaction`, `getCryptoTransaction`
- [x] **Ticker bar config** — modal para seleccionar qué tickers aparecen en la barra. API: `getActiveTickerIds`, `saveActiveTickerIds`
- [x] **Coin catalog** — UI para browse/toggle/agregar coins custom/eliminar custom/marcar favorito. API: `getCoinCatalog`, `addCustomCoin`, `deleteCustomCoin`, `setFavoriteCoin`
- [x] **Refresh prices / last updated** — botón "sync prices" y display de cuándo fue la última actualización de precios
- [x] **IPC CSV import** — importar historial de precios desde exchange. API: `importIpcCsv`, `getIpcSummary`
- [ ] **On-chain wallet import** — modal para ingresar address, auto-detección de red, preview y confirmar. NOTA: requiere backend (no hay Tauri commands para on-chain import)
- [x] **Tax: wallet exclusion list** — toggles para excluir wallets específicas del cálculo de impuestos. Actualmente siempre envía `excluded_wallet_ids: []`
- [ ] **Tax: sync missing prices** — botón para resolver precios históricos faltantes. API: `syncTaxMissingPrices`
- [x] **Tax: IPC summary display** — mostrar resumen de datos IPC importados en el tab Tax
- [x] **Wallet icon edit** — UI para cambiar el ícono de una wallet (renombrar + editar via panel de detalle)

---

## FinancesPage

- [x] **Edit account** — modal para editar nombre, tipo, currency, balance inicial de una cuenta existente (actualmente solo se puede eliminar)
- [ ] **Account icon editing** — UI para cambiar el ícono de una cuenta. API: `updateAccountIcon`
- [ ] **Edit transfer** — poder editar una transferencia existente. API: `updateTransfer`
- [~] **Account archive/unarchive** — `delete_account` ya llama `archive_account` en el backend (soft-delete). Falta un toggle para des-archivar si se quiere mostrar/ocultar cuentas archivadas

---

## HabitsPage

- [x] **Edit streak reward** — modal para editar un reward existente (actualmente solo se puede eliminar)
- [x] **Edit goal** — modal para editar un goal existente incluyendo checkpoints (actualmente solo se puede eliminar)
- [x] **Goal archive** — botón para archivar/completar un goal y moverlo al historial. API: `completeGoal`, `archiveGoal`
- [x] **Heatmap year navigation** — botones prev/next para navegar entre años en el heatmap (actualmente solo muestra el año actual)

---

## SettingsPage

- [ ] **Login wallpaper** — selector de imagen para el fondo de la pantalla de login + botón reset. NOTA: requiere Tauri command para set_login_wallpaper (solo existe en controller, no expuesto como command)
- [ ] **On-chain custom endpoints** — sección para configurar RPC URLs por chain (BTC, LTC, Polygon, ETH, Arbitrum, Base, BSC, Solana, Tron, Nano) con toggle para habilitar/deshabilitar. NOTA: requiere backend (no hay Tauri commands ni API para esto aun)

---

## LoginPage

- [x] **Restore from backup** — link/botón "Restore from backup" en la pantalla de login cuando ya existe un vault (en Slint estaba disponible desde login, en Svelte solo desde Settings)

---

## Notas

- Todas las APIs necesarias ya existen en `/app/ui-svelte/src/lib/api/`
- Prioridad sugerida: Edit account (Finances) → Add/Edit crypto transaction → Edit reward/goal (Habits) → On-chain import → Ticker config → Resto
