package com.vais.intellij

import com.intellij.lexer.LexerBase
import com.intellij.psi.tree.IElementType

/**
 * Lexer for Vais language.
 *
 * Tokenizes Vais source code for syntax highlighting.
 */
class VaisLexer : LexerBase() {
    private var buffer: CharSequence = ""
    private var bufferEnd: Int = 0
    private var tokenStart: Int = 0
    private var tokenEnd: Int = 0
    private var tokenType: IElementType? = null

    override fun start(buffer: CharSequence, startOffset: Int, endOffset: Int, initialState: Int) {
        this.buffer = buffer
        this.bufferEnd = endOffset
        this.tokenStart = startOffset
        this.tokenEnd = startOffset
        advance()
    }

    override fun getState(): Int = 0

    override fun getTokenType(): IElementType? = tokenType

    override fun getTokenStart(): Int = tokenStart

    override fun getTokenEnd(): Int = tokenEnd

    override fun advance() {
        tokenStart = tokenEnd
        if (tokenStart >= bufferEnd) {
            tokenType = null
            return
        }

        val c = buffer[tokenStart]

        when {
            // Whitespace
            c.isWhitespace() -> {
                tokenEnd = tokenStart + 1
                while (tokenEnd < bufferEnd && buffer[tokenEnd].isWhitespace()) {
                    tokenEnd++
                }
                tokenType = VaisTokenTypes.WHITE_SPACE
            }

            // Line comment
            c == '/' && tokenStart + 1 < bufferEnd && buffer[tokenStart + 1] == '/' -> {
                tokenEnd = tokenStart + 2
                while (tokenEnd < bufferEnd && buffer[tokenEnd] != '\n') {
                    tokenEnd++
                }
                tokenType = VaisTokenTypes.LINE_COMMENT
            }

            // Block comment
            c == '/' && tokenStart + 1 < bufferEnd && buffer[tokenStart + 1] == '*' -> {
                tokenEnd = tokenStart + 2
                while (tokenEnd + 1 < bufferEnd) {
                    if (buffer[tokenEnd] == '*' && buffer[tokenEnd + 1] == '/') {
                        tokenEnd += 2
                        break
                    }
                    tokenEnd++
                }
                if (tokenEnd >= bufferEnd) tokenEnd = bufferEnd
                tokenType = VaisTokenTypes.BLOCK_COMMENT
            }

            // String literal
            c == '"' -> {
                tokenEnd = tokenStart + 1
                while (tokenEnd < bufferEnd) {
                    val ch = buffer[tokenEnd]
                    if (ch == '"') {
                        tokenEnd++
                        break
                    }
                    if (ch == '\\' && tokenEnd + 1 < bufferEnd) {
                        tokenEnd += 2
                        continue
                    }
                    tokenEnd++
                }
                tokenType = VaisTokenTypes.STRING
            }

            // Character literal
            c == '\'' -> {
                tokenEnd = tokenStart + 1
                while (tokenEnd < bufferEnd && buffer[tokenEnd] != '\'') {
                    if (buffer[tokenEnd] == '\\' && tokenEnd + 1 < bufferEnd) {
                        tokenEnd += 2
                        continue
                    }
                    tokenEnd++
                }
                if (tokenEnd < bufferEnd) tokenEnd++
                tokenType = VaisTokenTypes.CHAR
            }

            // Number
            c.isDigit() -> {
                tokenEnd = tokenStart + 1
                // Handle hex, binary, octal
                if (c == '0' && tokenEnd < bufferEnd) {
                    when (buffer[tokenEnd]) {
                        'x', 'X' -> {
                            tokenEnd++
                            while (tokenEnd < bufferEnd && buffer[tokenEnd].isHexDigit()) {
                                tokenEnd++
                            }
                        }
                        'b', 'B' -> {
                            tokenEnd++
                            while (tokenEnd < bufferEnd && buffer[tokenEnd] in "01") {
                                tokenEnd++
                            }
                        }
                        'o', 'O' -> {
                            tokenEnd++
                            while (tokenEnd < bufferEnd && buffer[tokenEnd] in "01234567") {
                                tokenEnd++
                            }
                        }
                        else -> {
                            while (tokenEnd < bufferEnd && (buffer[tokenEnd].isDigit() || buffer[tokenEnd] == '.')) {
                                tokenEnd++
                            }
                        }
                    }
                } else {
                    while (tokenEnd < bufferEnd && (buffer[tokenEnd].isDigit() || buffer[tokenEnd] == '.')) {
                        tokenEnd++
                    }
                }
                // Handle type suffix (i64, f64, etc.)
                if (tokenEnd < bufferEnd && buffer[tokenEnd] in "iufIUF") {
                    tokenEnd++
                    while (tokenEnd < bufferEnd && buffer[tokenEnd].isDigit()) {
                        tokenEnd++
                    }
                }
                tokenType = VaisTokenTypes.NUMBER
            }

            // Identifier or keyword
            c.isLetter() || c == '_' -> {
                tokenEnd = tokenStart + 1
                while (tokenEnd < bufferEnd && (buffer[tokenEnd].isLetterOrDigit() || buffer[tokenEnd] == '_')) {
                    tokenEnd++
                }
                val text = buffer.subSequence(tokenStart, tokenEnd).toString()
                tokenType = when (text) {
                    // Single-character keywords
                    "F" -> VaisTokenTypes.KW_F
                    "S" -> VaisTokenTypes.KW_S
                    "E" -> VaisTokenTypes.KW_E
                    "T" -> VaisTokenTypes.KW_T
                    "I" -> VaisTokenTypes.KW_I
                    "L" -> VaisTokenTypes.KW_L
                    "M" -> VaisTokenTypes.KW_M
                    "U" -> VaisTokenTypes.KW_U
                    "A" -> VaisTokenTypes.KW_A

                    // Full keywords
                    "let" -> VaisTokenTypes.KW_LET
                    "mut" -> VaisTokenTypes.KW_MUT
                    "if" -> VaisTokenTypes.KW_IF
                    "else" -> VaisTokenTypes.KW_ELSE
                    "loop" -> VaisTokenTypes.KW_LOOP
                    "while" -> VaisTokenTypes.KW_WHILE
                    "for" -> VaisTokenTypes.KW_FOR
                    "in" -> VaisTokenTypes.KW_IN
                    "match" -> VaisTokenTypes.KW_MATCH
                    "return" -> VaisTokenTypes.KW_RETURN
                    "break" -> VaisTokenTypes.KW_BREAK
                    "continue" -> VaisTokenTypes.KW_CONTINUE
                    "fn" -> VaisTokenTypes.KW_FN
                    "struct" -> VaisTokenTypes.KW_STRUCT
                    "enum" -> VaisTokenTypes.KW_ENUM
                    "trait" -> VaisTokenTypes.KW_TRAIT
                    "impl" -> VaisTokenTypes.KW_IMPL
                    "pub" -> VaisTokenTypes.KW_PUB
                    "use" -> VaisTokenTypes.KW_USE
                    "async" -> VaisTokenTypes.KW_ASYNC
                    "await" -> VaisTokenTypes.KW_AWAIT
                    "where" -> VaisTokenTypes.KW_WHERE
                    "self" -> VaisTokenTypes.KW_SELF
                    "Self" -> VaisTokenTypes.KW_SELF_TYPE

                    // Boolean literals
                    "true", "false" -> VaisTokenTypes.BOOL

                    // Built-in types
                    "i8", "i16", "i32", "i64", "i128",
                    "u8", "u16", "u32", "u64", "u128",
                    "f32", "f64", "bool", "char", "str" -> VaisTokenTypes.BUILTIN_TYPE

                    else -> VaisTokenTypes.IDENTIFIER
                }
            }

            // Operators and punctuation
            else -> {
                tokenEnd = tokenStart + 1
                tokenType = when (c) {
                    '(' -> VaisTokenTypes.LPAREN
                    ')' -> VaisTokenTypes.RPAREN
                    '{' -> VaisTokenTypes.LBRACE
                    '}' -> VaisTokenTypes.RBRACE
                    '[' -> VaisTokenTypes.LBRACKET
                    ']' -> VaisTokenTypes.RBRACKET
                    ',' -> VaisTokenTypes.COMMA
                    ';' -> VaisTokenTypes.SEMICOLON
                    ':' -> {
                        if (tokenEnd < bufferEnd && buffer[tokenEnd] == ':') {
                            tokenEnd++
                            VaisTokenTypes.COLON_COLON
                        } else {
                            VaisTokenTypes.COLON
                        }
                    }
                    '.' -> VaisTokenTypes.DOT
                    '@' -> VaisTokenTypes.AT
                    '#' -> VaisTokenTypes.HASH
                    '=' -> {
                        if (tokenEnd < bufferEnd && buffer[tokenEnd] == '=') {
                            tokenEnd++
                            VaisTokenTypes.EQ_EQ
                        } else if (tokenEnd < bufferEnd && buffer[tokenEnd] == '>') {
                            tokenEnd++
                            VaisTokenTypes.FAT_ARROW
                        } else {
                            VaisTokenTypes.EQ
                        }
                    }
                    '+' -> VaisTokenTypes.PLUS
                    '-' -> {
                        if (tokenEnd < bufferEnd && buffer[tokenEnd] == '>') {
                            tokenEnd++
                            VaisTokenTypes.ARROW
                        } else {
                            VaisTokenTypes.MINUS
                        }
                    }
                    '*' -> VaisTokenTypes.STAR
                    '/' -> VaisTokenTypes.SLASH
                    '%' -> VaisTokenTypes.PERCENT
                    '<' -> {
                        if (tokenEnd < bufferEnd && buffer[tokenEnd] == '=') {
                            tokenEnd++
                            VaisTokenTypes.LT_EQ
                        } else if (tokenEnd < bufferEnd && buffer[tokenEnd] == '<') {
                            tokenEnd++
                            VaisTokenTypes.SHL
                        } else {
                            VaisTokenTypes.LT
                        }
                    }
                    '>' -> {
                        if (tokenEnd < bufferEnd && buffer[tokenEnd] == '=') {
                            tokenEnd++
                            VaisTokenTypes.GT_EQ
                        } else if (tokenEnd < bufferEnd && buffer[tokenEnd] == '>') {
                            tokenEnd++
                            VaisTokenTypes.SHR
                        } else {
                            VaisTokenTypes.GT
                        }
                    }
                    '!' -> {
                        if (tokenEnd < bufferEnd && buffer[tokenEnd] == '=') {
                            tokenEnd++
                            VaisTokenTypes.NOT_EQ
                        } else {
                            VaisTokenTypes.BANG
                        }
                    }
                    '&' -> {
                        if (tokenEnd < bufferEnd && buffer[tokenEnd] == '&') {
                            tokenEnd++
                            VaisTokenTypes.AND_AND
                        } else {
                            VaisTokenTypes.AMP
                        }
                    }
                    '|' -> {
                        if (tokenEnd < bufferEnd && buffer[tokenEnd] == '|') {
                            tokenEnd++
                            VaisTokenTypes.OR_OR
                        } else if (tokenEnd < bufferEnd && buffer[tokenEnd] == '>') {
                            tokenEnd++
                            VaisTokenTypes.PIPE_ARROW
                        } else {
                            VaisTokenTypes.PIPE
                        }
                    }
                    '^' -> VaisTokenTypes.CARET
                    '~' -> VaisTokenTypes.TILDE
                    '?' -> VaisTokenTypes.QUESTION
                    '_' -> VaisTokenTypes.UNDERSCORE
                    else -> VaisTokenTypes.BAD_CHARACTER
                }
            }
        }
    }

    override fun getBufferSequence(): CharSequence = buffer

    override fun getBufferEnd(): Int = bufferEnd

    private fun Char.isHexDigit(): Boolean = this.isDigit() || this in 'a'..'f' || this in 'A'..'F'
}
