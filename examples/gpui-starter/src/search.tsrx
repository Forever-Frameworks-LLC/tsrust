import { View, Text, Node, Button, Dialog, DialogContent, Command, CommandItem, CommandEmpty, Kbd, Shortcut, signal, navigate } from 'std:ui';
import { SearchEntry, searchEntries } from './search-index.tsr';

interface Hit {
  entry: int;
  score: int;
}

/** How well one entry matches every query word: titles beat keywords, keywords beat descriptions; 0 means a word is missing. */
function score(entry: ReadOnly<SearchEntry>, words: ReadOnly<string[]>): int {
  const page = entry.page.toLowerCase();
  const section = entry.section.toLowerCase();
  let total = 0;
  for (const word of words) {
    let best = 0;
    if (page.startsWith(word)) {
      best = 12;
    } else if (page.includes(word)) {
      best = 8;
    }

    if (best < 6 && section.includes(word)) {
      best = 6;
    }

    if (best < 4 && (entry.keywords.startsWith(word) || entry.keywords.includes(` ${word}`))) {
      best = 4;
    }

    if (best < 2 && entry.text.includes(word)) {
      best = 2;
    }

    if (best === 0) {
      return 0;
    }

    total += best;
  }
  // Whole pages rank above their examples at equal relevance.
  return entry.section.length === 0 ? total + 1 : total;
}

/** The best matches for a query, most relevant first. */
export function rank(query: string, entries: ReadOnly<SearchEntry[]>, limit: int): int[] {
  const words = query.toLowerCase().split(' ').filter((word: string) => word.length > 0);
  const hits: Hit[] = [];
  if (words.length === 0) {
    return [];
  }

  for (let index = 0; index < entries.length; index += 1) {
    const value = score(entries[index], words);
    if (value > 0) {
      hits.push({
        entry: index, score: value
      });
    }
  }

  hits.sort((left: Hit, right: Hit) => right.score - left.score);
  return hits.slice(0, limit).map((hit: Hit) => hit.entry);
}

/** Command rows for the best matches; each row's value is the page route. */
function results(query: string): Node[] {
  const entries = searchEntries();
  return rank(query, entries, 12).map((index: int) => {
    const hit = entries[index];
    const label = hit.section.length === 0 ? clone(hit.page) : `${hit.page} › ${hit.section}`;
    return <CommandItem value={clone(hit.route)} icon={hit.section.length === 0 ? 'file-text' : 'hash'}>{label}</CommandItem>;
  });
}

/** The title-bar search: a button and ⌘K open a dialog that ranks every page and example as you type. */
export function Search(): Node {
  const open = signal(false);
  const query = signal('');
  return (
    <View className="flex-row items-center">
      <Shortcut command="open-search" keys="secondary-k" onTrigger={() => { open.set(true); }} />
      <Button outline={true} size="small" icon="search" onPress={() => { open.set(true); }}>Search docs</Button>
      <View className="pl-2"><Kbd>secondary-k</Kbd></View>
      <Dialog open={open.get()} onOpenChange={(next) => { open.set(next); }}>
        <DialogContent className="w-[560px]">
          <Command placeholder="Search components, examples and props..." filterable={false} autoFocus={true} query={query.get()} onQuery={(text) => { query.set(text); }} onSelect={(route) => { open.set(false); navigate(route); }} maxHeight={420}>
            {...results(query.get())}
            <CommandEmpty><Text variant="muted">{query.get().length === 0 ? 'Type to search, such as "modal", "spreadsheet" or "loading".' : 'No matches.'}</Text></CommandEmpty>
          </Command>
        </DialogContent>
      </Dialog>
    </View>
  );
}
