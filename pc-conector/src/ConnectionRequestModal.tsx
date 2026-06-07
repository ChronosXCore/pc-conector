import { useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { LaptopIcon, CheckIcon } from './Icons'

interface ConnectionRequest {
  ip: string
  hostname: string
}

interface Props {
  request: ConnectionRequest | null
  onClose: () => void
}

export default function ConnectionRequestModal({ request, onClose }: Props) {
  const [alwaysAllow, setAlwaysAllow] = useState(false)
  const [loading, setLoading] = useState(false)

  if (!request) return null

  const handleApprove = async () => {
    try {
      setLoading(true)
      await invoke('approve_connection', { ip: request.ip, alwaysAllow })
      onClose()
    } catch (e) {
      console.error('Error aprobando conexión:', e)
    } finally {
      setLoading(false)
    }
  }

  const handleReject = async () => {
    try {
      setLoading(true)
      await invoke('reject_connection', { ip: request.ip })
      onClose()
    } catch (e) {
      console.error('Error rechazando conexión:', e)
    } finally {
      setLoading(false)
    }
  }

  return (
    <div className="modal-overlay" onClick={handleReject}>
      <div className="modal connection-request-modal" onClick={(e) => e.stopPropagation()}>
        {/* Animated icon */}
        <div className="conn-request-icon">
          <div className="conn-request-pulse" />
          <LaptopIcon size={32} />
        </div>

        <h2 className="conn-request-title">Solicitud de Conexión</h2>
        <p className="conn-request-subtitle">
          Un equipo quiere conectarse a tu PC
        </p>

        <div className="conn-request-card">
          <div className="conn-request-info-row">
            <span className="conn-request-label">Hostname</span>
            <span className="conn-request-value">{request.hostname}</span>
          </div>
          <div className="conn-request-info-row">
            <span className="conn-request-label">Dirección IP</span>
            <span className="conn-request-value mono">{request.ip}</span>
          </div>
        </div>

        <label className="checkbox-container conn-always-allow" style={{ margin: '0 0 20px' }}>
          <input
            type="checkbox"
            checked={alwaysAllow}
            onChange={(e) => setAlwaysAllow(e.target.checked)}
          />
          <span className="custom-checkbox" />
          <span>Permitir siempre a este equipo (sin preguntar)</span>
        </label>

        <div className="conn-request-actions">
          <button
            className="btn btn-danger"
            onClick={handleReject}
            disabled={loading}
          >
            Rechazar
          </button>
          <button
            className="btn btn-success"
            onClick={handleApprove}
            disabled={loading}
          >
            <CheckIcon size={16} />
            {loading ? 'Procesando...' : 'Aceptar'}
          </button>
        </div>
      </div>
    </div>
  )
}
