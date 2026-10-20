import { View, Text, Link, Svg, Path, Node, signal, withTheme, ThemeConfig } from 'std:ui';
import { Empty, EmptyHeader, EmptyMedia, EmptyTitle, EmptyDescription, EmptyContent } from '../components/ui/empty';
import { Button } from '../components/ui/button-v3';
import config from '../../tsrust.config.json';

function Folder(): Node {
  return (<Svg fill="none" stroke="currentColor" strokeWidth={2} viewBox="0 0 24 24"><Path d="M3 7V5H9L11 7H21V19H3Z"/></Svg>);
}

export function EmptyProjects(): Node {
  const created = signal(false);
  return (
    <View id="empty-projects" className="gap-4">
      {created.get() ? <View className="rounded-xl border p-6 gap-2"><Text as="h3" className="font-medium">Untitled project</Text><Text className="text-sm text-muted-foreground">Your workspace is ready.</Text></View> : <Empty className="border"><EmptyHeader><EmptyMedia variant="icon"><Folder/></EmptyMedia><EmptyTitle as="h3">No projects yet</EmptyTitle><EmptyDescription>Create a project to keep your work in one place. <Link href="/docs">Read the guide</Link></EmptyDescription></EmptyHeader><EmptyContent><Button id="empty-create" onPress={() => { created.set(true); }}>Create project</Button></EmptyContent></Empty>}
      <Button id="empty-reset" variant="outline" onPress={() => { created.set(false); }}>Reset example</Button>
    </View>
  );
}

export function EmptyMediaStates(): Node {
  const stage = signal(0);
  const large = signal(false);
  return (
    <View id="empty-media-states" className="gap-4">
      <Empty id="empty-live" className="border">
        <EmptyHeader>
          <EmptyMedia id="empty-live-media" variant={stage.get() === 0 ? 'icon' : stage.get() === 1 ? null : 'default'} className="w-auto gap-2 px-2">
            <Svg id="empty-default-icon" fill="none" stroke="currentColor" strokeWidth={2}><Path d="M4 12L9 17L20 6"/></Svg>
            <Svg id="empty-explicit-icon" className={large.get() ? 'size-6' : 'size-4'} fill="none" stroke="currentColor" strokeWidth={2}><Path d="M4 4H20V20H4Z"/></Svg>
          </EmptyMedia>
          <EmptyTitle>Media, on your terms</EmptyTitle>
          <EmptyDescription id="empty-description">Variant styles stay separate from explicit icon sizes. <Link id="empty-guide" href="/docs">Read the guide</Link></EmptyDescription>
        </EmptyHeader>
        <EmptyContent><Text id="empty-variant">Variant: {stage.get() === 0 ? 'icon' : stage.get() === 1 ? 'none' : 'default'}</Text><Button id="empty-cycle" onPress={() => { stage.set((stage.get() + 1) % 3); }}>Change media variant</Button><Button id="empty-size" variant="outline" onPress={() => { large.set(!large.get()); }}>Change icon size</Button></EmptyContent>
      </Empty>
    </View>
  );
}

export function EmptyLayouts(): Node {
  return (
    <View id="empty-layouts" className="grid grid-cols-1 md:grid-cols-2 gap-4 items-stretch">
      <Empty className="border"><EmptyHeader><EmptyMedia><Text className="font-mono text-xl">0</Text></EmptyMedia><EmptyTitle>No matches</EmptyTitle><EmptyDescription>Try a shorter search or remove a filter.</EmptyDescription></EmptyHeader></Empty>
      <Empty className="border"><EmptyHeader><EmptyMedia variant="icon"><Folder/></EmptyMedia><EmptyTitle>Your files belong here</EmptyTitle><EmptyDescription>Add your first file to this workspace.</EmptyDescription></EmptyHeader><EmptyContent><Link href="/docs" className="rounded border px-3 py-2 text-sm">Explore documentation</Link></EmptyContent></Empty>
    </View>
  );
}

export function EmptyTheme(): Node {
  const mode = signal('light');
  const theme: ThemeConfig = {
    schemaVersion: 1, light: clone(config.ui.theme.light), dark: clone(config.ui.theme.dark), spacing: {
      sm: 10, md: 20, lg: 30
    }, radius: clone(config.ui.theme.radius), typography: {
      base: 20, xl: 28, headingFamily: 'serif'
    }
  };
  return withTheme(
    <View id="empty-theme" className="gap-4 rounded-xl bg-background text-foreground p-4">
      <Empty className="border"><EmptyHeader><EmptyMedia variant="icon"><Folder/></EmptyMedia><EmptyTitle id="empty-themed-title">A different rhythm</EmptyTitle><EmptyDescription>Larger type, a serif heading and a custom spacing scale.</EmptyDescription></EmptyHeader></Empty>
      <Button id="empty-local-theme" variant="outline" onPress={() => { mode.set(mode.get() === 'light' ? 'dark' : 'light'); }}>Toggle example theme</Button>
    </View>, theme, mode);
}
