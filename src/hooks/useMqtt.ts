import { useEffect, useState } from 'react'
import { mqttConnect, mqttDisconnect, mqttStatus } from '../lib/api'

export function useMqtt(broker: string, port: number, clientName: string, username?: string, password?: string) {
  const [connected, setConnected] = useState(false)
  const [connecting, setConnecting] = useState(false)
  const [error, setError] = useState<string | null>(null)

  // The backend's connection can drop and reconnect on its own (network
  // blips, broker restarts). Poll the real status instead of trusting the
  // one-time optimistic flag set by connect(), so the UI doesn't keep
  // claiming "connected" after a link that has actually gone down.
  useEffect(() => {
    const interval = setInterval(async () => {
      try {
        const status = await mqttStatus()
        setConnected(status.connected)
      } catch (err) {
        console.error('Failed to fetch MQTT status:', err)
      }
    }, 1000)
    return () => clearInterval(interval)
  }, [])

  const connect = async () => {
    setConnecting(true)
    try {
      // This resolves once the connection attempt has been kicked off in the
      // backend, not once the broker handshake actually completes — the
      // status poll above is what flips `connected` once it's real.
      await mqttConnect(broker, port, clientName, username, password)
      setError(null)
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err)
      setError(message)
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
