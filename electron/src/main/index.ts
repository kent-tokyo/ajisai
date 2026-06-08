import { app, BrowserWindow } from 'electron'
import { join } from 'path'
import { autoUpdater } from 'electron-updater'
import { sidecar } from './sidecar'
import { registerIpcHandlers } from './ipc-handlers'

let mainWindow: BrowserWindow | null = null

function setupAutoUpdater() {
  if (process.env.NODE_ENV === 'development') {
    console.log('Skipping auto-update in development mode')
    return
  }

  autoUpdater.checkForUpdatesAndNotify()

  autoUpdater.on('update-available', () => {
    console.log('Update available, downloading...')
  })

  autoUpdater.on('update-downloaded', () => {
    console.log('Update downloaded, will install on restart')
  })

  autoUpdater.on('error', (err) => {
    console.error('Update error:', err)
  })
}

function createWindow() {
  mainWindow = new BrowserWindow({
    width: 1280,
    height: 800,
    minWidth: 800,
    minHeight: 500,
    webPreferences: {
      preload: join(__dirname, '../preload/index.js'),
      contextIsolation: true,
      enableRemoteModule: false,
      nodeIntegration: false,
      sandbox: true
    }
  })

  const isDev = process.env.NODE_ENV === 'development'
  const indexHtml = join(__dirname, '../renderer/index.html')

  if (isDev) {
    mainWindow.loadURL('http://localhost:5173')
    mainWindow.webContents.openDevTools()
  } else {
    mainWindow.loadFile(indexHtml)
  }

  mainWindow.on('closed', () => {
    mainWindow = null
  })

  registerIpcHandlers(mainWindow)
}

app.on('ready', () => {
  setupAutoUpdater()
  sidecar.start()
  createWindow()
})

app.on('window-all-closed', () => {
  sidecar.stop()
  if (process.platform !== 'darwin') {
    app.quit()
  }
})

app.on('activate', () => {
  if (mainWindow === null) {
    createWindow()
  }
})
