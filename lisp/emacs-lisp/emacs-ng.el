(provide 'emacs-ng)

(defun js-init-lisp-thread ()
    (make-thread '(lambda () (js-lisp-thread))))

(defun js-eval-string (txt)
    (with-temp-buffer
        (insert txt)
        (eval-buffer)))