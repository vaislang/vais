package com.vais.intellij.debug

import com.intellij.execution.configurations.ConfigurationFactory
import com.intellij.execution.configurations.ConfigurationType
import com.vais.intellij.VaisIcons
import javax.swing.Icon

/**
 * Debug configuration type for Vais programs.
 *
 * Uses the Vais DAP server (vais-dap) for debugging support
 * including breakpoints, stepping, and variable inspection.
 */
class VaisDebugConfigurationType : ConfigurationType {
    override fun getDisplayName(): String = "Vais Debug"

    override fun getConfigurationTypeDescription(): String =
        "Debug a Vais program using vais-dap (Debug Adapter Protocol)"

    override fun getIcon(): Icon = VaisIcons.FILE

    override fun getId(): String = "VaisDebugConfiguration"

    override fun getConfigurationFactories(): Array<ConfigurationFactory> {
        return arrayOf(VaisDebugConfigurationFactory(this))
    }
}
