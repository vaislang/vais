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
 * Manages LSP server lifecycle per project. Provides document synchronization
 * (didOpen/didClose/didChange) and exposes the LSP server for IDE features
 * like completion, hover, go-to-definition, and references.
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

            // Initialize the server with comprehensive capabilities
            val initParams = InitializeParams().apply {
                rootUri = "file://$basePath"
                capabilities = ClientCapabilities().apply {
                    textDocument = TextDocumentClientCapabilities().apply {
                        synchronization = SynchronizationCapabilities().apply {
                            dynamicRegistration = true
                            didSave = true
                            willSave = false
                            willSaveWaitUntil = false
                        }
                        completion = CompletionCapabilities().apply {
                            completionItem = CompletionItemCapabilities().apply {
                                snippetSupport = true
                                documentationFormat = listOf(MarkupKind.MARKDOWN, MarkupKind.PLAINTEXT)
                            }
                            contextSupport = true
                        }
                        hover = HoverCapabilities().apply {
                            contentFormat = listOf(MarkupKind.MARKDOWN, MarkupKind.PLAINTEXT)
                        }
                        definition = DefinitionCapabilities().apply {
                            dynamicRegistration = true
                        }
                        references = ReferencesCapabilities().apply {
                            dynamicRegistration = true
                        }
                        formatting = FormattingCapabilities()
                        diagnostic = DiagnosticCapabilities()
                        signatureHelp = SignatureHelpCapabilities()
                        documentSymbol = DocumentSymbolCapabilities().apply {
                            dynamicRegistration = true
                        }
                        inlayHint = InlayHintCapabilities().apply {
                            dynamicRegistration = true
                        }
                        codeAction = CodeActionCapabilities().apply {
                            dynamicRegistration = true
                        }
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
 *
 * Provides convenience methods for document synchronization and
 * LSP feature requests.
 */
class VaisLspConnection(
    val server: LanguageServer,
    private val process: Process,
    val client: VaisLanguageClient
) {
    private val logger = Logger.getInstance(VaisLspConnection::class.java)
    private var documentVersion = ConcurrentHashMap<String, Int>()

    /**
     * Notify the server that a document was opened.
     */
    fun didOpen(uri: String, text: String) {
        documentVersion[uri] = 1
        server.textDocumentService.didOpen(
            DidOpenTextDocumentParams(
                TextDocumentItem(uri, "vais", 1, text)
            )
        )
    }

    /**
     * Notify the server that a document was changed.
     */
    fun didChange(uri: String, text: String) {
        val version = (documentVersion[uri] ?: 0) + 1
        documentVersion[uri] = version
        server.textDocumentService.didChange(
            DidChangeTextDocumentParams(
                VersionedTextDocumentIdentifier(uri, version),
                listOf(TextDocumentContentChangeEvent(text))
            )
        )
    }

    /**
     * Notify the server that a document was closed.
     */
    fun didClose(uri: String) {
        documentVersion.remove(uri)
        server.textDocumentService.didClose(
            DidCloseTextDocumentParams(
                TextDocumentIdentifier(uri)
            )
        )
    }

    /**
     * Notify the server that a document was saved.
     */
    fun didSave(uri: String, text: String? = null) {
        server.textDocumentService.didSave(
            DidSaveTextDocumentParams(
                TextDocumentIdentifier(uri),
                text
            )
        )
    }

    fun isAlive(): Boolean {
        return process.isAlive
    }

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
