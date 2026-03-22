# Diseño On-Chain: Bitcoin y Litecoin

## Objetivo

Implementar lectura confiable de historial para chains UTXO (BTC/LTC) con
importación coherente para balances e impuestos.

## Entrada Esperada

Tipos de referencia de wallet (fase incremental):

1. Direcciones individuales.
2. Lista de direcciones.
3. XPUB/YPUB/ZPUB (cuando el adaptador esté listo).

Parámetros mínimos:

- `wallet_name` (nombre interno Sanctum)
- `network` (`bitcoin` o `litecoin`)
- `start_date` opcional (para reducir costo de sync)

## Cobertura Fase Inicial (MVP)

- Detección de entradas/salidas por transacción.
- Cálculo de monto neto por asset.
- Fee de red como `fee_coin_symbol` + `fee_amount` para egresos.
- Dedupe e idempotencia en re-sync.

Mapeo inicial:

- Inflow neto -> `transfer/deposit`
- Outflow neto -> `transfer/withdrawal`

## Reglas de Clasificación

Para cada tx de una wallet:

1. Calcular valor neto recibido/enviado.
2. Determinar dirección principal (`deposit` o `withdrawal`).
3. Registrar fee solo cuando exista certeza de gasto de red.
4. Adjuntar `tx_hash` en `notes` con formato acotado.

Importante:

- No inferir compra/venta fiat desde la cadena UTXO sin evidencia adicional.
- Mantener clasificación conservadora como transferencia en fase inicial.

## Litecoin MWEB (MimbleWimble)

Limitación técnica esperada:

- MWEB reduce visibilidad pública detallada de montos/enlaces internos.
- Se puede observar parte del flujo (peg-in/peg-out), pero no siempre
  reconstruir actividad completa solo con datos públicos.

Política recomendada:

- Soporte inicial explícito para UTXO transparente.
- MWEB marcado como `partial support` con warning descriptivo.
- Permitir complementar con import manual/CSV cuando falte trazabilidad.

## Riesgos y Mitigación

Riesgos:

- Cambio interno (change outputs) mal interpretado.
- CoinJoin/mezclas difíciles de clasificar.
- Diferencias entre indexadores.

Mitigaciones:

- Estrategia conservadora (skip + warning en ambigüedad).
- Fixtures de prueba reales anonimizadas.
- Comparación de balances finales por ventana temporal.

## Criterios de Aceptación

1. Re-sync idempotente (sin duplicados).
2. Balance BTC/LTC consistente con historial importado.
3. Fees reflejadas correctamente sin romper validación.
4. Warnings claros para casos MWEB/ambigüos.
5. Sin regresión en parsers CSV existentes.

