package com.vais.intellij.run

import com.intellij.execution.configurations.ConfigurationType
import com.intellij.execution.configurations.ConfigurationFactory
import com.vais.intellij.VaisIcons
import javax.swing.Icon

class VaisRunConfigurationType : ConfigurationType {
    override fun getDisplayName(): String {
        return "Vais"
    }

    override fun getConfigurationTypeDescription(): String {
        return "Run a Vais program"
    }

    override fun getIcon(): Icon {
        return VaisIcons.FILE
    }

    override fun getId(): String {
        return "VaisRunConfiguration"
    }

    override fun getConfigurationFactories(): Array<ConfigurationFactory> {
        return arrayOf(VaisRunConfigurationFactory(this))
    }

    companion object {
        fun getInstance(): VaisRunConfigurationType {
            return ConfigurationType.CONFIGURATION_TYPE_EP.findExtension(VaisRunConfigurationType::class.java)!!
        }
    }
}
