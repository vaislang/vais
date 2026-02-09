// Main application entry point
import * as monaco from 'monaco-editor';
import { registerVaisLanguage } from './vais-language.js';
import { examples, getExampleList, getExampleCode } from './examples.js';
import { VaisCompiler } from './compiler.js';

// Monaco workers are handled by vite-plugin-monaco-editor
// No need for manual MonacoEnvironment configuration

class Playground {
  constructor() {
    this.editor = null;
    this.compiler = new VaisCompiler();
    this.currentExample = null;
    this.isRunning = false;
    this.currentTarget = 'native';

    this.init();
  }

  async init() {
    // Register Vais language
    registerVaisLanguage(monaco);

    // Create editor
    this.createEditor();

    // Setup UI
    this.setupExamplesList();
    this.setupEventListeners();

    // Load default example
    this.loadExample('hello-world');

    // Initialize compiler
    try {
      await this.compiler.initialize();
      const modeLabel = this.compiler.getModeLabel();
      this.updateStatus('ready', `Ready (${modeLabel})`);
    } catch (error) {
      this.updateStatus('error', 'Compiler initialization failed');
      this.appendOutput(`Error: ${error.message}`, 'error');
    }
  }

  createEditor() {
    const editorContainer = document.getElementById('editor');

    this.editor = monaco.editor.create(editorContainer, {
      value: '',
      language: 'vais',
      theme: 'vais-dark',
      fontSize: 14,
      fontFamily: 'Monaco, Menlo, "Ubuntu Mono", Consolas, monospace',
      minimap: {
        enabled: true
      },
      scrollBeyondLastLine: false,
      automaticLayout: true,
      tabSize: 4,
      insertSpaces: true,
      lineNumbers: 'on',
      renderWhitespace: 'selection',
      cursorBlinking: 'smooth',
      folding: true,
      bracketPairColorization: {
        enabled: true
      },
      suggest: {
        snippetsPreventQuickSuggestions: false
      }
    });

    // Add keyboard shortcuts
    this.editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.Enter, () => {
      this.runCode();
    });

    this.editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyS, (e) => {
      e?.preventDefault();
      this.formatCode();
    });
  }

  setupExamplesList() {
    const examplesList = document.getElementById('examples-list');
    const exampleSelect = document.getElementById('example-select');

    const examplesData = getExampleList();

    // Populate sidebar
    examplesData.forEach(({ key, name, description }) => {
      const button = document.createElement('button');
      button.className = 'example-item';
      button.textContent = name;
      button.title = description;
      button.dataset.example = key;
      button.addEventListener('click', () => this.loadExample(key));
      examplesList.appendChild(button);
    });

    // Populate dropdown
    examplesData.forEach(({ key, name }) => {
      const option = document.createElement('option');
      option.value = key;
      option.textContent = name;
      exampleSelect.appendChild(option);
    });

    // Dropdown change handler
    exampleSelect.addEventListener('change', (e) => {
      const key = e.target.value;
      if (key) {
        this.loadExample(key);
      }
    });
  }

  setupEventListeners() {
    // Run button
    document.getElementById('run-btn').addEventListener('click', () => {
      this.runCode();
    });

    // Format button
    document.getElementById('format-btn').addEventListener('click', () => {
      this.formatCode();
    });

    // Clear button
    document.getElementById('clear-btn').addEventListener('click', () => {
      this.clearOutput();
    });

    // Target selector
    document.getElementById('target-select').addEventListener('change', (e) => {
      this.currentTarget = e.target.value;
      this.updateRunButtonLabel();
    });

    // Keyboard shortcuts
    document.addEventListener('keydown', (e) => {
      if ((e.ctrlKey || e.metaKey) && e.key === 's') {
        e.preventDefault();
        this.formatCode();
      }
    });
  }

  loadExample(key) {
    const code = getExampleCode(key);
    if (code) {
      this.editor.setValue(code);
      this.currentExample = key;

      // Update UI
      document.querySelectorAll('.example-item').forEach(item => {
        item.classList.toggle('active', item.dataset.example === key);
      });

      document.getElementById('example-select').value = key;

      // Clear output
      this.clearOutput();
    }
  }

  async runCode() {
    if (this.isRunning) {
      return;
    }

    this.isRunning = true;
    const actionLabel = this.currentTarget === 'js' ? 'Compiling...' : 'Running...';
    this.updateStatus('running', actionLabel);
    this.setRunLoading(true);
    this.showOutputLoading();

    try {
      const code = this.editor.getValue();

      // Compile and run with selected target
      const result = await this.compiler.compileAndRun(code, this.currentTarget);

      this.clearOutput();

      if (result.success) {
        const successMsg = this.currentTarget === 'js'
          ? 'Compilation to JavaScript successful!'
          : 'Compilation successful!';
        this.appendOutput(successMsg, 'success');

        // Show warnings if any
        if (result.warnings && result.warnings.length > 0) {
          result.warnings.forEach(warning => {
            this.appendOutput(this.compiler.formatWarning(warning), 'warning');
          });
        }

        // Show output
        if (result.output) {
          this.appendOutput('', 'line');
          if (this.currentTarget === 'js') {
            this.appendOutput('Generated Code:', 'info');
          } else {
            this.appendOutput('Output:', 'info');
          }
          this.appendOutput(result.output, 'line');
        }

        const statusMsg = this.currentTarget === 'js'
          ? 'Compilation completed'
          : 'Execution completed';
        this.updateStatus('success', statusMsg);
      } else {
        this.appendOutput('Compilation failed!', 'error');

        // Show errors
        if (result.errors && result.errors.length > 0) {
          result.errors.forEach(error => {
            this.appendOutput(this.compiler.formatError(error), 'error');
          });
        }

        // Show warnings
        if (result.warnings && result.warnings.length > 0) {
          result.warnings.forEach(warning => {
            this.appendOutput(this.compiler.formatWarning(warning), 'warning');
          });
        }

        this.updateStatus('error', 'Compilation failed');
      }
    } catch (error) {
      this.clearOutput();
      this.appendOutput(`Runtime error: ${error.message}`, 'error');
      this.updateStatus('error', 'Error');
    } finally {
      this.isRunning = false;
      this.setRunLoading(false);
    }
  }

  setRunLoading(loading) {
    const runBtn = document.getElementById('run-btn');
    if (loading) {
      runBtn.classList.add('loading');
      const label = this.currentTarget === 'js' ? 'Compiling...' : 'Compiling...';
      runBtn.innerHTML = `<span class="btn-spinner"></span> ${label}`;
    } else {
      runBtn.classList.remove('loading');
      this.updateRunButtonLabel();
    }
  }

  updateRunButtonLabel() {
    const runBtn = document.getElementById('run-btn');
    const label = this.currentTarget === 'js' ? 'Compile' : 'Run';
    runBtn.innerHTML = `<svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor"><path d="M11.596 8.697l-6.363 3.692c-.54.313-1.233-.066-1.233-.697V4.308c0-.63.692-1.01 1.233-.696l6.363 3.692a.802.802 0 0 1 0 1.393z"/></svg> ${label}`;
  }

  showOutputLoading() {
    const output = document.getElementById('output');
    const loadingText = this.currentTarget === 'js'
      ? 'Compiling to JavaScript...'
      : 'Compiling and running...';
    output.innerHTML = `<div class="output-loading"><div class="output-loading-spinner"></div><span class="output-loading-text">${loadingText}</span></div>`;
  }

  formatCode() {
    // Simple formatting (in production, this would use the actual formatter)
    const code = this.editor.getValue();
    const formatted = this.simpleFormat(code);
    this.editor.setValue(formatted);
    this.appendOutput('Code formatted', 'success');
  }

  simpleFormat(code) {
    // Very basic formatting
    // In production, this would call the vaisc formatter
    let lines = code.split('\n');
    let indent = 0;
    let formatted = [];

    lines.forEach(line => {
      const trimmed = line.trim();

      // Decrease indent for closing braces
      if (trimmed.startsWith('}')) {
        indent = Math.max(0, indent - 1);
      }

      // Add indented line
      if (trimmed) {
        formatted.push('    '.repeat(indent) + trimmed);
      } else {
        formatted.push('');
      }

      // Increase indent for opening braces
      if (trimmed.endsWith('{')) {
        indent++;
      }
    });

    return formatted.join('\n');
  }

  clearOutput() {
    const output = document.getElementById('output');
    output.innerHTML = '<div class="output-placeholder">Output will appear here...</div>';
    this.updateStatus('ready', 'Ready');
  }

  appendOutput(text, type = 'line') {
    const output = document.getElementById('output');

    // Remove placeholder if present
    const placeholder = output.querySelector('.output-placeholder');
    if (placeholder) {
      placeholder.remove();
    }

    const line = document.createElement('div');
    line.className = `output-${type}`;
    line.textContent = text;
    output.appendChild(line);

    // Auto-scroll to bottom
    output.scrollTop = output.scrollHeight;
  }

  updateStatus(type, text) {
    const statusDot = document.querySelector('.status-dot');
    const statusText = document.querySelector('.status-text');

    statusDot.className = `status-dot ${type}`;
    statusText.textContent = text;
  }
}

// Initialize playground when DOM is ready
document.addEventListener('DOMContentLoaded', () => {
  window.playground = new Playground();
});
