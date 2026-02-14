# Módulo de Impuestos Crypto

Guía de referencia del motor tributario en `src/features/crypto/tax/`.

## Arquitectura

```
src/features/crypto/tax/
├── engine/
│   ├── mod.rs      # build_tax_report() — orquestador principal
│   ├── lots.rs     # Gestión de lotes, enajenaciones, ajuste IPC
│   ├── swaps.rs    # Resolución de pares swap y enajenación
│   ├── period.rs   # Helpers de año calendario, prev_month_key
│   └── types.rs    # Lot, TaxConfig, DisposalRequest, TaxPeriod
├── ipc.rs          # Parser CSV de IPC (formato INE, meses en español, flexible)
├── report.rs       # TaxReport, TaxDisposal, LotAllocation, exportación CSV
├── summary.rs      # TaxSummaryPayload, TaxReadinessItem
├── types.rs        # TaxJurisdiction, TaxMethod, TaxPeriodSettings, TaxTxType
└── mod.rs          # Re-exportaciones y helpers resolve_type/resolve_subtype
```

Capa de servicio: `src/features/crypto/service_tax.rs`
Callbacks de UI: `src/ui/callbacks/crypto/tax.rs`

## Jurisdicciones

### Chile (SII)

**Impuesto aplicable:** Impuesto Global Complementario (IGC) para personas naturales.

**Fórmula de ganancia:**

```
ganancia = beneficio − costo_de_adquisición_reajustado
ganancia_final = ganancia × (IPC_noviembre / IPC_mes_anterior_venta)
```

Hay **dos** ajustes IPC obligatorios:

1. **Costo de adquisición**: se reajusta por IPC desde el mes anterior a la
   compra hasta el mes anterior a la venta.
2. **Ganancia resultante**: se reajusta por IPC desde el mes anterior a la venta
   hasta noviembre (mes anterior al cierre del año tributario, 31 de diciembre).

**Comisiones (fees):** Las personas naturales sin contabilidad completa **no
pueden** deducir comisiones como costo de adquisición ni como gasto. El fee en
crypto sí genera una enajenación (evento tributable separado).

**Airdrops / Staking / Forks:** Se reconocen con costo **$0** al recibirlos
(Oficio Ord. Nº979/2022). La ganancia completa se realiza al enajenar.

**Minería:** Se declara primero bajo Impuesto de Primera Categoría (IDPC) y
luego la enajenación tributa bajo IGC.

**Métodos de valorización:** FIFO, LIFO, HIFO, CPP — libre elección para
personas naturales bajo IGC. Para contribuyentes de 1ª Categoría debe ser FIFO
o CPP, manteniendo consistencia al menos 5 años.

**Tipo de cambio:** Dólar Observado del Banco Central / SII. En fines de semana
o feriado se usa el día hábil inmediatamente siguiente.

**No tributa:** Comprar con fiat, HODL, transferir entre wallets propias
(excepto la comisión).

**No hay IVA** en compra/venta de criptomonedas.

### Declaración en el SII (Formulario 22)

> **Warning: Los códigos de casilla pueden cambiar año a año.** Siempre verificar en
> el suplemento tributario del SII correspondiente al Año Tributario vigente.

| Resultado | Línea | Casilla | Notas |
|-----------|-------|---------|-------|
| **Ganancia** (mayor valor) | 10 | **1032** | Monto total de ganancias reajustadas |
| **Pérdida** | 17 | **169** | Solo hasta el monto declarado en casillas 105, 155, 152, 1032, 1891, 1104 |
| Minería — ingresos (IDPC) | 5 | **955** | Valorización al recibir |
| Minería — costos (IDPC) | 5 | **954** | Gastos deducibles (electricidad, equipos, etc.) |

**Exención:** No se paga impuesto si la renta neta global anual no supera
13,5 UTA (aprox. $10.400.000 CLP, varía cada año).

**Período:** Año tributario base = 1 de enero al 31 de diciembre. Se declara en abril
del año siguiente (Operación Renta).

### USA (IRS)

- Crypto se clasifica como **propiedad**; las enajenaciones generan ganancias o pérdidas de capital.
- Corto plazo: tenencia menor a 365 días. Largo plazo: 365 días o más.
- Las comisiones se suman al costo base (en compra) o reducen el monto recibido (en venta).
- Airdrops/Staking se reconocen como **ingreso** al valor justo de mercado (FMV) al recibirlos;
  ese FMV pasa a ser el costo base.
- No hay ajuste por inflación.
- Reporte: Form 8949 + Schedule D.

### Other (Genérico / Internacional)

Jurisdicción genérica para usuarios fuera de Chile y USA. Aplica reglas
estándar internacionales sin ajustes específicos de ningún país. **No
reemplaza la asesoría de un profesional tributario local.**

- Las comisiones se suman al costo base (en compra) o reducen el monto
  recibido (en venta) — mismo comportamiento que USA.
- Airdrops/Staking se reconocen como **ingreso** al FMV; ese FMV pasa a ser
  el costo base.
- Corto plazo: tenencia menor a 365 días. Largo plazo: 365 días o más.
- No hay ajuste por inflación (sin IPC).
- Todos los métodos de costo base disponibles (FIFO, LIFO, HIFO, CPP).
- Sin guía de declaración específica — el reporte incluye un aviso para
  consultar al asesor tributario local.

> **Warning:** Cada país tiene reglas propias que pueden diferir
> significativamente de estas reglas genéricas. Ejemplos: Alemania exime
> ganancias si la tenencia supera 1 año; Francia no grava swaps
> crypto-to-crypto; Países Bajos usa impuesto al patrimonio en vez de
> ganancias de capital. Usa esta opción solo como punto de partida y valida
> con un profesional.

## Métodos de Costo Base

| Método | Descripción |
|--------|-------------|
| **FIFO** | Primero en entrar, primero en salir — lotes más antiguos primero |
| **LIFO** | Último en entrar, primero en salir — lotes más recientes primero |
| **HIFO** | Mayor costo primero — lotes de mayor costo primero. En Chile, este cálculo es **IPC-aware** (compara costos reajustados para la selección). |
| **CPP** | Costo Promedio Ponderado — promedio móvil del costo |

## IPC (Chile)

Datos del Índice de Precios al Consumidor. Se importan manualmente como CSV
(offline-first, sin descargas automáticas).

Fuente recomendada: INE — Serie histórica empalmada IPC.

El parser (`ipc.rs`) soporta:
- Formatos: `YYYY-MM`, `YYYY/MM`, `MM/YYYY`, nombres de mes en español.
- Headers: `Periodo`, `Año`, `Mes`, `Índice`, `IPC`, `Valor`, etc.
- Separadores decimales: punto y coma (ej. `1.234,50`).
- CSVs sin header (auto-detección).

## Tipos de Transacción (Clasificación Tributaria)

| TaxTxType | Subtipos | Ejemplo |
|-----------|----------|---------|
| `trade` | buy, sell, swap, other | Compra/venta/permuta |
| `income` | interest, reward, airdrop, gift, staking, mining, fork, payment, rebate, other | Ingresos |
| `expense` | payment, gift, fee, lost, stolen, donation, sell, other | Gastos/pérdidas |
| `transfer` | deposit, withdrawal | Movimiento entre wallets |

## Configuración

Almacenados en `TaxPeriodSettings` (por año):
- `jurisdiction`: Chile / USA / Other
- `method`: FIFO / LIFO / HIFO / CPP
- `include_swaps`: si los swaps generan enajenaciones
- `include_fee_crypto`: si los fees en crypto generan enajenación
- `excluded_wallet_ids`: lista de IDs de billeteras a omitir del reporte (ej. cuentas de test o donaciones).

Por defecto: Chile, HIFO, include_swaps=true, include_fee_crypto=true.

## Ajustes Manuales (Overrides)

El motor respeta campos de anulación manual en las transacciones para casos complejos:
- `override_proceeds`: ignora el cálculo automático de precio de venta y usa este valor.
- `override_cost_basis`: ignora el costo de adquisición original y usa este valor (útil para herencias o errores de historial).

## Resolutor de Swaps

Para pares de swap, el motor usa un sistema de puntaje para identificar la "pata de salida" (enajenación):
1. Prioriza transacciones con `override_proceeds`.
2. Prioriza transacciones con comisiones (`fees`) pagadas.
3. Prioriza transacciones con precio de mercado explícito.
4. Si hay empate, usa el orden alfabético de los IDs.

## Exportaciones

- **CSV de Reporte Tributario:** Resumen + enajenaciones + desglose de lotes + advertencias.
- **CSV de Historial de Transacciones:** Todas las transacciones del período con
  clasificación type/subtype.

## Advertencias del Motor

| Código | Significado |
|--------|-------------|
| `ipc_missing` | Datos IPC no cargados o incompletos para el período |
| `missing_price` | Transacción sin precio; excluida del reporte |
| `fee_missing_price` | Enajenación de fee sin precio para valorizar |
| `swap_missing_price` | Swap sin precio en ninguna pata |
| `swap_inferred` | Dirección del swap inferida (sin fee explícito) |
| `swap_unpaired` | Swap sin transacción contraparte |
| `insufficient_lots` | Lotes insuficientes para cubrir la venta |
| `no_lots` | Sin lotes disponibles para el activo |
| `invalid_date` | Fecha inválida en transacción |
| `invalid_type` | Tipo de transacción no reconocido |
| `income_missing_price` | Ingreso sin precio para calcular valor |
| `sii_casilla_may_change` | Recordatorio: las casillas del F22 pueden variar por año |
