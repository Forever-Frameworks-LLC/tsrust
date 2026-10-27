import { useRef, useState, type FormEvent } from 'react'
import { connectApi, defaultApiEndpoint, type ApiSession, type Task } from '@/lib/api'
import { ApiProvider, useApi } from '@/lib/api-provider'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Checkbox } from '@/components/ui/checkbox'
import { Field, FieldDescription, FieldGroup, FieldLabel } from '@/components/ui/field'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert'
import { Empty, EmptyDescription, EmptyHeader, EmptyTitle } from '@/components/ui/empty'

function titleError(title: string) {
  const bytes = new TextEncoder().encode(title.trim()).length
  return bytes < 1 || bytes > 200 ? 'Use 1 to 200 UTF-8 bytes after trimming.' : undefined
}
function Failure({ message }: { message: string }) {
  return <Alert variant="destructive"><AlertTitle>Could not finish</AlertTitle><AlertDescription>{message}</AlertDescription></Alert>
}
function Connect({ onConnect }: { onConnect: (session: ApiSession) => void }) {
  const [endpoint, setEndpoint] = useState(defaultApiEndpoint)
  const [token, setToken] = useState('')
  const [error, setError] = useState<string>()
  function connect(event: FormEvent) {
    event.preventDefault()
    try {
      const session = connectApi({ endpoint, token })
      setToken('')
      onConnect(session)
    } catch (error) { setError(error instanceof Error ? error.message : 'Check the service connection.') }
  }
  return <Card className="mx-auto w-full max-w-lg"><CardHeader><CardTitle>Connect to your tasks</CardTitle><CardDescription>Use the API address and access token supplied by your service.</CardDescription></CardHeader><CardContent>
    <form onSubmit={connect} className="flex flex-col gap-5"><FieldGroup>
      <Field><FieldLabel htmlFor="endpoint">API address</FieldLabel><Input id="endpoint" type="url" required value={endpoint} onChange={event => setEndpoint(event.target.value)} /></Field>
      <Field><FieldLabel htmlFor="token">Access token</FieldLabel><Input id="token" type="password" required autoComplete="off" value={token} onChange={event => setToken(event.target.value)} aria-describedby="token-note" /><FieldDescription id="token-note">Kept in memory for this connection. Never saved in your browser.</FieldDescription></Field>
    </FieldGroup>{error ? <Failure message={error} /> : null}<Button type="submit">Connect</Button></form>
  </CardContent></Card>
}
function TaskRow({ task }: { task: Task }) {
  const api = useApi()
  const update = api.tasks.update.useMutation()
  const remove = api.tasks.delete.useMutation()
  const [editing, setEditing] = useState(false)
  const [deleting, setDeleting] = useState(false)
  const [draft, setDraft] = useState(task.title)
  const [error, setError] = useState<string>()
  const editButton = useRef<HTMLButtonElement>(null)
  const busy = update.isPending || remove.isPending
  async function save(title: string, completed: boolean) {
    const invalid = titleError(title)
    if (invalid) { setError(invalid); return }
    setError(undefined)
    try {
      const result = await update.mutateAsync({ id: task.id, title: title.trim(), completed })
      if (result.kind === 'error') { setError(result.message); return }
      setEditing(false)
      editButton.current?.focus()
    } catch { setError('The update could not be confirmed. Refresh before trying again.') }
  }
  async function erase() {
    setError(undefined)
    try {
      const result = await remove.mutateAsync({ id: task.id })
      if (result.kind === 'error') setError(result.message)
      else document.getElementById('new-title')?.focus()
    } catch { setError('The deletion could not be confirmed. Refresh before trying again.') }
  }
  return <li className="flex flex-col gap-3 border-b py-4 last:border-0" aria-busy={busy}>
    <div className="flex items-center gap-3"><Checkbox id={`done-${task.id}`} checked={task.completed} disabled={busy || editing || deleting} onCheckedChange={checked => void save(task.title, checked)} aria-label={`Mark ${task.title} ${task.completed ? 'incomplete' : 'complete'}`} /><label htmlFor={`done-${task.id}`} className={`min-w-0 flex-1 break-words ${task.completed ? 'text-muted-foreground line-through' : ''}`}>{task.title}</label><Button ref={editButton} variant="ghost" disabled={busy || deleting} onClick={() => { setDraft(task.title); setError(undefined); setEditing(true) }} aria-label={`Edit ${task.title}`}>Edit</Button><Button variant="ghost" disabled={busy || editing} onClick={() => setDeleting(true)} aria-label={`Delete ${task.title}`}>Delete</Button></div>
    {editing ? <form onSubmit={event => { event.preventDefault(); void save(draft, task.completed) }} className="flex flex-col gap-3"><Field data-invalid={!!error}><FieldLabel htmlFor={`edit-${task.id}`}>Task title</FieldLabel><Input id={`edit-${task.id}`} autoFocus value={draft} onChange={event => setDraft(event.target.value)} disabled={busy} aria-invalid={!!error} aria-describedby={error ? `error-${task.id}` : undefined} /></Field><div className="flex gap-2"><Button type="submit" disabled={busy}>{busy ? 'Saving…' : 'Save changes'}</Button><Button type="button" variant="outline" disabled={busy} onClick={() => { setEditing(false); setError(undefined); editButton.current?.focus() }}>Cancel</Button></div></form> : null}
    {deleting ? <div className="flex flex-wrap items-center gap-2"><p>Delete this task permanently?</p><Button variant="destructive" disabled={busy} onClick={() => void erase()}>Confirm delete</Button><Button variant="outline" disabled={busy} onClick={() => setDeleting(false)}>Keep task</Button></div> : null}
    {error ? <p id={`error-${task.id}`} role="alert" className="text-sm text-destructive">{error}</p> : null}
  </li>
}
function TaskList() {
  const api = useApi()
  const [cursors, setCursors] = useState([0])
  const after = cursors[cursors.length - 1]
  const query = api.tasks.list.useQuery({ after, limit: 100 })
  const create = api.tasks.create.useMutation()
  const [title, setTitle] = useState('')
  const [error, setError] = useState<string>()
  const [notice, setNotice] = useState('')
  const input = useRef<HTMLInputElement>(null)
  const page = query.data?.kind === 'page' ? query.data : undefined
  const failure = query.isError ? 'The service could not be reached or access was denied. Check your connection and retry.' : query.data?.kind === 'error' ? query.data.message : undefined
  async function add(event: FormEvent) {
    event.preventDefault()
    const invalid = titleError(title)
    if (invalid) { setError(invalid); input.current?.focus(); return }
    setError(undefined)
    setNotice('')
    try {
      const result = await create.mutateAsync({ title: title.trim() })
      if (result.kind === 'error') { setError(result.message); return }
      setTitle('')
      setCursors([0])
      setNotice('Task added.')
      input.current?.focus()
    } catch { setError('The new task could not be confirmed. Refresh before trying again.') }
  }
  return <Card><CardHeader><CardTitle>Your tasks</CardTitle><CardDescription>A little focus, one task at a time.</CardDescription></CardHeader><CardContent className="flex flex-col gap-6">
    <form onSubmit={event => void add(event)} className="flex flex-col gap-3"><Field data-invalid={!!error}><FieldLabel htmlFor="new-title">New task</FieldLabel><Input ref={input} id="new-title" placeholder="What needs doing?" value={title} onChange={event => setTitle(event.target.value)} disabled={create.isPending} aria-invalid={!!error} aria-describedby={error ? 'create-error title-help' : 'title-help'} /><FieldDescription id="title-help">Up to 200 UTF-8 bytes, including spaces.</FieldDescription></Field>{error ? <p id="create-error" role="alert" className="text-sm text-destructive">{error}</p> : null}<Button type="submit" className="self-start" disabled={create.isPending}>{create.isPending ? 'Adding…' : 'Add task'}</Button></form>
    <p role="status" className="sr-only">{notice || (query.isPending ? 'Loading tasks…' : query.isFetching ? 'Refreshing tasks…' : '')}</p>
    <div className="flex items-center justify-between gap-3"><h2 className="font-medium">Task list</h2><Button variant="outline" disabled={query.isFetching} onClick={() => void query.refetch()}>{query.isFetching ? 'Refreshing…' : 'Refresh'}</Button></div>
    {failure ? <Failure message={failure} /> : null}{query.isPending ? <p>Loading tasks…</p> : null}
    {page?.items.length === 0 ? <Empty><EmptyHeader><EmptyTitle>{after === 0 ? 'A clear start' : 'No more tasks'}</EmptyTitle><EmptyDescription>{after === 0 ? 'Add your first task above.' : 'Go back to the previous page.'}</EmptyDescription></EmptyHeader></Empty> : null}
    {page && page.items.length > 0 ? <ul aria-label="Tasks">{page.items.map(task => <TaskRow key={task.id} task={task} />)}</ul> : null}
    <nav aria-label="Task pages" className="flex items-center justify-between gap-3"><Button variant="outline" disabled={cursors.length === 1 || query.isFetching} onClick={() => setCursors(previous => previous.slice(0, -1))}>Previous page</Button><span className="text-sm text-muted-foreground">Page {cursors.length}</span><Button variant="outline" disabled={!page || page.items.length < 100 || query.isFetching} onClick={() => { if (page?.items.length) setCursors(previous => [...previous, page.items[page.items.length - 1].id]) }}>Next page</Button></nav>
  </CardContent></Card>
}
export default function Tasks({ initialSession }: { initialSession?: ApiSession } = {}) {
  const [session, setSession] = useState<ApiSession | undefined>(initialSession)
  function disconnect() {
    session?.dispose()
    setSession(undefined)
  }
  return <main className="mx-auto flex min-h-svh max-w-3xl flex-col gap-8 px-4 py-10 sm:px-8 sm:py-16"><header className="flex flex-wrap items-center justify-between gap-4"><div><p className="text-sm font-medium text-muted-foreground">Tsrust / Tasks</p><h1 className="mt-2 text-3xl font-semibold tracking-tight">Make room for what matters.</h1></div>{session && !initialSession ? <Button variant="outline" onClick={disconnect}>Disconnect</Button> : null}</header>{session ? <ApiProvider session={session}><p className="break-all text-sm text-muted-foreground">Connected to {session.endpoint}</p><TaskList /></ApiProvider> : <Connect onConnect={setSession} />}<footer className="text-sm text-muted-foreground">Your work, saved by your service.</footer></main>
}
