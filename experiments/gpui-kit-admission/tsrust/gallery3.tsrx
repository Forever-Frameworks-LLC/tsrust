import {
  View, Text, Node, signal, run, Switch, NumberInput, Message,
  InputGroup, InputGroupAddon, InputGroupText, InputGroupButton, Input, Icon, Diff, Marker, Attachment, AttachmentGroup, DataTable, Settings, SettingsPage, SettingsGroup, SettingItem, MessageScroller, Button
} from 'std:ui';

const PATCH = '--- a/src/main.rs\n+++ b/src/main.rs\n@@ -1,4 +1,4 @@\n fn main() {\n-    run();\n+    run_app();\n     exit(0);\n }\n';

function Gallery(): Node {
  const query = signal('tsrust');
  const pressed = signal(0);
  const row = signal(-1);
  const dark = signal(false);
  const size = signal(14.0);
  const removed = signal('none');
  const messages = signal(3);
  return (
    <View className="flex-row size-full gap-4 p-4">
      <View className="flex-col gap-3 w-1/2 min-w-0">
        <InputGroup><InputGroupAddon><Icon name="search" /><InputGroupText>https://</InputGroupText></InputGroupAddon><Input value={query.get()} onChange={(text) => { query.set(clone(text.target.value)); }} /><InputGroupAddon align="inline-end"><InputGroupText>.dev</InputGroupText><InputGroupButton onClick={() => { pressed.set(pressed.get() + 1); }}>Go</InputGroupButton></InputGroupAddon></InputGroup>
        <Text>Query {query} · pressed {pressed}</Text>
        <Marker variant="separator" icon="calendar" text="Yesterday" />
        <Marker variant="border" loading={true} text="Syncing" />
        <AttachmentGroup>
          <Attachment title="report.pdf" description="PDF · 2.4 MB" icon="file-text" onRemove={() => { removed.set('report.pdf'); }} />
          <Attachment title="upload.zip" status="uploading" progress={40} />
          <Attachment title="broken.png" status="failed" onRetry={() => { removed.set('retry'); }} />
        </AttachmentGroup>
        <Text>Attachment action {removed}</Text>
        <DataTable columns={['Name', 'Age', 'City']} rows={JSON.stringify([['Ada', '36', 'London'], ['Alan', '41', 'Wilmslow'], ['Grace', '85', 'Arlington']])} sortable={true} onSelect={(index) => { row.set(index); }} className="h-40" />
        <Text>Selected row {row}</Text>
        <Diff patch={clone(PATCH)} className="h-48" />
      </View>
      <View className="flex-col gap-3 w-1/2 min-w-0">
        <Settings className="h-64">
          <SettingsPage title="General">
            <SettingsGroup title="Appearance">
              <SettingItem title="Dark mode" description="Use the dark theme."><Switch checked={dark.get()} onChange={(on) => { dark.set(on); }} /></SettingItem>
              <SettingItem title="Font size"><NumberInput value={size.get()} onChange={(value) => { size.set(value); }} className="w-32" /></SettingItem>
            </SettingsGroup>
          </SettingsPage>
          <SettingsPage title="Advanced"><SettingsGroup title="Network"><SettingItem title="Offline"><Switch /></SettingItem></SettingsGroup></SettingsPage>
        </Settings>
        <Text>Dark {dark} · size {size}</Text>
        <Button onPress={() => { messages.set(messages.get() + 1); }}>Add message</Button>
        <MessageScroller className="h-48">
          <Message author="Ada"><Text>First</Text></Message>
          <Message author="Alan"><Text>Second</Text></Message>
          <Message author="You" align="end" variant="primary"><Text>Third</Text></Message>
        </MessageScroller>
      </View>
    </View>
  );
}

function main() {
  unwrap(run(Gallery(), 'Tsrust GPUI Kit gallery 3', {
    width: 1200, height: 820
  }));
}
