<div align="center">

# 🤝 Guía de Contribución — PC Conector

*¡Gracias por tu interés en contribuir a PC Conector!*
*Thank you for your interest in contributing to PC Conector!*

</div>

---

## 📖 Tabla de Contenidos

- [Código de Conducta](#-código-de-conducta)
- [¿Cómo Contribuir?](#-cómo-contribuir)
- [Reportar Bugs](#-reportar-bugs)
- [Solicitar Funciones](#-solicitar-funciones)
- [Configurar el Entorno](#-configurar-el-entorno)
- [Proceso de Pull Request](#-proceso-de-pull-request)
- [Convenciones de Código](#-convenciones-de-código)
- [Convenciones de Commits](#-convenciones-de-commits)

---

## 📜 Código de Conducta

Este proyecto se rige por el [Código de Conducta](CODE_OF_CONDUCT.md). Al participar, aceptas cumplirlo. Por favor, reporta comportamientos inaceptables.

---

## 💡 ¿Cómo Contribuir?

Hay muchas formas de contribuir, ¡no solo con código!

- 🐛 **Reportar bugs** — Ayúdanos a encontrar y corregir problemas
- 💡 **Sugerir funciones** — Propón mejoras y nuevas ideas
- 📚 **Mejorar documentación** — Clarifica, traduce o amplía la docs
- 🧪 **Escribir tests** — Aumenta la cobertura de pruebas
- 🎨 **Mejorar UI/UX** — Diseño y experiencia de usuario
- 🌐 **Traducciones** — Ayuda a hacer el proyecto más accesible
- ⭐ **¡Dale una estrella!** — Es simple pero muy motivador

---

## 🐛 Reportar Bugs

Antes de crear un reporte de bug:

1. **Busca** si ya existe un [issue similar](https://github.com/ChronosXCore/pc-conector/issues)
2. Asegúrate de estar usando la **versión más reciente**
3. Verifica si el problema es reproducible

Usa el [template de Bug Report](.github/ISSUE_TEMPLATE/bug_report.md) e incluye:
- Pasos detallados para reproducir
- Comportamiento esperado vs actual
- Tu entorno (OS, versión, etc.)
- Logs relevantes de la consola

---

## 💡 Solicitar Funciones

¿Tienes una idea genial? Usa el [template de Feature Request](.github/ISSUE_TEMPLATE/feature_request.md).

Antes de crear una solicitud:
1. Revisa si ya existe una [solicitud similar](https://github.com/ChronosXCore/pc-conector/issues?q=label%3Aenhancement)
2. Considera si la función está alineada con la [visión del proyecto](docs/VISION.md)

---

## ⚙️ Configurar el Entorno de Desarrollo

### Prerrequisitos

```bash
# Verificar Node.js (requiere ≥18)
node --version

# Verificar npm
npm --version

# Verificar Rust (requiere ≥1.75)
rustc --version
cargo --version
```

### Instalación

```bash
# 1. Fork el repositorio en GitHub
# 2. Clonar tu fork
git clone https://github.com/TU-USUARIO/pc-conector.git
cd pc-conector

# 3. Agregar el repositorio original como upstream
git remote add upstream https://github.com/ChronosXCore/pc-conector.git

# 4. Instalar dependencias del frontend
cd pc-conector
npm install

# 5. Ejecutar en modo desarrollo
npm run tauri dev
```

### Estructura del Código

```
pc-conector/src/          # Componentes React (TypeScript)
pc-conector/src-tauri/    # Backend Rust
  ├── src/
  │   ├── main.rs         # Punto de entrada de Tauri
  │   ├── network.rs      # Módulo de red (mDNS + WebSocket)
  │   ├── clipboard.rs    # Sincronización de portapapeles
  │   ├── input.rs        # Captura/simulación de input
  │   └── audio.rs        # Streaming de audio (CPAL + Opus)
  └── Cargo.toml          # Dependencias Rust
```

---

## 🔄 Proceso de Pull Request

1. **Sincroniza** tu fork con el repositorio principal:
   ```bash
   git fetch upstream
   git checkout main
   git merge upstream/main
   ```

2. **Crea una rama** con un nombre descriptivo:
   ```bash
   # Para funciones nuevas:
   git checkout -b feature/nombre-de-la-funcion
   
   # Para correcciones:
   git checkout -b fix/descripcion-del-bug
   
   # Para documentación:
   git checkout -b docs/mejora-documentacion
   ```

3. **Desarrolla tus cambios** siguiendo las convenciones de código.

4. **Haz commit** con mensajes descriptivos (ver [Convenciones de Commits](#-convenciones-de-commits)).

5. **Envía tus cambios** a tu fork:
   ```bash
   git push origin feature/nombre-de-la-funcion
   ```

6. **Abre un Pull Request** en GitHub usando el template provisto.

### Criterios de Revisión

Los PRs serán revisados considerando:
- ✅ Funcionalidad correcta y sin regresiones
- ✅ Código limpio y bien comentado
- ✅ Tests incluidos (si aplica)
- ✅ Documentación actualizada
- ✅ Compatibilidad con Windows y Linux

---

## 🎨 Convenciones de Código

### TypeScript / React

```typescript
// ✅ Correcto: Componentes funcionales con tipos explícitos
interface DeviceCardProps {
  device: NetworkDevice;
  onConnect: (id: string) => void;
}

export const DeviceCard: React.FC<DeviceCardProps> = ({ device, onConnect }) => {
  return <div className="device-card">...</div>;
};

// ❌ Incorrecto: any, sin tipos
const DeviceCard = ({ device, onConnect }: any) => { ... };
```

### Rust

```rust
// ✅ Correcto: Manejo de errores explícito, documentación
/// Descubre dispositivos en la red local usando mDNS.
/// 
/// # Returns
/// Lista de dispositivos encontrados con nombre, IP y tipo.
pub async fn discover_devices() -> Result<Vec<NetworkDevice>, NetworkError> {
    // ...
}

// ✅ Usar ? para propagación de errores
let socket = TcpListener::bind(addr).await?;
```

### CSS

```css
/* ✅ Usar variables CSS del sistema de temas */
.device-card {
  background: var(--glass-bg);
  border: 1px solid var(--border-color);
  box-shadow: var(--shadow-neon);
}

/* ❌ No usar valores hardcoded de colores */
.device-card {
  background: #1e1b4b; /* ❌ */
}
```

---

## 📝 Convenciones de Commits

Usamos [Conventional Commits](https://www.conventionalcommits.org/):

```
tipo(ámbito): descripción corta

[cuerpo opcional]

[footer opcional]
```

### Tipos de Commit

| Tipo | Uso |
|------|-----|
| `feat` | Nueva función |
| `fix` | Corrección de bug |
| `docs` | Solo documentación |
| `style` | Formato, sin cambios lógicos |
| `refactor` | Refactorización sin cambios funcionales |
| `perf` | Mejora de rendimiento |
| `test` | Añadir o modificar tests |
| `chore` | Tareas de mantenimiento |

### Ejemplos

```bash
git commit -m "feat(audio): add Opus codec compression for lower latency"
git commit -m "fix(clipboard): resolve sync delay on Linux (#42)"
git commit -m "docs(readme): update installation instructions for Ubuntu"
git commit -m "perf(network): optimize mDNS discovery scan interval"
```

---

## ❓ ¿Tienes Preguntas?

- Abre un [Issue con la etiqueta `question`](https://github.com/ChronosXCore/pc-conector/issues/new?labels=question)
- Revisa la [documentación del proyecto](docs/)

---

<div align="center">

*¡Gracias por hacer PC Conector mejor! 🚀*
*Thank you for making PC Conector better!*

</div>
