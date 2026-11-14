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
