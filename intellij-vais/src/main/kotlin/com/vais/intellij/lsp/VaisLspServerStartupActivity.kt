package com.vais.intellij.lsp

import com.intellij.ide.AppLifecycleListener
import com.intellij.openapi.diagnostic.Logger

/**
 * Application lifecycle listener for Vais LSP server.
 *
 * Handles initialization when the IDE starts and ensures
 * all LSP servers are properly shut down when the IDE exits.
 */
class VaisLspServerStartupActivity : AppLifecycleListener {
    private val logger = Logger.getInstance(VaisLspServerStartupActivity::class.java)

    override fun appStarted() {
        logger.info("Vais Language Support plugin initialized")
    }

    override fun appClosing() {
        logger.info("Vais Language Support plugin shutting down - stopping all LSP servers")
        VaisLspManager.getInstance().stopAllServers()
    }
}
