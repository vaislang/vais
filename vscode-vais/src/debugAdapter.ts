import * as vscode from 'vscode';
import * as path from 'path';
import * as fs from 'fs';

/**
 * Debug Adapter for Vais language.
 *
 * This adapter wraps the vais-dap server and provides the bridge between
 * VSCode's debug protocol and the DAP server.
 */
export class VaisDebugAdapterDescriptorFactory implements vscode.DebugAdapterDescriptorFactory {
    createDebugAdapterDescriptor(
        _session: vscode.DebugSession,
        _executable: vscode.DebugAdapterExecutable | undefined
    ): vscode.ProviderResult<vscode.DebugAdapterDescriptor> {
        // Get configured path to vais-dap
        const config = vscode.workspace.getConfiguration('vais');
        let dapPath = config.get<string>('debugAdapter.path', 'vais-dap');

        // If it's just "vais-dap", try to find it in the workspace
        if (dapPath === 'vais-dap') {
            const workspaceRoot = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
            if (workspaceRoot) {
                const localPath = path.join(workspaceRoot, 'target', 'release', 'vais-dap');
                if (fs.existsSync(localPath)) {
                    dapPath = localPath;
                }
            }
        }

        // Return executable descriptor
        return new vscode.DebugAdapterExecutable(dapPath, []);
    }
}

/**
 * Configuration provider for Vais debug sessions.
 *
 * Provides default values and resolves variables in launch configurations.
 */
export class VaisDebugConfigurationProvider implements vscode.DebugConfigurationProvider {
    /**
     * Massage a debug configuration just before a debug session is being launched.
     */
    resolveDebugConfiguration(
        folder: vscode.WorkspaceFolder | undefined,
        config: vscode.DebugConfiguration,
        _token?: vscode.CancellationToken
    ): vscode.ProviderResult<vscode.DebugConfiguration> {
        // If launch.json is missing or empty
        if (!config.type && !config.request && !config.name) {
            const editor = vscode.window.activeTextEditor;
            if (editor && editor.document.languageId === 'vais') {
                config.type = 'vais';
                config.name = 'Launch';
                config.request = 'launch';
                config.program = '${file}';
                config.stopOnEntry = true;
            }
        }

        // Ensure required fields are present
        if (!config.program) {
            return vscode.window.showInformationMessage(
                'Cannot find a program to debug'
            ).then(_ => {
                return undefined;
            });
        }

        // Set defaults
        if (config.stopOnEntry === undefined) {
            config.stopOnEntry = true;
        }
        if (config.autoCompile === undefined) {
            config.autoCompile = true;
        }
        if (config.optLevel === undefined) {
            config.optLevel = 0;
        }
        if (config.cwd === undefined && folder) {
            config.cwd = folder.uri.fsPath;
        }

        return config;
    }
}

/**
 * Inline values provider for Vais debugging.
 *
 * Shows variable values inline in the editor during debugging.
 */
export class VaisDebugAdapterInlineValuesProvider implements vscode.InlineValuesProvider {
    onDidChangeInlineValues?: vscode.Event<void> | undefined;

    provideInlineValues(
        document: vscode.TextDocument,
        viewport: vscode.Range,
        context: vscode.InlineValueContext,
        _token: vscode.CancellationToken
    ): vscode.ProviderResult<vscode.InlineValue[]> {
        const allValues: vscode.InlineValue[] = [];

        for (let line = viewport.start.line; line < context.stoppedLocation.end.line; line++) {
            const lineText = document.lineAt(line).text;

            // Look for variable assignments (simple heuristic)
            // Pattern: identifier := value
            const assignmentRegex = /(\w+)\s*:=\s*/g;
            let match;

            while (match = assignmentRegex.exec(lineText)) {
                const varName = match[1];
                const varRange = new vscode.Range(
                    line,
                    match.index,
                    line,
                    match.index + varName.length
                );

                // Request variable value from debugger
                allValues.push(new vscode.InlineValueVariableLookup(varRange, varName, false));
            }
        }

        return allValues;
    }
}

/**
 * Activate debug-related features.
 */
export function activateDebugger(context: vscode.ExtensionContext) {
    // Register debug adapter factory
    context.subscriptions.push(
        vscode.debug.registerDebugAdapterDescriptorFactory(
            'vais',
            new VaisDebugAdapterDescriptorFactory()
        )
    );

    // Register debug configuration provider
    context.subscriptions.push(
        vscode.debug.registerDebugConfigurationProvider(
            'vais',
            new VaisDebugConfigurationProvider()
        )
    );

    // Register inline values provider
    context.subscriptions.push(
        vscode.languages.registerInlineValuesProvider(
            { language: 'vais' },
            new VaisDebugAdapterInlineValuesProvider()
        )
    );

    // Register command to generate launch.json
    context.subscriptions.push(
        vscode.commands.registerCommand('vais.debug.generateLaunchConfig', () => {
            generateLaunchConfig();
        })
    );
}

/**
 * Generate a launch.json configuration for Vais debugging.
 */
async function generateLaunchConfig() {
    const workspaceFolder = vscode.workspace.workspaceFolders?.[0];
    if (!workspaceFolder) {
        vscode.window.showErrorMessage('No workspace folder open');
        return;
    }

    const vscodeDir = path.join(workspaceFolder.uri.fsPath, '.vscode');
    const launchJsonPath = path.join(vscodeDir, 'launch.json');

    // Create .vscode directory if it doesn't exist
    if (!fs.existsSync(vscodeDir)) {
        fs.mkdirSync(vscodeDir, { recursive: true });
    }

    const launchConfig = {
        version: '0.2.0',
        configurations: [
            {
                type: 'vais',
                request: 'launch',
                name: 'Debug Vais Program',
                program: '${workspaceFolder}/main.vais',
                stopOnEntry: true,
                autoCompile: true,
                optLevel: 0,
                args: [],
                cwd: '${workspaceFolder}'
            },
            {
                type: 'vais',
                request: 'launch',
                name: 'Debug Current File',
                program: '${file}',
                stopOnEntry: true,
                autoCompile: true
            },
            {
                type: 'vais',
                request: 'attach',
                name: 'Attach to Process',
                pid: 0,
                stopOnAttach: true
            }
        ]
    };

    const launchJsonContent = JSON.stringify(launchConfig, null, 2);

    // Check if launch.json already exists
    if (fs.existsSync(launchJsonPath)) {
        const choice = await vscode.window.showWarningMessage(
            'launch.json already exists. Do you want to overwrite it?',
            'Yes',
            'No'
        );
        if (choice !== 'Yes') {
            return;
        }
    }

    // Write launch.json
    fs.writeFileSync(launchJsonPath, launchJsonContent);

    // Open the file
    const document = await vscode.workspace.openTextDocument(launchJsonPath);
    await vscode.window.showTextDocument(document);

    vscode.window.showInformationMessage('Generated launch.json for Vais debugging');
}
