<div align="center">

<img src="./assets/sanct-app.png" alt="Sanctum" width="120" height="120" />

<h1>SANCTUM</h1>

<p align="center">
  <img src="https://readme-typing-svg.demolab.com?font=Fira+Code&weight=600&size=25&pause=1000&color=8B5CF6&center=true&vCenter=true&width=435&lines=Tu+fortaleza+personal.;Privado.;Encriptado.;Local." alt="Typing SVG">
</p>

<div align="center">

[![English](https://img.shields.io/badge/In-English-8b5cf6?style=for-the-badge)](README.md) [![Website](https://img.shields.io/badge/🌐_Website-Sanctum-blueviolet?style=for-the-badge)](https://kyronix.codeberg.page/Sanctum/)

</div>

<div align="center">
    <a href="">
      <img src="https://img.shields.io/badge/Core-Rust-orange?style=for-the-badge&logo=rust" alt="Rust" />
    </a>
    <a href="">
      <img src="https://img.shields.io/badge/Shell-Tauri%202-blue?style=for-the-badge" alt="Tauri" />
    </a>
    <a href="">
      <img src="https://img.shields.io/badge/UI-Svelte%205-orange?style=for-the-badge" alt="Svelte" />
    </a>
    <a href="">
      <img src="https://img.shields.io/badge/Security-SQLCipher-green?style=for-the-badge&logo=sqlite" alt="SQLCipher" />
    </a>
    <a href="">
      <img src="https://img.shields.io/badge/Env-Nix-blueviolet?style=for-the-badge&logo=nixos" alt="Nix" />
    </a>
  </div>

<br />
</div>

---

![Sanctum](assets/sanctum-img.png)

<div align="center">

[![Licencia](https://img.shields.io/badge/Licencia-GPLv3-8b5cf6?style=flat-square)](LICENSE) &nbsp;
![Plataforma](https://img.shields.io/badge/Plataforma-Linux%20%7C%20macOS%20%7C%20Windows%20%7C%20Android-informational?style=flat-square) &nbsp;
![Estado](https://img.shields.io/badge/Estado-Alpha-orange?style=flat-square) &nbsp;
![PRs](https://img.shields.io/badge/PRs-bienvenidos-brightgreen?style=flat-square)

**[Acerca de](#acerca-de)** · **[Características](#características)** · **[Importación](#importación-e-integraciones)** · **[Seguridad](#seguridad-y-privacidad)** · **[Plataformas](#plataformas-y-autohospedaje)** · **[Instalación](#instalación)** · **[Docs](docs/INSTALL_ES.md)**

</div>

## Acerca de

> [!CAUTION]
> **NO ESTÁ LISTO PARA USAR — EN DESARROLLO ACTIVO.**

**Sanctum** es una bóveda orientada a la privacidad para tus finanzas y crypto
— en **escritorio y Android**. Todo funciona sobre hardware que controlas
tú: almacenamiento cifrado, cero telemetría, sin cuentas, sin nube corporativa. Tú
controlas las llaves, la base de datos y los respaldos — nadie más.

Úsalo totalmente offline con una base de datos local cifrada, o levanta **tu propio
servidor Sanctum** y comparte una sola bóveda privada entre todos tus dispositivos
— escritorio, Android y cualquier navegador (incluido iOS) — por tu propia red.

Está pensado para quienes quieren un único lugar auditable donde llevar sus
finanzas sin entregar su vida financiera a un tercero.

## Características

### Dashboard Unificado
Patrimonio neto y tendencias combinando tus finanzas **y** crypto en una sola vista.

### Finanzas
Cuentas, categorías, transferencias y un ledger completo de transacciones, con
soporte multi-divisa (USD, CLP, EUR y más).

### Crypto
- Billeteras, trades y swaps con balance dinámico de portafolio.
- Sincronización privada de precios vía CoinGecko, con soporte de proxy / Tor.
- **Motor de impuestos offline** para Chile (SII), USA (IRS) y jurisdicciones
  internacionales, aplicando el método de costo correcto según las reglas locales
  (FIFO, CPP y más). Ver **[CRYPTO_TAX.md](docs/CRYPTO_TAX.md)** para entender la
  lógica y los fundamentos legales detrás.

### Confiabilidad
- Respaldos cifrados (SQLCipher) con seguridad de restauración y rollback.
- Ingesta JSON/CSV/TXT con validación por fila y detección de duplicados.
- Interfaz multi-idioma (EN/ES).

## Importación e Integraciones

Sanctum es **offline- y CSV-first** — diseñado para viajes y baja conectividad.
Todos los imports son best-effort, se validan por fila, hacen deduplicación y **no
realizan llamadas de red** durante la ingesta.

> [!IMPORTANT]
> Cada archivo de exchange/wallet se procesa **localmente en tu dispositivo**.
> Nada se sube a un tercero — el único servidor que existe es uno que hospedas tú.

**Formatos soportados:**

- **JSON** *(recomendado)* — formato completo usado por el Sanctum Generator.
- **CSV** — exportes de hojas de cálculo (archivos separados por tipo).
- **TXT** — notas por línea con prefijos para captura rápida.

**Integraciones de exchange y wallet:**

| Integración | Estado | Entrada | Notas |
| :-- | :-- | :-- | :-- |
| Kraken | Disponible | CSV (`Ledgers`, `Trades`) | Sube uno o ambos archivos para cobertura spot completa. |
| Binance | Disponible | CSV (`All Statements`, `Spot Trade History`) | Balances, actividad spot y movimientos de ledger asociados. |
| MEXC | Disponible | CSV (17 tipos de reporte) | Spot, Statement, Funding, Fiat, Futures y exportes relacionados. |
| NotBank (ex-CryptoMarket) | Disponible | CSV (`Transaction`, `Trade Activity`) | Movimientos de cuenta y trading desde reportes Exchange Pro. |
| Feather Wallet | Disponible | CSV (export de historial) | Historial de wallet Monero en formato Feather. |
| Monero GUI Wallet | Disponible | CSV (export de historial) | Historial de wallet Monero en formato Monero GUI. |

<details>
<summary><b>Integraciones planificadas</b></summary>

| Integración | Estado | Entrada | Notas |
| :-- | :-- | :-- | :-- |
| Coinbase | Planificado | CSV | Flujos de estado de cuenta e historial de trades. |
| Bybit | Planificado | CSV | Exportes de historial spot/funding. |
| OKX | Planificado | CSV | Formatos de exportación de cuenta y trades. |
| KuCoin | Planificado | CSV | Imports CSV de estado/trades. |
| Bitget | Planificado | CSV | Reportes de wallet y spot. |
| Buda | Planificado | CSV | Exportes de transacciones/trades. |
| Orionx | Planificado | CSV | Exportes de transacciones/trades. |
| APIs de Exchange (solo lectura) | Planificado | API | Sincronización directa futura — solo lectura, sin trading/retiros. |

</details>

## Seguridad y Privacidad

Sanctum se apoya en tres pilares:

1. **Sin nube corporativa.** Sin telemetría, sin cuentas, sin servidores de terceros. Tus datos solo viven en hardware que controlas tú — tu dispositivo, o un servidor que hospedas tú.
2. **Almacenamiento blindado.** SQLCipher (AES-256) cifra toda la base de datos con una contraseña maestra que controlas tú.
3. **Conexiones externas mitigadas.** La sincronización de precios usa relleno de tráfico para ocultar tu portafolio y soporta proxies configurados por el usuario (SOCKS5/Tor, HTTP).

> [!NOTE]
> Minimizar los metadatos no es lo mismo que eliminarlos: conectarse a cualquier
> API externa revela inherentemente tu IP a ese proveedor a menos que enrutes el
> tráfico por un proxy.

Los respaldos están cifrados en reposo e incluyen seguridad de restauración y rollback.

## Plataformas y Autohospedaje

Sanctum funciona como app nativa en **escritorio (Linux, macOS, Windows)** y
**Android**, y como **app web** para cualquier otro dispositivo — incluido
**iOS** — servida desde un servidor que hospedas tú mismo.

Dos formas de usarlo, elegibles por dispositivo:

- **Local** *(por defecto)* — una base de datos cifrada totalmente offline que
  vive solo en ese dispositivo. Sin servidor, sin red.
- **Autohospedado** — levanta tu propio **servidor Sanctum** como única fuente de
  verdad y comparte una sola bóveda entre todos tus dispositivos. Accedé a él de
  forma privada por tu LAN o una VPN mesh como **Tailscale** — nunca tiene que
  estar expuesto a internet.

> [!IMPORTANT]
> Sigue sin existir una nube de Sanctum. El único servidor que existe es uno que
> **tú** levantas, sobre hardware que **tú** controlas.

## Tecnologías

Sanctum prioriza el rendimiento, la seguridad de tipos y la auditabilidad.

| Componente      | Tecnología           | Rol                                           |
| :-------------- | :------------------- | :-------------------------------------------- |
| **Núcleo**      | **Rust**             | Lógica de negocio, validación y cálculos.     |
| **Shell**       | **Tauri 2**          | Shell nativo ligero con WebView.              |
| **Frontend**    | **Svelte 5 + TS**    | UI reactiva con TypeScript y Vite.            |
| **Base de Datos** | **SQLite + SQLCipher** | Almacenamiento relacional cifrado localmente. |
| **Entorno**     | **Nix + Direnv**     | Entorno de desarrollo reproducible y hermético. |

El mismo núcleo Rust impulsa todos los destinos — escritorio, Android y el
servidor autohospedado opcional — así que la lógica de negocio vive en un solo lugar.

## Instalación

> [!NOTE]
> La configuración completa para **Linux, macOS y Windows** — incluyendo el
> toolchain de requisitos — está en la **[Guía de Instalación](docs/INSTALL_ES.md)**.

El repositorio incluye un [`install.sh`](install.sh) que compila desde el código
fuente e instala Sanctum. En Linux instala el binario, la entrada de escritorio y
el icono; en macOS instala solo el binario.

```bash
git clone https://codeberg.org/Kyronix/Sanctum.git
cd Sanctum
./install.sh --user      # compila + instala en ~/.local, sin sudo
```

### Inicio Rápido (Nix, para desarrollo)

Este proyecto usa **Nix Flakes** para un entorno reproducible, además de
**Node.js** y **pnpm** para el frontend Svelte. Todos los comandos se ejecutan
desde la **raíz del repositorio**:

```bash
direnv allow                          # o: nix develop
cd ui-svelte && pnpm install && cd ..  # solo la primera vez
cargo tauri dev                       # ejecutar en modo desarrollo
cargo tauri build                     # compilar binario de producción
```

### Android

Con el toolchain móvil de Tauri configurado, compila y ejecuta en un dispositivo conectado:

```bash
cargo tauri android dev     # ejecutar en un dispositivo conectado (USB o ADB inalámbrico)
cargo tauri android build   # compilar un APK / AAB de release
```

Consulta **[docs/INSTALL_ES.md](docs/INSTALL_ES.md)** para el toolchain manual y
notas específicas por plataforma.

## Transparencia en el Desarrollo

Este proyecto abraza la colaboración abierta sin comprometer la auditabilidad.

- **Arquitectura dirigida por humanos.** La privacidad y la integridad de los datos son la prioridad, diseñadas y dirigidas por humanos.
- **Desarrollo asistido por IA.** La mayor parte del código se genera o refactoriza con LLMs de frontera bajo estricta auditoría humana. Los modelos principales, en orden, son **Claude Opus 4.7**, **Claude Sonnet 4.6** y **DeepSeek V4 Pro**, entre otros modelos de frontera.
- **Auditable por diseño.** El código completo está abierto para inspección — verifica tú mismo que no hay telemetría oculta.

## Comunidad y Contribución

¿Encontraste un bug o tienes una idea? Usa el
[Issue Tracker](https://codeberg.org/Kyronix/Sanctum/issues) para errores y
sugerencias.

## Aviso Legal

**Sanctum está actualmente en ALPHA.** La encriptación es estándar de la industria,
pero el software está en desarrollo activo y puede cambiar sin previo aviso.
**Mantén siempre copias de seguridad de tus claves de recuperación.**

## Licencia

Código abierto bajo la **Licencia Pública General GNU v3.0**. Consulta el archivo
[LICENSE](LICENSE) para más detalles.

-----

<div align="center">
<sub>Construido con ❤️, 🦀 Rust y ❄️ Nix</sub>
</div>
