# PC Conector - Tech Stack

## 🛠️ Stack Tecnológico Elegido

### Frontend
| Tecnología | Versión | Propósito |
|-----------|---------|-----------|
| React | 19.x | UI Framework |
| TypeScript | 6.x | Lenguaje frontend |
| Vite | 8.x | Bundler / Dev Server |
| CSS Modules | - | Estilos |

### Backend (Rust)
| Librería | Propósito |
|----------|-----------|
| **Tauri** v2 | Framework de escritorio multiplataforma |
| **rdev** | Captura global de eventos de teclado y mouse |
| **cpal** | Captura y reproducción de audio |
| **libp2p** o **quinn** | Networking P2P (descubrimiento, comunicación) |
| **serde** / **serde_json** | Serialización de datos |
| **tokio** | Async runtime para networking |
| **log** / **tracing** | Logging |

### Protocolos de Red
| Protocolo | Uso |
|-----------|-----|
| **mDNS** (libp2p mDNS) | Descubrimiento automático de peers |
| **TCP** o **QUIC** | Comunicación principal de datos |
| **Opus** (códec) | Compresión de audio en tiempo real |

### Herramientas de Desarrollo
| Herramienta | Uso |
|------------|-----|
| VS Code | Editor principal |
| ESLint | Linting TypeScript |
| rust-analyzer | LSP para Rust |
| cargo | Gestor de paquetes Rust |
| npm | Gestor de paquetes frontend |

## 🎯 Justificación de Elecciones

### ¿Por qué Tauri + Rust?
1. **Rendimiento**: Binarios pequeños, baja huella de memoria
2. **Seguridad**: Rust previene errores de memoria
3. **Multiplataforma**: Un solo código para Windows y Linux
4. **UI Moderna**: Frontend web con toda la flexibilidad de React
5. **Acceso nativo**: Control total del sistema (inputs, audio)

### ¿Por qué no Electron?
- Mayor consumo de RAM
- Binarios más pesados
- Rendimiento inferior para tareas de sistema

### ¿Por qué no Qt/C++?
- Mayor complejidad de desarrollo
- UI menos moderna
- Gestión manual de memoria riesgosa
