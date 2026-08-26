import { useEffect, useState } from 'react'
import { mqttConnect, mqttDisconnect } from '../lib/api'

export function useMqtt(broker: string, port: number, clientName: string) {
  const [connected, setConnected] = useState(false)
  const [connecting, setConnecting] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const connect = async () => {
    setConnecting(true)
    try {
      await mqttConnect(broker, port, clientName)
      setConnected(true)
      setError(null)
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err)
      setError(message)
      setConnected(false)
      console.error('MQTT connection failed:', err)
    } finally {
      setConnecting(false)
    }
  }

  const disconnect = async () => {
    try {
      await mqttDisconnect()
      setConnected(false)
      setError(null)
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err)
      setError(message)
      console.error('MQTT disconnection failed:', err)
    }
  }

  return { connected, connecting, error, connect, disconnect }
}
