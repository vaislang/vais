;;; vais-mode.el --- Major mode for Vais programming language -*- lexical-binding: t; -*-

;; Copyright (C) 2026

;; Author: Vais Contributors
;; Version: 0.1.0
;; Package-Requires: ((emacs "25.1"))
;; Keywords: languages vais
;; URL: https://github.com/your-repo/vais

;;; Commentary:

;; This package provides a major mode for editing Vais source code.
;; Vais is a modern systems programming language with unique single-character
;; keywords and concise syntax.
;;
;; Features:
;; - Syntax highlighting (font-lock)
;; - Automatic indentation
;; - Comment handling (# single-line comments)
;; - Basic editing commands

;;; Code:

(defgroup vais nil
  "Support for Vais programming language."
  :group 'languages
  :prefix "vais-")

(defcustom vais-indent-offset 4
  "Indentation offset for Vais code."
  :type 'integer
  :group 'vais
  :safe #'integerp)

(defcustom vais-mode-hook nil
  "Hook run when entering Vais mode."
  :type 'hook
  :group 'vais)

;;; Syntax table

(defvar vais-mode-syntax-table
  (let ((table (make-syntax-table)))
    ;; Comments: # to end of line
    (modify-syntax-entry ?# "<" table)
    (modify-syntax-entry ?\n ">" table)

    ;; Strings
    (modify-syntax-entry ?\" "\"" table)
    (modify-syntax-entry ?\' "\"" table)

    ;; Operators
    (modify-syntax-entry ?@ "." table)
    (modify-syntax-entry ?: "." table)
    (modify-syntax-entry ?= "." table)
    (modify-syntax-entry ?- "." table)
    (modify-syntax-entry ?> "." table)
    (modify-syntax-entry ?+ "." table)
    (modify-syntax-entry ?* "." table)
    (modify-syntax-entry ?/ "." table)
    (modify-syntax-entry ?% "." table)
    (modify-syntax-entry ?< "." table)
    (modify-syntax-entry ?! "." table)
    (modify-syntax-entry ?& "." table)
    (modify-syntax-entry ?| "." table)
    (modify-syntax-entry ?? "." table)

    ;; Parentheses, brackets, braces
    (modify-syntax-entry ?\( "()" table)
    (modify-syntax-entry ?\) ")(" table)
    (modify-syntax-entry ?\[ "(]" table)
    (modify-syntax-entry ?\] ")[" table)
    (modify-syntax-entry ?\{ "(}" table)
    (modify-syntax-entry ?\} "){" table)

    ;; Underscores are part of symbols
    (modify-syntax-entry ?_ "w" table)

    table)
  "Syntax table for Vais mode.")

;;; Font-lock (Syntax Highlighting)

(defconst vais-keywords
  '("F" "S" "E" "I" "L" "M" "W" "T" "X" "V" "C" "R" "B" "N" "A"
    "if" "else" "loop" "while" "match" "break" "continue" "return"
    "let" "const" "fn" "struct" "enum" "trait" "impl" "async" "await"
    "mut" "ref" "self" "Self" "super" "crate" "pub" "use" "mod"
    "as" "in" "where" "for" "type" "extern" "unsafe" "dyn")
  "Vais language keywords.")

(defconst vais-types
  '("i8" "i16" "i32" "i64" "u8" "u16" "u32" "u64"
    "f32" "f64" "bool" "str" "char" "void"
    "isize" "usize")
  "Vais primitive types.")

(defconst vais-constants
  '("true" "false" "nil" "null" "None" "Some" "Ok" "Err")
  "Vais built-in constants.")

(defconst vais-font-lock-keywords
  (list
   ;; Keywords (single-character and full words)
   `(,(regexp-opt vais-keywords 'symbols) . font-lock-keyword-face)

   ;; Types
   `(,(regexp-opt vais-types 'symbols) . font-lock-type-face)

   ;; Constants
   `(,(regexp-opt vais-constants 'symbols) . font-lock-constant-face)

   ;; Function definitions: F function_name
   '("\\<F\\s-+\\([a-zA-Z_][a-zA-Z0-9_]*\\)" 1 font-lock-function-name-face)

   ;; Struct definitions: S StructName
   '("\\<S\\s-+\\([A-Z][a-zA-Z0-9_]*\\)" 1 font-lock-type-face)

   ;; Enum definitions: E EnumName
   '("\\<E\\s-+\\([A-Z][a-zA-Z0-9_]*\\)" 1 font-lock-type-face)

   ;; Trait definitions: W TraitName or T TraitName
   '("\\<[WT]\\s-+\\([A-Z][a-zA-Z0-9_]*\\)" 1 font-lock-type-face)

   ;; Impl blocks: X TypeName
   '("\\<X\\s-+\\([A-Z][a-zA-Z0-9_]*\\)" 1 font-lock-type-face)

   ;; Type annotations: : Type or -> Type
   '("\\(?::\\|->\\)\\s-*\\([a-zA-Z_][a-zA-Z0-9_]*\\)" 1 font-lock-type-face)

   ;; Function calls
   '("\\([a-zA-Z_][a-zA-Z0-9_]*\\)\\s-*(" 1 font-lock-function-name-face)

   ;; Variable declarations: V varname := or varname :=
   '("\\([a-zA-Z_][a-zA-Z0-9_]*\\)\\s-*:=" 1 font-lock-variable-name-face)

   ;; Self reference
   '("\\<self\\>" . font-lock-variable-name-face)
   '("\\<Self\\>" . font-lock-type-face)

   ;; Special operators
   '("@" . font-lock-builtin-face)  ;; Tail recursion operator
   '(":=" . font-lock-keyword-face)  ;; Assignment
   '("->" . font-lock-keyword-face)  ;; Return type
   '("=>" . font-lock-keyword-face)  ;; Match arms

   ;; Numbers
   '("\\<[0-9]+\\(?:\\.[0-9]+\\)?\\(?:[eE][+-]?[0-9]+\\)?\\>" . font-lock-constant-face)

   ;; Boolean literals
   '("\\<\\(true\\|false\\)\\>" . font-lock-constant-face)

   ;; Attributes/Macros (if any)
   '("@\\[.*?\\]" . font-lock-preprocessor-face))
  "Font-lock keywords for Vais mode.")

;;; Indentation

(defun vais-indent-line ()
  "Indent current line as Vais code."
  (interactive)
  (let ((indent-col 0)
        (current-indent (current-indentation)))
    (save-excursion
      (beginning-of-line)
      (if (bobp)
          (setq indent-col 0)
        ;; Find previous non-blank line
        (let ((prev-indent 0)
              (in-block nil))
          (save-excursion
            (forward-line -1)
            (while (and (not (bobp))
                       (looking-at "^\\s-*$"))
              (forward-line -1))
            (setq prev-indent (current-indentation))
            ;; Check if previous line opens a block
            (when (looking-at ".*[{([:]\\s-*$")
              (setq in-block t))
            ;; Check if previous line is a function/struct/etc definition
            (when (looking-at "^\\s-*[FSEWMTXILV]\\s-+")
              (setq in-block t)))

          ;; Calculate indentation
          (cond
           ;; Closing brace/bracket/paren: decrease indent
           ((looking-at "^\\s-*[}\\])]")
            (setq indent-col (max 0 (- prev-indent vais-indent-offset))))
           ;; After opening block: increase indent
           (in-block
            (setq indent-col (+ prev-indent vais-indent-offset)))
           ;; Default: same as previous line
           (t
            (setq indent-col prev-indent))))))

    ;; Apply indentation
    (if (<= (current-column) (current-indentation))
        (indent-line-to indent-col)
      (save-excursion
        (indent-line-to indent-col)))))

;;; Movement functions

(defun vais-beginning-of-defun (&optional arg)
  "Move to the beginning of a Vais function definition.
With ARG, do it that many times."
  (interactive "^p")
  (or arg (setq arg 1))
  (if (< arg 0)
      (vais-end-of-defun (- arg))
    (dotimes (_ arg)
      (re-search-backward "^\\s-*F\\s-+[a-zA-Z_][a-zA-Z0-9_]*" nil t))))

(defun vais-end-of-defun (&optional arg)
  "Move to the end of a Vais function definition.
With ARG, do it that many times."
  (interactive "^p")
  (or arg (setq arg 1))
  (if (< arg 0)
      (vais-beginning-of-defun (- arg))
    (dotimes (_ arg)
      (re-search-forward "^\\s-*F\\s-+[a-zA-Z_][a-zA-Z0-9_]*" nil t)
      (forward-line 1)
      (while (and (not (eobp))
                  (or (looking-at "^\\s-*$")
                      (not (looking-at "^[^ \t\n]"))))
        (forward-line 1)))))

;;; Keymap

(defvar vais-mode-map
  (let ((map (make-sparse-keymap)))
    ;; Indentation
    (define-key map (kbd "RET") 'newline-and-indent)
    (define-key map (kbd "TAB") 'indent-for-tab-command)

    ;; Navigation
    (define-key map (kbd "C-M-a") 'vais-beginning-of-defun)
    (define-key map (kbd "C-M-e") 'vais-end-of-defun)

    ;; Comment
    (define-key map (kbd "C-c C-c") 'comment-region)
    (define-key map (kbd "C-c C-u") 'uncomment-region)

    map)
  "Keymap for Vais mode.")

;;; Mode definition

;;;###autoload
(define-derived-mode vais-mode prog-mode "Vais"
  "Major mode for editing Vais programming language files.

Vais is a systems programming language with concise syntax using
single-character keywords.

\\{vais-mode-map}"
  :syntax-table vais-mode-syntax-table

  ;; Comments
  (setq-local comment-start "# ")
  (setq-local comment-end "")
  (setq-local comment-start-skip "#+\\s-*")

  ;; Font lock
  (setq-local font-lock-defaults '(vais-font-lock-keywords))

  ;; Indentation
  (setq-local indent-line-function 'vais-indent-line)
  (setq-local electric-indent-chars
              (append '(?\{ ?\} ?\( ?\) ?\[ ?\]) electric-indent-chars))

  ;; Navigation
  (setq-local beginning-of-defun-function 'vais-beginning-of-defun)
  (setq-local end-of-defun-function 'vais-end-of-defun)

  ;; Imenu support
  (setq-local imenu-generic-expression
              '(("Functions" "^\\s-*F\\s-+\\([a-zA-Z_][a-zA-Z0-9_]*\\)" 1)
                ("Structs" "^\\s-*S\\s-+\\([A-Z][a-zA-Z0-9_]*\\)" 1)
                ("Enums" "^\\s-*E\\s-+\\([A-Z][a-zA-Z0-9_]*\\)" 1)
                ("Traits" "^\\s-*[WT]\\s-+\\([A-Z][a-zA-Z0-9_]*\\)" 1)))

  ;; File extension
  (add-to-list 'auto-mode-alist '("\\.vais\\'" . vais-mode)))

;;;###autoload
(add-to-list 'auto-mode-alist '("\\.vais\\'" . vais-mode))

(provide 'vais-mode)

;;; vais-mode.el ends here
