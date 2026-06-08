import { useEffect, useRef } from 'react'
import { usePipelineStore } from '../store/pipelineStore'
import './LogPanel.css'

export function LogPanel() {
  const { logLines, clearLog } = usePipelineStore()
  const logsEndRef = useRef<HTMLDivElement>(null)

  // Auto-scroll to bottom when new logs arrive
  useEffect(() => {
    logsEndRef.current?.scrollIntoView({ behavior: 'smooth' })
  }, [logLines])

  return (
    <div className="log-panel">
      <div className="log-header">
        <h4>Log</h4>
        <button className="log-clear-btn" onClick={clearLog} title="Clear logs">
          Clear
        </button>
      </div>

      <div className="log-content">
        {logLines.length === 0 ? (
          <div className="log-empty">Ready to run pipeline...</div>
        ) : (
          <div className="log-lines">
            {logLines.map((line, idx) => (
              <div key={idx} className="log-line">
                <span className="log-text">{line}</span>
              </div>
            ))}
            <div ref={logsEndRef} />
          </div>
        )}
      </div>
    </div>
  )
}
