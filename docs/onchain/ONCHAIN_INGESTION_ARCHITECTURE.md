# Arquitectura de Ingesta On-Chain

## Principios

1. Reusar el pipeline existente de ingestión crypto.
2. Mantener separación de capas:
   - `UI callbacks -> Controller -> Features -> DB`.
3. No acoplar parser/normalización a una API concreta.
4. Seguridad y privacidad por defecto (mínima exposición de datos).

## Alcance Inicial

- Redes objetivo de fase 1/2:
  - Bitcoin
  - Litecoin
  - Polygon
- Modalidad:
  - Read-only (sin trading, sin retiros, sin firma).
  - Import de historial y sincronización manual bajo demanda.

## Estructura Recomendada

Rutas sugeridas para mantener consistencia con el proyecto:

```text
src/
├── features/
│   └── ingestion/
│       ├── onchain/
│       │   ├── mod.rs
│       │   ├── service.rs              # orquestación on-chain
│       │   ├── types.rs                # tipos normalizados
│       │   ├── provider.rs             # traits / puertos
│       │   ├── btc_ltc.rs              # adaptador UTXO
│       │   └── polygon.rs              # adaptador EVM
│       └── service.rs                  # integración final con import summary
├── controller/
│   └── ingestion.rs                    # casos de uso UI -> feature
└── ui/
    └── callbacks/
        └── ingestion.rs                # wiring de UI para iniciar sync on-chain
```

## Contrato Normalizado Interno

Antes de escribir a `ImportCryptoTransaction`, cada adaptador debe producir un
formato neutral.

Campos mínimos sugeridos:

- `chain`: `bitcoin | litecoin | polygon`
- `wallet_ref`: address/xpub/account tag
- `tx_hash`
- `timestamp_utc`
- `asset_symbol`
- `direction`: `in | out`
- `amount_asset`
- `fee_symbol` (opcional)
- `fee_amount` (opcional)
- `counterparty` (opcional)
- `raw_kind`: etiqueta del proveedor (opcional)

Regla de salida:

- Convertir este formato a `ImportCryptoTransaction`.
- Mantener `fee_coin_symbol` y `fee_amount` siempre coherentes (ambos o ninguno).

## Mapeo a Tipos Sanctum

Mapeo inicial conservador:

- Entrada de fondos -> `transaction_type=transfer`, `subtype=deposit`
- Salida de fondos -> `transaction_type=transfer`, `subtype=withdrawal`

Notas:

- Swaps y operaciones DeFi complejas deben marcarse para revisión en fases
  posteriores si no hay certeza suficiente.
- Si falta precio histórico, registrar warning de tax readiness, no inventar
  valorización.

## Deduplicación

Usar clave determinística por origen on-chain para prevenir duplicados en
resync:

- `chain + wallet_ref + tx_hash + asset_symbol + direction + amount`

Reglas:

- Re-sync debe ser idempotente.
- Cualquier evento ambiguo debe preferir `skip + warning` antes que
  insertar datos potencialmente incorrectos.

## Privacidad

- No loggear direcciones completas ni identificadores sensibles en texto plano.
- Redactar hashes/direcciones en logs UI (ej. prefijo/sufijo).
- Permitir priorizar endpoints de usuario (nodo propio/proxy) cuando exista.

## Errores y Readiness

Categorías recomendadas:

- `source_unreachable`
- `rate_limited`
- `invalid_wallet_ref`
- `unsupported_chain_feature`
- `missing_price`
- `ambiguous_classification`

Cada warning debe indicar:

- Qué pasó.
- Cuántas transacciones impacta.
- Qué acción puede tomar el usuario.

## No Objetivos (fase inicial)

- Trading por API.
- Firma/transmisión de transacciones.
- Decodificación total de protocolos DeFi complejos.

