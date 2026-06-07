import { ScreensIcon, InfoIcon } from './Icons'

interface ScreenProps {
  screens: {
    id: string
    name: string
    x: number
    y: number
    width: number
    height: number
    is_primary: boolean
  }[]
  onUpdate: (screens: ScreenProps['screens']) => void
}

export default function ScreenArrangement({ screens }: ScreenProps) {
  return (
    <div className="panel">
      <h2>Configuración de Pantallas</h2>
      <p className="panel-subtitle">Visualiza la disposición y el tamaño relativo de tus pantallas</p>
      
      <div className="screen-canvas">
        {screens.length === 0 ? (
          <div className="empty-state">
            <div className="empty-icon">
              <ScreensIcon size={28} />
            </div>
            <p>No hay pantallas configuradas. Conecta un monitor adicional.</p>
          </div>
        ) : (
          screens.map((screen) => {
            const widthPx = Math.max(150, screen.width / 8)
            const heightPx = Math.max(100, screen.height / 8)
            
            return (
              <div
                key={screen.id}
                className="screen-item"
                style={{
                  left: `calc(50% + ${screen.x / 10}px - ${widthPx / 2}px)`,
                  top: `calc(50% + ${screen.y / 10}px - ${heightPx / 2}px)`,
                  width: `${widthPx}px`,
                  height: `${heightPx}px`
                }}
              >
                <ScreensIcon size={24} style={{ opacity: 0.8, marginBottom: '6px' }} />
                <span>{screen.name}</span>
                <span style={{ fontSize: '10px', opacity: 0.6, marginTop: '2px' }}>
                  {screen.width}x{screen.height}
                </span>
                {screen.is_primary && (
                  <span className="primary-badge">Principal</span>
                )}
              </div>
            )
          })
        )}
      </div>

      <div className="screen-actions">
        <div className="hint">
          <InfoIcon size={16} />
          <span>La configuración de pantallas se detecta automáticamente al conectar.</span>
        </div>
      </div>
    </div>
  )
}
