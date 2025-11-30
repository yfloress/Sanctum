# TODO

## Seguridad
- Añadir límite de reintentos y auto-bloqueo tras fallos de contraseña.
- Implementar autolock/timeout de sesión con cierre de conexión si hay inactividad.
- Comando de cambio de contraseña (re-encryption) para endurecer bóvedas ya creadas.
- Verificador de fuerza de contraseña (p.ej. zxcvbn) en frontend y validación adicional en backend.
- Opción para no recordar la ruta de la bóveda o guardarla a través de keyring del SO.
- Limpieza explícita de buffers sensibles en frontend (inputs/estados) y backend cuando sea posible.
- Revisión de logs/errores para evitar detalles internos en mensajes a usuario.
