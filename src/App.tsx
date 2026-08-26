import { useEffect, useState } from 'react'
import { RoboEyes } from './components/RoboEyes'
import { ContactsList } from './components/ContactsList'
import { StatusBar } from './components/StatusBar'
import { SettingsModal } from './components/SettingsModal'
import { AppConfig } from './lib/types'
import { useConfig } from './hooks/useConfig'
import { useContacts } from './hooks/useContacts'
import { useMqtt } from './hooks/useMqtt'

function App() {
  const { config, loading: configLoading } = useConfig()
  const { contacts, openCount } = useContacts()
  const [showSettings, setShowSettings] = useState(false)
  const [mqttInitialized, setMqttInitialized] = useState(false)

  const mqtt = useMqtt(
    config?.mqtt.broker || 'localhost',
    config?.mqtt.port || 1883,
    config?.mqtt.client_name || 'opendoor_monitor',
  )

  // Auto-connect to MQTT when config is loaded
  useEffect(() => {
    if (config && !mqttInitialized && !mqtt.connected && !mqtt.connecting) {
      mqtt.connect()
      setMqttInitialized(true)
    }
  }, [config, mqttInitialized, mqtt])

  // Keyboard shortcut for settings (Ctrl+Comma)
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.ctrlKey && e.code === 'Comma') {
        e.preventDefault()
        setShowSettings((prev) => !prev)
      }
    }

    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [])

  const handleSettingsSave = async (newConfig: AppConfig) => {
    // Config hook will auto-save
  }

  if (configLoading || !config) {
    return (
      <div className="app loading">
        <div className="loader">Loading configuration...</div>
      </div>
    )
  }

  const hasOpenContacts = openCount > 0

  return (
    <div className="app">
      <StatusBar mqttConnected={mqtt.connected} broker={config.mqtt.broker} />

      <div className="main-content">
        {hasOpenContacts ? (
          <ContactsList
            contacts={contacts.filter((c) => !c.contact)}
            scrollInterval={config.display.scroll_interval_ms}
          />
        ) : (
          <RoboEyes />
        )}
      </div>

      <div className="toolbar">
        <button className="btn-settings" onClick={() => setShowSettings(true)} title="Settings (Ctrl+,)">
          ⚙️
        </button>
      </div>

      {showSettings && config && (
        <SettingsModal
          config={config}
          onSave={handleSettingsSave}
          onClose={() => setShowSettings(false)}
        />
      )}
    </div>
  )
}

export default App
