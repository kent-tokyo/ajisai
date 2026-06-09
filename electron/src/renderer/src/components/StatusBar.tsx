import { usePipelineStore } from '../store/pipelineStore'
import { StatusDoneIcon, StatusErrorIcon, WarningIcon, ClipboardIcon, PlayIcon, UndoIcon, RedoIcon } from './Icons'
import './StatusBar.css'

export function StatusBar() {
  const { pipeline, isRunning, nodeStatuses, canUndo, canRedo } = usePipelineStore()

  const nodeCount = pipeline.nodes.length
  const edgeCount = pipeline.edges.length
  const runningCount = Object.values(nodeStatuses).filter((s) => s === 'running').length
  const doneCount = Object.values(nodeStatuses).filter((s) => s === 'done').length
  const errorCount = Object.values(nodeStatuses).filter((s) => s === 'error').length

  // Capacity warning
  const capacityStatus = nodeCount > 45 ? 'high' : nodeCount > 30 ? 'medium' : 'ok'
  const capacityMessage =
    capacityStatus === 'high'
      ? 'Performance may degrade (50+ nodes limit)'
      : capacityStatus === 'medium'
        ? 'Complex pipeline (30+ nodes)'
        : ''

  return (
    <div className="status-bar">
      <div className="status-left">
        <span className="status-item">
          <ClipboardIcon size={14} color="var(--text)" /> {pipeline.name}
        </span>
      </div>

      <div className="status-center">
        <span className="status-item" style={{ opacity: canUndo() ? 1 : 0.4 }}>
          <UndoIcon size={12} color="var(--text-dim)" /> Undo
        </span>
        <span className="status-item" style={{ opacity: canRedo() ? 1 : 0.4 }}>
          <RedoIcon size={12} color="var(--text-dim)" /> Redo
        </span>

        {isRunning && (
          <>
            <span className="status-item status-running">
              <PlayIcon size={12} color="var(--accent)" /> Running
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
                <StatusDoneIcon size={12} color="var(--accent)" /> {doneCount} done
              </span>
            )}
            {errorCount > 0 && (
              <span className="status-item status-error">
                <StatusErrorIcon size={12} color="var(--accent)" /> {errorCount} error
              </span>
            )}
          </>
        )}
      </div>

      <div className="status-right">
        {capacityMessage && (
          <span className={`status-item status-capacity-${capacityStatus}`}>
            {capacityStatus === 'high' && <WarningIcon size={14} color="var(--accent)" />}
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
