import { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import type { LinkedDevice } from './types'
import { LaptopIcon, CheckIcon } from './Icons'

interface LinkedDevicesPanelProps {
  connectedPeers: string[]
  onConnect: (ip: string) => void
  onDisconnect: (ip: string) => void
  onLink: (ip: string, name: string) => Promise<void>
  onUnlink: (ip: string) => Promise<void>
}

export default function LinkedDevicesPanel({
  connectedPeers,
  onConnect,
  onDisconnect,
  onLink,
  onUnlink,
}: LinkedDevicesPanelProps) {
  const [linkedDevices, setLinkedDevices] = useState<LinkedDevice[]>([])
  const [loading, setLoading] = useState(false)

  const loadLinked = async () => {
    try {
      const devices = await invoke<LinkedDevice[]>('get_linked_devices')
      setLinkedDevices(devices)
    } catch (e) {
      console.error('Error cargando dispositivos vinculados:', e)
    }
  }

  useEffect(() => {
    loadLinked()
  }, [])

  const handleUnlink = async (ip: string) => {
    setLoading(true)
    await onUnlink(ip)
    await loadLinked()
    setLoading(false)
  }

  if (linkedDevices.length === 0) return null

  return (
    <div className="linked-devices-panel">
      <div className="linked-devices-header">
        <span className="linked-devices-title">
          <CheckIcon size={14} color="var(--accent)" />
          Dispositivos vinculados
        </span>
        <span className="linked-devices-count">{linkedDevices.length}</span>
      </div>

      <div className="linked-devices-list">
        {linkedDevices.map((device) => {
          const isConnected = connectedPeers.some(p => p.startsWith(device.ip))
          return (
            <div key={device.ip} className="linked-device-row">
              <div className="linked-device-info">
                <span className={`linked-device-dot ${isConnected ? 'active' : ''}`} />
                <LaptopIcon size={16} />
                <div className="linked-device-text">
                  <span className="linked-device-name">{device.name}</span>
                  <span className="linked-device-ip">{device.ip}</span>
                </div>
              </div>
              <div className="linked-device-actions">
                {isConnected ? (
                  <button
                    className="btn btn-danger btn-tiny"
                    onClick={() => onDisconnect(device.ip)}
                    disabled={loading}
                  >
                    Desconectar
                  </button>
                ) : (
                  <button
                    className="btn btn-primary btn-tiny"
                    onClick={() => onConnect(device.ip)}
                    disabled={loading}
                  >
                    Conectar
                  </button>
                )}
                <button
                  className="btn btn-tiny btn-ghost"
                  onClick={() => handleUnlink(device.ip)}
                  disabled={loading}
                  title="Desvincular dispositivo"
                >
                  ✕
                </button>
              </div>
            </div>
          )
        })}
      </div>
    </div>
  )
}
