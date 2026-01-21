package com.vais.intellij

import com.intellij.psi.tree.IElementType
import com.intellij.psi.tree.TokenSet

/**
 * Token types for Vais language.
 */
object VaisTokenTypes {
    // Special tokens
    val WHITE_SPACE = VaisElementType("WHITE_SPACE")
    val BAD_CHARACTER = VaisElementType("BAD_CHARACTER")

    // Comments
    val LINE_COMMENT = VaisElementType("LINE_COMMENT")
    val BLOCK_COMMENT = VaisElementType("BLOCK_COMMENT")

    // Literals
    val STRING = VaisElementType("STRING")
    val CHAR = VaisElementType("CHAR")
    val NUMBER = VaisElementType("NUMBER")
    val BOOL = VaisElementType("BOOL")

    // Identifiers
    val IDENTIFIER = VaisElementType("IDENTIFIER")
    val BUILTIN_TYPE = VaisElementType("BUILTIN_TYPE")

    // Single-character keywords
    val KW_F = VaisElementType("KW_F")       // Function
    val KW_S = VaisElementType("KW_S")       // Struct
    val KW_E = VaisElementType("KW_E")       // Enum
    val KW_T = VaisElementType("KW_T")       // Trait
    val KW_I = VaisElementType("KW_I")       // If
    val KW_L = VaisElementType("KW_L")       // Loop
    val KW_M = VaisElementType("KW_M")       // Match
    val KW_U = VaisElementType("KW_U")       // Use
    val KW_A = VaisElementType("KW_A")       // Async

    // Full keywords
    val KW_LET = VaisElementType("KW_LET")
    val KW_MUT = VaisElementType("KW_MUT")
    val KW_IF = VaisElementType("KW_IF")
    val KW_ELSE = VaisElementType("KW_ELSE")
    val KW_LOOP = VaisElementType("KW_LOOP")
    val KW_WHILE = VaisElementType("KW_WHILE")
    val KW_FOR = VaisElementType("KW_FOR")
    val KW_IN = VaisElementType("KW_IN")
    val KW_MATCH = VaisElementType("KW_MATCH")
    val KW_RETURN = VaisElementType("KW_RETURN")
    val KW_BREAK = VaisElementType("KW_BREAK")
    val KW_CONTINUE = VaisElementType("KW_CONTINUE")
    val KW_FN = VaisElementType("KW_FN")
    val KW_STRUCT = VaisElementType("KW_STRUCT")
    val KW_ENUM = VaisElementType("KW_ENUM")
    val KW_TRAIT = VaisElementType("KW_TRAIT")
    val KW_IMPL = VaisElementType("KW_IMPL")
    val KW_PUB = VaisElementType("KW_PUB")
    val KW_USE = VaisElementType("KW_USE")
    val KW_ASYNC = VaisElementType("KW_ASYNC")
    val KW_AWAIT = VaisElementType("KW_AWAIT")
    val KW_WHERE = VaisElementType("KW_WHERE")
    val KW_SELF = VaisElementType("KW_SELF")
    val KW_SELF_TYPE = VaisElementType("KW_SELF_TYPE")

    // Operators
    val PLUS = VaisElementType("PLUS")
    val MINUS = VaisElementType("MINUS")
    val STAR = VaisElementType("STAR")
    val SLASH = VaisElementType("SLASH")
    val PERCENT = VaisElementType("PERCENT")
    val EQ = VaisElementType("EQ")
    val EQ_EQ = VaisElementType("EQ_EQ")
    val NOT_EQ = VaisElementType("NOT_EQ")
    val LT = VaisElementType("LT")
    val GT = VaisElementType("GT")
    val LT_EQ = VaisElementType("LT_EQ")
    val GT_EQ = VaisElementType("GT_EQ")
    val BANG = VaisElementType("BANG")
    val AND_AND = VaisElementType("AND_AND")
    val OR_OR = VaisElementType("OR_OR")
    val AMP = VaisElementType("AMP")
    val PIPE = VaisElementType("PIPE")
    val CARET = VaisElementType("CARET")
    val TILDE = VaisElementType("TILDE")
    val SHL = VaisElementType("SHL")
    val SHR = VaisElementType("SHR")
    val ARROW = VaisElementType("ARROW")
    val FAT_ARROW = VaisElementType("FAT_ARROW")
    val PIPE_ARROW = VaisElementType("PIPE_ARROW")

    // Punctuation
    val LPAREN = VaisElementType("LPAREN")
    val RPAREN = VaisElementType("RPAREN")
    val LBRACE = VaisElementType("LBRACE")
    val RBRACE = VaisElementType("RBRACE")
    val LBRACKET = VaisElementType("LBRACKET")
    val RBRACKET = VaisElementType("RBRACKET")
    val COMMA = VaisElementType("COMMA")
    val SEMICOLON = VaisElementType("SEMICOLON")
    val COLON = VaisElementType("COLON")
    val COLON_COLON = VaisElementType("COLON_COLON")
    val DOT = VaisElementType("DOT")
    val AT = VaisElementType("AT")
    val HASH = VaisElementType("HASH")
    val QUESTION = VaisElementType("QUESTION")
    val UNDERSCORE = VaisElementType("UNDERSCORE")

    // Token sets
    val COMMENTS = TokenSet.create(LINE_COMMENT, BLOCK_COMMENT)
    val STRINGS = TokenSet.create(STRING, CHAR)
    val KEYWORDS = TokenSet.create(
        KW_F, KW_S, KW_E, KW_T, KW_I, KW_L, KW_M, KW_U, KW_A,
        KW_LET, KW_MUT, KW_IF, KW_ELSE, KW_LOOP, KW_WHILE, KW_FOR, KW_IN,
        KW_MATCH, KW_RETURN, KW_BREAK, KW_CONTINUE, KW_FN, KW_STRUCT,
        KW_ENUM, KW_TRAIT, KW_IMPL, KW_PUB, KW_USE, KW_ASYNC, KW_AWAIT,
        KW_WHERE, KW_SELF, KW_SELF_TYPE
    )
    val OPERATORS = TokenSet.create(
        PLUS, MINUS, STAR, SLASH, PERCENT, EQ, EQ_EQ, NOT_EQ,
        LT, GT, LT_EQ, GT_EQ, BANG, AND_AND, OR_OR, AMP, PIPE,
        CARET, TILDE, SHL, SHR, ARROW, FAT_ARROW, PIPE_ARROW
    )
}

/**
 * Element type for Vais tokens.
 */
class VaisElementType(debugName: String) : IElementType(debugName, VaisLanguage)
