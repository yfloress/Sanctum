# TODO

## Seguridad

### ✅ Completado

- [x] Limpieza de memoria con `secrecy` crate (SecretString + Zeroize)
- [x] Rate limiting: límite de 5 reintentos con bloqueo de 5 minutos
- [x] Mensajes de error genéricos que no revelan información interna
- [x] Optimización de query de balance (1 query en vez de 2)
- [x] Eliminación de código innecesario (`init_db`, `connection()`,
      `get_db_path` duplicado)

### 🔄 Pendiente

- [ ] Implementar autolock/timeout de sesión con cierre de conexión si hay
      inactividad
- [ ] Comando de cambio de contraseña (re-encryption) para endurecer bóvedas ya
      creadas
- [ ] Verificador de fuerza de contraseña (p.ej. zxcvbn) en frontend
- [ ] Opción para no recordar la ruta de la bóveda o guardarla a través de
      keyring del SO
- [ ] Limpieza explícita de buffers sensibles en frontend (inputs/estados React)

## Funcionalidades

### 🔄 Pendiente

- [ ] Editar transacciones existentes
- [ ] Eliminar transacciones
- [ ] Filtros por fecha, categoría, tipo en el historial
- [ ] Exportación de datos (CSV, JSON)
- [ ] Gráficos de gastos por categoría
- [ ] Categorías personalizables
- [ ] Múltiples cuentas/bóvedas
- [ ] Backup automático
- [ ] Internacionalización (i18n)

## Optimización

### ✅ Completado

- [x] Query de balance optimizada con CASE WHEN
- [x] Frontend optimizado con useCallback y useMemo

### 🔄 Pendiente

- [ ] Paginación de transacciones para grandes volúmenes
- [ ] Cache de balance para evitar recálculos frecuentes
- [ ] Lazy loading de transacciones antiguas
