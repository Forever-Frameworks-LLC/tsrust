import { ApiError, manifest, type Client, type Transport, type CallOptions } from '@tsrust-example/tasks-api';

export type NamedBridge<T> = { readonly [K in keyof T]: T[K] extends (...args: never[]) => Promise<unknown>
  ? (input: unknown) => Promise<unknown> : NamedBridge<T[K]> };
export type DesktopBridge = NamedBridge<Client>;

export function operationAt(root: unknown, name: string): (input: unknown) => Promise<unknown> {
  let value = root;
  for (const part of name.split('.')) {
    if (value === null || typeof value !== 'object' || !Object.hasOwn(value, part)) throw new Error('Missing declared desktop operation');
    value = (value as Record<string, unknown>)[part];
  }
  if (typeof value !== 'function') throw new Error('Missing declared desktop operation');
  return value as (input: unknown) => Promise<unknown>;
}

export function desktopTransport(bridge: DesktopBridge): Transport {
  const methods = new Map(manifest.operations.map(operation => [operation.name, operationAt(bridge, operation.name)]));
  return Object.freeze({ async request(operation: string, input: unknown, options?: CallOptions) {
    const invoke = methods.get(operation);
    if (!invoke) throw new ApiError('api-operation', 'Operation is not declared', operation);
    const cancelled = () => new ApiError('api-cancelled', 'Desktop request cancelled; backend completion is unknown', operation);
    const signal = options?.signal;
    if (signal?.aborted) throw cancelled();
    let abort: () => void = () => {};
    try {
      return await Promise.race([invoke(input), new Promise<never>((_, reject) => {
        abort = () => reject(cancelled()); signal?.addEventListener('abort', abort, { once: true });
        if (signal?.aborted) abort();
      })]);
    } finally { signal?.removeEventListener('abort', abort); }
  } });
}
