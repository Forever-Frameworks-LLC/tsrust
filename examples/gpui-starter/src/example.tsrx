import { View, Text, Node, Child, Link, Markdown, CopyButton, Separator, Tabs, Tab, signal } from 'std:ui';

interface PageHeaderProps {
  summary: string;
  slug: string;
}

/** A component page's summary and a link to GPUI Kit's own page; the selected nav item names the page. */
export function PageHeader(props: PageHeaderProps): Node {
  return (
    <View className="flex-col gap-3">
      <Text className="text-muted-foreground">{props.summary}</Text>
      <Link href={`https://gpui-kit.com/component/${props.slug}`}>GPUI Kit reference</Link>
      <Separator />
    </View>
  );
}

interface ExampleProps {
  title: string;
  description: string;
  code: string;
  children?: Child[];
}

/** One documented example: a heading, then a card that switches between the live preview and its source, with copy in the card header. */
export function Example(props: ExampleProps): Node {
  const tab = signal(0);
  const code = props.code;
  return (
    <View className="flex-col gap-3">
      <Text className="text-xl font-semibold">{props.title}</Text>
      <Text className="text-muted-foreground">{props.description}</Text>
      <View className="flex-col rounded-lg border border-border">
        <View className="flex-row items-center px-2 py-1 border-b border-border">
          <Tabs variant="segmented" selected={tab.get()} onChange={(index) => { tab.set(index); }}><Tab label="Preview" /><Tab label="Code" /></Tabs>
          <View className="grow" />
          <CopyButton value={clone(code)} />
        </View>
        <View hidden={tab.get() !== 0} className="flex-row flex-wrap items-center gap-4 p-8 min-h-[120px]" children={props.children ?? []} />
        <View hidden={tab.get() !== 1} className="p-2"><Markdown content={`\`\`\`tsx\n${code}\n\`\`\``} /></View>
      </View>
    </View>
  );
}
