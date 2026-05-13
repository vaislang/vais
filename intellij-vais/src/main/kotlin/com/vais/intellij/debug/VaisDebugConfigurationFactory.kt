package com.vais.intellij.debug

import com.intellij.execution.configurations.ConfigurationFactory
import com.intellij.execution.configurations.ConfigurationType
import com.intellij.execution.configurations.RunConfiguration
import com.intellij.execution.configurations.RunConfigurationOptions
import com.intellij.openapi.project.Project

/**
 * Factory for creating Vais debug run configurations.
 */
class VaisDebugConfigurationFactory(type: ConfigurationType) : ConfigurationFactory(type) {
    override fun getId(): String = "VaisDebugConfigurationFactory"

    override fun createTemplateConfiguration(project: Project): RunConfiguration {
        return VaisDebugConfiguration(project, this, "Vais Debug")
    }

    override fun getOptionsClass(): Class<out RunConfigurationOptions> = VaisDebugConfigurationOptions::class.java
}
