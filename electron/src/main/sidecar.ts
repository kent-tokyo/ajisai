import { ChildProcess, spawn } from 'child_process'
import { createInterface } from 'readline'
import path from 'path'
import { app } from 'electron'

export class Sidecar {
  private proc: ChildProcess | null = null
  private pendingRequests = new Map<number, {
    resolve: (v: unknown) => void
    reject: (e: Error) => void
  }>()
  private notificationHandlers = new Map<string, (params: unknown) => void>()
  private nextId = 1

  start(): void {
    const binaryPath = app.isPackaged
      ? path.join(process.resourcesPath, 'ajisai-server')
      : path.resolve(__dirname, '../../../target/debug/ajisai-server')

    console.log(`Starting Rust sidecar from: ${binaryPath}`)
    this.proc = spawn(binaryPath, [], {
      stdio: ['pipe', 'pipe', 'inherit'],
    })

    const rl = createInterface({ input: this.proc.stdout! })
    rl.on('line', (line) => this.handleLine(line))

    this.proc.on('error', (err) => {
      console.error('Sidecar process error:', err)
    })

    this.proc.on('exit', (code) => {
      console.error(`ajisai-server exited with code ${code}`)
      this.pendingRequests.forEach(({ reject }) =>
        reject(new Error(`Server exited with code ${code}`))
      )
      this.pendingRequests.clear()
    })
  }

  private handleLine(line: string): void {
    try {
      const msg = JSON.parse(line)
      if ('id' in msg) {
        const pending = this.pendingRequests.get(msg.id)
        if (!pending) return
        this.pendingRequests.delete(msg.id)
        if (msg.error) pending.reject(new Error(msg.error.message))
        else pending.resolve(msg.result)
      } else if ('method' in msg) {
        const handler = this.notificationHandlers.get(msg.method)
        handler?.(msg.params)
      }
    } catch (e) {
      console.error('Failed to parse server line:', line, e)
    }
  }

  call<T>(method: string, params: unknown = {}): Promise<T> {
    return new Promise((resolve, reject) => {
      const id = this.nextId++
      this.pendingRequests.set(id, {
        resolve: resolve as any,
        reject
      })
      const line = JSON.stringify({ id, method, params }) + '\n'
      if (!this.proc?.stdin?.write(line)) {
        reject(new Error('Failed to write to sidecar stdin'))
      }
    })
  }

  on(method: string, handler: (params: unknown) => void): void {
    this.notificationHandlers.set(method, handler)
  }

  stop(): void {
    this.proc?.stdin?.end()
    this.proc?.kill()
    this.proc = null
  }
}

export const sidecar = new Sidecar()
