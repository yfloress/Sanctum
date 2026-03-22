# On-Chain Import (Plan Base)

Este directorio documenta el diseño para agregar lectura on-chain en Sanctum
sin romper el flujo actual de importación por CSV.

## Estado

- `In Progress`: base on-chain ya implementada en backend para **Bitcoin**,
  **Litecoin** y **Polygon**.
- Flujo actual: providers + controller + servicio de ingesta + endpoints
  configurables.
- Pendiente para habilitación pública: UX dedicada de import on-chain,
  endurecimiento adicional y validación extendida en datasets reales.

## Objetivo

Permitir importar historial de wallets desde datos on-chain (read-only), con:

- Mapeo consistente a `ImportCryptoTransaction`.
- Reutilización de deduplicación y validación existentes.
- Compatibilidad con motor de impuestos y readiness checks.
- Privacidad primero (sin exponer datos innecesarios).

## Documentos

- `docs/onchain/ONCHAIN_INGESTION_ARCHITECTURE.md`
  Arquitectura propuesta y contratos internos.
- `docs/onchain/ONCHAIN_BITCOIN_LITECOIN.md`
  Diseño para UTXO chains (BTC/LTC) con notas sobre MWEB.
- `docs/onchain/ONCHAIN_POLYGON.md`
  Diseño para cadena EVM (Polygon): nativo + ERC-20.
- `docs/onchain/ONCHAIN_ROADMAP.md`
  Fases, criterios de aceptación y estrategia de pruebas.
