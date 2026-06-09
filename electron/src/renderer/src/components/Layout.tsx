import { useState } from 'react'
import { Canvas } from './Canvas'
import { Sidebar } from './Sidebar'
import { LogPanel } from './LogPanel'
import { MetricsPanel } from './MetricsPanel'
import { ConfigForm } from './ConfigForm'
import { StatusBar } from './StatusBar'
import { usePipelineStore } from '../store/pipelineStore'
import './Layout.css'

type BottomTab = 'logs' | 'metrics'

export function Layout() {
  const [activeTab, setActiveTab] = useState<BottomTab>('logs')
  const [sidebarWidth, setSidebarWidth] = useState(() => {
    const saved = localStorage.getItem('ajisai_sidebar_width')
    return saved ? parseInt(saved, 10) : 240
  })
  const [propertiesWidth, setPropertiesWidth] = useState(() => {
    const saved = localStorage.getItem('ajisai_properties_width')
    return saved ? parseInt(saved, 10) : 280
  })
  const [bottomPanelHeight, setBottomPanelHeight] = useState(() => {
    const saved = localStorage.getItem('ajisai_bottom_panel_height')
    return saved ? parseInt(saved, 10) : 160
  })
  const [isDragging, setIsDragging] = useState<'sidebar' | 'properties' | 'bottom' | null>(null)

  const handleMouseMove = (e: React.MouseEvent) => {
    if (!isDragging) return

    if (isDragging === 'sidebar') {
      const newWidth = Math.max(180, Math.min(400, e.clientX - 48))
      setSidebarWidth(newWidth)
      localStorage.setItem('ajisai_sidebar_width', String(newWidth))
    } else if (isDragging === 'properties') {
      const mainContentRect = document.querySelector('.main-content')?.getBoundingClientRect()
      if (mainContentRect) {
        const newWidth = Math.max(240, Math.min(400, mainContentRect.right - e.clientX))
        setPropertiesWidth(newWidth)
        localStorage.setItem('ajisai_properties_width', String(newWidth))
      }
    } else if (isDragging === 'bottom') {
      const layoutRect = document.querySelector('.layout')?.getBoundingClientRect()
      if (layoutRect) {
        const newHeight = Math.max(100, Math.min(300, layoutRect.bottom - e.clientY))
        setBottomPanelHeight(newHeight)
        localStorage.setItem('ajisai_bottom_panel_height', String(newHeight))
      }
    }
  }

  const handleMouseUp = () => {
    setIsDragging(null)
  }
  const { selectedNodeId, pipeline, updateNodeConfig } = usePipelineStore()

  const selectedNode = selectedNodeId
    ? pipeline.nodes.find((n) => n.id === selectedNodeId)
    : null

  return (
    <div
      className="layout"
      onMouseMove={handleMouseMove}
      onMouseUp={handleMouseUp}
      onMouseLeave={handleMouseUp}
      style={{ userSelect: isDragging ? 'none' : 'auto' }}
    >
      {/* Activity Bar */}
      <div className="activity-bar">
        <div className="activity-icon" title="Pipeline Editor">
          📊
        </div>
      </div>

      {/* Main Content Area */}
      <div
        className="main-content"
        style={{
          gridTemplateColumns: `${sidebarWidth}px 1fr ${propertiesWidth}px`,
        }}
      >
        {/* Sidebar (Transform Palette) */}
        <Sidebar />

        {/* Sidebar Resize Handle */}
        <div
          className="resize-handle resize-handle-vertical"
          onMouseDown={() => setIsDragging('sidebar')}
        />

        {/* Canvas */}
        <Canvas />

        {/* Properties Resize Handle */}
        <div
          className="resize-handle resize-handle-vertical"
          onMouseDown={() => setIsDragging('properties')}
        />

        {/* Properties Panel */}
        <div className="properties-panel">
          <ConfigForm
            node={selectedNode || null}
            onConfigChange={updateNodeConfig}
          />
        </div>
      </div>

      {/* Bottom Resize Handle */}
      <div
        className="resize-handle resize-handle-horizontal"
        onMouseDown={() => setIsDragging('bottom')}
      />

      {/* Bottom Tabs */}
      <div className="bottom-tabs-header">
        <button
          className={`tab-button ${activeTab === 'logs' ? 'active' : ''}`}
          onClick={() => setActiveTab('logs')}
        >
          Log
        </button>
        <button
          className={`tab-button ${activeTab === 'metrics' ? 'active' : ''}`}
          onClick={() => setActiveTab('metrics')}
        >
          Metrics
        </button>
      </div>

      {/* Bottom Panel (Log or Metrics) */}
      <div className="bottom-panel" style={{ height: `${bottomPanelHeight}px` }}>
        {activeTab === 'logs' ? <LogPanel /> : <MetricsPanel />}
      </div>

      {/* Status Bar */}
      <StatusBar />
    </div>
  )
}
