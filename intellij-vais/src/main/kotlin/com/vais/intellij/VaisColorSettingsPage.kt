package com.vais.intellij

import com.intellij.openapi.editor.colors.TextAttributesKey
import com.intellij.openapi.fileTypes.SyntaxHighlighter
import com.intellij.openapi.options.colors.AttributesDescriptor
import com.intellij.openapi.options.colors.ColorDescriptor
import com.intellij.openapi.options.colors.ColorSettingsPage
import javax.swing.Icon

/**
 * Color settings page for Vais language.
 *
 * Allows users to customize syntax highlighting colors.
 */
class VaisColorSettingsPage : ColorSettingsPage {
    companion object {
        private val DESCRIPTORS = arrayOf(
            AttributesDescriptor("Keyword", VaisSyntaxHighlighter.KEYWORD),
            AttributesDescriptor("Built-in Type", VaisSyntaxHighlighter.BUILTIN_TYPE),
            AttributesDescriptor("Identifier", VaisSyntaxHighlighter.IDENTIFIER),
            AttributesDescriptor("String", VaisSyntaxHighlighter.STRING),
            AttributesDescriptor("Character", VaisSyntaxHighlighter.CHAR),
            AttributesDescriptor("Number", VaisSyntaxHighlighter.NUMBER),
            AttributesDescriptor("Boolean", VaisSyntaxHighlighter.BOOL),
            AttributesDescriptor("Line Comment", VaisSyntaxHighlighter.LINE_COMMENT),
            AttributesDescriptor("Block Comment", VaisSyntaxHighlighter.BLOCK_COMMENT),
            AttributesDescriptor("Operator", VaisSyntaxHighlighter.OPERATOR),
            AttributesDescriptor("Parentheses", VaisSyntaxHighlighter.PARENTHESES),
            AttributesDescriptor("Braces", VaisSyntaxHighlighter.BRACES),
            AttributesDescriptor("Brackets", VaisSyntaxHighlighter.BRACKETS),
            AttributesDescriptor("Comma", VaisSyntaxHighlighter.COMMA),
            AttributesDescriptor("Semicolon", VaisSyntaxHighlighter.SEMICOLON),
            AttributesDescriptor("Dot", VaisSyntaxHighlighter.DOT),
            AttributesDescriptor("Bad Character", VaisSyntaxHighlighter.BAD_CHARACTER)
        )
    }

    override fun getIcon(): Icon? = VaisIcons.FILE

    override fun getHighlighter(): SyntaxHighlighter = VaisSyntaxHighlighter()

    override fun getDemoText(): String = """
// Vais Language Demo
// Single-character keywords: F (function), S (struct), E (enum), T (trait)

/* Multi-line
   comment */

// Function definition
F add(a: i64, b: i64) -> i64 = a + b

// Struct definition
S Point {
    x: f64,
    y: f64
}

// Enum definition
E Option<T> {
    Some(T),
    None
}

// Trait definition
T Display {
    F display(self) -> str
}

// Main function
F main() {
    let x = 42
    let name = "Hello, Vais!"
    let pi = 3.14159
    let flag = true

    I x > 0 {
        puts("Positive")
    } else {
        puts("Non-positive")
    }

    L {
        I x == 0 { break }
        x = x - 1
    }

    // Self-recursion operator
    F fib(n: i64) -> i64 = I n <= 1 { n } else { @(n-1) + @(n-2) }
}
    """.trimIndent()

    override fun getAdditionalHighlightingTagToDescriptorMap(): Map<String, TextAttributesKey>? = null

    override fun getAttributeDescriptors(): Array<AttributesDescriptor> = DESCRIPTORS

    override fun getColorDescriptors(): Array<ColorDescriptor> = ColorDescriptor.EMPTY_ARRAY

    override fun getDisplayName(): String = "Vais"
}
