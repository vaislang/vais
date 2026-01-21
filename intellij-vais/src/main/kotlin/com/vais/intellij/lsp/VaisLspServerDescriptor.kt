package com.vais.intellij.lsp

import com.intellij.execution.configurations.GeneralCommandLine
import com.intellij.openapi.diagnostic.Logger
import com.intellij.openapi.project.Project
import java.io.File

/**
 * LSP server configuration for Vais language server.
 *
 * Manages the connection to vais-lsp binary.
 */
class VaisLspServerDescriptor(val project: Project) {
    private val logger = Logger.getInstance(VaisLspServerDescriptor::class.java)

    fun isSupportedFile(extension: String?): Boolean {
        return extension == "vais"
    }

    fun createCommandLine(): GeneralCommandLine {
        val lspPath = findLspBinary()
        logger.info("Using LSP binary: $lspPath")
        return GeneralCommandLine(lspPath)
            .withWorkDirectory(project.basePath)
            .withCharset(Charsets.UTF_8)
    }

    fun findLspBinary(): String {
        // Try to find vais-lsp in common locations
        val possiblePaths = listOf(
            // Cargo install location
            System.getProperty("user.home") + "/.cargo/bin/vais-lsp",
            // Project local build
            project.basePath + "/target/release/vais-lsp",
            project.basePath + "/target/debug/vais-lsp",
            // macOS specific
            "/usr/local/bin/vais-lsp",
            "/opt/homebrew/bin/vais-lsp",
            // Linux specific
            "/usr/bin/vais-lsp"
        )

        for (path in possiblePaths) {
            val file = File(path)
            if (file.exists() && file.canExecute()) {
                return file.absolutePath
            }
        }

        // Try which command on Unix systems
        try {
            val process = ProcessBuilder("which", "vais-lsp")
                .redirectErrorStream(true)
                .start()
            val result = process.inputStream.bufferedReader().readText().trim()
            if (result.isNotEmpty() && File(result).exists()) {
                return result
            }
        } catch (e: Exception) {
            logger.warn("Could not run 'which' command: ${e.message}")
        }

        // Default to PATH lookup
        return "vais-lsp"
    }

    fun isLspAvailable(): Boolean {
        return try {
            val path = findLspBinary()
            File(path).exists() || path == "vais-lsp"
        } catch (e: Exception) {
            false
        }
    }
}
