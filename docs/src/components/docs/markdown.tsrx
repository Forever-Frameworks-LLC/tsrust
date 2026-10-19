import { View, Text, Node, Child, Link } from 'std:ui';
import { CodeBlock } from './code-block';
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

function inlineContent(page: ReadOnly<DocPage>, catalog: ReadOnly<DocCatalog>, parent: int): Child[] {
  let children: Child[] = [];
  for (let i = parent + 1; i < page.document.nodes.length; i++) {
    const node = page.document.nodes[i];
    if (node.parent < parent) {
      break;
    }

    if (node.parent !== parent) {
      continue;
    }

    if (node.kind === 'text' || node.kind === 'html-literal') {
      children.push(clone(node.text));
    } else if (node.kind === 'soft-break') {
      children.push(' ');
    } else if (node.kind === 'hard-break') {
      children.push('\n');
    } else if (node.kind === 'code') {
      children.push(<Text as="code" className="font-mono text-foreground">{clone(node.text)}</Text>);
    } else if (node.kind === 'strong') {
      children.push(<Text as="strong" children={inlineContent(page,catalog,i)}/>);
    } else if (node.kind === 'emphasis') {
      children.push(<Text as="em" children={inlineContent(page,catalog,i)}/>);
    } else if (node.kind === 'strike') {
      children.push(<Text as="del" children={inlineContent(page,catalog,i)}/>);
    } else if (node.kind === 'link') {
      if (node.destination.startsWith('/')) {
        children.push(<Link href={linkTarget(node.destination,catalog)} className="text-primary" children={inlineContent(page,catalog,i)}/>);
      } else if (node.destination.startsWith('#')) {
        children.push(<Link href={docsPath(page.route)+'#'+page.metadata.id+'-'+node.destination.slice(1)} className="text-primary" children={inlineContent(page,catalog,i)}/>);
      } else {
        children.push(content(page.document.nodes, i) + ' (' + node.destination + ')');
      }
    } else if (node.kind === 'image') {
      children.push('Image: ' + content(page.document.nodes, i) + ' (' + node.destination + ')');
    } else if (node.kind === 'task') {
      children.push(node.checked === true ? '[x] ' : '[ ] ');
    }
  }

  return children;
}

function heading(text: Child[], level: int, id: string): Node {
  if (level === 1) {
    return (<Text id={id} as="h1" className="text-3xl font-semibold leading-snug" children={text}/>);
  }

  if (level === 2) {
    return (<Text id={id} as="h2" className="text-xl font-semibold pt-6" children={text}/>);
  }

  if (level === 3) {
    return (<Text id={id} as="h3" className="text-base font-semibold pt-4" children={text}/>);
  }

  if (level === 4) {
    return (<Text id={id} as="h4" className="font-semibold" children={text}/>);
  }

  if (level === 5) {
    return (<Text id={id} as="h5" className="font-semibold" children={text}/>);
  }

  return (<Text id={id} as="h6" className="font-semibold" children={text}/>);
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
  let inlineItem = false;
  let column = 0;
  for (let i = 0; i < page.document.nodes.length; i++) {
    const node = page.document.nodes[i];
    if (node.parent !== parent) {
      continue;
    }

    if (parent >= 0 && page.document.nodes[parent].kind === 'item' && (node.kind === 'text' || node.kind === 'code' || node.kind === 'strong' || node.kind === 'emphasis' || node.kind === 'strike' || node.kind === 'link' || node.kind === 'image' || node.kind === 'task' || node.kind === 'soft-break' || node.kind === 'hard-break')) {
      if (!inlineItem) {
        blocks.push(<Text className="leading-normal text-muted-foreground" children={inlineContent(page,catalog,parent)}/>);
        inlineItem = true;
      }

      continue;
    }

    const text = content(page.document.nodes, i);
    if (node.kind === 'heading') {
      blocks.push(heading(inlineContent(page, catalog, i), node.level, page.metadata.id + '-' + node.anchor));
    } else if (node.kind === 'paragraph') {
      blocks.push(<Text as="p" className="leading-normal text-muted-foreground" children={inlineContent(page,catalog,i)}/>);
    } else if (node.kind === 'code-block') {
      blocks.push(<CodeBlock code={clone(node.raw)} tokens={clone(node.tokens)} language={clone(node.language)}/>);
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
      const table = page.document.nodes[page.document.nodes[parent].parent];
      let cellStyle = 'text-sm leading-normal text-left';
      if (column < table.alignment.length) {
        if (table.alignment[column] === 'center') {
          cellStyle = 'text-sm leading-normal text-center';
        } else if (table.alignment[column] === 'right') {
          cellStyle = 'text-sm leading-normal text-right';
        }
      }

      column++;
      if (page.document.nodes[parent].kind === 'table-head') {
        blocks.push(<View as="th" className="grow min-w-0 w-1/2 p-3"><Text className={cellStyle} children={inlineContent(page,catalog,i)}/></View>);
      } else {
        blocks.push(<View as="td" className="grow min-w-0 w-1/2 p-3"><Text className={cellStyle} children={inlineContent(page,catalog,i)}/></View>);
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
        blocks.push(<ExampleCard id={page.metadata.id + '-' + id} title="Try the example" description="The preview and source come from the same checked project file." source={clone(example.source)} code={clone(example.code)} tokens={clone(example.tokens)}>{renderExample(clone(id))}</ExampleCard>);
      }
    }
  }

  blocks.push(ApiReference(page, catalog));
  return (<View className="gap-5" children={blocks}/>);
}
