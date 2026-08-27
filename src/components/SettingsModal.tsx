import React, { useState } from 'react'
import { AppConfig } from '../lib/types'

interface SettingsModalProps {
  config: AppConfig
  onSave: (config: AppConfig) => Promise<void>
  onClose: () => void
}

export const SettingsModal: React.FC<SettingsModalProps> = ({ config, onSave, onClose }) => {
  const [localConfig, setLocalConfig] = useState<AppConfig>(config)
  const [newTopic, setNewTopic] = useState('')
  const [newFriendly, setNewFriendly] = useState('')

  const handleSave = async () => {
    await onSave(localConfig)
    onClose()
  }

  const handleAddTopic = () => {
    if (newTopic && newFriendly) {
      setLocalConfig({
        ...localConfig,
        subscriptions: {
          ...localConfig.subscriptions,
          [newTopic]: newFriendly,
        },
      })
      setNewTopic('')
      setNewFriendly('')
    }
  }

  const handleRemoveTopic = (topic: string) => {
    const { [topic]: _, ...rest } = localConfig.subscriptions
    setLocalConfig({
      ...localConfig,
      subscriptions: rest,
    })
  }

  return (
    <div className="settings-modal-overlay">
      <div className="settings-modal">
        <h2>Settings</h2>

        <div className="settings-section">
          <h3>MQTT Broker</h3>
          <input
            type="text"
            placeholder="Broker IP/Hostname"
            value={localConfig.mqtt.broker}
            onChange={(e) =>
              setLocalConfig({
                ...localConfig,
                mqtt: { ...localConfig.mqtt, broker: e.target.value },
              })
            }
          />
          <input
            type="number"
            placeholder="Port"
            value={localConfig.mqtt.port}
            onChange={(e) =>
              setLocalConfig({
                ...localConfig,
                mqtt: { ...localConfig.mqtt, port: parseInt(e.target.value) },
              })
            }
          />
          <input
            type="text"
            placeholder="Client Name"
            value={localConfig.mqtt.client_name}
            onChange={(e) =>
              setLocalConfig({
                ...localConfig,
                mqtt: { ...localConfig.mqtt, client_name: e.target.value },
              })
            }
          />
          <input
            type="text"
            placeholder="Username (optional)"
            value={localConfig.mqtt.username || ''}
            onChange={(e) =>
              setLocalConfig({
                ...localConfig,
                mqtt: { ...localConfig.mqtt, username: e.target.value || undefined },
              })
            }
          />
          <input
            type="password"
            placeholder="Password (optional)"
            value={localConfig.mqtt.password || ''}
            onChange={(e) =>
              setLocalConfig({
                ...localConfig,
                mqtt: { ...localConfig.mqtt, password: e.target.value || undefined },
              })
            }
          />
        </div>

        <div className="settings-section">
          <h3>Display</h3>
          <label>
            Brightness:
            <input
              type="range"
              min="0"
              max="255"
              value={localConfig.display.brightness}
              onChange={(e) =>
                setLocalConfig({
                  ...localConfig,
                  display: { ...localConfig.display, brightness: parseInt(e.target.value) },
                })
              }
            />
            {localConfig.display.brightness}
          </label>
          <label>
            Clock Font Size (px):
            <input
              type="range"
              min="12"
              max="300"
              value={localConfig.display.clock_font_size}
              onChange={(e) =>
                setLocalConfig({
                  ...localConfig,
                  display: { ...localConfig.display, clock_font_size: parseInt(e.target.value) },
                })
              }
            />
            {localConfig.display.clock_font_size}
          </label>
          <label>
            Contact Name Font Size (px):
            <input
              type="range"
              min="8"
              max="100"
              value={localConfig.display.contact_name_font_size}
              onChange={(e) =>
                setLocalConfig({
                  ...localConfig,
                  display: { ...localConfig.display, contact_name_font_size: parseInt(e.target.value) },
                })
              }
            />
            {localConfig.display.contact_name_font_size}
          </label>
          <label>
            Scroll Interval (ms):
            <input
              type="number"
              value={localConfig.display.scroll_interval_ms}
              onChange={(e) =>
                setLocalConfig({
                  ...localConfig,
                  display: { ...localConfig.display, scroll_interval_ms: parseInt(e.target.value) },
                })
              }
            />
          </label>
          <label>
            Highlight Duration (ms):
            <input
              type="range"
              min="500"
              max="10000"
              step="100"
              value={localConfig.display.highlight_duration_ms}
              onChange={(e) =>
                setLocalConfig({
                  ...localConfig,
                  display: { ...localConfig.display, highlight_duration_ms: parseInt(e.target.value) },
                })
              }
            />
            {localConfig.display.highlight_duration_ms}
          </label>
          <label>
            <input
              type="checkbox"
              checked={localConfig.display.flip_screen}
              onChange={(e) =>
                setLocalConfig({
                  ...localConfig,
                  display: { ...localConfig.display, flip_screen: e.target.checked },
                })
              }
            />
            Flip Screen
          </label>
        </div>

        <div className="settings-section">
          <h3>Subscriptions</h3>
          <div className="add-topic">
            <input
              type="text"
              placeholder="Topic"
              value={newTopic}
              onChange={(e) => setNewTopic(e.target.value)}
            />
            <input
              type="text"
              placeholder="Friendly Name"
              value={newFriendly}
              onChange={(e) => setNewFriendly(e.target.value)}
            />
            <button onClick={handleAddTopic}>Add</button>
          </div>

          <div className="topics-list">
            {Object.entries(localConfig.subscriptions).map(([topic, friendly]) => (
              <div key={topic} className="topic-item">
                <span>{friendly}</span>
                <span className="topic">{topic}</span>
                <button onClick={() => handleRemoveTopic(topic)}>×</button>
              </div>
            ))}
          </div>
        </div>

        <div className="settings-actions">
          <button onClick={handleSave} className="btn-save">
            Save
          </button>
          <button onClick={onClose} className="btn-cancel">
            Cancel
          </button>
        </div>
      </div>
    </div>
  )
}
