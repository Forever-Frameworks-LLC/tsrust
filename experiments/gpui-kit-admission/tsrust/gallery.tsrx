import {
  View, Text, Button, Input, Node, signal, run,
  Badge, Tag, Label, Kbd, Separator, Skeleton, Spinner, Progress, ProgressCircle, Avatar, Alert, ShimmerText, Empty, GroupBox,
  DescriptionList, DescriptionItem, Breadcrumb, BreadcrumbItem, Clipboard, Checkbox, Switch, RadioGroup, Radio, Toggle, Rating,
  Pagination, Slider, Stepper, StepperItem, Tabs, Tab, Accordion, AccordionItem, Collapsible, Select, NumberInput, OtpInput,
  DatePicker, ColorPicker, Tooltip, Popover, PopoverTrigger, PopoverContent, HoverCard, HoverCardTrigger, HoverCardContent,
  Dialog, Sheet, DropdownMenu, MenuItem, ContextMenu, Sidebar, SidebarGroup, SidebarItem, Resizable, ResizablePanel,
  Table, TableHeader, TableBody, TableRow, TableHead, TableCell
} from 'std:ui';

function Gallery(): Node {
  const accepted = signal(false);
  const dark = signal(true);
  const size = signal(0);
  const bold = signal(false);
  const stars = signal(3);
  const page = signal(2);
  const volume = signal(40.0);
  const tab = signal(0);
  const color = signal('Green');
  const quantity = signal(2.0);
  const code = signal('');
  const due = signal('2026-11-09');
  const accent = signal('#3b82f6');
  const dialog = signal(false);
  const sheet = signal(false);
  const expanded = signal(true);
  const picked = signal('none');
  return (
    <View className="flex-row size-full">
      <Sidebar>
        <SidebarGroup label="Workspace">
          <SidebarItem label="Components" active={true} />
          <SidebarItem label="Settings" onClick={() => { picked.set('settings'); }} />
        </SidebarGroup>
      </Sidebar>
      <View className="flex-col gap-4 p-6 grow overflow-auto">
        <Breadcrumb><BreadcrumbItem label="Home" /><BreadcrumbItem label="Components" /></Breadcrumb>
        <Tabs selected={tab.get()} onChange={(index) => { tab.set(index); }}><Tab label="Display" /><Tab label="Inputs" /></Tabs>
        <View className="flex-row gap-3 items-center">
          <Badge count={stars.get()}><Text>Inbox</Text></Badge>
          <Tag variant="success">Shipped</Tag>
          <Label text="Email" secondary="(required)" />
          <Kbd keys="cmd-k" />
          <Spinner />
          <ProgressCircle value={75} />
          <Avatar name="Ada Lovelace" />
          <Clipboard value="npm install" />
        </View>
        <Separator label="or" />
        <Progress value={volume.get()} className="w-64" />
        <Skeleton className="w-48 h-4" />
        <ShimmerText text="Thinking..." />
        <Alert variant="warning" title="Heads up" message="Unsaved changes." />
        <Empty title="No projects" description="Create one to get started." />
        <GroupBox title="Account"><Text>Signed in</Text></GroupBox>
        <DescriptionList columns={2}><DescriptionItem label="Plan" value="Pro" /><DescriptionItem label="Region" value="us-east" /></DescriptionList>
        <View className="flex-row gap-4 items-center">
          <Checkbox label="Accept terms" checked={accepted.get()} onChange={(checked) => { accepted.set(checked); }} />
          <Switch label="Dark mode" checked={dark.get()} onChange={(on) => { dark.set(on); }} />
          <Toggle label="Bold" checked={bold.get()} onChange={(on) => { bold.set(on); }} />
        </View>
        <RadioGroup selected={size.get()} onChange={(index) => { size.set(index); }}><Radio label="Small" /><Radio label="Large" /></RadioGroup>
        <Rating value={stars.get()} onChange={(value) => { stars.set(value); }} />
        <Pagination page={page.get()} total={10} onChange={(next) => { page.set(next); }} />
        <Slider value={volume.get()} onChange={(value) => { volume.set(value); }} className="w-64" />
        <Stepper selected={1}><StepperItem label="Account" /><StepperItem label="Billing" /><StepperItem label="Confirm" /></Stepper>
        <Accordion><AccordionItem title="Shipping" open={expanded.get()}><Text>2-3 days</Text></AccordionItem><AccordionItem title="Returns"><Text>30 days</Text></AccordionItem></Accordion>
        <Collapsible open={expanded.get()}><Text>More detail</Text></Collapsible>
        <Select items={['Red', 'Green', 'Blue']} value={color.get()} onChange={(value) => { color.set(value); }} className="w-48" />
        <NumberInput value={quantity.get()} onChange={(value) => { quantity.set(value); }} className="w-48" />
        <OtpInput length={6} onChange={(entered) => { code.set(entered); }} />
        <DatePicker value={due.get()} onChange={(date) => { due.set(date); }} className="w-48" />
        <ColorPicker value={accent.get()} label="Accent" onChange={(chosen) => { accent.set(chosen); }} />
        <View className="flex-row gap-3">
          <Tooltip text="Copy link"><Button>Share</Button></Tooltip>
          <Popover><PopoverTrigger><Button>Filters</Button></PopoverTrigger><PopoverContent><Text>Options</Text></PopoverContent></Popover>
          <HoverCard><HoverCardTrigger><Text>@ada</Text></HoverCardTrigger><HoverCardContent><Text>Mathematician</Text></HoverCardContent></HoverCard>
          <DropdownMenu label="Actions"><MenuItem label="Duplicate" onSelect={() => { picked.set('duplicate'); }} /><MenuItem label="Delete" disabled={true} /></DropdownMenu>
          <Button onPress={() => { dialog.set(true); }}>Open dialog</Button>
          <Button onPress={() => { sheet.set(true); }}>Open sheet</Button>
        </View>
        <ContextMenu><Text>Right-click me</Text><MenuItem label="Copy" onSelect={() => { picked.set('copy'); }} /></ContextMenu>
        <Text>Last action: {picked}</Text>
        <Resizable className="h-32"><ResizablePanel size={200}><Text>Files</Text></ResizablePanel><ResizablePanel><Text>Editor</Text></ResizablePanel></Resizable>
        <Table><TableHeader><TableRow><TableHead>Name</TableHead><TableHead>Role</TableHead></TableRow></TableHeader><TableBody><TableRow><TableCell>Ada</TableCell><TableCell>Engineer</TableCell></TableRow></TableBody></Table>
        <Dialog open={dialog.get()} title="Rename" okText="Save" cancelText="Cancel" onClose={() => { dialog.set(false); }}><Input placeholder="Name" /></Dialog>
        <Sheet open={sheet.get()} title="Details" onClose={() => { sheet.set(false); }}><Text>Sheet body</Text></Sheet>
      </View>
    </View>
  );
}

function main() {
  unwrap(run(Gallery(), 'Tsrust GPUI Kit gallery', {
    width: 1100, height: 800
  }));
}
