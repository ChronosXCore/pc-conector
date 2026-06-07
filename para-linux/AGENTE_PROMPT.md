# 🤖 PROMPT PARA EL AGENTE DE IA — PC Linux (NetBridge v1.2)

Copia TODO el texto a partir de aquí y pégalo en el chat de tu agente de IA en la otra computadora Linux:

---

## TAREA

Eres el agente de configuración de **NetBridge**, una aplicación KVM de red LAN. Tu misión es configurar y ejecutar NetBridge en esta PC Linux para que se conecte automáticamente con el PC Windows de la misma red.

Sigue TODOS los pasos en orden, sin saltarte ninguno.

---

## PASO 1 — Instalar dependencias del sistema

Detecta si el sistema usa `apt`, `pacman`, `dnf` o `zypper` y ejecuta el comando correspondiente:

**Ubuntu/Debian/Mint:**
```bash
sudo apt update && sudo apt install -y curl git build-essential pkg-config libssl-dev libgtk-3-dev libwebkit2gtk-4.1-dev libayatana-appindicator3-dev librsvg2-dev libxdo-dev libxtst-dev libx11-dev libxrandr-dev libxi-dev unzip nodejs npm
```

**Arch Linux/Manjaro:**
```bash
sudo pacman -Syu --noconfirm curl git base-devel pkg-config openssl gtk3 webkit2gtk-4.1 libayatana-appindicator librsvg xdotool libxtst libxrandr nodejs npm
```

**Fedora/RHEL:**
```bash
sudo dnf install -y curl git gcc pkg-config openssl-devel gtk3-devel webkit2gtk4.1-devel librsvg2-devel libxtst-devel libxrandr-devel nodejs npm
```

---

## PASO 2 — Instalar Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source "$HOME/.cargo/env"
rustup update stable
```

Verifica: `rustc --version` → debe mostrar 1.75 o superior.

---

## PASO 3 — Descargar NetBridge desde GitHub

```bash
cd ~
git clone https://github.com/ChronosXCore/pc-conector.git netbridge
cd netbridge/pc-conector
```

Si el repositorio ya existe, actualízalo:
```bash
cd ~/netbridge
git pull origin main
cd pc-conector
```

---

## PASO 4 — Instalar dependencias de Node.js

```bash
npm install
```

---

## PASO 5 — Configurar el Firewall

**UFW (Ubuntu/Debian):**
```bash
sudo ufw allow 9876/udp comment "NetBridge QUIC"
sudo ufw allow 9875/udp comment "NetBridge Discovery"
sudo ufw allow 5353/udp comment "NetBridge mDNS"
sudo ufw reload
```

**firewalld (Fedora/Arch):**
```bash
sudo firewall-cmd --add-port=9876/udp --permanent
sudo firewall-cmd --add-port=9875/udp --permanent
sudo firewall-cmd --add-port=5353/udp --permanent
sudo firewall-cmd --reload
```

---

## PASO 6 — Compilar y ejecutar NetBridge

```bash
cd ~/netbridge/pc-conector
npm run tauri:dev
```

> ⚠️ La primera compilación tarda entre 5 y 15 minutos. Espera hasta ver `Running app` en la salida.

---

## PASO 7 — Verificar que funciona

Una vez que la ventana de NetBridge se abra:

1. En el **Panel Principal**, anota la IP que aparece en "Tu IP"
2. Verifica en los logs: `Servidor QUIC iniciado en puerto 9876`
3. Verifica: `Servicio de descubrimiento mDNS+UDP iniciado`

---

## PASO 8 — Conectar con el PC Windows

**Opción A — Búsqueda mDNS (automática):**
Panel Principal → clic en **"BUSCAR PCS EN LA RED"** (sin marcar "Búsqueda libre")

**Opción B — Búsqueda libre ARP:**
Marcar **"Búsqueda libre (Tabla ARP)"** → clic en **"BUSCAR PCS EN LA RED"**

**Opción C — IP directa (más confiable):**
Panel Principal → "Conexión manual por IP" → escribe la IP del PC Windows → **Conectar**

---

## PASO 9 — Configurar el paso del mouse entre PCs

Una vez conectados:

1. Ve a la pestaña **Pantallas**
2. Verás pantallas locales (azul) y remotas (púrpura)
3. **Arrastra** la pantalla del otro PC al lado donde físicamente está
4. Clic en **"Aplicar disposición"**
5. Ve a **Servicios** → activa **Compartir Mouse** y **Compartir Teclado**
6. Mueve el cursor hasta el borde → pasa automáticamente al otro PC

---

## PASO 10 — Vincular para auto-conexión

1. Panel Principal → en dispositivo encontrado → clic **"Vincular"**
2. Ajustes → activar **"Conectar automáticamente"**

A partir de ahora, al abrir NetBridge en ambas PCs se conectan solos en ~3 segundos.

---

## NOTAS IMPORTANTES

- **Token de seguridad:** Por defecto `123456` (mismo en ambos PCs). Ajustes → Seguridad.
- **Puertos:** UDP 9876 (QUIC), UDP 9875 (discovery), UDP 5353 (mDNS)
- **Modal de aprobación:** al conectar aparece una ventana pidiendo aceptar. Marcar "Permitir siempre" para no volver a preguntar.
- **Wayland:** Si hay problemas con captura de mouse: `GDK_BACKEND=x11 npm run tauri:dev`
- **Grupo input:** Si no captura el mouse: `sudo usermod -aG input $USER` y reinicia sesión.

---

## TROUBLESHOOTING

| Problema | Solución |
|----------|----------|
| `webkit2gtk not found` | `sudo apt install libwebkit2gtk-4.1-dev` |
| No aparece ventana | Verificar `echo $DISPLAY` (debe mostrar `:0` o similar) |
| No conecta con Windows | Usar Conexión manual por IP |
| Error captura mouse | `sudo usermod -aG input $USER` + reiniciar sesión |
| `cargo: command not found` | `source ~/.cargo/env` |
| Compilación lenta/falla | `CARGO_BUILD_JOBS=2 npm run tauri:dev` |

---

Cuando termines, reporta:
- ✅ IP de este PC Linux
- ✅ Si la ventana de NetBridge se abrió
- ✅ Si conectó con el PC Windows
- ❌ Cualquier error encontrado
