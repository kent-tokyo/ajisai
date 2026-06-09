import { memo, useMemo } from 'react'
import { usePipelineStore } from '../store/pipelineStore'
import './DataFlowChart.css'

// Branch node types that support conditional output
const BRANCH_NODE_TYPES = ['SwitchCase', 'ConditionalBranch', 'Filter']

// Color palette for multiple outputs
const EDGE_COLORS = [
  'var(--accent)',
  '#CE9178',
  '#9CDCFE',
  '#C586C0',
  '#4EC9B0',
  '#DCDCAA',
]

function DataFlowChartComponent() {
  const { pipeline, currentMetrics } = usePipelineStore()

  if (!currentMetrics || pipeline.nodes.length === 0) {
    return (
      <div className="dataflow-chart">
        <div className="dataflow-empty">No pipeline data to visualize</div>
      </div>
    )
  }

  const metrics = currentMetrics

  // Memoize branch node detection to avoid recalculation on every render
  const { outgoingEdges, branchNodes, nodePositions } = useMemo(() => {
    // Detect branch nodes (nodes with multiple outputs)
    const outgoingEdgesMap = new Map<string, string[]>()
    const branchNodesSet = new Set<string>()

    pipeline.edges.forEach((edge) => {
      if (!outgoingEdgesMap.has(edge.from)) {
        outgoingEdgesMap.set(edge.from, [])
      }
      outgoingEdgesMap.get(edge.from)!.push(edge.to)
    })

    // Mark nodes as branch if they have multiple outputs or are known branch types
    pipeline.nodes.forEach((node) => {
      const outputCount = outgoingEdgesMap.get(node.id)?.length || 0
      if (outputCount > 1 || BRANCH_NODE_TYPES.some(t => node.type_name.includes(t))) {
        branchNodesSet.add(node.id)
      }
    })

    // Build node position map with branch-aware layout
    const nodePositionsMap = new Map<string, { x: number; y: number }>()

    // First pass: position nodes in sequence, marking branch nodes
    let nodeIdx = 0
    const nodeOrder = new Map<string, number>()

    pipeline.nodes.forEach((node) => {
      nodeOrder.set(node.id, nodeIdx++)
    })

    // Layout: horizontal flow with vertical offset for branches
    pipeline.nodes.forEach((node) => {
      const idx = nodeOrder.get(node.id) || 0

      if (branchNodesSet.has(node.id)) {
        // Branch nodes positioned with some horizontal spacing
        nodePositionsMap.set(node.id, {
          x: 80 + idx * 120,
          y: 80, // Center vertically
        })
      } else {
        nodePositionsMap.set(node.id, {
          x: 80 + idx * 120,
          y: 40 + Math.floor(idx / 4) * 100,
        })
      }
    })

    return { outgoingEdges: outgoingEdgesMap, branchNodes: branchNodesSet, nodePositions: nodePositionsMap }
  }, [pipeline.nodes, pipeline.edges])

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

      <svg className="dataflow-svg" viewBox="0 0 900 350" preserveAspectRatio="xMidYMid meet">
        {/* Draw edges (data flows) */}
        {pipeline.edges.map((edge, edgeIdx) => {
          const from = nodePositions.get(edge.from)
          const to = nodePositions.get(edge.to)

          if (!from || !to) return null

          const fromMetrics = getNodeMetrics(edge.from)
          const rowsFlow = fromMetrics?.rows_out || 0
          const isBranchEdge = branchNodes.has(edge.from)

          // For branch edges, rotate through color palette
          const outputIndex = outgoingEdges.get(edge.from)?.indexOf(edge.to) || 0
          const edgeColor = isBranchEdge
            ? EDGE_COLORS[outputIndex % EDGE_COLORS.length]
            : 'var(--accent)'

          // Scale line width by row count
          const lineWidth = Math.max(1, Math.min(6, (rowsFlow / maxRows) * 6))

          // Curve path for branch edges
          const midX = (from.x + 40 + to.x) / 2
          const midY = isBranchEdge
            ? Math.min(from.y + 16, to.y + 16) - 40 + (outputIndex % 2) * 20
            : (from.y + 16 + to.y + 16) / 2

          return (
            <g key={`edge-${edgeIdx}`} className={isBranchEdge ? 'dataflow-branch-edge' : ''}>
              {/* Curved or straight arrow line */}
              {isBranchEdge ? (
                <path
                  d={`M ${from.x + 40} ${from.y + 16} Q ${midX} ${midY} ${to.x} ${to.y + 16}`}
                  stroke={edgeColor}
                  strokeWidth={lineWidth}
                  fill="none"
                  opacity="0.7"
                  strokeLinecap="round"
                />
              ) : (
                <line
                  x1={from.x + 40}
                  y1={from.y + 16}
                  x2={to.x}
                  y2={to.y + 16}
                  stroke={edgeColor}
                  strokeWidth={lineWidth}
                  opacity="0.6"
                />
              )}

              {/* Arrow head */}
              <polygon
                points={`${to.x},${to.y + 16} ${to.x - 6},${to.y + 12} ${to.x - 6},${to.y + 20}`}
                fill={edgeColor}
                opacity={isBranchEdge ? '0.8' : '0.6'}
              />

              {/* Flow label (rows) */}
              <text
                x={midX}
                y={isBranchEdge ? midY - 8 : midY - 4}
                className="dataflow-label"
                fill={edgeColor}
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
          const isBranch = branchNodes.has(node.id)

          // Branch nodes: use diamond shape
          if (isBranch) {
            const size = 16
            return (
              <g key={`node-${node.id}`} className="dataflow-branch-node">
                {/* Diamond shape for branch node */}
                <polygon
                  points={`${pos.x + 40},${pos.y} ${pos.x + 40 + size},${pos.y + 16} ${pos.x + 40},${pos.y + 32} ${pos.x + 40 - size},${pos.y + 16}`}
                  fill="var(--bg-input)"
                  stroke={statusColor}
                  strokeWidth="2"
                />
                {/* Label inside diamond */}
                <text x={pos.x + 40} y={pos.y + 20} className="dataflow-node-label-small">
                  {node.label}
                </text>
              </g>
            )
          }

          // Regular nodes: rectangular
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
        <div className="legend-item">
          <div className="legend-diamond" style={{ borderColor: 'var(--status-done)' }} />
          <span>Branch condition</span>
        </div>
      </div>
    </div>
  )
}

export const DataFlowChart = memo(DataFlowChartComponent)
