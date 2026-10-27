import { app, BrowserWindow, ipcMain, net, protocol, session, Menu } from 'electron';
import { mkdir, realpath, stat } from 'node:fs/promises';
import { mkdirSync } from 'node:fs';
import path from 'node:path';
import { pathToFileURL } from 'node:url';
import { ApiError, manifest } from '@tsrust-example/tasks-api';
import { startBackend } from './backend';
import { operationAt } from './bridge';
import { desktopApp } from './lib/app-config';

protocol.registerSchemesAsPrivileged([{ scheme: 'tsrust', privileges: { standard: true, secure: true, supportFetchAPI: true } }]);
app.enableSandbox();
if (process.env.TSRUST_DATA_DIR) {
  const directory = path.resolve(process.env.TSRUST_DATA_DIR); mkdirSync(directory, { recursive: true }); app.setPath('userData', directory);
}
const hidden = process.env.TSRUST_HEADLESS === '1';
let window: BrowserWindow | undefined;
let backend: Awaited<ReturnType<typeof startBackend>> | undefined;
let quitting = false, quitAllowed = false, active = 0;
const productionUrl = 'tsrust://tasks/index.html';
const rendererUrl = MAIN_WINDOW_VITE_DEV_SERVER_URL || productionUrl;
const policy = `default-src 'none'; script-src 'self'${app.isPackaged ? '' : " 'unsafe-inline'"}; style-src 'self' 'unsafe-inline'; font-src 'self'; img-src 'self' data:; connect-src ${app.isPackaged ? "'none'" : "'self' ws://127.0.0.1:5183"}; frame-src 'none'; object-src 'none'; base-uri 'none'; form-action 'none'`;
const failure = (code: string, message: string) => ({ ok: false, error: { code, message } });

function protectWindow(target: BrowserWindow) {
  target.webContents.setWindowOpenHandler(() => ({ action: 'deny' }));
  target.webContents.on('will-navigate', event => event.preventDefault());
  target.webContents.on('will-redirect', event => event.preventDefault());
  target.webContents.on('will-attach-webview', event => event.preventDefault());
}

async function showFailure(message: string) {
  const previous = window;
  window = new BrowserWindow({ width: 800, height: 600, title: desktopApp.title, show: !hidden,
    webPreferences: { sandbox: true, contextIsolation: true, nodeIntegration: false } });
  protectWindow(window);
  if (previous && !previous.isDestroyed()) previous.destroy();
  const html = `<!doctype html><meta http-equiv="Content-Security-Policy" content="default-src 'none'; style-src 'unsafe-inline'"><title>Service unavailable</title><style>body{font:18px system-ui;margin:4rem;max-width:40rem}</style><h1>Service unavailable</h1><p>${message}</p><p>Close the app and reopen it to try again.</p>`;
  await window.loadURL('data:text/html;charset=utf-8,' + encodeURIComponent(html));
}

async function assets() {
  const root = await realpath(path.join(__dirname, '../renderer', MAIN_WINDOW_VITE_NAME));
  protocol.handle('tsrust', async request => {
    try {
      const url = new URL(request.url);
      if (url.hostname !== 'tasks' || !['GET', 'HEAD'].includes(request.method)) return new Response(null, { status: 404 });
      const pathname = decodeURIComponent(url.pathname);
      if (pathname.includes('\\') || pathname.includes('\0') || pathname.includes(':') || pathname.split('/').some(part => part === '..' || part.startsWith('.'))) return new Response(null, { status: 400 });
      const file = await realpath(path.resolve(root, '.' + pathname));
      if (!file.startsWith(root + path.sep) || !(await stat(file)).isFile()) return new Response(null, { status: 404 });
      const response = await net.fetch(pathToFileURL(file).href);
      const headers = new Headers(response.headers); headers.set('Content-Security-Policy', policy); headers.set('X-Content-Type-Options', 'nosniff');
      return new Response(request.method === 'HEAD' ? null : response.body, { status: response.status, headers });
    } catch { return new Response(null, { status: 404 }); }
  });
}

app.on('before-quit', event => {
  if (quitAllowed) return;
  event.preventDefault();
  if (quitting) return;
  quitting = true;
  void (async () => {
    try { await backend?.stop(); }
    catch { console.error('Desktop backend required forced termination; completion may be unknown'); }
    finally { quitAllowed = true; app.quit(); }
  })();
});
app.on('window-all-closed', () => app.quit());

void app.whenReady().then(async () => {
  if (process.platform !== 'darwin') Menu.setApplicationMenu(null);
  session.defaultSession.setPermissionRequestHandler((_contents, _permission, callback) => callback(false));
  session.defaultSession.setPermissionCheckHandler(() => false);
  session.defaultSession.webRequest.onHeadersReceived((details, callback) => callback({ responseHeaders: {
    ...details.responseHeaders, 'Content-Security-Policy': [policy], 'X-Content-Type-Options': ['nosniff'],
  } }));
  const resources = app.isPackaged ? path.join(process.resourcesPath, 'backend') : path.resolve(__dirname, '../../resources/backend');
  const data = app.getPath('userData'); await mkdir(data, { recursive: true });
  try {
    backend = await startBackend({ executable: path.join(resources, `tasks-backend${process.platform === 'win32' ? '.exe' : ''}`),
      database: path.join(data, 'tasks.sqlite'), migrations: path.join(resources, 'migrations') });
    console.log(JSON.stringify({ kind: 'desktop-backend-ready', pid: backend.pid }));
    if (!MAIN_WINDOW_VITE_DEV_SERVER_URL) await assets();
    window = new BrowserWindow({ width: 1100, height: 850, title: desktopApp.title, show: !hidden,
      // Leave the header's bottom border below the native control overlay.
      titleBarStyle: 'hidden', ...(process.platform !== 'darwin' ? { titleBarOverlay: { ...desktopApp.titleBar, height: desktopApp.titleBar.height - 1 } } : {}),
      webPreferences: { preload: path.join(__dirname, 'preload.cjs'), sandbox: true,
        contextIsolation: true, nodeIntegration: false, webSecurity: true, webviewTag: false, backgroundThrottling: !hidden } });
    protectWindow(window);
    for (const operation of manifest.operations) {
      const invoke = operationAt(backend.api, operation.name);
      ipcMain.handle(`tsrust.api.${operation.name}`, async (event, input: unknown) => {
        if (!window || window.isDestroyed() || event.sender !== window.webContents ||
            event.senderFrame !== window.webContents.mainFrame || event.senderFrame.url !== rendererUrl || quitting) {
          return failure('api-denied', 'Access denied');
        }
        if (active >= 16) return failure('api-busy', 'Too many active desktop requests');
        active++;
        try {
          if (Buffer.byteLength(JSON.stringify({ operation: operation.name, input })) > 65536) return failure('api-request-limit', 'API request is too large');
          return { ok: true, value: await invoke(input) };
        } catch (error) {
          return error instanceof ApiError ? failure(error.code, error.message) : failure('api-desktop', 'The desktop service could not complete this call');
        } finally { active--; }
      });
    }
    await window.loadURL(rendererUrl);
    void backend.exited.then(async () => {
      if (!quitting) { console.error('Desktop backend stopped unexpectedly'); await showFailure('The local service stopped. Your last operation may not have completed.'); }
    }).catch(() => app.quit());
  } catch {
    try { await backend?.stop(); } catch { /* stop already performs bounded termination */ }
    await showFailure('The local service could not start.');
  }
}).catch(() => { console.error('Desktop initialization failed'); app.exit(1); });
