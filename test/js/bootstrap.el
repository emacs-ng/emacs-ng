(defun handler (e) (print e))
(setenv "DENO_DIR" "test/js/")
(js-initialize :js-error-handler 'handler)
(eval-js-file "./js/main.js")
(sleep-for 999999) ;; Since we are running async tests, we want to keep the event loop
                   ;; running to allow them to finish. We will manually
                   ;;exit the program upon completion
