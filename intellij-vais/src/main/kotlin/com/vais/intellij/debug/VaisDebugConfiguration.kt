package com.vais.intellij.debug

import com.intellij.execution.Executor
import com.intellij.execution.configurations.*
import com.intellij.execution.runners.ExecutionEnvironment
import com.intellij.execution.configurations.RunConfigurationOptions
import com.intellij.openapi.options.SettingsEditor
import com.intellij.openapi.project.Project
import com.intellij.openapi.ui.LabeledComponent
import com.intellij.openapi.ui.TextFieldWithBrowseButton
import com.intellij.openapi.fileChooser.FileChooserDescriptorFactory
import com.intellij.ui.components.JBTextField
import org.jdom.Element
import java.awt.GridBagConstraints
import java.awt.GridBagLayout
import java.awt.Insets
import java.io.File
import javax.swing.JComponent
import javax.swing.JPanel

/**
 * Persisted options for a Vais debug configuration.
 */
class VaisDebugConfigurationOptions : RunConfigurationOptions() {
    var vaisFile by string("")
    var compilerPath by string("vaisc")
    var dapServerPath by string("vais-dap")
    var programArguments by string("")
}

/**
 * Debug configuration for Vais programs.
 *
 * Compiles the Vais file with debug info, then launches vais-dap
 * as a DAP server to debug the resulting binary.
 */
class VaisDebugConfiguration(
    project: Project,
    factory: ConfigurationFactory,
    name: String
) : RunConfigurationBase<VaisDebugConfigurationOptions>(project, factory, name) {

    var vaisFile: String
        get() = options.vaisFile ?: ""
        set(value) { options.vaisFile = value }

    var compilerPath: String
        get() = options.compilerPath ?: "vaisc"
        set(value) { options.compilerPath = value }

    var dapServerPath: String
        get() = options.dapServerPath ?: "vais-dap"
        set(value) { options.dapServerPath = value }

    var programArguments: String
        get() = options.programArguments ?: ""
        set(value) { options.programArguments = value }

    override fun getOptions(): VaisDebugConfigurationOptions {
        return super.getOptions() as VaisDebugConfigurationOptions
    }

    override fun getState(executor: Executor, environment: ExecutionEnvironment): RunProfileState {
        return VaisDebugRunState(environment, this)
    }

    override fun getConfigurationEditor(): SettingsEditor<out RunConfiguration> {
        return VaisDebugConfigurationEditor()
    }

    override fun checkConfiguration() {
        if (vaisFile.isEmpty()) {
            throw RuntimeConfigurationError("Vais source file is not specified")
        }
        val file = File(vaisFile)
        if (!file.exists()) {
            throw RuntimeConfigurationError("Vais source file does not exist: $vaisFile")
        }
        if (!file.name.endsWith(".vais")) {
            throw RuntimeConfigurationWarning("File does not have .vais extension")
        }
    }

    override fun readExternal(element: Element) {
        super.readExternal(element)
        vaisFile = element.getAttributeValue("vaisFile") ?: ""
        compilerPath = element.getAttributeValue("compilerPath") ?: "vaisc"
        dapServerPath = element.getAttributeValue("dapServerPath") ?: "vais-dap"
        programArguments = element.getAttributeValue("programArguments") ?: ""
    }

    override fun writeExternal(element: Element) {
        super.writeExternal(element)
        element.setAttribute("vaisFile", vaisFile)
        element.setAttribute("compilerPath", compilerPath)
        element.setAttribute("dapServerPath", dapServerPath)
        element.setAttribute("programArguments", programArguments)
    }
}

/**
 * Settings editor UI for the Vais debug configuration.
 */
class VaisDebugConfigurationEditor : SettingsEditor<VaisDebugConfiguration>() {
    private val vaisFileField = TextFieldWithBrowseButton()
    private val compilerPathField = JBTextField()
    private val dapServerPathField = JBTextField()
    private val argumentsField = JBTextField()

    init {
        vaisFileField.addBrowseFolderListener(
            "Select Vais File",
            "Choose the .vais source file to debug",
            null,
            FileChooserDescriptorFactory.createSingleFileDescriptor("vais")
        )
    }

    override fun createEditor(): JComponent {
        val panel = JPanel(GridBagLayout())
        val gbc = GridBagConstraints()
        gbc.fill = GridBagConstraints.HORIZONTAL
        gbc.insets = Insets(5, 5, 5, 5)
        gbc.weightx = 1.0

        gbc.gridx = 0
        gbc.gridy = 0
        panel.add(LabeledComponent.create(vaisFileField, "Vais file:"), gbc)

        gbc.gridy = 1
        panel.add(LabeledComponent.create(compilerPathField, "Compiler path (vaisc):"), gbc)

        gbc.gridy = 2
        panel.add(LabeledComponent.create(dapServerPathField, "DAP server path (vais-dap):"), gbc)

        gbc.gridy = 3
        panel.add(LabeledComponent.create(argumentsField, "Program arguments:"), gbc)

        return panel
    }

    override fun resetEditorFrom(configuration: VaisDebugConfiguration) {
        vaisFileField.text = configuration.vaisFile
        compilerPathField.text = configuration.compilerPath
        dapServerPathField.text = configuration.dapServerPath
        argumentsField.text = configuration.programArguments
    }

    override fun applyEditorTo(configuration: VaisDebugConfiguration) {
        configuration.vaisFile = vaisFileField.text
        configuration.compilerPath = compilerPathField.text
        configuration.dapServerPath = dapServerPathField.text
        configuration.programArguments = argumentsField.text
    }
}
