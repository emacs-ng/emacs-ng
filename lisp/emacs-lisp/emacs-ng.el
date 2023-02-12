(provide 'emacs-ng)

(defun js-init-lisp-thread ()
    (make-thread '(lambda () (js-lisp-thread))))

(defun js-eval-lisp-string (txt)
    (with-temp-buffer
        (insert txt)
        (eval-buffer)))