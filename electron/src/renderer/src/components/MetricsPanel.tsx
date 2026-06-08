import { usePipelineStore } from '../store/pipelineStore'
import './MetricsPanel.css'

export function MetricsPanel() {
  const { currentMetrics } = usePipelineStore()

  if (!currentMetrics) {
    return (
      <div className="metrics-panel">
        <div className="metrics-empty">Run pipeline to see metrics</div>
      </div>
    )
  }

  const metrics = currentMetrics

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
              {metrics.node_metrics
                .sort((a, b) => b.elapsed_ms - a.elapsed_ms)
                .map((node) => (
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
          💡 Sort by slowest transform to find bottlenecks
        </p>
      </div>
    </div>
  )
}
