import { useState, useEffect, useRef, useMemo, memo } from 'react'
import { usePipelineStore } from '../store/pipelineStore'
import './LogPanel.css'

type LogLevel = 'all' | 'info' | 'warn' | 'error'

function parseLogLevel(line: string): LogLevel {
  if (line.includes('[error]') || line.includes('✕')) return 'error'
  if (line.includes('[warn]')) return 'warn'
  if (line.includes('[info]')) return 'info'
  return 'info'
}

function LogPanelComponent() {
  const { logLines, clearLog } = usePipelineStore()
  const [searchText, setSearchText] = useState('')
  const [levelFilter, setLevelFilter] = useState<LogLevel>('all')
  const logsEndRef = useRef<HTMLDivElement>(null)

  // Auto-scroll to bottom when new logs arrive
  useEffect(() => {
    logsEndRef.current?.scrollIntoView({ behavior: 'smooth' })
  }, [logLines])

  // Filter logs based on search and level
  const filteredLogs = useMemo(() => {
    return logLines.filter((line) => {
      const matchesSearch = searchText === '' ||
        line.toLowerCase().includes(searchText.toLowerCase())

      const matchesLevel = levelFilter === 'all' ||
        parseLogLevel(line) === levelFilter

      return matchesSearch && matchesLevel
    })
  }, [logLines, searchText, levelFilter])

  return (
    <div className="log-panel">
      <div className="log-header">
        <h4>Log ({filteredLogs.length})</h4>
        <div className="log-controls">
          <input
            type="text"
            className="log-search"
            placeholder="Search logs..."
            value={searchText}
            onChange={(e) => setSearchText(e.target.value)}
          />
          <select
            className="log-filter"
            value={levelFilter}
            onChange={(e) => setLevelFilter(e.target.value as LogLevel)}
          >
            <option value="all">All</option>
            <option value="info">Info</option>
            <option value="warn">Warning</option>
            <option value="error">Error</option>
          </select>
          <button className="log-clear-btn" onClick={clearLog} title="Clear logs">
            Clear
          </button>
        </div>
      </div>

      <div className="log-content">
        {logLines.length === 0 ? (
          <div className="log-empty">Ready to run pipeline...</div>
        ) : filteredLogs.length === 0 ? (
          <div className="log-empty">No logs match filter</div>
        ) : (
          <div className="log-lines">
            {filteredLogs.map((line, idx) => {
              const level = parseLogLevel(line)
              return (
                <div key={idx} className={`log-line log-${level}`}>
                  <span className="log-text">{line}</span>
                </div>
              )
            })}
            <div ref={logsEndRef} />
          </div>
        )}
      </div>
    </div>
  )
}

export const LogPanel = memo(LogPanelComponent)
