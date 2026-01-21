package com.vais.intellij.lsp

import com.intellij.openapi.diagnostic.Logger
import com.intellij.openapi.project.Project
import com.intellij.openapi.project.ProjectManagerListener

/**
 * Project listener for Vais LSP server lifecycle management.
 *
 * Handles LSP server startup and shutdown per project.
 */
class VaisLspProjectListener : ProjectManagerListener {
    private val logger = Logger.getInstance(VaisLspProjectListener::class.java)

    override fun projectOpened(project: Project) {
        logger.info("Project opened: ${project.name} - Vais LSP will start on .vais file open")
    }

    override fun projectClosing(project: Project) {
        logger.info("Project closing: ${project.name}")
    }
}
