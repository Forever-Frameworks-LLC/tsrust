import { View, Text, Node, Child, Link, CopyButton, signal, Button } from 'std:ui';
import { CodeBlock } from './code-block';
import { loadExample, renderExample } from '../../../target/docs-content/catalog';
import { rustMessage } from '../../examples/rust-message';

interface ChapterProps {
  id: string;
  label: string;
  title: string;
  description: string;
  result: string;
  children?: Child[];
}

/** Visibility affects the result marker only. Reading, focus and scrolling remain unrestricted. */
function Chapter(props: ChapterProps): Node {
  const entered = signal(false);
  return (
    <View id={props.id} as="section" className="border-t py-16 md:py-24 gap-8" onVisibilityChange={(visible) => { if (visible) { entered.set(true); } }}>
      <View className="grid grid-cols-1 lg:grid-cols-2 gap-8 md:gap-16">
        <View className="gap-6">
          <Text className="text-xs font-mono text-primary">{props.label}</Text>
          <Text as="h2" className="text-3xl md:text-4xl font-semibold leading-tight max-w-128">{props.title}</Text>
          <Text className="text-base md:text-xl text-muted-foreground leading-normal max-w-128">{props.description}</Text>
          <View className="h-12 justify-center">
            <View hidden={!entered.get()}><Text data-slot="chapter-result" className="text-sm font-mono text-primary">{props.result}</Text></View>
          </View>
        </View>
        <View className="min-w-0 justify-center gap-4" children={props.children}/>
      </View>
    </View>
  );
}

function source(id: ReadOnly<string>): Node {
  const example = loadExample(id);
  return <CodeBlock code={example.code} tokens={example.tokens} label={example.source}/>;
}

function workspace(): Node {
  const example = loadExample('card-project');
  const sourceVisible = signal(false);
  return (
    <View className="gap-4">
      <View id="home-workspace-preview" hidden={sourceVisible.get()}>{renderExample('card-project')}</View>
      <View id="home-workspace-source" hidden={!sourceVisible.get()}><CodeBlock code={example.code} tokens={example.tokens} label="Project settings source"/></View>
      <Button className="self-start text-sm px-4 py-2 rounded border hover:bg-muted focus:ring" onPress={() => { sourceVisible.set(!sourceVisible.get()); }}>{sourceVisible.get() ? 'Preview' : 'View source'}</Button>
    </View>
  );
}

export function Landing(): Node {
  const started = signal(false);
  return (
    <View className="px-6 md:px-12 lg:px-16">
      <View as="section" className="py-16 md:py-24 gap-12" onVisibilityChange={(visible) => { if (visible) { started.set(true); } }}>
        <View className="flex-row items-center gap-3"><Text className="font-mono text-xs text-primary">TYPESCRIPT SYNTAX / RUST EXECUTION</Text></View>
        <Text as="h1" className="text-3xl md:text-5xl lg:text-7xl font-semibold leading-tight">Write TypeScript.{'\n'}<Text className="text-primary">Ship Native Rust.</Text></Text>
        <View className="grid grid-cols-1 lg:grid-cols-2 gap-8 md:gap-16">
          <View className="gap-8">
            <Text className="text-xl leading-normal text-muted-foreground max-w-128">TypeScript-style source. Checked ownership. Direct Rust imports. Build a service or a shared web and native interface from a project you control.</Text>
            <View className="flex-row flex-wrap gap-3"><Link href="/docs/getting-started/installation" className="px-6 py-3 rounded bg-primary text-primary-foreground focus:ring">Start building {'↗'}</Link><Link href="/docs" className="px-6 py-3 rounded border hover:bg-muted focus:ring">Read the docs</Link></View>
          </View>
          <View className="border rounded-lg bg-card overflow-hidden">
            <View className="px-6 py-3 border-b flex-row justify-between gap-4"><Text className="font-mono text-xs text-muted-foreground">BUILD NATIVE</Text><Text className="font-mono text-xs text-primary">.tsr → Rust</Text></View>
            <View className="p-6 gap-4"><Text as="code" className="font-mono text-sm">$ tsrust build</Text><View className="h-16 justify-center"><View hidden={!started.get()}><Text className="font-mono text-sm text-primary">Called from this page:{'\n'}{rustMessage()}</Text></View></View><Text className="text-xs text-muted-foreground">A Rust-backed executable, with an explicit project entrypoint.</Text></View>
          </View>
        </View>
      </View>
      <Chapter id="familiar" label="SERVICES" title="Readable routes. Native execution." description="Compose handlers and middleware with Router. Keep application policy in application code. Axum handles the HTTP foundation." result="HTTP / Router over Axum">
        {source('native-service')}
        <Link href="/docs/getting-started/project-structure" className="text-sm text-primary py-2 focus:ring">See the project structure {'↗'}</Link>
      </Chapter>
      <Chapter id="rust" label="RUST INTEROP" title="When you need Rust, import Rust." description="Use a checked Rust binding alongside your Tsrust code. Keep native APIs and explicit ownership at the boundary. Both web and desktop compile this example." result="{ .tsr + .rs } / one checked build">
        {source('rust-message')}
        <View className="border-l-2 border-primary pl-4 gap-2"><Text className="font-mono text-xs text-muted-foreground">RUNNING IN THIS APP</Text><Text className="font-mono text-xl">{rustMessage()}</Text></View>
      </Chapter>
      <Chapter id="fabric" label="WEB + NATIVE" title="One interface. Two places to call home." description="Build web and desktop interfaces from the same .tsrx component. Add bundled components offline, edit their files and set the theme in your project configuration." result="WEB + NATIVE / shared component state">
        {workspace()}
      </Chapter>
      <View as="section" className="border-t py-16 md:py-24 gap-8">
        <View className="flex-row items-center justify-between gap-4"><Text className="font-mono text-xs text-primary">LOCAL BY DESIGN</Text><Text className="font-mono text-xs text-muted-foreground">NO REGISTRY REQUEST</Text></View>
        <View className="grid grid-cols-1 md:grid-cols-2 gap-8 md:gap-16">
          <View className="gap-6"><Text as="h2" className="text-3xl md:text-4xl font-semibold leading-tight">Install the source.{'\n'}Make it yours.</Text><Text className="leading-normal text-muted-foreground max-w-128">Components arrive as editable project files. Shared design tokens control light and dark themes. Your changes stay yours when the library moves forward.</Text><Link href="/docs/ui/components/button" className="text-primary py-2 focus:ring">Explore components {'↗'}</Link></View>
          <View className="border rounded-lg bg-muted p-6 gap-6"><View className="flex-row items-center justify-between gap-4"><Text className="font-mono text-xs">OFFLINE COMPONENTS</Text><CopyButton value="tsrust ui add button" className="text-xs rounded border px-3 py-2 focus:ring"/></View><View role="region" aria-label="Offline component commands" tabIndex={0} className="min-w-0 overflow-auto"><Text as="code" className="font-mono text-base">$ tsrust ui add button{'\n\n'}src/components/ui/button.tsrx{'\n'}tsrust.config.json{'\n'}ui-components.lock.json</Text></View></View>
        </View>
      </View>
      <View as="section" className="border-t py-16 md:py-24 gap-8">
        <Text className="font-mono text-xs text-primary">START BUILDING</Text>
        <Text as="h2" className="text-4xl md:text-6xl font-semibold leading-tight max-w-192">Keep your syntax.{'\n'}Choose your foundation.</Text>
        <View className="flex-row flex-wrap gap-3"><Link href="https://github.com/Forever-Frameworks-LLC/tsrust/releases" className="px-6 py-3 rounded bg-primary text-primary-foreground focus:ring">GitHub Releases {'↗'}</Link><Link href="/docs/getting-started/installation" className="px-6 py-3 rounded border hover:bg-muted focus:ring">Build from source</Link></View>
      </View>
      <View as="footer" className="border-t py-6 flex-row flex-wrap justify-between gap-4"><Text className="font-mono text-sm">tsrust.</Text><Text className="text-xs text-muted-foreground">TypeScript-style syntax. Rust execution.</Text></View>
    </View>
  );
}
