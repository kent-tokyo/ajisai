import { memo, useMemo } from 'react'
import { usePipelineStore } from '../store/pipelineStore'
import { DataFlowChart } from './DataFlowChart'
import './MetricsPanel.css'

function MetricsPanelComponent() {
  const { currentMetrics } = usePipelineStore()

  if (!currentMetrics) {
    return (
      <div className="metrics-panel">
        <div className="metrics-empty">Run pipeline to see metrics</div>
      </div>
    )
  }

  const metrics = currentMetrics

  // Memoize sorted node metrics to avoid re-sorting on every render
  const sortedNodeMetrics = useMemo(
    () => [...metrics.node_metrics].sort((a, b) => b.elapsed_ms - a.elapsed_ms),
    [metrics.node_metrics]
  )

  return (
    <div className="metrics-panel">
      <div className="metrics-header">
        <h4>Performance Metrics</h4>
        <span className="metrics-pipeline">{metrics.pipeline_name}</span>
      </div>

      <div className="metrics-summary">
        <div className="metric-card">
          <div className="metric-label">Total Time</div>
          <div className="metric-value">{(metrics.total_elapsed_ms / 1000).toFixed(2)}s</div>
        </div>
        <div className="metric-card">
          <div className="metric-label">Rows Processed</div>
          <div className="metric-value">{metrics.total_rows_written}</div>
        </div>
        <div className="metric-card">
          <div className="metric-label">Throughput</div>
          <div className="metric-value">{metrics.avg_throughput.toFixed(0)} row/s</div>
        </div>
      </div>

      {/* Data Flow Diagram */}
      <DataFlowChart />

      {metrics.node_metrics.length > 0 && (
        <div className="metrics-table">
          <h5>Transform Performance</h5>
          <table className="metrics-table-body">
            <thead>
              <tr>
                <th>Transform</th>
                <th>Type</th>
                <th>Time (ms)</th>
                <th>In</th>
                <th>Out</th>
                <th>Rate</th>
              </tr>
            </thead>
            <tbody>
              {sortedNodeMetrics.map((node) => (
                  <tr key={node.node_id}>
                    <td className="metric-name">{node.node_label}</td>
                    <td className="metric-type">{node.type_name}</td>
                    <td className="metric-number">{node.elapsed_ms}</td>
                    <td className="metric-number">{node.rows_in}</td>
                    <td className="metric-number">{node.rows_out}</td>
                    <td className="metric-number">{node.throughput.toFixed(0)}/s</td>
                  </tr>
                ))}
            </tbody>
          </table>
        </div>
      )}

      <div className="metrics-footer">
        <p className="metrics-hint">
          Data flow width represents row volume. Narrow flows may indicate data loss or filtering.
        </p>
      </div>
    </div>
  )
}

export const MetricsPanel = memo(MetricsPanelComponent)
