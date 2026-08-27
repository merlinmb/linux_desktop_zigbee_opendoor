import React, { useEffect, useState } from 'react'

interface StatusBarProps {
  mqttConnected: boolean
  broker: string
  onOpenSettings: () => void
}

export const StatusBar: React.FC<StatusBarProps> = ({ mqttConnected, broker, onOpenSettings }) => {
  const [time, setTime] = useState(new Date())

  useEffect(() => {
    const timer = setInterval(() => setTime(new Date()), 1000)
    return () => clearInterval(timer)
  }, [])

  return (
    <div className="status-bar" data-tauri-drag-region>
      <div className="status-left" data-tauri-drag-region>
        <span
          className={`mqtt-indicator ${mqttConnected ? 'connected' : 'disconnected'}`}
          data-tauri-drag-region
        />
        <span className="broker-name" data-tauri-drag-region>
          {broker}
        </span>
        <span className="time" data-tauri-drag-region>
          {time.toLocaleTimeString()}
        </span>
      </div>
      <div className="status-right">
        <button className="btn-settings" onClick={onOpenSettings} title="Settings (Ctrl+,)">
          ⚙️
        </button>
      </div>
    </div>
  )
}
