# ==================== Sanctum Spanish Translations ====================
# Archivo de traducción al español
#
# Sintaxis Fluent: https://projectfluent.org/fluent/guide/
# - Variables: { $variableName }
# - Plurales: { $count -> [one] elemento *[other] elementos }

# ==================== Common ====================
app-name = SANCTUM
app-subtitle = Tu Fortaleza Financiera Personal

# Common actions
action-save = Guardar
action-cancel = Cancelar
action-delete = Eliminar
action-edit = Editar
action-create = Crear
action-add = Agregar
action-close = Cerrar
action-confirm = Confirmar
action-back = Atrás
action-next = Siguiente
action-submit = Enviar
action-archive = Archivar
action-restore = Restaurar
action-clear = Limpiar

# Common labels
label-name = Nombre
label-description = Descripción
label-amount = Monto
label-date = Fecha
label-category = Categoría
label-type = Tipo
label-status = Estado
label-balance = Saldo
label-total = Total
label-notes = Notas
label-color = Color
label-icon = Ícono
label-currency = Moneda
label-search = Buscar
label-filter = Filtrar
label-loading = Cargando...
label-none = Ninguno
label-all = Todos
label-yes = Sí
label-no = No

# Time
time-today = Hoy
time-yesterday = Ayer
time-days-ago = hace { $count } días
time-week = Semana
time-month = Mes
time-year = Año

# Validation
validation-required = Este campo es requerido
validation-invalid-amount = Monto inválido
validation-invalid-date = Fecha inválida

# ==================== Login Page ====================
login-title = SANCTUM
login-subtitle = Tu Fortaleza Financiera Personal
login-password-placeholder = contraseña
login-password-create-placeholder = crear contraseña
login-unlock = DESBLOQUEAR
login-create = CREAR BÓVEDA
login-unlocking = DESBLOQUEANDO...
login-creating = CREANDO...
login-password-required = Contraseña requerida
login-encryption-note = CIFRADO AES-256
login-weak-password-confirm = Contraseña débil: haz clic en crear nuevamente para confirmar.
login-show = VER
login-hide = OCULTAR

# ==================== Sidebar ====================
nav-dashboard = PANEL
nav-finances = FINANZAS
nav-crypto = CRYPTO
nav-habits = HÁBITOS
nav-settings = AJUSTES
nav-lock = BLOQUEAR

# ==================== Dashboard ====================
dashboard-title = Panel
dashboard-welcome = Bienvenido de vuelta
dashboard-net-worth = Patrimonio Neto
dashboard-total-balance = Saldo Total
dashboard-monthly-income = Ingresos del Mes
dashboard-monthly-expenses = Gastos del Mes
dashboard-recent-transactions = Transacciones Recientes
dashboard-no-transactions = Sin transacciones recientes
dashboard-view-all = Ver Todo
dashboard-quick-actions = Acciones Rápidas
dashboard-add-transaction = Agregar Transacción
dashboard-add-account = Agregar Cuenta

# ==================== Finances ====================
finances-title = FINANZAS
finances-accounts = CUENTAS
finances-transactions = Transacciones
finances-add-account = Agregar Cuenta
finances-add-transaction = Agregar Transacción
finances-no-accounts = Sin cuentas aún
finances-no-transactions = Sin transacciones encontradas
finances-transfer = Transferir
finances-income = Ingreso
finances-expense = Gasto
finances-transfer-funds = Transferir Fondos

# Account types
account-type-bank = Banco
account-type-cash = Efectivo
account-type-savings = Ahorros
account-type-credit = Tarjeta de Crédito
account-type-other = Otro

# Transaction filters
filter-all-accounts = Todas las Cuentas
filter-all-types = Todos los Tipos
filter-all-categories = Todas las Categorías
filter-date-range = Rango de Fechas
filter-this-month = Este Mes
filter-last-month = Mes Pasado
filter-this-year = Este Año
filter-custom = Personalizado

# ==================== Crypto ====================
crypto-title = Crypto
crypto-portfolio = Portafolio
crypto-wallets = BILLETERAS
crypto-tax-tab = IMPUESTOS
crypto-assets = ACTIVOS
crypto-add-wallet = Agregar Billetera
crypto-add-transaction = Agregar Transacción
crypto-no-wallets = Sin billeteras aún
crypto-no-assets = Sin activos encontrados
crypto-total-value = Valor Total
crypto-price = Precio
crypto-holdings = Tenencias
crypto-change-24h = Cambio 24h
crypto-market-cap = Cap. de Mercado
crypto-volume = Volumen

# Wallet types
wallet-type-exchange = Exchange
wallet-type-hardware = Hardware
wallet-type-software = Software
wallet-type-multi = Multi-Firma

# Transaction types
crypto-tx-buy = Compra
crypto-tx-sell = Venta
crypto-tx-transfer-in = Entrada
crypto-tx-transfer-out = Salida
crypto-tx-swap = Intercambio

# Transaction messages
crypto-tx-added = Activo agregado exitosamente
crypto-tx-transfer-added = Transferencia agregada exitosamente
crypto-tx-swap-added = Intercambio agregado exitosamente
crypto-tx-deleted = Transacción eliminada
crypto-tx-wallet-required = Primero crea una billetera
crypto-tx-two-wallets-required = Crea dos billeteras para mover activos
crypto-tx-amount-required = El monto es obligatorio
crypto-tx-price-required = El precio es obligatorio
crypto-tx-coins-required = Agrega monedas en configuración primero
crypto-tx-different-wallets = Selecciona dos billeteras distintas
crypto-tx-to-amount-required = El monto de destino es obligatorio
crypto-tx-swap-different-assets = Los activos del intercambio deben ser distintos

# ==================== Habits ====================
habits-title = HÁBITOS
habits-my-habits = Mis Hábitos
habits-add-habit = Agregar Hábito
habits-no-habits = Sin hábitos aún
habits-streak = Racha
habits-best-streak = Mejor
habits-current-streak = Actual
habits-completion-rate = Tasa de Completado
habits-days = { $count ->
    [one] { $count } día
   *[other] { $count } días
}

# Habit categories
habit-category-mind = Mente
habit-category-body = Cuerpo
habit-category-spirit = Espíritu

# Habit frequency
habit-frequency-daily = Diario
habit-frequency-weekly = Semanal

# Analytics
habits-analytics = Analíticas
habits-life-balance = Balance de Vida (Últimos 30 Días)
habits-weekday-efficiency = Eficiencia por Día
habits-empty-chart = El gráfico está vacío, pero tu potencial está lleno.
habits-empty-chart-subtitle = Tu leyenda aparecerá aquí. Empieza a escribirla hoy.
habits-complete-to-see = Completa hábitos para ver tu patrón semanal.
habits-discover-days = Descubre en qué días eres más consistente.

# Habits Tab sections
habits-add-button = + HÁBITO
habits-yearly-overview = RESUMEN ANUAL
habits-my-habits-section = MIS HÁBITOS
habits-no-tracked-month = No hay hábitos registrados este mes
habits-create-to-build = Crea un hábito para comenzar a construir consistencia
habits-analytics-section = ANALÍTICAS
habits-weekly-report = REPORTE SEMANAL
habits-insights = PERSPECTIVAS
habits-summary = RESUMEN DE HÁBITOS
habits-select-placeholder = Selecciona un hábito...

# ==================== Rewards ====================
rewards-title = Recompensas
rewards-goals = Metas
rewards-streak-rewards = Recompensas de Racha
rewards-history = Historial
rewards-add-goal = Agregar Meta
rewards-add-reward = Agregar Recompensa
rewards-no-goals = Sin metas aún
rewards-no-rewards = Sin recompensas aún
rewards-no-history = Sin historial aún
rewards-progress = Progreso
rewards-milestones = Hitos
rewards-unlocked = Desbloqueado
rewards-locked = Bloqueado
rewards-claim = Reclamar
rewards-completed = Completado

# ==================== Settings ====================
# NOTA: Los titulos de secciones de Settings deben ir en MAYUSCULAS.
settings-title = Ajustes
settings-general = GENERAL
settings-appearance = APARIENCIA
settings-security = SEGURIDAD
settings-data = DATOS
settings-about = ACERCA DE

# General settings
settings-language = Idioma
settings-language-desc = Idioma de la interfaz
settings-currency = Moneda
settings-currency-desc = Moneda predeterminada
settings-preferred-currency = Moneda Preferida
settings-preferred-currency-desc = Moneda base para mostrar montos (solo UI)

# Appearance settings
settings-dark-mode = Modo Oscuro
settings-dark-mode-desc = Activar tema oscuro
settings-dark-mode-title = Modo Oscuro
settings-dark-mode-toggle-desc = Cambiar entre temas oscuro y claro (crema/dorado)
settings-wallpaper-title = Fondo de inicio de sesión
settings-wallpaper-desc = Imagen de fondo para la pantalla de bloqueo
settings-wallpaper-default = Predeterminado
settings-wallpaper-select = SELECCIONAR IMAGEN
settings-wallpaper-reset = RESTABLECER
settings-wallpaper-note = Se guarda en config.toml (no está cifrado).
settings-wallpaper-formats = Formatos admitidos: PNG, JPG, JPEG, WEBP, BMP, GIF, TIFF

# Security settings
settings-session-timeout = Tiempo de Sesión
settings-session-timeout-desc = Bloqueo automático por inactividad
settings-timeout-5min = 5 minutos
settings-timeout-15min = 15 minutos
settings-timeout-30min = 30 minutos
settings-timeout-1hour = 1 hora
settings-timeout-never = Nunca
settings-timeout-warning = Los cambios de tiempo se aplicarán en la próxima apertura de bóveda.

# Crypto settings
settings-auto-fetch = Actualizar Precios
settings-auto-fetch-desc = Actualizar precios de crypto automáticamente
settings-auto-fetch-title = Actualizar Precios de Crypto Automáticamente
settings-auto-fetch-toggle-desc = Actualizar precios cada minuto mientras la app está activa (usa red)
settings-proxy = Proxy
settings-proxy-enabled = Habilitar Proxy
settings-proxy-url = URL del Proxy
settings-proxy-title = Usar Proxy de Red
settings-proxy-toggle-desc = Enrutar solicitudes de precios de crypto a través de un proxy (opcional)
settings-proxy-placeholder = http://127.0.0.1:8080 o socks5h://127.0.0.1:9050

# Data settings
settings-reset = Restablecer Ajustes
settings-reset-desc = Restablecer todos los ajustes
settings-reset-confirm = ¿Estás seguro de que quieres restablecer todos los ajustes?

# ==================== Modals ====================
modal-add-account-title = Agregar Cuenta
modal-edit-account-title = Editar Cuenta
modal-add-transaction-title = Agregar Transacción
modal-edit-transaction-title = Editar Transacción
modal-transfer-title = Transferir Fondos
modal-add-wallet-title = Agregar Billetera
modal-edit-wallet-title = Editar Billetera
modal-add-habit-title = Agregar Hábito
modal-edit-habit-title = Editar Hábito
modal-add-goal-title = Agregar Meta
modal-edit-goal-title = Editar Meta
modal-add-reward-title = Agregar Recompensa
modal-edit-reward-title = Editar Recompensa

# Confirmation dialogs
confirm-delete-title = Confirmar Eliminación
confirm-delete-message = Esta acción no se puede deshacer.
confirm-delete-account = ¿Estás seguro de que quieres eliminar esta cuenta?
confirm-delete-transaction = ¿Estás seguro de que quieres eliminar esta transacción?
confirm-delete-wallet = ¿Estás seguro de que quieres eliminar esta billetera?
confirm-delete-habit = ¿Estás seguro de que quieres eliminar este hábito?

# ==================== Notifications ====================
notify-success = Éxito
notify-error = Error
notify-saved = Cambios guardados
notify-deleted = Elemento eliminado
notify-created = Elemento creado
notify-updated = Elemento actualizado

# ==================== Empty States ====================
empty-no-data = Sin datos disponibles
empty-add-first = Agrega tu primer { $item } para comenzar
empty-no-results = Sin resultados
empty-try-different = Intenta con una búsqueda o filtro diferente

# ==================== Errors ====================
error-generic = Algo salió mal
error-connection = Error de conexión
error-invalid-input = Entrada inválida
error-not-found = No encontrado
error-unauthorized = Acceso no autorizado

# ==================== Misc UI Text ====================
bank-icons-title = ÍCONOS DE BANCO
no-expenses-recorded = Sin gastos registrados
fee-label = Comisión
empty-no-transactions-account = Sin transacciones para esta cuenta
crypto-total-holdings = TENENCIAS TOTALES
crypto-no-wallet-data = Sin datos de billetera disponibles
crypto-no-transactions-found = Sin transacciones encontradas
crypto-portfolio-distribution = DISTRIBUCIÓN DEL PORTAFOLIO
confirm-delete-generic = Esto eliminará permanentemente

# ==================== Dashboard Extended ====================
dashboard-total-net-worth = PATRIMONIO NETO TOTAL
dashboard-exchange-rate-warning = Tipo de cambio no disponible para algunas monedas. Saldos mostrados con tasa de respaldo 1:1.
dashboard-loading = Cargando panel...
dashboard-retry = REINTENTAR
dashboard-usd-clp = USD/CLP

# ==================== Finances Extended ====================
finances-activity = ACTIVIDAD
finances-account = CUENTA
finances-all-accounts = Todas las cuentas
finances-all-categories = Todas las categorías
finances-load-more = CARGAR MÁS
finances-configure = CONFIGURAR
finances-transaction-categories = Categorías de Transacciones
finances-manage-categories = Administrar categorías de ingresos y gastos
finances-delete-transaction = Eliminar Transacción
finances-delete-confirm = ¿Estás seguro de que quieres eliminar

# ==================== Crypto Extended ====================
crypto-portfolio-title = PORTAFOLIO CRYPTO
crypto-last-updated = Última actualización
crypto-last-updated-info = Última actualización: {$value}
crypto-last-updated-never = Nunca
crypto-last-updated-today-at = Hoy a las {$time}
crypto-coin-limit = Límite de monedas alcanzado (50). Algunos activos no se actualizarán.
crypto-skipped = Omitidos
crypto-your-holdings = TUS TENENCIAS
crypto-no-assets-yet = Sin activos registrados aún
crypto-create-wallet-first = Crea una billetera primero, luego agrega tus criptomonedas
crypto-start-adding = Comienza agregando una billetera y tu primer activo
crypto-import-csv = IMPORTAR CSV
crypto-unrealized = NO REALIZADO
crypto-realized-ytd = REALIZADO (YTD)
crypto-roi = ROI
crypto-tax-title = IMPUESTOS Y REPORTES
crypto-tax-subtab-settings = CONFIGURACIÓN
crypto-tax-subtab-summary = RESUMEN
crypto-tax-period-label = PERIODO TRIBUTARIO (AÑO)
crypto-tax-period-placeholder = 2025
crypto-tax-jurisdiction-label = JURISDICCIÓN
crypto-tax-jurisdiction-cl = Chile
crypto-tax-jurisdiction-us = EE.UU.
crypto-tax-jurisdiction-other = Otro
crypto-tax-method-label = MÉTODO DE COSTO
crypto-tax-include-swaps = Incluir swaps como gravables
crypto-tax-include-swaps-desc = Trata swaps como disposiciones para el reporte.
crypto-tax-include-fee-crypto = Incluir comisión en cripto
crypto-tax-include-fee-crypto-desc = Trata la comisión en cripto como disposición gravable.
crypto-tax-save-settings = Guardar configuración tributaria
crypto-tax-report-title = GENERAR REPORTE
crypto-tax-report-desc = Genera un reporte tributario del periodo seleccionado. La exportación es un CSV local.
crypto-tax-report-generate = Generar reporte
crypto-tax-report-export = Exportar CSV
crypto-tax-report-summary-label = RESUMEN DEL REPORTE
crypto-tax-report-summary-empty = Aún no se ha generado un reporte
crypto-tax-report-summary = Disposiciones: {$disposals} | Ingresos: {$proceeds} | Costo: {$cost} | Ganancia: {$gain}
crypto-tax-report-summary-us = Disposiciones: {$disposals} | Ingresos: {$proceeds} | Costo: {$cost} | Ganancia: {$gain} | Corto: {$short} | Largo: {$long}
crypto-tax-report-warnings-label = ADVERTENCIAS
crypto-tax-report-warnings-empty = Sin advertencias
crypto-tax-report-warnings-count = Advertencias: {$count} (ver CSV)
crypto-tax-report-generated = Reporte generado
crypto-tax-report-exported = Reporte exportado
crypto-tax-summary-title = RESUMEN TRIBUTARIO
crypto-tax-summary-empty = Genera un reporte para ver el resumen
crypto-tax-summary-capital = GANANCIAS DE CAPITAL
crypto-tax-summary-income = INGRESO GRAVABLE
crypto-tax-summary-balance = SALDO A FIN DE AÑO
crypto-tax-summary-proceeds = Ingresos
crypto-tax-summary-cost = Costo base
crypto-tax-summary-gain = Ganancia / Pérdida
crypto-tax-summary-income-total = Total ingresos
crypto-tax-summary-balance-total = Valor total
crypto-tax-summary-reports = REPORTES
crypto-tax-summary-export-history = Exportar historial de transacciones
crypto-tax-summary-export-capital = Exportar ganancias de capital
crypto-tax-summary-simulation = SIMULACIÓN
crypto-tax-summary-transactions = Transacciones calculadas
crypto-tax-summary-volume = Volumen procesado
crypto-tax-summary-short-term = CORTO PLAZO
crypto-tax-summary-long-term = LARGO PLAZO
crypto-tax-summary-disposals = Enajenaciones
crypto-tax-readiness-banner = problemas por revisar antes de declarar
crypto-tax-empty-title = Sin reporte generado
crypto-tax-empty-desc = Configura tus ajustes y genera un reporte para ver tu resumen tributario
crypto-tax-exports-title = EXPORTAR REPORTES
crypto-tax-exports-capital-desc = Enajenaciones detalladas con desglose de lotes
crypto-tax-exports-history-desc = Todas las transacciones con clasificación tributaria
crypto-tax-report-details = DETALLE DEL REPORTE
crypto-tax-settings-advanced = OPCIONES AVANZADAS
crypto-tax-wallet-exclusions = EXCLUSIÓN DE BILLETERAS
crypto-tax-wallet-exclusions-desc = Excluye billeteras del cálculo tributario. Las transacciones en billeteras excluidas no aparecerán en los reportes.
crypto-tax-wallet-none = No se encontraron billeteras
crypto-tax-wallet-excluded-label = excluida
crypto-tax-filing-title = GUÍA DE DECLARACIÓN
crypto-tax-save-generate = Guardar y generar
crypto-tax-readiness-settings-suffix = transacciones en el período
crypto-tax-readiness-history-warn-suffix = enajenaciones con lotes insuficientes
crypto-tax-readiness-prices-invalid-suffix = Fechas o tipos inválidos encontrados
crypto-tax-readiness-prices-warn-suffix = elementos sin datos de precio
crypto-tax-readiness-transfers-warn-suffix = transferencias sin contrapartida
crypto-tax-readiness-balances-warn-detail = Algunas enajenaciones superan la cantidad disponible en lotes
crypto-tax-readiness-sii-gain-detail = Ganancia -> F22 Linea 10, Casilla 1032. Aviso: Las casillas pueden cambiar cada ano.
crypto-tax-readiness-sii-loss-detail = Perdida -> F22 Linea 17, Casilla 169 (tope). Aviso: Las casillas pueden cambiar cada ano.
crypto-tax-readiness-sii-neutral-detail = Sin ganancia ni perdida neta. Aviso: Las casillas del F22 pueden cambiar cada ano.
crypto-tax-readiness-usa-filing-detail = Reportar en Form 8949 + Schedule D.
crypto-tax-readiness-other-filing-detail = Aviso: Revisa la legislacion tributaria especifica de tu pais -- las reglas varian significativamente entre jurisdicciones. Este reporte usa reglas estandar internacionales (comisiones en costo base, FMV para ingresos, corto/largo plazo a 365 dias). Consulta a un asesor tributario local antes de declarar.
crypto-tax-readiness-title = LISTA DE CONTROL
crypto-tax-readiness-desc = Revisa los problemas antes de declarar
crypto-tax-readiness-settings = Revisar configuración
crypto-tax-readiness-settings-count = {$count} transacciones en el período
crypto-tax-readiness-history = Revisar historial
crypto-tax-readiness-history-warn = {$count} enajenaciones con lotes insuficientes
crypto-tax-readiness-balances = Revisar saldos
crypto-tax-readiness-balances-warn = Algunas enajenaciones superan la cantidad disponible en lotes
crypto-tax-readiness-prices = Resolver precios faltantes
crypto-tax-readiness-prices-invalid = Se encontraron fechas o tipos de transacción inválidos
crypto-tax-readiness-prices-warn = {$count} elementos sin datos de precio
crypto-tax-readiness-transfers = Revisar transferencias
crypto-tax-readiness-transfers-warn = {$count} transferencias sin contrapartida
crypto-tax-readiness-filing = Guía de declaración
crypto-tax-readiness-sii-f22 = Formulario 22 SII
crypto-tax-readiness-sii-gain = Ganancia -> F22 Linea 10, Casilla 1032. Aviso: Las casillas pueden cambiar cada ano.
crypto-tax-readiness-sii-loss = Perdida -> F22 Linea 17, Casilla 169 (tope: casillas 105+155+152+1032+1891+1104). Aviso: Las casillas pueden cambiar cada ano.
crypto-tax-readiness-sii-neutral = Sin ganancia ni perdida neta. Aviso: Las casillas del F22 pueden cambiar cada ano.
crypto-tax-readiness-usa-filing = USA: Reportar en Form 8949 + Schedule D.
crypto-tax-readiness-other-filing = Aviso: La legislacion de tu pais puede diferir. Consulta a un asesor tributario local antes de declarar.
crypto-tax-sii-casilla-warning = Los códigos de casilla del Formulario 22 pueden cambiar cada Año Tributario. Verifica siempre el suplemento tributario del SII vigente.
crypto-tax-ipc-title = IPC (Chile)
crypto-tax-ipc-desc = Descarga la serie oficial del IPC, conviértela a CSV e impórtala aquí. Por defecto no se realizan conexiones a Internet.
crypto-tax-ipc-source-label = FUENTE OFICIAL (DESCARGA MANUAL)
crypto-tax-ipc-source-url = https://www.ine.gob.cl/docs/default-source/%C3%ADndice-de-precios-al-consumidor/cuadros-estadisticos/series-empalmadas-y-antecedentes-historicos/series-empalmadas-diciembre-2009-a-la-fecha/serie-hist%C3%B3rica-empalmada-ipc-diciembre-2009-a-la-fecha-xls.xlsx
crypto-tax-ipc-import = Importar IPC (CSV)
crypto-tax-ipc-copy-url = Copiar URL
crypto-tax-ipc-summary-label = ESTADO IPC
crypto-tax-ipc-empty = No hay datos IPC cargados
crypto-tax-ipc-summary = Cargado: {$first} -> {$last} ({$count} meses)
crypto-tax-ipc-import-success = IPC importado: {$count} meses ({$first} -> {$last})
crypto-tax-settings-saved = Configuración tributaria guardada
crypto-tax-period-required = Debes indicar el período tributario
crypto-tax-advanced = CLASIFICACIÓN TRIBUTARIA (OPCIONAL)
modal-transaction-type = CATEGORÍA TRIBUTARIA
modal-transaction-type-placeholder = trade / income / expense / transfer
modal-transaction-subtype = SUBTIPO TRIBUTARIO
modal-transaction-subtype-placeholder = airdrop / staking / fee / other
modal-tax-override-proceeds = SOBREESCRIBIR INGRESOS
modal-tax-override-cost = SOBREESCRIBIR COSTO BASE
crypto-assets-across-wallets = { $assets } activos en { $wallets } billeteras
crypto-wallet = BILLETERA
crypto-value = VALOR

crypto-add-first-wallet = Agrega tu primera billetera para empezar a rastrear tus criptos
crypto-no-wallets-created = No hay billeteras creadas
crypto-delete-wallet = ¿Eliminar Billetera?
crypto-delete-wallet-confirm-prefix = Esto eliminará permanentemente "
crypto-delete-wallet-confirm-suffix = " y todo su historial de transacciones.
crypto-delete-wallet-warning-title = La billetera tiene transacciones
crypto-delete-wallet-warning-prefix = Esta billetera contiene 
crypto-delete-wallet-warning-suffix =  transacción(es). Eliminarla las borrará permanentemente.
crypto-delete-wallet-force = Eliminar de todos modos
crypto-loading-portfolio = Cargando portafolio...
crypto-syncing-prices = Sincronizando precios...
crypto-syncing-wait = Esto puede tomar unos segundos

# ==================== Habits Extended ====================
habits-rewards = RECOMPENSAS
habits-history = HISTORIAL

# ==================== Settings Extended ====================
settings-configure-experience = Configura tu experiencia en Sanctum
settings-proxy-tip = Tip: socks5h:// enruta el DNS a través del proxy para mayor privacidad.
settings-data-encrypted = Tus datos están cifrados localmente
settings-military-grade = Toda la información sensible está protegida con cifrado de grado militar
settings-reset-defaults = RESTABLECER VALORES

# ==================== Common Actions Extended ====================
action-view-all = VER TODO →
action-retry = REINTENTAR
action-load-more = CARGAR MÁS
action-configure = CONFIGURAR
action-transfer = TRANSFERIR

# ==================== Components ====================
# Account Item
account-balance = Saldo

# Crypto Widgets
crypto-holdings-label = Tenencias
crypto-price-label = Precio

# Crypto Charts
crypto-no-priced-assets = Sin activos con precio aún
crypto-sync-to-see = Sincroniza precios para ver la distribución
crypto-portfolio-trend = TENDENCIA DEL PORTAFOLIO (180 DÍAS)
crypto-value-label = VALOR
crypto-cost-label = COSTO
crypto-no-trend = Sin datos de tendencia aún
crypto-sync-daily = Sincroniza precios diariamente para construir historial

# Habit Heatmap
heatmap-less = Menos
heatmap-more = Más

# Habits Tab
habits-selected-hint = HÁBITO SELECCIONADO · Haz clic en un hábito arriba para ver estadísticas

# History Tab
history-total-achievements = LOGROS TOTALES

# Streak Rewards
rewards-ready-claim = Listo para reclamar
rewards-next = Siguiente
rewards-all-unlocked = ¡Todos los hitos desbloqueados!

# Wallet Detail
wallet-no-holdings = Sin tenencias en esta billetera

# Icon Selector
icon-choose = ELEGIR ÍCONO
icon-exchanges = Exchanges
icon-wallet-icons = Íconos de Billetera

# Forms
form-search-coin = Buscar moneda...
form-date-format = AAAA-MM-DD
form-all = TODO
form-habit = HÁBITO

# ==================== Modals ====================
# Add Account
modal-delete-account = ELIMINAR CUENTA
modal-save-account = GUARDAR CUENTA
modal-delete-account-confirm = Esto eliminará permanentemente y todo su historial de transacciones.

# Add Transaction
modal-no-accounts = No hay cuentas disponibles. Crea una primero.

# Add Crypto Transaction
modal-new-crypto-transaction = NUEVA TRANSACCIÓN CRYPTO
modal-save-transaction = GUARDAR TRANSACCIÓN
modal-create-wallet-first = Crea una billetera en la pestaña Billeteras primero
modal-create-another-wallet = Crea otra billetera para mover activos

# Edit Crypto Transaction
modal-edit-crypto-transaction = EDITAR TRANSACCIÓN CRYPTO
modal-save-changes = GUARDAR CAMBIOS

# Add Wallet
modal-new-wallet = NUEVA BILLETERA
modal-wallet-type = TIPO DE BILLETERA
modal-create-wallet = CREAR BILLETERA

# Add Habit
modal-category = CATEGORÍA
modal-color = COLOR
modal-habit-name-placeholder = ej. Leer 10 páginas
modal-habit-description-placeholder = Por qué este hábito es importante

# Configure Categories
modal-category-settings = CONFIGURACIÓN DE CATEGORÍAS
modal-manage-categories = Administrar categorías de ingresos y gastos para transacciones.
modal-expense-categories = CATEGORÍAS DE GASTOS
modal-income-categories = CATEGORÍAS DE INGRESOS
modal-no-expense-categories = Sin categorías de gastos
modal-no-income-categories = Sin categorías de ingresos
modal-add-new-category = AGREGAR NUEVA CATEGORÍA
modal-category-name = Nombre de categoría
modal-default = PREDETERMINADO

# Configure Ticker
modal-crypto-settings = CONFIGURACIÓN CRYPTO
modal-manage-price-bar = Administrar barra de precios y catálogo de monedas.
modal-price-bar = BARRA DE PRECIOS
modal-remove = ELIMINAR

# Add Transaction
modal-account = CUENTA
modal-expense = GASTO
modal-income = INGRESO

# Add Habit
modal-checkpoints = PUNTOS DE CONTROL
modal-checkpoint-desc = Descripción del punto de control...

# Add Reward
modal-consecutive = CONSECUTIVO
modal-accumulative = ACUMULATIVO
modal-type = TIPO
modal-milestones = HITOS
modal-reward-placeholder = Recompensa...

# Configure Ticker
modal-coin-catalog = CATÁLOGO DE MONEDAS
modal-max-coins = Máximo 50 monedas activas para actualización de precios.
modal-catalog-info = El catálogo se usa para la barra de precios y transacciones.
modal-coin-list = LISTA DE MONEDAS
modal-add-coin = AGREGAR MONEDA
modal-removing-info = Eliminar monedas solo las oculta aquí.
modal-select-all = SELECCIONAR TODO
modal-remove-selected = ELIMINAR SELECCIONADOS

# Sidebar branding
sidebar-logo = S
sidebar-title = SANCTUM

# Crypto Widgets
crypto-holdings-small = Tenencias
crypto-price-small = Precio

# Streak Rewards (with arguments)
rewards-ready-claim-with = Listo para reclamar: { $reward }
rewards-next-with = Siguiente: { $reward }

# ==================== Crypto Transaction Modal ====================
modal-from-asset = ACTIVO ORIGEN
modal-to-asset = ACTIVO DESTINO
modal-cryptocurrency = CRIPTOMONEDA
modal-from-wallet = BILLETERA ORIGEN
modal-to-wallet = BILLETERA DESTINO
modal-from-amount = MONTO ORIGEN
modal-to-amount = MONTO DESTINO
modal-to-amount-optional = MONTO DESTINO (opcional)
modal-same-as-from = Igual que ORIGEN
modal-price-usd = PRECIO (USD)
modal-optional = Opcional
modal-required = Requerido
modal-fee-usd = COMISIÓN (USD)
modal-fee-coin-optional = MONEDA COMISIÓN (OPCIONAL)
modal-fee-amount = MONTO COMISIÓN
modal-notes = NOTAS
modal-transaction-details = Detalles de la transacción...
modal-date = FECHA
modal-fetch-price-date = Buscar precio para la fecha seleccionada
modal-search-coins = Buscar monedas...

# Section labels (crypto transaction modal)
section-asset-wallet = ACTIVO Y BILLETERA
section-amount = MONTO
section-advanced = AVANZADO
section-details = DETALLES
section-fee-crypto = COMISIONES EN CRYPTO
section-tax = CLASIFICACIÓN TRIBUTARIA

# Transaction summary
tx-summary-buying = Comprando
tx-summary-selling = Vendiendo
tx-summary-swapping = Intercambiando
tx-summary-moving = Moviendo
tx-summary-receiving = Recibiendo
tx-summary-sending = Enviando
tx-summary-at = a
tx-summary-per-coin = /moneda
tx-summary-to = hacia
tx-summary-from = desde

# Category tabs (type selector)
tx-category-trade = COMERCIO
tx-category-transfer = TRANSFERENCIA
tx-category-income = INGRESO
tx-category-expense = GASTO

# Scenario labels (type selector sub-options)
tx-scenario-deposit = Deposito
tx-scenario-withdrawal = Retiro
tx-scenario-interest = Interes
tx-scenario-gift = Regalo
tx-scenario-reward = Recompensa
tx-scenario-other = Otro
tx-scenario-payment = Pago
tx-scenario-donation = Donacion
tx-scenario-fee = Comision
tx-scenario-lost = Perdido
tx-scenario-stolen = Robado
tx-scenario-buy = COMPRA
tx-scenario-sell = VENTA
tx-scenario-swap = SWAP
tx-scenario-move = MOVER
tx-scenario-airdrop = Airdrop
tx-scenario-staking = Staking
tx-scenario-mining = Mineria
tx-scenario-fork = Fork

# Type badge labels (edit modal)
tx-type-buy = COMPRA
tx-type-sell = VENTA
tx-type-swap = SWAP
tx-type-transfer-in = TRANSFERENCIA ENTRADA
tx-type-transfer-out = TRANSFERENCIA SALIDA

# ==================== Goal Modal ====================
modal-new-goal = NUEVA META
modal-edit-goal = EDITAR META
modal-goal-name = NOMBRE DE META
modal-goal-name-placeholder = ej. Correr un maratón
modal-description-optional = DESCRIPCIÓN (OPCIONAL)
modal-goal-description-placeholder = Por qué esta meta es importante...
modal-reward = RECOMPENSA
modal-reward-placeholder-goal = ej. Zapatillas nuevas
modal-deadline-optional = FECHA LÍMITE (OPCIONAL)
modal-create-goal = CREAR META

# ==================== Reward Modal ====================
modal-new-streak-reward = NUEVA RECOMPENSA DE RACHA
modal-edit-reward = EDITAR RECOMPENSA
modal-consecutive-desc = Los días deben ser consecutivos (se reinicia si fallas)
modal-accumulative-desc = Acumula días a lo largo del tiempo
modal-target-days = DÍAS OBJETIVO
modal-of-total-days = DE DÍAS TOTALES
modal-days-label = días
modal-create-reward = CREAR RECOMPENSA

# ==================== Configure Ticker Extended ====================
modal-add-custom-coin = AGREGAR MONEDA PERSONALIZADA
modal-coingecko-hint = Usa el ID de CoinGecko (minúsculas, guiones). Ejemplo: litecoin
modal-symbol-hint = El símbolo usa solo letras. Ejemplo: LTC
modal-coingecko-id = ID COINGECKO
modal-coingecko-id-placeholder = ej. litecoin
modal-name-placeholder = ej. Litecoin
modal-symbol = SÍMBOLO
modal-symbol-placeholder = ej. LTC
modal-save-configuration = GUARDAR CONFIGURACIÓN

# ==================== Wallet Modal ====================
modal-wallet-name = NOMBRE DE BILLETERA

# ==================== Transfer Modal ====================
modal-edit-transfer = EDITAR TRANSFERENCIA
modal-from = DESDE
modal-to = HACIA
modal-transfer-action = TRANSFERIR

# ==================== Icon Modals ====================
modal-select-bank-icon = SELECCIONAR ÍCONO DE BANCO
modal-select-icon = SELECCIONAR ÍCONO
modal-save-icon = GUARDAR ÍCONO

# ==================== Common Button Labels ====================
button-add = AGREGAR
button-sync = ↻ SINCRONIZAR
button-syncing = SINCRONIZANDO...
button-add-transaction = + AGREGAR TRANSACCIÓN
button-add-transaction-short = + TRANSACCIÓN
button-new-entry = + NUEVA ENTRADA
button-new-account = + NUEVA CUENTA

# ==================== Page Titles and Sections ====================
section-fiat = FIAT
section-spending-breakdown = DESGLOSE DE GASTOS
section-recent-activity = ACTIVIDAD RECIENTE
section-recent-transactions = TRANSACCIONES RECIENTES
section-my-accounts = MIS CUENTAS
section-finance-settings = AJUSTES DE FINANZAS
section-transactions = TRANSACCIONES
section-wallet-breakdown = DESGLOSE POR BILLETERA

# ==================== Settings Page ====================
section-regional = REGIONAL
section-data-sync = DATOS Y SINCRONIZACIÓN
section-about = ACERCA DE
settings-version-label = Versión
settings-encryption-label = Cifrado
settings-database-label = Base de Datos
settings-encryption-type = AES-256-GCM
settings-storage-type = SQLite (Cifrado)

# ==================== Vault Backup ====================
vault-backup-section = RESPALDO DE BÓVEDA
vault-export-button = EXPORTAR RESPALDO
vault-restore-button = RESTAURAR RESPALDO
vault-restore-from-backup = Restaurar desde respaldo...

vault-export-success = Respaldo de bóveda creado exitosamente
vault-restore-success = Bóveda restaurada exitosamente. Por favor inicia sesión.
vault-export-failed = Error al exportar la bóveda
vault-restore-failed = Error al restaurar la bóveda

vault-restore-warning-title = ¿Restaurar Bóveda desde Respaldo?
vault-restore-warning-desc = Esto reemplazará tu bóveda actual con el archivo de respaldo. Todos los datos actuales serán sobrescritos. Esta acción no se puede deshacer.
vault-restore-file-label = ARCHIVO DE RESPALDO
vault-restore-cancel = CANCELAR
vault-restore-confirm = RESTAURAR BÓVEDA

vault-invalid-backup = Archivo de respaldo inválido
vault-backup-too-large = Archivo de respaldo demasiado grande (máx 1GB)
vault-permission-denied = Permiso denegado al acceder al archivo
vault-backup-encryption-note = Los respaldos mantienen cifrado completo. Nunca exportes a ubicaciones no confiables.

# ==================== Asset/Wallet Details ====================
section-transaction-history = HISTORIAL DE TRANSACCIONES

# ==================== Transaction Entry Modal ====================
modal-new-entry = NUEVA ENTRADA
modal-edit-entry = EDITAR ENTRADA
modal-save-entry = GUARDAR ENTRADA
modal-add-note = Agregar una nota...

# ==================== Finances Extended (Search/Empty States) ====================
finances-search-placeholder = Buscar por descripción, categoría, fecha...
finances-no-matching = Sin transacciones coincidentes
finances-no-transactions-yet = Sin transacciones aún
finances-try-adjusting = Intenta limpiar o ajustar tus filtros
finances-add-first-entry = Agrega tu primera entrada para empezar a rastrear tus finanzas
finances-no-accounts-configured = Sin cuentas configuradas
finances-create-account = Crea una cuenta para administrar tus fondos

# ==================== Crypto Extended (Buttons) ====================
crypto-add-wallet-button = + BILLETERA

# ==================== Habits Extended (Summary Labels) ====================
habits-current-streak-label = RACHA ACTUAL
habits-best-streak-label = MEJOR RACHA (365D)
habits-days-label = días
habits-completion-rate-label = TASA DE COMPLETADO
habits-completions-label = COMPLETADOS (30D)

# ==================== Rewards Extended (Sections/Buttons) ====================
rewards-streak-rewards-section = RECOMPENSAS DE RACHA
rewards-add-reward-button = + RECOMPENSA
rewards-no-streak-rewards = Sin recompensas de racha aún
rewards-link-habit-desc = Vincula un hábito y establece recompensas por hitos
rewards-goals-section = METAS
rewards-add-goal-button = + META
rewards-no-goals-set = Sin metas establecidas
rewards-create-goal-desc = Crea una meta con puntos de control para seguir tu progreso

# ==================== Rewards Progress ====================
rewards-days-to-go = días restantes

# ==================== History Tab ====================
history-achievements-section = LOGROS
history-no-achievements = Sin logros aún
history-complete-to-earn = Completa metas para ganar trofeos

# ==================== Importación de Datos ====================
import-title = Importar Datos
import-description = Importar transacciones y registros de hábitos desde archivos externos
import-select-file = SELECCIONAR ARCHIVO
import-supported-formats = Formatos soportados: JSON, CSV, TXT
import-max-size = Tamaño máximo de archivo: 10MB

import-processing = Procesando archivo...
import-validating = Validando datos...
import-inserting = Insertando registros...

import-success = Importación completada exitosamente
import-partial = Importación completada con algunos problemas
import-failed = Importación fallida

import-summary-title = RESUMEN DE IMPORTACIÓN
import-total-processed = Total Procesados
import-inserted = Insertados
import-skipped = Omitidos
import-errors = Errores
import-preview-title = VISTA PREVIA
import-preview-subtitle = Revisa los cambios detectados antes de importar.
import-preview-file-label = ARCHIVO
import-preview-format-label = FORMATO
import-preview-type-label = TIPO
import-preview-confirm = IMPORTAR
import-preview-cancel = CANCELAR

import-error-details = DETALLES DE ERRORES
import-skipped-reasons = RAZONES DE OMISIÓN
import-line = Línea { $line }
import-field = Campo: { $field }

import-error-file-too-large = Archivo muy grande. El tamaño máximo es { $maxSize }MB
import-error-unsupported-format = Formato de archivo no soportado. Use JSON, CSV o TXT
import-error-invalid-json = Formato JSON inválido
import-error-no-data = No se encontraron datos en el archivo
import-error-account-not-found = Cuenta no encontrada: { $name }
import-error-habit-not-found = Hábito no encontrado: { $name }
import-error-category-not-found = Categoría no encontrada: { $name }
import-error-currency-mismatch = Moneda no coincide para la cuenta { $account }
import-error-duplicate = Entrada duplicada omitida
import-error-currency-mismatch-detail = Moneda incompatible: la importación tiene { $import } pero la cuenta { $account } usa { $expected }
import-error-category-not-found-detail = Categoría no encontrada: { $name } (tipo: { $type })
import-error-destination-account-not-found = Cuenta destino no encontrada: { $name }
import-error-same-account-transfer = No se puede transferir a la misma cuenta
import-error-wallet-not-found = Wallet no encontrada: { $name }
import-error-crypto-not-found = Activo crypto no encontrado en el catálogo: { $symbol }
import-error-insufficient-crypto-balance = Balance insuficiente de { $symbol } en { $wallet }: tiene { $available }, necesita { $required }
import-skipped-duplicate-transaction = Transacción duplicada (misma fecha/cuenta/monto/tipo/descripción)
import-skipped-habit-not-completed = Hábito no completado (completed=false)
import-skipped-habit-already-logged = Hábito ya registrado para esta fecha
import-skipped-duplicate-crypto = Transacción crypto duplicada (misma fecha/wallet/moneda/tipo/monto)
import-skipped-crypto-not-found = Activo crypto no encontrado en el catálogo (fila omitida)

import-format-json = JSON (Exportación de Sanctum Web)
import-format-csv = CSV (Excel/Sheets)
import-format-text = Texto Plano

import-preview-change-transaction = Transacción
import-preview-change-income = Ingreso
import-preview-change-expense = Gasto
import-preview-change-transfer = Transferencia
import-preview-change-habit = Registro de Hábito
import-preview-change-crypto = Transacción Crypto
import-preview-changes = CAMBIOS PREVISTOS

settings-import = IMPORTAR DATOS
settings-import-desc = Importar transacciones y hábitos desde archivos

# ==================== Importación CSV de Exchanges ====================
import-exchange-title = Importar Exchange
import-exchange-description = Importar historial de transacciones desde exchanges y wallets
import-exchange-select-file = SELECCIONAR ARCHIVO
import-exchange-supported = Soportados: Kraken, Binance, MEXC, NotBank, Feather Wallet, Monero GUI Wallet
import-exchange-wallet-label = WALLET DESTINO
import-exchange-wallet-placeholder = Nombre del wallet para las transacciones importadas
import-exchange-detected = Formato detectado
import-exchange-not-detected = No se pudo detectar el formato del exchange. Soportados: Kraken, Binance, MEXC, NotBank, Feather Wallet, Monero GUI Wallet.
import-exchange-default-wallet = Usando wallet por defecto
import-exchange-importing = Importando transacciones de { $exchange }...
import-exchange-success = Importación de { $exchange } completada
import-exchange-kraken-ledger = Kraken Ledger
import-exchange-kraken-trades = Kraken Trades
import-exchange-binance-all = Binance Todos los Estados
import-exchange-binance-spot = Binance Historial Spot
import-exchange-feather = Feather Wallet
import-exchange-monero-gui = Monero GUI Wallet
import-exchange-mexc-spot = MEXC Historial Spot
import-exchange-hint-kraken = Kraken: Documents > exportar los CSV Ledgers y Trades (puedes subir ambos juntos).
import-exchange-hint-kraken-pro = Kraken Pro: History > Statements > exportar los CSV Ledgers y Trades (puedes subir ambos juntos).
import-exchange-hint-binance = Exportar desde Binance: Órdenes > Historial de Transacciones > Generar Todos los Estados
import-exchange-hint-feather = Exportar desde Feather Wallet: Historial > Exportar CSV
import-exchange-hint-monero-gui = Exportar desde Monero GUI: Wallet > Historial > Exportar CSV
import-exchange-hint-notbank = NotBank (CryptoMarket): Exchange Pro > Reports > Single Report > Transaction, Trade Activity y Profit and Loss (puedes subir los tres juntos).
import-exchange-coin-added = Moneda {$symbol} agregada. Reintentando importación del exchange...
import-exchange-coin-add-failed = No se pudo agregar la moneda {$symbol}: {$reason}
import-exchange-coin-invalid = Símbolo inválido para creación automática de moneda: {$symbol}
import-exchange-coin-retry-unavailable = No hay una importación de exchange pendiente para reintentar.
import-exchange-hint-mexc = MEXC: Help Center > Account Data Export > seleccionar reportes necesarios > convertir a CSV. Soporta 17 tipos de reportes CSV y puedes subir varios archivos a la vez.
settings-exchange-import = IMPORTAR EXCHANGE
settings-exchange-import-desc = Importar transacciones crypto desde archivos CSV de exchanges

# ==================== Seleccion de Wallet para Exchange ====================
exchange-wallet-select-title = WALLET DESTINO
exchange-wallet-select-subtitle = Selecciona un wallet existente o crea uno nuevo para las transacciones importadas
exchange-wallet-tab-select = SELECCIONAR WALLET
exchange-wallet-tab-create = CREAR NUEVO
exchange-wallet-select-label = WALLETS DISPONIBLES
exchange-wallet-no-wallets = No se encontraron wallets. Cambia a la pestana de crear para agregar uno.
exchange-wallet-select-required = Selecciona un wallet para continuar
exchange-wallet-name-required = El nombre del wallet es requerido
exchange-wallet-continue = CONTINUAR
exchange-wallet-category-software = Software Wallet
exchange-wallet-category-hardware = Hardware Wallet
exchange-wallet-category-exchange = Exchange
