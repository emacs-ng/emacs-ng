(provide 'emacs-ng)

(defun js-init-lisp-thread ()
    (make-thread '(lambda () (js-lisp-thread))))

(defun js-eval-lisp-string (string)
  (condition-case err
    (eval (car (read-from-string string)))
    (error (cons 'js-error (prin1-to-string err)))))