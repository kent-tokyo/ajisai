import { useEffect, useState } from 'react'
import { usePipelineStore } from '../store/pipelineStore'
import './MenuBar.css'

interface MenuBarProps {
  onRun: () => void
  onSave: () => void
  onOpen: () => void
  onNew: () => void
  isRunning: boolean
}

export function MenuBar({ onRun, onSave, onOpen, onNew, isRunning }: MenuBarProps) {
  const { pipeline, clearLog, undo, redo, canUndo, canRedo } = usePipelineStore()
  const [showFileMenu, setShowFileMenu] = useState(false)
  const [showPipelineMenu, setShowPipelineMenu] = useState(false)

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Ctrl+S or Cmd+S: Save
      if ((e.ctrlKey || e.metaKey) && e.key === 's') {
        e.preventDefault()
        onSave()
      }
      // Ctrl+R or Cmd+R: Run
      if ((e.ctrlKey || e.metaKey) && e.key === 'r') {
        e.preventDefault()
        onRun()
      }
      // Ctrl+O or Cmd+O: Open
      if ((e.ctrlKey || e.metaKey) && e.key === 'o') {
        e.preventDefault()
        onOpen()
      }
      // Ctrl+N or Cmd+N: New
      if ((e.ctrlKey || e.metaKey) && e.key === 'n') {
        e.preventDefault()
        onNew()
      }
      // Ctrl+Z or Cmd+Z: Undo
      if ((e.ctrlKey || e.metaKey) && e.key === 'z' && !e.shiftKey) {
        e.preventDefault()
        undo()
      }
      // Ctrl+Shift+Z, Ctrl+Y, Cmd+Shift+Z, Cmd+Y: Redo
      if ((e.ctrlKey || e.metaKey) && (e.key === 'y' || (e.key === 'z' && e.shiftKey))) {
        e.preventDefault()
        redo()
      }
    }

    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [onRun, onSave, onOpen, onNew, undo, redo])

  return (
    <menu className="menu-bar">
      <div className="menu-item">
        <button
          className="menu-label"
          onClick={() => setShowFileMenu(!showFileMenu)}
        >
          File
        </button>
        {showFileMenu && (
          <div className="menu-dropdown">
            <button onClick={() => { onNew(); setShowFileMenu(false); }} className="menu-option">
              New <span className="shortcut">Ctrl+N</span>
            </button>
            <button onClick={() => { onOpen(); setShowFileMenu(false); }} className="menu-option">
              Open <span className="shortcut">Ctrl+O</span>
            </button>
            <button onClick={() => { onSave(); setShowFileMenu(false); }} className="menu-option">
              Save <span className="shortcut">Ctrl+S</span>
            </button>
            <hr className="menu-divider" />
            <button onClick={() => window.close()} className="menu-option">
              Exit
            </button>
          </div>
        )}
      </div>

      <div className="menu-item">
        <button
          className="menu-label"
          onClick={() => setShowPipelineMenu(!showPipelineMenu)}
        >
          Pipeline
        </button>
        {showPipelineMenu && (
          <div className="menu-dropdown">
            <button
              onClick={() => { onRun(); setShowPipelineMenu(false); }}
              className="menu-option"
              disabled={isRunning}
            >
              Run <span className="shortcut">Ctrl+R</span>
            </button>
            <button
              onClick={() => { clearLog(); setShowPipelineMenu(false); }}
              className="menu-option"
            >
              Clear Log
            </button>
            <div style={{ fontSize: '11px', padding: '8px 12px', color: 'var(--text-dim)' }}>
              {pipeline.nodes.length} nodes • {pipeline.edges.length} connections
            </div>
          </div>
        )}
      </div>

      <div style={{ flex: 1 }} />

      <div className="menu-item">
        <button
          className={`menu-run-btn ${isRunning ? 'running' : ''}`}
          onClick={onRun}
          disabled={isRunning}
        >
          {isRunning ? '⟳ Running...' : '▶ Run Pipeline'}
        </button>
      </div>
    </menu>
  )
}
