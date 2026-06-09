import { useEffect, useRef } from 'react'
import { usePipelineStore } from '../store/pipelineStore'

const AUTOSAVE_INTERVAL = 5000 // 5 seconds

/**
 * Auto-save pipeline to localStorage every 5 seconds if dirty
 * Also save on window unload to prevent data loss
 */
export function useAutoSave() {
  const { isDirty, pipeline, autoSaveToLocalStorage } = usePipelineStore()
  const intervalRef = useRef<NodeJS.Timeout | null>(null)

  // Set up auto-save interval
  useEffect(() => {
    // Clear existing interval
    if (intervalRef.current) {
      clearInterval(intervalRef.current)
    }

    // Create new interval that saves if dirty
    intervalRef.current = setInterval(() => {
      const state = usePipelineStore.getState()
      if (state.isDirty) {
        state.autoSaveToLocalStorage()
      }
    }, AUTOSAVE_INTERVAL)

    return () => {
      if (intervalRef.current) {
        clearInterval(intervalRef.current)
      }
    }
  }, [])

  // Save on window unload (e.g., closing browser/app)
  useEffect(() => {
    const handleBeforeUnload = () => {
      const state = usePipelineStore.getState()
      if (state.isDirty || state.pipeline.nodes.length > 0) {
        state.autoSaveToLocalStorage()
      }
    }

    window.addEventListener('beforeunload', handleBeforeUnload)
    return () => window.removeEventListener('beforeunload', handleBeforeUnload)
  }, [])

  // Save immediately when pipeline changes dramatically
  useEffect(() => {
    if (isDirty && pipeline.nodes.length > 0) {
      // Trigger auto-save for large changes
      const state = usePipelineStore.getState()
      if (pipeline.nodes.length % 10 === 0) {
        // Save every 10 nodes added/removed
        state.autoSaveToLocalStorage()
      }
    }
  }, [isDirty, pipeline.nodes.length])
}
