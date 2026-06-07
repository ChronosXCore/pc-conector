import { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import './App.css'
import ScreenArrangement from './ScreenArrangement'
import ServicesPanel from './ServicesPanel'
import AudioPanel from './AudioPanel'
import SettingsPanel from './SettingsPanel'
import type { AudioDevice, AppConfig, Tab } from './types'
import ShareModal from './ShareModal'
import {
  DashboardIcon,
  ScreensIcon,
  ServicesIcon,
  AudioIcon,
  SettingsIcon,
  LaptopIcon,
  SearchIcon,
  InfoIcon,
  CheckIcon
} from './Icons'

export default function App() {
  const [activeTab, setActiveTab] = useState<Tab>('dashboard')
  const [connected, setConnected] = useState(false)
  const [connectedPeers, setConnectedPeers] = useState<string[]>([])
  const [peers, setPeers] = useState<string[]>([])
  const [config, setConfig] = useState<AppConfig | null>(null)
  const [audioInputs, setAudioInputs] = useState<AudioDevice[]>([])
  const [audioOutputs, setAudioOutputs] = useState<AudioDevice[]>([])
  const [statusMessage, setStatusMessage] = useState('Listo para conectar')
  const [isSearching, setIsSearching] = useState(false)
  const [isFreeSearch, setIsFreeSearch] = useState(false)
  const [shareModalOpen, setShareModalOpen] = useState(false)
  const [shareTargetIp, setShareTargetIp] = useState('')
  const [connectingAddr, setConnectingAddr] = useState<string | null>(null)

  useEffect(() => {
    loadConfig()
    checkConnection()
  }, [])

  // Dynamic theme management
  useEffect(() => {
    if (!config?.general?.theme) return
    const theme = config.general.theme
    if (theme === 'light') {
      document.documentElement.setAttribute('data-theme', 'light')
    } else if (theme === 'dark') {
      document.documentElement.setAttribute('data-theme', 'dark')
    } else {
      document.documentElement.removeAttribute('data-theme')
    }
  }, [config?.general?.theme])

  const loadConfig = async () => {
    try {
      const cfg = await invoke<AppConfig>('get_config')
      setConfig(cfg)
    } catch (e) {
      console.error('Error al cargar config:', e)
    }
  }

  const checkConnection = async () => {
    try {
      const status = await invoke<boolean>('get_connection_status')
      setConnected(status)
      const activePeers = await invoke<string[]>('get_connected_peers')
      setConnectedPeers(activePeers)
    } catch (e) {
      console.error(e)
    }
  }

  const handleDiscover = async () => {
    try {
      setIsSearching(true)
      setStatusMessage('Buscando dispositivos en la red...')
      
      const delay = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));
      
      if (isFreeSearch) {
        // Ejecutar la llamada Tauri y el retraso artificial en paralelo para mostrar la animación
        const [found] = await Promise.all([
          invoke<string[]>('start_free_discovery'),
          delay(1800)
        ]);
        setPeers(found)
        setIsSearching(false)
        if (found.length > 0) {
          setStatusMessage(`Se encontraron ${found.length} dispositivo(s) en la red (ARP)`)
        } else {
          setStatusMessage('No se encontraron dispositivos en la tabla ARP.')
        }
      } else {
        const found = await invoke<string[]>('start_discovery')
        setPeers(found)
        setIsSearching(false)
        if (found.length > 0) {
          setStatusMessage(`Se encontraron ${found.length} PC(s) en la red (mDNS)`)
        } else {
          setStatusMessage('No se encontraron PCs con mDNS. Intenta "Búsqueda libre" o Conexión Manual.')
        }
      }
    } catch (e) {
      setIsSearching(false)
      setStatusMessage(`Error: ${e}`)
    }
  }

  const handleConnect = async (addr: string) => {
    const targetAddr = addr.split(' - ')[1] || addr;
    try {
      setConnectingAddr(targetAddr)
      setStatusMessage(`Conectando a ${targetAddr}...`)
      await invoke('connect_to_peer', { addr: targetAddr })
      await checkConnection()
      setStatusMessage(`Conectado a ${targetAddr}`)
    } catch (e) {
      setStatusMessage(`Error al conectar: ${e}`)
    } finally {
      setConnectingAddr(null)
    }
  }

  const handleDisconnectFromPeer = async (addr: string) => {
    try {
      setStatusMessage(`Desconectando de ${addr}...`)
      await invoke('disconnect_from_peer', { addr })
      await checkConnection()
      setStatusMessage(`Desconectado de ${addr}`)
    } catch (e) {
      setStatusMessage(`Error al desconectar de ${addr}: ${e}`)
    }
  }

  const handleDisconnect = async () => {
    try {
      await invoke('disconnect')
      await checkConnection()
      setStatusMessage('Desconectado de todos los dispositivos')
    } catch (e) {
      setStatusMessage(`Error: ${e}`)
    }
  }

  const handleUpdateConfig = async (newConfig: AppConfig) => {
    try {
      await invoke('update_config', { config: newConfig })
      setConfig(newConfig)
      setStatusMessage('Configuración guardada')
    } catch (e) {
      setStatusMessage(`Error: ${e}`)
    }
  }

  const loadAudioDevices = async () => {
    try {
      const [inputs, outputs] = await invoke<[AudioDevice[], AudioDevice[]]>('list_audio_devices')
      setAudioInputs(inputs)
      setAudioOutputs(outputs)
    } catch (e) {
      console.error('Error al listar dispositivos:', e)
    }
  }

  return (
    <div className="app">
      <aside className="sidebar">
        <div className="sidebar-header">
          <h1>PC Conector</h1>
          <div className={`status-indicator ${connected ? 'connected' : 'disconnected'}`}>
            <span className="status-dot" />
            <span>{connected ? 'Conectado' : 'Desconectado'}</span>
          </div>
        </div>

        <nav className="sidebar-nav">
          <button
            className={`nav-item ${activeTab === 'dashboard' ? 'active' : ''}`}
            onClick={() => setActiveTab('dashboard')}
          >
            <span className="nav-icon">
              <DashboardIcon />
            </span>
            <span>PANEL PRINCIPAL</span>
          </button>
          <button
            className={`nav-item ${activeTab === 'screens' ? 'active' : ''}`}
            onClick={() => setActiveTab('screens')}
          >
            <span className="nav-icon">
              <ScreensIcon />
            </span>
            <span>PANTALLAS</span>
          </button>
          <button
            className={`nav-item ${activeTab === 'services' ? 'active' : ''}`}
            onClick={() => setActiveTab('services')}
          >
            <span className="nav-icon">
              <ServicesIcon />
            </span>
            <span>SERVICIOS</span>
          </button>
          <button
            className={`nav-item ${activeTab === 'audio' ? 'active' : ''}`}
            onClick={() => { setActiveTab('audio'); loadAudioDevices() }}
          >
            <span className="nav-icon">
              <AudioIcon />
            </span>
            <span>AUDIO</span>
          </button>
          <button
            className={`nav-item ${activeTab === 'settings' ? 'active' : ''}`}
            onClick={() => setActiveTab('settings')}
          >
            <span className="nav-icon">
              <SettingsIcon />
            </span>
            <span>AJUSTES</span>
          </button>
        </nav>
      </aside>

      <main className="main-content">
        <header className="top-bar">
          <div className="status-bar">
            <span className="status-message">
              {connected ? <CheckIcon size={16} color="var(--success)" /> : <InfoIcon size={16} color="var(--text-secondary)" />}
              {statusMessage}
            </span>
            {connected && (
              <button className="btn btn-danger btn-small" onClick={handleDisconnect}>
                Desconectar
              </button>
            )}
          </div>
        </header>

        <div className="content">
          {activeTab === 'dashboard' && (
            <Dashboard
              connected={connected}
              peers={peers}
              connectedPeers={connectedPeers}
              isSearching={isSearching}
              isFreeSearch={isFreeSearch}
              setIsFreeSearch={setIsFreeSearch}
              connectingAddr={connectingAddr}
              onDiscover={handleDiscover}
              onConnect={handleConnect}
              onDisconnectFromPeer={handleDisconnectFromPeer}
              onShareApp={(ip) => {
                setShareTargetIp(ip)
                setShareModalOpen(true)
              }}
            />
          )}
          {activeTab === 'screens' && config && (
            <ScreenArrangement
              screens={config.screens}
              onUpdate={(screens) => handleUpdateConfig({ ...config, screens })}
            />
          )}
          {activeTab === 'services' && config && (
            <ServicesPanel
              services={config.services}
              onUpdate={(services) => handleUpdateConfig({ ...config, services })}
            />
          )}
          {activeTab === 'audio' && config && (
            <AudioPanel
              audio={config.audio}
              inputs={audioInputs}
              outputs={audioOutputs}
              onUpdate={(audio) => handleUpdateConfig({ ...config, audio })}
            />
          )}
          {activeTab === 'settings' && config && (
            <SettingsPanel
              config={config}
              onUpdate={handleUpdateConfig}
            />
          )}
        </div>
      </main>

      <ShareModal 
        isOpen={shareModalOpen}
        onClose={() => setShareModalOpen(false)}
        ipAddress={shareTargetIp}
      />
    </div>
  )
}

// ===== DASHBOARD =====
function Dashboard({
  connected,
  peers,
  connectedPeers,
  isSearching,
  isFreeSearch,
  setIsFreeSearch,
  connectingAddr,
  onDiscover,
  onConnect,
  onDisconnectFromPeer,
  onShareApp
}: {
  connected: boolean
  peers: string[]
  connectedPeers: string[]
  isSearching: boolean
  isFreeSearch: boolean
  setIsFreeSearch: (val: boolean) => void
  connectingAddr: string | null
  onDiscover: () => void
  onConnect: (addr: string) => void
  onDisconnectFromPeer: (addr: string) => void
  onShareApp: (ip: string) => void
}) {
  const [manualIp, setManualIp] = useState('')

  const isManualConnecting = connectingAddr === manualIp;

  return (
    <div className="panel">
      <h2>Panel Principal</h2>
      <p className="panel-subtitle">Conecta dispositivos en tu red local en segundos</p>

      <div className="dashboard-hero">
        <div className={`radar-container ${isSearching ? 'scanning' : ''}`}>
          <div className="radar-circle" />
          <div className="radar-circle" />
          <div className="radar-circle" />
          {isSearching && <div className="radar-wave" />}
          {isSearching && <div className="radar-wave radar-wave-2" />}
          <div className="radar-core">
            <LaptopIcon size={32} />
          </div>
        </div>

        <label className="checkbox-container">
          <input
            type="checkbox"
            checked={isFreeSearch}
            onChange={(e) => setIsFreeSearch(e.target.checked)}
            disabled={isSearching}
          />
          <span className="custom-checkbox" />
          <span>Búsqueda libre (Tabla ARP)</span>
        </label>

        <button
          className="btn btn-primary btn-large"
          onClick={onDiscover}
          disabled={isSearching}
        >
          <SearchIcon size={20} />
          {isSearching ? 'BUSCANDO...' : 'BUSCAR PCS EN LA RED'}
        </button>
      </div>

      <div className="manual-connect-card">
        <h3>Conexión manual por IP</h3>
        <div className="manual-connect-form">
          <input
            type="text"
            placeholder="Ej: 192.168.1.15"
            value={manualIp}
            onChange={(e) => setManualIp(e.target.value)}
            disabled={!!connectingAddr}
            className="manual-input"
            onKeyDown={(e) => {
              if (e.key === 'Enter' && manualIp && !connectingAddr) {
                onConnect(manualIp);
              }
            }}
          />
          <button
            className="btn btn-primary"
            onClick={() => onConnect(manualIp)}
            disabled={!manualIp || !!connectingAddr}
          >
            {isManualConnecting ? 'Conectando...' : 'Conectar'}
          </button>
        </div>
      </div>

      {peers.length === 0 && !connected && !isSearching && (
        <div className="empty-state">
          <div className="empty-icon">
            <InfoIcon size={28} />
          </div>
          <p>No se encontraron PCs en la red local. Haz clic en el botón de arriba para comenzar a buscar o usa Conexión manual por IP.</p>
        </div>
      )}

      {peers.length > 0 && !isSearching && (
        <div className="peers-list">
          <h3>{isFreeSearch ? 'Dispositivos locales (ARP)' : 'PCs con PC Conector (mDNS)'}</h3>
          {peers.map((peer, i) => {
            const parts = peer.split(' - ');
            const name = parts[0] || peer;
            const addr = parts[1] || peer;
            const isPeerConnected = connectedPeers.includes(addr);
            const isThisConnecting = connectingAddr === addr;
            return (
              <div key={i} className="peer-card">
                <div className="peer-info">
                  <span className={`peer-icon ${isPeerConnected ? 'connected' : ''}`}>
                    <LaptopIcon size={22} />
                  </span>
                  <div className="peer-details">
                    <div style={{ display: 'flex', alignItems: 'center' }}>
                      <span className="peer-name">{parts[1] ? name : 'Dispositivo Local'}</span>
                      {isPeerConnected && (
                        <span style={{
                          fontSize: '11px',
                          backgroundColor: 'rgba(46, 213, 115, 0.15)',
                          color: '#2ed573',
                          padding: '2px 6px',
                          borderRadius: '4px',
                          marginLeft: '8px',
                          fontWeight: 'bold'
                        }}>
                          Conectado
                        </span>
                      )}
                    </div>
                    <span className="peer-address">{addr}</span>
                  </div>
                </div>
                <div style={{ display: 'flex', gap: '8px' }}>
                  {isFreeSearch && (
                    <button
                      className="btn btn-primary btn-small"
                      onClick={() => onShareApp(addr)}
                      style={{ background: 'rgba(139, 92, 246, 0.12)', color: 'var(--accent)', boxShadow: 'none' }}
                      disabled={!!connectingAddr}
                    >
                      Compartir
                    </button>
                  )}
                  {isPeerConnected ? (
                    <button
                      className="btn btn-danger btn-small"
                      onClick={() => onDisconnectFromPeer(addr)}
                      disabled={!!connectingAddr}
                    >
                      Desconectar
                    </button>
                  ) : (
                    <button
                      className="btn btn-primary btn-small"
                      onClick={() => onConnect(peer)}
                      disabled={!!connectingAddr}
                    >
                      {isThisConnecting ? 'Conectando...' : 'Conectar'}
                    </button>
                  )}
                </div>
              </div>
            );
          })}
        </div>
      )}

      <div className="help-card">
        <h4><InfoIcon size={16} /> ¿Problemas para conectar?</h4>
        <p>
          Si no ves los equipos o no conectan, es muy probable que el Firewall de Windows o Linux esté bloqueando la conexión.
        </p>
        <ul className="help-list">
          <li><strong>Windows Firewall:</strong> Asegúrate de permitir el programa <code>app.exe</code> en redes Privadas.</li>
          <li><strong>Linux (UFW):</strong> Habilita los puertos de red: <code>sudo ufw allow 9876/udp</code> y <code>sudo ufw allow 5353/udp</code>.</li>
          <li><strong>IP Directa:</strong> Puedes saltar el descubrimiento escribiendo la IP de la otra PC en el cuadro de arriba "Conexión manual por IP".</li>
        </ul>
      </div>
    </div>
  )
}
