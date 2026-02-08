package com.vais.intellij.run

import com.intellij.execution.Executor
import com.intellij.execution.configurations.*
import com.intellij.execution.runners.ExecutionEnvironment
import com.intellij.openapi.options.SettingsEditor
import com.intellij.openapi.project.Project
import org.jdom.Element
import java.io.File

class VaisRunConfiguration(
    project: Project,
    factory: ConfigurationFactory,
    name: String
) : RunConfigurationBase<VaisRunState>(project, factory, name) {

    var vaisFile: String = ""
    var compilerPath: String = "vaisc"
    var optimizationLevel: Int = 0
    var arguments: String = ""

    override fun getState(executor: Executor, environment: ExecutionEnvironment): RunProfileState {
        return VaisRunState(environment, this)
    }

    override fun getConfigurationEditor(): SettingsEditor<out RunConfiguration> {
        return VaisRunConfigurationEditor()
    }

    override fun checkConfiguration() {
        if (vaisFile.isEmpty()) {
            throw RuntimeConfigurationError("Vais file is not specified")
        }
        val file = File(vaisFile)
        if (!file.exists()) {
            throw RuntimeConfigurationError("Vais file does not exist: $vaisFile")
        }
        if (!file.name.endsWith(".vais")) {
            throw RuntimeConfigurationWarning("File does not have .vais extension")
        }
    }

    override fun readExternal(element: Element) {
        super.readExternal(element)
        vaisFile = element.getAttributeValue("vaisFile") ?: ""
        compilerPath = element.getAttributeValue("compilerPath") ?: "vaisc"
        optimizationLevel = element.getAttributeValue("optimizationLevel")?.toIntOrNull() ?: 0
        arguments = element.getAttributeValue("arguments") ?: ""
    }

    override fun writeExternal(element: Element) {
        super.writeExternal(element)
        element.setAttribute("vaisFile", vaisFile)
        element.setAttribute("compilerPath", compilerPath)
        element.setAttribute("optimizationLevel", optimizationLevel.toString())
        element.setAttribute("arguments", arguments)
    }
}
