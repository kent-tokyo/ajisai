import { useCallback, useEffect, useMemo } from 'react'
import {
  ReactFlow,
  Node as FlowNode,
  Edge as FlowEdge,
  Controls,
  Background,
  useNodesState,
  useEdgesState,
  Connection,
  addEdge,
} from '@xyflow/react'
import '@xyflow/react/dist/style.css'
import { TransformNodeWidget } from './TransformNodeWidget'
import { usePipelineStore } from '../store/pipelineStore'
import type { Node as PipelineNode, Edge as PipelineEdge, NodeStatus } from '../types/pipeline'
import './Canvas.css'

const nodeTypes = {
  transform: TransformNodeWidget,
}

const CanvasComponent = () => {
  const {
    pipeline,
    selectedNodeId,
    nodeStatuses,
    addNode,
    selectNode,
    addEdge: addPipelineEdge,
    removeEdge: removePipelineEdge,
    updateNodePos,
  } = usePipelineStore()

  // Convert pipeline nodes to React Flow nodes
  const initialNodes: FlowNode[] = pipeline.nodes.map((node: PipelineNode) => ({
    id: node.id,
    data: {
      node,
      status: nodeStatuses[node.id] || 'idle',
      displayName: node.label,
    },
    position: { x: node.pos[0], y: node.pos[1] },
    type: 'transform',
  }))

  // Convert pipeline edges to React Flow edges
  const initialEdges: FlowEdge[] = pipeline.edges.map((edge: PipelineEdge) => ({
    id: `${edge.from}-${edge.to}`,
    source: edge.from,
    target: edge.to,
  }))

  const [nodes, setNodes, onNodesChange] = useNodesState(initialNodes)
  const [edges, setEdges, onEdgesChange] = useEdgesState(initialEdges)

  // Sync nodes state back to store when they change
  useEffect(() => {
    nodes.forEach((node) => {
      if (node.position.x !== pipeline.nodes.find((n) => n.id === node.id)?.pos[0] ||
          node.position.y !== pipeline.nodes.find((n) => n.id === node.id)?.pos[1]) {
        updateNodePos(node.id, [node.position.x, node.position.y])
      }
    })
  }, [nodes, pipeline.nodes, updateNodePos])

  // Handle new connections
  const onConnect = useCallback(
    (connection: Connection) => {
      if (connection.source && connection.target) {
        addPipelineEdge(connection.source, connection.target)
        setEdges((eds) => addEdge(connection, eds))
      }
    },
    [addPipelineEdge, setEdges]
  )

  // Handle node selection
  const onNodeClick = useCallback(
    (_event: React.MouseEvent, node: FlowNode) => {
      selectNode(node.id)
    },
    [selectNode]
  )

  // Handle node deletion
  const onNodeDelete = useCallback(
    (nodesToDelete: FlowNode[]) => {
      // Note: deletion via Delete key is handled by keyboard handler in App.tsx
      // This is called when edges connected to a deleted node are removed
    },
    []
  )

  // Handle pane click (deselect)
  const onPaneClick = useCallback(() => {
    selectNode(null)
  }, [selectNode])

  // Handle edge deletion
  const onEdgesDelete = useCallback(
    (edgesToDelete: FlowEdge[]) => {
      edgesToDelete.forEach((edge) => {
        const source = nodes.find((n) => n.id === edge.source)
        const target = nodes.find((n) => n.id === edge.target)
        if (source && target) {
          removePipelineEdge(source.id, target.id)
        }
      })
    },
    [nodes, removePipelineEdge]
  )

  return (
    <div className="canvas-container">
      <ReactFlow
        nodes={nodes}
        edges={edges}
        onNodesChange={onNodesChange}
        onEdgesChange={onEdgesChange}
        onConnect={onConnect}
        onNodeClick={onNodeClick}
        onNodesDelete={onNodeDelete}
        onEdgesDelete={onEdgesDelete}
        onPaneClick={onPaneClick}
        nodeTypes={nodeTypes}
        fitView
      >
        <Background />
        <Controls />
      </ReactFlow>
    </div>
  )
}

export const Canvas = () => {
  // Memoize the entire canvas to prevent unnecessary re-renders
  return useMemo(() => <CanvasComponent />, [])
}
