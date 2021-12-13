;;; ng-lsp-mode.el --- async lsp-mode           -*- lexical-binding: t; -*-

(defvar ng-lsp-mode-pipe nil)
(defvar lsp-global-workspace nil)

(defun ng-lsp-make-message-advice (orig-fun &rest args)
  (nth 0 args))
(advice-add 'lsp--make-message :around #'ng-lsp-make-message-advice)
  
(defun ng-lsp-start-workspace-advice (orig-fun &rest args)
  (setq lsp-global-workspace (apply orig-fun args)))
(advice-add 'lsp--start-workspace :around #'ng-lsp-start-workspace-advice)

(defun ng-lsp-emacsng-send-no-wait (message proc)
  (let ((id (plist-get message :id))
        (msg (plist-get message :method))
        (params (plist-get message :params)))
    (if id
        (when (stringp (plist-get message :method))
          (lsp-async-send-request ng-lsp-mode-pipe msg params (number-to-string id)))
      (lsp-async-send-notification ng-lsp-mode-pipe msg params))))

(defun ng-lsp-send-no-wait-advice (orig-fun &rest args)
  (unless lsp--flushing-delayed-changes
    (let ((lsp--flushing-delayed-changes t))
      (lsp--flush-delayed-changes)))
  (condition-case err
      (apply 'ng-lsp-emacsng-send-no-wait args)
    ('error (lsp--error "Sending to process failed with the following error: %s"
                     (error-message-string err)))))
(advice-add 'lsp--send-no-wait :around #'ng-lsp-send-no-wait-advice)

(defun ng-lsp-handler (proc output)
  (let ((result (lsp-handler proc output)))
    (lsp--parser-on-message result lsp-global-workspace)))

;; Local Variables:
;; byte-compile-warnings: (not unresolved free-vars lexical)
;; End:
(provide 'ng-lsp-mode)
;;; ng-lsp-mode.el ends here
