;;; vterm.el --- This package implements a terminal via libvterm -*- lexical-binding: t; -*-

(require 'term)
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
The foreground color is used as ANSI color 0 and the background
color is used as ANSI color 7."
  :group 'vterm)

(defface vterm-color-black
  `((t :inherit term-color-black))
  "Face used to render black color code.
The foreground color is used as ANSI color 0 and the background
color is used as ANSI color 8."
  :group 'vterm)

(defface vterm-color-red
  `((t :inherit term-color-red))
  "Face used to render red color code.
The foreground color is used as ANSI color 1 and the background
color is used as ANSI color 9."
  :group 'vterm)

(defface vterm-color-green
  `((t :inherit term-color-green))
  "Face used to render green color code.
The foreground color is used as ANSI color 2 and the background
color is used as ANSI color 10."
  :group 'vterm)

(defface vterm-color-yellow
  `((t :inherit term-color-yellow))
  "Face used to render yellow color code.
The foreground color is used as ANSI color 3 and the background
color is used as ANSI color 11."
  :group 'vterm)

(defface vterm-color-blue
  `((t :inherit term-color-blue))
  "Face used to render blue color code.
The foreground color is used as ANSI color 4 and the background
color is used as ANSI color 12."
  :group 'vterm)

(defface vterm-color-magenta
  `((t :inherit term-color-magenta))
  "Face used to render magenta color code.
The foreground color is used as ansi color 5 and the background
color is used as ansi color 13."
  :group 'vterm)

(defface vterm-color-cyan
  `((t :inherit term-color-cyan))
  "Face used to render cyan color code.
The foreground color is used as ansi color 6 and the background
color is used as ansi color 14."
  :group 'vterm)

(defface vterm-color-white
  `((t :inherit term-color-white))
  "Face used to render white color code.
The foreground color is used as ansi color 7 and the background
color is used as ansi color 15."
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
    (define-key map [tab]                       #'vterm-send-tab)
    (define-key map (kbd "TAB")                 #'vterm-send-tab)
    (define-key map [backtab]                   #'vterm-self-insert)
    (define-key map [backspace]                 #'vterm-send-backspace)
    (define-key map (kbd "DEL")                 #'vterm-send-backspace)
    (define-key map [M-backspace]               #'vterm-send-meta-backspace)
    (define-key map (kbd "M-DEL")               #'vterm-send-meta-backspace)
    (define-key map [return]                    #'vterm-send-return)
    (define-key map (kbd "RET")                 #'vterm-send-return)
    (define-key map [left]                      #'vterm-send-left)
    (define-key map [right]                     #'vterm-send-right)
    (define-key map [up]                        #'vterm-send-up)
    (define-key map [down]                      #'vterm-send-down)
    (define-key map [prior]                     #'vterm-send-prior)
    (define-key map [next]                      #'vterm-send-next)
    (define-key map [home]                      #'vterm-self-insert)
    (define-key map [end]                       #'vterm-self-insert)
    (define-key map [escape]                    #'vterm-self-insert)
    (define-key map [remap yank]                #'vterm-yank)
    (define-key map [remap yank-pop]            #'vterm-yank-pop)
    (define-key map (kbd "C-SPC")               #'vterm-self-insert)
    (define-key map (kbd "C-_")                 #'vterm-self-insert)
    (define-key map (kbd "C-/")                 #'vterm-undo)
    (define-key map (kbd "M-.")                 #'vterm-send-meta-dot)
    (define-key map (kbd "M-,")                 #'vterm-send-meta-comma)
    (define-key map (kbd "C-c C-y")             #'vterm-self-insert)
    (define-key map (kbd "C-c C-c")             #'vterm-send-ctrl-c)
    (define-key map [remap self-insert-command] #'vterm-self-insert)

    (define-key map (kbd "C-c C-t")             #'vterm-copy-mode)
    map)
  "Keymap for `vterm-mode'.")

(defvar vterm-copy-mode-map (make-sparse-keymap)
  "Minor mode map for `vterm-copy-mode'.")
(define-key vterm-copy-mode-map (kbd "C-c C-t")        #'vterm-copy-mode)
(define-key vterm-copy-mode-map [return]               #'vterm-copy-mode-done)
(define-key vterm-copy-mode-map (kbd "RET")            #'vterm-copy-mode-done)

(defvar-local vterm--copy-saved-point nil)

(define-minor-mode vterm-copy-mode
  "Toggle vterm copy mode."
  :group 'vterm
  :lighter " VTermCopy"
  :keymap vterm-copy-mode-map
  (if vterm-copy-mode
      (progn                            ;enable vterm-copy-mode
        (use-local-map nil)
        (vterm-send-stop)
        (setq vterm--copy-saved-point (point)))
    (if vterm--copy-saved-point
        (goto-char vterm--copy-saved-point))
    (use-local-map vterm-mode-map)
    (vterm-send-start)))

(defun vterm-copy-mode-done ()
  "Save the active region to the kill ring and exit `vterm-copy-mode'."
  (interactive)
  (unless vterm-copy-mode
    (user-error "This command is effective only in vterm-copy-mode"))
  (unless (region-active-p)
    (user-error "No region is active"))
  (kill-ring-save (region-beginning) (region-end))
  (vterm-copy-mode -1))

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


(defcustom vterm-term-environment-variable "xterm-256color"
  "TERM value for terminal."
  :type 'string
  :group 'vterm)

(define-derived-mode vterm-mode fundamental-mode "VTerm"
  "Major mode for vterm buffer."
  (buffer-disable-undo)
  (let ((buf (current-buffer))
        (height (window-body-height))
        (width (window-body-width))
        proc
        ;; scrollback works only correctly with this ???
        (scrollback (- vterm-max-scrollback 61)))
    (let ((process-environment (append `(,(concat "TERM="
						                        vterm-term-environment-variable)
                                       "INSIDE_EMACS=vterm"
                                       "LINES"
                                       "COLUMNS")
                                     process-environment))
        (process-adaptive-read-buffering nil))
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
        (let ((height (window-body-height window))
              (width (window-body-width window))
              (inhibit-read-only t))
          (set-process-window-size vterm--process height width)
          (vterm-set-size vterm--term height width))))))

(defun vterm-self-insert ()
  "Sends invoking key to libvterm."
  (interactive)
  (when vterm--term
    (let* ((modifiers (event-modifiers last-input-event))
           (shift (memq 'shift modifiers))
           (meta (memq 'meta modifiers))
           (ctrl (memq 'control modifiers)))
      (when-let ((key (key-description (vector (event-basic-type last-input-event)))))
        (vterm-send-key key shift meta ctrl)))))

(defun vterm-send-key (key &optional shift meta ctrl)
  "Sends KEY to libvterm with optional modifiers SHIFT, META and CTRL."
  (when vterm--term
    (let ((inhibit-redisplay t)
          (inhibit-read-only t))
      (when (and (not (symbolp last-input-event)) shift (not meta) (not ctrl))
        (setq key (upcase key)))
      (vterm-update vterm--term key shift meta ctrl))))

(defun vterm-send-start ()
  "Output from the system is started when the system receives START."
  (interactive)
  (vterm-send-key "<start>"))

(defun vterm-send-stop ()
  "Output from the system is stopped when the system receives STOP."
  (interactive)
  (vterm-send-key "<stop>"))

(defun vterm-send-return ()
  "Sends `<return>' to the libvterm."
  (interactive)
  (vterm-send-key "<return>"))

(defun vterm-send-tab ()
  "Sends `<tab>' to the libvterm."
  (interactive)
  (vterm-send-key "<tab>"))

(defun vterm-send-backspace ()
  "Sends `<backspace>' to the libvterm."
  (interactive)
  (vterm-send-key "<backspace>"))

(defun vterm-send-meta-backspace ()
  "Sends `M-<backspace>' to the libvterm."
  (interactive)
  (vterm-send-key "<backspace>" nil t))

(defun vterm-send-up ()
  "Sends `<up>' to the libvterm."
  (interactive)
  (vterm-send-key "<up>"))

(defun vterm-send-down ()
  "Sends `<down>' to the libvterm."
  (interactive)
  (vterm-send-key "<down>"))

(defun vterm-send-left()
  "Sends `<left>' to the libvterm."
  (interactive)
  (vterm-send-key "<left>"))

(defun vterm-send-right()
  "Sends `<right>' to the libvterm."
  (interactive)
  (vterm-send-key "<right>"))

(defun vterm-send-prior()
  "Sends `<prior>' to the libvterm."
  (interactive)
  (vterm-send-key "<prior>"))

(defun vterm-send-next()
  "Sends `<next>' to the libvterm."
  (interactive)
  (vterm-send-key "<next>"))

(defun vterm-send-meta-dot()
  "Sends `M-.' to the libvterm."
  (interactive)
  (vterm-send-key "." nil t))

(defun vterm-send-meta-comma()
  "Sends `M-,' to the libvterm."
  (interactive)
  (vterm-send-key "," nil t))

(defun vterm-send-ctrl-c ()
  "Sends `C-c' to the libvterm."
  (interactive)
  (vterm-send-key "c" nil nil t))

(defun vterm-undo ()
  "Sends `C-_' to the libvterm."
  (interactive)
  (vterm-send-key "_" nil nil t))

(defun vterm-yank (&optional arg)
  "Implementation of `yank' (paste) in vterm."
  (interactive "P")
  (let ((inhibit-read-only t))
    (cl-letf (((symbol-function 'insert-for-yank)
               #'(lambda(str) (vterm-send-string str t))))
      (yank arg))))

(defun vterm-yank-pop(&optional arg)
  "Implementation of `yank-pop' in vterm."
  (interactive "p")
  (let ((inhibit-read-only t)
        (yank-undo-function #'(lambda(_start _end) (vterm-undo))))
    (cl-letf (((symbol-function 'insert-for-yank)
               #'(lambda(str) (vterm-send-string str t))))
      (yank-pop arg))))

(defun vterm-send-string (string &optional paste-p)
  "Send the string STRING to vterm.
Optional argument PASTE-P paste-p."
  (when vterm--term
    (when paste-p
      (vterm-update vterm--term "<start_paste>" nil nil nil))
    (dolist (char (string-to-list string))
      (vterm-update vterm--term (char-to-string char) nil nil nil))
    (when paste-p
      (vterm-update vterm--term "<end_paste>" nil nil nil))))

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

(defun vterm-set-directory (path)
  "Set `default-directory' to PATH."
  (if (string-match "^\\(.*?\\)@\\(.*?\\):\\(.*?\\)$" path)
      (progn
        (let ((user (match-string 1 path))
              (host (match-string 2 path))
              (dir (match-string 3 path)))
          (if (and (string-equal user user-login-name)
                   (string-equal host (system-name)))
              (progn
                (when (file-directory-p dir)
                  (setq default-directory dir)))
            (setq default-directory (concat "/-:" path)))))
    (when (file-directory-p path)
      (setq default-directory path))))

(defun vterm--goto-line(n)
  "Go to line N and return true on success.
if N is negative backward-line from end of buffer."
  (cond
   ((> n 0)
    (goto-char (point-min))
    (eq 0 (forward-line (1- n))))
   (t
    (goto-char (point-max))
    (eq 0 (forward-line n)))))

(defun vterm--delete-lines (line-num count &optional delete-whole-line)
  "Delete COUNT lines from LINE-NUM.
if LINE-NUM is negative backward-line from end of buffer.
 If option DELETE-WHOLE-LINE is non-nil, then this command kills
 the whole line including its terminating newline"
  (save-excursion
    (when (vterm--goto-line line-num)
      (delete-region (point) (point-at-eol count))
      (when (and delete-whole-line
                 (looking-at "\n"))
        (delete-char 1)))))

(defvar-local vterm--redraw-timer nil)

(defvar vterm-timer-delay 0.05
  "Delay for refreshing the buffer after receiving updates from libvterm.
Improves performance when receiving large bursts of data.
If nil, never delay")

(defun vterm--invalidate()
  "The terminal buffer is invalidated, the buffer needs redrawing."
  (if vterm-timer-delay
      (unless vterm--redraw-timer
        (setq vterm--redraw-timer
              (run-with-timer vterm-timer-delay nil
                              #'vterm--delayed-redraw (current-buffer))))
    (vterm--delayed-redraw (current-buffer))))

(defun vterm--delayed-redraw(buffer)
  "Redraw the terminal buffer .
Argument BUFFER the terminal buffer."
  (when (buffer-live-p buffer)
    (with-current-buffer buffer
      (let ((inhibit-redisplay t)
            (inhibit-read-only t))
        (setq vterm--redraw-timer nil)
        (when vterm--term
          (when (and (require 'display-line-numbers nil 'noerror)
                     (get-buffer-window buffer t)
                     (ignore-errors (display-line-numbers-update-width)))
            (window--adjust-process-windows))
          (vterminal-redraw vterm--term))))))

(provide 'vterm)
;;; vterm.el ends here

