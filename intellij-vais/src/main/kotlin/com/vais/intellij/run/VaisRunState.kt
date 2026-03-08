package com.vais.intellij.run

import com.intellij.execution.configurations.CommandLineState
import com.intellij.execution.configurations.GeneralCommandLine
import com.intellij.execution.process.ProcessHandler
import com.intellij.execution.process.ProcessHandlerFactory
import com.intellij.execution.runners.ExecutionEnvironment
import com.intellij.openapi.diagnostic.Logger
import java.io.File

class VaisRunState(
    environment: ExecutionEnvironment,
    private val configuration: VaisRunConfiguration
) : CommandLineState(environment) {

    private val logger = Logger.getInstance(VaisRunState::class.java)

    override fun startProcess(): ProcessHandler {
        val vaisFile = File(configuration.vaisFile)
        val workDir = configuration.effectiveWorkingDirectory()
        val baseName = vaisFile.nameWithoutExtension

        // Build compile command based on target
        val compileCommandLine = GeneralCommandLine()
            .withExePath(configuration.compilerPath)
            .withWorkDirectory(workDir)

        when (configuration.target) {
            "js" -> {
                logger.info("Compiling ${configuration.vaisFile} to JavaScript")
                compileCommandLine
                    .withParameters("build")
                    .withParameters(configuration.vaisFile)
                    .withParameters("--target", "js")
                    .withParameters("-o", "$baseName.mjs")
                    .withParameters("--opt=${configuration.optimizationLevel}")
            }
            "wasm" -> {
                logger.info("Compiling ${configuration.vaisFile} to WASM")
                compileCommandLine
                    .withParameters("build")
                    .withParameters(configuration.vaisFile)
                    .withParameters("--target", "wasm32-unknown-unknown")
                    .withParameters("-o", "$baseName.wasm")
                    .withParameters("--opt=${configuration.optimizationLevel}")
            }
            else -> {
                logger.info("Compiling ${configuration.vaisFile} to native binary")
                compileCommandLine
                    .withParameters("build")
                    .withParameters(configuration.vaisFile)
                    .withParameters("-o", baseName)
                    .withParameters("--opt=${configuration.optimizationLevel}")
            }
        }

        // Add environment variables to compilation
        val envVars = configuration.parseEnvironmentVariables()
        envVars.forEach { (key, value) ->
            compileCommandLine.withEnvironment(key, value)
        }

        // Step 1: Compile
        val compileProcess = ProcessHandlerFactory.getInstance()
            .createProcessHandler(compileCommandLine)

        compileProcess.startNotify()
        compileProcess.waitFor()

        if (compileProcess.exitCode != 0) {
            throw RuntimeException("Compilation failed with exit code ${compileProcess.exitCode}")
        }

        // Step 2: Execute based on target
        val runCommandLine = when (configuration.target) {
            "js" -> {
                val jsFile = File(workDir, "$baseName.mjs").absolutePath
                GeneralCommandLine("node", jsFile)
                    .withWorkDirectory(workDir)
            }
            "wasm" -> {
                // Use wasmtime or similar runtime
                val wasmFile = File(workDir, "$baseName.wasm").absolutePath
                GeneralCommandLine("wasmtime", wasmFile)
                    .withWorkDirectory(workDir)
            }
            else -> {
                val binaryPath = File(workDir, baseName).absolutePath
                GeneralCommandLine(binaryPath)
                    .withWorkDirectory(workDir)
            }
        }

        // Add user-specified arguments
        if (configuration.arguments.isNotEmpty()) {
            runCommandLine.withParameters(*configuration.arguments.split("\\s+".toRegex()).toTypedArray())
        }

        // Add environment variables to execution
        envVars.forEach { (key, value) ->
            runCommandLine.withEnvironment(key, value)
        }

        return ProcessHandlerFactory.getInstance().createColoredProcessHandler(runCommandLine)
    }
}
