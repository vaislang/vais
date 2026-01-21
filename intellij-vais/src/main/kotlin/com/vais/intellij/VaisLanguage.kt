package com.vais.intellij

import com.intellij.lang.Language

/**
 * Vais language definition for IntelliJ IDEA.
 */
object VaisLanguage : Language("Vais") {
    private fun readResolve(): Any = VaisLanguage

    override fun getDisplayName(): String = "Vais"

    override fun isCaseSensitive(): Boolean = true
}
