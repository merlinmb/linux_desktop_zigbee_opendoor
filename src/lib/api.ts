import { invoke } from '@tauri-apps/api/tauri'
import { AppConfig, ContactStatus, MqttStatus } from './types'

export async function configLoad(): Promise<AppConfig> {
  return invoke('config_load')
}

export async function configSave(config: AppConfig): Promise<void> {
  return invoke('config_save', { newConfig: config })
}

export async function mqttConnect(
  broker: string,
  port: number,
  clientName: string,
  username?: string,
  password?: string,
): Promise<string> {
  return invoke('mqtt_connect', { broker, port, clientName, username, password })
}

export async function mqttDisconnect(): Promise<void> {
  return invoke('mqtt_disconnect')
}

export async function mqttStatus(): Promise<MqttStatus> {
  return invoke('mqtt_status')
}

export async function mqttSubscribe(topic: string, friendlyName: string): Promise<void> {
  return invoke('mqtt_subscribe', { topic, friendlyName })
}

export async function mqttUnsubscribe(topic: string): Promise<void> {
  return invoke('mqtt_unsubscribe', { topic })
}

export async function contactsGetAll(): Promise<ContactStatus[]> {
  return invoke('contacts_get_all')
}

export async function contactsCountOpen(): Promise<number> {
  return invoke('contacts_count_open')
}
