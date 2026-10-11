import React, { useEffect, useState } from 'react'

function formatTime(date: Date): string {
  const hours = String(date.getHours()).padStart(2, '0')
  const minutes = String(date.getMinutes()).padStart(2, '0')
  return `${hours}:${minutes}`
}

interface ClockProps {
  fontSize?: number
}

export const Clock: React.FC<ClockProps> = ({ fontSize = 156 }) => {
  const [now, setNow] = useState(new Date())

  useEffect(() => {
    const timer = setInterval(() => setNow(new Date()), 1000)
    return () => clearInterval(timer)
  }, [])

  return (
    <div className="clock" data-tauri-drag-region style={{ fontSize: `${fontSize}px` }}>
      {formatTime(now)}
    </div>
  )
}
