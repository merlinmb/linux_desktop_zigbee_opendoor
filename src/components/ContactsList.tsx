import React, { useEffect, useState } from 'react'
import { ContactStatus } from '../lib/types'

interface ContactsListProps {
  contacts: ContactStatus[]
  scrollInterval: number
  contactNameFontSize?: number
  highlightDurationMs?: number
}

export const ContactsList: React.FC<ContactsListProps> = ({
  contacts,
  scrollInterval,
  contactNameFontSize = 24,
  highlightDurationMs = 3000
}) => {
  const [scrollOffset, setScrollOffset] = useState(0)
  const [lastScroll, setLastScroll] = useState(Date.now())
  const [highlightedTopic, setHighlightedTopic] = useState<string | null>(null)
  // Start at 0 (not contacts.length) so the door that causes this list to first
  // mount (Clock -> ContactsList, i.e. 0 open -> 1 open) is still detected as new.
  const [previousContactCount, setPreviousContactCount] = useState(0)

  // Detect newly opened doors and highlight them
  useEffect(() => {
    if (contacts.length > previousContactCount) {
      // A new door opened - highlight the first newly opened one (should be the most recent)
      const newTopic = contacts[0]?.topic
      if (newTopic) {
        setHighlightedTopic(newTopic)
        setScrollOffset(0) // Jump to the newly opened door
        setPreviousContactCount(contacts.length)

        const timeout = setTimeout(() => {
          setHighlightedTopic(null)
        }, highlightDurationMs)

        return () => clearTimeout(timeout)
      }
    }
    setPreviousContactCount(contacts.length)
    return undefined
  }, [contacts.length, previousContactCount, highlightDurationMs])

  // Auto-scroll through contacts (but not while a door is highlighted)
  useEffect(() => {
    if (contacts.length === 0 || highlightedTopic !== null) return

    const interval = setInterval(() => {
      const now = Date.now()
      if (now - lastScroll >= scrollInterval) {
        setScrollOffset((prev) => (prev + 1) % contacts.length)
        setLastScroll(now)
      }
    }, 100)

    return () => clearInterval(interval)
  }, [contacts, scrollInterval, lastScroll, highlightedTopic])

  const openCount = contacts.length
  const current = contacts.length > 0 ? contacts[scrollOffset % contacts.length] : null
  const isHighlighted = highlightedTopic && current?.topic === highlightedTopic

  return (
    <div className="contacts-list">
      <div className="contacts-header">
        <span className="open-count">{openCount} Open</span>
      </div>

      <div className="contacts-items">
        {current && (
          <div
            className={`contact-item ${isHighlighted ? 'highlighted' : ''}`}
            style={isHighlighted ? { '--highlight-duration': `${highlightDurationMs}ms` } as React.CSSProperties : undefined}
          >
            <span className="contact-name" style={{ fontSize: `${contactNameFontSize}px` }}>
              {current.friendly_name}
            </span>
            <span className="last-seen">
              {current.last_seen ? new Date(current.last_seen).toLocaleTimeString() : 'N/A'}
            </span>
          </div>
        )}
      </div>
    </div>
  )
}
