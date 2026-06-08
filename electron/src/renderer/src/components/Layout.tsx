import { Canvas } from './Canvas'
import { Sidebar } from './Sidebar'
import { LogPanel } from './LogPanel'
import { ConfigForm } from './ConfigForm'
import { StatusBar } from './StatusBar'
import { usePipelineStore } from '../store/pipelineStore'
import './Layout.css'

export function Layout() {
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

      {/* Log Panel (Bottom) */}
      <LogPanel />

      {/* Status Bar */}
      <StatusBar />
    </div>
  )
}
