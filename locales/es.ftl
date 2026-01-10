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
finances-title = Finanzas
finances-accounts = Cuentas
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
crypto-wallets = Billeteras
crypto-assets = Activos
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

# ==================== Habits ====================
habits-title = Hábitos
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
settings-title = Ajustes
settings-general = General
settings-appearance = Apariencia
settings-security = Seguridad
settings-data = Datos
settings-about = Acerca de

# General settings
settings-language = Idioma
settings-language-desc = Idioma de la interfaz
settings-currency = Moneda
settings-currency-desc = Moneda predeterminada

# Appearance settings
settings-dark-mode = Modo Oscuro
settings-dark-mode-desc = Activar tema oscuro

# Security settings
settings-session-timeout = Tiempo de Sesión
settings-session-timeout-desc = Bloqueo automático por inactividad
settings-timeout-5min = 5 minutos
settings-timeout-15min = 15 minutos
settings-timeout-30min = 30 minutos
settings-timeout-1hour = 1 hora
settings-timeout-never = Nunca

# Crypto settings
settings-auto-fetch = Actualizar Precios
settings-auto-fetch-desc = Actualizar precios de crypto automáticamente
settings-proxy = Proxy
settings-proxy-enabled = Habilitar Proxy
settings-proxy-url = URL del Proxy

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
