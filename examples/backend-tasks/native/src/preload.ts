import { contextBridge, ipcRenderer } from 'electron';
import { manifest } from '@tsrust-example/tasks-api';

// Only checked named capabilities cross the context boundary. No IPC object,
// credential, endpoint, filesystem or process primitive reaches the renderer.
const bridge = Object.create(null);
for (const operation of manifest.operations) {
  const names = operation.name.split('.'); const leaf = names.pop()!;
  let namespace = bridge;
  for (const name of names) namespace = namespace[name] ??= Object.create(null);
  namespace[leaf] = async (input: unknown) => {
    try { return await ipcRenderer.invoke(`tsrust.api.${operation.name}`, input); }
    catch { return { ok: false, error: { code: 'api-desktop', message: 'The desktop service could not complete this call' } }; }
  };
}
contextBridge.exposeInMainWorld('tsrust', bridge);
