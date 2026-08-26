import { useEffect, useState, useCallback } from 'react'
import { configLoad, configSave } from '../lib/api'
import { AppConfig } from '../lib/types'

export function useConfig() {
  const [config, setConfig] = useState<AppConfig | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  const loadConfig = useCallback(async () => {
    try {
      const loaded = await configLoad()
      setConfig(loaded)
      setError(null)
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err)
      setError(message)
      console.error('Failed to load config:', err)
    } finally {
      setLoading(false)
    }
  }, [])

  const saveConfig = useCallback(
    async (newConfig: AppConfig) => {
      try {
        await configSave(newConfig)
        setConfig(newConfig)
        setError(null)
        return true
      } catch (err) {
        const message = err instanceof Error ? err.message : String(err)
        setError(message)
        console.error('Failed to save config:', err)
        return false
      }
    },
    [],
  )

  useEffect(() => {
    loadConfig()
  }, [loadConfig])

  return { config, loading, error, saveConfig, reload: loadConfig }
}
