import * as path from 'path';
import * as fs from 'fs';
import * as vscode from 'vscode';
import {
    LanguageClient,
    LanguageClientOptions,
    ServerOptions,
    Executable,
} from 'vscode-languageclient/node';

let client: LanguageClient | undefined;

export function activate(context: vscode.ExtensionContext) {
    // Get configuration
    const config = vscode.workspace.getConfiguration('vais');
    const serverPath = config.get<string>('languageServer.path', 'vais-lsp');

    // Find the LSP server executable
    let command = serverPath;

    // If it's just "vais-lsp", try to find it in PATH or in the workspace
    if (command === 'vais-lsp') {
        // Try to find in workspace target/release directory
        const workspaceRoot = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
        if (workspaceRoot) {
            const localPath = path.join(workspaceRoot, 'target', 'release', 'vais-lsp');
            if (fs.existsSync(localPath)) {
                command = localPath;
            }
        }
    }

    // Configure the server executable
    const serverExecutable: Executable = {
        command: command,
        args: [],
    };

    const serverOptions: ServerOptions = {
        run: serverExecutable,
        debug: serverExecutable,
    };

    // Configure the client options
    const clientOptions: LanguageClientOptions = {
        documentSelector: [
            { scheme: 'file', language: 'vais' },
            { scheme: 'untitled', language: 'vais' }
        ],
        synchronize: {
            // Notify the server about file changes to '.vais' files in the workspace
            fileEvents: vscode.workspace.createFileSystemWatcher('**/*.vais')
        },
    };

    // Create the language client
    client = new LanguageClient(
        'vaisLanguageServer',
        'Vais Language Server',
        serverOptions,
        clientOptions
    );

    // Start the client and language server
    client.start().catch((error) => {
        vscode.window.showErrorMessage(
            `Failed to start Vais Language Server: ${error.message}\n\n` +
            `Make sure 'vais-lsp' is installed and accessible in your PATH, or configure ` +
            `the path in settings (vais.languageServer.path).`
        );
    });

    // Register configuration change handler
    context.subscriptions.push(
        vscode.workspace.onDidChangeConfiguration((e) => {
            if (e.affectsConfiguration('vais.languageServer.path') ||
                e.affectsConfiguration('vais.trace.server')) {
                vscode.window.showInformationMessage(
                    'Vais language server configuration changed. Please reload the window for changes to take effect.',
                    'Reload'
                ).then((selection) => {
                    if (selection === 'Reload') {
                        vscode.commands.executeCommand('workbench.action.reloadWindow');
                    }
                });
            }
        })
    );

    // Add status bar item
    const statusBarItem = vscode.window.createStatusBarItem(
        vscode.StatusBarAlignment.Right,
        100
    );
    statusBarItem.text = '$(check) Vais LSP';
    statusBarItem.tooltip = 'Vais Language Server is active';
    statusBarItem.show();
    context.subscriptions.push(statusBarItem);

    // Update status bar when server state changes
    if (client) {
        client.onDidChangeState((event) => {
            if (event.newState === 2) { // State.Running
                statusBarItem.text = '$(check) Vais LSP';
                statusBarItem.tooltip = 'Vais Language Server is running';
                statusBarItem.color = undefined;
            } else if (event.newState === 1) { // State.Starting
                statusBarItem.text = '$(sync~spin) Vais LSP';
                statusBarItem.tooltip = 'Vais Language Server is starting...';
                statusBarItem.color = undefined;
            } else { // State.Stopped
                statusBarItem.text = '$(x) Vais LSP';
                statusBarItem.tooltip = 'Vais Language Server is not running';
                statusBarItem.color = new vscode.ThemeColor('errorForeground');
            }
        });
    }
}

export function deactivate(): Thenable<void> | undefined {
    if (!client) {
        return undefined;
    }
    return client.stop();
}
