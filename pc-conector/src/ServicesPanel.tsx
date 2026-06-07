import React from 'react'
import { ClipboardIcon, MouseIcon, KeyboardIcon, AudioIcon } from './Icons'

interface ServicesProps {
  services: {
    clipboard_sync: boolean
    mouse_sharing: boolean
    keyboard_sharing: boolean
    audio_sharing: boolean
  }
  onUpdate: (services: ServicesProps['services']) => void
}

export default function ServicesPanel({ services, onUpdate }: ServicesProps) {
  const toggle = (key: keyof typeof services) => {
    onUpdate({ ...services, [key]: !services[key] })
  }

  return (
    <div className="panel">
      <h2>Servicios</h2>
      <p className="panel-subtitle">Activa o desactiva los servicios compartidos con el PC remoto</p>
      <div className="services-grid">
        <ServiceToggle
          icon={<ClipboardIcon size={24} />}
          title="Portapapeles"
          desc="Sincronizar portapapeles entre PCs"
          enabled={services.clipboard_sync}
          onToggle={() => toggle('clipboard_sync')}
        />
        <ServiceToggle
          icon={<MouseIcon size={24} />}
          title="Ratón"
          desc="Compartir movimiento del ratón entre pantallas"
          enabled={services.mouse_sharing}
          onToggle={() => toggle('mouse_sharing')}
        />
        <ServiceToggle
          icon={<KeyboardIcon size={24} />}
          title="Teclado"
          desc="Compartir entrada de teclado entre PCs"
          enabled={services.keyboard_sharing}
          onToggle={() => toggle('keyboard_sharing')}
        />
        <ServiceToggle
          icon={<AudioIcon size={24} />}
          title="Audio"
          desc="Transmitir audio entre PCs"
          enabled={services.audio_sharing}
          onToggle={() => toggle('audio_sharing')}
        />
      </div>
    </div>
  )
}

function ServiceToggle({
  icon,
  title,
  desc,
  enabled,
  onToggle
}: {
  icon: React.ReactNode
  title: string
  desc: string
  enabled: boolean
  onToggle: () => void
}) {
  return (
    <div className={`service-card ${enabled ? 'enabled' : ''}`} onClick={onToggle}>
      <div className="service-icon-container">
        {icon}
      </div>
      <div className="service-info">
        <h4>{title}</h4>
        <p>{desc}</p>
      </div>
      <div className={`toggle ${enabled ? 'on' : 'off'}`}>
        <div className="toggle-knob" />
      </div>
    </div>
  )
}
