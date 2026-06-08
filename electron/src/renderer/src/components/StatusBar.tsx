import { usePipelineStore } from '../store/pipelineStore'
import './StatusBar.css'

export function StatusBar() {
  const { pipeline, isRunning, nodeStatuses } = usePipelineStore()

  const nodeCount = pipeline.nodes.length
  const edgeCount = pipeline.edges.length
  const runningCount = Object.values(nodeStatuses).filter((s) => s === 'running').length
  const doneCount = Object.values(nodeStatuses).filter((s) => s === 'done').length
  const errorCount = Object.values(nodeStatuses).filter((s) => s === 'error').length

  // Capacity warning
  const capacityStatus = nodeCount > 45 ? 'high' : nodeCount > 30 ? 'medium' : 'ok'
  const capacityMessage =
    capacityStatus === 'high'
      ? '⚠️ Performance may degrade (50+ nodes limit)'
      : capacityStatus === 'medium'
        ? '⚡ Complex pipeline (30+ nodes)'
        : ''

  return (
    <div className="status-bar">
      <div className="status-left">
        <span className="status-item">
          📋 {pipeline.name}
        </span>
      </div>

      <div className="status-center">
        {isRunning && (
          <>
            <span className="status-item status-running">
              ▶ Running
            </span>
            <span className="status-item">
              {runningCount} running
            </span>
          </>
        )}
        {!isRunning && (
          <>
            {doneCount > 0 && (
              <span className="status-item status-done">
                ✓ {doneCount} done
              </span>
            )}
            {errorCount > 0 && (
              <span className="status-item status-error">
                ✕ {errorCount} error
              </span>
            )}
          </>
        )}
      </div>

      <div className="status-right">
        {capacityMessage && (
          <span className={`status-item status-capacity-${capacityStatus}`}>
            {capacityMessage}
          </span>
        )}
        <span className="status-item">
          {nodeCount} nodes
        </span>
        <span className="status-item">
          {edgeCount} edges
        </span>
      </div>
    </div>
  )
}
