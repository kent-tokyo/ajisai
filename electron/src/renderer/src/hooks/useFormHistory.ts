import { useCallback, useEffect, useState } from 'react'

export interface ConfigSnapshot {
  id: string
  type_name: string
  config: Record<string, unknown>
  timestamp: number
  label: string
}

const STORAGE_KEY = 'ajisai_form_history'
const MAX_HISTORY = 20

export function useFormHistory() {
  const [history, setHistory] = useState<ConfigSnapshot[]>(() => {
    const stored = localStorage.getItem(STORAGE_KEY)
    return stored ? JSON.parse(stored) : []
  })

  // Save history to localStorage whenever it changes
  useEffect(() => {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(history.slice(0, MAX_HISTORY)))
  }, [history])

  const addSnapshot = useCallback(
    (type_name: string, config: Record<string, unknown>, label?: string) => {
      const snapshot: ConfigSnapshot = {
        id: `${Date.now()}-${Math.random()}`,
        type_name,
        config,
        timestamp: Date.now(),
        label: label || new Date().toLocaleTimeString(),
      }

      setHistory((prev) => [snapshot, ...prev].slice(0, MAX_HISTORY))
    },
    []
  )

  const getHistoryFor = useCallback(
    (type_name: string) => history.filter((item) => item.type_name === type_name),
    [history]
  )

  const clearHistory = useCallback(() => {
    setHistory([])
  }, [])

  const removeSnapshot = useCallback((id: string) => {
    setHistory((prev) => prev.filter((item) => item.id !== id))
  }, [])

  return {
    history,
    addSnapshot,
    getHistoryFor,
    clearHistory,
    removeSnapshot,
  }
}
