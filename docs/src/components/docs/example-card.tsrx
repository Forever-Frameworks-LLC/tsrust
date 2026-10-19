import { View, Text, Node, Child, signal } from 'std:ui';
import { Button } from '../ui/button';
import { Card, CardHeader, CardTitle, CardDescription, CardContent } from '../ui/card';
import { CodeBlock } from './code-block';
import { DocCodeToken } from '../../../target/docs-content/catalog';

export interface ExampleCardProps {
  id: string;
  title: string;
  description?: string;
  source: string;
  code: string;
  tokens: DocCodeToken[];
  children?: Child[];
}

/** Retains the live example while showing its checked source. Source is selectable read-only text. */
export function ExampleCard(props: ExampleCardProps): Node {
  const showSource = signal(false);
  const previewId = clone(props.id) + '-preview';
  const sourceId = props.id + '-source';
  const sourceLabel = clone(props.title) + ' source';
  const previewLabel = clone(props.title) + ' preview';
  return (
    <Card>
      <CardHeader>
        <CardTitle><Text as="h2" className="text-xl font-semibold">{props.title}</Text></CardTitle>
        {props.description === undefined ? null : <CardDescription>{props.description}</CardDescription>}
      </CardHeader>
      <CardContent className="gap-4">
        <View id={previewId} hidden={showSource.get()} role="region" aria-label={previewLabel} tabIndex={0} className="min-h-32 max-h-96 overflow-auto" children={props.children} />
        <View id={sourceId} hidden={!showSource.get()} className="gap-2">
          <Text className="text-sm font-mono text-muted-foreground">{props.source}</Text>
          <CodeBlock label={sourceLabel} code={props.code} tokens={props.tokens} />
        </View>
        <Button variant="outline" onPress={() => { showSource.set(!showSource.get()); }}>{showSource.get() ? 'Show preview' : 'Show source'}</Button>
      </CardContent>
    </Card>
  );
}
