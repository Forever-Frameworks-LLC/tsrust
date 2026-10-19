import { View, Text, Node, Child, signal } from 'std:ui';
import { Button } from '../ui/button';
import { Card, CardHeader, CardTitle, CardDescription, CardContent } from '../ui/card';
import { Textarea } from '../ui/textarea';

export interface ExampleCardProps {
  id: string;
  title: string;
  description: string;
  source: string;
  code: string;
  children?: Child[];
}

/** Retains the live example while showing its checked source. Source is selectable read-only text. */
export function ExampleCard(props: ExampleCardProps): Node {
  const showSource = signal(false);
  const previewId = clone(props.id) + '-preview';
  const sourceId = props.id + '-source';
  const sourceLabel = clone(props.title) + ' source';
  return (
    <Card>
      <CardHeader>
        <CardTitle><Text as="h2" className="text-xl font-semibold">{props.title}</Text></CardTitle>
        <CardDescription>{props.description}</CardDescription>
      </CardHeader>
      <CardContent className="gap-4">
        <View id={previewId} hidden={showSource.get()} className="h-64 overflow-auto">{props.children}</View>
        <View id={sourceId} hidden={!showSource.get()} className="gap-2">
          <Text className="text-sm font-mono text-muted-foreground">{props.source}</Text>
          <Textarea aria-label={sourceLabel} value={props.code} readOnly className="w-full h-64 font-mono text-sm" />
          <Text className="text-sm text-muted-foreground">Read-only source. Select text and use your platform's copy shortcut. Syntax colors and a dedicated copy action are still pending.</Text>
        </View>
        <Button variant="outline" onPress={() => { showSource.set(!showSource.get()); }}>{showSource.get() ? 'Show preview' : 'Show source'}</Button>
      </CardContent>
    </Card>
  );
}
