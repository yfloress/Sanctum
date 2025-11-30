# Guía de Uso - Interfaz de Bóveda Segura

## Descripción General

La interfaz de Sanctum proporciona una experiencia de usuario intuitiva para gestionar tu bóveda financiera encriptada. La aplicación tiene dos estados principales:

1. **Bóveda Cerrada**: Requiere contraseña para acceder
2. **Bóveda Abierta**: Acceso completo a tus datos financieros

---

## Características Implementadas

### Pantalla de Bóveda Cerrada

**Elementos de la Interfaz:**
- Icono de candado cerrado animado
- Título "Sanctum"
- Campo de ruta de bóveda (opcional) para crear/abrir en una ubicación específica
- Campo de contraseña maestra
- Botón "Abrir Bóveda" y botón "Crear nueva"
- Mensajes de error/éxito
- Información de seguridad AES-256

**Funcionalidad:**
- Verificación automática del estado de la BD al cargar
- Validación de contraseña (mínimo 8 caracteres)
- Mensajes de error descriptivos en caso de fallo
- Animación de carga durante la inicialización
- Auto-focus en el campo de contraseña

**Validaciones:**
- Contraseña no puede estar vacía
- Longitud mínima de 8 caracteres
- Manejo de errores de inicialización

### Pantalla de Bóveda Abierta

**Elementos de la Interfaz:**
- Icono de candado abierto
- Estado de conexión (badge verde "Activa")
- Ruta de la base de datos
- Lista de características de seguridad activas
- Botón "Cerrar Bóveda"

**Información Mostrada:**
- Estado de la conexión a la BD
- Ubicación física del archivo `sanctum.db`
- Confirmación de encriptación AES-256
- Modo WAL activado
- Estado de migraciones

---

## Flujo de Usuario

### Primer Uso

1. La aplicación detecta que no hay bóveda inicializada
2. Se muestra el formulario de contraseña maestra
3. Usuario ingresa una contraseña segura (min. 8 caracteres)
4. Sistema inicializa la base de datos con encriptación SQLCipher
5. Se ejecutan las migraciones automáticamente
6. Usuario es dirigido a la pantalla de bóveda abierta

### Uso Recurrente

1. Al abrir la aplicación, se verifica si hay una conexión activa
2. Si no hay conexión: solicita contraseña y permite escoger ruta (recuerda la última usada)
3. Si hay conexión: muestra directamente la pantalla de bóveda abierta

### Cerrar Sesión

1. Usuario hace clic en "Cerrar Bóveda"
2. La conexión a la BD se cierra de forma segura
3. Usuario regresa a la pantalla de contraseña

---

## Manejo de Errores

### Errores Comunes y Soluciones

**"La contraseña no puede estar vacía"**
- Causa: Campo de contraseña vacío
- Solución: Ingresa una contraseña válida

**"La contraseña debe tener al menos 8 caracteres"**
- Causa: Contraseña muy corta
- Solución: Usa una contraseña más larga y segura

**"La contraseña de la base de datos es inválida"**
- Causa: Contraseña incorrecta para una BD existente
- Solución: Verifica que estés usando la contraseña correcta

**"La base de datos ya está inicializada"**
- Causa: Intento de inicializar una BD ya abierta
- Solución: Recarga la aplicación o cierra la conexión actual

**"No hay ninguna base de datos abierta"**
- Causa: Intento de cerrar una conexión inexistente
- Solución: Primero abre la bóveda

---

## Diseño y Estilos

### Sistema de Colores

**Modo Claro:**
- Fondo: Gradiente púrpura (#667eea a #764ba2)
- Tarjeta: Blanco (#ffffff)
- Texto: Gris oscuro (#1f2937)
- Primario: Azul (#2563eb)
- Éxito: Verde (#10b981)
- Error: Rojo (#ef4444)

**Modo Oscuro:**
- Automático según preferencias del sistema
- Tarjeta: Gris muy oscuro (#111827)
- Texto: Blanco grisáceo (#f9fafb)
- Mantiene los colores de acción

### Animaciones

- **slideIn**: Entrada suave de la tarjeta
- **pulse**: Pulsación del icono de candado
- **shake**: Animación del candado al cargar
- **fadeIn**: Aparición de mensajes
- **spin**: Loader durante operaciones

### Responsive

- Adaptado para móviles (max-width: 640px)
- Padding reducido en pantallas pequeñas
- Iconos y títulos escalados apropiadamente

---

## Arquitectura del Componente

### Estado de React

```typescript
isInitialized: boolean      // Estado de la bóveda
isLoading: boolean           // Operación en progreso
password: string             // Contraseña ingresada
error: string                // Mensaje de error actual
dbPath: string               // Ruta de la BD
successMessage: string       // Mensaje de éxito
```

### Hooks Utilizados

**useEffect:**
- Se ejecuta al montar el componente
- Verifica el estado inicial de la BD
- Carga la ruta de la BD si está inicializada

**useState:**
- Gestiona todos los estados locales del componente
- Actualiza la UI en tiempo real

### Funciones Principales

**checkDatabaseStatus()**
- Verifica si la BD está inicializada
- Actualiza el estado de `isInitialized`
- Carga la ruta de la BD si es necesario

**loadDbPath()**
- Obtiene la ruta del archivo de BD
- Actualiza `dbPath` para mostrarlo en la UI

**handleInitializeVault()**
- Valida la contraseña
- Llama al comando `init_db`
- Maneja errores y éxitos
- Actualiza el estado a "abierta"

**handleCloseVault()**
- Llama al comando `close_db`
- Cierra la conexión de forma segura
- Actualiza el estado a "cerrada"

---

## Seguridad

### Mejores Prácticas Implementadas

1. **Contraseña no se almacena**: Se limpia del estado después de usar
2. **Input type="password"**: Oculta visualmente la contraseña
3. **Validación cliente**: Antes de enviar al backend
4. **Mensajes de error genéricos**: No revelan detalles técnicos
5. **Deshabilitación de botones**: Durante operaciones asíncronas

### Recomendaciones Adicionales

1. **Usa contraseñas fuertes**: Combina letras, números y símbolos
2. **No compartas tu contraseña**: Es la única clave de tus datos
3. **Cierra la bóveda**: Cuando no estés usando la aplicación
4. **Backup regular**: Exporta tus datos periódicamente

---

## Comandos Tauri Utilizados

```typescript
// Verificar estado
await invoke<boolean>("is_db_initialized");

// Inicializar bóveda
await invoke<string>("init_db", { password });

// Obtener ruta
await invoke<string>("get_db_path");

// Cerrar bóveda
await invoke<string>("close_db");
```

---

## Próximas Características

La interfaz está lista para integrar:

1. **Gestión de Transacciones**
   - Crear, leer, actualizar, eliminar transacciones
   - Formularios de ingreso de datos

2. **Dashboard Financiero**
   - Resumen de balance
   - Gráficos de gastos por categoría
   - Tendencias temporales

3. **Categorías Personalizables**
   - CRUD de categorías
   - Iconos y colores personalizados

4. **Reportes y Exportación**
   - Exportar datos a CSV/JSON
   - Reportes PDF
   - Filtros avanzados

5. **Configuración**
   - Cambiar contraseña maestra
   - Temas personalizados
   - Preferencias de la app

---

## Desarrollo

### Estructura de Archivos

```
src/
├── App.tsx           # Componente principal de bóveda
├── App.css           # Estilos completos
└── main.tsx          # Entry point
```

### Ejecutar en Desarrollo

```bash
# Desde el directorio del proyecto
cd sanctum

# Usando Deno (recomendado)
deno task tauri dev

# O usando npm si prefieres
npm run tauri dev
```

### Compilar para Producción

```bash
# Usando Deno
deno task tauri build

# O usando npm
npm run tauri build
```

**Nota**: Este proyecto está configurado para usar **Deno** como runtime principal.

---

## Detalles Técnicos Importantes

### Correcciones Implementadas en db.rs

**1. Uso de `pragma_update` en lugar de `execute`**

El código original intentaba usar `conn.execute()` para establecer PRAGMAs, pero esto causaba errores:

```rust
// INCORRECTO (versión original)
conn.execute(&format!("PRAGMA key = '{}';", password), [])?;

// CORRECTO (versión corregida)
conn.pragma_update(None, "key", password)
    .map_err(|_| DbError::InvalidPassword)?;
```

**Razones del cambio:**
- Evita SQL injection al usar la API segura de rusqlite
- `pragma_update` es el método oficial para modificar PRAGMAs
- Maneja correctamente los valores retornados por los PRAGMAs

**2. Health Check con `query_row`**

```rust
// INCORRECTO (versión original)
self.conn.execute("SELECT 1", [])?;

// CORRECTO (versión corregida)
self.conn.query_row("SELECT 1", [], |_| Ok(()))?;
```

**Problema**: `execute()` espera 0 filas afectadas, pero `SELECT 1` devuelve una fila, causando un error.

**Solución**: `query_row()` consume correctamente la fila retornada y valida la conexión.

**3. Import del trait Manager**

```rust
use tauri::{AppHandle, Manager};
```

Se agregó el trait `Manager` para acceder al método `path()` de `AppHandle`.

---

## Solución de Problemas

**La aplicación no carga:**
- Verifica que el backend de Rust esté compilado
- Revisa la consola del navegador (DevTools)
- Comprueba que todas las dependencias estén instaladas

**Los estilos no se aplican:**
- Asegúrate de que `App.css` esté importado en `App.tsx`
- Limpia la caché del navegador
- Reinicia el servidor de desarrollo

**Los comandos Tauri fallan:**
- Verifica que los comandos estén registrados en `lib.rs`
- Comprueba que el estado `DbState` esté inicializado
- Revisa los logs del backend de Tauri

**Errores relacionados con PRAGMAs:**
- Si ves errores de "query returned unexpected number of rows", el código ya está corregido
- Asegúrate de usar `pragma_update` para PRAGMAs y `query_row` para queries SELECT

**Problemas con Deno:**
- Si no tienes Deno instalado: `curl -fsSL https://deno.land/install.sh | sh`
- Verifica que `deno.lock` esté presente en el proyecto
- Los comandos de npm también funcionan como alternativa

---

## Soporte

Para más información técnica:
- Backend: `src-tauri/src/db.rs` (con correcciones de pragma_update y query_row)
- Comandos: `src-tauri/src/commands.rs`
- Documentación API: `src-tauri/DB_COMMANDS.md` (actualizado con detalles técnicos)
- [Documentación de rusqlite](https://docs.rs/rusqlite/latest/rusqlite/)
- [Documentación de Deno](https://deno.land/manual)

## Ambiente de Desarrollo

Este proyecto utiliza:
- **Backend**: Rust + Tauri v2 + rusqlite 0.32 + SQLCipher
- **Frontend**: React 19 + TypeScript + Vite 7
- **Runtime**: Deno (no npm, aunque npm también funciona)
- **Base de Datos**: SQLite con encriptación AES-256
- **Correcciones**: pragma_update para PRAGMAs, query_row para SELECT
