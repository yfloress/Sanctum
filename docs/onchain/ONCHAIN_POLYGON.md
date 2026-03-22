# Diseño On-Chain: Polygon

## Objetivo

Agregar lectura on-chain para wallets Polygon con foco en balances y trazabilidad
fiscal básica, sin introducir clasificaciones agresivas de baja confianza.

## Entrada Esperada

- Dirección EVM (`0x...`)
- `wallet_name` en Sanctum
- `start_date` opcional

## Cobertura Fase Inicial (MVP)

1. Transferencias nativas de la red (POL/MATIC según ticker en app).
2. Transferencias ERC-20 estándar (`Transfer` events).
3. Fee de red en token nativo para transacciones salientes.

Mapeo inicial:

- Inflow -> `transfer/deposit`
- Outflow -> `transfer/withdrawal`

## Cobertura Fase Posterior

- Detección de swaps DEX con mayor certeza.
- Heurísticas para bridge/contract interactions complejas.
- Mejoras de clasificación `trade/swap` cuando exista evidencia sólida.

## Reglas de Normalización

Por cada evento relevante:

1. Determinar símbolo de activo.
2. Convertir cantidad a decimal humano (según `decimals`).
3. Identificar dirección de flujo respecto a wallet observada.
4. Unificar timestamp UTC y hash.
5. Construir tx importable con fee coherente.

## Riesgos y Mitigación

Riesgos:

- Contratos complejos (routers, LP, vaults) difíciles de mapear a una acción fiscal.
- Tokens sin metadata consistente.
- Eventos parciales en proveedores de datos.

Mitigaciones:

- Fase 1 limitada a movimientos claros.
- Casos complejos marcados como `needs_review`.
- Reintento manual y fallback de precios igual que flujo tax actual.

## Criterios de Aceptación

1. Balance final de activos coincide con historial importado de eventos soportados.
2. Fees en POL/MATIC aparecen correctamente sin invalidar transacciones.
3. Sin falsos positivos de swaps en fase inicial.
4. Warnings accionables para eventos no clasificables.
5. Integración estable con readiness checks de impuestos.

