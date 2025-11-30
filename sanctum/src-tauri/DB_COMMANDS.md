# Documentación de Comandos de Base de Datos - Sanctum

## Arquitectura del Estado

La aplicación mantiene una conexión global a la base de datos encriptada usando el siguiente estado:

```rust
pub struct DbState {
    pub db: Mutex<Option<Database>>,
}
```

Este estado se gestiona automáticamente por Tauri y es thread-safe.

## Correcciones Importantes Implementadas

### 1. Uso de `pragma_update` en lugar de `execute`
Se cambió de usar `conn.execute()` a `conn.pragma_update()` para los PRAGMAs de SQLite:
- **`PRAGMA key`**: Evita SQL injection al usar la API segura de rusqlite
- **`PRAGMA journal_mode`**: `pragma_update` es el método correcto porque WAL retorna un valor

### 2. Health Check con `query_row`
Se corrigió el health check para usar `query_row` en lugar de `execute`:
- **Problema**: `execute` espera 0 filas afectadas, pero `SELECT 1` devuelve una fila
- **Solución**: `query_row` consume correctamente la fila retornada

### 3. Import de `Manager` trait
Se agregó `use tauri::Manager;` para acceder al método `path()` de `AppHandle`

---

## Comandos Disponibles

### 1. `init_db` - Inicializar Base de Datos

Inicializa la conexión a la base de datos SQLCipher con una contraseña.

#### Parámetros
- `password: string` - Contraseña para encriptar/desencriptar la base de datos
  (usa la ruta por defecto y la guarda como última ruta usada)

#### Retorna
- `Promise<string>` - Mensaje de éxito
- **Throws**: Error si falla la inicialización o si ya existe una conexión

#### Ejemplo TypeScript/JavaScript

```typescript
import { invoke } from '@tauri-apps/api/core';

async function initializeDatabase() {
  try {
    const password = 'mi_password_super_segura_123';
    const result = await invoke<string>('init_db', { password });
    console.log('SUCCESS:', result);
    // Output: "Base de datos inicializada correctamente"
  } catch (error) {
    console.error('ERROR:', error);
    // Ejemplos de errores:
    // - "La contraseña no puede estar vacía"
    // - "La base de datos ya está inicializada..."
    // - "Error al inicializar la base de datos: ..."
  }
}
```

#### Validaciones
- Contraseña no puede estar vacía
- Contraseña mínima de 8 caracteres
- Valida la clave con `cipher_integrity_check` para detectar contraseñas incorrectas
- No permite múltiples inicializaciones simultáneas
- Ejecuta health check automático después de conectar
- Activa WAL mode y ejecuta migraciones

---

### 1.1 `create_db` - Crear Nueva Bóveda

Crea una base de datos nueva en la ruta indicada (o la ruta por defecto si no se indica).

#### Parámetros
- `password: string` - Contraseña maestra
- `path?: string` - Ruta completa del archivo `.db` (opcional)

#### Notas
- Falla si ya existe un archivo en la ruta indicada (usa `open_db` en ese caso).
- Guarda la ruta como la última utilizada.

---

### 1.2 `open_db` - Abrir Bóveda Existente

Abre una base de datos ya creada.

#### Parámetros
- `password: string` - Contraseña maestra
- `path?: string` - Ruta completa (opcional). Si no se pasa:
  - Usa la última ruta guardada (config.json)
  - Si no hay última ruta, usa la ruta por defecto

#### Notas
- Valida la clave con `cipher_integrity_check` para detectar contraseñas incorrectas.
- Guarda la ruta como la última utilizada.

---

### 2. `is_db_initialized` - Verificar Estado

Verifica si existe una conexión activa a la base de datos.

#### Retorna
- `Promise<boolean>` - `true` si está inicializada, `false` si no

#### Ejemplo TypeScript/JavaScript

```typescript
import { invoke } from '@tauri-apps/api/core';

async function checkDatabaseStatus() {
  try {
    const isInitialized = await invoke<boolean>('is_db_initialized');
    
    if (isInitialized) {
      console.log('Base de datos lista para usar');
    } else {
      console.log('Debes inicializar la base de datos primero');
    }
  } catch (error) {
    console.error('Error:', error);
  }
}
```

---

### 3. `close_db` - Cerrar Conexión

Cierra la conexión activa a la base de datos de forma segura.

#### Retorna
- `Promise<string>` - Mensaje de confirmación
- **Throws**: Error si no hay conexión abierta

#### Ejemplo TypeScript/JavaScript

```typescript
import { invoke } from '@tauri-apps/api/core';

async function closeDatabaseConnection() {
  try {
    const result = await invoke<string>('close_db');
    console.log('SUCCESS:', result);
    // Output: "Base de datos cerrada correctamente"
  } catch (error) {
    console.error('ERROR:', error);
    // Error: "No hay ninguna base de datos abierta"
  }
}
```

#### Nota
El destructor de `Connection` se encarga automáticamente de liberar recursos cuando se elimina la referencia.

---

### 4. `get_db_path` - Obtener Ruta de la BD

Obtiene la ruta activa si hay conexión. Si no, retorna la última ruta usada o la ruta por defecto.

#### Retorna
- `Promise<string>` - Ruta absoluta

#### Ejemplo TypeScript/JavaScript

```typescript
import { invoke } from '@tauri-apps/api/core';

async function getDatabasePath() {
  try {
    const dbPath = await invoke<string>('get_db_path');
    console.log('Base de datos ubicada en:', dbPath);
    // Ejemplo Linux: "/home/user/.local/share/sanctum/sanctum.db"
    // Ejemplo Windows: "C:\\Users\\user\\AppData\\Roaming\\sanctum\\sanctum.db"
    // Ejemplo macOS: "/Users/user/Library/Application Support/sanctum/sanctum.db"
  } catch (error) {
    console.error('Error:', error);
  }
}
```

---

## Flujo Completo de Uso

### Componente React/Vue Ejemplo

```typescript
import { invoke } from '@tauri-apps/api/core';
import { useState, useEffect } from 'react';

function DatabaseManager() {
  const [isInitialized, setIsInitialized] = useState(false);
  const [dbPath, setDbPath] = useState('');
  const [error, setError] = useState('');

  // Verificar estado al montar el componente
  useEffect(() => {
    checkStatus();
    loadDbPath();
  }, []);

  async function checkStatus() {
    try {
      const initialized = await invoke<boolean>('is_db_initialized');
      setIsInitialized(initialized);
    } catch (err) {
      setError(`Error verificando estado: ${err}`);
    }
  }

  async function loadDbPath() {
    try {
      const path = await invoke<string>('get_db_path');
      setDbPath(path);
    } catch (err) {
      setError(`Error obteniendo ruta: ${err}`);
    }
  }

  async function handleInitialize(password: string) {
    try {
      setError('');
      const result = await invoke<string>('init_db', { password });
      console.log(result);
      setIsInitialized(true);
    } catch (err) {
      setError(`Error: ${err}`);
    }
  }

  async function handleClose() {
    try {
      setError('');
      const result = await invoke<string>('close_db');
      console.log(result);
      setIsInitialized(false);
    } catch (err) {
      setError(`Error: ${err}`);
    }
  }

  return (
    <div>
      <h2>Gestión de Base de Datos</h2>
      
      <div>
        <strong>Estado:</strong> {isInitialized ? 'Activa' : 'No inicializada'}
      </div>
      
      <div>
        <strong>Ubicación:</strong> {dbPath}
      </div>

      {error && <div style={{ color: 'red' }}>{error}</div>}

      {!isInitialized ? (
        <button onClick={() => {
          const password = prompt('Ingresa tu contraseña:');
          if (password) handleInitialize(password);
        }}>
          Inicializar Base de Datos
        </button>
      ) : (
        <button onClick={handleClose}>
          Cerrar Conexión
        </button>
      )}
    </div>
  );
}

export default DatabaseManager;
```

---

## Seguridad

### Consideraciones Importantes

1. **Nunca hardcodees la contraseña** en el código del frontend
2. **Usa un prompt seguro** o almacenamiento encriptado del SO para obtener la contraseña
3. **No registres la contraseña** en logs o console.log
4. **Implementa rate limiting** para intentos de contraseña en producción
5. La base de datos usa **encriptación AES-256** via SQLCipher

### Ejemplo de Manejo Seguro de Contraseña

```typescript
// NUNCA HACER ESTO
const password = 'mi_password_123'; // Hardcoded!

// HACER ESTO
async function getSecurePassword(): Promise<string> {
  // Opción 1: Prompt del usuario (básico)
  const password = prompt('Ingresa tu contraseña de Sanctum:');
  
  // Opción 2: Modal personalizado con input type="password"
  // Opción 3: Integración con keychain del SO
  // Opción 4: Biometría + derivación de clave
  
  if (!password || password.length < 8) {
    throw new Error('Contraseña inválida');
  }
  
  return password;
}
```

---

## Testing

### Desde DevTools del Navegador

```javascript
// En la consola del navegador (DevTools)
const { invoke } = window.__TAURI__.core;

// Inicializar
await invoke('init_db', { password: 'test123' });

// Verificar estado
await invoke('is_db_initialized');

// Obtener ruta
await invoke('get_db_path');

// Cerrar
await invoke('close_db');
```

### Ejecutar la Aplicación en Desarrollo

Este proyecto usa **Deno** como runtime de Node.js:

```bash
# Desde el directorio del proyecto
cd sanctum

# Ejecutar en modo desarrollo
deno task tauri dev

# Compilar para producción
deno task tauri build
```

**Nota**: Si ves referencias a `npm` en otros archivos, ignóralas. El proyecto está configurado para usar Deno.

---

## Diagrama de Estados

```
┌─────────────────┐
│   App Startup   │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  No Initialized │◄─────────┐
│   (db = None)   │          │
└────────┬────────┘          │
         │                   │
         │ init_db()         │ close_db()
         ▼                   │
┌─────────────────┐          │
│   Initialized   │──────────┘
│  (db = Some())  │
└─────────────────┘
         │
         │ health_check()
         │ transactions CRUD
         │ queries...
         ▼
```

---

## Detalles Técnicos de Implementación

### Configuración Segura de SQLCipher

```rust
// 1. Establecer la contraseña (Encriptación)
// Usamos pragma_update para evitar SQL Injection de forma segura
conn.pragma_update(None, "key", password)
    .map_err(|_| DbError::InvalidPassword)?;

// 2. Activar modo WAL (Rendimiento)
// Usamos pragma_update porque WAL retorna string "wal" y execute fallaría
conn.pragma_update(None, "journal_mode", "WAL")
    .map_err(DbError::Sqlite)?;
```

### Health Check Correcto

```rust
// CORRECTO: Usamos query_row en lugar de execute
// execute espera 0 filas afectadas. SELECT 1 devuelve una fila.
// query_row consume esa fila y retorna Ok, evitando el error.
self.conn
    .query_row("SELECT 1", [], |_| Ok(()))
    .map_err(DbError::Sqlite)?;
```

---

## Próximos Pasos

Una vez que la base de datos esté inicializada, podrás:

1. Crear comandos para operaciones CRUD de transacciones
2. Implementar queries de reportes financieros
3. Agregar exportación/importación de datos
4. Implementar backup automático
5. Agregar sincronización (si se requiere)

---

## Soporte

Para más información sobre la implementación interna, consulta:
- `src/db.rs` - Módulo de base de datos (con correcciones de pragma_update y query_row)
- `src/commands.rs` - Comandos Tauri
- `FRONTEND_GUIDE.md` - Guía de uso del frontend
- [Documentación de Tauri](https://tauri.app)
- [Documentación de SQLCipher](https://www.zetetic.net/sqlcipher/)
- [Documentación de rusqlite](https://docs.rs/rusqlite/latest/rusqlite/)

## Ambiente de Desarrollo

Este proyecto utiliza:
- **Backend**: Rust + Tauri v2 + rusqlite + SQLCipher
- **Frontend**: React + TypeScript + Vite
- **Runtime**: Deno (no npm)
- **Base de Datos**: SQLite con encriptación AES-256
