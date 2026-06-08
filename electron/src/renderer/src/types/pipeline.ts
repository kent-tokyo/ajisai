export interface Node {
  id: string
  type_name: string
  label: string
  pos: [number, number]
  config: Record<string, unknown>
}

export interface Edge {
  from: string
  to: string
}

export interface PipelineState {
  name: string
  nodes: Node[]
  edges: Edge[]
  node_seq?: number
}

export interface TransformDescriptor {
  type_name: string
  display_name: string
  default_config: Record<string, unknown>
}

export interface TransformCategory {
  name: string
  transforms: TransformDescriptor[]
}

export type NodeStatus = 'idle' | 'running' | 'done' | 'error'

export interface ProgressEvent {
  node_id: string
  status: NodeStatus
  rows_in: number
  rows_out: number
  message?: string
}

export interface ExecutionStats {
  elapsed_ms: number
  rows_read: number
  rows_written: number
}

export interface NodeMetrics {
  node_id: string
  node_label: string
  type_name: string
  elapsed_ms: number
  rows_in: number
  rows_out: number
  throughput: number // rows/sec
  start_time?: number
  end_time?: number
}

export interface PipelineMetrics {
  pipeline_name: string
  total_elapsed_ms: number
  total_rows_read: number
  total_rows_written: number
  node_metrics: NodeMetrics[]
  start_time?: number
  end_time?: number
  avg_throughput: number // total_rows_written / (total_elapsed_ms / 1000)
}
