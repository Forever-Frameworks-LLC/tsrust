import { View, Node } from 'std:ui';
import { DocPage, DocCatalog } from '../../../target/docs-content/catalog';
import { MarkdownContent, docsPath as contentPath, routePath as contentRoute } from './markdown-content';
import { ApiReference } from './api-reference';

export function docsPath(path: string): string {
  return contentPath(path);
}

export function routePath(path: string): string {
  return contentRoute(path);
}

export function Markdown(page: ReadOnly<DocPage>, catalog: ReadOnly<DocCatalog>): Node {
  return (<View className="gap-5">{MarkdownContent(page,catalog)}{ApiReference(page,catalog)}</View>);
}
