# Cómo Sanctum calcula tus impuestos de criptomonedas

Esta guía explica **la lógica y los fundamentos** del módulo tributario de Sanctum:
en qué se basa, qué decisiones toma y por qué. Está pensada para que cualquier
persona pueda entender el razonamiento **sin necesidad de leer el código**.

> [!CAUTION]
>  **Esto es una estimación, no asesoría tributaria.** El módulo está en estado
> **beta/experimental**. Te da un punto de partida bien fundamentado, pero la
> responsabilidad final de tu declaración es tuya y de tu contador. Las leyes
> cambian y tienen matices que ningún software reemplaza.

> [!IMPORTANT]
> **Todo ocurre en tu computador.** Sanctum no envía tus datos a ningún
> servidor. El cálculo es 100% local y offline.

---

## La idea central: lotes, enajenaciones y ganancia

Casi toda la tributación de criptomonedas en el mundo se basa en la misma idea
sencilla, aunque cada país le cambie los detalles:

1. **Cada vez que adquieres** cripto (la compras, la recibes, te la pagan) se
   crea un **lote**: un paquete con *cuánto* recibiste, *cuándo*, y *cuánto te
   costó* (su "costo base").
2. **Cuando dispones** de esa cripto (la vendes, la cambias por otra, pagas con
   ella) ocurre una **enajenación**: un evento que puede generar impuesto.
3. La **ganancia** de esa enajenación es, en esencia:

   ```
   ganancia = lo que recibiste  −  lo que te había costado
   ```

   Si el resultado es positivo, hay ganancia (mayor valor); si es negativo, hay
   pérdida.

Toda la complejidad real está en responder dos preguntas:

- **¿Cuánto te costó exactamente?** → de eso se encargan los *métodos de costo*
  y los *ajustes* (como la inflación en Chile).
- **¿Qué eventos cuentan como enajenación?** → de eso se encarga la
  clasificación de transacciones.

---

## ¿Qué eventos tributan y cuáles no?

| Evento | ¿Tributa? |
|---|---|
| Comprar cripto con dinero (fiat) | No (solo creas un lote) |
| Mantener (HODL) | No |
| Transferir entre tus propias billeteras | No (salvo la comisión de red) |
| **Vender** cripto por dinero | **Sí** |
| **Cambiar** una cripto por otra (swap) | **Sí**, en la mayoría de jurisdicciones |
| **Pagar** con cripto un bien o servicio | **Sí** (es una disposición) |
| Recibir un airdrop / recompensa / staking | Depende del país (ver abajo) |

Un **swap** (ej. cambiar BTC por ETH) se trata como **dos cosas a la vez**:
vendes el BTC (enajenación) y compras el ETH (nuevo lote). Por eso un swap puede
generar impuesto aunque nunca hayas tocado dinero "real".

---

## Los métodos de costo: ¿cuál lote vendiste?

Si compraste BTC tres veces a precios distintos y luego vendes una parte,
¿de cuál de esas compras salió lo que vendiste? La respuesta cambia el impuesto.
Existen cuatro métodos:

| Método | Qué asume | Efecto típico |
|---|---|---|
| **FIFO** | Vendes primero lo más **antiguo** | El más aceptado por las autoridades |
| **LIFO** | Vendes primero lo más **reciente** | — |
| **HIFO** | Vendes primero lo de **mayor costo** | Tiende a minimizar la ganancia |
| **CPP** | Usas el **costo promedio** de todo lo que tienes | Suaviza los resultados |

**Importante:** no todos los países aceptan todos los métodos. Sanctum
**solo te ofrece los métodos válidos** para la jurisdicción que elijas (ver
cada país más abajo). El método por defecto es **FIFO**, porque es el aceptado
de forma prácticamente universal.

---

## Chile (SII)

En Chile, para una **persona natural**, el mayor valor por vender criptomonedas
tributa con el **Impuesto Global Complementario (IGC)**. Sanctum implementa las
reglas que el Servicio de Impuestos Internos ha fijado en sus oficios y guías.

### El doble ajuste por inflación (IPC) — lo más particular de Chile

Chile corrige los montos por inflación. Esto se hace en **dos pasos**, y es la
parte donde más se equivocan otras herramientas:

1. **Se reajusta el costo de compra.** Lo que pagaste se "trae a valor de hoy"
   según la variación del IPC entre el **mes anterior a la compra** y el **mes
   anterior a la venta**.
   *(Fundamento: Art. 17 N°8 letra m) de la Ley de la Renta.)*

2. **Se reajusta la ganancia ya calculada.** Una vez obtenida la ganancia, esta
   se vuelve a reajustar por IPC desde el **mes anterior a la venta** hasta
   **noviembre** (el último mes con IPC conocido al cierre del año tributario, el
   31 de diciembre).
   *(Fundamento: Art. 54 N°3, inciso penúltimo, de la Ley de la Renta, que rige
   el reajuste de las rentas para incluirlas en la Renta Bruta Global del IGC.
   El propio SII lo aplica en su guía "Economía Digital – Renta".)*

**Ejemplo ilustrativo** (los porcentajes son inventados, solo para mostrar la
mecánica):

```
Compras 1 unidad en enero por           $10.000.000
Reajuste del costo por IPC (+5%)    →    $10.500.000   (costo reajustado)
Vendes en noviembre por                  $15.000.000

Ganancia = 15.000.000 − 10.500.000  =     $4.500.000
Segundo reajuste de la ganancia (+0,7%) = $4.531.500   ← este monto va al F22
```
> [!WARNING]
> **Cuidado con el doble conteo al declarar.** El monto que Sanctum entrega
> ya viene reajustado. Al traspasarlo al Formulario 22, no lo vuelvas a
> reajustar a mano: el formulario, para este código, **no** lo reajusta
> automáticamente, así que se declara el valor ya ajustado tal cual.

### Comisiones (fees)

Para una persona natural sin contabilidad completa, las comisiones que cobra el
exchange **no se pueden descontar**: ni se suman al costo de compra ni se restan
del precio de venta.
*(Fundamento: Oficio SII N°1474/2020 y FAQ SII 001.250.7830, vigente a oct-2025.)*

Algunas guías comerciales dicen lo contrario; según el SII, para persona natural,
están equivocadas.

### Cripto recibida sin pagar

| Cómo la recibiste | Costo que se le asigna |
|---|---|
| **Airdrop / Staking / Fork** | **$0** (toda la venta futura es ganancia) |
| **Minería** | **Valor de mercado al momento de extraerla** |

*(Fundamentos: Oficio SII N°979/2022 para airdrop/staking; criterio análogo para
fork al no existir oficio específico; Oficio SII N°1803/2022 para minería.)*

La lógica: si no pagaste nada por ella, su costo es cero. La minería es la
excepción, porque se considera el valor de mercado del día en que la obtienes.

### Métodos aceptados en Chile

**Solo FIFO y CPP.** LIFO y HIFO **no son aceptados** por el SII. Sanctum, cuando
eliges Chile, **oculta** LIFO y HIFO para que no puedas elegir un método inválido
por accidente. El CPP solo es admisible cuando no es posible acreditar los costos
más antiguos. Una vez elegido, el método debería mantenerse por al menos 5 años.
*(Fundamento: Oficio SII N°1474/2020, aplicando el Art. 30 de la Ley de la Renta.)*

### Pérdidas

Las pérdidas se pueden compensar **dentro del mismo año** contra ganancias del
mismo tipo, pero **no se arrastran** a años siguientes (para persona natural sin
contabilidad completa).
*(Fundamento: Oficio SII N°979/2022 y Circular SII N°43/2021.)*

### Otros puntos

- **No hay IVA** en la compra/venta de criptomonedas (son bienes incorporales).
- **No hay distinción** entre corto y largo plazo: toda ganancia tributa igual.
- **No existe una exención específica para cripto.** El "tramo exento" que mucha
  gente menciona es simplemente el **primer tramo del IGC (0%)**, que beneficia a
  todas las rentas por igual (alrededor de 13,5 UTA al cierre del año). No es un
  beneficio propio de las criptomonedas.

### Cómo se declara (Formulario 22)

> [!IMPORTANT]
> Los códigos del formulario pueden cambiar cada año. Verifica siempre el
> suplemento tributario del SII del Año Tributario vigente.

| Concepto | Línea | Código |
|---|---|---|
| Ganancia de **fuente chilena** | 10 | **1032** |
| Ganancia de **fuente extranjera** (cripto en exchanges del exterior) | 11 | **1104** |
| Pérdida | 17 | 169 |
| Minería — ingresos / costos (1ª Categoría) | 5 | 955 / 954 |

**Nuevas Declaraciones Juradas (desde 2026):** los exchanges con domicilio en
Chile deben informar al SII las operaciones de sus usuarios mediante las
**DJ 1963 y 1964** (creadas por las Resoluciones Exentas SII N°113 y N°114 de
agosto de 2025). Las presenta el exchange, no tú — pero el SII las cruza con tu
declaración. Es decir: el SII ya sabe; conviene declarar bien.

---

## Estados Unidos (IRS)

En EE.UU. la cripto se trata como **propiedad**, y las enajenaciones generan
ganancias o pérdidas de capital.

- **Comisiones:** sí se consideran — se suman al costo al comprar y reducen lo
  recibido al vender.
- **Cripto recibida (airdrop, staking, minería, fork):** se reconoce como
  **ingreso** a su valor de mercado al recibirla; ese valor pasa a ser su costo
  base. *(A diferencia de Chile, donde el fork va a $0.)*
- **Plazo de tenencia:** importa. Si la tuviste **más de un año** (por fecha de
  aniversario, no por "365 días" exactos), la ganancia es de **largo plazo** y
  suele pagar menos.
- **Métodos:** FIFO e Identificación Específica (que habilita LIFO y HIFO). El
  **costo promedio (CPP) no es válido** para cripto en EE.UU., por lo que Sanctum
  lo oculta cuando eliges esta jurisdicción.
- **No hay ajuste por inflación.**
- **Swaps** sí tributan (no hay intercambio "libre de impuesto" tipo §1031).
- **Pérdidas:** se pueden usar contra ganancias y hasta US$3.000 contra otros
  ingresos, con arrastre indefinido a años futuros.
- **Reporte:** Form 8949 + Schedule D.

> [!IMPORTANT]
> **Limitación conocida (regla 2025):** desde 2025 el IRS exige rastrear el costo
> **por billetera/cuenta** y no de forma global (Rev. Proc. 2024-28). El motor
> actual agrupa los lotes de forma global por activo. Mientras esto no se ajuste,
> el resultado para EE.UU. puede no calzar exactamente con la nueva regla por
> billetera. Está en la hoja de ruta posterior al alpha.

---

## Internacional (genérico)

Para usuarios fuera de Chile y EE.UU., Sanctum ofrece un modo **internacional**
con reglas estándar de "ganancia de capital realizada por lotes":

- Comisiones al costo (como EE.UU.).
- Cripto recibida = ingreso a valor de mercado.
- Distinción corto/largo plazo en torno a un año.
- Todos los métodos disponibles (FIFO/LIFO/HIFO/CPP).
- Sin ajuste por inflación.

> [!WARNING]
>  **Esto es un punto de partida, no la ley de tu país.** La tributación de
> cripto **no está armonizada** entre países y muchos se salen de este modelo.
> Ejemplos reales: Alemania exime la ganancia si mantienes más de un año; Francia
> no grava los cambios cripto-a-cripto, solo cuando pasas a euros; Países Bajos
> usa un impuesto al patrimonio en vez de a la ganancia; Reino Unido usa un
> sistema de "pooling" distinto a FIFO/HIFO. Si vives en uno de esos países, este
> modo dará números equivocados. Úsalo solo como base y valida con un profesional
> local.

La estrategia de Sanctum para la cobertura global no es programar las 200 leyes
del mundo, sino: (1) un modo internacional honesto y bien etiquetado, (2) una
**exportación impecable de enajenaciones** que cualquier contador pueda adaptar a
su formulario local, y (3) **módulos por país aportados por la comunidad**.

---

## El IPC (solo Chile)

El cálculo chileno necesita los datos del **Índice de Precios al Consumidor**.
Como Sanctum es offline-first, **no los descarga solo**: tú los importas una vez
como un archivo CSV (por ejemplo, la serie histórica del INE). El sistema es
flexible con el formato (acepta meses en español, distintos separadores y
encabezados). Si faltan datos de IPC para el período, el reporte te avisa en vez
de calcular mal en silencio.

---

## Honestidad del módulo: qué te avisa

El motor no calcula a ciegas: cuando algo no está claro, **lo marca** en vez de
inventar un número. Por ejemplo, te avisa cuando una transacción no tiene precio,
cuando faltan datos de IPC, cuando no hay suficientes lotes para cubrir una venta,
cuando un swap quedó sin su contraparte, o cuando un código del Formulario 22
podría haber cambiado de un año a otro. Esas advertencias aparecen junto al
reporte para que sepas exactamente dónde mirar.

---

## En qué se basa todo esto (fuentes)

**Chile — SII:** Ley sobre Impuesto a la Renta (Art. 17 N°8 letra m), Art. 30,
Art. 52, Art. 54); Oficios N°963/2018 (no IVA), N°1474/2020 (costo, comisiones,
métodos), N°979/2022 (airdrop/staking $0), N°1803/2022 (minería), N°2208/2022;
Circular N°43/2021 (pérdidas); guía SII "Economía Digital – Renta"; FAQ de
criptomonedas del SII (actualizadas en octubre de 2025); Resoluciones Exentas
N°113 y N°114 de 2025 (DJ 1963/1964).

**EE.UU. — IRS:** tratamiento como propiedad; Rev. Rul. 2019-24 y 2023-14
(ingreso por airdrop/staking); Rev. Proc. 2024-28 (costo por billetera);
Forms 8949 / Schedule D.

> [!CAUTION]
> Estas fuentes fueron contrastadas entre sí y contra el texto primario. Aun así,
> **la palabra final la tiene un profesional tributario de tu país.** Sanctum te
> ahorra el trabajo pesado de reconstruir el historial y aplicar las reglas, pero
> no firma tu declaración por ti.

---

## Para desarrolladores

La referencia técnica del motor (archivos, estructuras, capa de servicio y
callbacks de UI) vive junto al código en `src/features/crypto/tax/`. Este
documento describe la lógica; el código describe la implementación.
