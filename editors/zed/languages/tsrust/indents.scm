(_ "{" "}" @end) @indent
(_ "[" "]" @end) @indent
(_ "(" ")" @end) @indent
(type_arguments "<" ">" @end) @indent
(type_parameters "<" ">" @end) @indent
(jsx_element (jsx_closing_element) @end) @indent
(if_statement consequence: (statement_block)? @end) @indent
(else_clause (statement_block)? @end) @indent
(for_statement body: (statement_block)? @end) @indent
(for_in_statement body: (statement_block)? @end) @indent
(while_statement body: (statement_block)? @end) @indent
