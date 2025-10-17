// VSCode Extension for RavensOne Language Support
import * as vscode from 'vscode';
import * as path from 'path';
import { spawn } from 'child_process';

let outputChannel: vscode.OutputChannel;

export function activate(context: vscode.ExtensionContext) {
    console.log('RavensOne extension activated');

    outputChannel = vscode.window.createOutputChannel('RavensOne');
    outputChannel.appendLine('RavensOne extension activated!');

    // Register commands
    registerCompileCommand(context);
    registerCheckCommand(context);
    registerFormatCommand(context);
    registerNewComponentCommand(context);

    // Start language server if available
    // startLanguageServer(context);

    outputChannel.appendLine('All commands registered');
}

function registerCompileCommand(context: vscode.ExtensionContext) {
    const compileCmd = vscode.commands.registerCommand('raven.compile', async () => {
        const editor = vscode.window.activeTextEditor;
        if (!editor) {
            vscode.window.showErrorMessage('No active editor');
            return;
        }

        const document = editor.document;
        if (document.languageId !== 'raven') {
            vscode.window.showErrorMessage('Current file is not a RavensOne file');
            return;
        }

        const filePath = document.uri.fsPath;
        const outputPath = filePath.replace(/\.raven$/, '.wasm');

        outputChannel.show();
        outputChannel.appendLine(`\n🔥 Compiling ${path.basename(filePath)}...`);

        const ravenPath = getRavenCompilerPath();
        const process = spawn(ravenPath, ['compile', filePath, '--output', outputPath]);

        process.stdout.on('data', (data) => {
            outputChannel.appendLine(data.toString());
        });

        process.stderr.on('data', (data) => {
            outputChannel.appendLine(`Error: ${data.toString()}`);
        });

        process.on('close', (code) => {
            if (code === 0) {
                outputChannel.appendLine(`✅ Compilation successful: ${path.basename(outputPath)}`);
                vscode.window.showInformationMessage(`Compiled to ${path.basename(outputPath)}`);
            } else {
                outputChannel.appendLine(`❌ Compilation failed with code ${code}`);
                vscode.window.showErrorMessage(`Compilation failed`);
            }
        });
    });

    context.subscriptions.push(compileCmd);
}

function registerCheckCommand(context: vscode.ExtensionContext) {
    const checkCmd = vscode.commands.registerCommand('raven.check', async () => {
        const editor = vscode.window.activeTextEditor;
        if (!editor) {
            vscode.window.showErrorMessage('No active editor');
            return;
        }

        const document = editor.document;
        if (document.languageId !== 'raven') {
            vscode.window.showErrorMessage('Current file is not a RavensOne file');
            return;
        }

        outputChannel.show();
        outputChannel.appendLine(`\n🔍 Type checking ${path.basename(document.uri.fsPath)}...`);

        const ravenPath = getRavenCompilerPath();
        const process = spawn(ravenPath, ['check', document.uri.fsPath]);

        process.stdout.on('data', (data) => {
            outputChannel.appendLine(data.toString());
        });

        process.stderr.on('data', (data) => {
            outputChannel.appendLine(data.toString());
        });

        process.on('close', (code) => {
            if (code === 0) {
                outputChannel.appendLine('✅ No type errors');
                vscode.window.showInformationMessage('Type check passed');
            } else {
                outputChannel.appendLine('❌ Type errors found');
                vscode.window.showWarningMessage('Type check failed');
            }
        });
    });

    context.subscriptions.push(checkCmd);
}

function registerFormatCommand(context: vscode.ExtensionContext) {
    const formatCmd = vscode.commands.registerCommand('raven.format', async () => {
        const editor = vscode.window.activeTextEditor;
        if (!editor) {
            vscode.window.showErrorMessage('No active editor');
            return;
        }

        const document = editor.document;
        if (document.languageId !== 'raven') {
            vscode.window.showErrorMessage('Current file is not a RavensOne file');
            return;
        }

        outputChannel.appendLine(`\n✨ Formatting ${path.basename(document.uri.fsPath)}...`);

        const ravenPath = getRavenCompilerPath();
        const process = spawn(ravenPath, ['fmt', document.uri.fsPath]);

        process.on('close', (code) => {
            if (code === 0) {
                outputChannel.appendLine('✅ File formatted');
                vscode.window.showInformationMessage('File formatted successfully');
                // Reload the document
                vscode.workspace.openTextDocument(document.uri).then(doc => {
                    vscode.window.showTextDocument(doc);
                });
            } else {
                outputChannel.appendLine('❌ Format failed');
                vscode.window.showErrorMessage('Format failed');
            }
        });
    });

    context.subscriptions.push(formatCmd);
}

function registerNewComponentCommand(context: vscode.ExtensionContext) {
    const newComponentCmd = vscode.commands.registerCommand('raven.newComponent', async () => {
        const componentName = await vscode.window.showInputBox({
            prompt: 'Enter component name',
            placeHolder: 'MyComponent',
            validateInput: (value) => {
                if (!value || value.trim().length === 0) {
                    return 'Component name cannot be empty';
                }
                if (!/^[A-Z][a-zA-Z0-9]*$/.test(value)) {
                    return 'Component name must start with uppercase letter';
                }
                return null;
            }
        });

        if (!componentName) {
            return;
        }

        const workspaceFolder = vscode.workspace.workspaceFolders?.[0];
        if (!workspaceFolder) {
            vscode.window.showErrorMessage('No workspace folder open');
            return;
        }

        const componentsDir = path.join(workspaceFolder.uri.fsPath, 'src', 'components');
        const componentFile = path.join(componentsDir, `${componentName}.raven`);

        const template = `// ${componentName} Component
component ${componentName}() {
    let count = Signal::new(0);

    let increment = || {
        count.set(count.get() + 1);
    };

    <div>
        <h2>"${componentName}"</h2>
        <p>"Count: " {count.get()}</p>
        <button onclick={increment}>"Click me"</button>
    </div>
}
`;

        const fs = require('fs');
        const fsPromises = fs.promises;

        try {
            await fsPromises.mkdir(componentsDir, { recursive: true });
            await fsPromises.writeFile(componentFile, template);

            const doc = await vscode.workspace.openTextDocument(componentFile);
            await vscode.window.showTextDocument(doc);

            vscode.window.showInformationMessage(`Created component: ${componentName}`);
            outputChannel.appendLine(`✅ Created ${componentFile}`);
        } catch (error) {
            vscode.window.showErrorMessage(`Failed to create component: ${error}`);
            outputChannel.appendLine(`❌ Error: ${error}`);
        }
    });

    context.subscriptions.push(newComponentCmd);
}

function getRavenCompilerPath(): string {
    const config = vscode.workspace.getConfiguration('raven');
    const customPath = config.get<string>('compilerPath');

    if (customPath) {
        return customPath;
    }

    // Default to 'raven' in PATH
    return 'raven';
}

export function deactivate() {
    if (outputChannel) {
        outputChannel.dispose();
    }
}
