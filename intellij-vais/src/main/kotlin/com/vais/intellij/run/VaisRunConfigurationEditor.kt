package com.vais.intellij.run

import com.intellij.openapi.fileChooser.FileChooserDescriptorFactory
import com.intellij.openapi.options.SettingsEditor
import com.intellij.openapi.project.Project
import com.intellij.openapi.ui.LabeledComponent
import com.intellij.openapi.ui.TextFieldWithBrowseButton
import com.intellij.ui.components.JBTextField
import java.awt.GridBagConstraints
import java.awt.GridBagLayout
import java.awt.Insets
import javax.swing.JComboBox
import javax.swing.JComponent
import javax.swing.JPanel

class VaisRunConfigurationEditor(private val project: Project) : SettingsEditor<VaisRunConfiguration>() {
    private val vaisFileField = TextFieldWithBrowseButton()
    private val compilerPathField = JBTextField()
    private val optimizationLevelCombo = JComboBox(arrayOf("0", "1", "2", "3"))
    private val targetCombo = JComboBox(arrayOf("native", "js", "wasm"))
    private val argumentsField = JBTextField()
    private val workingDirectoryField = TextFieldWithBrowseButton()
    private val envVarsField = JBTextField()

    init {
        vaisFileField.addBrowseFolderListener(
            "Select Vais File",
            null,
            project,
            FileChooserDescriptorFactory.createSingleFileDescriptor("vais")
        )
        workingDirectoryField.addBrowseFolderListener(
            "Select Working Directory",
            null,
            project,
            FileChooserDescriptorFactory.createSingleFolderDescriptor()
        )
    }

    override fun createEditor(): JComponent {
        val panel = JPanel(GridBagLayout())
        val gbc = GridBagConstraints()
        gbc.fill = GridBagConstraints.HORIZONTAL
        gbc.insets = Insets(5, 5, 5, 5)
        gbc.weightx = 1.0

        // Vais file field
        gbc.gridx = 0
        gbc.gridy = 0
        panel.add(LabeledComponent.create(vaisFileField, "Vais file:"), gbc)

        // Compiler path field
        gbc.gridy = 1
        panel.add(LabeledComponent.create(compilerPathField, "Compiler path:"), gbc)

        // Target selection
        gbc.gridy = 2
        panel.add(LabeledComponent.create(targetCombo, "Target:"), gbc)

        // Optimization level
        gbc.gridy = 3
        panel.add(LabeledComponent.create(optimizationLevelCombo, "Optimization level:"), gbc)

        // Working directory
        gbc.gridy = 4
        panel.add(LabeledComponent.create(workingDirectoryField, "Working directory:"), gbc)

        // Arguments field
        gbc.gridy = 5
        panel.add(LabeledComponent.create(argumentsField, "Program arguments:"), gbc)

        // Environment variables
        gbc.gridy = 6
        panel.add(LabeledComponent.create(envVarsField, "Environment variables (KEY=VAL;...):"), gbc)

        return panel
    }

    override fun resetEditorFrom(configuration: VaisRunConfiguration) {
        vaisFileField.text = configuration.vaisFile
        compilerPathField.text = configuration.compilerPath
        optimizationLevelCombo.selectedIndex = configuration.optimizationLevel.coerceIn(0, 3)
        targetCombo.selectedItem = configuration.target
        argumentsField.text = configuration.arguments
        workingDirectoryField.text = configuration.workingDirectory
        envVarsField.text = configuration.environmentVariables
    }

    override fun applyEditorTo(configuration: VaisRunConfiguration) {
        configuration.vaisFile = vaisFileField.text
        configuration.compilerPath = compilerPathField.text
        configuration.optimizationLevel = optimizationLevelCombo.selectedIndex
        configuration.target = targetCombo.selectedItem as? String ?: "native"
        configuration.arguments = argumentsField.text
        configuration.workingDirectory = workingDirectoryField.text
        configuration.environmentVariables = envVarsField.text
    }
}
