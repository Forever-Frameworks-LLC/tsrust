import { createClient, type Transport } from '../../examples/api-contract/generated/index.js';

declare const transport: Transport;
const api = createClient(transport);
const rows: Promise<{ kind: 'task'; id: number; title: string; completed: boolean }[]> = api.tasks.list({ search: null });
const created: Promise<{ kind: 'task'; id: number; title: string; completed: boolean }> = api.tasks.create({ title: 'Type-checked client' });
const count: Promise<number> = api.math.count(3);
const marker: Promise<'__TSRUST_API_ID__'> = api.meta.marker();
void [rows, created, count, marker];

// @ts-expect-error Only declared backend operations are public.
api.serverOnly();
// @ts-expect-error Required input fields stay required.
api.tasks.create({});
// @ts-expect-error Nullable search does not permit numbers.
api.tasks.list({ search: 3 });
// @ts-expect-error Results preserve their inferred element type.
const wrong: Promise<string[]> = api.tasks.list({ search: null });
void wrong;
