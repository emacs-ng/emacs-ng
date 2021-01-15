(defun handler (e) (print e))
(setenv "DENO_DIR" "test/js/")
(js-initialize :js-error-handler 'handler)
(eval-js-file "./js/main.js")
;; Since we are in batch
;; manually tick the event
;; loop
(run-with-timer t 0.1 'js-tick-event-loop 'handler)
;; Since we are running async tests, we want to keep the event loop
;; running to allow them to finish. We will manually
;; exit the program upon completion
(sleep-for 999999)
