import { Handle, Position } from '@xyflow/react'
import type { Node as PipelineNode } from '../types/pipeline'
import './TransformNodeWidget.css'

interface TransformNodeWidgetProps {
  data: {
    node: PipelineNode
    status?: 'idle' | 'running' | 'done' | 'error'
    displayName?: string
  }
  selected?: boolean
}

const CATEGORY_COLORS: Record<string, string> = {
  'I/O': '#4EC9B0',
  'Transform': '#CE9178',
  'Join / Lookup': '#9CDCFE',
  'Variables / Flow': '#C586C0',
}

export function TransformNodeWidget({ data, selected }: TransformNodeWidgetProps) {
  const getCategoryColor = (typeName: string): string => {
    // Simple heuristic to map transform type to category
    if (typeName.includes('Csv') || typeName.includes('Json') || typeName.includes('Excel') ||
        typeName.includes('Parquet') || typeName.includes('Xml') || typeName.includes('Table') ||
        typeName.includes('Rest') || typeName.includes('File') || typeName.includes('Generate')) {
      return CATEGORY_COLORS['I/O']
    }
    if (typeName.includes('Merge') || typeName.includes('Stream') || typeName.includes('Database')) {
      return CATEGORY_COLORS['Join / Lookup']
    }
    if (typeName.includes('SetVariable') || typeName.includes('GetVariable') || typeName.includes('Switch')) {
      return CATEGORY_COLORS['Variables / Flow']
    }
    return CATEGORY_COLORS['Transform']
  }

  const statusColor = {
    idle: '#858585',
    running: '#DDB853',
    done: '#6A9955',
    error: '#F48771',
  }[data.status || 'idle']

  const statusAnimation = data.status === 'running' ? 'pulse' : 'none'

  return (
    <div className={`transform-node ${selected ? 'selected' : ''}`}>
      <Handle type="target" position={Position.Top} />

      <div
        className="node-header"
        style={{ borderTopColor: getCategoryColor(data.node.type_name) }}
      >
        <div className="node-title">
          <span className="node-type">{data.displayName || data.node.type_name}</span>
        </div>
        <div
          className={`status-dot ${data.status || 'idle'}`}
          style={{
            backgroundColor: statusColor,
            animation: statusAnimation === 'pulse' ? 'pulse 1s infinite' : 'none'
          }}
        />
      </div>

      <div className="node-body">
        <span className="node-label">{data.node.label}</span>
      </div>

      <Handle type="source" position={Position.Bottom} />
    </div>
  )
}
