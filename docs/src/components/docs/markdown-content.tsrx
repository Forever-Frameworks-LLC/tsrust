import { View, Text, Node, Child, Link } from 'std:ui';
import { CodeBlock } from './code-block';
import { DocPage, DocNode, DocCatalog, renderExample, pageId } from '../../../target/docs-content/catalog';
import { ExampleCard } from './example-card';

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
      children.push(inlineText('code', [clone(node.text)]));
    } else if (node.kind === 'strong') {
      children.push(inlineText('strong', inlineContent(page, catalog, i)));
    } else if (node.kind === 'emphasis') {
      children.push(inlineText('em', inlineContent(page, catalog, i)));
    } else if (node.kind === 'strike') {
      children.push(inlineText('del', inlineContent(page, catalog, i)));
    } else if (node.kind === 'link') {
      if (node.destination.startsWith('/')) {
        children.push(inlineLink(linkTarget(node.destination, catalog), inlineContent(page, catalog, i)));
      } else if (node.destination.startsWith('https://')) {
        children.push(inlineLink(clone(node.destination), inlineContent(page, catalog, i)));
      } else if (node.destination.startsWith('#')) {
        children.push(inlineLink(docsPath(page.route) + '#' + page.metadata.id + '-' + node.destination.slice(1), inlineContent(page, catalog, i)));
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
    return (<Text id={id} as="h1" className="text-3xl font-semibold leading-snug outline-none" children={text}/>);
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

  return docsPath(parts[0]) + '#' + pageId(parts[0]) + '-' + parts[1];
}

function appendExamples(page: ReadOnly<DocPage>, catalog: ReadOnly<DocCatalog>, blocks: Mutable<Child[]>): void {
  for (const id of page.metadata.examples) {
    for (const example of catalog.examples) {
      if (id === example.id) {
        blocks.push(<ExampleCard id={page.metadata.id + '-' + id} title={id.split('-').join(' ')} source={clone(example.source)} code={clone(example.code)} tokens={clone(example.tokens)}>{renderExample(clone(id))}</ExampleCard>);
      }
    }
  }

}

// Keep JSX construction outside recursive dispatch so inactive kinds do not retain props frames.
function inlineText(tag: 'code' | 'strong' | 'em' | 'del', children: Child[]): Node {
  const classes = tag === 'code' ? 'font-mono text-foreground' : '';
  return (<Text as={tag} className={classes} children={children}/>);
}

function inlineLink(href: string, children: Child[]): Node {
  return (<Link href={href} className="text-primary" children={children}/>);
}

function textBlock(children: Child[], tag: 'span' | 'p' | 'code', classes: string): Node {
  return (<Text as={tag} className={classes} children={children}/>);
}

function codeBlock(node: ReadOnly<DocNode>): Node {
  return (<CodeBlock code={clone(node.raw)} tokens={clone(node.tokens)} language={clone(node.language)}/>);
}

function separatorBlock(): Node {
  return (<View role="separator" className="border-t"/>);
}

function containerBlock(node: ReadOnly<DocNode>, children: Child[], marker: string): Node {
  if (node.kind === 'quote') {
    return (<View as="blockquote" className="border-l-2 pl-4 gap-4" children={children}/>);
  }

  if (node.kind === 'list') {
    return (<View as={node.ordinal === null ? 'ul' : 'ol'} className="pl-4 gap-2" children={children}/>);
  }

  if (node.kind === 'item') {
    return (<View as="li" className="flex-row gap-3"><Text className="text-muted-foreground">{marker}</Text><View className="grow min-w-0 gap-2" children={children}/></View>);
  }

  if (node.kind === 'table') {
    return (<View as="table" className="border rounded overflow-auto" children={children}/>);
  }

  if (node.kind === 'table-head') {
    return (<View as="thead"><View as="tr" className="flex-row bg-muted font-semibold" children={children}/></View>);
  }

  if (node.kind === 'table-row') {
    return (<View as="tr" className="flex-row border-t" children={children}/>);
  }

  throw 'Invalid Markdown container';
}

function tableCell(page: ReadOnly<DocPage>, catalog: ReadOnly<DocCatalog>, index: int, column: int): Node {
  const node = page.document.nodes[index];
  const parent = page.document.nodes[node.parent];
  const table = page.document.nodes[parent.parent];
  let classes = 'text-sm leading-normal text-left';
  if (column < table.alignment.length) {
    if (table.alignment[column] === 'center') {
      classes = 'text-sm leading-normal text-center';
    } else if (table.alignment[column] === 'right') {
      classes = 'text-sm leading-normal text-right';
    }
  }

  const children = inlineContent(page, catalog, index);
  return (<View as={parent.kind === 'table-head' ? 'th' : 'td'} className="grow min-w-0 w-1/2 p-3"><Text className={classes} children={children}/></View>);
}

function renderBlock(page: ReadOnly<DocPage>, catalog: ReadOnly<DocCatalog>, index: int, listItem: int, column: int): Node | null {
  const node = page.document.nodes[index];
  if (node.kind === 'quote' || node.kind === 'list' || node.kind === 'item' || node.kind === 'table' || node.kind === 'table-head' || node.kind === 'table-row') {
    let marker = '';
    if (node.kind === 'item') {
      const list = page.document.nodes[node.parent];
      marker = list.ordinal === null ? '•' : String(list.ordinal + listItem) + '.';
    }

    const children = renderBlocks(page, catalog, index);
    return containerBlock(node, children, marker);
  }

  if (node.kind === 'table-cell') {
    return tableCell(page, catalog, index, column);
  }

  if (node.kind === 'heading') {
    return heading(inlineContent(page, catalog, index), node.level, page.metadata.id + '-' + node.anchor);
  }

  if (node.kind === 'paragraph') {
    return textBlock(inlineContent(page, catalog, index), 'p', 'leading-normal text-muted-foreground');
  }

  if (node.kind === 'code-block') {
    return codeBlock(node);
  }

  if (node.kind === 'rule') {
    return separatorBlock();
  }

  if (node.kind === 'html-literal') {
    return textBlock([clone(node.text)], 'code', 'font-mono text-sm');
  }

  if (node.kind === 'task') {
    return textBlock([node.checked === true ? '[x]' : '[ ]'], 'span', '');
  }

  if (node.kind === 'text' || node.kind === 'code') {
    return textBlock([clone(node.text)], 'span', 'leading-normal text-muted-foreground');
  }

  if (node.kind === 'strong' || node.kind === 'emphasis' || node.kind === 'strike' || node.kind === 'link') {
    return textBlock([content(page.document.nodes, index)], 'span', 'leading-normal text-muted-foreground');
  }

  return null;
}

function renderBlocks(page: ReadOnly<DocPage>, catalog: ReadOnly<DocCatalog>, parent: int): Child[] {
  let blocks: Child[] = [];
  let examplesInserted = false;
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
        blocks.push(textBlock(inlineContent(page, catalog, parent), 'span', 'leading-normal text-muted-foreground'));
        inlineItem = true;
      }

      continue;
    }

    if (parent === -1 && !examplesInserted && node.kind === 'heading' && node.level !== 1) {
      appendExamples(page, catalog, blocks);
      examplesInserted = true;
    }

    const block = renderBlock(page, catalog, i, listItem, column);
    if (node.kind === 'item') {
      listItem++;
    }

    if (node.kind === 'table-cell') {
      column++;
    }

    if (block !== null) {
      blocks.push(block);
    }
  }

  if (parent === -1 && !examplesInserted) {
    appendExamples(page, catalog, blocks);
  }

  return blocks;
}

export function MarkdownContent(page: ReadOnly<DocPage>, catalog: ReadOnly<DocCatalog>): Node {
  let blocks = renderBlocks(page, catalog, -1);

  return (<View className="gap-5" children={blocks}/>);
}
