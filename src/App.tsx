import { useEffect, useState } from 'react'
import { Clock } from './components/Clock'
import { ContactsList } from './components/ContactsList'
import { StatusBar } from './components/StatusBar'
import { SettingsModal } from './components/SettingsModal'
import { AppConfig } from './lib/types'
import { mqttSubscribe, mqttUnsubscribe } from './lib/api'
import { useConfig } from './hooks/useConfig'
import { useContacts } from './hooks/useContacts'
import { useMqtt } from './hooks/useMqtt'

function App() {
  const { config, loading: configLoading, saveConfig } = useConfig()
  const { contacts, openCount } = useContacts()
  const [showSettings, setShowSettings] = useState(false)
  const [mqttInitialized, setMqttInitialized] = useState(false)
  const [subscribed, setSubscribed] = useState(false)

  const mqtt = useMqtt(
    config?.mqtt.broker || 'localhost',
    config?.mqtt.port || 1883,
    config?.mqtt.client_name || 'opendoor_monitor',
    config?.mqtt.username,
    config?.mqtt.password,
  )

  // Auto-connect to MQTT when config is loaded
  useEffect(() => {
    if (config && !mqttInitialized && !mqtt.connected && !mqtt.connecting) {
      mqtt.connect()
      setMqttInitialized(true)
    }
  }, [config, mqttInitialized, mqtt])

  // Subscribe to configured topics once connected
  useEffect(() => {
    if (mqtt.connected && config && !subscribed) {
      Object.entries(config.subscriptions).forEach(([topic, friendlyName]) => {
        mqttSubscribe(topic, friendlyName).catch((err) => {
          console.error(`Failed to subscribe to ${topic}:`, err)
        })
      })
      setSubscribed(true)
    }
  }, [mqtt.connected, config, subscribed])

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
    const oldSubscriptions = config?.subscriptions ?? {}
    const saved = await saveConfig(newConfig)
    if (!saved || !mqtt.connected) return

    for (const [topic, friendlyName] of Object.entries(newConfig.subscriptions)) {
      if (oldSubscriptions[topic] !== friendlyName) {
        try {
          await mqttSubscribe(topic, friendlyName)
        } catch (err) {
          console.error(`Failed to subscribe to ${topic}:`, err)
        }
      }
    }

    for (const topic of Object.keys(oldSubscriptions)) {
      if (!(topic in newConfig.subscriptions)) {
        try {
          await mqttUnsubscribe(topic)
        } catch (err) {
          console.error(`Failed to unsubscribe from ${topic}:`, err)
        }
      }
    }
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
      <StatusBar
        mqttConnected={mqtt.connected}
        broker={config.mqtt.broker}
        onOpenSettings={() => setShowSettings(true)}
      />

      <div className="main-content" data-tauri-drag-region>
        {hasOpenContacts ? (
          <ContactsList
            contacts={contacts.filter((c) => !c.contact)}
            scrollInterval={config.display.scroll_interval_ms}
            contactNameFontSize={config.display.contact_name_font_size}
            highlightDurationMs={config.display.highlight_duration_ms}
          />
        ) : (
          <Clock fontSize={config.display.clock_font_size} />
        )}
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
