(provide 'emacs-ng)

(defun js-init-lisp-thread ()
    (make-thread '(lambda () (js-lisp-thread))))

(defun js-eval-lisp-string (string)
  (condition-case err
    (eval (car (read-from-string string)))
    (error (cons 'js-error (prin1-to-string err)))))

(defun js-resolve-with-callback (id callback)
  (run-with-timer 0.1 nil #'(lambda (id callback)
                                (defvar result (js-resolve id))
                                (if (eq result 'js-not-ready)
                                  (js-resolve-with-callback id callback)
                                  (funcall callback result))
    ) id callback))