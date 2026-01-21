package com.vais.intellij.lsp

import com.intellij.openapi.diagnostic.Logger
import com.intellij.openapi.project.Project
import org.eclipse.lsp4j.*
import org.eclipse.lsp4j.jsonrpc.Launcher
import org.eclipse.lsp4j.launch.LSPLauncher
import org.eclipse.lsp4j.services.LanguageClient
import org.eclipse.lsp4j.services.LanguageServer
import java.util.concurrent.CompletableFuture
import java.util.concurrent.ConcurrentHashMap

/**
 * LSP client and server manager for Vais language.
 *
 * Manages LSP server lifecycle per project.
 */
class VaisLspManager private constructor() {
    private val logger = Logger.getInstance(VaisLspManager::class.java)
    private val servers = ConcurrentHashMap<String, VaisLspConnection>()

    companion object {
        @Volatile
        private var instance: VaisLspManager? = null

        fun getInstance(): VaisLspManager {
            return instance ?: synchronized(this) {
                instance ?: VaisLspManager().also { instance = it }
            }
        }
    }

    fun getConnection(project: Project): VaisLspConnection? {
        return servers[project.basePath]
    }

    fun startServer(project: Project): VaisLspConnection? {
        val basePath = project.basePath ?: return null

        if (servers.containsKey(basePath)) {
            return servers[basePath]
        }

        val descriptor = VaisLspServerDescriptor(project)
        if (!descriptor.isLspAvailable()) {
            logger.warn("vais-lsp binary not found")
            return null
        }

        return try {
            val commandLine = descriptor.createCommandLine()
            val process = commandLine.createProcess()

            val client = VaisLanguageClient()
            val launcher = LSPLauncher.createClientLauncher(
                client,
                process.inputStream,
                process.outputStream
            )

            launcher.startListening()
            val server = launcher.remoteProxy

            // Initialize the server
            val initParams = InitializeParams().apply {
                rootUri = "file://$basePath"
                capabilities = ClientCapabilities().apply {
                    textDocument = TextDocumentClientCapabilities().apply {
                        completion = CompletionCapabilities().apply {
                            completionItem = CompletionItemCapabilities().apply {
                                snippetSupport = true
                            }
                        }
                        hover = HoverCapabilities()
                        definition = DefinitionCapabilities()
                        references = ReferencesCapabilities()
                        formatting = FormattingCapabilities()
                        diagnostic = DiagnosticCapabilities()
                    }
                }
            }

            server.initialize(initParams).thenAccept { result ->
                logger.info("Vais LSP server initialized: ${result.capabilities}")
                server.initialized(InitializedParams())
            }

            val connection = VaisLspConnection(server, process, client)
            servers[basePath] = connection
            connection
        } catch (e: Exception) {
            logger.error("Failed to start Vais LSP server", e)
            null
        }
    }

    fun stopServer(project: Project) {
        val basePath = project.basePath ?: return
        servers.remove(basePath)?.shutdown()
    }

    fun stopAllServers() {
        servers.values.forEach { it.shutdown() }
        servers.clear()
    }
}

/**
 * Represents an active LSP connection.
 */
class VaisLspConnection(
    val server: LanguageServer,
    private val process: Process,
    val client: VaisLanguageClient
) {
    private val logger = Logger.getInstance(VaisLspConnection::class.java)

    fun shutdown() {
        try {
            server.shutdown().get()
            server.exit()
            process.destroyForcibly()
        } catch (e: Exception) {
            logger.warn("Error shutting down LSP server", e)
        }
    }
}

/**
 * LSP client implementation for receiving messages from the server.
 */
class VaisLanguageClient : LanguageClient {
    private val logger = Logger.getInstance(VaisLanguageClient::class.java)

    var diagnosticsHandler: ((PublishDiagnosticsParams) -> Unit)? = null

    override fun telemetryEvent(obj: Any?) {
        logger.debug("Telemetry: $obj")
    }

    override fun publishDiagnostics(diagnostics: PublishDiagnosticsParams?) {
        diagnostics?.let { params ->
            logger.info("Diagnostics for ${params.uri}: ${params.diagnostics.size} issues")
            diagnosticsHandler?.invoke(params)
        }
    }

    override fun showMessage(params: MessageParams?) {
        params?.let {
            logger.info("LSP Message [${it.type}]: ${it.message}")
        }
    }

    override fun showMessageRequest(params: ShowMessageRequestParams?): CompletableFuture<MessageActionItem> {
        return CompletableFuture.completedFuture(null)
    }

    override fun logMessage(params: MessageParams?) {
        params?.let {
            logger.debug("LSP Log [${it.type}]: ${it.message}")
        }
    }
}
