package com.vais.intellij.run

import com.intellij.execution.configurations.ConfigurationFactory
import com.intellij.execution.configurations.ConfigurationType
import com.intellij.execution.configurations.RunConfiguration
import com.intellij.openapi.project.Project

class VaisRunConfigurationFactory(type: ConfigurationType) : ConfigurationFactory(type) {
    override fun getId(): String {
        return "Vais"
    }

    override fun createTemplateConfiguration(project: Project): RunConfiguration {
        return VaisRunConfiguration(project, this, "Vais")
    }

    override fun getName(): String {
        return "Vais"
    }
}
