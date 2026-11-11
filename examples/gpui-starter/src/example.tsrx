import { View, Text, Node, Child, Link, Markdown, CopyButton, Separator, Tabs, Tab, TabsContent } from 'std:ui';

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
  children?: Child[];
}

interface ExampleCodeProps {
  children: string[];
}

/** A documented example composed from its heading and tabbed content. */
export function Example(props: ExampleProps): Node {
  return <View className="flex-col gap-3" children={props.children ?? []} />;
}

export function ExampleTitle(props: ExampleProps): Node {
  return <Text className="text-xl font-semibold" children={props.children ?? []} />;
}

export function ExampleDescription(props: ExampleProps): Node {
  return <Text className="text-muted-foreground" children={props.children ?? []} />;
}

/** Native tabs own selection and retain both panels while switching. */
export function ExampleTabs(props: ExampleProps): Node {
  return (
    <Tabs variant="segmented" defaultValue="preview" className="rounded-lg border border-border">
      <Tab value="preview">Preview</Tab>
      <Tab value="code">Code</Tab>
      {...props.children ?? []}
    </Tabs>
  );
}

export function ExamplePreview(props: ExampleProps): Node {
  return (
    <TabsContent value="preview">
      <View className="flex-row flex-wrap items-center gap-4 p-8 min-h-[120px]" children={props.children ?? []} />
    </TabsContent>
  );
}

export function ExampleCode(props: ExampleCodeProps): Node {
  const code = props.children.join('');
  return (
    <TabsContent value="code">
      <View className="flex-col gap-2 p-2">
        <View className="flex-row justify-end"><CopyButton value={clone(code)} /></View>
        <Markdown content={`\`\`\`tsx\n${code}\n\`\`\``} />
      </View>
    </TabsContent>
  );
}
