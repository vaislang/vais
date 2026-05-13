package com.vais.intellij.debug

import com.intellij.execution.configurations.GeneralCommandLine
import com.intellij.execution.process.ProcessHandlerFactory
import com.intellij.openapi.diagnostic.Logger
import com.intellij.xdebugger.XDebugProcess
import com.intellij.xdebugger.XDebugSession
import com.intellij.xdebugger.breakpoints.XBreakpointHandler
import com.intellij.xdebugger.breakpoints.XBreakpointType
import com.intellij.xdebugger.breakpoints.XLineBreakpoint
import com.intellij.xdebugger.evaluation.XDebuggerEditorsProvider
import com.intellij.xdebugger.frame.XExecutionStack
import com.intellij.xdebugger.frame.XStackFrame
import com.intellij.xdebugger.frame.XSuspendContext
import java.io.File
import java.io.PrintWriter

/**
 * XDebug process implementation for Vais DAP-based debugging.
 *
 * Lifecycle:
 *   1. Compiles the Vais source with debug info (--opt=0).
 *   2. Spawns vais-dap with --stdin-stdout to communicate over DAP via stdio.
 *   3. Sends DAP Initialize → Launch → ConfigurationDone handshake.
 *   4. Handles Stopped/terminated events for step, continue, and breakpoints.
 */
class VaisDebugProcess(
    session: XDebugSession,
    private val configuration: VaisDebugConfiguration
) : XDebugProcess(session) {

    private val logger = Logger.getInstance(VaisDebugProcess::class.java)

    private var dapProcess: Process? = null
    private var dapWriter: PrintWriter? = null
    private var dapSeq: Int = 1

    private val breakpointHandler = VaisLineBreakpointHandler(this)

    // -------------------------------------------------------------------------
    // XDebugProcess overrides
    // -------------------------------------------------------------------------

    override fun getEditorsProvider(): XDebuggerEditorsProvider = VaisEditorsProvider()

    @Suppress("UNCHECKED_CAST")
    override fun getBreakpointHandlers(): Array<XBreakpointHandler<*>> =
        arrayOf(breakpointHandler)

    override fun startStepOver(context: XSuspendContext?) {
        sendDapRequest("next", mapOf("threadId" to 1))
    }

    override fun startStepInto(context: XSuspendContext?) {
        sendDapRequest("stepIn", mapOf("threadId" to 1))
    }

    override fun startStepOut(context: XSuspendContext?) {
        sendDapRequest("stepOut", mapOf("threadId" to 1))
    }

    override fun resume(context: XSuspendContext?) {
        sendDapRequest("continue", mapOf("threadId" to 1))
    }

    override fun stop() {
        sendDapRequest("disconnect", mapOf("restart" to false))
        dapProcess?.destroyForcibly()
    }

    override fun sessionInitialized() {
        startDapSession()
    }

    // -------------------------------------------------------------------------
    // DAP session management
    // -------------------------------------------------------------------------

    private fun startDapSession() {
        val vaisFile = File(configuration.vaisFile)
        val workDir = configuration.effectiveWorkingDirectory()
        val baseName = vaisFile.nameWithoutExtension

        // Step 1: Compile with no optimization for better debug experience.
        logger.info("Compiling ${configuration.vaisFile} with debug symbols")
        val compileCmd = GeneralCommandLine()
            .withExePath(configuration.compilerPath)
            .withParameters("build", configuration.vaisFile, "-o", baseName, "--opt=0")
            .withWorkDirectory(workDir)

        try {
            val compileHandler = ProcessHandlerFactory.getInstance().createProcessHandler(compileCmd)
            compileHandler.startNotify()
            compileHandler.waitFor(30_000L)
            if (compileHandler.exitCode != 0) {
                session.reportError("Vais compilation failed (exit ${compileHandler.exitCode})")
                return
            }
        } catch (e: Exception) {
            session.reportError("Failed to compile Vais file: ${e.message}")
            return
        }

        // Step 2: Launch vais-dap DAP server over stdio.
        val binaryPath = File(workDir, baseName).absolutePath
        logger.info("Launching vais-dap for binary: $binaryPath")

        val dapCmdParts = mutableListOf(
            findDapBinary(configuration.dapServerPath),
            "--stdin-stdout",
            "--program", binaryPath,
            "--source", configuration.vaisFile
        )
        if (configuration.programArguments.isNotEmpty()) {
            dapCmdParts += listOf("--args") +
                    configuration.programArguments.split("\\s+".toRegex())
        }

        try {
            val pb = ProcessBuilder(dapCmdParts)
                .directory(File(workDir))
                .redirectErrorStream(false)
            dapProcess = pb.start()
            dapWriter = PrintWriter(dapProcess!!.outputStream, true)

            // Start async reader for DAP responses.
            Thread(::readDapResponses, "vais-dap-reader").also {
                it.isDaemon = true
                it.start()
            }

            // DAP handshake: Initialize → Launch → ConfigurationDone.
            sendDapRequest(
                "initialize", mapOf(
                    "adapterID" to "vais-dap",
                    "clientID" to "intellij-vais",
                    "linesStartAt1" to true,
                    "columnsStartAt1" to true,
                    "pathFormat" to "path",
                    "supportsVariableType" to true,
                    "supportsRunInTerminalRequest" to false
                )
            )
            sendDapRequest(
                "launch", mapOf(
                    "program" to binaryPath,
                    "stopOnEntry" to false,
                    "noDebug" to false
                )
            )
            sendDapRequest("configurationDone", emptyMap<String, Any>())

            // Register breakpoints that were set before the session started.
            breakpointHandler.flushPendingBreakpoints()

            logger.info("vais-dap session started")
        } catch (e: Exception) {
            session.reportError("Failed to launch vais-dap: ${e.message}")
            logger.error("Failed to launch vais-dap", e)
        }
    }

    private fun findDapBinary(configured: String): String {
        if (configured != "vais-dap") return configured
        val candidates = listOf(
            System.getProperty("user.home") + "/.cargo/bin/vais-dap",
            "/usr/local/bin/vais-dap",
            "/opt/homebrew/bin/vais-dap",
            "/usr/bin/vais-dap"
        )
        return candidates.firstOrNull { File(it).canExecute() } ?: "vais-dap"
    }

    // -------------------------------------------------------------------------
    // DAP stdio communication (Content-Length framing per DAP spec)
    // -------------------------------------------------------------------------

    internal fun sendBreakpoint(file: String, line: Int) {
        sendDapRequest(
            "setBreakpoints", mapOf(
                "source" to mapOf("path" to file),
                "breakpoints" to listOf(mapOf("line" to line))
            )
        )
    }

    internal fun clearBreakpoint(file: String, line: Int) {
        sendDapRequest(
            "setBreakpoints", mapOf(
                "source" to mapOf("path" to file),
                "breakpoints" to emptyList<Map<String, Any>>()
            )
        )
    }

    private fun sendDapRequest(command: String, args: Map<String, Any>) {
        val argsJson = args.entries.joinToString(",") { (k, v) -> "\"$k\":${toJson(v)}" }
        val body = """{"seq":${dapSeq++},"type":"request","command":"$command","arguments":{$argsJson}}"""
        val message = "Content-Length: ${body.length}\r\n\r\n$body"
        try {
            dapWriter?.print(message)
            dapWriter?.flush()
            logger.debug("DAP -> $command")
        } catch (e: Exception) {
            logger.warn("Failed to send DAP request '$command': ${e.message}")
        }
    }

    private fun toJson(value: Any?): String = when (value) {
        null -> "null"
        is Boolean -> value.toString()
        is Number -> value.toString()
        is String -> "\"${value.replace("\\", "\\\\").replace("\"", "\\\"")}\""
        is List<*> -> "[${value.joinToString(",") { toJson(it) }}]"
        is Map<*, *> -> "{${value.entries.joinToString(",") { (k, v) -> "\"$k\":${toJson(v)}" }}}"
        else -> "\"$value\""
    }

    private fun readDapResponses() {
        val reader = dapProcess?.inputStream?.bufferedReader() ?: return
        try {
            while (true) {
                val headerLine = reader.readLine() ?: break
                if (!headerLine.startsWith("Content-Length:")) continue
                val length = headerLine.removePrefix("Content-Length:").trim().toIntOrNull() ?: continue
                reader.readLine() // blank separator
                val buf = CharArray(length)
                var offset = 0
                while (offset < length) {
                    val read = reader.read(buf, offset, length - offset)
                    if (read == -1) break
                    offset += read
                }
                handleDapMessage(String(buf, 0, offset))
            }
        } catch (e: Exception) {
            logger.debug("DAP reader ended: ${e.message}")
        }
    }

    private fun handleDapMessage(json: String) {
        logger.debug("DAP <- $json")
        when {
            json.contains("\"event\":\"stopped\"") ->
                session.positionReached(VaisSuspendContext())

            json.contains("\"event\":\"terminated\"") ||
                    json.contains("\"event\":\"exited\"") ->
                session.stop()
        }
    }
}

// ---------------------------------------------------------------------------
// Breakpoint handler
// ---------------------------------------------------------------------------

@Suppress("UNCHECKED_CAST")
class VaisLineBreakpointHandler(
    private val process: VaisDebugProcess
) : XBreakpointHandler<XLineBreakpoint<*>>(
    VaisLineBreakpointType::class.java as Class<out XBreakpointType<XLineBreakpoint<*>, *>>
) {
    private val pending = mutableListOf<XLineBreakpoint<*>>()
    private var ready = false

    fun flushPendingBreakpoints() {
        ready = true
        pending.forEach { registerBreakpoint(it) }
        pending.clear()
    }

    override fun registerBreakpoint(breakpoint: XLineBreakpoint<*>) {
        if (!ready) {
            pending.add(breakpoint)
            return
        }
        val path = breakpoint.presentableFilePath ?: return
        process.sendBreakpoint(path, breakpoint.line + 1) // DAP lines are 1-based
    }

    override fun unregisterBreakpoint(breakpoint: XLineBreakpoint<*>, temporary: Boolean) {
        val path = breakpoint.presentableFilePath ?: return
        process.clearBreakpoint(path, breakpoint.line + 1)
    }
}

// ---------------------------------------------------------------------------
// Suspend context — minimal, used to notify the session that execution stopped
// ---------------------------------------------------------------------------

class VaisSuspendContext : XSuspendContext() {
    override fun getActiveExecutionStack(): XExecutionStack = VaisExecutionStack()
}

class VaisExecutionStack : XExecutionStack("Vais Thread") {
    override fun getTopFrame(): XStackFrame? = null

    override fun computeStackFrames(
        firstFrameIndex: Int,
        container: XExecutionStack.XStackFrameContainer
    ) {
        container.addStackFrames(emptyList(), true)
    }
}
