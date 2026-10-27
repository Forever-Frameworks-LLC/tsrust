// Starter integration: application-owned and editable, like lib/utils.ts.
// API generation updates the imported SDK, never this application's auth policy.
import { QueryClient } from '@tanstack/react-query'
import { createReactClient } from '@tsrust-example/tasks-api/react-query'
import { createHttpTransport } from '@tsrust-example/tasks-api/http'
import type { Client, Transport } from '@tsrust-example/tasks-api'

export type Task = Extract<Awaited<ReturnType<Client['tasks']['create']>>, { kind: 'task' }>
export type ApiSession = ReturnType<typeof createApiSession>

export function createApiSession(transport: Transport, label: string) {
  const cache = new QueryClient()
  // Each authenticated connection gets an independent cache and principal scope.
  const api = createReactClient(transport, { scope: JSON.stringify([label, crypto.randomUUID()]) })
  return { api, cache, endpoint: label, dispose() { void cache.cancelQueries(); cache.clear() } }
}

export function defaultApiEndpoint() {
  return new URL(import.meta.env.VITE_TSRUST_API_URL || '/api', window.location.href).href
}

// This demo uses an application-owned bearer token. Replace this credential
// policy with the app's session/auth provider; never put secrets in VITE_ values.
export function connectApi({ endpoint = defaultApiEndpoint(), token }: { endpoint?: string; token: string }) {
  const url = new URL(endpoint)
  const loopback = ['localhost', '127.0.0.1', '[::1]'].includes(url.hostname)
  if (url.username || url.password || url.hash || url.search || (url.protocol !== 'https:' && !(url.protocol === 'http:' && loopback))) throw new Error('Use an HTTPS API URL, or HTTP on localhost, without credentials, query or fragment.')
  if (!/^[a-f0-9]{64}$/.test(token)) throw new Error('Use the service’s 64-character lowercase hexadecimal access token.')
  return createApiSession(createHttpTransport({ url: url.href, headers: () => ({ authorization: `Bearer ${token}` }) }), url.href)
}
