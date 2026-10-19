import { View, Text, Node, Child, CopyButton } from 'std:ui';
import { DocCodeToken } from '../../../target/docs-content/catalog';

export interface CodeBlockProps {
  code: string;
  tokens: DocCodeToken[];
  language?: string;
  label?: string;
}

function tokenStyle(kind: string): string {
  if (kind === 'comment') {
    return 'text-muted-foreground';
  }

  if (kind === 'keyword') {
    return 'text-syntax-keyword font-semibold';
  }

  if (kind === 'string') {
    return 'text-syntax-string';
  }

  if (kind === 'number') {
    return 'text-syntax-number';
  }

  if (kind === 'type' || kind === 'component') {
    return 'text-syntax-type';
  }

  return 'text-foreground';
}

/** Renders exact compiler lexemes. Copy always uses the unmodified source. */
export function CodeBlock(props: CodeBlockProps): Node {
  let runs: Child[] = [];
  let joined = '';
  for (const token of props.tokens) {
    joined += token.text;
    runs.push(<Text data-slot={clone(token.class)} className={tokenStyle(clone(token.class))}>{clone(token.text)}</Text>);
  }

  if (joined !== props.code) {
    throw 'Code tokens do not match their source';
  }

  return (
    <View className="rounded-lg border bg-muted overflow-hidden">
      <View className="flex-row items-center justify-between border-b px-4 py-2 gap-4">
        <Text className="text-xs font-mono text-muted-foreground">{props.language ?? 'tsrust'}</Text>
        <CopyButton value={props.code} className="text-xs rounded px-3 py-2 hover:bg-background focus:ring" />
      </View>
      <View role="region" aria-label={props.label ?? 'Code example'} tabIndex={0} className="max-h-64 overflow-auto p-4">
        <Text as="code" className="font-mono text-sm leading-normal" children={runs}/>
      </View>
    </View>
  );
}
