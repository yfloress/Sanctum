# Roadmap de Implementación On-Chain

## Resumen

Prioridad funcional:

1. Bitcoin
2. Litecoin
3. Polygon

Solo cuando estas tres integraciones estén estables y validadas, se habilita
soporte público de forma progresiva.

## Estado Actual (2026-02-27)

- Fase 0: base técnica implementada.
- Fase 1: implementación MVP de Bitcoin/Litecoin activa (Esplora, dirección
  única, solo lectura).
- Fase 2: implementación MVP de Polygon activa (Blockscout v2, nativo + token
  transfers básicos).
- Pendiente: UX dedicada de import on-chain, endurecimiento y validación fiscal
  ampliada (Fase 3/4).

## Fase 0: Base Técnica

Entregables:

- Tipos internos on-chain y contrato normalizado.
- Trait de proveedor desacoplado de API específica.
- Wiring mínimo en controller/UI para flujo manual.

Criterio de salida:

- Compila sin warnings nuevos.
- Sin cambios de comportamiento en import CSV actual.

## Fase 1: Bitcoin + Litecoin (UTXO transparente)

Entregables:

- Lectura de historial y mapeo `transfer/deposit|withdrawal`.
- Fee de red correctamente registrada.
- Dedupe idempotente por clave determinística.

Criterio de salida:

- Balances coherentes en casos de prueba controlados.
- Warnings claros en casos ambiguos.
- Cobertura de tests unitarios + integración.

## Fase 2: Polygon (nativo + ERC-20)

Entregables:

- Lectura de transferencias nativas.
- Lectura de eventos ERC-20.
- Integración con pipeline de import crypto existente.

Criterio de salida:

- Balance final consistente en fixtures conocidas.
- Sin clasificación falsa de swaps complejos.

## Fase 3: Calidad Fiscal y UX

Entregables:

- Readiness warnings más descriptivos para on-chain.
- Mejoras de resolución de precios faltantes.
- Mejoras de mensajes de revisión manual.

Criterio de salida:

- Reporte fiscal coherente en dataset de validación.
- Warnings accionables y sin ruido excesivo.

## Fase 4: Endurecimiento

Entregables:

- Validaciones de rendimiento y robustez.
- Control de errores de proveedor/rate limits.
- Documentación de límites conocidos (ej. MWEB parcial).

Criterio de salida:

- Flujo estable en re-sync repetidos.
- Sin regresiones en exchanges/wallets ya soportados.

## Estrategia de Pruebas

- Unit tests para normalización y clasificación.
- Integration tests por cadena con fixtures anonimizadas.
- Regression tests para dedupe.
- Test de no regresión para parsers actuales (Kraken, Binance, MEXC, NotBank,
  Feather, Monero GUI).

## Reglas Operativas

- No introducir archivos de código excesivamente grandes.
- Mantener módulos por dominio (UTXO vs EVM).
- Evitar hardcodes frágiles de símbolos/fees.
- Cualquier caso no confiable: `skip + warning` antes de inferencia agresiva.
