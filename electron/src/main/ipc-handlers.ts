import { ipcMain, BrowserWindow, dialog } from 'electron'
import { sidecar } from './sidecar'

export function registerIpcHandlers(win: BrowserWindow): void {
  // RPC methods
  ipcMain.handle('rpc:ping', () =>
    sidecar.call('ping')
  )

  ipcMain.handle('rpc:getTransforms', () =>
    sidecar.call('get_transforms')
  )

  ipcMain.handle('rpc:validatePipeline', (_e, pipeline) =>
    sidecar.call('validate_pipeline', { pipeline })
  )

  ipcMain.handle('rpc:loadPipeline', async (_e, filePath?: string) => {
    const path = filePath || (await dialog.showOpenDialog(win, {
      filters: [
        { name: 'HPL Files', extensions: ['hpl'] },
        { name: 'JSON Files', extensions: ['json'] },
        { name: 'All Files', extensions: ['*'] }
      ]
    })).filePaths[0]

    if (!path) throw new Error('No file selected')
    return sidecar.call('load_pipeline', { path })
  })

  ipcMain.handle('rpc:savePipeline', async (_e, pipeline, filePath?: string, format = 'json') => {
    const path = filePath || (await dialog.showSaveDialog(win, {
      defaultPath: 'pipeline.json',
      filters: [
        { name: 'JSON Files', extensions: ['json'] },
        { name: 'HPL Files', extensions: ['hpl'] }
      ]
    })).filePath

    if (!path) throw new Error('No file selected')
    return sidecar.call('save_pipeline', { pipeline, path, format })
  })

  ipcMain.handle('rpc:runPipeline', (_e, pipeline) =>
    sidecar.call('run_pipeline', { pipeline })
  )

  // Forward notifications to renderer
  sidecar.on('pipeline/progress', (params) => {
    win.webContents.send('notify:pipelineProgress', params)
  })

  sidecar.on('pipeline/log', (params) => {
    win.webContents.send('notify:pipelineLog', params)
  })
}
