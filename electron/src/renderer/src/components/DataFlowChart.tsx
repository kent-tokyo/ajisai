import { usePipelineStore } from '../store/pipelineStore'
import './DataFlowChart.css'

export function DataFlowChart() {
  const { pipeline, currentMetrics } = usePipelineStore()

  if (!currentMetrics || pipeline.nodes.length === 0) {
    return (
      <div className="dataflow-chart">
        <div className="dataflow-empty">No pipeline data to visualize</div>
      </div>
    )
  }

  const metrics = currentMetrics

  // Build node position map for layout
  const nodePositions = new Map<string, { x: number; y: number }>()
  const sortedNodeIds = pipeline.nodes.map((n) => n.id)

  // Simple linear layout
  pipeline.nodes.forEach((node, idx) => {
    nodePositions.set(node.id, {
      x: 60 + (idx % 5) * 140,
      y: 40 + Math.floor(idx / 5) * 80,
    })
  })

  // Get metrics for each node
  const getNodeMetrics = (nodeId: string) => {
    return metrics.node_metrics.find((m) => m.node_id === nodeId)
  }

  // Get max rows for scaling
  const maxRows = Math.max(
    ...metrics.node_metrics.map((m) => m.rows_out),
    1
  )

  return (
    <div className="dataflow-chart">
      <div className="dataflow-header">
        <h5>Data Flow Diagram</h5>
      </div>

      <svg className="dataflow-svg" viewBox="0 0 800 300" preserveAspectRatio="xMidYMid meet">
        {/* Draw edges (data flows) */}
        {pipeline.edges.map((edge, idx) => {
          const from = nodePositions.get(edge.from)
          const to = nodePositions.get(edge.to)

          if (!from || !to) return null

          const fromMetrics = getNodeMetrics(edge.from)
          const toMetrics = getNodeMetrics(edge.to)
          const rowsFlow = fromMetrics?.rows_out || 0

          // Scale line width by row count
          const lineWidth = Math.max(1, Math.min(6, (rowsFlow / maxRows) * 6))

          return (
            <g key={`edge-${idx}`}>
              {/* Arrow line */}
              <line
                x1={from.x + 40}
                y1={from.y + 16}
                x2={to.x}
                y2={to.y + 16}
                stroke="var(--accent)"
                strokeWidth={lineWidth}
                opacity="0.6"
              />
              {/* Arrow head */}
              <polygon
                points={`${to.x},${to.y + 16} ${to.x - 6},${to.y + 12} ${to.x - 6},${to.y + 20}`}
                fill="var(--accent)"
                opacity="0.6"
              />
              {/* Flow label (rows) */}
              <text
                x={(from.x + 40 + to.x) / 2}
                y={(from.y + 16 + to.y + 16) / 2 - 4}
                className="dataflow-label"
              >
                {rowsFlow} rows
              </text>
            </g>
          )
        })}

        {/* Draw nodes */}
        {pipeline.nodes.map((node) => {
          const pos = nodePositions.get(node.id)
          if (!pos) return null

          const nodeMetrics = getNodeMetrics(node.id)
          const rowsOut = nodeMetrics?.rows_out || 0
          const statusColor =
            nodeMetrics && rowsOut > 0 ? 'var(--status-done)' : 'var(--status-idle)'

          return (
            <g key={`node-${node.id}`}>
              {/* Node background */}
              <rect
                x={pos.x}
                y={pos.y}
                width="80"
                height="32"
                rx="4"
                fill="var(--bg-input)"
                stroke={statusColor}
                strokeWidth="2"
              />
              {/* Node label */}
              <text x={pos.x + 40} y={pos.y + 12} className="dataflow-node-label">
                {node.label}
              </text>
              {/* Row count */}
              <text x={pos.x + 40} y={pos.y + 26} className="dataflow-node-rows">
                {rowsOut} rows
              </text>
            </g>
          )
        })}
      </svg>

      <div className="dataflow-legend">
        <div className="legend-item">
          <div className="legend-dot" style={{ borderColor: 'var(--status-done)' }} />
          <span>Transform executed</span>
        </div>
        <div className="legend-item">
          <div className="legend-dot" style={{ borderColor: 'var(--status-idle)' }} />
          <span>No data output</span>
        </div>
      </div>
    </div>
  )
}
