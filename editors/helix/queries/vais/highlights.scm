; Comments
(comment) @comment

; Keywords
(
  [
    "F" "S" "E" "I" "L" "M" "W" "T" "X" "V" "C" "R" "B" "N" "A"
    "fn" "struct" "enum" "if" "loop" "match" "while" "trait" "impl" "let" "const" "return" "break" "continue" "async"
  ] @keyword
)

; Built-in types
(
  [
    "i32" "i64" "u32" "u64" "f32" "f64" "bool" "string" "void" "u8" "i8"
  ] @type.builtin
)

; Function calls
(function_call
  name: (identifier) @function)

; Function definitions
(function
  name: (identifier) @function)

; Struct definitions
(struct
  name: (identifier) @type)

; Enum definitions
(enum
  name: (identifier) @type)

; Trait definitions
(trait
  name: (identifier) @type)

; Variables and identifiers
(identifier) @variable

; Numbers
(number) @constant.numeric

; Strings
(string) @string

; Self recursion operator
"@" @operator

; Operators
[
  "+" "-" "*" "/" "%"
  "=" "==" "!=" "<" ">" "<=" ">="
  "&&" "||" "!"
  "&" "|" "^" "<<" ">>"
  "+=" "-=" "*=" "/=" "%=" "&=" "|=" "^="
  "." "," ";" ":" "::" "->" "=>"
] @operator

; Punctuation
[
  "(" ")" "{" "}" "[" "]"
] @punctuation.bracket

; Special tokens
(attribute) @attribute
