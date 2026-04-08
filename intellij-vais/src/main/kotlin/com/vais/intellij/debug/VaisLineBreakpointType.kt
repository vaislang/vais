package com.vais.intellij.debug

import com.intellij.openapi.project.Project
import com.intellij.openapi.vfs.VirtualFile
import com.intellij.psi.PsiElement
import com.intellij.psi.PsiFile
import com.intellij.xdebugger.breakpoints.XBreakpointProperties
import com.intellij.xdebugger.breakpoints.XLineBreakpointTypeBase
import com.intellij.xdebugger.evaluation.XDebuggerEditorsProvider
import com.intellij.xdebugger.evaluation.XDebuggerEditorsProviderBase
import com.vais.intellij.VaisFileType

/**
 * Breakpoint type for .vais source files.
 *
 * Allows the user to set line breakpoints in the editor gutter.
 * These are forwarded to vais-dap via the DAP setBreakpoints request.
 */
class VaisLineBreakpointType :
    XLineBreakpointTypeBase(ID, "Vais Line Breakpoints", VaisEditorsProvider()) {

    companion object {
        const val ID = "vais-line"
    }

    override fun canPutAt(file: VirtualFile, line: Int, project: Project): Boolean {
        return file.extension == "vais"
    }

    override fun isSuspendThreadSupported(): Boolean = false
}

/**
 * Editors provider for Vais breakpoints.
 *
 * Provides the file type used in expression evaluation pop-ups.
 * createExpressionCodeFragment is not invoked unless expression evaluation
 * is triggered, so a minimal implementation returning a dummy fragment suffices.
 */
class VaisEditorsProvider : XDebuggerEditorsProviderBase() {
    override fun getFileType() = VaisFileType

    override fun createExpressionCodeFragment(
        project: Project,
        text: String,
        context: PsiElement?,
        isPhysical: Boolean
    ): PsiFile {
        // Vais does not implement expression evaluation yet; return a plain file.
        val factory = com.intellij.psi.PsiFileFactory.getInstance(project)
        return factory.createFileFromText(
            "vais_expression.vais",
            VaisFileType,
            text
        )
    }
}
