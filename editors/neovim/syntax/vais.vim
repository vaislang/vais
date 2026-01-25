" Vim syntax file
" Language: Vais
" Maintainer: Vais Language Team
" Latest Revision: 2026-01-26

if exists("b:current_syntax")
  finish
endif

" Keywords - Single character keywords
syntax keyword vaisKeyword F S E I L M W T X V C R B N A
syntax keyword vaisKeyword nextgroup=vaisIdentifier skipwhite

" Control flow keywords
syntax keyword vaisControlFlow if else match loop while for break continue return

" Other keywords
syntax keyword vaisOther mut pub use mod as in where impl trait struct enum fn let const
syntax keyword vaisStorage mut pub

" Async keywords
syntax keyword vaisAsync async await

" Types - Numeric types
syntax keyword vaisTypeNumeric i8 i16 i32 i64 u8 u16 u32 u64 f32 f64

" Types - Other built-in types
syntax keyword vaisType bool str Self

" Type names (CamelCase)
syntax match vaisTypeName "\<[A-Z][a-zA-Z0-9_]*\>"

" Boolean constants
syntax keyword vaisBoolean true false

" Language constants
syntax keyword vaisConstant None Some

" Numbers
syntax match vaisNumberHex "\<0x[0-9a-fA-F_]\+\>"
syntax match vaisNumberBin "\<0b[01_]\+\>"
syntax match vaisNumberOct "\<0o[0-7_]\+\>"
syntax match vaisNumberFloat "\<[0-9][0-9_]*\.[0-9][0-9_]*\([eE][+-]\?[0-9_]\+\)\?\>"
syntax match vaisNumberDec "\<[0-9][0-9_]*\>"

" Operators
syntax match vaisOperatorSpecial "@"
syntax match vaisOperatorAssign ":="
syntax match vaisOperatorArrow "->"
syntax match vaisOperatorFatArrow "=>"
syntax match vaisOperatorQuestion "?"
syntax match vaisOperatorComparison "==\|!=\|<\|>\|<=\|>="
syntax match vaisOperatorLogical "&&\|||\|!"
syntax match vaisOperatorArithmetic "+\|-\|*\|/\|%"
syntax match vaisOperatorBitwise "&\||\|\^\|<<\|>>"
syntax match vaisOperatorAssignment "=\|+=\|-=\|*=\|/=\|%=\|&=\||=\|\^=\|<<=\|>>="
syntax match vaisOperatorAccess "\."

" Strings
syntax region vaisString start=+"+ skip=+\\\\\|\\"+ end=+"+ contains=vaisEscape
syntax region vaisString start=+'+ skip=+\\\\\|\\'+ end=+'+ contains=vaisEscape

" String escape sequences
syntax match vaisEscape "\\x[0-9A-Fa-f]\{2\}" contained
syntax match vaisEscape "\\u{[0-9A-Fa-f]\+}" contained
syntax match vaisEscape "\\[0nrt\\'\"]" contained

" Comments
syntax match vaisComment "#.*$"

" Function names
syntax match vaisFunction "\<[a-z_][a-zA-Z0-9_]*\>\ze\s*("

" Built-in functions
syntax keyword vaisBuiltinFunction puts putchar print_i64 malloc free strlen

" Identifiers
syntax match vaisIdentifier "\<[a-z_][a-zA-Z0-9_]*\>"

" Highlighting
highlight default link vaisKeyword Keyword
highlight default link vaisControlFlow Conditional
highlight default link vaisOther Keyword
highlight default link vaisStorage StorageClass
highlight default link vaisAsync Keyword
highlight default link vaisTypeNumeric Type
highlight default link vaisType Type
highlight default link vaisTypeName Type
highlight default link vaisBoolean Boolean
highlight default link vaisConstant Constant
highlight default link vaisNumberHex Number
highlight default link vaisNumberBin Number
highlight default link vaisNumberOct Number
highlight default link vaisNumberFloat Float
highlight default link vaisNumberDec Number
highlight default link vaisOperatorSpecial Operator
highlight default link vaisOperatorAssign Operator
highlight default link vaisOperatorArrow Operator
highlight default link vaisOperatorFatArrow Operator
highlight default link vaisOperatorQuestion Operator
highlight default link vaisOperatorComparison Operator
highlight default link vaisOperatorLogical Operator
highlight default link vaisOperatorArithmetic Operator
highlight default link vaisOperatorBitwise Operator
highlight default link vaisOperatorAssignment Operator
highlight default link vaisOperatorAccess Operator
highlight default link vaisString String
highlight default link vaisEscape SpecialChar
highlight default link vaisComment Comment
highlight default link vaisFunction Function
highlight default link vaisBuiltinFunction Function
highlight default link vaisIdentifier Identifier

let b:current_syntax = "vais"
