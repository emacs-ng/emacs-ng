;;; ng-magit.el --- ng facilities for magit -*- lexical-binding: t -*-

(defun ng-magit-blame--run (orig-fun &rest args)
  (let ((dir (expand-file-name (project-root (project-current)))))
    (with-current-buffer (current-buffer)
      (unless magit-blame-mode
        (add-hook 'window-scroll-functions #'ng-magit-blame-scroll-refresh-hook t t)
        (magit-blame-mode 1))
      (message "Blaming...")
      (let* ((revision (or magit-buffer-refname magit-buffer-revision ""))
             (file (magit-file-relative-name nil (not magit-buffer-file-name)))
             (args (if (memq magit-blame-type '(final removal))
                       (cons "--reverse" args)
                     args))
             (beg (line-number-at-pos (window-start)))
             (end (line-number-at-pos (1- (window-end nil t))))
             (arg-string (format "git blame -L %s,%s --incremental -- %s" beg end file)))
        (let ((proc (git-make-process "sh" `("-c" ,arg-string) #'git-blame-handler dir)))
          (process-put proc 'command-buf (current-buffer)))))))
(advice-add 'magit-blame--run :around #'ng-magit-blame--run)

(defun ng-git-blame-chunk (&rest args)
  "Create blame chunk."
  (magit-blame-chunk
   :orig-rev   (plist-get args :orig-rev)
   :orig-line  (plist-get args :orig-line)
   :final-line  (plist-get args :final-line)
   :num-lines   (plist-get args :num-lines)))

(defun ng-git-blame-sentinel (proc)
  (sit-for 0.1)
  (delete-process proc)
  (while (eq (process-status proc) 'run)
    (sit-for 0.05)))

(defun ng-git-blame-make-overlays (proc chunk revinfo beg end)
  "Call magit's functions to create blame overlays."
  (when (process-live-p proc)
   (with-current-buffer (process-get proc 'command-buf)
     (save-excursion
       (save-restriction
         (magit-blame--remove-overlays beg end)
         (magit-blame--make-margin-overlays chunk revinfo beg end)
         (magit-blame--make-heading-overlay chunk revinfo beg end)
         (magit-blame--make-highlight-overlay chunk beg))))))

;; `window-scroll-functions` hook is special as func gets called with
;; window and window-start as args
(defun ng-magit-blame-scroll-refresh-hook (_window pos)
  "Refresh blame when window start has changed.
Also check if there's already a running blame process."
  (let ((proc (concat "async-msg-buffer")))
    (unless (process-live-p (get-process proc))
      (magit-blame--refresh))))

(defun ng-magit-blame-quit-hook (orig-fun &rest args)
  "Reset variable and hook when quitting blame minor mode."
  (remove-hook 'window-scroll-functions 'ng-magit-blame-scroll-refresh-hook t)
  (apply orig-fun args)
  ;; just to be safe
  (ignore-errors (delete-process proc)))
(advice-add 'magit-blame-quit :around #'ng-magit-blame-quit-hook)

;; -*-no-byte-compile: t; -*-
;; Local Variables:
;; byte-compile-warnings: (not unresolved free-vars lexical)
;; End:
(provide 'ng-magit)
