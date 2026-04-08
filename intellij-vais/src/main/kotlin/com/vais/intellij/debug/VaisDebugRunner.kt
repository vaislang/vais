package com.vais.intellij.debug

import com.intellij.debugger.impl.GenericDebuggerRunner
import com.intellij.execution.configurations.RunProfile
import com.intellij.execution.configurations.RunProfileState
import com.intellij.execution.configurations.RunnerSettings
import com.intellij.execution.executors.DefaultDebugExecutor
import com.intellij.execution.runners.ExecutionEnvironment
import com.intellij.execution.ui.RunContentDescriptor
import com.intellij.openapi.diagnostic.Logger
import com.intellij.xdebugger.XDebugProcess
import com.intellij.xdebugger.XDebugProcessStarter
import com.intellij.xdebugger.XDebugSession
import com.intellij.xdebugger.XDebuggerManager

/**
 * Program runner for Vais debug sessions.
 *
 * Registered under the DefaultDebugExecutor so "Debug" actions on
 * VaisDebugConfiguration launch VaisDebugProcess, which drives the
 * vais-dap DAP server.
 */
class VaisDebugRunner : GenericDebuggerRunner() {

    private val logger = Logger.getInstance(VaisDebugRunner::class.java)

    override fun getRunnerId(): String = "VaisDebugRunner"

    override fun canRun(executorId: String, profile: RunProfile): Boolean {
        return executorId == DefaultDebugExecutor.EXECUTOR_ID &&
                profile is VaisDebugConfiguration
    }

    override fun doExecute(
        state: RunProfileState,
        environment: ExecutionEnvironment
    ): RunContentDescriptor? {
        val configuration = environment.runProfile as? VaisDebugConfiguration
            ?: return super.doExecute(state, environment)

        logger.info("Starting Vais debug session for: ${configuration.vaisFile}")

        val debuggerManager = XDebuggerManager.getInstance(environment.project)
        return debuggerManager.startSession(environment, object : XDebugProcessStarter() {
            override fun start(session: XDebugSession): XDebugProcess {
                return VaisDebugProcess(session, configuration)
            }
        }).runContentDescriptor
    }
}
