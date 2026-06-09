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
  const { selectedNodeId, pipeline, updateNodeConfig } = usePipelineStore()

  const selectedNode = selectedNodeId
    ? pipeline.nodes.find((n) => n.id === selectedNodeId)
    : null

  return (
    <div className="layout">
      {/* Activity Bar */}
      <div className="activity-bar">
        <div className="activity-icon" title="Pipeline Editor">
          📊
        </div>
      </div>

      {/* Main Content Area */}
      <div className="main-content">
        {/* Sidebar (Transform Palette) */}
        <Sidebar />

        {/* Canvas */}
        <Canvas />

        {/* Properties Panel */}
        <div className="properties-panel">
          <ConfigForm
            node={selectedNode || null}
            onConfigChange={updateNodeConfig}
          />
        </div>
      </div>

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
      <div className="bottom-panel">
        {activeTab === 'logs' ? <LogPanel /> : <MetricsPanel />}
      </div>

      {/* Status Bar */}
      <StatusBar />
    </div>
  )
}
