<div align="center">

<h1>
[![Typing SVG](https://readme-typing-svg.herokuapp.com?duration=6500&color=777777&background=00000000&width=500&height=120&lines=++SANCTUM)](https://git.io/typing-svg)
</h1>

[![Typing SVG](https://readme-typing-svg.herokuapp.com?duration=6500&color=777777&background=00000000&width=500&height=120&lines=++SANCTUM)](https://git.io/typing-svg)

<p>
    <strong>Tu fortaleza financiera personal.</strong>
    <br />
    Privado. Encriptado. Local.
  </p>

[English Version](README.md)

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

![Sanctum](./.img/sanctum.png)

## Acerca de

> **NO ESTA LISTO PARA USAR, EN DESARROLLO**

**Sanctum** es una aplicación de escritorio diseñada para quienes se niegan a comprometer su privacidad. A diferencia de las soluciones basadas en la nube o los pesados wrappers web, Sanctum se ejecuta completamente offline en tu máquina Linux con rendimiento nativo.

Tus datos financieros, portafolio de criptomonedas y hábitos se almacenan en una **base de datos SQLite local encriptada con SQLCipher**. Tú tienes las llaves. Sin servidores, sin rastreo, sin fugas.

## Características Principales

- **Encriptación de Grado Militar:** Arquitectura de conocimiento cero. La base de datos es ilegible sin tu contraseña maestra (AES-256 + 600k iteraciones).
- **Libro Mayor Financiero:** Gestión avanzada de billeteras, ingresos, gastos y transferencias. Auditoría completa.
- **Portafolio Crypto:** Seguimiento de inversiones, cálculo de PnL (Ganancias/Pérdidas) y agregación de activos multi-wallet.
- **Seguimiento de Hábitos:** Monitoreo de rachas y consistencia diaria integrado en tu flujo de trabajo.
- **Analíticas:** Gráficos de trayectoria de patrimonio neto y desglose de gastos.
- **Rendimiento Nativo:** Construido con Slint y Skia para una interfaz ligera y acelerada por GPU (Sin consumo de RAM de WebKit/Chromium).
- **Nativo de Linux:** Optimizado para el ecosistema de escritorio Linux (Soporte Wayland y X11).

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

Este proyecto utiliza **Nix Flakes** para garantizar un entorno reproducible sin contaminar tu sistema global. No necesitas instalar Rust o bibliotecas del sistema manualmente.

### Requisitos Previos

- [Gestor de Paquetes Nix](https://nixos.org/download.html)
- [Direnv](https://direnv.net/) (Opcional, pero muy recomendado)
- Git

### Inicio Rápido

1. **Clonar el repositorio:**
   ```bash
   git clone https://codeberg.org/Kyronix/Sanctum.git
   cd Sanctum
   ````

2.  **Activar el Entorno:**

      - **Opción A (Recomendada con `direnv`):** Si tienes `direnv` instalado, simplemente permite el entorno. Esto cargará automáticamente Rust, dependencias de Slint y bibliotecas del sistema (Wayland/GL) al entrar en la carpeta.

        ```bash
        direnv allow
        ```

      - **Opción B (Manual con Nix):**

        ```bash
        nix develop
        ```

3.  **Ejecutar en Modo Desarrollo:** Una vez dentro de la shell de Nix, lanza la app:

    ```bash
    cargo run
    ```

Para instrucciones detalladas en otras plataformas (Linux manual, macOS, Windows), revisa [docs/BUILDING_ES.md](docs/BUILDING_ES.md).

## Transparencia en el Desarrollo

Este es un proyecto Open Source moderno que abraza la evolución del desarrollo de software.

  - **Arquitectura y Visión:** Diseñado y dirigido por humanos, priorizando la privacidad y la seguridad local.
  - **Colaboración con IA:** Partes del código han sido generadas y refactorizadas con la asistencia de LLMs avanzados. Modelos como **Gemini 3 Pro**, **Claude Opus 4.5**, **Claude Sonnet 4.5** y **Codex 5.2** han sido utilizados bajo estricta supervisión humana y auditoría para asegurar la seguridad y la lógica de negocio.
  - **Auditabilidad:** El código es abierto para que cualquiera pueda verificar que no hay telemetría oculta ni vectores de ataque.

## Aviso Legal

**Sanctum está actualmente en ALPHA.** Aunque la encriptación utilizada es estándar en la industria, el software está en desarrollo activo. Mantén siempre copias de seguridad de tus claves de recuperación.

## Licencia

Este proyecto es de código abierto y está disponible bajo la **Licencia Pública General GNU v3.0**. Consulta el archivo [LICENSE](LICENSE) para más información.

-----

<div align="center">
<sub>Construido con ❤️, 🦀 Rust y ❄️ Nix</sub>
</div>
