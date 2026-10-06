//! Tolerant token extents for virtual scopes around unbraced control bodies.
use super::SourceIndex;
impl SourceIndex {
    pub(super) fn source_statement_end(&self, start: usize, depth: usize) -> usize {
        let last = self.tokens.len().saturating_sub(1);
        if start >= self.tokens.len() || depth >= 128 {
            return last;
        }
        if self.text(start) == "{" {
            return self.pairs[start].unwrap_or(last);
        }
        if matches!(self.text(start), "if" | "for" | "while" | "switch")
            && self.text(start + 1) == "("
        {
            let Some(header) = self.pairs[start + 1] else {
                return last;
            };
            let mut end = self.source_statement_end(header + 1, depth + 1);
            if self.text(start) == "if" && self.text(end + 1) == "else" {
                end = self.source_statement_end(end + 2, depth + 1);
            }
            return end;
        }
        if self.text(start) == "do" {
            let end = self.source_statement_end(start + 1, depth + 1);
            if self.text(end + 1) == "while" && self.text(end + 2) == "(" {
                let close = self.pairs[end + 2].unwrap_or(last);
                return if self.text(close + 1) == ";" {
                    close + 1
                } else {
                    close
                };
            }
            return end;
        }
        if self.text(start) == "try" {
            let mut end = self.source_statement_end(start + 1, depth + 1);
            if self.text(end + 1) == "catch" {
                let body = if self.text(end + 2) == "(" {
                    self.pairs[end + 2].map_or(last, |close| close + 1)
                } else {
                    end + 2
                };
                end = self.source_statement_end(body, depth + 1);
            }
            if self.text(end + 1) == "finally" {
                end = self.source_statement_end(end + 2, depth + 1);
            }
            return end;
        }
        if self.text(start + 1) == ":" {
            return self.source_statement_end(start + 2, depth + 1);
        }
        let mut token = start;
        while token < self.tokens.len() {
            match self.text(token) {
                ";" => return token,
                "}" => return token.saturating_sub(1),
                "(" | "[" | "{" => {
                    if let Some(close) = self.pairs[token] {
                        token = close;
                    } else {
                        return last;
                    }
                }
                _ => {}
            }
            token += 1;
        }
        last
    }
}
