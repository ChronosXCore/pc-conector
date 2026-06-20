import React, { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import type { AudioDevice, AudioRoute } from './types'
import { AudioIcon } from './Icons'
import AudioPatchbay from './AudioPatchbay'

interface AudioPanelProps {
  audio: {
    input_device: string | null
    output_device: string | null
    stream_microphone: boolean
    stream_speakers: boolean
    sample_rate: number
    bitrate: number
    routes: AudioRoute[]
  }
  inputs: AudioDevice[]
  outputs: AudioDevice[]
  connectedPeers: string[]
  onUpdate: (audio: AudioPanelProps['audio']) => void
}

export default function AudioPanel({
  audio,
  inputs,
  outputs,
  connectedPeers,
  onUpdate
}: AudioPanelProps) {
  const [localHostname, setLocalHostname] = useState<string>('PC Local')
  const [remoteDevices, setRemoteDevices] = useState<Record<string, [AudioDevice[], AudioDevice[]]>>({})
  const [statusMessage, setStatusMessage] = useState<string | null>(null)

  // 1. Cargar hostname local
  useEffect(() => {
    invoke<{ hostname: string }>('get_local_ips')
      .then((res) => {
        if (res && res.hostname) {
          setLocalHostname(res.hostname)
        }
      })
      .catch((e) => console.error('Error al obtener hostname local:', e))
  }, [])

  // 2. Cargar dispositivos remotos y escuchar actualizaciones
  const loadRemoteDevices = async () => {
    try {
      const res = await invoke<Record<string, [AudioDevice[], AudioDevice[]]>>('get_remote_audio_devices')
      setRemoteDevices(res)
    } catch (e) {
      console.error('Error al obtener dispositivos remotos:', e)
    }
  }

  useEffect(() => {
    let unlisten: (() => void) | null = null

    const setupListener = async () => {
      try {
        unlisten = await listen('remote-audio-devices-changed', () => {
          loadRemoteDevices()
        })
      } catch (e) {
        console.error('Error al configurar listener de audio remoto:', e)
      }
    }

    setupListener()
    loadRemoteDevices()

    // Solicitar a los peers que sincronicen sus dispositivos de audio
    invoke('refresh_audio_devices').catch((e) => console.error('Error al refrescar dispositivos:', e))

    return () => {
      if (unlisten) unlisten()
    }
  }, [connectedPeers])

  // 3. Aplicar rutas
  const handleApplyRoutes = async (newRoutes: AudioRoute[]) => {
    try {
      setStatusMessage('Aplicando rutas de audio...')
      await invoke('apply_audio_routes', { routes: newRoutes })
      onUpdate({ ...audio, routes: newRoutes })
      setStatusMessage('¡Rutas de audio aplicadas con éxito!')
      setTimeout(() => setStatusMessage(null), 3000)
    } catch (e) {
      setStatusMessage(`Error al aplicar rutas: ${e}`)
    }
  }

  const handleCancelRoutes = () => {
    setStatusMessage('Restaurando configuración previa...')
    // Restaurar a las guardadas en la config actual
    setTimeout(() => setStatusMessage(null), 1500)
  }

  return (
    <div className="panel audio-panel-layout">
      <h2>Enrutador de Audio Visual</h2>
      <p className="panel-subtitle font-medium">
        Gestiona la transmisión de sonido en tiempo real en la red local.
      </p>

      {statusMessage && (
        <div className={`status-banner ${statusMessage.includes('Error') ? 'error' : 'success'}`}>
          {statusMessage}
        </div>
      )}

      {/* Canvas del Patchbay */}
      <AudioPatchbay
        localHostname={localHostname}
        localInputs={inputs}
        localOutputs={outputs}
        connectedPeers={connectedPeers}
        remoteDevices={remoteDevices}
        savedRoutes={audio.routes || []}
        onApply={handleApplyRoutes}
        onCancel={handleCancelRoutes}
      />

      {/* Configuración de Calidad */}
      <div className="audio-section" style={{ marginTop: '30px' }}>
        <h3>
          <AudioIcon size={18} />
          Ajustes de Calidad y Transmisión
        </h3>
        
        <div className="quality-grid">
          <div className="select-group">
            <label>Frecuencia de muestreo</label>
            <select
              value={audio.sample_rate}
              onChange={(e) => onUpdate({ ...audio, sample_rate: Number(e.target.value) })}
            >
              <option value={44100}>44100 Hz (Calidad CD)</option>
              <option value={48000}>48000 Hz (Calidad de Estudio)</option>
              <option value={96000}>96000 Hz (Alta resolución)</option>
            </select>
          </div>

          <div className="select-group">
            <label>Bitrate de transmisión</label>
            <select
              value={audio.bitrate}
              onChange={(e) => onUpdate({ ...audio, bitrate: Number(e.target.value) })}
            >
              <option value={64000}>64 kbps (Bajo consumo)</option>
              <option value={128000}>128 kbps (Estándar recomendado)</option>
              <option value={256000}>256 kbps (Ultra nítido)</option>
            </select>
          </div>
        </div>
      </div>
    </div>
  )
}
