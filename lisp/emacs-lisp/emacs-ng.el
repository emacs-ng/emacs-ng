(provide 'emacs-ng)

(defun js-init-lisp-thread ()
    (make-thread '(lambda () (js-lisp-thread))))

(defun js-eval-lisp-string (string)
  (eval (car (read-from-string string))))