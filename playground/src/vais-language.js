// Monaco Editor language definition for Vais
export function registerVaisLanguage(monaco) {
  // Register language
  monaco.languages.register({ id: 'vais' });

  // Define tokens
  monaco.languages.setMonarchTokensProvider('vais', {
    defaultToken: '',
    tokenPostfix: '.vais',

    keywords: [
      'F', 'S', 'E', 'I', 'L', 'M', 'T', 'U', 'R', 'C', 'O', 'A', 'Y',
      'B', 'W', 'X', 'P', 'D', 'N', 'G',
      'break', 'continue', 'return', 'true', 'false',
      'async', 'await', 'pub', 'mut', 'const', 'static',
      'impl', 'trait', 'where', 'self', 'Self', 'super',
      'unsafe', 'extern', 'type', 'let', 'in'
    ],

    typeKeywords: [
      'i8', 'i16', 'i32', 'i64', 'i128',
      'u8', 'u16', 'u32', 'u64', 'u128',
      'f32', 'f64', 'bool', 'char', 'str',
      'isize', 'usize', 'void'
    ],

    operators: [
      '=', '>', '<', '!', '~', '?', ':',
      '==', '<=', '>=', '!=', '&&', '||', '++', '--',
      '+', '-', '*', '/', '&', '|', '^', '%',
      '<<', '>>', '>>>', '+=', '-=', '*=', '/=', '&=',
      '|=', '^=', '%=', '<<=', '>>=', '>>>=', '=>',
      '@', ':=', '|>', '..'
    ],

    symbols: /[=><!~?:&|+\-*\/\^%@]+/,
    escapes: /\\(?:[abfnrtv\\"']|x[0-9A-Fa-f]{1,4}|u[0-9A-Fa-f]{4}|U[0-9A-Fa-f]{8})/,

    tokenizer: {
      root: [
        // Identifiers and keywords
        [/[a-z_$][\w$]*/, {
          cases: {
            '@typeKeywords': 'type',
            '@keywords': 'keyword',
            '@default': 'identifier'
          }
        }],
        [/[A-Z][\w\$]*/, 'type.identifier'],

        // Whitespace
        { include: '@whitespace' },

        // Delimiters and operators
        [/[{}()\[\]]/, '@brackets'],
        [/[<>](?!@symbols)/, '@brackets'],
        [/@symbols/, {
          cases: {
            '@operators': 'operator',
            '@default': ''
          }
        }],

        // Numbers
        [/\d*\.\d+([eE][\-+]?\d+)?[fFdD]?/, 'number.float'],
        [/0[xX][0-9a-fA-F]+/, 'number.hex'],
        [/0[bB][01]+/, 'number.binary'],
        [/\d+/, 'number'],

        // Delimiter: after number because of .\d floats
        [/[;,.]/, 'delimiter'],

        // Strings
        [/"([^"\\]|\\.)*$/, 'string.invalid'],
        [/"/, 'string', '@string'],

        // Characters
        [/'[^\\']'/, 'string'],
        [/(')(@escapes)(')/, ['string', 'string.escape', 'string']],
        [/'/, 'string.invalid']
      ],

      whitespace: [
        [/[ \t\r\n]+/, ''],
        [/#.*$/, 'comment'],
        [/\/\*/, 'comment', '@comment'],
        [/\/\/.*$/, 'comment']
      ],

      comment: [
        [/[^\/*]+/, 'comment'],
        [/\*\//, 'comment', '@pop'],
        [/[\/*]/, 'comment']
      ],

      string: [
        [/[^\\"]+/, 'string'],
        [/@escapes/, 'string.escape'],
        [/\\./, 'string.escape.invalid'],
        [/"/, 'string', '@pop']
      ]
    }
  });

  // Define theme
  monaco.editor.defineTheme('vais-dark', {
    base: 'vs-dark',
    inherit: true,
    rules: [
      { token: 'keyword', foreground: 'C586C0', fontStyle: 'bold' },
      { token: 'type', foreground: '4EC9B0' },
      { token: 'type.identifier', foreground: '4EC9B0' },
      { token: 'identifier', foreground: '9CDCFE' },
      { token: 'number', foreground: 'B5CEA8' },
      { token: 'number.float', foreground: 'B5CEA8' },
      { token: 'number.hex', foreground: 'B5CEA8' },
      { token: 'number.binary', foreground: 'B5CEA8' },
      { token: 'string', foreground: 'CE9178' },
      { token: 'string.escape', foreground: 'D7BA7D' },
      { token: 'comment', foreground: '6A9955', fontStyle: 'italic' },
      { token: 'operator', foreground: 'D4D4D4' },
      { token: 'delimiter', foreground: 'D4D4D4' }
    ],
    colors: {
      'editor.background': '#1e1e1e',
      'editor.foreground': '#d4d4d4',
      'editorLineNumber.foreground': '#858585',
      'editorLineNumber.activeForeground': '#c6c6c6',
      'editorCursor.foreground': '#ffffff',
      'editor.selectionBackground': '#264f78',
      'editor.inactiveSelectionBackground': '#3a3d41'
    }
  });

  // Configure language features
  monaco.languages.setLanguageConfiguration('vais', {
    comments: {
      lineComment: '#',
      blockComment: ['/*', '*/']
    },
    brackets: [
      ['{', '}'],
      ['[', ']'],
      ['(', ')']
    ],
    autoClosingPairs: [
      { open: '{', close: '}' },
      { open: '[', close: ']' },
      { open: '(', close: ')' },
      { open: '"', close: '"', notIn: ['string'] },
      { open: "'", close: "'", notIn: ['string', 'comment'] }
    ],
    surroundingPairs: [
      { open: '{', close: '}' },
      { open: '[', close: ']' },
      { open: '(', close: ')' },
      { open: '"', close: '"' },
      { open: "'", close: "'" }
    ],
    folding: {
      markers: {
        start: new RegExp('^\\s*#region\\b'),
        end: new RegExp('^\\s*#endregion\\b')
      }
    }
  });

  // Add completion provider
  monaco.languages.registerCompletionItemProvider('vais', {
    provideCompletionItems: (model, position) => {
      const suggestions = [
        // Keywords
        { label: 'F', kind: monaco.languages.CompletionItemKind.Keyword, insertText: 'F ${1:name}(${2:params}) -> ${3:type} {\n    ${4}\n}', insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet },
        { label: 'S', kind: monaco.languages.CompletionItemKind.Keyword, insertText: 'S ${1:Name} {\n    ${2:field}: ${3:type}\n}', insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet },
        { label: 'E', kind: monaco.languages.CompletionItemKind.Keyword, insertText: 'E ${1:Name} {\n    ${2:Variant}\n}', insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet },
        { label: 'I', kind: monaco.languages.CompletionItemKind.Keyword, insertText: 'I ${1:condition} {\n    ${2}\n}', insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet },
        { label: 'L', kind: monaco.languages.CompletionItemKind.Keyword, insertText: 'L ${1:i}:${2:0..10} {\n    ${3}\n}', insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet },
        { label: 'M', kind: monaco.languages.CompletionItemKind.Keyword, insertText: 'M ${1:expr} {\n    ${2:pattern} => ${3:result}\n}', insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet },
        { label: 'B', kind: monaco.languages.CompletionItemKind.Keyword, insertText: 'B', documentation: 'break' },
        { label: 'W', kind: monaco.languages.CompletionItemKind.Keyword, insertText: 'W ${1:TraitName} {\n    ${2}\n}', insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet, documentation: 'trait definition' },
        { label: 'X', kind: monaco.languages.CompletionItemKind.Keyword, insertText: 'X ${1:StructName} {\n    ${2}\n}', insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet, documentation: 'impl block' },
        { label: 'D', kind: monaco.languages.CompletionItemKind.Keyword, insertText: 'D ${1:expression}', insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet, documentation: 'defer statement' },
        { label: 'U', kind: monaco.languages.CompletionItemKind.Keyword, insertText: 'U ${1:module}', insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet, documentation: 'import module' },
        { label: 'P', kind: monaco.languages.CompletionItemKind.Keyword, insertText: 'P ${1}', insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet, documentation: 'pub (public visibility)' },
        { label: 'R', kind: monaco.languages.CompletionItemKind.Keyword, insertText: 'R ${1:value}', insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet, documentation: 'return value' },
        { label: 'N', kind: monaco.languages.CompletionItemKind.Keyword, insertText: 'N F ${1:name}(${2:params}) -> ${3:type}', insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet, documentation: 'extern function' },
        { label: 'G', kind: monaco.languages.CompletionItemKind.Keyword, insertText: 'G ${1:name}: ${2:type} = ${3:value}', insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet, documentation: 'global variable' },
        { label: 'break', kind: monaco.languages.CompletionItemKind.Keyword, insertText: 'break' },
        { label: 'continue', kind: monaco.languages.CompletionItemKind.Keyword, insertText: 'continue' },
        { label: 'return', kind: monaco.languages.CompletionItemKind.Keyword, insertText: 'return ${1:value}', insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet },

        // Types
        { label: 'i64', kind: monaco.languages.CompletionItemKind.TypeParameter, insertText: 'i64' },
        { label: 'i32', kind: monaco.languages.CompletionItemKind.TypeParameter, insertText: 'i32' },
        { label: 'f64', kind: monaco.languages.CompletionItemKind.TypeParameter, insertText: 'f64' },
        { label: 'bool', kind: monaco.languages.CompletionItemKind.TypeParameter, insertText: 'bool' },
        { label: 'str', kind: monaco.languages.CompletionItemKind.TypeParameter, insertText: 'str' },

        // Built-in functions
        { label: 'puts', kind: monaco.languages.CompletionItemKind.Function, insertText: 'puts("${1:text}")', insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet },
        { label: 'print', kind: monaco.languages.CompletionItemKind.Function, insertText: 'print("${1:text}")', insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet },
        { label: 'putchar', kind: monaco.languages.CompletionItemKind.Function, insertText: 'putchar(${1:char})', insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet },
        { label: 'printf', kind: monaco.languages.CompletionItemKind.Function, insertText: 'printf("${1:format}", ${2:args})', insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet },

        // Snippets
        { label: 'main', kind: monaco.languages.CompletionItemKind.Snippet, insertText: 'F main() {\n    ${1}\n}', insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet, documentation: 'Main function template' },
        { label: 'fn', kind: monaco.languages.CompletionItemKind.Snippet, insertText: 'F ${1:name}(${2:params}) -> ${3:i64} {\n    ${4}\n}', insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet, documentation: 'Function template' }
      ];

      return { suggestions };
    }
  });
}
