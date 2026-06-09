import { useEffect } from 'react'
import { usePipelineStore } from './store/pipelineStore'
import { MenuBar } from './components/MenuBar'
import { Layout } from './components/Layout'
import type { PipelineState } from './types/pipeline'
import './App.css'

export default function App() {
  const {
    pipeline,
    setCategories,
    setPipeline,
    isRunning,
    setRunning,
    appendLog,
    recordNodeMetrics,
    resetMetrics,
    finalizeMetrics,
  } = usePipelineStore()

  useEffect(() => {
    const init = async () => {
      try {
        // Load saved pipeline from localStorage
        const saved = localStorage.getItem('ajisai_pipeline')
        if (saved) {
          try {
            const savedPipeline = JSON.parse(saved)
            setPipeline(savedPipeline)
          } catch (e) {
            console.error('Failed to restore pipeline:', e)
          }
        }

        // Connect to server
        await window.ajisai.ping()
        const transforms = await window.ajisai.getTransforms()
        setCategories(transforms.categories)
      } catch (err) {
        console.error('Failed to initialize app:', err)
      }
    }

    init()

    // Subscribe to progress notifications
    const unsubProgress = window.ajisai.onPipelineProgress?.((params) => {
      appendLog(`[${params.status}] ${params.node_id}`)
      // Record node metrics for dashboard
      recordNodeMetrics(params.node_id, {
        elapsed_ms: 0,
        rows_in: params.rows_in,
        rows_out: params.rows_out,
        throughput: params.rows_out > 0 ? params.rows_out / Math.max(1, (params.elapsed_ms || 1) / 1000) : 0,
      })
    })

    const unsubLog = window.ajisai.onPipelineLog?.((params) => {
      appendLog(`[${params.level}] ${params.message}`)
    })

    return () => {
      unsubProgress?.()
      unsubLog?.()
    }
  }, [setCategories, appendLog, setPipeline])

  // Auto-save pipeline to localStorage
  useEffect(() => {
    localStorage.setItem('ajisai_pipeline', JSON.stringify(pipeline))
  }, [pipeline])

  const handleNew = () => {
    if (window.confirm('Create new pipeline? Unsaved changes will be lost.')) {
      setPipeline({
        name: 'Untitled Pipeline',
        nodes: [],
        edges: [],
      })
      appendLog('New pipeline created')
    }
  }

  const handleOpen = async () => {
    try {
      const result = await window.ajisai.loadPipeline()
      setPipeline(result.pipeline)
      appendLog(`Loaded pipeline: ${result.pipeline.name}`)
    } catch (err: any) {
      appendLog(`Error loading pipeline: ${err.message}`)
    }
  }

  const handleSave = async () => {
    try {
      await window.ajisai.savePipeline(pipeline)
      appendLog(`Saved pipeline: ${pipeline.name}`)
    } catch (err: any) {
      appendLog(`Error saving pipeline: ${err.message}`)
    }
  }

  const handleRun = async () => {
    if (isRunning) return

    if (pipeline.nodes.length === 0) {
      appendLog('Error: Pipeline has no transforms')
      return
    }

    try {
      setRunning(true)
      resetMetrics()
      appendLog('▶ Starting pipeline execution...')

      const stats = await window.ajisai.runPipeline(pipeline)

      appendLog(`✓ Pipeline completed`)
      appendLog(`  Elapsed: ${stats.elapsed_ms}ms`)
      appendLog(`  Rows read: ${stats.rows_read}`)
      appendLog(`  Rows written: ${stats.rows_written}`)

      // Finalize metrics for dashboard display
      finalizeMetrics()

      setRunning(false)
    } catch (err: any) {
      appendLog(`✕ Pipeline failed: ${err.message}`)
      setRunning(false)
    }
  }

  return (
    <div className="app">
      <MenuBar
        onRun={handleRun}
        onSave={handleSave}
        onOpen={handleOpen}
        onNew={handleNew}
        isRunning={isRunning}
      />
      <Layout />
    </div>
  )
}
