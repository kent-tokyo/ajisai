import { contextBridge, ipcRenderer } from 'electron'

const ajisaiAPI = {
  ping: () => ipcRenderer.invoke('rpc:ping'),

  getTransforms: () => ipcRenderer.invoke('rpc:getTransforms'),

  validatePipeline: (pipeline: unknown) =>
    ipcRenderer.invoke('rpc:validatePipeline', pipeline),

  loadPipeline: (filePath?: string) =>
    ipcRenderer.invoke('rpc:loadPipeline', filePath),

  savePipeline: (pipeline: unknown, filePath?: string, format?: string) =>
    ipcRenderer.invoke('rpc:savePipeline', pipeline, filePath, format),

  runPipeline: (pipeline: unknown) =>
    ipcRenderer.invoke('rpc:runPipeline', pipeline),

  onPipelineProgress: (cb: (params: unknown) => void) => {
    ipcRenderer.on('notify:pipelineProgress', (_e, params) => cb(params))
    return () => ipcRenderer.removeAllListeners('notify:pipelineProgress')
  },

  onPipelineLog: (cb: (params: unknown) => void) => {
    ipcRenderer.on('notify:pipelineLog', (_e, params) => cb(params))
    return () => ipcRenderer.removeAllListeners('notify:pipelineLog')
  },
}

contextBridge.exposeInMainWorld('ajisai', ajisaiAPI)

declare global {
  interface Window {
    ajisai: typeof ajisaiAPI
  }
}
