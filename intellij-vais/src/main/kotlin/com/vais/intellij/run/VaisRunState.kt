package com.vais.intellij.run

import com.intellij.execution.configurations.CommandLineState
import com.intellij.execution.configurations.GeneralCommandLine
import com.intellij.execution.process.ProcessHandler
import com.intellij.execution.process.ProcessHandlerFactory
import com.intellij.execution.runners.ExecutionEnvironment
import java.io.File

class VaisRunState(
    environment: ExecutionEnvironment,
    private val configuration: VaisRunConfiguration
) : CommandLineState(environment) {

    override fun startProcess(): ProcessHandler {
        val vaisFile = File(configuration.vaisFile)
        val parentDir = vaisFile.parent ?: "."
        val baseName = vaisFile.nameWithoutExtension

        // Step 1: Compile the Vais file
        val compileCommandLine = GeneralCommandLine()
            .withExePath(configuration.compilerPath)
            .withParameters("build")
            .withParameters(configuration.vaisFile)
            .withParameters("-o", baseName)
            .withParameters("--opt=${configuration.optimizationLevel}")
            .withWorkDirectory(parentDir)

        val compileProcess = ProcessHandlerFactory.getInstance()
            .createProcessHandler(compileCommandLine)

        // Wait for compilation to complete
        compileProcess.startNotify()
        compileProcess.waitFor()

        if (compileProcess.exitCode != 0) {
            throw RuntimeException("Compilation failed with exit code ${compileProcess.exitCode}")
        }

        // Step 2: Execute the compiled binary
        val binaryPath = File(parentDir, baseName).absolutePath
        val runCommandLine = GeneralCommandLine()
            .withExePath(binaryPath)
            .withWorkDirectory(parentDir)

        // Add user-specified arguments
        if (configuration.arguments.isNotEmpty()) {
            runCommandLine.withParameters(*configuration.arguments.split("\\s+".toRegex()).toTypedArray())
        }

        return ProcessHandlerFactory.getInstance().createColoredProcessHandler(runCommandLine)
    }
}
