import * as vscode from 'vscode';
import * as path from 'path';
import {
    LanguageClient,
    LanguageClientOptions,
    ServerOptions,
    TransportKind
} from 'vscode-languageclient/node';

let client: LanguageClient | undefined;

export function activate(context: vscode.ExtensionContext) {
    console.log('AOEL extension activated');

    // Start LSP client if enabled
    const config = vscode.workspace.getConfiguration('aoel');
    if (config.get<boolean>('lsp.enabled', true)) {
        startLspClient(context);
    }

    // Register commands
    context.subscriptions.push(
        vscode.commands.registerCommand('aoel.run', runAoelFile),
        vscode.commands.registerCommand('aoel.format', formatAoelFile),
        vscode.commands.registerCommand('aoel.check', checkAoelFile),
        vscode.commands.registerCommand('aoel.repl', startRepl),
        vscode.commands.registerCommand('aoel.showAst', showAst)
    );

    // Format on save
    if (config.get<boolean>('format.onSave', false)) {
        context.subscriptions.push(
            vscode.workspace.onWillSaveTextDocument((event) => {
                if (event.document.languageId === 'aoel') {
                    event.waitUntil(formatDocument(event.document));
                }
            })
        );
    }

    // Status bar item
    const statusBar = vscode.window.createStatusBarItem(
        vscode.StatusBarAlignment.Right,
        100
    );
    statusBar.text = '$(symbol-function) AOEL';
    statusBar.tooltip = 'AOEL Language Support';
    statusBar.command = 'aoel.run';
    statusBar.show();
    context.subscriptions.push(statusBar);
}

function startLspClient(context: vscode.ExtensionContext) {
    const config = vscode.workspace.getConfiguration('aoel');
    const serverPath = config.get<string>('lsp.path', 'aoel-lsp');

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
        documentSelector: [{ scheme: 'file', language: 'aoel' }],
        synchronize: {
            fileEvents: vscode.workspace.createFileSystemWatcher('**/*.aoel')
        }
    };

    client = new LanguageClient(
        'aoel-lsp',
        'AOEL Language Server',
        serverOptions,
        clientOptions
    );

    client.start().catch((error) => {
        console.error('Failed to start AOEL LSP:', error);
        vscode.window.showWarningMessage(
            'AOEL Language Server not found. Some features may be limited.'
        );
    });

    context.subscriptions.push({
        dispose: () => client?.stop()
    });
}

async function runAoelFile() {
    const editor = vscode.window.activeTextEditor;
    if (!editor || editor.document.languageId !== 'aoel') {
        vscode.window.showErrorMessage('No AOEL file is open');
        return;
    }

    await editor.document.save();
    const filePath = editor.document.fileName;
    const config = vscode.workspace.getConfiguration('aoel');
    const useJit = config.get<boolean>('run.jit', false);

    const terminal = vscode.window.createTerminal('AOEL');
    terminal.show();

    if (useJit) {
        terminal.sendText('aoel run --jit "' + filePath + '"');
    } else {
        terminal.sendText('aoel run "' + filePath + '"');
    }
}

async function formatAoelFile() {
    const editor = vscode.window.activeTextEditor;
    if (!editor || editor.document.languageId !== 'aoel') {
        vscode.window.showErrorMessage('No AOEL file is open');
        return;
    }

    const formatted = await formatDocument(editor.document);
    if (formatted && formatted.length > 0) {
        await editor.edit((editBuilder) => {
            const fullRange = new vscode.Range(
                editor.document.positionAt(0),
                editor.document.positionAt(editor.document.getText().length)
            );
            editBuilder.replace(fullRange, formatted[0].newText);
        });
    }
}

async function formatDocument(
    document: vscode.TextDocument
): Promise<vscode.TextEdit[]> {
    const config = vscode.workspace.getConfiguration('aoel');
    const indentWidth = config.get<number>('format.indentWidth', 4);

    return new Promise((resolve) => {
        const { exec } = require('child_process');
        exec(
            'aoel format --indent ' + indentWidth + ' "' + document.fileName + '"',
            (error: Error | null, stdout: string, stderr: string) => {
                if (error) {
                    console.error('Format error:', stderr);
                    resolve([]);
                } else {
                    const fullRange = new vscode.Range(
                        document.positionAt(0),
                        document.positionAt(document.getText().length)
                    );
                    resolve([vscode.TextEdit.replace(fullRange, stdout)]);
                }
            }
        );
    });
}

async function checkAoelFile() {
    const editor = vscode.window.activeTextEditor;
    if (!editor || editor.document.languageId !== 'aoel') {
        vscode.window.showErrorMessage('No AOEL file is open');
        return;
    }

    await editor.document.save();
    const filePath = editor.document.fileName;

    const terminal = vscode.window.createTerminal('AOEL Check');
    terminal.show();
    terminal.sendText('aoel check "' + filePath + '"');
}

async function startRepl() {
    const terminal = vscode.window.createTerminal('AOEL REPL');
    terminal.show();
    terminal.sendText('aoel repl');
}

async function showAst() {
    const editor = vscode.window.activeTextEditor;
    if (!editor || editor.document.languageId !== 'aoel') {
        vscode.window.showErrorMessage('No AOEL file is open');
        return;
    }

    await editor.document.save();
    const filePath = editor.document.fileName;
    const fileName = path.basename(filePath);

    const { exec } = require('child_process');
    exec('aoel ast "' + filePath + '"', (error: Error | null, stdout: string, stderr: string) => {
        if (error) {
            vscode.window.showErrorMessage('AST Error: ' + stderr);
            return;
        }

        const panel = vscode.window.createWebviewPanel(
            'aoelAst',
            'AOEL AST',
            vscode.ViewColumn.Beside,
            {}
        );

        panel.webview.html = 
            '<!DOCTYPE html>' +
            '<html>' +
            '<head>' +
            '<style>' +
            'body { font-family: monospace; padding: 1rem; }' +
            'pre { white-space: pre-wrap; }' +
            '</style>' +
            '</head>' +
            '<body>' +
            '<h2>AST for ' + escapeHtml(fileName) + '</h2>' +
            '<pre>' + escapeHtml(stdout) + '</pre>' +
            '</body>' +
            '</html>';
    });
}

function escapeHtml(text: string): string {
    return text
        .replace(/&/g, '&amp;')
        .replace(/</g, '&lt;')
        .replace(/>/g, '&gt;')
        .replace(/"/g, '&quot;');
}

export function deactivate(): Thenable<void> | undefined {
    if (!client) {
        return undefined;
    }
    return client.stop();
}
