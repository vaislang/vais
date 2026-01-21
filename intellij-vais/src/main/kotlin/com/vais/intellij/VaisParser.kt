package com.vais.intellij

import com.intellij.lang.ASTNode
import com.intellij.lang.PsiBuilder
import com.intellij.lang.PsiParser
import com.intellij.psi.tree.IElementType

/**
 * Minimal parser for Vais language.
 *
 * The actual parsing is done by the LSP server.
 * This parser creates a simple file structure.
 */
class VaisParser : PsiParser {
    override fun parse(root: IElementType, builder: PsiBuilder): ASTNode {
        val marker = builder.mark()

        // Just consume all tokens - LSP handles real parsing
        while (!builder.eof()) {
            builder.advanceLexer()
        }

        marker.done(root)
        return builder.treeBuilt
    }
}
