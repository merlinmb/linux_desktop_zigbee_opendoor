import React, { useEffect, useState } from 'react'

interface StatusBarProps {
  mqttConnected: boolean
  broker: string
}

export const StatusBar: React.FC<StatusBarProps> = ({ mqttConnected, broker }) => {
  const [time, setTime] = useState(new Date())

  useEffect(() => {
    const timer = setInterval(() => setTime(new Date()), 1000)
    return () => clearInterval(timer)
  }, [])

  return (
    <div className="status-bar">
      <div className="status-left">
        <span className={`mqtt-indicator ${mqttConnected ? 'connected' : 'disconnected'}`} />
        <span className="broker-name">{broker}</span>
      </div>
      <div className="status-right">
        <span className="time">{time.toLocaleTimeString()}</span>
      </div>
    </div>
  )
}
