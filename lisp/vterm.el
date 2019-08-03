(require 'subr-x)
(require 'cl-lib)
(require 'color)
(require 'ansi-color)

(defface vterm
  '((t :inherit default))
  "Default face to use in Term mode."
  :group 'vterm)

(defcustom vterm-shell (getenv "SHELL")
  "The shell that gets run in the vterm."
  :type 'string
  :group 'vterm)

(defcustom vterm-display-method 'switch-to-buffer
  "Default display method."
  :type '(choice (const :tag "Display buffer." 'switch-to-buffer)
                 (const :tag "Pop to buffer." 'pop-to-buffer))
  :group 'vterm)

(defcustom vterm-max-scrollback 1000
  "Maximum 'scrollback' value."
  :type 'number
  :group 'vterm)

(defcustom vterm-keymap-exceptions '("C-c" "C-x" "C-u" "C-g" "C-h" "M-x" "M-o" "C-v" "M-v" "C-y" "M-y")
  "Exceptions for vterm-keymap.

If you use a keybinding with a prefix-key, add that prefix-key to
this list. Note that after doing so that prefix-key cannot be sent
to the terminal anymore."
  :type '(repeat string)
  :set (lambda (sym val)
         (set sym val)
         (when (fboundp 'vterm--exclude-keys)
           (vterm--exclude-keys val)))
  :group 'vterm)

(defvar vterm--term nil
  "Pointer to Term.")
(make-variable-buffer-local 'vterm--term)

(defface vterm-color-default
  `((t :inherit default))
  "The default normal color and bright color.
the foreground color are used for normal color,
and background color are used for bright color. "
  :group 'vterm)

(defface vterm-color-black
  `((t :foreground ,(aref ansi-color-names-vector 0)
       :background ,(aref ansi-color-names-vector 0)))
  "Face used to render black color code.
the foreground color are used for normal color,
and background color are used for bright color. "
  :group 'vterm)

(defface vterm-color-red
  `((t :foreground ,(aref ansi-color-names-vector 1)
       :background ,(aref ansi-color-names-vector 1)))
  "Face used to render red color code.
the foreground color are used for normal color,
and background color are used for bright color. "
  :group 'vterm)

(defface vterm-color-green
  `((t :foreground ,(aref ansi-color-names-vector 2)
       :background ,(aref ansi-color-names-vector 2)))
  "Face used to render green color code.
the foreground color are used for normal color,
and background color are used for bright color. "
  :group 'vterm)

(defface vterm-color-yellow
  `((t :foreground ,(aref ansi-color-names-vector 3)
       :background ,(aref ansi-color-names-vector 3)))
  "Face used to render yellow color code.
the foreground color are used for normal color,
and background color are used for bright color. "
  :group 'vterm)

(defface vterm-color-blue
  `((t :foreground ,(aref ansi-color-names-vector 4)
       :background ,(aref ansi-color-names-vector 4)))
  "Face used to render blue color code.
the foreground color are used for normal color,
and background color are used for bright color. "
  :group 'vterm)

(defface vterm-color-magenta
  `((t :foreground ,(aref ansi-color-names-vector 5)
       :background ,(aref ansi-color-names-vector 5)))
  "Face used to render magenta color code.
the foreground color are used for normal color,
and background color are used for bright color. "
  :group 'vterm)

(defface vterm-color-cyan
  `((t :foreground ,(aref ansi-color-names-vector 6)
       :background ,(aref ansi-color-names-vector 6)))
  "Face used to render cyan color code.
the foreground color are used for normal color,
and background color are used for bright color. "
  :group 'vterm)

(defface vterm-color-white
  `((t :foreground ,(aref ansi-color-names-vector 7)
       :background ,(aref ansi-color-names-vector 7)))
  "Face used to render white color code.
the foreground color are used for normal color,
and background color are used for bright color. "
  :group 'vterm)

(defvar vterm-color-palette
  [vterm-color-black
   vterm-color-red
   vterm-color-green
   vterm-color-yellow
   vterm-color-blue
   vterm-color-magenta
   vterm-color-cyan
   vterm-color-white]
  "Color palette for the foreground and background.")

(defun vterm--get-color(index)
  "Get color by index from `vterm-color-palette'.
Argument INDEX index of color."
  (cond
   ((and (>= index 0)(< index 8 ))
    (face-foreground
     (elt vterm-color-palette index)
     nil 'default))
   ((and (>= index 8 )(< index 16 ))
    (face-background
     (elt vterm-color-palette (% index 8))
     nil 'default))
   ((= index -1)               ;-1 foreground
     (face-foreground 'vterm-color-default nil 'default))
   (t                                   ;-2 background
    (face-background 'vterm-color-default nil 'default))))

(defvar vterm-buffer-name "*vterm*"
  "Buffer name for vterm buffers.")

(defvar vterm-mode-map
  (let ((map (make-sparse-keymap)))
    (define-key map [tab]                       #'vterm-self-insert)
    (define-key map [backspace]                 #'vterm-self-insert)
    (define-key map [M-backspace]               #'vterm-self-insert)
    (define-key map [return]                    #'vterm-self-insert)
    (define-key map [left]                      #'vterm-self-insert)
    (define-key map [right]                     #'vterm-self-insert)
    (define-key map [up]                        #'vterm-self-insert)
    (define-key map [down]                      #'vterm-self-insert)
    (define-key map [home]                      #'vterm-self-insert)
    (define-key map [end]                       #'vterm-self-insert)
    (define-key map [escape]                    #'vterm-self-insert)
    (define-key map [remap self-insert-command] #'vterm-self-insert)
    (define-key map [remap yank]                #'vterm-yank)
    (define-key map (kbd "C-c C-y")             #'vterm-self-insert)
    (define-key map (kbd "C-c C-c")             #'vterm-send-ctrl-c)
    map)
  "Keymap for `vterm-mode'.")

;; Function keys and most of C- and M- bindings
(defun vterm--exclude-keys (exceptions)
  (mapc (lambda (key)
          (define-key vterm-mode-map (kbd key) nil))
        exceptions)
  (mapc (lambda (key)
          (define-key vterm-mode-map (kbd key) #'vterm--self-insert))
        (append (cl-loop for number from 1 to 12
                         for key = (format "<f%i>" number)
                         unless (member key exceptions)
                         collect key)
                (cl-loop for prefix in '("C-" "M-")
                         append (cl-loop for char from ?a to ?z
                                         for key = (format "%s%c" prefix char)
                                         unless (member key exceptions)
                                         collect key)))))

(vterm--exclude-keys vterm-keymap-exceptions)

(defvar-local vterm--process nil
  "Shell process of current term.")

(define-derived-mode vterm-mode fundamental-mode "VTerm"
  "Major mode for vterm buffer."
  (buffer-disable-undo)
  (let ((buf (current-buffer))
        (height (window-body-height))
        (width (window-body-width))
        proc
        (scrollback (- vterm-max-scrollback 61)))
    (let ((process-environment (append '("TERM=xterm-256color"
                                         "INSIDE_EMACS=vterm"
                                         "LINES"
                                         "COLUMNS")
                                       process-environment))
          ;; (process-adaptive-read-buffering nil))
          )
      (setq proc (make-process
                  :name "vterm"
                  :buffer buf
                  :command `("/bin/sh" "-c"
                             ,(format "stty -nl sane iutf8 rows %d columns %d >/dev/null && exec %s"
                                      height
                                      width
                                      vterm-shell))
                  :coding 'no-conversion
                  :connection-type 'pty
                  :filter #'vterm-filter
                  :sentinel #'vterm-sentinel)))
    (setq vterm--term (vterm-new height width (setq vterm--process proc) scrollback))
    )
  (setq buffer-read-only t)
  (setq-local scroll-conservatively 101)
  (setq-local scroll-margin 0)
  (add-hook 'window-size-change-functions #'vterm-resize-window t t)
  )

(defun vterm-filter (process output)
  "I/O Event. Feeds PROCESS's OUTPUT to the virtual terminal.

Then triggers a redraw from the module."
  (let ((inhibit-redisplay t)
        (inhibit-read-only t))
    (with-current-buffer (process-buffer process)
      (vterm-write-input vterm--term output)
      (vterm-update vterm--term))))

(defun vterm-sentinel (proc string)
  (let ((buf (process-buffer proc)))
    (when (buffer-live-p buf)
      (kill-buffer buf))))

(defun vterm-resize-window (frame)
  "Callback triggered by a size change of the FRAME.

Feeds the size change to the virtual terminal."
  (dolist (window (window-list frame))
    (with-current-buffer (window-buffer window)
      (when vterm--term
        (let ((height (- (window-body-height window) 1))
              (width (- (window-body-width window) 2))
              (inhibit-read-only t))
          (set-process-window-size vterm--process height width)
          ;; (set-process-window-size (vterm-process vterm--term) height width)
          (vterm-set-size vterm--term height width))))))

(defun vterm-self-insert ()
  "Sends invoking key to libvterm."
  (interactive)
    (let* ((modifiers (event-modifiers last-input-event))
           (shift (memq 'shift modifiers))
           (meta (memq 'meta modifiers))
           (ctrl (memq 'control modifiers)))
      (when-let ((key (key-description (vector (event-basic-type last-input-event)))))
        (vterm-send-key key shift meta ctrl))))

(defun vterm-send-key (key &optional shift meta ctrl)
  "Sends KEY to libvterm with optional modifiers SHIFT, META and CTRL."
    (let ((inhibit-redisplay t)
          (inhibit-read-only t))
      (when (and shift (not meta) (not ctrl))
        (setq key (upcase key)))
      (vterm-update vterm--term key shift meta ctrl)))

(defun vterm-send-ctrl-c ()
  "Sends C-c to the libvterm."
  (interactive)
  (vterm-send-key "c" nil nil t))

(defun vterm-yank ()
  "Implementation of `yank' (paste) in vterm."
  (interactive)
  (vterm-send-string (current-kill 0)))

(defun vterm-send-string (string)
  "Send the string STRING to vterm."
  (when vterm--term
    (dolist (char (string-to-list string))
      (vterm-update vterm--term (char-to-string char) nil nil nil))))

;;;###autoload
(defun vterm (&optional arg)
  "Display vterminal. If called with prefix arg open new terminal."
  (interactive "P")
  (let ((buffer (if arg
                    (generate-new-buffer vterm-buffer-name)
                  (get-buffer-create vterm-buffer-name))))
    (when (or arg (not (get-buffer-process buffer)))
      (with-current-buffer buffer
        (vterm-mode)))
    (vterm-resize-window (selected-frame))
    (funcall vterm-display-method buffer)))

(provide 'vterm)
;;; vterm.el ends here
