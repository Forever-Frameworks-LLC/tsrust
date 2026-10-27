// Editable desktop bootstrap. No endpoint, credential or IPC setup in components.
import { createApiSession } from '../../../web/src/lib/api';
import { desktopTransport, type DesktopBridge } from '../bridge';
declare global { interface Window { tsrust: DesktopBridge; } }
export const session = createApiSession(desktopTransport(window.tsrust), 'your desktop service');
export const api = session.api;
