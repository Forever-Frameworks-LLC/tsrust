import ts from './node_modules/typescript/lib/typescript.js';

// Ignore layout only. Raw token spelling retains literal contents and operators.
export function codeTokens(source, retainComments = false) {
    const scanner = ts.createScanner(ts.ScriptTarget.Latest, !retainComments, ts.LanguageVariant.Standard, source);
    const tokens = [], templates = [];
    let braceDepth = 0;
    for (let kind = scanner.scan(); kind !== ts.SyntaxKind.EndOfFileToken; kind = scanner.scan()) {
        if (kind === ts.SyntaxKind.WhitespaceTrivia || kind === ts.SyntaxKind.NewLineTrivia) continue;
        if (kind === ts.SyntaxKind.CloseBraceToken && templates.at(-1) === braceDepth) {
            kind = scanner.reScanTemplateToken(false);
            if (kind === ts.SyntaxKind.TemplateTail) templates.pop();
        } else if (kind === ts.SyntaxKind.OpenBraceToken) {
            braceDepth++;
        } else if (kind === ts.SyntaxKind.CloseBraceToken) {
            braceDepth--;
        } else if (kind === ts.SyntaxKind.TemplateHead) {
            templates.push(braceDepth);
        }
        tokens.push({text: scanner.getTokenText(), start: scanner.getTokenPos(), end: scanner.getTextPos()});
    }
    return tokens;
}

let lastSource, lastTokens;

export function findCodeSnippet(source, snippet) {
    if (!Array.isArray(source) && source !== lastSource) {
        lastSource = source;
        lastTokens = codeTokens(source);
    }
    const tokens = Array.isArray(source) ? source : lastTokens;
    const expected = codeTokens(snippet);
    if (!expected.length) throw new Error('A source contract snippet must contain tokens');
    const matches = [];
    for (let index = 0; index + expected.length <= tokens.length; index++) {
        if (expected.every((token, offset) => token.text === tokens[index + offset].text)) {
            matches.push({start: tokens[index].start, end: tokens[index + expected.length - 1].end});
        }
    }
    return matches;
}

export function containsCodeSnippet(source, snippet) {
    return findCodeSnippet(source, snippet).length > 0;
}

export function replaceCodeSnippet(source, before, after) {
    const match = findCodeSnippet(source, before)[0];
    if (!match) throw new Error('Missing source mutation snippet: ' + before);
    return source.slice(0, match.start) + after + source.slice(match.end);
}
