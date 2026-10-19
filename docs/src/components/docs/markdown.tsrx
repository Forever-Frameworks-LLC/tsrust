import { View, Text, Node, Child, Link } from 'std:ui';
import { Textarea } from '../ui/textarea';
import { DocPage, DocNode, DocCatalog, renderExample } from '../../../target/docs-content/catalog';
import { ExampleCard } from './example-card';
import { ApiReference } from './api-reference';

export function docsPath(path: string): string {
  return path === '/' ? '/docs' : '/docs' + path;
}

export function routePath(path: string): string {
  const parts = path.split('#');
  let route = clone(parts[0]);
  if (route.length > 1 && route.endsWith('/')) {
    route = route.slice(0, -1);
  }

  return route;
}

function content(nodes: ReadOnly<DocNode[]>, index: int): string {
  let value = '';
  for (let i = index + 1; i < nodes.length; i++) {
    const node = nodes[i];
    if (node.parent < index) {
      break;
    }

    if (node.kind === 'text' || node.kind === 'code' || node.kind === 'html-literal') {
      value += node.text;
    } else if (node.kind === 'soft-break') {
      value += ' ';
    } else if (node.kind === 'hard-break') {
      value += '\n';
    } else if (node.kind === 'task') {
      value += node.checked === true ? '[x] ' : '[ ] ';
    }
  }

  return value;
}

function heading(text: string, level: int, id: string): Node {
  if (level === 1) {
    return (<Text id={id} as="h1" className="text-3xl font-semibold leading-snug">{text}</Text>);
  }

  if (level === 2) {
    return (<Text id={id} as="h2" className="text-xl font-semibold pt-6">{text}</Text>);
  }

  if (level === 3) {
    return (<Text id={id} as="h3" className="text-base font-semibold pt-4">{text}</Text>);
  }

  if (level === 4) {
    return (<Text id={id} as="h4" className="font-semibold">{text}</Text>);
  }

  if (level === 5) {
    return (<Text id={id} as="h5" className="font-semibold">{text}</Text>);
  }

  return (<Text id={id} as="h6" className="font-semibold">{text}</Text>);
}

function linkTarget(destination: string, catalog: ReadOnly<DocCatalog>): string {
  const parts = destination.split('#');
  if (parts.length === 1) {
    return docsPath(destination);
  }

  for (const page of catalog.pages) {
    if (page.route === parts[0]) {
      return docsPath(page.route) + '#' + page.metadata.id + '-' + parts[1];
    }
  }

  throw 'Documentation link target is missing';
}

function renderBlocks(page: ReadOnly<DocPage>, catalog: ReadOnly<DocCatalog>, parent: int): Child[] {
  let blocks: Child[] = [];
  let listItem = 0;
  for (let i = 0; i < page.document.nodes.length; i++) {
    const node = page.document.nodes[i];
    if (node.parent !== parent) {
      continue;
    }

    const text = content(page.document.nodes, i);
    if (node.kind === 'heading') {
      blocks.push(heading(text, node.level, page.metadata.id + '-' + node.anchor));
    } else if (node.kind === 'paragraph') {
      blocks.push(<Text as="p" className="leading-normal text-muted-foreground">{text}</Text>);
      for (let j = i + 1; j < page.document.nodes.length; j++) {
        const inline = page.document.nodes[j];
        if (inline.parent < i) {
          break;
        }

        if (inline.kind === 'link') {
          const label = content(page.document.nodes, j);
          if (inline.destination.startsWith('/')) {
            blocks.push(<Link href={linkTarget(inline.destination, catalog)} className="text-sm font-medium py-1">{label} {'↗'}</Link>);
          } else if (inline.destination.startsWith('#')) {
            blocks.push(<Link href={docsPath(page.route) + '#' + page.metadata.id + '-' + inline.destination.slice(1)} className="text-sm font-medium py-1">{label}</Link>);
          } else {
            blocks.push(<Text className="font-mono text-sm">{label}: {clone(inline.destination)}</Text>);
          }
        } else if (inline.kind === 'image') {
          blocks.push(<Text className="text-sm">Image: {content(page.document.nodes, j)} ({clone(inline.destination)})</Text>);
        }
      }
    } else if (node.kind === 'code-block') {
      blocks.push(<View className="rounded-lg border bg-muted p-4 gap-2"><Text className="text-xs font-mono text-muted-foreground">{clone(node.language)}</Text><Textarea aria-label="Code example" value={clone(node.raw)} readOnly className="font-mono text-sm w-full h-48 border-0 bg-transparent"/></View>);
    } else if (node.kind === 'quote') {
      blocks.push(<View as="blockquote" className="border-l-2 pl-4 gap-4" children={renderBlocks(page, catalog, i)}/>);
    } else if (node.kind === 'list') {
      if (node.ordinal === null) {
        blocks.push(<View as="ul" className="pl-4 gap-2" children={renderBlocks(page, catalog, i)}/>);
      } else {
        blocks.push(<View as="ol" className="pl-4 gap-2" children={renderBlocks(page, catalog, i)}/>);
      }
    } else if (node.kind === 'item') {
      const list = page.document.nodes[parent];
      const marker = list.ordinal === null ? '•' : String(list.ordinal + listItem) + '.';
      listItem++;
      blocks.push(<View as="li" className="flex-row gap-3"><Text className="text-muted-foreground">{marker}</Text><View className="grow min-w-0 gap-2" children={renderBlocks(page, catalog, i)}/></View>);
    } else if (node.kind === 'table') {
      blocks.push(<View as="table" className="border rounded overflow-auto" children={renderBlocks(page, catalog, i)}/>);
    } else if (node.kind === 'table-head') {
      blocks.push(<View as="thead"><View as="tr" className="flex-row bg-muted font-semibold" children={renderBlocks(page, catalog, i)}/></View>);
    } else if (node.kind === 'table-row') {
      blocks.push(<View as="tr" className="flex-row border-t" children={renderBlocks(page, catalog, i)}/>);
    } else if (node.kind === 'table-cell') {
      if (page.document.nodes[parent].kind === 'table-head') {
        blocks.push(<View as="th" className="grow min-w-0 w-1/2 p-3"><Text className="text-sm leading-normal">{text}</Text></View>);
      } else {
        blocks.push(<View as="td" className="grow min-w-0 w-1/2 p-3"><Text className="text-sm leading-normal">{text}</Text></View>);
      }
    } else if (node.kind === 'rule') {
      blocks.push(<View role="separator" className="border-t"/>);
    } else if (node.kind === 'html-literal') {
      blocks.push(<Text as="code" className="font-mono text-sm">{clone(node.text)}</Text>);
    } else if (node.kind === 'task') {
      blocks.push(<Text>{node.checked === true ? '[x]' : '[ ]'}</Text>);
    } else if (node.kind === 'text' || node.kind === 'code') {
      blocks.push(<Text className="leading-normal text-muted-foreground">{clone(node.text)}</Text>);
    } else if (node.kind === 'strong' || node.kind === 'emphasis' || node.kind === 'strike' || node.kind === 'link') {
      blocks.push(<Text className="leading-normal text-muted-foreground">{text}</Text>);
    }
  }

  return blocks;
}

export function Markdown(page: ReadOnly<DocPage>, catalog: ReadOnly<DocCatalog>): Node {
  let blocks = renderBlocks(page, catalog, -1);
  for (const id of page.metadata.examples) {
    for (const example of catalog.examples) {
      if (id === example.id) {
        blocks.push(<ExampleCard id={page.metadata.id + '-' + id} title="Try the example" description="The preview and source come from the same checked project file." source={clone(example.source)} code={clone(example.code)}>{renderExample(clone(id))}</ExampleCard>);
      }
    }
  }

  blocks.push(ApiReference(page, catalog));
  return (<View className="gap-5" children={blocks}/>);
}
