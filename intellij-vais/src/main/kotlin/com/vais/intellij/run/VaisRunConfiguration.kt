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
    var workingDirectory: String = ""
    var target: String = "native"  // native, js, wasm
    var environmentVariables: String = ""

    override fun getState(executor: Executor, environment: ExecutionEnvironment): RunProfileState {
        return VaisRunState(environment, this)
    }

    override fun getConfigurationEditor(): SettingsEditor<out RunConfiguration> {
        return VaisRunConfigurationEditor(project)
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
        if (workingDirectory.isNotEmpty() && !File(workingDirectory).isDirectory) {
            throw RuntimeConfigurationError("Working directory does not exist: $workingDirectory")
        }
    }

    /**
     * Returns the effective working directory.
     * Falls back to the directory containing the Vais file if not explicitly set.
     */
    fun effectiveWorkingDirectory(): String {
        if (workingDirectory.isNotEmpty()) return workingDirectory
        val vaisFileObj = File(vaisFile)
        return vaisFileObj.parent ?: project.basePath ?: "."
    }

    /**
     * Parses environment variables from the "KEY=VALUE" format (one per line or semicolon-separated).
     */
    fun parseEnvironmentVariables(): Map<String, String> {
        if (environmentVariables.isEmpty()) return emptyMap()
        return environmentVariables
            .split(";", "\n")
            .filter { it.contains("=") }
            .associate {
                val parts = it.split("=", limit = 2)
                parts[0].trim() to parts[1].trim()
            }
    }

    override fun readExternal(element: Element) {
        super.readExternal(element)
        vaisFile = element.getAttributeValue("vaisFile") ?: ""
        compilerPath = element.getAttributeValue("compilerPath") ?: "vaisc"
        optimizationLevel = element.getAttributeValue("optimizationLevel")?.toIntOrNull() ?: 0
        arguments = element.getAttributeValue("arguments") ?: ""
        workingDirectory = element.getAttributeValue("workingDirectory") ?: ""
        target = element.getAttributeValue("target") ?: "native"
        environmentVariables = element.getAttributeValue("environmentVariables") ?: ""
    }

    override fun writeExternal(element: Element) {
        super.writeExternal(element)
        element.setAttribute("vaisFile", vaisFile)
        element.setAttribute("compilerPath", compilerPath)
        element.setAttribute("optimizationLevel", optimizationLevel.toString())
        element.setAttribute("arguments", arguments)
        element.setAttribute("workingDirectory", workingDirectory)
        element.setAttribute("target", target)
        element.setAttribute("environmentVariables", environmentVariables)
    }
}
