import { useState, useEffect, useRef, useCallback } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { ScreensIcon, InfoIcon, LaptopIcon, CheckIcon } from './Icons'
import type { ScreenInfo } from './types'

interface VirtualScreen {
  id: string
  name: string
  owner: string  // 'local' or peer IP
  x: number
  y: number
  width: number
  height: number
  is_primary: boolean
}

interface RemoteScreens {
  [addr: string]: ScreenInfo[]
}

export default function ScreenArrangement() {
  const [localScreens, setLocalScreens] = useState<ScreenInfo[]>([])
  const [remoteScreens, setRemoteScreens] = useState<RemoteScreens>({})
  const [virtualLayout, setVirtualLayout] = useState<VirtualScreen[]>([])
  const [loading, setLoading] = useState(true)
  const [saving, setSaving] = useState(false)
  const [saved, setSaved] = useState(false)
  const [dragging, setDragging] = useState<string | null>(null)
  const [positions, setPositions] = useState<Record<string, { x: number; y: number }>>({})
  const dragOffset = useRef({ x: 0, y: 0 })
  const containerRef = useRef<HTMLDivElement>(null)

  const refresh = useCallback(async () => {
    try {
      setLoading(true)
      const [local, remote, vLayout] = await Promise.all([
        invoke<ScreenInfo[]>('get_local_screens'),
        invoke<RemoteScreens>('get_remote_screens'),
        invoke<VirtualScreen[]>('get_virtual_layout'),
      ])
      setLocalScreens(local)
      setRemoteScreens(remote)
      setVirtualLayout(vLayout)
      // Init positions from virtual layout
      const pos: Record<string, { x: number; y: number }> = {}
      for (const vs of vLayout) {
        pos[vs.id] = { x: vs.x, y: vs.y }
      }
      setPositions(pos)
    } catch (e) {
      console.error('Error al obtener pantallas:', e)
    } finally {
      setLoading(false)
    }
  }, [])

  useEffect(() => {
    refresh()
    const interval = setInterval(refresh, 6000)
    return () => clearInterval(interval)
  }, [refresh])

  // Build combined list from local + remote, applying user positions
  const combined: VirtualScreen[] = virtualLayout.length > 0
    ? virtualLayout
    : [
        ...localScreens.map(s => ({
          ...s, owner: 'local',
        })),
        ...Object.entries(remoteScreens).flatMap(([addr, screens]) =>
          screens.map((s, i) => ({
            ...s,
            id: `${addr}-${s.id}`,
            owner: addr,
            x: (localScreens[0]?.width ?? 1920) + s.x,
          }))
        ),
      ]

  // Scale for display
  const SCALE = 0.10
  const MIN_W = 120
  const MIN_H = 72
  const canvasW = 700
  const canvasH = 300

  const getDisplaySize = (s: VirtualScreen) => ({
    w: Math.max(MIN_W, Math.round(s.width * SCALE)),
    h: Math.max(MIN_H, Math.round(s.height * SCALE)),
  })

  const getPos = (s: VirtualScreen) => {
    if (positions[s.id] !== undefined) return positions[s.id]
    return { x: s.x * SCALE, y: s.y * SCALE }
  }

  // Drag handlers
  const onMouseDown = (e: React.MouseEvent, id: string) => {
    e.preventDefault()
    const rect = (e.target as HTMLElement).closest('.screen-item')!.getBoundingClientRect()
    dragOffset.current = { x: e.clientX - rect.left, y: e.clientY - rect.top }
    setDragging(id)
    setSaved(false)
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

  const handleApplyLayout = async () => {
    try {
      setSaving(true)
      // Build updated VirtualScreen list with new positions
      const updated: VirtualScreen[] = combined.map(vs => {
        const pos = positions[vs.id]
        const realX = pos ? Math.round(pos.x / SCALE) : vs.x
        const realY = pos ? Math.round(pos.y / SCALE) : vs.y
        return { ...vs, x: realX, y: realY }
      })
      await invoke('set_virtual_layout', { layout: updated })
      setVirtualLayout(updated)
      setSaved(true)
      setTimeout(() => setSaved(false), 3000)
    } catch (e) {
      console.error('Error al guardar layout:', e)
    } finally {
      setSaving(false)
    }
  }

  const peerCount = Object.keys(remoteScreens).length
  const totalScreens = combined.length
  const hasUnsavedChanges = dragging === null && Object.keys(positions).length > 0

  return (
    <div className="panel">
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start', marginBottom: '12px' }}>
        <div>
          <h2>Configuración de Pantallas</h2>
          <p className="panel-subtitle">
            {peerCount > 0
              ? `${totalScreens} pantalla(s) — Este equipo + ${peerCount} PC(s) remoto(s) • Arrastra para reposicionar`
              : 'Muestra las pantallas de todos los equipos conectados'}
          </p>
        </div>
        <div style={{ display: 'flex', gap: '8px' }}>
          <button className="btn btn-small" onClick={refresh} disabled={loading}>
            {loading ? 'Actualizando...' : 'Actualizar'}
          </button>
          {hasUnsavedChanges && (
            <button
              className="btn btn-primary btn-small"
              onClick={handleApplyLayout}
              disabled={saving}
            >
              {saved
                ? <><CheckIcon size={14} /> Guardado</>
                : saving ? 'Guardando...' : 'Aplicar disposición'}
            </button>
          )}
        </div>
      </div>

      {/* Legend */}
      <div style={{ display: 'flex', gap: '16px', marginBottom: '12px', flexWrap: 'wrap' }}>
        <div className="screen-legend-item screen-legend-local">
          <div className="screen-legend-dot" />
          <span>Este equipo</span>
        </div>
        {Object.keys(remoteScreens).map(addr => (
          <div key={addr} className="screen-legend-item screen-legend-remote">
            <div className="screen-legend-dot remote" />
            <span>{addr.split(':')[0]}</span>
          </div>
        ))}
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
        <>
          {/* Canvas */}
          <div className="screen-canvas-wrapper">
            <div
              className="screen-canvas"
              ref={containerRef}
              style={{
                position: 'relative',
                width: '100%',
                height: `${canvasH}px`,
                userSelect: 'none',
                cursor: dragging ? 'grabbing' : 'default',
              }}
            >
              {/* Grid lines */}
              <div className="screen-canvas-grid" />

              {combined.map((screen) => {
                const { w, h } = getDisplaySize(screen)
                const pos = getPos(screen)
                const isLocal = screen.owner === 'local'
                const isDraggingThis = dragging === screen.id
                const cx = 20 + pos.x
                const cy = canvasH / 2 - h / 2 + pos.y * SCALE

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
                      ? <LaptopIcon size={18} style={{ opacity: 0.85, marginBottom: '3px' }} />
                      : <ScreensIcon size={18} style={{ opacity: 0.85, marginBottom: '3px' }} />
                    }
                    <span style={{ fontSize: '10px', fontWeight: 700, textAlign: 'center', lineHeight: 1.2 }}>
                      {screen.name.replace('\\\\.\\DISPLAY', 'Display ')}
                    </span>
                    <span style={{ fontSize: '9px', opacity: 0.6, marginTop: '2px' }}>
                      {screen.width}×{screen.height}
                    </span>
                    <span style={{
                      fontSize: '8px',
                      marginTop: '3px',
                      padding: '1px 6px',
                      borderRadius: '999px',
                      background: isLocal ? 'rgba(99,179,237,0.2)' : 'rgba(154,114,243,0.2)',
                      color: isLocal ? '#90cdf4' : '#c4b5fd',
                      maxWidth: '90%',
                      textAlign: 'center',
                      overflow: 'hidden',
                      textOverflow: 'ellipsis',
                      whiteSpace: 'nowrap',
                    }}>
                      {isLocal ? 'Este equipo' : screen.owner.split(':')[0]}
                    </span>
                    {screen.is_primary && (
                      <span className="primary-badge">Principal</span>
                    )}
                  </div>
                )
              })}
            </div>
          </div>

          {/* Info box */}
          <div className="hint" style={{ marginTop: '12px' }}>
            <InfoIcon size={16} />
            <span>
              Arrastra las pantallas para definir la disposición virtual.
              {peerCount > 0
                ? ' El mouse pasará automáticamente al otro PC cuando llegue al borde compartido.'
                : ' Conecta un PC remoto para ver sus pantallas y habilitar el paso del mouse.'}
              {' '}Pulsa <strong>Aplicar disposición</strong> para guardar los cambios.
            </span>
          </div>
        </>
      )}
    </div>
  )
}
