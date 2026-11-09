import { View, Text, Node, Child, Link, Markdown, CopyButton, Separator } from 'std:ui';

interface PageHeaderProps {
  title: string;
  summary: string;
  slug: string;
}

/** A component page heading: name, summary and a link to GPUI Kit's own page. */
export function PageHeader(props: PageHeaderProps): Node {
  return (
    <View className="flex-col gap-3">
      <Text className="text-3xl font-semibold">{props.title}</Text>
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

/** One documented example: its live preview, then the exact source to copy. */
export function Example(props: ExampleProps): Node {
  const code = props.code;
  return (
    <View className="flex-col gap-3">
      <Text className="text-xl font-semibold">{props.title}</Text>
      <Text className="text-muted-foreground">{props.description}</Text>
      <View className="flex-row flex-wrap items-center gap-4 rounded-lg border border-border p-6" children={props.children ?? []} />
      <View className="flex-row items-start gap-2">
        <View className="grow min-w-0"><Markdown content={`\`\`\`tsx\n${code}\n\`\`\``} /></View>
        <CopyButton value={clone(code)} />
      </View>
    </View>
  );
}
