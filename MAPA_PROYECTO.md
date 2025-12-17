# Mapa del Proyecto Sanctum

---
## Cargo.toml
**Tipo:** Config
**Resumen:** Archivo de configuración del proyecto Rust, define dependencias y metadatos.
**Estructura Clave:**
- `[dependencies]`: Slint, Tokio, Rusqlite, Serde, Reqwest, Secrecy, etc.
**Dependencias:** N/A
---
## src/main.rs
**Tipo:** Rust Backend
**Resumen:** Punto de entrada de la aplicación, inicializa el UI de Slint, el controlador y conecta los callbacks de la interfaz con la lógica de negocio.
**Estructura Clave:**
- `main`: Función principal que inicia el logger, controlador y UI.
- `reload_accounts`, `reload_transactions`, `reload_portfolio`: Funciones helper para actualizar modelos UI.
- Conexiones de Callbacks: `AuthAdapter`, `AccountAdapter`, `TransactionAdapter`, `CryptoAdapter`, `HabitAdapter`.
**Dependencias:** `sanctum::controller`, `sanctum::models`, `sanctum::security_log`
---
## src/lib.rs
**Tipo:** Rust Backend
**Resumen:** Define la estructura de módulos de la librería.
**Estructura Clave:**
- Módulos públicos: `controller`, `crypto`, `db`, `models`, `security_log`, `services`.
**Dependencias:** N/A
---
## src/db.rs
**Tipo:** Rust Backend
**Resumen:** Capa de acceso a datos, maneja la conexión SQLite cifrada (SQLCipher), migraciones, seguridad y caché.
**Estructura Clave:**
- `struct Database`: Envoltorio de conexión `rusqlite`.
- `init`: Inicialización y configuración de cifrado (SQLCipher).
- `apply_sqlcipher_hardening`: Configuración de seguridad (PBKDF2, HMAC).
- **Seguridad:** `check_rate_limit`, `record_failed_attempt` (Protección fuerza bruta).
- **Finanzas:** `create_account`, `get_accounts`, `create_transaction`, `get_balance_summary`.
- **Cripto:** `create_wallet`, `add_crypto_transaction`, `migrate_crypto_ledger`, `get_aggregated_portfolio`.
- **Caché:** `save_exchange_rate`, `load_crypto_prices` (Soporte offline).
- **Migraciones:** `run_migrations`, `migrate_habits_tables`.
**Dependencias:** `rusqlite`, `secrecy`, `uuid`, `chrono`
---
## src/models.rs
**Tipo:** Rust Backend
**Resumen:** Define las estructuras de datos fundamentales compartidas entre la DB y la lógica.
**Estructura Clave:**
- `struct Account`, `struct AccountBalance`: Cuentas fiat y saldos calculados.
- `struct Transaction`, `struct BalanceSummary`: Transacciones y resumen financiero.
- `struct CryptoAsset`, `struct AggregatedAsset`: Datos de mercado y portafolio calculado.
- `struct CryptoTransaction`, `struct CryptoWallet`: Ledger cripto.
- `struct Habit`, `struct HabitLog`: Seguimiento de hábitos.
**Dependencias:** `serde`
---
## src/controller.rs
**Tipo:** Rust Backend
**Resumen:** Orquestador de lógica de negocio, valida entradas, gestiona sesiones y coordina DB y Servicios.
**Estructura Clave:**
- `struct AppController`: Controlador principal.
- `create_db`, `open_db`: Gestión de la bóveda cifrada.
- `get_analytics_summary`, `get_net_worth_history`: Lógica de análisis financiero.
- `get_aggregated_portfolio`: Cálculo de portafolio cripto.
- `check_persistent_rate_limit`: Verificación de seguridad persistente.
**Dependencias:** `crate::db`, `crate::models`, `crate::crypto`
---
## src/crypto.rs
**Tipo:** Rust Backend
**Resumen:** Cliente API para obtener precios de criptomonedas (CoinGecko).
**Estructura Clave:**
- `fetch_crypto_prices`: Obtiene precios actuales.
- `fetch_clp_usd_rate`: Obtiene tasa de cambio.
- `validate_coin_id`: Validación de seguridad.
**Dependencias:** `reqwest`, `serde`
---
## src/security_log.rs
**Tipo:** Rust Backend
**Resumen:** Sistema de registro de eventos de seguridad y auditoría.
**Estructura Clave:**
- `enum SecurityEvent`: Tipos de eventos (Login, Bloqueo, etc.).
- `log_security_event`: Función de registro.
**Dependencias:** `log`, `chrono`
---
## src/services/mod.rs
**Tipo:** Rust Backend
**Resumen:** Módulo raíz de servicios, expone sub-módulos.
**Estructura Clave:**
- `pub mod habit;`
**Dependencias:** N/A
---
## src/services/habit.rs
**Tipo:** Rust Backend
**Resumen:** Lógica específica para la gestión de hábitos.
**Estructura Clave:**
- `struct HabitService`: Servicio de hábitos.
- `create_habit`, `toggle_habit_completion`: Lógica de negocio de hábitos.
**Dependencias:** `crate::db`
---
## ui/app.slint
**Tipo:** Slint UI
**Resumen:** Ventana principal de la aplicación, gestiona la navegación global y modales.
**Estructura Clave:**
- `export component AppWindow`: Ventana raíz.
- Lógica condicional (`if`) para mostrar Login o Dashboard.
- Inclusión de Modales (`AddTransactionModal`, etc.).
**Dependencias:** `globals.slint`, `pages/*`, `modals/*`, `components/*`
---
## ui/globals.slint
**Tipo:** Slint UI
**Resumen:** Define estilos globales (Paleta), adaptadores de datos y estado global.
**Estructura Clave:**
- `global Palette`: Colores y tipografía.
- `global AppState`: Estado de navegación y sesión.
- `global AuthAdapter`: Callbacks (`unlock-vault`, `create-vault`).
- `global AccountAdapter`, `TransactionAdapter`, `CryptoAdapter`, `HabitAdapter`: Estructuras de datos y callbacks para comunicación con Rust.
**Dependencias:** N/A
---
## ui/widgets.slint
**Tipo:** Slint UI
**Resumen:** Componentes UI atómicos y reutilizables.
**Estructura Clave:**
- `export component UnderlineInput`: Campo de texto minimalista.
- `export component TextButton`, `BorderButton`: Botones.
- `export component SanctumCard`: Contenedor genérico.
**Dependencias:** `globals.slint`
---
## ui/components/account_item.slint
**Tipo:** Slint UI
**Resumen:** Representación visual de una fila de cuenta bancaria.
**Estructura Clave:**
- `export component AccountItem`: Muestra icono, nombre y saldo.
- `callback clicked()`: Evento al seleccionar.
**Dependencias:** `globals.slint`
---
## ui/components/analytics_charts.slint
**Tipo:** Slint UI
**Resumen:** Gráficos especializados para estadísticas de hábitos.
**Estructura Clave:**
- `export component DayEfficiencyChart`: Gráfico de barras para eficiencia diaria (últimos 90 días).
- `export component MonthEfficiencyChart`: Gráfico de línea para tendencia mensual.
**Dependencias:** `globals.slint`
---
## ui/components/asset_detail.slint
**Tipo:** Slint UI
**Resumen:** Panel lateral deslizable con detalles de un activo cripto.
**Estructura Clave:**
- `export component AssetDetailPanel`: Muestra desglose por wallet e historial.
- Conecta directamente con `CryptoAdapter`.
**Dependencias:** `globals.slint`
---
## ui/components/category_breakdown.slint
**Tipo:** Slint UI
**Resumen:** Lista visual de gastos por categoría con barras de progreso.
**Estructura Clave:**
- `export component CategoryBreakdown`: Itera sobre `CategoryData`.
**Dependencias:** `globals.slint`
---
## ui/components/charts.slint
**Tipo:** Slint UI
**Resumen:** Gráfico de línea principal utilizado en la aplicación.
**Estructura Clave:**
- `export component SanctumLineChart`: Gráfico con relleno degradado y ejes. Usado en `DashboardPage`.
**Dependencias:** `globals.slint`
---
## ui/components/crypto_widgets.slint
**Tipo:** Slint UI
**Resumen:** Colección de widgets específicos para la vista cripto.
**Estructura Clave:**
- `export component CryptoRow`: Fila de tabla de activos.
- `export component WalletRow`: Fila de lista de wallets.
- `export component CryptoCard`: Tarjeta de resumen de activo.
**Dependencias:** `globals.slint`
---
## ui/components/habit_heatmap.slint
**Tipo:** Slint UI
**Resumen:** Mapa de calor tipo GitHub para consistencia de hábitos.
**Estructura Clave:**
- `export component HabitHeatmap`: Renderiza cuadrícula de días coloreados por intensidad.
**Dependencias:** `globals.slint`
---
## ui/components/habit_row.slint
**Tipo:** Slint UI
**Resumen:** Fila interactiva para un hábito individual y su seguimiento mensual.
**Estructura Clave:**
- `export component HabitRow`: Muestra nombre, racha y días del mes interactivos.
- `callback toggle(string)`: Evento al marcar un día.
**Dependencias:** `globals.slint`
---
## ui/components/notification.slint
**Tipo:** Slint UI
**Resumen:** Componente de notificación flotante (Toast).
**Estructura Clave:**
- `export component NotificationToast`: Muestra mensajes de error o éxito temporalmente.
**Dependencias:** `globals.slint`
---
## ui/components/sidebar.slint
**Tipo:** Slint UI
**Resumen:** Barra de navegación lateral principal.
**Estructura Clave:**
- `export component SanctumSidebar`: Contiene botones de navegación (`Dashboard`, `Finances`, etc.) y botón de bloqueo.
**Dependencias:** `globals.slint`, `components/tactical_grid.slint`
---
## ui/components/tactical_grid.slint
**Tipo:** Slint UI
**Resumen:** Fondo decorativo con rejilla sutil.
**Estructura Clave:**
- `export component TacticalGrid`: Fondo estético.
**Dependencias:** `globals.slint`
---
## ui/components/transaction_item.slint
**Tipo:** Slint UI
**Resumen:** Fila de lista para una transacción financiera.
**Estructura Clave:**
- `export component TransactionItem`: Muestra fecha, categoría, descripción y monto.
**Dependencias:** `globals.slint`
---
## ui/modals/add_account.slint
**Tipo:** Slint UI
**Resumen:** Formulario modal para crear o editar cuentas bancarias.
**Estructura Clave:**
- `export component AddAccountModal`: Campos para nombre, tipo, divisa y saldo inicial.
- Llama a `AccountAdapter.create-account`.
**Dependencias:** `globals.slint`
---
## ui/modals/add_crypto_transaction.slint
**Tipo:** Slint UI
**Resumen:** Formulario modal para registrar operaciones cripto manuales.
**Estructura Clave:**
- `export component AddCryptoTransactionModal`: Selectores de moneda, wallet y campos numéricos.
- Llama a `CryptoAdapter.add-transaction`.
**Dependencias:** `globals.slint`
---
## ui/modals/add_crypto_wallet.slint
**Tipo:** Slint UI
**Resumen:** Formulario modal para crear wallets (portafolios) cripto.
**Estructura Clave:**
- `export component AddCryptoWalletModal`: Nombre y categoría de wallet.
- Llama a `CryptoAdapter.create-wallet`.
**Dependencias:** `globals.slint`
---
## ui/modals/add_habit.slint
**Tipo:** Slint UI
**Resumen:** Formulario modal para crear nuevos hábitos.
**Estructura Clave:**
- `export component AddHabitModal`: Nombre, descripción y selector de color.
- Llama a `HabitAdapter.create-habit`.
**Dependencias:** `globals.slint`
---
## ui/modals/add_transaction.slint
**Tipo:** Slint UI
**Resumen:** Formulario modal para ingresos y gastos.
**Estructura Clave:**
- `export component AddTransactionModal`: Selector de cuenta, categoría, monto y fecha.
- Llama a `TransactionAdapter.add-transaction`.
**Dependencias:** `globals.slint`
---
## ui/modals/configure_ticker.slint
**Tipo:** Slint UI
**Resumen:** Modal para configurar qué monedas aparecen en el ticker superior.
**Estructura Clave:**
- `export component ConfigureTickerModal`: Lista con interruptores (toggles).
- Llama a `CryptoAdapter.save-ticker-options`.
**Dependencias:** `globals.slint`, `widgets.slint`
---
## ui/modals/transfer_funds.slint
**Tipo:** Slint UI
**Resumen:** Formulario modal para transferir dinero entre cuentas.
**Estructura Clave:**
- `export component TransferFundsModal`: Selectores Origen/Destino y monto.
- Llama a `AccountAdapter.transfer-funds`.
**Dependencias:** `globals.slint`
---
## ui/pages/crypto.slint
**Tipo:** Slint UI
**Resumen:** Página principal de gestión de criptomonedas.
**Estructura Clave:**
- `export component CryptoPage`: Contiene Ticker superior, resumen de portafolio y pestañas (Activos/Wallets).
**Dependencias:** `globals.slint`, `components/crypto_widgets.slint`, `components/asset_detail.slint`
---
## ui/pages/dashboard.slint
**Tipo:** Slint UI
**Resumen:** Panel de control principal con resumen de patrimonio.
**Estructura Clave:**
- `export component DashboardPage`: Muestra patrimonio neto total, gráfico de rendimiento y actividad reciente.
**Dependencias:** `globals.slint`, `components/charts.slint`, `components/category_breakdown.slint`
---
## ui/pages/finances.slint
**Tipo:** Slint UI
**Resumen:** Página de gestión financiera fiat (transacciones y cuentas).
**Estructura Clave:**
- `export component FinancesPage`: Pestañas para ver actividad o lista de cuentas.
**Dependencias:** `globals.slint`, `components/transaction_item.slint`, `components/account_item.slint`
---
## ui/pages/habits.slint
**Tipo:** Slint UI
**Resumen:** Página de seguimiento de hábitos.
**Estructura Clave:**
- `export component HabitsPage`: Muestra lista de hábitos, heatmap anual y gráfico mensual.
**Dependencias:** `globals.slint`, `components/habit_*.slint`, `components/analytics_charts.slint`
---
## ui/pages/login.slint
**Tipo:** Slint UI
**Resumen:** Pantalla de autenticación y creación de bóveda.
**Estructura Clave:**
- `export component LoginPage`: Entrada de contraseña.
- Llama a `AuthAdapter.unlock-vault` o `create-vault`.
**Dependencias:** `globals.slint`, `widgets.slint`
---
## ui/pages/settings.slint
**Tipo:** Slint UI
**Resumen:** Página de configuración de la aplicación.
**Estructura Clave:**
- `export component SettingsPage`: Interruptores para opciones como actualización automática.
- Llama a `SettingsAdapter`.
**Dependencias:** `globals.slint`
