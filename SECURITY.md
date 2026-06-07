# 🔐 Política de Seguridad / Security Policy

## 🌍 Versiones Soportadas / Supported Versions

| Versión | Soporte de Seguridad |
|---------|---------------------|
| `main` (última) | ✅ Activo |
| Versiones anteriores | ❌ Sin soporte |

---

## 🚨 Reportar una Vulnerabilidad / Reporting a Vulnerability

Si descubres una vulnerabilidad de seguridad en PC Conector, por favor **NO** abras un Issue público.

*If you discover a security vulnerability in PC Conector, please do **NOT** open a public issue.*

### Proceso de Reporte / Reporting Process

1. **Abre un [Security Advisory](https://github.com/ChronosXCore/pc-conector/security/advisories/new)** privado en GitHub.

2. Incluye la siguiente información / Include the following information:
   - Descripción detallada de la vulnerabilidad
   - Pasos para reproducirla
   - Impacto potencial
   - Versión afectada
   - Posible solución (si tienes alguna)

3. **Recibirás una respuesta** dentro de las 48-72 horas.

4. Trabajaremos contigo para **entender y resolver el problema** antes de cualquier divulgación pública.

---

## ⏱️ Tiempo de Respuesta / Response Time

| Severidad | Tiempo de Respuesta | Tiempo de Resolución |
|-----------|---------------------|---------------------|
| 🔴 Crítica | 24 horas | 7 días |
| 🟠 Alta | 48 horas | 14 días |
| 🟡 Media | 72 horas | 30 días |
| 🟢 Baja | 1 semana | 60 días |

---

## 🛡️ Alcance / Scope

### Dentro del Alcance / In Scope

- Vulnerabilidades en la comunicación de red (mDNS, WebSocket, UDP)
- Problemas de autenticación o autorización entre dispositivos
- Exposición de datos sensibles del portapapeles
- Vulnerabilidades en el backend de Rust (`src-tauri/`)
- Problemas que permitan acceso no autorizado al PC

### Fuera del Alcance / Out of Scope

- Vulnerabilidades en dependencias de terceros (reportar upstream)
- Ataques que requieren acceso físico al dispositivo
- Ataques de ingeniería social
- Problemas que afectan solo a redes no confiables (WAN)

---

## 🙏 Reconocimientos / Acknowledgments

Los investigadores que reporten vulnerabilidades válidas serán reconocidos en las notas de la versión (con su permiso).

*Researchers who report valid vulnerabilities will be acknowledged in release notes (with their permission).*

---

## 📞 Contacto / Contact

Para preguntas no urgentes relacionadas con seguridad, puedes abrir un issue con la etiqueta `security-question`.

*For non-urgent security-related questions, you can open an issue with the `security-question` label.*
