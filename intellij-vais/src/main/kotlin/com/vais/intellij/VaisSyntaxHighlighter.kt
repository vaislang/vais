package com.vais.intellij

import com.intellij.lexer.Lexer
import com.intellij.openapi.editor.DefaultLanguageHighlighterColors
import com.intellij.openapi.editor.HighlighterColors
import com.intellij.openapi.editor.colors.TextAttributesKey
import com.intellij.openapi.editor.colors.TextAttributesKey.createTextAttributesKey
import com.intellij.openapi.fileTypes.SyntaxHighlighter
import com.intellij.openapi.fileTypes.SyntaxHighlighterBase
import com.intellij.openapi.fileTypes.SyntaxHighlighterFactory
import com.intellij.openapi.project.Project
import com.intellij.openapi.vfs.VirtualFile
import com.intellij.psi.tree.IElementType

/**
 * Syntax highlighter for Vais language.
 */
class VaisSyntaxHighlighter : SyntaxHighlighterBase() {
    companion object {
        // Text attribute keys
        val KEYWORD = createTextAttributesKey("VAIS_KEYWORD", DefaultLanguageHighlighterColors.KEYWORD)
        val BUILTIN_TYPE = createTextAttributesKey("VAIS_BUILTIN_TYPE", DefaultLanguageHighlighterColors.CLASS_NAME)
        val IDENTIFIER = createTextAttributesKey("VAIS_IDENTIFIER", DefaultLanguageHighlighterColors.IDENTIFIER)
        val STRING = createTextAttributesKey("VAIS_STRING", DefaultLanguageHighlighterColors.STRING)
        val CHAR = createTextAttributesKey("VAIS_CHAR", DefaultLanguageHighlighterColors.STRING)
        val NUMBER = createTextAttributesKey("VAIS_NUMBER", DefaultLanguageHighlighterColors.NUMBER)
        val BOOL = createTextAttributesKey("VAIS_BOOL", DefaultLanguageHighlighterColors.KEYWORD)
        val LINE_COMMENT = createTextAttributesKey("VAIS_LINE_COMMENT", DefaultLanguageHighlighterColors.LINE_COMMENT)
        val BLOCK_COMMENT = createTextAttributesKey("VAIS_BLOCK_COMMENT", DefaultLanguageHighlighterColors.BLOCK_COMMENT)
        val OPERATOR = createTextAttributesKey("VAIS_OPERATOR", DefaultLanguageHighlighterColors.OPERATION_SIGN)
        val PARENTHESES = createTextAttributesKey("VAIS_PARENTHESES", DefaultLanguageHighlighterColors.PARENTHESES)
        val BRACES = createTextAttributesKey("VAIS_BRACES", DefaultLanguageHighlighterColors.BRACES)
        val BRACKETS = createTextAttributesKey("VAIS_BRACKETS", DefaultLanguageHighlighterColors.BRACKETS)
        val COMMA = createTextAttributesKey("VAIS_COMMA", DefaultLanguageHighlighterColors.COMMA)
        val SEMICOLON = createTextAttributesKey("VAIS_SEMICOLON", DefaultLanguageHighlighterColors.SEMICOLON)
        val DOT = createTextAttributesKey("VAIS_DOT", DefaultLanguageHighlighterColors.DOT)
        val BAD_CHARACTER = createTextAttributesKey("VAIS_BAD_CHARACTER", HighlighterColors.BAD_CHARACTER)

        // Attribute key arrays
        private val KEYWORD_KEYS = arrayOf(KEYWORD)
        private val BUILTIN_TYPE_KEYS = arrayOf(BUILTIN_TYPE)
        private val IDENTIFIER_KEYS = arrayOf(IDENTIFIER)
        private val STRING_KEYS = arrayOf(STRING)
        private val CHAR_KEYS = arrayOf(CHAR)
        private val NUMBER_KEYS = arrayOf(NUMBER)
        private val BOOL_KEYS = arrayOf(BOOL)
        private val LINE_COMMENT_KEYS = arrayOf(LINE_COMMENT)
        private val BLOCK_COMMENT_KEYS = arrayOf(BLOCK_COMMENT)
        private val OPERATOR_KEYS = arrayOf(OPERATOR)
        private val PARENTHESES_KEYS = arrayOf(PARENTHESES)
        private val BRACES_KEYS = arrayOf(BRACES)
        private val BRACKETS_KEYS = arrayOf(BRACKETS)
        private val COMMA_KEYS = arrayOf(COMMA)
        private val SEMICOLON_KEYS = arrayOf(SEMICOLON)
        private val DOT_KEYS = arrayOf(DOT)
        private val BAD_CHARACTER_KEYS = arrayOf(BAD_CHARACTER)
        private val EMPTY_KEYS = emptyArray<TextAttributesKey>()
    }

    override fun getHighlightingLexer(): Lexer = VaisLexer()

    override fun getTokenHighlights(tokenType: IElementType?): Array<TextAttributesKey> {
        return when (tokenType) {
            // Keywords
            VaisTokenTypes.KW_F, VaisTokenTypes.KW_S, VaisTokenTypes.KW_E,
            VaisTokenTypes.KW_T, VaisTokenTypes.KW_I, VaisTokenTypes.KW_L,
            VaisTokenTypes.KW_M, VaisTokenTypes.KW_U, VaisTokenTypes.KW_A,
            VaisTokenTypes.KW_LET, VaisTokenTypes.KW_MUT, VaisTokenTypes.KW_IF,
            VaisTokenTypes.KW_ELSE, VaisTokenTypes.KW_LOOP, VaisTokenTypes.KW_WHILE,
            VaisTokenTypes.KW_FOR, VaisTokenTypes.KW_IN, VaisTokenTypes.KW_MATCH,
            VaisTokenTypes.KW_RETURN, VaisTokenTypes.KW_BREAK, VaisTokenTypes.KW_CONTINUE,
            VaisTokenTypes.KW_FN, VaisTokenTypes.KW_STRUCT, VaisTokenTypes.KW_ENUM,
            VaisTokenTypes.KW_TRAIT, VaisTokenTypes.KW_IMPL, VaisTokenTypes.KW_PUB,
            VaisTokenTypes.KW_USE, VaisTokenTypes.KW_ASYNC, VaisTokenTypes.KW_AWAIT,
            VaisTokenTypes.KW_WHERE, VaisTokenTypes.KW_SELF, VaisTokenTypes.KW_SELF_TYPE -> KEYWORD_KEYS

            // Built-in types
            VaisTokenTypes.BUILTIN_TYPE -> BUILTIN_TYPE_KEYS

            // Identifiers
            VaisTokenTypes.IDENTIFIER -> IDENTIFIER_KEYS

            // Literals
            VaisTokenTypes.STRING -> STRING_KEYS
            VaisTokenTypes.CHAR -> CHAR_KEYS
            VaisTokenTypes.NUMBER -> NUMBER_KEYS
            VaisTokenTypes.BOOL -> BOOL_KEYS

            // Comments
            VaisTokenTypes.LINE_COMMENT -> LINE_COMMENT_KEYS
            VaisTokenTypes.BLOCK_COMMENT -> BLOCK_COMMENT_KEYS

            // Operators
            VaisTokenTypes.PLUS, VaisTokenTypes.MINUS, VaisTokenTypes.STAR,
            VaisTokenTypes.SLASH, VaisTokenTypes.PERCENT, VaisTokenTypes.EQ,
            VaisTokenTypes.EQ_EQ, VaisTokenTypes.NOT_EQ, VaisTokenTypes.LT,
            VaisTokenTypes.GT, VaisTokenTypes.LT_EQ, VaisTokenTypes.GT_EQ,
            VaisTokenTypes.BANG, VaisTokenTypes.AND_AND, VaisTokenTypes.OR_OR,
            VaisTokenTypes.AMP, VaisTokenTypes.PIPE, VaisTokenTypes.CARET,
            VaisTokenTypes.TILDE, VaisTokenTypes.SHL, VaisTokenTypes.SHR,
            VaisTokenTypes.ARROW, VaisTokenTypes.FAT_ARROW, VaisTokenTypes.PIPE_ARROW,
            VaisTokenTypes.COLON, VaisTokenTypes.COLON_COLON, VaisTokenTypes.AT,
            VaisTokenTypes.HASH, VaisTokenTypes.QUESTION -> OPERATOR_KEYS

            // Punctuation
            VaisTokenTypes.LPAREN, VaisTokenTypes.RPAREN -> PARENTHESES_KEYS
            VaisTokenTypes.LBRACE, VaisTokenTypes.RBRACE -> BRACES_KEYS
            VaisTokenTypes.LBRACKET, VaisTokenTypes.RBRACKET -> BRACKETS_KEYS
            VaisTokenTypes.COMMA -> COMMA_KEYS
            VaisTokenTypes.SEMICOLON -> SEMICOLON_KEYS
            VaisTokenTypes.DOT -> DOT_KEYS

            // Bad character
            VaisTokenTypes.BAD_CHARACTER -> BAD_CHARACTER_KEYS

            else -> EMPTY_KEYS
        }
    }
}

/**
 * Factory for creating Vais syntax highlighters.
 */
class VaisSyntaxHighlighterFactory : SyntaxHighlighterFactory() {
    override fun getSyntaxHighlighter(project: Project?, virtualFile: VirtualFile?): SyntaxHighlighter {
        return VaisSyntaxHighlighter()
    }
}
