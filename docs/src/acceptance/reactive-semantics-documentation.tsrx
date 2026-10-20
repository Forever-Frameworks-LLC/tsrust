import { Button, Node, signal, document } from 'std:ui';

function Page(): Node {
  const busy = signal(false);
  return (<Button aria-label={busy.get()?'Saving':'Save'} onPress={()=>{busy.set(!busy.get());}}>Save</Button>);
}

export function verify() {
  const node = Page();
  console.log(document(node, 'Metadata').includes('aria-label="Save"'));
}
