import {
  View, Text, Button, Input, Node, signal, run, setTheme, notify, t, tWith, translateWith, setTranslations, setLocale,
  appInfo, checkForUpdates, installUpdate, relaunch,
  Sidebar, SidebarGroup, SidebarItem, Toolbar, ScrollArea, StatusBar, StatusBarLeft, StatusBarRight, Switch, Select,
  Markdown, Badge, Tag, Avatar, Kbd, Alert, Progress, ProgressCircle, Separator, DescriptionList, DescriptionItem,
  Checkbox, RadioGroup, Radio, Rating, Slider, NumberInput, OtpInput, DatePicker, ColorPicker, Stepper, StepperItem, Tabs, Tab,
  Table, TableHeader, TableBody, TableRow, TableHead, TableCell, List, Tree, TreeItem, Calendar, Chart,
  InputGroup, DataTable, Settings, SettingsPage, SettingsGroup, SettingItem, Diff, Marker, Attachment, AttachmentGroup,
  Dialog, Sheet, Popover, PopoverTrigger, PopoverContent, Tooltip, DropdownMenu, MenuItem, ContextMenu, Accordion, AccordionItem
} from 'std:ui';
import en from './locales/en.json';
import fr from './locales/fr.json';

function Overview(): Node {
  return (
    <View className="flex-col gap-4">
      <Text className="text-2xl font-semibold">{t('overview.title')}</Text>
      <Text>{t('overview.body')}</Text>
      <View className="flex-row gap-3 items-center">
        <Avatar name="Ada Lovelace" />
        <Badge count={3}><Text>Inbox</Text></Badge>
        <Tag variant="success">Native</Tag>
        <Kbd keys="cmd-k" />
        <ProgressCircle value={72} />
      </View>
      <Alert variant="info" title="Built with tsrust package" message="Signed, notarized and updated from GitHub releases." />
      <Markdown content={'### Highlights\n\n- Signals drive every component\n- One `.tsrx` file, compiled to a native binary\n- `tsrust package` makes the app, zip and DMG'} />
      <View className="flex-row gap-4 h-48">
        <Chart kind="bar" labels={['Mon', 'Tue', 'Wed', 'Thu', 'Fri']} values={[12, 19, 9, 22, 16]} className="w-72 h-48" />
        <Chart kind="pie" labels={['Rust', 'Kit', 'Tsrust']} values={[50, 30, 20]} className="w-56 h-48" />
      </View>
    </View>
  );
}

function Controls(): Node {
  const accepted = signal(true);
  const size = signal(0);
  const stars = signal(4);
  const volume = signal(60.0);
  const quantity = signal(2.0);
  const color = signal('Green');
  const due = signal('2026-11-09');
  const accent = signal('#f97316');
  const tab = signal(0);
  return (
    <View className="flex-col gap-4">
      <Tabs selected={tab.get()} onChange={(index) => { tab.set(index); }}><Tab label="General" /><Tab label="Advanced" /></Tabs>
      <View className="flex-row gap-4 items-center">
        <Checkbox label="Accept terms" checked={accepted.get()} onChange={(checked) => { accepted.set(checked); }} />
        <RadioGroup selected={size.get()} onChange={(index) => { size.set(index); }}><Radio label="Small" /><Radio label="Large" /></RadioGroup>
      </View>
      <Rating value={stars.get()} onChange={(value) => { stars.set(value); }} />
      <Slider value={volume.get()} onChange={(value) => { volume.set(value); }} className="w-72" />
      <Progress value={volume.get()} className="w-72" />
      <View className="flex-row gap-4 items-center">
        <Select items={['Red', 'Green', 'Blue']} value={color.get()} onChange={(value) => { color.set(clone(value)); }} className="w-48" />
        <NumberInput value={quantity.get()} onChange={(value) => { quantity.set(value); }} className="w-40" />
        <DatePicker value={due.get()} onChange={(date) => { due.set(clone(date)); }} className="w-48" />
      </View>
      <View className="flex-row gap-4 items-center">
        <ColorPicker value={accent.get()} label="Accent" onChange={(chosen) => { accent.set(clone(chosen)); }} />
        <OtpInput length={6} />
        <Input placeholder="Search" />
      </View>
      <Stepper selected={1}><StepperItem label="Account" /><StepperItem label="Billing" /><StepperItem label="Confirm" /></Stepper>
      <Text>Color {color} · due {due} · accent {accent} · {stars} stars</Text>
    </View>
  );
}

function Data(): Node {
  const picked = signal(0);
  const day = signal('2026-11-09');
  const file = signal('none');
  return (
    <View className="flex-col gap-4">
      <Table>
        <TableHeader><TableRow><TableHead>Name</TableHead><TableHead>Role</TableHead><TableHead>Status</TableHead></TableRow></TableHeader>
        <TableBody>
          <TableRow><TableCell>Ada</TableCell><TableCell>Engineer</TableCell><TableCell>Active</TableCell></TableRow>
          <TableRow><TableCell>Grace</TableCell><TableCell>Admiral</TableCell><TableCell>Away</TableCell></TableRow>
        </TableBody>
      </Table>
      <View className="flex-row gap-4">
        <List items={['Inbox', 'Drafts', 'Sent', 'Archive', 'Spam']} selected={picked.get()} onSelect={(index) => { picked.set(index); }} className="w-48 h-48" />
        <Tree onSelect={(id) => { file.set(clone(id)); }} className="w-56 h-48">
          <TreeItem id="src" label="src" expanded={true}><TreeItem id="src/main.tsrx" label="main.tsrx" /><TreeItem id="src/api.tsr" label="api.tsr" /></TreeItem>
          <TreeItem id="locales" label="locales" />
        </Tree>
        <Calendar value={day.get()} onChange={(date) => { day.set(clone(date)); }} />
      </View>
      <Text>Selected file {file} · day {day}</Text>
      <Chart kind="line" labels={['Jan', 'Feb', 'Mar', 'Apr', 'May']} values={[3, 7, 4, 9, 12]} className="w-96 h-48" />
    </View>
  );
}

const PATCH = '--- a/main.tsrx\n+++ b/main.tsrx\n@@ -1,3 +1,3 @@\n function main() {\n-  run(App());\n+  unwrap(run(App(), \'Starter\'));\n }\n';

function Workspace(): Node {
  const query = signal('');
  const row = signal(-1);
  const autosave = signal(true);
  return (
    <View className="flex-col gap-4">
      <InputGroup icon="search" placeholder="Filter people" value={query.get()} onChange={(text) => { query.set(clone(text)); }} />
      <DataTable columns={['Name', 'Role', 'Joined']} rows={[['Ada', 'Engineer', '1843'], ['Grace', 'Admiral', '1944'], ['Alan', 'Mathematician', '1936'], ['Katherine', 'Analyst', '1953']]} sortable={true} onSelect={(index) => { row.set(index); }} className="h-48" />
      <Text>Query "{query}" · selected row {row}</Text>
      <Marker variant="separator" icon="calendar" text="Today" />
      <AttachmentGroup>
        <Attachment title="design.pdf" description="PDF · 1.2 MB" icon="file-text" />
        <Attachment title="assets.zip" status="uploading" progress={65} />
      </AttachmentGroup>
      <Settings className="h-56">
        <SettingsPage title="Editor">
          <SettingsGroup title="Files">
            <SettingItem title="Autosave" description="Save after every change."><Switch checked={autosave.get()} onChange={(on) => { autosave.set(on); }} /></SettingItem>
          </SettingsGroup>
        </SettingsPage>
      </Settings>
      <Diff patch={clone(PATCH)} className="h-40" />
    </View>
  );
}

function Overlays(): Node {
  const dialog = signal(false);
  const sheet = signal(false);
  const last = signal('none');
  return (
    <View className="flex-col gap-4">
      <View className="flex-row gap-3">
        <Button onPress={() => { dialog.set(true); }}>Open dialog</Button>
        <Button onPress={() => { sheet.set(true); }}>Open sheet</Button>
        <Button onPress={() => { notify('Saved', { title: 'Done', variant: 'success' }); }}>Notify</Button>
        <Tooltip text="Copies a link"><Button>Share</Button></Tooltip>
        <Popover><PopoverTrigger><Button>Filters</Button></PopoverTrigger><PopoverContent><Text>Filter options</Text></PopoverContent></Popover>
        <DropdownMenu label="Actions"><MenuItem label="Duplicate" onSelect={() => { last.set('duplicate'); }} /><MenuItem label="Archive" onSelect={() => { last.set('archive'); }} /></DropdownMenu>
      </View>
      <ContextMenu><Text>Right-click here</Text><MenuItem label="Copy" onSelect={() => { last.set('copy'); }} /></ContextMenu>
      <Text>Last action {last}</Text>
      <Accordion><AccordionItem title="Shipping" open={true}><Text>Two to three days.</Text></AccordionItem><AccordionItem title="Returns"><Text>Thirty days.</Text></AccordionItem></Accordion>
      <Dialog open={dialog.get()} title="Rename" okText="Save" cancelText="Cancel" onClose={() => { dialog.set(false); }}><Input placeholder="Name" /></Dialog>
      <Sheet open={sheet.get()} title="Details" onClose={() => { sheet.set(false); }}><Text>Sheet content</Text></Sheet>
    </View>
  );
}

function Languages(): Node {
  const files = signal(1);
  return (
    <View className="flex-col gap-4 items-start">
      <Text className="text-xl font-semibold">{t('languages.title')}</Text>
      <Text>{tWith('languages.greeting', { name: 'Ada' })}</Text>
      <Text>{translateWith('languages.files', { count: files.get() })}</Text>
      <Button onPress={() => { files.set(files.get() + 1); }}>{t('languages.add')}</Button>
      <DatePicker value="2026-11-09" className="w-48" />
    </View>
  );
}

function About(): Node {
  const info = appInfo();
  const status = signal('');
  const available = signal(false);
  const installed = signal(false);
  const packaged = info.version !== '';
  return (
    <View className="flex-col gap-4">
      <DescriptionList columns={2}>
        <DescriptionItem label="Name" value={packaged ? clone(info.name) : 'Unpackaged'} />
        <DescriptionItem label="Version" value={packaged ? clone(info.version) : 'dev'} />
        <DescriptionItem label="Identifier" value={clone(info.id)} />
        <DescriptionItem label="Updates from" value={clone(info.repo)} />
      </DescriptionList>
      <Separator />
      <Text>{status.get() === '' ? translateWith('about.idle', {}) : status.get()}</Text>
      <View className="flex-row gap-3">
        <Button onPress={() => {
          status.set('Checking…');
          checkForUpdates((update) => {
            available.set(update.available);
            status.set(update.error !== '' ? `Could not check: ${update.error}` : update.available ? `Version ${update.latest} is available.` : `You have the latest version (${update.current}).`);
          });
        }}>{t('about.check')}</Button>
        <Button disabled={!available.get()} onPress={() => {
          status.set('Downloading and verifying…');
          installUpdate((error) => {
            if (error === '') {
              installed.set(true);
              status.set('Installed. Restart to use the new version.');
            } else {
              status.set(`Install failed: ${error}`);
            }
          });
        }}>{t('about.install')}</Button>
        <Button disabled={!installed.get()} onPress={() => { relaunch(); }}>{t('about.relaunch')}</Button>
      </View>
    </View>
  );
}

function App(): Node {
  const page = signal('overview');
  const dark = signal(false);
  const language = signal('en');
  return (
    <View className="flex-row size-full">
      <Sidebar>
        <SidebarGroup label={t('nav.group')}>
          <SidebarItem label={t('nav.overview')} active={page.get() === 'overview'} onClick={() => { page.set('overview'); }} />
          <SidebarItem label={t('nav.controls')} active={page.get() === 'controls'} onClick={() => { page.set('controls'); }} />
          <SidebarItem label={t('nav.data')} active={page.get() === 'data'} onClick={() => { page.set('data'); }} />
          <SidebarItem label={t('nav.workspace')} active={page.get() === 'workspace'} onClick={() => { page.set('workspace'); }} />
          <SidebarItem label={t('nav.overlays')} active={page.get() === 'overlays'} onClick={() => { page.set('overlays'); }} />
          <SidebarItem label={t('nav.languages')} active={page.get() === 'languages'} onClick={() => { page.set('languages'); }} />
          <SidebarItem label={t('nav.about')} active={page.get() === 'about'} onClick={() => { page.set('about'); }} />
        </SidebarGroup>
      </Sidebar>
      <View className="flex-col grow min-w-0">
        <Toolbar>
          <Text className="font-semibold">Tsrust Starter</Text>
          <View className="grow" />
          <Select items={['en', 'fr']} value={language.get()} onChange={(value) => { language.set(clone(value)); setLocale(value); }} className="w-24" />
          <Switch label="Dark" checked={dark.get()} onChange={(on) => { dark.set(on); setTheme(on ? 'dark' : 'light'); }} />
        </Toolbar>
        <ScrollArea className="grow">
          <View className="flex-col p-6">
            <View hidden={page.get() !== 'overview'}><Overview /></View>
            <View hidden={page.get() !== 'controls'}><Controls /></View>
            <View hidden={page.get() !== 'data'}><Data /></View>
            <View hidden={page.get() !== 'workspace'}><Workspace /></View>
            <View hidden={page.get() !== 'overlays'}><Overlays /></View>
            <View hidden={page.get() !== 'languages'}><Languages /></View>
            <View hidden={page.get() !== 'about'}><About /></View>
          </View>
        </ScrollArea>
        <StatusBar><StatusBarLeft><Text>Ready</Text></StatusBarLeft><StatusBarRight><Text>{language}</Text></StatusBarRight></StatusBar>
      </View>
    </View>
  );
}

function main() {
  setTranslations('en', en);
  setTranslations('fr', fr);
  setLocale('en');
  unwrap(run(App(), 'Tsrust Starter', {
    width: 1120, height: 780, minWidth: 720, minHeight: 520
  }));
}
