import { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import './App.css'
import ScreenArrangement from './ScreenArrangement'
import ServicesPanel from './ServicesPanel'
import AudioPanel from './AudioPanel'
import SettingsPanel from './SettingsPanel'
import type { AudioDevice, AppConfig, Tab } from './types'
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
      setStatusMessage('Buscando PCs en la red...')
      const found = await invoke<string[]>('start_discovery')
      setPeers(found)
      setIsSearching(false)
      if (found.length > 0) {
        setStatusMessage(`Se encontraron ${found.length} PC(s) en la red`)
      } else {
        setStatusMessage('No se encontraron PCs. Asegúrate de que PC Conector este abierto en ambos equipos.')
      }
    } catch (e) {
      setIsSearching(false)
      setStatusMessage(`Error: ${e}`)
    }
  }

  const handleConnect = async (addr: string) => {
    try {
      setStatusMessage(`Conectando a ${addr}...`)
      await invoke('connect_to_peer', { addr: addr.split(' - ')[1] || addr })
      await checkConnection()
      setStatusMessage(`Conectado a ${addr}`)
    } catch (e) {
      setStatusMessage(`Error al conectar: ${e}`)
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
              onDiscover={handleDiscover}
              onConnect={handleConnect}
              onDisconnectFromPeer={handleDisconnectFromPeer}
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
    </div>
  )
}

// ===== DASHBOARD =====
function Dashboard({
  connected,
  peers,
  connectedPeers,
  isSearching,
  onDiscover,
  onConnect,
  onDisconnectFromPeer
}: {
  connected: boolean
  peers: string[]
  connectedPeers: string[]
  isSearching: boolean
  onDiscover: () => void
  onConnect: (addr: string) => void
  onDisconnectFromPeer: (addr: string) => void
}) {
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

        <button
          className="btn btn-primary btn-large"
          onClick={onDiscover}
          disabled={isSearching}
        >
          <SearchIcon size={20} />
          {isSearching ? 'BUSCANDO...' : 'BUSCAR PCS EN LA RED'}
        </button>
      </div>

      {peers.length === 0 && !connected && !isSearching && (
        <div className="empty-state">
          <div className="empty-icon">
            <InfoIcon size={28} />
          </div>
          <p>No se encontraron PCs en la red local. Haz clic en el botón de arriba para comenzar a buscar.</p>
        </div>
      )}

      {peers.length > 0 && !isSearching && (
        <div className="peers-list">
          <h3>PCs Encontrados</h3>
          {peers.map((peer, i) => {
            const addr = peer.split(' - ')[1] || peer;
            const isPeerConnected = connectedPeers.includes(addr);
            return (
              <div key={i} className="peer-card">
                <div className="peer-info">
                  <span className={`peer-icon ${isPeerConnected ? 'connected' : ''}`}>
                    <LaptopIcon size={22} />
                  </span>
                  <div className="peer-details">
                    <div style={{ display: 'flex', alignItems: 'center' }}>
                      <span className="peer-name">{peer.split(' - ')[0] || peer}</span>
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
                {isPeerConnected ? (
                  <button
                    className="btn btn-danger btn-small"
                    onClick={() => onDisconnectFromPeer(addr)}
                  >
                    Desconectar
                  </button>
                ) : (
                  <button
                    className="btn btn-primary btn-small"
                    onClick={() => onConnect(peer)}
                  >
                    Conectar
                  </button>
                )}
              </div>
            );
          })}
        </div>
      )}
    </div>
  )
}
