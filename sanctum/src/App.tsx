import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

function App() {
  const [isInitialized, setIsInitialized] = useState<boolean>(false);
  const [isLoading, setIsLoading] = useState<boolean>(true);
  const [password, setPassword] = useState<string>("");
  const [error, setError] = useState<string>("");
  const [dbPath, setDbPath] = useState<string>("");
  const [successMessage, setSuccessMessage] = useState<string>("");

  // Verificar el estado de la BD al cargar
  useEffect(() => {
    checkDatabaseStatus();
  }, []);

  async function checkDatabaseStatus() {
    try {
      setIsLoading(true);
      setError("");
      const initialized = await invoke<boolean>("is_db_initialized");
      setIsInitialized(initialized);

      if (initialized) {
        await loadDbPath();
      }
    } catch (err) {
      setError(`Error al verificar estado: ${err}`);
    } finally {
      setIsLoading(false);
    }
  }

  async function loadDbPath() {
    try {
      const path = await invoke<string>("get_db_path");
      setDbPath(path);
    } catch (err) {
      console.error("Error al obtener ruta:", err);
    }
  }

  async function handleInitializeVault(e: React.FormEvent) {
    e.preventDefault();
    setError("");
    setSuccessMessage("");

    if (!password.trim()) {
      setError("La contraseña no puede estar vacía");
      return;
    }

    if (password.length < 8) {
      setError("La contraseña debe tener al menos 8 caracteres");
      return;
    }

    try {
      setIsLoading(true);
      const result = await invoke<string>("init_db", { password });
      setSuccessMessage(result);
      setIsInitialized(true);
      setPassword("");
      await loadDbPath();
    } catch (err) {
      setError(`Error: ${err}`);
    } finally {
      setIsLoading(false);
    }
  }

  async function handleCloseVault() {
    try {
      setIsLoading(true);
      setError("");
      const result = await invoke<string>("close_db");
      setSuccessMessage(result);
      setIsInitialized(false);
      setDbPath("");
    } catch (err) {
      setError(`Error: ${err}`);
    } finally {
      setIsLoading(false);
    }
  }

  if (isLoading && !isInitialized) {
    return (
      <div className="vault-container">
        <div className="vault-card">
          <div className="loader"></div>
          <p>Verificando estado de la bóveda...</p>
        </div>
      </div>
    );
  }

  return (
    <div className="vault-container">
      {!isInitialized ? (
        // Pantalla de Bóveda Cerrada
        <div className="vault-card">
          <div className="vault-header">
            <div className="vault-icon locked">🔒</div>
            <h1>Sanctum</h1>
            <p className="vault-subtitle">Bóveda Financiera Segura</p>
          </div>

          <form onSubmit={handleInitializeVault} className="vault-form">
            <div className="form-group">
              <label htmlFor="password">Contraseña Maestra</label>
              <input
                id="password"
                type="password"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                placeholder="Ingresa tu contraseña"
                disabled={isLoading}
                autoFocus
              />
              <span className="input-hint">Mínimo 8 caracteres</span>
            </div>

            {error && <div className="message error">{error}</div>}
            {successMessage && (
              <div className="message success">{successMessage}</div>
            )}

            <button type="submit" className="btn-primary" disabled={isLoading}>
              {isLoading ? "Abriendo bóveda..." : "Abrir Bóveda"}
            </button>
          </form>

          <div className="vault-footer">
            <p>
              Tus datos financieros están protegidos con encriptación AES-256
            </p>
          </div>
        </div>
      ) : (
        // Pantalla de Bóveda Abierta
        <div className="vault-card open">
          <div className="vault-header">
            <div className="vault-icon unlocked">🔓</div>
            <h1>Bóveda Abierta</h1>
            <p className="vault-subtitle">Tu información está accesible</p>
          </div>

          <div className="vault-info">
            <div className="info-section">
              <h3>Estado de la Conexión</h3>
              <div className="status-badge active">Activa</div>
            </div>

            <div className="info-section">
              <h3>Ubicación de la Base de Datos</h3>
              <code className="db-path">{dbPath || "Cargando..."}</code>
            </div>

            <div className="info-section">
              <h3>Seguridad</h3>
              <ul className="security-features">
                <li>Encriptación AES-256 habilitada</li>
                <li>Modo WAL activado</li>
                <li>Migraciones aplicadas correctamente</li>
              </ul>
            </div>
          </div>

          {error && <div className="message error">{error}</div>}
          {successMessage && (
            <div className="message success">{successMessage}</div>
          )}

          <button
            onClick={handleCloseVault}
            className="btn-secondary"
            disabled={isLoading}
          >
            {isLoading ? "Cerrando..." : "Cerrar Bóveda"}
          </button>

          <div className="vault-footer">
            <p>Recuerda cerrar la bóveda cuando termines</p>
          </div>
        </div>
      )}
    </div>
  );
}

export default App;
