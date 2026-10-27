import { createContext, useContext, type ReactNode } from 'react'
import { QueryClientProvider } from '@tanstack/react-query'
import type { ApiSession } from './api'

const ApiContext = createContext<ApiSession['api'] | undefined>(undefined)

export function ApiProvider({ session, children }: { session: ApiSession; children: ReactNode }) {
  return <QueryClientProvider client={session.cache}><ApiContext.Provider value={session.api}>{children}</ApiContext.Provider></QueryClientProvider>
}

export function useApi() {
  const api = useContext(ApiContext)
  if (!api) throw new Error('Render this component inside the starter ApiProvider')
  return api
}
