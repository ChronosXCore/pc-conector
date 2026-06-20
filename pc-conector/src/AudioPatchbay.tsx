import React, { useState, useEffect, useRef } from 'react'
import type { AudioDevice } from './types'

export interface AudioRoute {
  source_pc: string
  source_device: string
  dest_pc: string
  dest_device: string
}

interface AudioPatchbayProps {
  localHostname: string
  localInputs: AudioDevice[]  // Captura (Micrófonos)
  localOutputs: AudioDevice[] // Reproducción (Altavoces)
  connectedPeers: string[]
  remoteDevices: Record<string, [AudioDevice[], AudioDevice[]]> // peerIp -> [inputs, outputs]
  savedRoutes: AudioRoute[]
  onApply: (routes: AudioRoute[]) => void
  onCancel: () => void
}

interface Node {
  id: string // "local" o IP del peer
  name: string
  x: number
  y: number
  width: number
  height: number
  inputs: string[]  // Reproducción (Altavoces, etc.) - Lado Izquierdo
  outputs: string[] // Captura (Micrófonos, etc.) - Lado Derecho
}

interface DragState {
  fromNodeId: string
  fromPortIndex: number
  fromPortType: 'input' | 'output'
  fromPortName: string
  startX: number
  startY: number
  currentX: number
  currentY: number
}

export default function AudioPatchbay({
  localHostname,
  localInputs,
  localOutputs,
  connectedPeers,
  remoteDevices,
  savedRoutes,
  onApply,
  onCancel
}: AudioPatchbayProps) {
  const [routes, setRoutes] = useState<AudioRoute[]>([])
  const [nodes, setNodes] = useState<Node[]>([])
  const [dragState, setDragState] = useState<DragState | null>(null)
  const [hoveredRouteIndex, setHoveredRouteIndex] = useState<number | null>(null)
  
  const containerRef = useRef<SVGSVGElement | null>(null)

  // Sincronizar rutas locales al montar o cambiar savedRoutes
  useEffect(() => {
    setRoutes(savedRoutes)
  }, [savedRoutes])

  // Calcular la posición y tamaño de los nodos dinámicamente
  useEffect(() => {
    const list: Node[] = []
    
    // 1. Nodo Local
    const localInputNames = localOutputs.map(d => d.name || 'Predeterminado') // Playback en el lado izquierdo
    const localOutputNames = localInputs.map(d => d.name || 'Predeterminado') // Captura en el lado derecho

    // Altura mínima según la cantidad de puertos
    const maxPortsLocal = Math.max(localInputNames.length, localOutputNames.length)
    const localHeight = Math.max(220, 80 + maxPortsLocal * 45)

    list.push({
      id: 'local',
      name: `${localHostname} (Este Equipo)`,
      x: 50,
      y: 50,
      width: 280,
      height: localHeight,
      inputs: localInputNames,
      outputs: localOutputNames
    })

    // 2. Nodos Remotos
    connectedPeers.forEach((peerIp, index) => {
      const cleanIp = peerIp.split(':')[0]
      const peerData = remoteDevices[peerIp] || remoteDevices[cleanIp] || [[], []]
      const peerInputs = peerData[1] || []  // Outputs físicos del remoto = Playback (lado izquierdo)
      const peerOutputs = peerData[0] || [] // Inputs físicos del remoto = Captura (lado derecho)

      const peerInputNames = peerInputs.map(d => d.name || 'Predeterminado')
      const peerOutputNames = peerOutputs.map(d => d.name || 'Predeterminado')

      const maxPortsPeer = Math.max(peerInputNames.length, peerOutputNames.length)
      const peerHeight = Math.max(220, 80 + maxPortsPeer * 45)

      // Colocar los nodos remotos alineados a la derecha
      list.push({
        id: peerIp,
        name: `PC Remota (${cleanIp})`,
        x: 480,
        y: 50 + index * 260,
        width: 280,
        height: peerHeight,
        inputs: peerInputNames,
        outputs: peerOutputNames
      })
    })

    setNodes(list)
  }, [localHostname, localInputs, localOutputs, connectedPeers, remoteDevices])

  // Helpers para obtener coordenadas exactas de puertos
  const getPortCoords = (node: Node, type: 'input' | 'output', index: number) => {
    const headerHeight = 65
    const portSpacing = 45
    const x = type === 'input' ? node.x : node.x + node.width
    const y = node.y + headerHeight + index * portSpacing
    return { x, y }
  }

  // Manejo de Drag and Drop
  const handlePortMouseDown = (
    e: React.MouseEvent,
    node: Node,
    portIndex: number,
    portType: 'input' | 'output',
    portName: string
  ) => {
    e.preventDefault()
    if (!containerRef.current) return

    const rect = containerRef.current.getBoundingClientRect()
    const x = e.clientX - rect.left
    const y = e.clientY - rect.top

    setDragState({
      fromNodeId: node.id,
      fromPortIndex: portIndex,
      fromPortType: portType,
      fromPortName: portName,
      startX: x,
      startY: y,
      currentX: x,
      currentY: y
    })
  }

  const handleMouseMove = (e: React.MouseEvent) => {
    if (!dragState || !containerRef.current) return

    const rect = containerRef.current.getBoundingClientRect()
    const x = e.clientX - rect.left
    const y = e.clientY - rect.top

    setDragState({
      ...dragState,
      currentX: x,
      currentY: y
    })
  }

  const handleMouseUp = (e: React.MouseEvent) => {
    if (!dragState) return

    // Buscar si soltamos el ratón sobre un puerto compatible
    const targetElement = e.target as SVGElement
    const portData = targetElement.getAttribute('data-port')
    
    if (portData) {
      const [toNodeId, toPortType, toPortName] = portData.split('|')
      
      // Reglas de conexión:
      // 1. Debe conectar una salida (output) con una entrada (input).
      // 2. Deben ser de ordenadores distintos.
      const isCompatible = 
        dragState.fromPortType !== toPortType &&
        dragState.fromNodeId !== toNodeId

      if (isCompatible) {
        const source_pc = dragState.fromPortType === 'output' ? dragState.fromNodeId : toNodeId
        const source_device = dragState.fromPortType === 'output' ? dragState.fromPortName : toPortName
        const dest_pc = dragState.fromPortType === 'input' ? dragState.fromNodeId : toNodeId
        const dest_device = dragState.fromPortType === 'input' ? dragState.fromPortName : toPortName

        // Evitar duplicados
        const exists = routes.some(
          r => r.source_pc === source_pc &&
               r.source_device === source_device &&
               r.dest_pc === dest_pc &&
               r.dest_device === dest_device
        )

        if (!exists) {
          // Nota: CPAL local actual solo soporta 1 stream de captura y 1 de playback.
          // Limpia las rutas previas que tengan la misma máquina como origen o destino para evitar fallos.
          const cleanRoutes = routes.filter(
            r => !(r.source_pc === source_pc || r.dest_pc === dest_pc)
          )

          setRoutes([...cleanRoutes, { source_pc, source_device, dest_pc, dest_device }])
        }
      }
    }

    setDragState(null)
  }

  const handleDeleteRoute = (index: number) => {
    const updated = [...routes]
    updated.splice(index, 1)
    setRoutes(updated)
    setHoveredRouteIndex(null)
  }

  // Genera el SVG Path Bezier entre dos puntos
  const getBezierPath = (x1: number, y1: number, x2: number, y2: number) => {
    const dx = Math.max(80, Math.abs(x2 - x1) / 2)
    return `M ${x1} ${y1} C ${x1 + dx} ${y1}, ${x2 - dx} ${y2}, ${x2} ${y2}`
  }

  // Determinar la altura máxima de la zona de dibujo
  const canvasHeight = Math.max(500, 100 + nodes.length * 150)

  return (
    <div className="audio-patchbay-container">
      <div className="patchbay-header">
        <div>
          <h3>Rutas de Audio Activas</h3>
          <p className="panel-subtitle">Conecta la salida (Micrófono) de una PC a la entrada (Altavoces) de otra PC.</p>
        </div>
        <div className="patchbay-actions">
          <button className="btn btn-secondary" onClick={onCancel}>
            Cancelar
          </button>
          <button className="btn btn-primary" onClick={() => onApply(routes)}>
            Aplicar Cambios
          </button>
        </div>
      </div>

      <div className="patchbay-canvas-wrapper">
        <svg
          ref={containerRef}
          width="100%"
          height={canvasHeight}
          className="patchbay-svg"
          onMouseMove={handleMouseMove}
          onMouseUp={handleMouseUp}
          onMouseLeave={() => setDragState(null)}
        >
          {/* Definiciones para sombras y efectos de brillo */}
          <defs>
            <linearGradient id="localGrad" x1="0%" y1="0%" x2="100%" y2="100%">
              <stop offset="0%" stopColor="#1e293b" />
              <stop offset="100%" stopColor="#0f172a" />
            </linearGradient>
            <linearGradient id="remoteGrad" x1="0%" y1="0%" x2="100%" y2="100%">
              <stop offset="0%" stopColor="#1e1b4b" />
              <stop offset="100%" stopColor="#0f172a" />
            </linearGradient>
            <filter id="glow" x="-20%" y="-20%" width="140%" height="140%">
              <feGaussianBlur stdDeviation="6" result="blur" />
              <feComposite in="SourceGraphic" in2="blur" operator="over" />
            </filter>
            <filter id="shadow" x="-10%" y="-10%" width="120%" height="120%">
              <feDropShadow dx="0" dy="8" stdDeviation="12" floodColor="#000000" floodOpacity="0.5" />
            </filter>
          </defs>

          {/* 1. Dibujar conexiones existentes */}
          {routes.map((route, idx) => {
            const srcNode = nodes.find(n => n.id === route.source_pc)
            const destNode = nodes.find(n => n.id === route.dest_pc)
            if (!srcNode || !destNode) return null

            const srcPortIdx = srcNode.outputs.indexOf(route.source_device)
            const destPortIdx = destNode.inputs.indexOf(route.dest_device)
            if (srcPortIdx === -1 || destPortIdx === -1) return null

            const p1 = getPortCoords(srcNode, 'output', srcPortIdx)
            const p2 = getPortCoords(destNode, 'input', destPortIdx)
            const path = getBezierPath(p1.x, p1.y, p2.x, p2.y)

            const isHovered = hoveredRouteIndex === idx

            return (
              <g
                key={idx}
                onMouseEnter={() => setHoveredRouteIndex(idx)}
                onMouseLeave={() => setHoveredRouteIndex(null)}
                style={{ cursor: 'pointer' }}
              >
                {/* Línea invisible más gruesa para facilitar el hover y clics */}
                <path
                  d={path}
                  fill="none"
                  stroke="transparent"
                  strokeWidth="20"
                  onClick={() => handleDeleteRoute(idx)}
                />
                {/* Línea principal brillante */}
                <path
                  d={path}
                  fill="none"
                  stroke={isHovered ? '#ef4444' : '#10b981'}
                  strokeWidth={isHovered ? '4' : '3'}
                  filter={isHovered ? 'url(#glow)' : 'none'}
                  className="audio-flow-line"
                  style={{
                    strokeDasharray: '6, 4',
                    animation: 'flow 1s linear infinite'
                  }}
                />
                {/* Botón flotante para eliminar ruta al hacer hover */}
                {isHovered && (
                  <g transform={`translate(${(p1.x + p2.x) / 2}, ${(p1.y + p2.y) / 2})`}>
                    <circle r="12" fill="#ef4444" filter="url(#glow)" />
                    <text
                      textAnchor="middle"
                      dy="4"
                      fill="#ffffff"
                      fontSize="14"
                      fontWeight="bold"
                      style={{ pointerEvents: 'none' }}
                      onClick={() => handleDeleteRoute(idx)}
                    >
                      ×
                    </text>
                  </g>
                )}
              </g>
            )
          })}

          {/* 2. Dibujar línea de arrastre temporal (si se está conectando un cable) */}
          {dragState && (
            <path
              d={getBezierPath(dragState.startX, dragState.startY, dragState.currentX, dragState.currentY)}
              fill="none"
              stroke="#6366f1"
              strokeWidth="3"
              strokeDasharray="5,5"
            />
          )}

          {/* 3. Dibujar Tarjetas de Nodos (PCs) */}
          {nodes.map(node => {
            const isLocal = node.id === 'local'
            return (
              <g key={node.id} transform={`translate(0, 0)`} filter="url(#shadow)">
                {/* Cuerpo del Card */}
                <rect
                  x={node.x}
                  y={node.y}
                  width={node.width}
                  height={node.height}
                  rx="12"
                  fill={isLocal ? 'url(#localGrad)' : 'url(#remoteGrad)'}
                  stroke={isLocal ? '#3b82f6' : '#6366f1'}
                  strokeWidth="1.5"
                />

                {/* Cabecera del Card */}
                <text
                  x={node.x + 15}
                  y={node.y + 35}
                  fill="#f8fafc"
                  fontSize="15"
                  fontWeight="bold"
                >
                  {node.name}
                </text>
                <line
                  x1={node.x}
                  y1={node.y + 48}
                  x2={node.x + node.width}
                  y2={node.y + 48}
                  stroke="#334155"
                  strokeWidth="1"
                />

                {/* Puertos de Entrada (Lado Izquierdo) - Altavoces, etc. */}
                {node.inputs.map((portName, idx) => {
                  const coords = getPortCoords(node, 'input', idx)
                  const isConnected = routes.some(r => r.dest_pc === node.id && r.dest_device === portName)
                  
                  return (
                    <g key={`in-${idx}`}>
                      {/* Texto del puerto */}
                      <text
                        x={coords.x + 20}
                        y={coords.y + 4}
                        fill="#94a3b8"
                        fontSize="12"
                        textAnchor="start"
                      >
                        {portName.length > 28 ? `${portName.substring(0, 26)}...` : portName}
                      </text>
                      {/* Círculo del puerto interactivo */}
                      <circle
                        cx={coords.x}
                        cy={coords.y}
                        r="8"
                        fill={isConnected ? '#10b981' : '#475569'}
                        stroke="#1e293b"
                        strokeWidth="2"
                        data-port={`${node.id}|input|${portName}`}
                        onMouseDown={(e) => handlePortMouseDown(e, node, idx, 'input', portName)}
                        style={{ cursor: 'crosshair' }}
                      />
                    </g>
                  )
                })}

                {/* Puertos de Salida (Lado Derecho) - Micrófonos, etc. */}
                {node.outputs.map((portName, idx) => {
                  const coords = getPortCoords(node, 'output', idx)
                  const isConnected = routes.some(r => r.source_pc === node.id && r.source_device === portName)

                  return (
                    <g key={`out-${idx}`}>
                      {/* Texto del puerto */}
                      <text
                        x={coords.x - 20}
                        y={coords.y + 4}
                        fill="#94a3b8"
                        fontSize="12"
                        textAnchor="end"
                      >
                        {portName.length > 28 ? `${portName.substring(0, 26)}...` : portName}
                      </text>
                      {/* Círculo del puerto interactivo */}
                      <circle
                        cx={coords.x}
                        cy={coords.y}
                        r="8"
                        fill={isConnected ? '#10b981' : '#475569'}
                        stroke="#1e293b"
                        strokeWidth="2"
                        data-port={`${node.id}|output|${portName}`}
                        onMouseDown={(e) => handlePortMouseDown(e, node, idx, 'output', portName)}
                        style={{ cursor: 'crosshair' }}
                      />
                    </g>
                  )
                })}
              </g>
            )
          })}
        </svg>
      </div>
    </div>
  )
}
