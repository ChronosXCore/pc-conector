import type { AudioDevice } from './types'
import { AudioIcon } from './Icons'

interface AudioPanelProps {
  audio: {
    input_device: string | null
    output_device: string | null
    stream_microphone: boolean
    stream_speakers: boolean
    sample_rate: number
    bitrate: number
  }
  inputs: AudioDevice[]
  outputs: AudioDevice[]
  onUpdate: (audio: AudioPanelProps['audio']) => void
}

export default function AudioPanel({ audio, inputs, outputs, onUpdate }: AudioPanelProps) {
  return (
    <div className="panel">
      <h2>Audio</h2>
      <p className="panel-subtitle">Configura los dispositivos y la calidad de transmisión de audio</p>
      
      <div className="audio-section">
        <h3>
          <svg
            xmlns="http://www.w3.org/2000/svg"
            width="18"
            height="18"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="2.2"
            strokeLinecap="round"
            strokeLinejoin="round"
          >
            <path d="M12 2a3 3 0 0 0-3 3v7a3 3 0 0 0 6 0V5a3 3 0 0 0-3-3Z" />
            <path d="M19 10v2a7 7 0 0 1-14 0v-2" />
            <line x1="12" x2="12" y1="19" y2="22" />
          </svg>
          Micrófono
        </h3>
        <div
          className="toggle-row"
          onClick={() => onUpdate({ ...audio, stream_microphone: !audio.stream_microphone })}
        >
          <span>Transmitir micrófono</span>
          <div className={`toggle ${audio.stream_microphone ? 'on' : 'off'}`}>
            <div className="toggle-knob" />
          </div>
        </div>
        
        {audio.stream_microphone && (
          <div className="select-group">
            <label>Dispositivo de entrada</label>
            <select
              value={audio.input_device || ''}
              onChange={(e) => onUpdate({ ...audio, input_device: e.target.value || null })}
            >
              <option value="">Predeterminado del sistema</option>
              {inputs.map((d) => (
                <option key={d.name} value={d.name}>
                  {d.name}
                </option>
              ))}
            </select>
          </div>
        )}
      </div>

      <div className="audio-section">
        <h3>
          <svg
            xmlns="http://www.w3.org/2000/svg"
            width="18"
            height="18"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="2.2"
            strokeLinecap="round"
            strokeLinejoin="round"
          >
            <polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5" />
            <path d="M15.54 8.46a5 5 0 0 1 0 7.07" />
            <path d="M19.07 4.93a10 10 0 0 1 0 14.14" />
          </svg>
          Altavoces
        </h3>
        <div
          className="toggle-row"
          onClick={() => onUpdate({ ...audio, stream_speakers: !audio.stream_speakers })}
        >
          <span>Transmitir altavoces</span>
          <div className={`toggle ${audio.stream_speakers ? 'on' : 'off'}`}>
            <div className="toggle-knob" />
          </div>
        </div>
        
        {audio.stream_speakers && (
          <div className="select-group">
            <label>Dispositivo de salida</label>
            <select
              value={audio.output_device || ''}
              onChange={(e) => onUpdate({ ...audio, output_device: e.target.value || null })}
            >
              <option value="">Predeterminado del sistema</option>
              {outputs.map((d) => (
                <option key={d.name} value={d.name}>
                  {d.name}
                </option>
              ))}
            </select>
          </div>
        )}
      </div>

      <div className="audio-section">
        <h3>
          <AudioIcon size={18} />
          Calidad de Audio
        </h3>
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
        <div className="select-group" style={{ marginTop: '20px' }}>
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
  )
}
