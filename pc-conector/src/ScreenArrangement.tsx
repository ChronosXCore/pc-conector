import { useState, useEffect, useRef } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { ScreensIcon, InfoIcon, LaptopIcon } from './Icons'
import type { ScreenInfo } from './types'

interface RemoteScreens {
  [addr: string]: ScreenInfo[]
}

interface CombinedScreen extends ScreenInfo {
  owner: 'local' | string  // 'local' or peer IP
  ownerLabel: string
}

export default function ScreenArrangement() {
  const [localScreens, setLocalScreens] = useState<ScreenInfo[]>([])
  const [remoteScreens, setRemoteScreens] = useState<RemoteScreens>({})
  const [loading, setLoading] = useState(true)
  const [dragging, setDragging] = useState<string | null>(null)
  const [positions, setPositions] = useState<Record<string, { x: number; y: number }>>({})
  const dragOffset = useRef({ x: 0, y: 0 })
  const containerRef = useRef<HTMLDivElement>(null)

  const refresh = async () => {
    try {
      setLoading(true)
      const [local, remote] = await Promise.all([
        invoke<ScreenInfo[]>('get_local_screens'),
        invoke<RemoteScreens>('get_remote_screens'),
      ])
      setLocalScreens(local)
      setRemoteScreens(remote)
    } catch (e) {
      console.error('Error al obtener pantallas:', e)
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => {
    refresh()
    const interval = setInterval(refresh, 5000)
    return () => clearInterval(interval)
  }, [])

  // Build combined list
  const combined: CombinedScreen[] = [
    ...localScreens.map(s => ({ ...s, owner: 'local', ownerLabel: 'Este equipo' })),
    ...Object.entries(remoteScreens).flatMap(([addr, screens]) =>
      screens.map(s => ({ ...s, id: `${addr}-${s.id}`, owner: addr, ownerLabel: addr }))
    ),
  ]

  // Scale for display
  const SCALE = 0.12
  const MIN_W = 130
  const MIN_H = 80

  const getDisplaySize = (s: ScreenInfo) => ({
    w: Math.max(MIN_W, Math.round(s.width * SCALE)),
    h: Math.max(MIN_H, Math.round(s.height * SCALE)),
  })

  // Get current position (draggable or original)
  const getPos = (s: CombinedScreen) => {
    if (positions[s.id]) return positions[s.id]
    return { x: Math.round(s.x * SCALE), y: Math.round(s.y * SCALE) }
  }

  // Canvas bounds
  const canvasW = 680
  const canvasH = 320

  // Drag handlers
  const onMouseDown = (e: React.MouseEvent, id: string) => {
    e.preventDefault()
    const rect = (e.target as HTMLElement).closest('.screen-item')!.getBoundingClientRect()
    dragOffset.current = { x: e.clientX - rect.left, y: e.clientY - rect.top }
    setDragging(id)
  }

  useEffect(() => {
    if (!dragging) return
    const onMove = (e: MouseEvent) => {
      const container = containerRef.current
      if (!container) return
      const cr = container.getBoundingClientRect()
      const newX = e.clientX - cr.left - dragOffset.current.x
      const newY = e.clientY - cr.top - dragOffset.current.y
      setPositions(prev => ({ ...prev, [dragging]: { x: newX, y: newY } }))
    }
    const onUp = () => setDragging(null)
    window.addEventListener('mousemove', onMove)
    window.addEventListener('mouseup', onUp)
    return () => {
      window.removeEventListener('mousemove', onMove)
      window.removeEventListener('mouseup', onUp)
    }
  }, [dragging])

  const peerCount = Object.keys(remoteScreens).length
  const totalScreens = combined.length

  return (
    <div className="panel">
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start', marginBottom: '12px' }}>
        <div>
          <h2>Configuración de Pantallas</h2>
          <p className="panel-subtitle">
            {peerCount > 0
              ? `${totalScreens} pantalla(s) en total — Este equipo + ${peerCount} PC(s) remoto(s)`
              : 'Muestra las pantallas de todos los equipos conectados'}
          </p>
        </div>
        <button className="btn btn-small" onClick={refresh} disabled={loading}>
          {loading ? 'Actualizando...' : 'Actualizar'}
        </button>
      </div>

      {loading && combined.length === 0 ? (
        <div className="empty-state">
          <div className="empty-icon"><ScreensIcon size={28} /></div>
          <p>Detectando pantallas...</p>
        </div>
      ) : combined.length === 0 ? (
        <div className="empty-state">
          <div className="empty-icon"><ScreensIcon size={28} /></div>
          <p>No se detectaron pantallas. Conecta un monitor adicional.</p>
        </div>
      ) : (
        <div
          className="screen-canvas"
          ref={containerRef}
          style={{ position: 'relative', width: '100%', height: `${canvasH}px`, userSelect: 'none', cursor: dragging ? 'grabbing' : 'default' }}
        >
          {combined.map((screen) => {
            const { w, h } = getDisplaySize(screen)
            const pos = getPos(screen)
            const isLocal = screen.owner === 'local'
            const isDraggingThis = dragging === screen.id

            // Center offset so origin (0,0) is roughly center-left
            const cx = 40 + pos.x
            const cy = canvasH / 2 - 60 + pos.y

            return (
              <div
                key={screen.id}
                className={`screen-item ${isLocal ? 'screen-local' : 'screen-remote'} ${isDraggingThis ? 'screen-dragging' : ''}`}
                style={{
                  position: 'absolute',
                  left: `${cx}px`,
                  top: `${cy}px`,
                  width: `${w}px`,
                  height: `${h}px`,
                  cursor: 'grab',
                  zIndex: isDraggingThis ? 100 : 1,
                }}
                onMouseDown={(e) => onMouseDown(e, screen.id)}
              >
                {isLocal
                  ? <LaptopIcon size={20} style={{ opacity: 0.85, marginBottom: '4px' }} />
                  : <ScreensIcon size={20} style={{ opacity: 0.85, marginBottom: '4px' }} />
                }
                <span style={{ fontSize: '11px', fontWeight: 600 }}>{screen.name}</span>
                <span style={{ fontSize: '9px', opacity: 0.65, marginTop: '2px' }}>
                  {screen.width}×{screen.height}
                </span>
                <span style={{
                  fontSize: '8px',
                  marginTop: '2px',
                  padding: '1px 5px',
                  borderRadius: '999px',
                  background: isLocal ? 'rgba(99,179,237,0.25)' : 'rgba(154,114,243,0.25)',
                  color: isLocal ? '#90cdf4' : '#c4b5fd',
                }}>
                  {screen.ownerLabel}
                </span>
                {screen.is_primary && (
                  <span className="primary-badge">Principal</span>
                )}
              </div>
            )
          })}
        </div>
      )}

      {/* Legend */}
      <div style={{ display: 'flex', gap: '20px', marginTop: '16px', flexWrap: 'wrap' }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: '8px', fontSize: '12px', opacity: 0.75 }}>
          <div style={{ width: '12px', height: '12px', borderRadius: '3px', background: 'rgba(99,179,237,0.3)', border: '1px solid rgba(99,179,237,0.5)' }} />
          <span>Este equipo</span>
        </div>
        {Object.keys(remoteScreens).map(addr => (
          <div key={addr} style={{ display: 'flex', alignItems: 'center', gap: '8px', fontSize: '12px', opacity: 0.75 }}>
            <div style={{ width: '12px', height: '12px', borderRadius: '3px', background: 'rgba(154,114,243,0.3)', border: '1px solid rgba(154,114,243,0.5)' }} />
            <span>{addr}</span>
          </div>
        ))}
      </div>

      <div className="screen-actions" style={{ marginTop: '16px' }}>
        <div className="hint">
          <InfoIcon size={16} />
          <span>Arrastra las pantallas para visualizar cómo están dispuestas. Al conectar un equipo remoto, sus pantallas aparecen aquí automáticamente.</span>
        </div>
      </div>
    </div>
  )
}
