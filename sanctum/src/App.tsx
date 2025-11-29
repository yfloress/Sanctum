import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

const EXPENSE_CATEGORIES = [
  "Alimentación",
  "Transporte",
  "Vivienda",
  "Servicios",
  "Salud",
  "Ocio",
  "Educación",
  "Tecnología",
  "Otros",
];

const INCOME_CATEGORIES = [
  "Salario",
  "Freelance",
  "Inversiones",
  "Regalos",
  "Otros",
];

function App() {
  const [isInitialized, setIsInitialized] = useState<boolean>(false);
  const [isLoading, setIsLoading] = useState<boolean>(true);
  const [password, setPassword] = useState<string>("");
  const [error, setError] = useState<string>("");
  const [dbPath, setDbPath] = useState<string>("");
  const [successMessage, setSuccessMessage] = useState<string>("");

  // Estados del formulario de transacciones
  const [amount, setAmount] = useState<string>("");
  const [description, setDescription] = useState<string>("");
  const [category, setCategory] = useState<string>(EXPENSE_CATEGORIES[0]);
  const [isExpense, setIsExpense] = useState<boolean>(true);
  const [date, setDate] = useState<string>(
    new Date().toISOString().split("T")[0],
  );

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

  function handleExpenseToggle(checked: boolean) {
    setIsExpense(checked);
    // Resetear categoría al primer elemento de la lista correspondiente
    setCategory(checked ? EXPENSE_CATEGORIES[0] : INCOME_CATEGORIES[0]);
  }

  async function handleAddTransaction(e: React.FormEvent) {
    e.preventDefault();
    setError("");
    setSuccessMessage("");

    if (!amount || parseFloat(amount) <= 0) {
      setError("El monto debe ser mayor a cero");
      return;
    }

    if (!description.trim()) {
      setError("La descripción no puede estar vacía");
      return;
    }

    if (!category.trim()) {
      setError("La categoría no puede estar vacía");
      return;
    }

    try {
      setIsLoading(true);
      const amountInCents = Math.round(parseFloat(amount) * 100);
      const dateISO = new Date(date).toISOString();

      const transactionId = await invoke<string>("add_transaction", {
        amount: amountInCents,
        category: category.trim(),
        description: description.trim(),
        date: dateISO,
        isExpense: isExpense,
      });

      setSuccessMessage(
        `Transacción ${isExpense ? "de gasto" : "de ingreso"} creada exitosamente`,
      );

      // Limpiar formulario
      setAmount("");
      setDescription("");
      setCategory("");
      setDate(new Date().toISOString().split("T")[0]);
    } catch (err) {
      setError(`Error al crear transacción: ${err}`);
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
              <h3>Nueva Transacción</h3>
              <form
                onSubmit={handleAddTransaction}
                className="transaction-form"
              >
                <div className="form-row">
                  <div className="form-group">
                    <label htmlFor="amount">Monto ($)</label>
                    <input
                      id="amount"
                      type="number"
                      step="0.01"
                      value={amount}
                      onChange={(e) => setAmount(e.target.value)}
                      placeholder="0.00"
                      disabled={isLoading}
                    />
                  </div>

                  <div className="form-group">
                    <label htmlFor="date">Fecha</label>
                    <input
                      id="date"
                      type="date"
                      value={date}
                      onChange={(e) => setDate(e.target.value)}
                      disabled={isLoading}
                    />
                  </div>
                </div>

                <div className="form-group">
                  <label htmlFor="category">Categoría</label>
                  <select
                    id="category"
                    value={category}
                    onChange={(e) => setCategory(e.target.value)}
                    disabled={isLoading}
                  >
                    {(isExpense ? EXPENSE_CATEGORIES : INCOME_CATEGORIES).map(
                      (cat) => (
                        <option key={cat} value={cat}>
                          {cat}
                        </option>
                      ),
                    )}
                  </select>
                </div>

                <div className="form-group">
                  <label htmlFor="description">Descripción</label>
                  <input
                    id="description"
                    type="text"
                    value={description}
                    onChange={(e) => setDescription(e.target.value)}
                    placeholder="Describe la transacción"
                    disabled={isLoading}
                  />
                </div>

                <div className="form-group">
                  <label className="switch-label">
                    <input
                      type="checkbox"
                      checked={isExpense}
                      onChange={(e) => handleExpenseToggle(e.target.checked)}
                      disabled={isLoading}
                    />
                    <span className="switch-text">
                      {isExpense ? "Gasto" : "Ingreso"}
                    </span>
                  </label>
                </div>

                <button
                  type="submit"
                  className="btn-primary"
                  disabled={isLoading}
                >
                  {isLoading ? "Guardando..." : "Guardar Transacción"}
                </button>
              </form>
            </div>

            <div className="info-section">
              <h3>Estado de la Conexión</h3>
              <div className="status-badge active">Activa</div>
            </div>

            <div className="info-section">
              <h3>Ubicación de la Base de Datos</h3>
              <code className="db-path">{dbPath || "Cargando..."}</code>
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
