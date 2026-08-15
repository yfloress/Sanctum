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
action-delete = Eliminar
action-edit = Editar
action-close = Cerrar
action-undo = Deshacer

# Time
time-today = Hoy

# ==================== Login Page ====================
login-subtitle = Tu boveda personal privada
login-placeholder-unlock = Ingresa tu contrasena maestra
login-placeholder-create = Crea tu contrasena maestra
login-unlock = Desbloquear
login-create = Crear Boveda
login-confirm-create = Confirmar Creación
login-authenticating = Autenticando...
login-initializing = Inicializando...
login-weak-hint = Presiona de nuevo para confirmar con contrasena debil
login-restore = Restaurar desde respaldo
login-version = Sanctum v0.1.0

# ==================== Sidebar ====================
nav-dashboard = Panel
nav-finances = Finanzas
nav-crypto = Crypto
nav-settings = Ajustes
nav-lock = Bloquear
nav-collapse = Contraer
nav-expand = Expandir
nav-group-overview = Resumen
nav-group-vault = Bóveda
nav-group-system = Sistema
nav-hide-balances = Ocultar Saldos
nav-show-balances = Mostrar Saldos
nav-menu = Menú
nav-close = Cerrar

# ==================== Dashboard ====================
dashboard-retry = Reintentar
dashboard-net-worth = Patrimonio Neto
dashboard-fiat = Fiat
dashboard-crypto = Crypto
dashboard-income = Ingresos
dashboard-expenses = Gastos
dashboard-net = Neto
dashboard-last = ultimo
dashboard-net-worth-trend = Tendencia Patrimonial
dashboard-monthly-cash-flow = Flujo de Caja Mensual
dashboard-last-6-months = Ultimos 6 meses
dashboard-no-data-range = Sin datos para este rango
dashboard-spending-breakdown = Desglose de Gastos
dashboard-recent-activity = Actividad Reciente
dashboard-welcome = Bienvenido a Sanctum
dashboard-welcome-desc = Agrega cuentas y transacciones en la página de Finanzas para ver tu resumen aquí.

# Tabs & hero
finances-total-balance = Saldo Total
finances-tab-overview = Resumen
finances-tab-activity = Actividad
finances-tab-accounts = Cuentas
finances-tab-credits = Créditos
finances-tab-settings = Configuración

# Overview stats
finances-income-this-month = Ingresos este mes
finances-expenses-this-month = Gastos este mes
finances-net-this-month = Neto este mes

# Overview charts
finances-monthly-overview = Resumen Mensual
finances-balance-distribution = Distribución de Saldos
finances-no-positive-balances = Sin saldos positivos para mostrar
finances-expenses-by-category = Gastos por Categoría

# Overview accounts section
finances-accounts = Cuentas
finances-transfer = Transferir
finances-new-account = Nueva Cuenta
finances-no-accounts = Sin cuentas aun.
finances-recent-transactions = Transacciones Recientes
finances-view-all = Ver Todo
finances-no-transactions = Sin transacciones aun.

# Activity
finances-search-placeholder = Buscar transacciones...
finances-all-accounts = Todas las Cuentas
finances-all-categories = Todas las Categorías
finances-date-range = Rango de fechas
finances-date-all = Todo el tiempo
finances-date-this-month = Este mes
finances-date-last-30 = Últimos 30 días
finances-date-last-90 = Últimos 90 días
finances-date-this-year = Este año
finances-date-custom = Rango personalizado
finances-date-from = Desde
finances-date-to = Hasta
finances-clear = Limpiar
finances-new-entry = Nueva Entrada
finances-no-matching = Sin transacciones que coincidan
finances-no-transactions-yet = Sin transacciones aun
finances-load-more = Cargar Mas

# Accounts tab
finances-my-accounts = Mis Cuentas
finances-no-accounts-create = Sin cuentas aun. Crea tu primera cuenta.

# Settings/Categories
finances-new-category = Nueva Categoría
finances-category-placeholder = Nombre de categoría...
finances-expense = Gasto
finances-expenses = Gastos
finances-income = Ingreso
finances-add = Agregar

# Detail panel
finances-change-icon = Cambiar Icono
finances-close = Cerrar
finances-type = Tipo
finances-currency = Moneda
finances-balance = Saldo
finances-edit-account = Editar Cuenta
finances-delete-account = Eliminar Cuenta
finances-reconcile = Conciliar
finances-tags = Etiquetas
finances-tags-placeholder = chatarra, trabajo...
finances-tag-remove = Quitar {$tag}
finances-all-tags = Todas las Etiquetas
finances-bulk-tag = Agregar etiqueta
finances-bulk-tagged = {$count} etiquetadas

# Reconciliation
reconcile-title = Conciliar {$account}
reconcile-hint = Escribe el saldo que muestra tu banco y marca las filas que aparecen en tu cartola.
reconcile-statement = Saldo según tu banco
reconcile-marked = Marcado en Sanctum
reconcile-difference = Diferencia
reconcile-balanced = Todo calza.
reconcile-nothing-pending = No queda nada por confirmar en esta cuenta.
reconcile-confirm = Confirmar {$count}
reconcile-done = {$count} confirmadas
reconcile-confirmed = Confirmada contra tu banco

# Transaction modal
finances-edit-transaction = Editar Transacción
finances-add-transaction = Agregar Transacción
finances-account = Cuenta
finances-amount = Monto
finances-category = Categoría
finances-select = Seleccionar...
finances-description = Descripción
finances-date = Fecha
finances-cancel = Cancelar
finances-update = Actualizar
finances-add-btn = Agregar

# Account modal
finances-edit-account-modal = Editar Cuenta
finances-new-account-modal = Nueva Cuenta
finances-name = Nombre
finances-account-name-placeholder = Nombre de cuenta
finances-account-type-bank = Banco
finances-account-type-savings = Ahorros
finances-account-type-credit = Tarjeta de Crédito
finances-account-type-cash = Efectivo
finances-account-type-other = Otro
finances-initial-balance = Saldo Inicial
finances-icon = Ícono
finances-change = Cambiar
finances-create = Crear

# Transfer modal
finances-edit-transfer = Editar Transferencia
finances-transfer-funds = Transferir Fondos
finances-from = Desde
finances-to = Hacia
finances-transfer-note = Nota de transferencia
finances-transfer-btn = Transferir

# Toast messages
finances-tx-added = Transacción agregada
finances-tx-updated = Transacción actualizada
finances-tx-deleted = Transacción eliminada
finances-tx-restored = Transacción restaurada
finances-acc-created = Cuenta creada
finances-acc-updated = Cuenta actualizada
finances-acc-deleted = Cuenta eliminada
finances-acc-restored = Cuenta restaurada
finances-archived-accounts = Cuentas Archivadas
finances-restore = Restaurar
finances-duplicate = Duplicar
finances-tf-completed = Transferencia completada
finances-tf-updated = Transferencia actualizada
finances-cat-added = Categoría agregada
finances-cat-deleted = Categoría eliminada
finances-cat-restored = Categoría restaurada

# -- Tabs & Hero --
crypto-tab-portfolio = Portafolio
crypto-tab-wallets = Billeteras
crypto-tab-tax = Impuestos
crypto-tab-activity = Actividad
crypto-portfolio-value = Valor del Portafolio
crypto-last-updated-label = Última actualización: {$value}
crypto-welcome = Bienvenido a Crypto
crypto-welcome-desc = Agrega billeteras y transacciones en la pestaña Billeteras para empezar a seguir tu portafolio.

# -- Ticker bar --
crypto-no-tickers = Sin tickers configurados
crypto-sync-prices = Sincronizar precios
crypto-configure-ticker = Configurar ticker

# -- Portfolio tab --
crypto-new-transaction = Nueva Transacción
crypto-unrealized-pnl = Ganancias No Realizadas
crypto-realized-ytd = Realizadas (Anual)
crypto-roi = ROI
crypto-portfolio-trend = Tendencia del Portafolio
crypto-distribution = Distribución
crypto-recent-transactions = Transacciones Recientes
crypto-no-transactions = Sin transacciones aun.
crypto-search-transactions = Buscar transacciones...
crypto-no-matching = Sin transacciones coincidentes
crypto-load-more = Cargar Mas
crypto-no-assets-empty = Sin activos aun. Crea una billetera y agrega transacciones para comenzar.

# -- Wallets tab --
crypto-wallets-title = Billeteras
crypto-add-wallet = Agregar Billetera
crypto-no-wallets = Sin billeteras aun.
crypto-wallet-assets-one = activo
crypto-wallet-assets-other = activos
crypto-delete-wallet = Eliminar Billetera

# -- Wallet detail panel --
crypto-click-rename = Click para renombrar
crypto-holdings = Tenencias
crypto-transactions = Transacciones
crypto-save = Guardar
crypto-cancel = Cancelar
crypto-saving = Guardando...
crypto-edit = Editar
crypto-delete = Eliminar
crypto-duplicate = Duplicar
crypto-all-wallets = Todas las Billeteras
crypto-all-types = Todos los Tipos
crypto-type-trade = Operación
crypto-type-income = Ingreso
crypto-type-expense = Gasto
crypto-type-transfer = Transferencia
crypto-toast-tx-duplicated = Transacción duplicada
crypto-change = Cambiar
crypto-close = Cerrar
crypto-wallet-icon = Ícono de Billetera
crypto-change-icon = Cambiar Ícono

# -- Asset detail panel --
crypto-amount = Cantidad
crypto-value = Valor
crypto-allocation = Asignación

# -- Tax tab --
crypto-tax-period-id = Periodo Tributario
crypto-tax-period-placeholder = ej., 2024
crypto-tax-load-settings = Cargar Configuración
crypto-tax-jurisdiction = Jurisdicción
crypto-tax-method = Metodo
crypto-tax-generate-report = Generar Reporte

# -- Tax report --
crypto-tax-report-summary = Resumen del Reporte
crypto-tax-disposals = Enajenaciones
crypto-tax-total-proceeds = Ingresos Totales
crypto-tax-total-cost = Costo Total
crypto-tax-total-gain = Ganancia Total
crypto-tax-short-term = Ganancia Corto Plazo
crypto-tax-long-term = Ganancia Largo Plazo
crypto-tax-warnings = Advertencias
crypto-tax-readiness = Preparación
crypto-tax-events = Eventos (mostrando primeros 50)
crypto-tax-col-date = Fecha
crypto-tax-col-coin = Moneda
crypto-tax-col-amount = Cantidad
crypto-tax-col-proceeds = Ingresos
crypto-tax-col-cost-basis = Costo Base
crypto-tax-col-gain = Ganancia
crypto-tax-col-term = Plazo
crypto-tax-export-events = Exportar Eventos CSV
crypto-tax-export-history = Exportar Historial CSV

# -- IPC import --
crypto-ipc-label = Historial de Precios IPC
crypto-ipc-no-data = Sin datos IPC importados
crypto-ipc-import = Importar IPC CSV
crypto-ipc-desc = Descarga la serie oficial del IPC, conviértela a CSV e impórtala aquí. No se realizan conexiones de red por defecto.

# -- Tax settings modal --
crypto-tax-settings-title = Configuración Tributaria
crypto-tax-jurisdiction-us = Estados Unidos
crypto-tax-jurisdiction-cl = Chile
crypto-tax-jurisdiction-other = Otro
crypto-tax-method-fifo = FIFO
crypto-tax-method-lifo = LIFO
crypto-tax-method-hifo = HIFO
crypto-tax-method-avg = Costo Promedio
crypto-tax-method-chile-hint = Chile (SII) solo acepta FIFO y Costo Promedio.
crypto-tax-method-usa-hint = USA acepta FIFO e Identificación Específica (LIFO/HIFO); el costo promedio no se permite para cripto.
crypto-tax-include-swaps-label = Incluir Intercambios como Enajenación
crypto-tax-include-fee-label = Incluir Comisión Crypto como Enajenación
crypto-tax-exclude-wallets = Excluir Billeteras
crypto-tax-loading-settings = Cargando…
crypto-tax-regenerate = Regenerar
crypto-tax-onboarding-title = Declaración Tributaria
crypto-tax-onboarding-desc = Genera un reporte tributario de tus transacciones crypto. Sigue los pasos a continuación para comenzar.
crypto-tax-step1-title = Ingresa el año tributario
crypto-tax-step1-desc = Escribe el año a declarar (ej. 2024) y carga la configuración.
crypto-tax-step2-title = Configura jurisdicción y método
crypto-tax-step2-desc = Selecciona tu jurisdicción tributaria, método de costo base y opciones adicionales. Para Chile, importa los datos IPC.
crypto-tax-step3-title = Genera y exporta
crypto-tax-step3-desc = Genera el reporte, revisa advertencias, completa precios faltantes y exporta el CSV para tu declaración.
crypto-tax-chile-info-title = Notas Tributarias Chile
crypto-tax-chile-ipc = Los ajustes por IPC (corrección monetaria) se aplican automáticamente al costo base y las ganancias.
crypto-tax-chile-clp = Todos los valores del reporte se muestran en Pesos Chilenos (CLP). Para la declaración, usa el Dólar Observado publicado por el SII.
crypto-tax-chile-f22 = Declara en Formulario 22, Línea 10, código 1032 (mayor valor, otras rentas). Verifica los códigos vigentes en el suplemento tributario del SII.
crypto-tax-chile-exemption = La renta anual neta inferior a 13,5 UTA (~$11,3M CLP en 2026) está exenta de IGC.
crypto-tax-chile-fees = El tratamiento de comisiones puede variar. Consulta a un asesor tributario chileno para tu situación específica.
crypto-tax-beta-badge = Beta
crypto-tax-disclaimer = El reporte de impuestos es experimental. Las cifras son estimaciones, no asesoría tributaria — verifica los resultados con un profesional antes de declarar.
crypto-tax-exclude-wallets-desc = Las billeteras que excluyas quedan fuera del cálculo de impuestos (p. ej. billeteras de DeFi o solo para donaciones).
crypto-tax-no-wallets = No hay billeteras para excluir.
crypto-tax-excluded-suffix = billetera(s) excluida(s)
crypto-tax-saved = Guardado
crypto-tax-report-stale = Cambiaste ajustes desde que se generó este reporte. Regenéralo para aplicarlos.
crypto-tax-no-disposals = No hay enajenaciones gravables en este período. Los ingresos, transferencias y tenencias sin vender no generan ganancia hasta que vendas o hagas swap.
crypto-tax-export-title = Exportar CSV de impuestos
crypto-tax-taxable-income = Renta Gravable
crypto-tax-end-balance = Saldo Fin de Período
crypto-tax-tx-in-period = Transacciones en el Período
crypto-tax-volume = Volumen Procesado
crypto-tax-fetch-price = Obtener precio
crypto-tax-fetching = Obteniendo…
crypto-tax-toast-price-filled = Precio completado exitosamente

# -- Add wallet modal --
crypto-new-wallet = Nueva Billetera
crypto-wallet-name = Nombre
crypto-wallet-name-placeholder = Nombre de billetera
crypto-wallet-category = Categoria
crypto-wallet-create = Crear

# -- Ticker config modal --
crypto-ticker-tab = Ticker
crypto-coins-tab = Monedas
crypto-ticker-active = Activos — usa flechas para reordenar
crypto-ticker-no-selected = Sin tickers seleccionados.
crypto-ticker-add-coins = Agregar monedas
crypto-ticker-search = Buscar monedas...
crypto-ticker-save = Guardar
crypto-fx-stale = Tasa desactualizada — sincroniza para refrescarla

# -- Coin catalog --
crypto-custom-coin = Agregar Moneda Personalizada
crypto-custom-id = ID
crypto-custom-name = Nombre
crypto-custom-symbol = Simbolo
crypto-custom-add = Agregar

# -- Transaction modal --
crypto-tx-title = Nueva Transacción
crypto-tx-buy = Compra
crypto-tx-sell = Venta
crypto-tx-income = Ingreso
crypto-tx-fee = Comision
crypto-tx-transfer = Transferencia
crypto-tx-swap = Intercambio
crypto-tx-coin = Moneda
crypto-tx-search-coin = Buscar moneda...
crypto-tx-wallet = Billetera
crypto-tx-from-wallet = Billetera Origen
crypto-tx-to-wallet = Billetera Destino
crypto-tx-amount = Cantidad
crypto-tx-received-amount = Cantidad Recibida (opcional)
crypto-tx-received-placeholder = Igual al envio si vacio
# Transaction action labels (in list rows)
crypto-tx-received = Recibido {$detail}
crypto-tx-sent = Enviado {$detail}
crypto-tx-transferred = Transferido {$detail}
crypto-tx-sold = Vendido {$detail}
crypto-tx-swapped = Intercambiado {$detail}
crypto-tx-bought = Comprado {$detail}
crypto-tx-from-coin = Moneda Origen
crypto-tx-from-amount = Cantidad Origen
crypto-tx-to-coin = Moneda Destino
crypto-tx-to-amount = Cantidad Destino
crypto-tx-price = Precio (por moneda)
crypto-tx-fee-label = Comision
crypto-tx-date = Fecha
crypto-tx-notes = Notas (opcional)
crypto-tx-notes-placeholder = Notas...
crypto-tx-add = Agregar
crypto-tx-edit-title = Editar Transacción
crypto-tx-subtype = Subtipo
crypto-tx-fee-coin-id = Moneda Comision (opcional)
crypto-tx-fee-coin-amount = Cantidad Comision (opcional)
crypto-tx-override-proceeds = Forzar Ingresos (opcional)
crypto-tx-override-cost-basis = Forzar Costo Base (opcional)

# -- Toast messages --
crypto-toast-ticker-saved = Configuración del ticker guardada
crypto-toast-no-coins-sync = Sin monedas para sincronizar. Configura el ticker primero.
crypto-toast-custom-added = Moneda personalizada agregada
crypto-toast-custom-deleted = Moneda personalizada eliminada
crypto-toast-tx-added = Transacción agregada
crypto-toast-tx-updated = Transacción actualizada
crypto-toast-tx-deleted = Transacción eliminada
crypto-toast-tx-restored = Transacción restaurada
crypto-toast-wallet-restored = Billetera restaurada
crypto-toast-custom-restored = Moneda personalizada restaurada
crypto-toast-wallet-created = Billetera creada
crypto-toast-wallet-deleted = Billetera eliminada
crypto-toast-wallet-renamed = Billetera renombrada
crypto-toast-ipc-imported = Datos IPC importados
crypto-toast-settings-saved = Configuración guardada
crypto-toast-enter-period = Ingresa un periodo tributario
crypto-toast-exported = Exportado a {$path}

# ==================== Settings ====================
settings-title = Ajustes

# Section headers
settings-appearance = Apariencia
settings-regional = Regional
settings-security = Seguridad
settings-vault-backup = Respaldo de Boveda
settings-data-import = Importar Datos
settings-data-sync = Sincronización
settings-about = Acerca de
settings-reset-section = Restablecer

# Appearance
settings-dark-mode = Modo Oscuro
settings-dark-mode-desc = Alternar tema oscuro/claro
settings-background = Fondo
settings-background-desc = Elige el estilo del fondo de la aplicación
settings-bg-aurora = Aurora
settings-bg-diamonds = Diamantes
settings-bg-dots = Puntos
settings-bg-dragon = Dragón
settings-bg-stars = Estrellas

# Regional
settings-preferred-currency = Moneda Preferida
settings-language = Idioma

# Security
settings-session-timeout = Tiempo de Sesion
settings-session-timeout-desc = Bloqueo automatico por inactividad
settings-timeout-5min = 5 minutos
settings-timeout-15min = 15 minutos
settings-timeout-30min = 30 minutos
settings-timeout-1hour = 1 hora

# Vault Backup
settings-vault-note = Tu boveda esta cifrada con SQLCipher (AES-256).
settings-export-vault = Exportar Boveda
settings-last-backup = Último respaldo
settings-last-backup-never = nunca
settings-last-backup-days = hace {$count} días
settings-export-transactions = Exportar Transacciones
settings-export-transactions-desc = CSV plano de todo tu ledger, sin cifrar
settings-export-csv-done = {$count} transacciones exportadas
settings-export-btn = Exportar
settings-export-success = Respaldo guardado exitosamente

# Data Import
settings-import-generic = CSV Generico
settings-import-generic-desc = Importar transacciones desde un archivo CSV
settings-import-exchange = CSV de Exchange / Wallet
settings-import-exchange-desc = Importar desde Kraken, Binance, MEXC, Feather, Monero…
settings-import-custom = CSV Personalizado (mapeo manual)
settings-import-custom-desc = Importar desde cualquier otro exchange asignando sus columnas
settings-import-custom-intro = Asigna cada campo de Sanctum a una columna de tu CSV. Fecha, activo y monto son obligatorios.
settings-import-custom-preview = Vista previa de columnas (primera fila)
settings-import-custom-select = — Selecciona columna —
settings-import-custom-none = — Ninguna —
settings-import-custom-no-wallets = Primero crea una billetera en la sección Cripto.
settings-import-custom-date = Fecha
settings-import-custom-asset = Activo (moneda)
settings-import-custom-amount = Monto
settings-import-custom-type = Tipo
settings-import-custom-fee = Comisión
settings-import-custom-fee-currency = Moneda de la comisión
settings-import-custom-price = Precio
settings-import-custom-notes = Notas
settings-import-select-file = Seleccionar Archivo
settings-import-loading = Cargando...
settings-import-detected = Detectado:
settings-import-records = registros
settings-import-target-wallet = Billetera Destino
settings-import-wallet-placeholder = Nombre de billetera
settings-import-wallet-required = El nombre de billetera es obligatorio
settings-import-no-detection = No se pudo detectar el formato del exchange
settings-import-preview-btn = Vista Previa
settings-import-to-add = por agregar
settings-import-to-skip = por omitir
settings-import-importing = Importando...
settings-import-confirm = Confirmar Importación
settings-import-processed = Procesados:
settings-import-inserted = Insertados:
settings-import-skipped = Omitidos:
settings-import-errors = Errores
settings-import-line = Linea
settings-import-done = Listo

# Data Sync
settings-auto-fetch = Auto-actualizar Precios
settings-auto-fetch-desc = Obtener precios de crypto automaticamente al sincronizar
settings-use-proxy = Usar Proxy
settings-use-proxy-desc = Enrutar llamadas API a traves de un proxy
settings-proxy-url = URL del Proxy
settings-proxy-placeholder = socks5://127.0.0.1:9050

# About
settings-about-version = Versión
settings-about-encryption = Cifrado
settings-about-storage = Almacenamiento

# Reset
settings-reset-all = Restablecer Todos los Ajustes
settings-reset-all-desc = Restaurar valores predeterminados para todos los ajustes
settings-reset-btn = Restablecer
settings-reset-success = Ajustes restablecidos a valores predeterminados

# Common actions
settings-cancel = Cancelar

# Confirmation dialogs
confirm-delete-title = Confirmar Eliminación
confirm-delete-message = Esta acción no se puede deshacer.
confirm-delete-button = Eliminar
confirm-delete-account = ¿Estás seguro de que quieres eliminar esta cuenta?
confirm-delete-account-tx-count = Esto también eliminará {$count} transacción(es).
confirm-delete-category = ¿Estás seguro de que quieres eliminar esta categoría?
confirm-delete-transaction = ¿Estás seguro de que quieres eliminar esta transacción?
confirm-reset-settings = ¿Restablecer todos los ajustes a sus valores predeterminados? Esto no afectará tus datos.

# ==================== Importación de Datos ====================

import-errors = Errores

import-error-account-not-found = Cuenta no encontrada: { $name }
import-error-currency-mismatch-detail = Moneda incompatible: la importación tiene { $import } pero la cuenta { $account } usa { $expected }
import-error-category-not-found-detail = Categoría no encontrada: { $name } (tipo: { $type })
import-error-destination-account-not-found = Cuenta destino no encontrada: { $name }
import-error-same-account-transfer = No se puede transferir a la misma cuenta
import-error-wallet-not-found = Wallet no encontrada: { $name }
import-error-insufficient-crypto-balance = Balance insuficiente de { $symbol } en { $wallet }: tiene { $available }, necesita { $required }
import-skipped-duplicate-transaction = Transacción duplicada (misma fecha/cuenta/monto/tipo/descripción)
import-skipped-duplicate-crypto = Transacción crypto duplicada (misma fecha/wallet/moneda/tipo/monto)
import-skipped-crypto-not-found = Activo crypto no encontrado en el catálogo (fila omitida)

import-preview-change-income = Ingreso
import-preview-change-expense = Gasto
import-preview-change-transfer = Transferencia
import-preview-change-crypto = Transacción Crypto

# ==================== Importación CSV de Exchanges ====================
import-exchange-hint-kraken = Kraken: Documents > exportar los CSV Ledgers y Trades (puedes subir ambos juntos).
import-exchange-hint-kraken-pro = Kraken Pro: History > Statements > exportar los CSV Ledgers y Trades (puedes subir ambos juntos).
import-exchange-hint-binance = Exportar desde Binance: Órdenes > Historial de Transacciones > Generar Todos los Estados
import-exchange-hint-feather = Exportar desde Feather Wallet: Historial > Exportar CSV
import-exchange-hint-monero-gui = Exportar desde Monero GUI: Wallet > Historial > Exportar CSV
settings-import-exchange-help = Cómo exportar desde cada exchange o wallet
settings-import-wallet-missing = La wallet "{$name}" no existe todavía. ¿Crearla?
settings-import-create-wallet-preview = Crear wallet y previsualizar
import-exchange-hint-notbank = NotBank (CryptoMarket): Exchange Pro > Reports > Single Report > Transaction y Trade Activity (puedes subir ambos juntos).
import-exchange-hint-mexc = MEXC: Help Center > Account Data Export > seleccionar reportes necesarios > convertir a CSV. Soporta 17 tipos de reportes CSV y puedes subir varios archivos a la vez.

# ==================== Session Lock Warning ====================
session-warning-title = La bóveda está por bloquearse
session-warning-body = Se bloqueará en {$seconds} s por inactividad.
session-warning-stay = Seguir abierta
session-warning-lock-now = Bloquear ahora
session-locked = Bóveda bloqueada

# ==================== Master Password ====================
settings-change-password = Contraseña Maestra
settings-change-password-desc = Recifra toda la bóveda con una contraseña nueva
settings-change-password-btn = Cambiar
settings-password-current = Contraseña actual
settings-password-new = Contraseña nueva
settings-password-confirm = Confirmar contraseña nueva
settings-password-mismatch = Las contraseñas nuevas no coinciden
settings-password-changing = Recifrando…
settings-password-changed = Contraseña maestra cambiada
settings-password-rollback-at = Copia de rollback, aún con la contraseña antigua:
settings-password-backup-warning = Primero se guarda un respaldo, y es obligatorio. Ese respaldo conserva la contraseña ANTIGUA: restaurarlo más adelante requiere la contraseña que estás reemplazando ahora.

# el idioma de la interfaz. Las que crea el usuario se muestran tal cual.
category-food = Comida
category-transport = Transporte
category-utilities = Servicios
category-entertainment = Entretenimiento
category-health = Salud
category-shopping = Compras
category-education = Educación
category-other = Otros
category-salary = Sueldo
category-freelance = Freelance
category-investment = Inversión
category-gift = Regalo
category-transfer = Transferencia

# ==================== Recurring Entries ====================
finances-recurring = Entradas Recurrentes
finances-recurring-desc = Se crean solas en su fecha. Si abres la app después de un tiempo, se completa todo lo pendiente.
finances-recurring-new = Nueva
finances-recurring-frequency = Frecuencia
finances-recurring-weekly = Semanal
finances-recurring-monthly = Mensual
finances-recurring-yearly = Anual
finances-recurring-first = Primera vez
finances-recurring-next = Próxima
finances-recurring-paused = Pausada
finances-recurring-pause = Pausar
finances-recurring-resume = Reanudar
finances-recurring-added = Entrada recurrente guardada
finances-recurring-deleted = Entrada recurrente eliminada
finances-recurring-delete-confirm = ¿Eliminar esta entrada recurrente? Las transacciones que ya creó se conservan.
finances-recurring-applied = {$count} transacciones recurrentes agregadas

# ==================== Monthly Budgets ====================
finances-budgets = Presupuestos Mensuales
finances-budgets-desc = Un límite de gasto por categoría. El avance cubre el mes en curso y se reinicia el día 1.
finances-budget-new = Nuevo
finances-budget-saved = Presupuesto guardado
finances-budget-left = disponible
finances-budget-over = Excedido por
finances-no-budgets = Sin presupuestos aún.

# ==================== Credits ====================
finances-credits = Créditos
finances-credits-desc = Deudas que pagas en una cantidad fija de pagos con fecha. Marcar uno como pagado escribe el gasto en la cuenta de la que sale.
finances-credit-new = Nuevo
finances-credit-name = ¿Qué compraste?
finances-credit-installment = Monto de la cuota
finances-credit-count = Cuántas
finances-credit-first-due = Primera vence el
finances-credit-cash-price = Precio si lo pagaras todo de una vez (opcional)
finances-credit-total = Total
finances-credit-extra = Pagas {$amount} más de lo que cuesta la cosa
finances-credit-added = Crédito guardado
finances-credit-deleted = Crédito eliminado
finances-credit-delete-confirm = ¿Eliminar este crédito? Los pagos ya hechos se conservan en tu registro.
finances-no-credits = Sin créditos aún.
finances-credit-installments = cuotas
finances-credit-left = por pagar
finances-credit-next = Próxima
finances-credit-interest = Interés
finances-credit-done = Terminado
finances-credit-overdue = Vencidas: {$count}
finances-credit-ahead = Adelantado
finances-credit-on-track = Al día
finances-credit-pay = Pagar {$amount}
finances-credit-pay-short = Pagar
finances-credit-undo = Deshacer
finances-credit-paid = Cuota pagada
finances-credit-undone = Pago deshecho
finances-credit-show-schedule = Cuotas
finances-credit-hide-schedule = Ocultar cuotas
finances-credit-inst-paid = Pagada
finances-credit-inst-overdue = Vencida
finances-credit-inst-pending = Pendiente

finances-credit-kind-installments = Me dijeron la cuota
finances-credit-kind-loan = Me dijeron la tasa
finances-credit-down-payment = Pie
finances-credit-down-payment-optional = Pago inicial (opcional)
finances-credit-down-payment-date = Entregado el
finances-credit-principal = Monto financiado
finances-credit-rate = Tasa de interés (%)
finances-credit-rate-period = Expresada como
finances-credit-rate-monthly = Mensual
finances-credit-rate-annual = Anual
finances-credit-rate-monthly-short = mensual
finances-credit-suggested = Calculada a partir de la tasa. Si tu contrato dice otra cosa, escribe esa.
finances-credit-pay-next = Pagar siguiente
finances-credit-edit = Editar
finances-credit-edit-title = Corregir esta cuota
finances-credit-edit-desc = Para planes cuyas cuotas no son todas iguales: una final más grande, un mes sin pago, una cifra que salió distinta.
finances-credit-updated = Cuota actualizada
finances-credit-due-date = Vence el
finances-credit-charge = Recargo
finances-credit-charges = Recargos
finances-credit-add-charge = Agregar recargo
finances-credit-charge-title = Registrar un recargo
finances-credit-charge-desc = Un cobro por atraso o interés que agregó el acreedor. Escribe lo que efectivamente te cobraron: las reglas y las cifras detrás cambian según el país y el acreedor, así que la app no las adivina.
finances-credit-charge-amount = Monto cobrado
finances-credit-charge-note = ¿Por qué fue?
finances-credit-charge-added = Recargo registrado
finances-credit-breakdown = Desglose
finances-credit-col-payment = Cuota
finances-credit-col-interest = Interés
finances-credit-col-principal = Capital
finances-credit-col-balance = Saldo
finances-save = Guardar
# ==================== Activity Sorting ====================
finances-sort = Ordenar por
finances-sort-date-desc = Más recientes primero
finances-sort-date-asc = Más antiguas primero
finances-sort-amount-desc = Monto mayor
finances-sort-amount-asc = Monto menor

# ==================== Entry Form ====================
finances-date-today = Hoy
finances-date-yesterday = Ayer

# ==================== Bulk Actions ====================
finances-select-row = Seleccionar transacción
finances-bulk-selected = {$count} seleccionadas
finances-bulk-move = Mover a categoría
finances-bulk-deleted = {$count} transacciones eliminadas
finances-bulk-moved = {$count} transacciones movidas
finances-bulk-move-undone = Categorías restauradas
finances-bulk-restored = {$count} transacciones restauradas
finances-bulk-restored-partial = Se restauraron {$count} de {$total} transacciones
confirm-delete-transactions = ¿Eliminar {$count} transacciones?

# ==================== Command Palette ====================
palette-title = Paleta de comandos
palette-placeholder = Escribe un comando o busca en tus datos...
palette-no-results = Sin resultados
search-kind-account = Cuenta
search-kind-category = Categoría
search-kind-coin = Moneda
search-kind-transaction = Transacción
search-kind-wallet = Billetera

# ==================== Keyboard Shortcuts ====================
shortcuts-title = Atajos de Teclado
shortcuts-group-navigation = Navegación
shortcuts-group-actions = Acciones
shortcuts-group-dialogs = Diálogos
shortcuts-toggle-sidebar = Contraer o expandir la barra lateral
shortcuts-new-entry = Nueva entrada en la página actual
shortcuts-search = Ir al campo de búsqueda
shortcuts-lock = Bloquear la bóveda ahora
shortcuts-confirm = Confirmar el formulario abierto
shortcuts-close = Cerrar sin guardar
shortcuts-help = Mostrar esta lista
shortcuts-palette = Abrir la paleta de comandos
