// DiskAnalyzer — preload script.
// Per 07-ELECTRON-APP-SHELL.md §7.3.
//
// The preload is the ONLY bridge between the renderer and the main/Rust/LicenseClient.
// Every method is typed against @disk-analyzer/ipc's DiskAnalyzerAPI.

import { contextBridge, ipcRenderer } from 'electron';

const api = {
  scan: {
    start: (req: any) => ipcRenderer.invoke('scan:start', req),
    cancel: (_scanId: string) => ipcRenderer.invoke('scan:cancel'),
    status: (scanId: string) => ipcRenderer.invoke('scan:status', scanId),
    onProgress: (cb: (e: any) => void) => {
      const l = (_: unknown, e: any) => cb(e);
      ipcRenderer.on('scan:progress', l);
      return () => ipcRenderer.removeListener('scan:progress', l);
    },
    onBatch: (cb: (e: any) => void) => {
      const l = (_: unknown, e: any) => cb(e);
      ipcRenderer.on('scan:batch', l);
      return () => ipcRenderer.removeListener('scan:batch', l);
    },
    onComplete: (cb: (e: any) => void) => {
      const l = (_: unknown, e: any) => cb(e);
      ipcRenderer.on('scan:complete', l);
      return () => ipcRenderer.removeListener('scan:complete', l);
    },
    onError: (cb: (e: any) => void) => {
      const l = (_: unknown, e: any) => cb(e);
      ipcRenderer.on('scan:error', l);
      return () => ipcRenderer.removeListener('scan:error', l);
    },
  },

  license: {
    activate: (_key: string) => Promise.resolve({ ok: false, error: 'NOT_IMPLEMENTED' } as any),
    deactivate: () => Promise.resolve(),
    status: () => Promise.resolve({ kind: 'free' } as any),
    onChange: (_cb: (s: any) => void) => () => {},
  },

  db: {
    queryFiles: (_filter: any) => Promise.resolve([]),
    queryHistory: (_scanId: string) => Promise.resolve(null),
    listScans: (_limit: number) => Promise.resolve([]),
  },

  volumes: {
    list: () => Promise.resolve([]),
  },

  app: {
    openExternal: (url: string) => ipcRenderer.invoke('app:openExternal', url),
    showItemInFolder: (_path: string) => Promise.resolve(),
    moveToTrash: (_paths: string[]) => Promise.resolve({ moved: [], failed: [] }),
    checkForUpdates: () => Promise.resolve(null),
    quitAndInstall: () => Promise.resolve(),
    getVersion: () => ipcRenderer.sendSync('app:getVersion') as string,
    getPlatform: () => process.platform as 'win32' | 'darwin' | 'linux',
  },

  cleanup: {
    add: (_items: any[]) => Promise.resolve(),
    remove: (_ids: string[]) => Promise.resolve(),
    list: () => Promise.resolve([]),
    commit: () =>
      Promise.resolve({ moved: [], failedInUse: [], failedOther: [], totalFreedBytes: 0n }),
    onChange: (_cb: (items: any[]) => void) => () => {},
  },
} as const;

contextBridge.exposeInMainWorld('diskanalyzer', api);

export type DiskAnalyzerAPI = typeof api;
