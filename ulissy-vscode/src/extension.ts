// ULissy VS Code Extension
// A Programming Language for Moving Machines

import * as vscode from 'vscode';
import * as path from 'path';
import * as cp from 'child_process';
import {
    LanguageClient,
    LanguageClientOptions,
    ServerOptions,
    TransportKind
} from 'vscode-languageclient/node';

let client: LanguageClient | undefined;
let outputChannel: vscode.OutputChannel;
let diagnosticCollection: vscode.DiagnosticCollection;

// ============================================================================
// ACTIVATION
// ============================================================================

export function activate(context: vscode.ExtensionContext) {
    console.log('ULissy extension activating...');
    
    // Create output channel
    outputChannel = vscode.window.createOutputChannel('ULissy');
    context.subscriptions.push(outputChannel);
    
    // Create diagnostic collection
    diagnosticCollection = vscode.languages.createDiagnosticCollection('ulissy');
    context.subscriptions.push(diagnosticCollection);
    
    // Register commands
    registerCommands(context);
    
    // Start language server if enabled
    const config = vscode.workspace.getConfiguration('ulissy');
    if (config.get<boolean>('enableLSP', true)) {
        startLanguageServer(context);
    }
    
    // Register save handler for type checking
    if (config.get<boolean>('checkOnSave', true)) {
        context.subscriptions.push(
            vscode.workspace.onDidSaveTextDocument(onDocumentSave)
        );
    }
    
    // Show welcome message on first activation
    showWelcomeMessage(context);
    
    outputChannel.appendLine('ULissy extension activated!');
    outputChannel.appendLine(`Compiler path: ${getCompilerPath()}`);
}

export function deactivate(): Thenable<void> | undefined {
    if (!client) {
        return undefined;
    }
    return client.stop();
}

// ============================================================================
// COMMANDS
// ============================================================================

function registerCommands(context: vscode.ExtensionContext) {
    // Build command
    context.subscriptions.push(
        vscode.commands.registerCommand('ulissy.build', () => {
            const editor = vscode.window.activeTextEditor;
            if (editor && editor.document.languageId === 'ulissy') {
                buildFile(editor.document.uri);
            } else {
                vscode.window.showWarningMessage('No ULissy file is open');
            }
        })
    );
    
    // Check command
    context.subscriptions.push(
        vscode.commands.registerCommand('ulissy.check', () => {
            const editor = vscode.window.activeTextEditor;
            if (editor && editor.document.languageId === 'ulissy') {
                checkFile(editor.document.uri);
            } else {
                vscode.window.showWarningMessage('No ULissy file is open');
            }
        })
    );
    
    // Run command
    context.subscriptions.push(
        vscode.commands.registerCommand('ulissy.run', () => {
            const editor = vscode.window.activeTextEditor;
            if (editor && editor.document.languageId === 'ulissy') {
                runFile(editor.document.uri);
            } else {
                vscode.window.showWarningMessage('No ULissy file is open');
            }
        })
    );
    
    // Show tokens (debug)
    context.subscriptions.push(
        vscode.commands.registerCommand('ulissy.showTokens', () => {
            const editor = vscode.window.activeTextEditor;
            if (editor && editor.document.languageId === 'ulissy') {
                showTokens(editor.document.uri);
            }
        })
    );
    
    // Show AST (debug)
    context.subscriptions.push(
        vscode.commands.registerCommand('ulissy.showAST', () => {
            const editor = vscode.window.activeTextEditor;
            if (editor && editor.document.languageId === 'ulissy') {
                showAST(editor.document.uri);
            }
        })
    );
    
    // New project
    context.subscriptions.push(
        vscode.commands.registerCommand('ulissy.newProject', async () => {
            const name = await vscode.window.showInputBox({
                prompt: 'Project name',
                placeHolder: 'my-ulissy-app'
            });
            if (name) {
                createNewProject(name);
            }
        })
    );
    
    // Restart server
    context.subscriptions.push(
        vscode.commands.registerCommand('ulissy.restartServer', () => {
            restartLanguageServer(context);
        })
    );
}

// ============================================================================
// BUILD / CHECK / RUN
// ============================================================================

async function buildFile(uri: vscode.Uri) {
    const filePath = uri.fsPath;
    const compilerPath = getCompilerPath();
    
    outputChannel.clear();
    outputChannel.show(true);
    outputChannel.appendLine(`Building: ${filePath}`);
    outputChannel.appendLine('─'.repeat(60));
    
    try {
        const result = await runCompiler(['build', filePath]);
        
        if (result.exitCode === 0) {
            vscode.window.showInformationMessage('✓ Build successful!');
            outputChannel.appendLine('\n✓ Build completed successfully');
            
            // Parse output directory from result
            const outputMatch = result.stdout.match(/Output: (.+)/);
            if (outputMatch) {
                outputChannel.appendLine(`Output: ${outputMatch[1]}`);
            }
        } else {
            vscode.window.showErrorMessage('Build failed. See output for details.');
            parseDiagnostics(uri, result.stderr || result.stdout);
        }
        
        outputChannel.appendLine(result.stdout);
        if (result.stderr) {
            outputChannel.appendLine(result.stderr);
        }
    } catch (error) {
        handleCompilerError(error);
    }
}

async function checkFile(uri: vscode.Uri) {
    const filePath = uri.fsPath;
    
    outputChannel.clear();
    outputChannel.show(true);
    outputChannel.appendLine(`Type checking: ${filePath}`);
    outputChannel.appendLine('─'.repeat(60));
    
    try {
        const result = await runCompiler(['check', filePath]);
        
        diagnosticCollection.clear();
        
        if (result.exitCode === 0) {
            vscode.window.showInformationMessage('✓ Type check passed!');
            outputChannel.appendLine('\n✓ No errors found');
        } else {
            vscode.window.showWarningMessage('Type check found issues');
            parseDiagnostics(uri, result.stderr || result.stdout);
        }
        
        outputChannel.appendLine(result.stdout);
        if (result.stderr) {
            outputChannel.appendLine(result.stderr);
        }
    } catch (error) {
        handleCompilerError(error);
    }
}

async function runFile(uri: vscode.Uri) {
    const filePath = uri.fsPath;
    
    outputChannel.clear();
    outputChannel.show(true);
    outputChannel.appendLine(`Running: ${filePath}`);
    outputChannel.appendLine('─'.repeat(60));
    
    try {
        const result = await runCompiler(['run', filePath]);
        
        outputChannel.appendLine(result.stdout);
        if (result.stderr) {
            outputChannel.appendLine(result.stderr);
        }
        
        if (result.exitCode === 0) {
            outputChannel.appendLine('\n✓ Execution completed');
        } else {
            vscode.window.showErrorMessage('Execution failed');
        }
    } catch (error) {
        handleCompilerError(error);
    }
}

async function showTokens(uri: vscode.Uri) {
    const filePath = uri.fsPath;
    
    try {
        const result = await runCompiler(['lex', filePath]);
        
        const doc = await vscode.workspace.openTextDocument({
            content: result.stdout,
            language: 'json'
        });
        await vscode.window.showTextDocument(doc, vscode.ViewColumn.Beside);
    } catch (error) {
        handleCompilerError(error);
    }
}

async function showAST(uri: vscode.Uri) {
    const filePath = uri.fsPath;
    
    try {
        const result = await runCompiler(['parse', filePath]);
        
        const doc = await vscode.workspace.openTextDocument({
            content: result.stdout,
            language: 'json'
        });
        await vscode.window.showTextDocument(doc, vscode.ViewColumn.Beside);
    } catch (error) {
        handleCompilerError(error);
    }
}

async function createNewProject(name: string) {
    const folders = vscode.workspace.workspaceFolders;
    const targetDir = folders 
        ? folders[0].uri.fsPath 
        : require('os').homedir();
    
    outputChannel.clear();
    outputChannel.show(true);
    outputChannel.appendLine(`Creating new project: ${name}`);
    
    try {
        const result = await runCompiler(['new', name], { cwd: targetDir });
        
        if (result.exitCode === 0) {
            const projectPath = path.join(targetDir, name);
            
            // Open the new project
            const uri = vscode.Uri.file(projectPath);
            await vscode.commands.executeCommand('vscode.openFolder', uri);
            
            vscode.window.showInformationMessage(`Created ULissy project: ${name}`);
        } else {
            vscode.window.showErrorMessage('Failed to create project');
        }
        
        outputChannel.appendLine(result.stdout);
    } catch (error) {
        handleCompilerError(error);
    }
}

// ============================================================================
// COMPILER INTERFACE
// ============================================================================

interface CompilerResult {
    stdout: string;
    stderr: string;
    exitCode: number;
}

function getCompilerPath(): string {
    const config = vscode.workspace.getConfiguration('ulissy');
    return config.get<string>('compilerPath', 'ulissy');
}

async function runCompiler(
    args: string[], 
    options: { cwd?: string } = {}
): Promise<CompilerResult> {
    const compilerPath = getCompilerPath();
    
    return new Promise((resolve, reject) => {
        const proc = cp.spawn(compilerPath, args, {
            cwd: options.cwd || vscode.workspace.workspaceFolders?.[0]?.uri.fsPath,
            shell: true
        });
        
        let stdout = '';
        let stderr = '';
        
        proc.stdout.on('data', (data) => {
            stdout += data.toString();
        });
        
        proc.stderr.on('data', (data) => {
            stderr += data.toString();
        });
        
        proc.on('error', (error) => {
            reject(error);
        });
        
        proc.on('close', (code) => {
            resolve({
                stdout,
                stderr,
                exitCode: code || 0
            });
        });
    });
}

function handleCompilerError(error: any) {
    const message = error.message || String(error);
    
    if (message.includes('ENOENT')) {
        vscode.window.showErrorMessage(
            'ULissy compiler not found. Please install it or set the path in settings.',
            'Open Settings'
        ).then(selection => {
            if (selection === 'Open Settings') {
                vscode.commands.executeCommand(
                    'workbench.action.openSettings',
                    'ulissy.compilerPath'
                );
            }
        });
    } else {
        vscode.window.showErrorMessage(`Compiler error: ${message}`);
    }
    
    outputChannel.appendLine(`Error: ${message}`);
}

// ============================================================================
// DIAGNOSTICS
// ============================================================================

function parseDiagnostics(uri: vscode.Uri, output: string) {
    const diagnostics: vscode.Diagnostic[] = [];
    
    // Parse error format: error[E0123]: message
    //                       --> file:line:column
    const errorRegex = /error\[E(\d+)\]: (.+)\n\s*--> (.+):(\d+):(\d+)/g;
    let match;
    
    while ((match = errorRegex.exec(output)) !== null) {
        const [, code, message, file, lineStr, colStr] = match;
        const line = parseInt(lineStr, 10) - 1;
        const col = parseInt(colStr, 10) - 1;
        
        const range = new vscode.Range(
            new vscode.Position(line, col),
            new vscode.Position(line, col + 10) // Approximate error span
        );
        
        const diagnostic = new vscode.Diagnostic(
            range,
            message,
            vscode.DiagnosticSeverity.Error
        );
        diagnostic.code = `E${code}`;
        diagnostic.source = 'ulissy';
        
        diagnostics.push(diagnostic);
    }
    
    // Parse warning format
    const warnRegex = /warning\[W(\d+)\]: (.+)\n\s*--> (.+):(\d+):(\d+)/g;
    
    while ((match = warnRegex.exec(output)) !== null) {
        const [, code, message, file, lineStr, colStr] = match;
        const line = parseInt(lineStr, 10) - 1;
        const col = parseInt(colStr, 10) - 1;
        
        const range = new vscode.Range(
            new vscode.Position(line, col),
            new vscode.Position(line, col + 10)
        );
        
        const diagnostic = new vscode.Diagnostic(
            range,
            message,
            vscode.DiagnosticSeverity.Warning
        );
        diagnostic.code = `W${code}`;
        diagnostic.source = 'ulissy';
        
        diagnostics.push(diagnostic);
    }
    
    diagnosticCollection.set(uri, diagnostics);
}

// ============================================================================
// DOCUMENT HANDLERS
// ============================================================================

async function onDocumentSave(document: vscode.TextDocument) {
    if (document.languageId !== 'ulissy') {
        return;
    }
    
    const config = vscode.workspace.getConfiguration('ulissy');
    
    // Run type check on save
    if (config.get<boolean>('checkOnSave', true)) {
        try {
            const result = await runCompiler(['check', document.uri.fsPath]);
            
            diagnosticCollection.delete(document.uri);
            
            if (result.exitCode !== 0) {
                parseDiagnostics(document.uri, result.stderr || result.stdout);
            }
        } catch (error) {
            // Silently fail - don't interrupt save
            console.error('Type check failed:', error);
        }
    }
}

// ============================================================================
// LANGUAGE SERVER
// ============================================================================

function startLanguageServer(context: vscode.ExtensionContext) {
    const compilerPath = getCompilerPath();
    
    // Server options - run ulissy with --lsp flag
    const serverOptions: ServerOptions = {
        command: compilerPath,
        args: ['lsp'],
        transport: TransportKind.stdio
    };
    
    // Client options
    const clientOptions: LanguageClientOptions = {
        documentSelector: [{ scheme: 'file', language: 'ulissy' }],
        synchronize: {
            fileEvents: vscode.workspace.createFileSystemWatcher('**/*.ul')
        },
        outputChannel: outputChannel
    };
    
    // Create and start client
    client = new LanguageClient(
        'ulissyLanguageServer',
        'ULissy Language Server',
        serverOptions,
        clientOptions
    );
    
    client.start().then(() => {
        outputChannel.appendLine('Language server started');
    }).catch((error) => {
        outputChannel.appendLine(`Language server failed to start: ${error}`);
        // Continue without LSP - basic features still work
    });
}

function restartLanguageServer(context: vscode.ExtensionContext) {
    if (client) {
        client.stop().then(() => {
            startLanguageServer(context);
            vscode.window.showInformationMessage('ULissy language server restarted');
        });
    } else {
        startLanguageServer(context);
    }
}

// ============================================================================
// WELCOME MESSAGE
// ============================================================================

async function showWelcomeMessage(context: vscode.ExtensionContext) {
    const WELCOME_SHOWN_KEY = 'ulissy.welcomeShown';
    const shown = context.globalState.get<boolean>(WELCOME_SHOWN_KEY);
    
    if (!shown) {
        const selection = await vscode.window.showInformationMessage(
            'Welcome to ULissy! A Programming Language for Moving Machines.',
            'Create Project',
            'View Documentation',
            'Dismiss'
        );
        
        if (selection === 'Create Project') {
            vscode.commands.executeCommand('ulissy.newProject');
        } else if (selection === 'View Documentation') {
            vscode.env.openExternal(
                vscode.Uri.parse('https://github.com/GNS-Foundation/ULissy_Program')
            );
        }
        
        context.globalState.update(WELCOME_SHOWN_KEY, true);
    }
}
