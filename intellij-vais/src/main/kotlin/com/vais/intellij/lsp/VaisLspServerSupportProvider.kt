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
 * LSP server support provider for Vais language.
 *
 * Entry point for IntelliJ's LSP integration. Manages one LSP connection
 * per project, delegates to VaisLspManager for lifecycle control.
 */
class VaisLspServerSupportProvider {

    fun fileSupported(extension: String?): Boolean = extension == "vais"

    fun ensureStarted(project: Project): VaisLspConnection? =
        VaisLspManager.getInstance().startServer(project)

    fun getConnection(project: Project): VaisLspConnection? =
        VaisLspManager.getInstance().getConnection(project)
}

/**
 * LSP manager singleton: one connection per project basePath.
 *
 * Manages the connection to vais-lsp binary via LSP4J.
 * Provides document synchronization (didOpen/didChange/didClose/didSave)
 * and exposes the server proxy for completion, hover, go-to-definition,
 * diagnostics, and other LSP features.
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

        servers[basePath]?.takeIf { it.isAlive() }?.let { return it }

        val descriptor = VaisLspServerDescriptor(project)
        if (!descriptor.isLspAvailable()) {
            logger.warn("vais-lsp binary not found, skipping LSP startup")
            return null
        }

        return try {
            val commandLine = descriptor.createCommandLine()
            val process = commandLine.createProcess()

            val client = VaisLanguageClient()
            val launcher: Launcher<LanguageServer> = LSPLauncher.createClientLauncher(
                client,
                process.inputStream,
                process.outputStream
            )

            launcher.startListening()
            val server = launcher.remoteProxy

            // Initialize with full client capabilities for completion, hover,
            // goto-definition, diagnostics, inlay hints, signature help, code actions.
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
                        formatting = FormattingCapabilities().apply {
                            dynamicRegistration = true
                        }
                        diagnostic = DiagnosticCapabilities().apply {
                            dynamicRegistration = true
                        }
                        signatureHelp = SignatureHelpCapabilities().apply {
                            dynamicRegistration = true
                        }
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
 * An active LSP connection to vais-lsp.
 *
 * Wraps the LSP4J server proxy and provides convenience methods for
 * document synchronization notifications required by the protocol.
 */
class VaisLspConnection(
    val server: LanguageServer,
    private val process: Process,
    val client: VaisLanguageClient
) {
    private val logger = Logger.getInstance(VaisLspConnection::class.java)
    private val documentVersion = ConcurrentHashMap<String, Int>()

    fun didOpen(uri: String, text: String) {
        documentVersion[uri] = 1
        server.textDocumentService.didOpen(
            DidOpenTextDocumentParams(
                TextDocumentItem(uri, "vais", 1, text)
            )
        )
    }

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

    fun didClose(uri: String) {
        documentVersion.remove(uri)
        server.textDocumentService.didClose(
            DidCloseTextDocumentParams(TextDocumentIdentifier(uri))
        )
    }

    fun didSave(uri: String, text: String? = null) {
        server.textDocumentService.didSave(
            DidSaveTextDocumentParams(TextDocumentIdentifier(uri), text)
        )
    }

    fun isAlive(): Boolean = process.isAlive

    fun shutdown() {
        try {
            server.shutdown().get()
            server.exit()
            process.destroyForcibly()
        } catch (e: Exception) {
            logger.warn("Error shutting down LSP server: ${e.message}")
        }
    }
}

/**
 * LSP client callback receiver for messages pushed by the server.
 *
 * Diagnostics are forwarded to registered handlers so the IDE
 * can annotate editors with errors and warnings.
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
        params?.let { logger.info("LSP Message [${it.type}]: ${it.message}") }
    }

    override fun showMessageRequest(
        params: ShowMessageRequestParams?
    ): CompletableFuture<MessageActionItem> =
        CompletableFuture.completedFuture(null)

    override fun logMessage(params: MessageParams?) {
        params?.let { logger.debug("LSP Log [${it.type}]: ${it.message}") }
    }
}
