;;; vais-lsp.el --- LSP integration for Vais -*- lexical-binding: t; -*-

;; Copyright (C) 2026

;; Author: Vais Contributors
;; Version: 0.1.0
;; Package-Requires: ((emacs "26.1") (vais-mode "0.1.0"))
;; Keywords: languages vais lsp
;; URL: https://github.com/your-repo/vais

;;; Commentary:

;; This package provides LSP (Language Server Protocol) integration
;; for Vais programming language.
;;
;; Supports both lsp-mode and eglot.

;;; Code:

(require 'vais-mode)

;;; Configuration

(defgroup vais-lsp nil
  "LSP integration for Vais programming language."
  :group 'vais
  :prefix "vais-lsp-")

(defcustom vais-lsp-server-command '("vais-lsp")
  "Command to start the Vais LSP server.
This should be a list of strings, where the first element is the
executable name and the rest are arguments."
  :type '(repeat string)
  :group 'vais-lsp)

(defcustom vais-lsp-server-path nil
  "Path to the Vais LSP server executable.
If nil, the server is assumed to be in PATH."
  :type '(choice (const :tag "Use PATH" nil)
                 (file :tag "Custom path"))
  :group 'vais-lsp)

;;; LSP-mode integration

(defun vais-lsp-setup-lsp-mode ()
  "Configure lsp-mode for Vais."
  (when (require 'lsp-mode nil t)
    (let ((server-cmd (if vais-lsp-server-path
                          (cons vais-lsp-server-path (cdr vais-lsp-server-command))
                        vais-lsp-server-command)))
      (lsp-register-client
       (make-lsp-client
        :new-connection (lsp-stdio-connection server-cmd)
        :activation-fn (lsp-activate-on "vais")
        :major-modes '(vais-mode)
        :priority -1
        :server-id 'vais-lsp)))

    ;; Hook for automatic activation
    (add-hook 'vais-mode-hook #'lsp-deferred)))

;;; Eglot integration

(defun vais-lsp-setup-eglot ()
  "Configure eglot for Vais."
  (when (require 'eglot nil t)
    (let ((server-cmd (if vais-lsp-server-path
                          (cons vais-lsp-server-path (cdr vais-lsp-server-command))
                        vais-lsp-server-command)))
      (add-to-list 'eglot-server-programs
                   `(vais-mode . ,server-cmd)))

    ;; Hook for automatic activation
    (add-hook 'vais-mode-hook #'eglot-ensure)))

;;; LSP features configuration

(defun vais-lsp-configure-features ()
  "Configure LSP features for Vais mode."
  (when (require 'lsp-mode nil t)
    ;; Enable common features
    (setq-local lsp-enable-symbol-highlighting t)
    (setq-local lsp-enable-indentation nil)  ;; Use vais-mode's indentation
    (setq-local lsp-enable-on-type-formatting nil)
    (setq-local lsp-enable-snippet-support t)
    (setq-local lsp-enable-completion-at-point t)

    ;; Semantic tokens (if supported by vais-lsp)
    (setq-local lsp-semantic-tokens-enable t)))

;;; Utility functions

(defun vais-lsp-restart ()
  "Restart the Vais LSP server."
  (interactive)
  (cond
   ((and (fboundp 'lsp-workspace-restart)
         (bound-and-true-p lsp-mode))
    (lsp-workspace-restart))
   ((and (fboundp 'eglot-reconnect)
         (bound-and-true-p eglot--managed-mode))
    (call-interactively 'eglot-reconnect))
   (t
    (message "No active LSP session found"))))

(defun vais-lsp-server-version ()
  "Display the version of the Vais LSP server."
  (interactive)
  (let ((cmd (if vais-lsp-server-path
                 vais-lsp-server-path
               (car vais-lsp-server-command))))
    (if (executable-find cmd)
        (message "%s" (shell-command-to-string (format "%s --version" cmd)))
      (message "Vais LSP server not found in PATH"))))

;;;###autoload
(defun vais-lsp-install ()
  "Install and configure LSP support for Vais.
This function attempts to configure either lsp-mode or eglot,
depending on what is available."
  (interactive)
  (cond
   ((featurep 'lsp-mode)
    (vais-lsp-setup-lsp-mode)
    (message "Configured Vais LSP with lsp-mode"))
   ((featurep 'eglot)
    (vais-lsp-setup-eglot)
    (message "Configured Vais LSP with eglot"))
   (t
    (message "Neither lsp-mode nor eglot is available. Please install one of them."))))

;;; Keybindings

(defun vais-lsp-setup-keybindings ()
  "Setup LSP-related keybindings for Vais mode."
  (define-key vais-mode-map (kbd "C-c l r") 'vais-lsp-restart)
  (define-key vais-mode-map (kbd "C-c l v") 'vais-lsp-server-version))

;;; Minor mode for LSP features

;;;###autoload
(define-minor-mode vais-lsp-mode
  "Minor mode for Vais LSP integration."
  :lighter " VaisLSP"
  :keymap (let ((map (make-sparse-keymap)))
            (define-key map (kbd "C-c l r") 'vais-lsp-restart)
            (define-key map (kbd "C-c l v") 'vais-lsp-server-version)
            map)
  (if vais-lsp-mode
      (progn
        (vais-lsp-configure-features)
        (message "Vais LSP mode enabled"))
    (message "Vais LSP mode disabled")))

;;; Auto-setup

;;;###autoload
(with-eval-after-load 'vais-mode
  (when (or (featurep 'lsp-mode) (featurep 'eglot))
    (vais-lsp-install)))

(provide 'vais-lsp)

;;; vais-lsp.el ends here
