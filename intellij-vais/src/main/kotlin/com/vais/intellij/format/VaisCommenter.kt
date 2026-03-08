package com.vais.intellij.format

import com.intellij.lang.Commenter

/**
 * Commenter for Vais language.
 *
 * Supports line comments (# and //) and block comments.
 */
class VaisCommenter : Commenter {

    override fun getLineCommentPrefix(): String = "# "

    override fun getBlockCommentPrefix(): String = "/* "

    override fun getBlockCommentSuffix(): String = " */"

    override fun getCommentedBlockCommentPrefix(): String? = null

    override fun getCommentedBlockCommentSuffix(): String? = null
}
