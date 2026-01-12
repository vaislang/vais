import * as path from 'path';
import * as vscode from 'vscode';
import {
    LanguageClient,
    LanguageClientOptions,
    ServerOptions,
    TransportKind
} from 'vscode-languageclient/node';

let client: LanguageClient;

export function activate(context: vscode.ExtensionContext) {
    // Get server path from configuration
    const config = vscode.workspace.getConfiguration('aoel');
    let serverPath = config.get<string>('server.path') || 'aoel-lsp';

    // If relative path, resolve from workspace
    if (!path.isAbsolute(serverPath)) {
        const workspaceFolder = vscode.workspace.workspaceFolders?.[0];
        if (workspaceFolder) {
            const possiblePath = path.join(workspaceFolder.uri.fsPath, serverPath);
            if (require('fs').existsSync(possiblePath)) {
                serverPath = possiblePath;
            }
        }
    }

    const serverOptions: ServerOptions = {
        run: {
            command: serverPath,
            transport: TransportKind.stdio
        },
        debug: {
            command: serverPath,
            transport: TransportKind.stdio
        }
    };

    const clientOptions: LanguageClientOptions = {
        documentSelector: [
            { scheme: 'file', language: 'aoel' }
        ],
        synchronize: {
            fileEvents: vscode.workspace.createFileSystemWatcher('**/*.{aoel,v6b}')
        }
    };

    client = new LanguageClient(
        'aoel',
        'AOEL Language Server',
        serverOptions,
        clientOptions
    );

    client.start();
}

export function deactivate(): Thenable<void> | undefined {
    if (!client) {
        return undefined;
    }
    return client.stop();
}
