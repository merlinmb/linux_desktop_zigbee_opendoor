import { useEffect, useState } from 'react'
import { RoboEyes } from './components/RoboEyes'
import { ContactsList } from './components/ContactsList'
import { StatusBar } from './components/StatusBar'
import { SettingsModal } from './components/SettingsModal'
import { AppConfig, ContactStatus } from './lib/types'
import * as api from './lib/api'

function App() {
  const [config, setConfig] = useState<AppConfig | null>(null)
  const [contacts, setContacts] = useState<ContactStatus[]>([])
  const [showSettings, setShowSettings] = useState(false)
  const [mqttConnected, setMqttConnected] = useState(false)
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    const loadConfig = async () => {
      try {
        const loadedConfig = await api.configLoad()
        setConfig(loadedConfig)

        // Auto-connect to MQTT
        await api.mqttConnect(
          loadedConfig.mqtt.broker,
          loadedConfig.mqtt.port,
          loadedConfig.mqtt.client_name,
        )
        setMqttConnected(true)

        // Load contacts
        const allContacts = await api.contactsGetAll()
        setContacts(allContacts)
      } catch (err) {
        console.error('Failed to load config:', err)
      } finally {
        setLoading(false)
      }
    }

    loadConfig()
  }, [])

  useEffect(() => {
    const interval = setInterval(async () => {
      try {
        const allContacts = await api.contactsGetAll()
        setContacts(allContacts)
      } catch (err) {
        console.error('Failed to fetch contacts:', err)
      }
    }, 1000)

    return () => clearInterval(interval)
  }, [])

  const handleSettingsOpen = () => setShowSettings(true)
  const handleSettingsClose = () => setShowSettings(false)
  const handleSettingsSave = (newConfig: AppConfig) => {
    setConfig(newConfig)
  }

  if (loading || !config) {
    return <div className="app loading">Loading...</div>
  }

  const openContacts = contacts.filter((c) => !c.contact)
  const hasOpenContacts = openContacts.length > 0

  // Keyboard shortcut for settings (Ctrl+Comma)
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.ctrlKey && e.code === 'Comma') {
        setShowSettings((prev) => !prev)
      }
    }

    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [])

  return (
    <div className="app">
      <StatusBar mqttConnected={mqttConnected} broker={config.mqtt.broker} />

      <div className="main-content">
        {hasOpenContacts ? (
          <ContactsList contacts={openContacts} scrollInterval={config.display.scroll_interval_ms} />
        ) : (
          <RoboEyes />
        )}
      </div>

      <div className="toolbar">
        <button className="btn-settings" onClick={handleSettingsOpen} title="Settings (Ctrl+,)">
          ⚙️
        </button>
      </div>

      {showSettings && (
        <SettingsModal config={config} onSave={handleSettingsSave} onClose={handleSettingsClose} />
      )}
    </div>
  )
}

export default App
