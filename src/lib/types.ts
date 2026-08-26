export interface ContactStatus {
  topic: string
  friendly_name: string
  contact: boolean
  last_seen?: string
  battery?: number
  payload: string
}

export interface MqttConfig {
  broker: string
  port: number
  client_name: string
}

export interface DisplayConfig {
  brightness: number
  flip_screen: boolean
  scroll_interval_ms: number
}

export interface WindowConfig {
  x: number
  y: number
  always_on_top: boolean
  transparency: number
}

export interface AppConfig {
  mqtt: MqttConfig
  display: DisplayConfig
  window: WindowConfig
  subscriptions: Record<string, string>
}

export interface MqttStatus {
  connected: boolean
  broker: string
  client_name: string
}
