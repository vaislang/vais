package com.vais.intellij.lsp

import com.intellij.openapi.diagnostic.Logger
import com.intellij.openapi.fileEditor.FileEditorManager
import com.intellij.openapi.fileEditor.FileEditorManagerListener
import com.intellij.openapi.project.Project
import com.intellij.openapi.project.ProjectManagerListener
import com.intellij.openapi.vfs.VirtualFile

/**
 * Project listener for Vais LSP server lifecycle management.
 *
 * Handles LSP server startup and shutdown per project.
 * Auto-starts the LSP server when a .vais file is opened.
 */
class VaisLspProjectListener : ProjectManagerListener {
    private val logger = Logger.getInstance(VaisLspProjectListener::class.java)

    override fun projectOpened(project: Project) {
        logger.info("Project opened: ${project.name} - registering Vais file open listener")

        // Check if any .vais files are already open
        val openFiles = FileEditorManager.getInstance(project).openFiles
        if (openFiles.any { it.extension == "vais" }) {
            startLspServer(project)
        }

        // Listen for file open events to auto-start LSP on first .vais file
        project.messageBus.connect().subscribe(
            FileEditorManagerListener.FILE_EDITOR_MANAGER,
            object : FileEditorManagerListener {
                override fun fileOpened(source: FileEditorManager, file: VirtualFile) {
                    if (file.extension == "vais") {
                        startLspServer(project)
                    }
                }
            }
        )
    }

    override fun projectClosing(project: Project) {
        logger.info("Project closing: ${project.name} - stopping Vais LSP server")
        VaisLspManager.getInstance().stopServer(project)
    }

    private fun startLspServer(project: Project) {
        val manager = VaisLspManager.getInstance()
        if (manager.getConnection(project) == null) {
            logger.info("Auto-starting Vais LSP server for project: ${project.name}")
            val connection = manager.startServer(project)
            if (connection != null) {
                logger.info("Vais LSP server started successfully")
            } else {
                logger.warn("Failed to start Vais LSP server - vais-lsp binary not found")
            }
        }
    }
}
