import { useState } from 'react'
import { QRCode } from 'react-qr-code'

interface ShareModalProps {
  isOpen: boolean
  onClose: () => void
  ipAddress: string
}

export default function ShareModal({ isOpen, onClose, ipAddress }: ShareModalProps) {
  const [copied, setCopied] = useState(false)
  const repoUrl = 'https://github.com/ChronosXCore/pc-conector'

  if (!isOpen) return null

  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(repoUrl)
      setCopied(true)
      setTimeout(() => setCopied(false), 2000)
    } catch (err) {
      console.error('Error al copiar el enlace: ', err)
    }
  }

  return (
    <div className="modal-backdrop" onClick={onClose}>
      <div className="modal-card" onClick={(e) => e.stopPropagation()}>
        <div className="modal-header">
          <h3>Compartir PC Conector</h3>
          <button className="modal-close" onClick={onClose}>&times;</button>
        </div>
        
        <div className="modal-body">
          <p className="modal-description">
            Generando código de descarga para el dispositivo en <span className="highlight-ip">{ipAddress}</span>.
          </p>
          
          <div className="qr-container">
            <div className="qr-wrapper">
              <QRCode 
                value={repoUrl} 
                size={180}
                style={{ height: "auto", maxWidth: "100%", width: "100%" }}
              />
            </div>
          </div>
          
          <p className="qr-instruction">
            Escanea este código QR con tu celular o tablet para ir directamente al repositorio de GitHub y descargar la versión adecuada (Windows o Linux).
          </p>

          <div className="share-link-box">
            <input 
              type="text" 
              readOnly 
              value={repoUrl} 
              className="share-input"
            />
            <button 
              className={`btn ${copied ? 'btn-success' : 'btn-primary'} btn-small`} 
              onClick={handleCopy}
            >
              {copied ? '¡Copiado!' : 'Copiar Enlace'}
            </button>
          </div>
        </div>
      </div>
    </div>
  )
}
