package com.vais.intellij.debug

import com.intellij.execution.configurations.CommandLineState
import com.intellij.execution.configurations.GeneralCommandLine
import com.intellij.execution.process.ProcessHandler
import com.intellij.execution.process.ProcessHandlerFactory
import com.intellij.execution.runners.ExecutionEnvironment
import com.intellij.openapi.diagnostic.Logger
import java.io.File

/**
 * Run state for Vais debug sessions.
 *
 * Compiles the Vais program with debug info (-g flag equivalent),
 * then launches the vais-dap server to attach to the compiled binary.
 */
class VaisDebugRunState(
    environment: ExecutionEnvironment,
    private val configuration: VaisDebugConfiguration
) : CommandLineState(environment) {

    private val logger = Logger.getInstance(VaisDebugRunState::class.java)

    override fun startProcess(): ProcessHandler {
        val vaisFile = File(configuration.vaisFile)
        val parentDir = vaisFile.parent ?: "."
        val baseName = vaisFile.nameWithoutExtension

        // Step 1: Compile with debug info
        logger.info("Compiling ${configuration.vaisFile} with debug info")
        val compileCommandLine = GeneralCommandLine()
            .withExePath(configuration.compilerPath)
            .withParameters("build")
            .withParameters(configuration.vaisFile)
            .withParameters("-o", baseName)
            .withParameters("--opt=0") // No optimization for better debug experience
            .withWorkDirectory(parentDir)

        val compileProcess = ProcessHandlerFactory.getInstance()
            .createProcessHandler(compileCommandLine)

        compileProcess.startNotify()
        compileProcess.waitFor()

        if (compileProcess.exitCode != 0) {
            throw RuntimeException("Compilation failed with exit code ${compileProcess.exitCode}")
        }

        // Step 2: Launch vais-dap server with the compiled binary
        val binaryPath = File(parentDir, baseName).absolutePath
        logger.info("Starting vais-dap for binary: $binaryPath")

        val dapCommandLine = GeneralCommandLine()
            .withExePath(configuration.dapServerPath)
            .withParameters("--program", binaryPath)
            .withWorkDirectory(parentDir)

        // Add program arguments if specified
        if (configuration.programArguments.isNotEmpty()) {
            dapCommandLine.withParameters(
                "--args",
                *configuration.programArguments.split("\\s+".toRegex()).toTypedArray()
            )
        }

        return ProcessHandlerFactory.getInstance().createColoredProcessHandler(dapCommandLine)
    }
}
