package com.vais.intellij.format

import com.intellij.lang.BracePair
import com.intellij.lang.PairedBraceMatcher
import com.intellij.psi.PsiFile
import com.intellij.psi.tree.IElementType
import com.vais.intellij.VaisTokenTypes

/**
 * Brace matcher for Vais language.
 *
 * Provides auto-closing and matching highlight for:
 * - Parentheses: ()
 * - Braces: {}
 * - Brackets: []
 * - Angle brackets: <> (for generics)
 */
class VaisBraceMatcher : PairedBraceMatcher {

    override fun getPairs(): Array<BracePair> {
        return arrayOf(
            BracePair(VaisTokenTypes.LPAREN, VaisTokenTypes.RPAREN, false),
            BracePair(VaisTokenTypes.LBRACE, VaisTokenTypes.RBRACE, true),
            BracePair(VaisTokenTypes.LBRACKET, VaisTokenTypes.RBRACKET, false),
            BracePair(VaisTokenTypes.LT, VaisTokenTypes.GT, false)
        )
    }

    override fun isPairedBracesAllowedBeforeType(lbraceType: IElementType, contextType: IElementType?): Boolean {
        return true
    }

    override fun getCodeConstructStart(file: PsiFile?, openingBraceOffset: Int): Int {
        return openingBraceOffset
    }
}
