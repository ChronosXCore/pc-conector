import { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import type { AppConfig } from './types'

export default function SettingsPanel({
  config,
  onUpdate
}: {
  config: AppConfig
  onUpdate: (config: AppConfig) => void
}) {
  const [hotspotStatus, setHotspotStatus] = useState<string>('Off')
  const [hotspotLoading, setHotspotLoading] = useState(false)
  const [hotspotMsg, setHotspotMsg] = useState('')

  useEffect(() => {
    invoke<string>('get_wifi_hotspot_status')
      .then(status => {
        if (status === 'On' || status === 'Off' || status === 'InTransition') {
          setHotspotStatus(status)
        } else {
          setHotspotStatus('Off')
        }
      })
      .catch(err => {
        console.warn('Error al verificar punto de acceso:', err)
      })
  }, [])

  const toggleHotspot = async () => {
    try {
      setHotspotLoading(true)
      setHotspotMsg('')
      const nextEnable = hotspotStatus !== 'On'
      const res = await invoke<string>('toggle_wifi_hotspot', { enable: nextEnable })
      setHotspotStatus(nextEnable ? 'On' : 'Off')
      setHotspotMsg(res)
    } catch (e) {
      setHotspotMsg(`Error: ${e}`)
    } finally {
      setHotspotLoading(false)
    }
  }

  return (
    <div className="panel">
      <h2>Ajustes</h2>
      <p className="panel-subtitle">Configura el comportamiento general y de red de PC Conector</p>

      <div className="settings-section">
        <h3>
          <svg
            xmlns="http://www.w3.org/2000/svg"
            width="18"
            height="18"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="2.2"
            strokeLinecap="round"
            strokeLinejoin="round"
          >
            <rect width="18" height="18" x="3" y="3" rx="2" />
            <path d="M9 3v18" />
          </svg>
          General
        </h3>
        
        <div
          className="toggle-row"
          onClick={() =>
            onUpdate({
              ...config,
              general: { ...config.general, auto_start: !config.general.auto_start }
            })
          }
        >
          <span>Iniciar con el sistema</span>
          <div className={`toggle ${config.general.auto_start ? 'on' : 'off'}`}>
            <div className="toggle-knob" />
          </div>
        </div>

        <div
          className="toggle-row"
          onClick={() =>
            onUpdate({
              ...config,
              general: { ...config.general, auto_connect: !config.general.auto_connect }
            })
          }
        >
          <span>Conectar automáticamente</span>
          <div className={`toggle ${config.general.auto_connect ? 'on' : 'off'}`}>
            <div className="toggle-knob" />
          </div>
        </div>

        <div
          className="toggle-row"
          onClick={() =>
            onUpdate({
              ...config,
              general: { ...config.general, minimize_to_tray: !config.general.minimize_to_tray }
            })
          }
        >
          <span>Minimizar a bandeja</span>
          <div className={`toggle ${config.general.minimize_to_tray ? 'on' : 'off'}`}>
            <div className="toggle-knob" />
          </div>
        </div>

        <div className="select-group" style={{ marginTop: '20px' }}>
          <label>Idioma de la interfaz</label>
          <select
            value={config.general.language}
            onChange={(e) =>
              onUpdate({
                ...config,
                general: { ...config.general, language: e.target.value }
              })
            }
          >
            <option value="es">Español</option>
            <option value="en">English</option>
          </select>
        </div>

        <div className="select-group" style={{ marginTop: '20px' }}>
          <label>Tema visual</label>
          <select
            value={config.general.theme}
            onChange={(e) =>
              onUpdate({
                ...config,
                general: { ...config.general, theme: e.target.value }
              })
            }
          >
            <option value="dark">Oscuro (Espacial)</option>
            <option value="light">Claro (Limpio)</option>
            <option value="system">Tema del Sistema (Automático)</option>
          </select>
        </div>
      </div>

      <div className="settings-section">
        <h3>
          <svg
            xmlns="http://www.w3.org/2000/svg"
            width="18"
            height="18"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="2.2"
            strokeLinecap="round"
            strokeLinejoin="round"
          >
            <path d="M12 22c5.523 0 10-4.477 10-10S17.523 2 12 2 2 6.477 2 12s4.477 10 10 10z" />
            <path d="M12 6v6l4 2" />
          </svg>
          Conexión
        </h3>

        <div
          className="toggle-row"
          onClick={() =>
            onUpdate({
              ...config,
              connection: {
                ...config.connection,
                auto_reconnect: !config.connection.auto_reconnect
              }
            })
          }
        >
          <span>Reconexión automática</span>
          <div className={`toggle ${config.connection.auto_reconnect ? 'on' : 'off'}`}>
            <div className="toggle-knob" />
          </div>
        </div>

        <div
          className="toggle-row"
          onClick={() =>
            onUpdate({
              ...config,
              connection: {
                ...config.connection,
                encryption_enabled: !config.connection.encryption_enabled
              }
            })
          }
        >
          <span>Cifrado de conexión</span>
          <div className={`toggle ${config.connection.encryption_enabled ? 'on' : 'off'}`}>
            <div className="toggle-knob" />
          </div>
        </div>

        <div
          className="toggle-row"
          onClick={() =>
            onUpdate({
              ...config,
              connection: {
                ...config.connection,
                require_approval: !config.connection.require_approval
              }
            })
          }
        >
          <span>Requerir aprobación de conexión</span>
          <div className={`toggle ${config.connection.require_approval ? 'on' : 'off'}`}>
            <div className="toggle-knob" />
          </div>
        </div>

        <div className="select-group" style={{ marginTop: '20px' }}>
          <label>Intervalo de reconexión</label>
          <select
            value={config.connection.reconnect_interval}
            onChange={(e) =>
              onUpdate({
                ...config,
                connection: {
                  ...config.connection,
                  reconnect_interval: Number(e.target.value)
                }
              })
            }
          >
            <option value={5}>5 segundos</option>
            <option value={10}>10 segundos</option>
            <option value={30}>30 segundos</option>
            <option value={60}>60 segundos</option>
          </select>
        </div>

        <div className="select-group" style={{ marginTop: '20px' }}>
          <label>Token de Seguridad (debe ser idéntico en ambos PCs)</label>
          <input
            type="text"
            className="manual-input"
            value={config.connection.security_token}
            onChange={(e) =>
              onUpdate({
                ...config,
                connection: {
                  ...config.connection,
                  security_token: e.target.value
                }
              })
            }
            placeholder="Ej: 123456"
          />
        </div>
      </div>

      <div className="settings-section">
        <h3>
          <svg
            xmlns="http://www.w3.org/2000/svg"
            width="18"
            height="18"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="2.2"
            strokeLinecap="round"
            strokeLinejoin="round"
          >
            <path d="M4.9 19.1C1 15.2 1 8.8 4.9 4.9" />
            <path d="M7.8 16.2c-2.3-2.3-2.3-6.1 0-8.5" />
            <circle cx="12" cy="12" r="2" />
            <path d="M16.2 7.8c2.3 2.3 2.3 6.1 0 8.5" />
            <path d="M19.1 4.9C23 8.8 23 15.2 19.1 19.1" />
          </svg>
          Punto de Acceso Wi-Fi (AP)
        </h3>
        <p className="settings-desc" style={{ fontSize: '13px', opacity: 0.8, marginBottom: '12px' }}>
          Si tus computadoras no están conectadas a la misma red, puedes iniciar un Punto de Acceso (Hotspot) inalámbrico en esta PC y conectar la otra computadora a esta red.
        </p>

        <div className="hotspot-control-card" style={{ padding: '16px', background: 'rgba(255,255,255,0.03)', borderRadius: '8px', border: '1px solid rgba(255,255,255,0.05)' }}>
          <div className="hotspot-status-row" style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
            <span style={{ fontSize: '13px' }}>Estado del Punto de Acceso:</span>
            <span className={`hotspot-status-badge ${hotspotStatus === 'On' ? 'active' : ''}`} style={{
              fontSize: '11px',
              fontWeight: 700,
              padding: '4px 10px',
              borderRadius: '999px',
              background: hotspotStatus === 'On' ? 'rgba(72,187,120,0.2)' : 'rgba(255,255,255,0.05)',
              color: hotspotStatus === 'On' ? '#48bb78' : '#a0aec0'
            }}>
              {hotspotStatus === 'On' ? 'ACTIVO' : hotspotStatus === 'InTransition' ? 'TRANSICIÓN...' : 'INACTIVO'}
            </span>
          </div>

          <button
            className={`btn ${hotspotStatus === 'On' ? 'btn-danger' : 'btn-primary'}`}
            onClick={toggleHotspot}
            disabled={hotspotLoading}
            style={{ width: '100%', marginTop: '12px' }}
          >
            {hotspotLoading ? 'Procesando...' : hotspotStatus === 'On' ? 'DESACTIVAR PUNTO DE ACCESO' : 'ACTIVAR PUNTO DE ACCESO'}
          </button>

          {hotspotMsg && (
            <p className={`hotspot-feedback-msg ${hotspotMsg.startsWith('Error') ? 'error' : 'success'}`} style={{
              marginTop: '10px',
              fontSize: '12px',
              color: hotspotMsg.startsWith('Error') ? '#fc8181' : '#68d391'
            }}>
              {hotspotMsg}
            </p>
          )}

          <div style={{ marginTop: '16px', fontSize: '11px', opacity: 0.7, lineHeight: 1.4 }}>
            <strong>Instrucciones de conexión:</strong>
            <ul style={{ paddingLeft: '16px', margin: '4px 0 0' }}>
              <li><strong>Windows:</strong> Utiliza el nombre de red y contraseña configurados en la opción "Zona con cobertura inalámbrica móvil" (Mobile Hotspot) de Windows.</li>
              <li><strong>Linux:</strong> Se creará una red llamada <code>NetBridgeHotspot</code> con contraseña <code>netbridge1234</code>.</li>
            </ul>
          </div>
        </div>
      </div>
    </div>
  )
}
