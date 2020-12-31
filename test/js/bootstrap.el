(eval-js-file "./test/js/main.js")
(sleep-for 999999) ;; Since we are running async tests, we want to keep the event loop
                   ;; running to allow them to finish. We will manually
                   ;;exit the program upon completion
