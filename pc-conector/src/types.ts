export interface AudioDevice {
  name: string
  device_type: 'Input' | 'Output' | 'Both'
  is_default: boolean
  channels: number
  sample_rates: number[]
}

export interface AppConfig {
  general: {
    auto_start: boolean
    auto_connect: boolean
    language: string
    theme: string
    minimize_to_tray: boolean
  }
  services: {
    clipboard_sync: boolean
    mouse_sharing: boolean
    keyboard_sharing: boolean
    audio_sharing: boolean
  }
  screens: ScreenInfo[]
  audio: {
    input_device: string | null
    output_device: string | null
    stream_microphone: boolean
    stream_speakers: boolean
    sample_rate: number
    bitrate: number
  }
  connection: {
    peer_address: string | null
    auto_reconnect: boolean
    reconnect_interval: number
    encryption_enabled: boolean
    security_token: string
    require_approval: boolean
    allowed_devices: string[]
  }
  linked_devices: LinkedDevice[]
}

export type Tab = 'dashboard' | 'screens' | 'services' | 'audio' | 'settings'

export interface DiscoveredDevice {
  ip: string
  mac: string
  hostname: string
  device_type: 'pc' | 'laptop' | 'mobile' | 'router' | 'tv' | 'printer' | 'unknown'
  brand: string
  description: string
}

export interface LinkedDevice {
  ip: string
  name: string
  linked_at: number
}

export interface ScreenInfo {
  id: string
  name: string
  x: number
  y: number
  width: number
  height: number
  is_primary: boolean
}
