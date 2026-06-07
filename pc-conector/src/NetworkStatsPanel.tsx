import { useState, useEffect, useRef, useCallback } from 'react'
import { invoke } from '@tauri-apps/api/core'
import {
  SignalIcon,
  MonitorIcon,
  TargetIcon,
  ChartIcon,
  StatusDotIcon,
  ClipboardIcon,
  CheckIcon
} from './Icons'

interface PingResult {
  host: string
  latency_ms: number | null
  success: boolean
  error: string | null
}

interface LocalInfo {
  hostname: string
  ips: string[]
}

const MAX_HISTORY = 24
const PING_INTERVAL_MS = 3000

function qualityLabel(ms: number | null): { label: string; color: string } {
  if (ms === null) return { label: 'Sin respuesta', color: '#f43f5e' }
  if (ms < 20)   return { label: 'Excelente', color: '#10b981' }
  if (ms < 60)   return { label: 'Bueno',     color: '#22c55e' }
  if (ms < 120)  return { label: 'Regular',   color: '#f59e0b' }
  if (ms < 250)  return { label: 'Lento',     color: '#f97316' }
  return             { label: 'Muy lento',  color: '#f43f5e' }
}

function barHeight(ms: number | null, maxMs: number): number {
  if (ms === null) return 4
  const capped = Math.min(ms, maxMs)
  return Math.max(4, (capped / maxMs) * 100)
}

export default function NetworkStatsPanel() {
  const [isOpen, setIsOpen] = useState(false)
  const [localInfo, setLocalInfo] = useState<LocalInfo | null>(null)
  const [pingTarget, setPingTarget] = useState('8.8.8.8')
  const [pingInput, setPingInput] = useState('8.8.8.8')
  const [history, setHistory] = useState<(number | null)[]>([])
  const [isPinging, setIsPinging] = useState(false)
  const [copied, setCopied] = useState(false)
  const intervalRef = useRef<ReturnType<typeof setInterval> | null>(null)

  // Load local IPs
  useEffect(() => {
    invoke<LocalInfo>('get_local_ips').then(setLocalInfo).catch(console.error)
  }, [])

  const doPing = useCallback(async () => {
    if (isPinging) return
    setIsPinging(true)
    try {
      const result = await invoke<PingResult>('ping_host', { host: pingTarget })
      setHistory(prev => {
        const next = [...prev, result.latency_ms]
        return next.slice(-MAX_HISTORY)
      })
    } catch {
      setHistory(prev => [...prev, null].slice(-MAX_HISTORY))
    } finally {
      setIsPinging(false)
    }
  }, [pingTarget, isPinging])

  // Start/stop ping loop when panel is open
  useEffect(() => {
    if (isOpen) {
      doPing()
      intervalRef.current = setInterval(doPing, PING_INTERVAL_MS)
    } else {
      if (intervalRef.current) clearInterval(intervalRef.current)
      intervalRef.current = null
    }
    return () => { if (intervalRef.current) clearInterval(intervalRef.current) }
  }, [isOpen, pingTarget]) // eslint-disable-line react-hooks/exhaustive-deps

  const validSamples = history.filter((v): v is number => v !== null)
  const currentMs = history.length > 0 ? history[history.length - 1] : null
  const avgMs = validSamples.length ? validSamples.reduce((a, b) => a + b, 0) / validSamples.length : null
  const minMs = validSamples.length ? Math.min(...validSamples) : null
  const maxMs = validSamples.length ? Math.max(...validSamples) : null
  const chartMax = Math.max(200, maxMs ?? 200)
  const quality = qualityLabel(currentMs)

  const handleCopyIp = (ip: string) => {
    navigator.clipboard.writeText(ip).then(() => {
      setCopied(true)
      setTimeout(() => setCopied(false), 1500)
    })
  }

  const handleApplyTarget = () => {
    setHistory([])
    setPingTarget(pingInput)
  }

  return (
    <div className="net-stats-panel">
      {/* Header / trigger */}
      <div className="net-stats-header" onClick={() => setIsOpen(o => !o)}>
        <div className="net-stats-title">
          <span className="net-stats-icon" style={{ display: 'flex', alignItems: 'center' }}>
            <SignalIcon size={20} color="var(--accent)" />
          </span>
          <div>
            <span className="net-stats-label">Red & Rendimiento</span>
            {localInfo && localInfo.ips.length > 0 && (
              <span className="net-stats-ip-preview">{localInfo.ips[0]}</span>
            )}
          </div>
        </div>
        <div className="net-stats-right">
          {currentMs !== null && (
            <span className="net-stats-badge" style={{ color: quality.color, borderColor: quality.color }}>
              {currentMs.toFixed(0)} ms
            </span>
          )}
          <span className={`net-stats-chevron ${isOpen ? 'open' : ''}`}>▾</span>
        </div>
      </div>

      {/* Expanded body */}
      {isOpen && (
        <div className="net-stats-body">
          {/* Local IP section */}
          {localInfo && (
            <div className="net-stats-section">
              <p className="net-stats-section-title" style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
                <MonitorIcon size={14} style={{ color: 'var(--text-muted)' }} /> Esta máquina
              </p>
              <div className="net-stats-machine">
                <div className="net-stats-hostname">{localInfo.hostname}</div>
                {localInfo.ips.map((ip, i) => (
                  <div key={i} className="net-stats-ip-row">
                    <code className="net-stats-ip-code">{ip}</code>
                    <button
                      className="net-stats-copy-btn"
                      onClick={() => handleCopyIp(ip)}
                      title="Copiar IP"
                      style={{ display: 'flex', alignItems: 'center', gap: '6px' }}
                    >
                      {copied ? (
                        <>
                          <CheckIcon size={12} /> Copiada
                        </>
                      ) : (
                        <>
                          <ClipboardIcon size={12} /> Copiar
                        </>
                      )}
                    </button>
                  </div>
                ))}
                <p className="net-stats-hint">
                  Usa esta IP en el otro dispositivo para conectarte directamente.
                </p>
              </div>
            </div>
          )}

          {/* Ping target input */}
          <div className="net-stats-section">
            <p className="net-stats-section-title" style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
              <TargetIcon size={14} style={{ color: 'var(--text-muted)' }} /> Destino del ping
            </p>
            <div className="net-stats-target-row">
              <input
                className="net-stats-input"
                value={pingInput}
                onChange={e => setPingInput(e.target.value)}
                placeholder="IP o dominio"
                onKeyDown={e => e.key === 'Enter' && handleApplyTarget()}
              />
              <button className="net-stats-apply-btn" onClick={handleApplyTarget}>
                Aplicar
              </button>
            </div>
          </div>

          {/* Live chart */}
          <div className="net-stats-section">
            <div className="net-stats-chart-header" style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
              <p className="net-stats-section-title" style={{ display: 'flex', alignItems: 'center', gap: '6px', margin: 0 }}>
                <ChartIcon size={14} style={{ color: 'var(--text-muted)' }} /> Latencia en vivo
              </p>
              <span className="net-stats-quality-badge" style={{ color: quality.color, display: 'flex', alignItems: 'center', gap: '6px' }}>
                <StatusDotIcon size={8} color={quality.color} /> {quality.label}
              </span>
            </div>

            {history.length === 0 ? (
              <div className="net-stats-chart-empty">
                <div className="net-stats-spinner" />
                Midiendo...
              </div>
            ) : (
              <div className="net-stats-chart">
                {Array.from({ length: MAX_HISTORY }).map((_, i) => {
                  const idx = i - (MAX_HISTORY - history.length)
                  const val = idx >= 0 ? history[idx] : undefined
                  const h = val !== undefined ? barHeight(val, chartMax) : 0
                  const col = val === null ? '#f43f5e' : qualityLabel(val).color
                  return (
                    <div key={i} className="net-stats-bar-wrap">
                      <div
                        className="net-stats-bar"
                        style={{ height: `${h}%`, background: col, opacity: val !== undefined ? 1 : 0.1 }}
                        title={val !== null && val !== undefined ? `${val.toFixed(1)} ms` : 'Sin respuesta'}
                      />
                    </div>
                  )
                })}
              </div>
            )}

            {/* Stats row */}
            <div className="net-stats-stats">
              <div className="net-stats-stat">
                <span className="net-stats-stat-label">Actual</span>
                <span className="net-stats-stat-val" style={{ color: quality.color }}>
                  {currentMs !== null ? `${currentMs.toFixed(0)} ms` : '—'}
                </span>
              </div>
              <div className="net-stats-stat">
                <span className="net-stats-stat-label">Promedio</span>
                <span className="net-stats-stat-val">
                  {avgMs !== null ? `${avgMs.toFixed(0)} ms` : '—'}
                </span>
              </div>
              <div className="net-stats-stat">
                <span className="net-stats-stat-label">Mínimo</span>
                <span className="net-stats-stat-val" style={{ color: '#10b981' }}>
                  {minMs !== null ? `${minMs.toFixed(0)} ms` : '—'}
                </span>
              </div>
              <div className="net-stats-stat">
                <span className="net-stats-stat-label">Máximo</span>
                <span className="net-stats-stat-val" style={{ color: '#f43f5e' }}>
                  {maxMs !== null ? `${maxMs.toFixed(0)} ms` : '—'}
                </span>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}

