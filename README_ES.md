<div align="center">

<img src="./ui/assets/logo/sanctum_logo.svg" alt="Sanctum Logo" width="120" height="120" />

<h1>SANCTUM</h1>

<p align="center">
  <img src="https://readme-typing-svg.demolab.com?font=Fira+Code&weight=600&size=25&pause=1000&color=8B5CF6&center=true&vCenter=true&width=435&lines=Tu+fortaleza+personal.;Privado.;Encriptado.;Local." alt="Typing SVG">
</p>

<div align="center">

[![English](https://img.shields.io/badge/Language-English-8b5cf6?style=for-the-badge)](README.md) [![Website](https://img.shields.io/badge/🌐_Website-Sanctum-blueviolet?style=for-the-badge)](https://kyronix.codeberg.page/Sanctum/)

</div>

<div align="center">
    <a href="">
      <img src="https://img.shields.io/badge/Core-Rust-orange?style=for-the-badge&logo=rust" alt="Rust" />
    </a>
    <a href="">
      <img src="https://img.shields.io/badge/GUI-Slint-blue?style=for-the-badge" alt="Slint" />
    </a>
    <a href="">
      <img src="https://img.shields.io/badge/Render-Skia%20%2B%20OpenGL-informational?style=for-the-badge" alt="Skia" />
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

![Sanctum](assets/sanctum.jpg)

## Acerca de

> **NO ESTA LISTO PARA USAR, EN DESARROLLO**

**Sanctum** es una bóveda de escritorio orientada a privacidad para finanzas, crypto y hábitos.
Funciona localmente en Linux, con almacenamiento cifrado, cero telemetría y sin dependencia
de la nube. Tú controlas las llaves, la base de datos y los respaldos.

## Capacidades Actuales

- **Dashboard:** Patrimonio neto y tendencias combinando finanzas + crypto.
- **Finanzas:** Cuentas, categorías, transferencias y ledger completo.
- **Crypto:** Billeteras, trades, swaps, balances, sincronización privada de precios vía CoinGecko y **motor de impuestos avanzado** (Chile/SII, USA/IRS, Internacional).
- **Impuestos Crypto:** Motor tributario offline con soporte para múltiples jurisdicciones, métodos de valorización (FIFO, LIFO, HIFO, CPP), ajustes por IPC (Chile) y generación de reportes tributarios.
- **Hábitos:** Registros diarios, heatmaps, rachas y recompensas/objetivos.
- **Importación y Respaldos:** Ingesta JSON/CSV/TXT con preview + dedup, **importación manual de IPC** para ajustes tributarios y respaldos cifrados.
- **Configuración:** Moneda USD/CLP, idioma EN/ES y proxy para APIs de crypto.

## Formatos de Importación

Sanctum acepta formatos offline pensados para viajes o baja conectividad:

- **JSON** (recomendado): Formato completo usado por el Sanctum Generator.
- **CSV**: Exportes desde hojas de cálculo (archivos separados por tipo).
- **TXT**: Notas por línea con prefijos para captura rápida.

Todos los imports son **best-effort**, se validan por fila y hacen deduplicación.
No se realizan llamadas de red durante la ingesta.

## Tecnologías

Sanctum está construido priorizando el rendimiento, la seguridad de tipos y la auditabilidad.

| Componente           | Tecnología             | Descripción                                                       |
| :------------------- | :--------------------- | :---------------------------------------------------------------- |
| **Núcleo** | **Rust** | Lógica de negocio, cálculos financieros y seguridad.              |
| **Framework GUI** | **Slint** | Kit de herramientas UI nativo en Rust. Ligero y tipado.           |
| **Renderizado** | **Skia / OpenGL** | Renderizado de gráficos 2D de alto rendimiento vía Winit.         |
| **Base de Datos** | **SQLite + SQLCipher** | Almacenamiento relacional encriptado localmente.                  |
| **Entorno** | **Nix + Direnv** | Entorno de desarrollo reproducible y hermético.                   |

## Instalación y Desarrollo

Este proyecto utiliza **Nix Flakes** para garantizar un entorno reproducible sin contaminar tu sistema global.

> **Nota:** Para instalación manual en Linux, macOS o Windows, consulta la [Guía de Compilación](docs/BUILDING_ES.md).

### Inicio Rápido (Nix)

1. **Clonar el repositorio:**
   ```bash
   git clone https://codeberg.org/Kyronix/Sanctum.git
   cd Sanctum
   ```

2.  **Activar el Entorno:**
    ```bash
    direnv allow
    ```

3. **Verificar el entorno:**
   ```bash
   # Or without --release
   nix develop -c cargo run --release
   ```

Para instrucciones detalladas en otras plataformas (Linux manual, macOS, Windows), revisa [docs/BUILDING_ES.md](docs/BUILDING_ES.md).

## Transparencia en el Desarrollo

Este es un proyecto Open Source moderno que abraza la evolución del desarrollo de software.

  - **Arquitectura y Visión:** Diseñado y dirigido por humanos, priorizando la privacidad y la seguridad local.
  - **Colaboración con IA:** La mayor parte del código ha sido generada y/o
    refactorizada con LLMs bajo estricta auditoría humana. Los modelos usados,
    en orden, son **Claude Opus 4.5 (ahora 4.6)**, **Claude Sonnet 4.5**, **Codex 5.2 (ahora 5.3)** y
    **Gemini 3 Pro**.
  - **Auditabilidad:** El código es abierto para que cualquiera pueda verificar que no hay telemetría oculta ni vectores de ataque.

## Aviso Legal

**Sanctum está actualmente en ALPHA.** Aunque la encriptación utilizada es estándar en la industria, el software está en desarrollo activo. Mantén siempre copias de seguridad de tus claves de recuperación.

## Licencia

Este proyecto es de código abierto y está disponible bajo la **Licencia Pública General GNU v3.0**. Consulta el archivo [LICENSE](LICENSE) para más información.

-----

<div align="center">
<sub>Construido con ❤️, 🦀 Rust y ❄️ Nix</sub>
</div>
