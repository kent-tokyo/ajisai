import { create } from 'zustand'
import type { PipelineState, Node, Edge, NodeStatus, TransformCategory, PipelineMetrics, NodeMetrics } from '../types/pipeline'

interface PipelineStore {
  pipeline: PipelineState
  selectedNodeId: string | null
  nodeStatuses: Record<string, NodeStatus>
  isRunning: boolean
  logLines: string[]
  categories: TransformCategory[]

  // Metrics tracking
  currentMetrics: PipelineMetrics | null
  nodeMetricsMap: Record<string, NodeMetrics>

  // Undo/Redo
  undoStack: PipelineState[]
  redoStack: PipelineState[]

  // State persistence
  isDirty: boolean
  lastSavedAt: number | null

  setPipeline: (pipeline: PipelineState) => void
  setName: (name: string) => void
  addNode: (typeName: string, pos: [number, number]) => void
  removeNode: (id: string) => void
  updateNodeConfig: (id: string, config: Record<string, unknown>) => void
  updateNodeLabel: (id: string, label: string) => void
  updateNodePos: (id: string, pos: [number, number]) => void
  addEdge: (from: string, to: string) => void
  removeEdge: (from: string, to: string) => void

  selectNode: (id: string | null) => void

  setRunning: (running: boolean) => void
  setNodeStatus: (nodeId: string, status: NodeStatus) => void

  appendLog: (line: string) => void
  clearLog: () => void

  setCategories: (categories: TransformCategory[]) => void

  // Metrics
  recordNodeMetrics: (nodeId: string, metrics: Partial<NodeMetrics>) => void
  resetMetrics: () => void
  finalizeMetrics: () => void

  // Undo/Redo
  undo: () => void
  redo: () => void
  canUndo: () => boolean
  canRedo: () => boolean

  // State persistence
  markSaved: () => void
  markDirty: () => void
  autoSaveToLocalStorage: () => void
  restoreFromLocalStorage: () => void
}

const MAX_UNDO_STACK = 50

const pushUndoStack = (state: PipelineStore, newPipeline: PipelineState) => ({
  pipeline: newPipeline,
  undoStack: [state.pipeline, ...state.undoStack].slice(0, MAX_UNDO_STACK),
  redoStack: [],
  isDirty: true,
})

const AUTOSAVE_KEY = 'ajisai_autosave_pipeline'
const AUTOSAVE_TIME_KEY = 'ajisai_autosave_time'

export const usePipelineStore = create<PipelineStore>((set, get) => ({
  pipeline: { name: 'Untitled Pipeline', nodes: [], edges: [] },
  selectedNodeId: null,
  nodeStatuses: {},
  isRunning: false,
  logLines: [],
  categories: [],
  currentMetrics: null,
  nodeMetricsMap: {},
  undoStack: [],
  redoStack: [],
  isDirty: false,
  lastSavedAt: null,

  setPipeline: (newPipeline) => set({
    pipeline: newPipeline,
    nodeStatuses: {},
    selectedNodeId: null,
    undoStack: [],
    redoStack: [],
  }),

  setName: (name) => set((state) => {
    const newPipeline = { ...state.pipeline, name }
    return pushUndoStack(state, newPipeline)
  }),

  addNode: (typeName, pos) => set((state) => {
    const newId = `node_${Math.random().toString(36).substr(2, 9)}`
    const newNode: Node = {
      id: newId,
      type_name: typeName,
      label: typeName,
      pos,
      config: {}
    }
    const newPipeline = {
      ...state.pipeline,
      nodes: [...state.pipeline.nodes, newNode]
    }
    return pushUndoStack(state, newPipeline)
  }),

  removeNode: (id) => set((state) => {
    const newPipeline = {
      ...state.pipeline,
      nodes: state.pipeline.nodes.filter((n) => n.id !== id),
      edges: state.pipeline.edges.filter((e) => e.from !== id && e.to !== id)
    }
    return pushUndoStack(state, newPipeline)
  }),

  updateNodeConfig: (id, config) => set((state) => {
    const newPipeline = {
      ...state.pipeline,
      nodes: state.pipeline.nodes.map((n) =>
        n.id === id ? { ...n, config } : n
      )
    }
    return pushUndoStack(state, newPipeline)
  }),

  updateNodeLabel: (id, label) => set((state) => {
    const newPipeline = {
      ...state.pipeline,
      nodes: state.pipeline.nodes.map((n) =>
        n.id === id ? { ...n, label } : n
      )
    }
    return pushUndoStack(state, newPipeline)
  }),

  updateNodePos: (id, pos) => set((state) => {
    const newPipeline = {
      ...state.pipeline,
      nodes: state.pipeline.nodes.map((n) =>
        n.id === id ? { ...n, pos } : n
      )
    }
    return pushUndoStack(state, newPipeline)
  }),

  addEdge: (from, to) => set((state) => {
    const edge: Edge = { from, to }
    if (state.pipeline.edges.some((e) => e.from === from && e.to === to)) {
      return state
    }
    const newPipeline = {
      ...state.pipeline,
      edges: [...state.pipeline.edges, edge]
    }
    return pushUndoStack(state, newPipeline)
  }),

  removeEdge: (from, to) => set((state) => {
    const newPipeline = {
      ...state.pipeline,
      edges: state.pipeline.edges.filter((e) => !(e.from === from && e.to === to))
    }
    return pushUndoStack(state, newPipeline)
  }),

  selectNode: (id) => set({ selectedNodeId: id }),

  setRunning: (running) => set({ isRunning: running }),

  setNodeStatus: (nodeId, status) => set((state) => ({
    nodeStatuses: { ...state.nodeStatuses, [nodeId]: status }
  })),

  appendLog: (line) => set((state) => {
    const lines = [...state.logLines, line]
    if (lines.length > 500) {
      lines.splice(0, 100)
    }
    return { logLines: lines }
  }),

  clearLog: () => set({ logLines: [] }),

  setCategories: (categories) => set({ categories }),

  undo: () => set((state) => {
    if (state.undoStack.length === 0) return state
    const [next, ...rest] = state.undoStack
    return {
      ...state,
      pipeline: next,
      undoStack: rest,
      redoStack: [state.pipeline, ...state.redoStack],
      selectedNodeId: null,
      nodeStatuses: {},
    }
  }),

  redo: () => set((state) => {
    if (state.redoStack.length === 0) return state
    const [next, ...rest] = state.redoStack
    return {
      ...state,
      pipeline: next,
      redoStack: rest,
      undoStack: [state.pipeline, ...state.undoStack],
      selectedNodeId: null,
      nodeStatuses: {},
    }
  }),

  recordNodeMetrics: (nodeId, metrics) => set((state) => {
    const existing = state.nodeMetricsMap[nodeId] || {
      node_id: nodeId,
      node_label: state.pipeline.nodes.find((n) => n.id === nodeId)?.label || nodeId,
      type_name: state.pipeline.nodes.find((n) => n.id === nodeId)?.type_name || '',
      elapsed_ms: 0,
      rows_in: 0,
      rows_out: 0,
      throughput: 0,
    }
    return {
      nodeMetricsMap: {
        ...state.nodeMetricsMap,
        [nodeId]: { ...existing, ...metrics }
      }
    }
  }),

  resetMetrics: () => set({
    currentMetrics: null,
    nodeMetricsMap: {},
  }),

  finalizeMetrics: () => set((state) => {
    const nodeMetrics = Object.values(state.nodeMetricsMap)
    const totalElapsedMs = Math.max(...nodeMetrics.map((m) => m.elapsed_ms), 0)
    const totalRowsRead = nodeMetrics.reduce((sum, m) => sum + m.rows_in, 0)
    const totalRowsWritten = nodeMetrics.reduce((sum, m) => sum + m.rows_out, 0)
    const avgThroughput = totalElapsedMs > 0 ? totalRowsWritten / (totalElapsedMs / 1000) : 0

    const metrics: PipelineMetrics = {
      pipeline_name: state.pipeline.name,
      total_elapsed_ms: totalElapsedMs,
      total_rows_read: totalRowsRead,
      total_rows_written: totalRowsWritten,
      node_metrics: nodeMetrics,
      avg_throughput: avgThroughput,
    }

    return { currentMetrics: metrics }
  }),

  canUndo: () => get().undoStack.length > 0,
  canRedo: () => get().redoStack.length > 0,

  markSaved: () => set({
    isDirty: false,
    lastSavedAt: Date.now(),
  }),

  markDirty: () => set({ isDirty: true }),

  autoSaveToLocalStorage: () => {
    const state = get()
    const data = {
      pipeline: state.pipeline,
      timestamp: Date.now(),
    }
    localStorage.setItem(AUTOSAVE_KEY, JSON.stringify(data))
    localStorage.setItem(AUTOSAVE_TIME_KEY, String(Date.now()))
  },

  restoreFromLocalStorage: () => {
    const stored = localStorage.getItem(AUTOSAVE_KEY)
    if (stored) {
      try {
        const data = JSON.parse(stored)
        set({
          pipeline: data.pipeline,
          selectedNodeId: null,
          nodeStatuses: {},
          undoStack: [],
          redoStack: [],
          isDirty: false,
          lastSavedAt: data.timestamp,
        })
        return true
      } catch (err) {
        console.error('Failed to restore pipeline:', err)
        return false
      }
    }
    return false
  },
}))
