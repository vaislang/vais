package com.vais.intellij

import com.intellij.extapi.psi.PsiFileBase
import com.intellij.openapi.fileTypes.FileType
import com.intellij.psi.FileViewProvider

/**
 * PSI file implementation for Vais source files.
 */
class VaisFile(viewProvider: FileViewProvider) : PsiFileBase(viewProvider, VaisLanguage) {
    override fun getFileType(): FileType = VaisFileType

    override fun toString(): String = "Vais File"
}
