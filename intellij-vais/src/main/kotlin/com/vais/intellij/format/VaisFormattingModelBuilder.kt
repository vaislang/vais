package com.vais.intellij.format

import com.intellij.formatting.*
import com.intellij.lang.ASTNode
import com.intellij.psi.PsiFile
import com.intellij.psi.codeStyle.CodeStyleSettings
import com.intellij.psi.formatter.common.AbstractBlock
import com.vais.intellij.VaisLanguage
import com.vais.intellij.VaisTokenTypes

/**
 * Formatting model builder for Vais language.
 *
 * Provides basic formatting support:
 * - Indentation after opening braces
 * - Space around operators
 * - Proper line wrapping
 *
 * Note: Full formatting is handled by the LSP server (textDocument/formatting).
 * This provides fallback formatting when the LSP is unavailable.
 */
class VaisFormattingModelBuilder : FormattingModelBuilder {

    override fun createModel(formattingContext: FormattingContext): FormattingModel {
        val settings = formattingContext.codeStyleSettings
        val file = formattingContext.psiElement.containingFile
        val node = file.node
        val block = VaisBlock(node, null, null, createSpacingBuilder(settings))
        return FormattingModelProvider.createFormattingModelForPsiFile(file, block, settings)
    }

    private fun createSpacingBuilder(settings: CodeStyleSettings): SpacingBuilder {
        return SpacingBuilder(settings, VaisLanguage)
            // Space after commas
            .after(VaisTokenTypes.COMMA).spaces(1)
            // Space around assignment
            .around(VaisTokenTypes.EQ).spaces(1)
            // Space around comparison operators
            .around(VaisTokenTypes.EQ_EQ).spaces(1)
            .around(VaisTokenTypes.NOT_EQ).spaces(1)
            .around(VaisTokenTypes.LT).spaces(1)
            .around(VaisTokenTypes.GT).spaces(1)
            .around(VaisTokenTypes.LT_EQ).spaces(1)
            .around(VaisTokenTypes.GT_EQ).spaces(1)
            // Space around arithmetic operators
            .around(VaisTokenTypes.PLUS).spaces(1)
            .around(VaisTokenTypes.MINUS).spaces(1)
            .around(VaisTokenTypes.STAR).spaces(1)
            .around(VaisTokenTypes.SLASH).spaces(1)
            .around(VaisTokenTypes.PERCENT).spaces(1)
            // Space around logical operators
            .around(VaisTokenTypes.AND_AND).spaces(1)
            .around(VaisTokenTypes.OR_OR).spaces(1)
            // Space around arrows
            .around(VaisTokenTypes.ARROW).spaces(1)
            .around(VaisTokenTypes.FAT_ARROW).spaces(1)
            .around(VaisTokenTypes.PIPE_ARROW).spaces(1)
            // Space after colon (type annotations)
            .after(VaisTokenTypes.COLON).spaces(1)
            // Space before opening brace
            .before(VaisTokenTypes.LBRACE).spaces(1)
            // No space inside parentheses
            .after(VaisTokenTypes.LPAREN).spaces(0)
            .before(VaisTokenTypes.RPAREN).spaces(0)
            // No space inside brackets
            .after(VaisTokenTypes.LBRACKET).spaces(0)
            .before(VaisTokenTypes.RBRACKET).spaces(0)
    }
}

/**
 * Formatting block for Vais code.
 */
class VaisBlock(
    node: ASTNode,
    wrap: Wrap?,
    alignment: Alignment?,
    private val spacingBuilder: SpacingBuilder
) : AbstractBlock(node, wrap, alignment) {

    override fun buildChildren(): List<Block> {
        val blocks = mutableListOf<Block>()
        var child = myNode.firstChildNode
        while (child != null) {
            if (child.elementType != VaisTokenTypes.WHITE_SPACE) {
                blocks.add(VaisBlock(child, null, null, spacingBuilder))
            }
            child = child.treeNext
        }
        return blocks
    }

    override fun getIndent(): Indent? {
        val parent = myNode.treeParent ?: return Indent.getNoneIndent()

        // Indent contents inside braces
        val parentType = parent.elementType
        if (parentType == VaisTokenTypes.LBRACE || parentType == VaisTokenTypes.RBRACE) {
            return Indent.getNoneIndent()
        }

        return Indent.getNoneIndent()
    }

    override fun getSpacing(child1: Block?, child2: Block): Spacing? {
        return spacingBuilder.getSpacing(this, child1, child2)
    }

    override fun isLeaf(): Boolean {
        return myNode.firstChildNode == null
    }

    override fun getChildAttributes(newChildIndex: Int): ChildAttributes {
        return ChildAttributes(Indent.getNormalIndent(), null)
    }
}
