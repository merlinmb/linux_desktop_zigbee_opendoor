import React, { useEffect, useState } from 'react'
import { ContactStatus } from '../lib/types'

interface ContactsListProps {
  contacts: ContactStatus[]
  scrollInterval: number
}

export const ContactsList: React.FC<ContactsListProps> = ({ contacts, scrollInterval }) => {
  const [scrollOffset, setScrollOffset] = useState(0)
  const [lastScroll, setLastScroll] = useState(Date.now())

  useEffect(() => {
    if (contacts.length === 0) return

    const interval = setInterval(() => {
      const now = Date.now()
      if (now - lastScroll >= scrollInterval) {
        setScrollOffset((prev) => (prev + 1) % contacts.length)
        setLastScroll(now)
      }
    }, 100)

    return () => clearInterval(interval)
  }, [contacts, scrollInterval, lastScroll])

  const openCount = contacts.length
  const current = contacts.length > 0 ? contacts[scrollOffset % contacts.length] : null

  return (
    <div className="contacts-list">
      <div className="contacts-header">
        <span className="open-count">{openCount} Open</span>
      </div>

      <div className="contacts-items">
        {current && (
          <div className="contact-item">
            <span className="contact-name">{current.friendly_name}</span>
            <span className="last-seen">
              {current.last_seen ? new Date(current.last_seen).toLocaleTimeString() : 'N/A'}
            </span>
          </div>
        )}
      </div>
    </div>
  )
}
