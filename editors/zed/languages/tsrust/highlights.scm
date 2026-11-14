(comment) @comment
(string) @string
(template_string) @string
(number) @number
(type_identifier) @type
(predefined_type) @type.builtin
(property_identifier) @property
(identifier) @variable
(call_expression function: (identifier) @function)
(function_declaration name: (identifier) @function)
["import" "export" "from" "function" "return" "const" "let" "if" "else" "for" "while" "try" "catch" "finally" "throw" "async" "await" "interface" "type"] @keyword
[(true) (false)] @boolean
"match" @keyword
(wildcard_pattern) @variable.special
(match_pattern variant: (identifier) @constructor)
(match_pattern binding: (identifier) @variable)
((type_identifier) @type.builtin
 (#any-of? @type.builtin "int" "Task" "Result" "Function" "ReadOnly" "Mutable" "Owned" "Once"))
["break" "continue" "switch" "case" "default" "new" "delete" "typeof" "instanceof" "in" "of" "void"] @keyword
["(" ")" "[" "]" "{" "}"] @punctuation.bracket
[";" "," "."] @punctuation.delimiter
["=>" "=" "+" "-" "*" "/" "%" "==" "===" "!=" "!==" "<" ">" "<=" ">=" "&&" "||" "??" "!"] @operator
(jsx_opening_element name: (identifier) @tag)
(jsx_closing_element name: (identifier) @tag)
(jsx_self_closing_element name: (identifier) @tag)
(jsx_attribute (property_identifier) @attribute)
