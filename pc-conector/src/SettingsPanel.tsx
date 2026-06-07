import type { AppConfig } from './types'

export default function SettingsPanel({
  config,
  onUpdate
}: {
  config: AppConfig
  onUpdate: (config: AppConfig) => void
}) {
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
    </div>
  )
}
