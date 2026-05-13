package com.vais.intellij

import com.intellij.lang.ASTNode
import com.intellij.lang.ParserDefinition
import com.intellij.lang.PsiParser
import com.intellij.lexer.Lexer
import com.intellij.openapi.project.Project
import com.intellij.psi.FileViewProvider
import com.intellij.psi.PsiElement
import com.intellij.psi.PsiFile
import com.intellij.psi.impl.source.tree.LeafPsiElement
import com.intellij.psi.tree.IFileElementType
import com.intellij.psi.tree.TokenSet

/**
 * Parser definition for Vais language.
 *
 * Note: The actual parsing is done by the LSP server.
 * This provides minimal parsing support for the IDE.
 */
class VaisParserDefinition : ParserDefinition {
    companion object {
        val FILE = IFileElementType(VaisLanguage)
    }

    override fun createLexer(project: Project?): Lexer = VaisLexer()

    override fun createParser(project: Project?): PsiParser = VaisParser()

    override fun getFileNodeType(): IFileElementType = FILE

    override fun getCommentTokens(): TokenSet = VaisTokenTypes.COMMENTS

    override fun getStringLiteralElements(): TokenSet = VaisTokenTypes.STRINGS

    override fun createElement(node: ASTNode): PsiElement {
        // Return a leaf PSI element stub -- actual parsing is delegated to the LSP server.
        // This prevents UnsupportedOperationException when the IDE creates PSI elements
        // from the AST during indexing or code analysis.
        return LeafPsiElement(node.elementType, node.text)
    }

    override fun createFile(viewProvider: FileViewProvider): PsiFile = VaisFile(viewProvider)
}
