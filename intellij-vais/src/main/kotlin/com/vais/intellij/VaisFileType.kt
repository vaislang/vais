package com.vais.intellij

import com.intellij.openapi.fileTypes.LanguageFileType
import javax.swing.Icon

/**
 * File type for .vais files.
 */
object VaisFileType : LanguageFileType(VaisLanguage) {
    override fun getName(): String = "Vais"

    override fun getDescription(): String = "Vais language file"

    override fun getDefaultExtension(): String = "vais"

    override fun getIcon(): Icon? = VaisIcons.FILE
}
