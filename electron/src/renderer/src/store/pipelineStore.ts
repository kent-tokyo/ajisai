import { create } from 'zustand'
import type { PipelineState, Node, Edge, NodeStatus, TransformCategory } from '../types/pipeline'

interface PipelineStore {
  pipeline: PipelineState
  selectedNodeId: string | null
  nodeStatuses: Record<string, NodeStatus>
  isRunning: boolean
  logLines: string[]
  categories: TransformCategory[]

  // Undo/Redo
  undoStack: PipelineState[]
  redoStack: PipelineState[]

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

  // Undo/Redo
  undo: () => void
  redo: () => void
  canUndo: () => boolean
  canRedo: () => boolean
}

const MAX_UNDO_STACK = 50

const pushUndoStack = (state: PipelineStore, newPipeline: PipelineState) => ({
  pipeline: newPipeline,
  undoStack: [state.pipeline, ...state.undoStack].slice(0, MAX_UNDO_STACK),
  redoStack: [],
})

export const usePipelineStore = create<PipelineStore>((set, get) => ({
  pipeline: { name: 'Untitled Pipeline', nodes: [], edges: [] },
  selectedNodeId: null,
  nodeStatuses: {},
  isRunning: false,
  logLines: [],
  categories: [],
  undoStack: [],
  redoStack: [],

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

  canUndo: () => get().undoStack.length > 0,
  canRedo: () => get().redoStack.length > 0,
}))
