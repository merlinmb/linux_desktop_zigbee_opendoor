import { useEffect, useState, useCallback } from 'react'
import { contactsGetAll } from '../lib/api'

export function useContacts() {
  const [contacts, setContacts] = useState<any[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  const fetchContacts = useCallback(async () => {
    try {
      const all = await contactsGetAll()
      setContacts(all)
      setError(null)
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err)
      setError(message)
      console.error('Failed to fetch contacts:', err)
    } finally {
      setLoading(false)
    }
  }, [])

  useEffect(() => {
    fetchContacts()
    const interval = setInterval(fetchContacts, 1000)
    return () => clearInterval(interval)
  }, [fetchContacts])

  const openCount = contacts.filter((c) => !c.contact).length

  return { contacts, openCount, loading, error, refetch: fetchContacts }
}
