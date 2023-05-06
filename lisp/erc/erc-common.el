;;; erc-common.el --- Macros and types for ERC  -*- lexical-binding:t -*-

;; Copyright (C) 2022-2023 Free Software Foundation, Inc.
;;
;; Maintainer: Amin Bandali <bandali@gnu.org>, F. Jason Park <jp@neverwas.me>
;; Keywords: comm, IRC, chat, client, internet
;;
;; This file is part of GNU Emacs.
;;
;; GNU Emacs is free software: you can redistribute it and/or modify
;; it under the terms of the GNU General Public License as published
;; by the Free Software Foundation, either version 3 of the License,
;; or (at your option) any later version.
;;
;; GNU Emacs is distributed in the hope that it will be useful, but
;; WITHOUT ANY WARRANTY; without even the implied warranty of
;; MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
;; General Public License for more details.
;;
;; You should have received a copy of the GNU General Public License
;; along with GNU Emacs.  If not, see <https://www.gnu.org/licenses/>.

;;; Commentary:
;;; Code:

(eval-when-compile (require 'cl-lib) (require 'subr-x))
(require 'erc-compat)

(defvar erc--casemapping-rfc1459)
(defvar erc--casemapping-rfc1459-strict)
(defvar erc-channel-users)
(defvar erc-dbuf)
(defvar erc-insert-this)
(defvar erc-log-p)
(defvar erc-modules)
(defvar erc-send-this)
(defvar erc-server-process)
(defvar erc-server-users)
(defvar erc-session-server)

(declare-function erc--get-isupport-entry "erc-backend" (key &optional single))
(declare-function erc-get-buffer "erc" (target &optional proc))
(declare-function erc-server-buffer "erc" nil)
(declare-function widget-apply-action "wid-edit" (widget &optional event))
(declare-function widget-at "wid-edit" (&optional pos))
(declare-function widget-create-child-and-convert "wid-edit"
                  (parent type &rest args))
(declare-function widget-default-format-handler "wid-edit" (widget escape))
(declare-function widget-get-sibling "wid-edit" (widget))
(declare-function widget-move "wid-edit" (arg &optional suppress-echo))
(declare-function widget-type "wid-edit" (widget))

(cl-defstruct erc-input
  string insertp sendp refoldp)

(cl-defstruct (erc--input-split (:include erc-input
                                          (string :read-only)
                                          (insertp erc-insert-this)
                                          (sendp erc-send-this)))
  (lines nil :type (list-of string))
  (cmdp nil :type boolean))

(cl-defstruct (erc-server-user (:type vector) :named)
  ;; User data
  nickname host login full-name info
  ;; Buffers
  (buffers nil))

(cl-defstruct (erc-channel-user (:type vector) :named)
  voice halfop op admin owner
  ;; Last message time (in the form of the return value of
  ;; (current-time)
  ;;
  ;; This is useful for ordered name completion.
  (last-message-time nil))

(cl-defstruct erc--target
  (string "" :type string :documentation "Received name of target.")
  (symbol nil :type symbol :documentation "Case-mapped name as symbol."))

;; At some point, it may make sense to add a query type with an
;; account field, which may help support reassociation across
;; reconnects and nick changes (likely requires v3 extensions).
;;
;; These channel variants should probably take on a `joined' field to
;; track "joinedness", which `erc-server-JOIN', `erc-server-PART',
;; etc. should toggle.  Functions like `erc--current-buffer-joined-p'
;; may find it useful.

(cl-defstruct (erc--target-channel (:include erc--target)))
(cl-defstruct (erc--target-channel-local (:include erc--target-channel)))

;; Beginning in 5.5/29.1, the `tags' field may take on one of two
;; differing types.  See `erc-tags-format' for details.

(cl-defstruct (erc-response (:conc-name erc-response.))
  (unparsed "" :type string)
  (sender "" :type string)
  (command "" :type string)
  (command-args '() :type list)
  (contents "" :type string)
  (tags '() :type list))

;; After dropping 28, we can use prefixed "erc-autoload" cookies.
(defun erc--normalize-module-symbol (symbol)
  "Return preferred SYMBOL for `erc--modules'."
  (while-let ((canonical (get symbol 'erc--module))
              ((not (eq canonical symbol))))
    (setq symbol canonical))
  symbol)

(defvar erc--inside-mode-toggle-p nil
  "Non-nil when a module's mode toggle is updating module membership.
This serves as a flag to inhibit the mutual recursion that would
otherwise occur between an ERC-defined minor-mode function, such
as `erc-services-mode', and the custom-set function for
`erc-modules'.  For historical reasons, the latter calls
`erc-update-modules', which, in turn, enables the minor-mode
functions for all member modules.  Also non-nil when a mode's
widget runs its set function.")

(defun erc--favor-changed-reverted-modules-state (name op)
  "Be more nuanced in displaying Custom state of `erc-modules'.
When `customized-value' differs from `saved-value', allow widget
to behave normally and show \"SET for current session\", as
though `customize-set-variable' or similar had been applied.
However, when `customized-value' and `standard-value' match but
differ from `saved-value', prefer showing \"CHANGED outside
Customize\" to prevent the widget from seeing a `standard'
instead of a `set' state, which precludes any actual saving."
  ;; Although the button "Apply and save" is fortunately grayed out,
  ;; `Custom-save' doesn't actually save (users must click the magic
  ;; state button instead).  The default behavior described in the doc
  ;; string is intentional and was introduced by bug#12864 "Make state
  ;; button interaction less confusing".  However, it is unfriendly to
  ;; rogue libraries (like ours) that insist on mutating user options
  ;; as a matter of course.
  (custom-load-symbol 'erc-modules)
  (funcall (get 'erc-modules 'custom-set) 'erc-modules
           (funcall op (erc--normalize-module-symbol name) erc-modules))
  (when (equal (pcase (get 'erc-modules 'saved-value)
                 (`((quote ,saved) saved)))
               erc-modules)
    (customize-mark-as-set 'erc-modules)))

(defun erc--assemble-toggle (localp name ablsym mode val body)
  (let ((arg (make-symbol "arg")))
    `(defun ,ablsym ,(if localp `(&optional ,arg) '())
       ,(erc--fill-module-docstring
         (if val "Enable" "Disable")
         " ERC " (symbol-name name) " mode."
         (when localp
           (concat "\nWhen called interactively,"
                   " do so in all buffers for the current connection.")))
       (interactive ,@(when localp '("p")))
       ,@(if localp
             `((when (derived-mode-p 'erc-mode)
                 (if ,arg
                     (erc-with-all-buffers-of-server erc-server-process nil
                       (,ablsym))
                   (setq ,mode ,val)
                   ,@body)))
           ;; No need for `default-value', etc. because a buffer-local
           ;; `erc-modules' only influences the next session and
           ;; doesn't survive the major-mode reset that soon follows.
           `((unless
                 (or erc--inside-mode-toggle-p
                     ,@(let ((v `(memq ',(erc--normalize-module-symbol name)
                                       erc-modules)))
                         `(,(if val v `(not ,v)))))
               (let ((erc--inside-mode-toggle-p t))
                 (erc--favor-changed-reverted-modules-state
                  ',name #',(if val 'cons 'delq))))
             (setq ,mode ,val)
             ,@body)))))

;; This is a migration helper that determines a module's `:group'
;; keyword argument from its name or alias.  A (global) module's minor
;; mode variable appears under the group's Custom menu.  Like
;; `erc--normalize-module-symbol', it must run when the module's
;; definition (rather than that of `define-erc-module') is expanded.
;; For corner cases in which this fails or the catch-all of `erc' is
;; more inappropriate, (global) modules can declare a top-level
;;
;;   (put 'foo 'erc-group 'erc-bar)
;;
;; where `erc-bar' is the group and `foo' is the normalized module.
;; Do this *before* the module's definition.  If `define-erc-module'
;; ever accepts arbitrary keywords, passing an explicit `:group' will
;; obviously be preferable.

(defun erc--find-group (&rest symbols)
  (catch 'found
    (dolist (s symbols)
      (let* ((downed (downcase (symbol-name s)))
             (known (intern-soft (concat "erc-" downed))))
        (when (and known
                   (or (get known 'group-documentation)
                       (rassq known custom-current-group-alist)))
          (throw 'found known))
        (when (setq known (intern-soft (concat "erc-" downed "-mode")))
          (when-let ((found (custom-group-of-mode known)))
            (throw 'found found))))
      (when-let ((found (get (erc--normalize-module-symbol s) 'erc-group)))
        (throw 'found found)))
    'erc))

;; This exists as a separate, top-level function to prevent the byte
;; compiler from warning about widget-related dependencies not being
;; loaded at runtime.

(defun erc--tick-module-checkbox (name &rest _) ; `name' must be normalized
  (customize-variable-other-window 'erc-modules)
  ;; Move to `erc-modules' section.
  (while (not (eq (widget-type (widget-at)) 'checkbox))
    (widget-move 1 t))
  ;; This search for a checkbox can fail when `name' refers to a
  ;; third-party module that modifies `erc-modules' (improperly) on
  ;; load.
  (let (w)
    (while (and (eq (widget-type (widget-at)) 'checkbox)
                (not (and (setq w (widget-get-sibling (widget-at)))
                          (eq (widget-value w) name))))
      (setq w nil)
      (widget-move 1 t)) ; the `suppress-echo' arg exists in 27.2
    (unless w
      (error "Failed to find %s in `erc-modules' checklist" name))
    (widget-apply-action (widget-at))
    (message "Hit %s to apply or %s to apply and save."
             (substitute-command-keys "\\[Custom-set]")
             (substitute-command-keys "\\[Custom-save]"))))

;; This stands apart to avoid needing forward declarations for
;; `wid-edit' functions in every file requiring `erc-common'.
(defun erc--make-show-me-widget (widget escape &rest plist)
  (if (eq escape ?i)
      (apply #'widget-create-child-and-convert widget 'push-button plist)
    (widget-default-format-handler widget escape)))

(defun erc--prepare-custom-module-type (name)
  `(let* ((name (erc--normalize-module-symbol ',name))
          (fmtd (format " `%s' " name)))
     `(boolean
       :format "%{%t%}: %i %[Deprecated Toggle%] %v \n%h\n"
       :format-handler
       ,(lambda (widget escape)
          (erc--make-show-me-widget
           widget escape
           :button-face '(custom-variable-obsolete custom-button)
           :tag "Show Me"
           :action (apply-partially #'erc--tick-module-checkbox name)
           :help-echo (lambda (_)
                        (let ((hasp (memq name erc-modules)))
                          (concat (if hasp "Remove" "Add") fmtd
                                  (if hasp "from" "to")
                                  " `erc-modules'.")))))
       :action widget-toggle-action
       :documentation-property
       ,(lambda (_)
          (let ((hasp (memq name erc-modules)))
            (concat
             "Setting a module's minor-mode variable is "
             (propertize "ineffective" 'face 'error)
             ".\nPlease " (if hasp "remove" "add") fmtd
             (if hasp "from" "to") " `erc-modules' directly instead.\n"
             "You can do so now by clicking "
             (propertize "Show Me" 'face 'custom-variable-obsolete)
             " above."))))))

(defun erc--fill-module-docstring (&rest strings)
  (with-temp-buffer
    (emacs-lisp-mode)
    (insert "(defun foo ()\n"
            (format "%S" (apply #'concat strings))
            "\n(ignore))")
    (goto-char (point-min))
    (forward-line 2)
    (let ((emacs-lisp-docstring-fill-column 65)
          (sentence-end-double-space t))
      (fill-paragraph))
    (goto-char (point-min))
    (nth 3 (read (current-buffer)))))

(defmacro erc--find-feature (name alias)
  `(pcase (erc--find-group ',name ,(and alias (list 'quote alias)))
     ('erc (and-let* ((file (or (macroexp-file-name) buffer-file-name)))
             (intern (file-name-base file))))
     (v v)))

(defmacro define-erc-module (name alias doc enable-body disable-body
                                  &optional local-p)
  "Define a new minor mode using ERC conventions.
Symbol NAME is the name of the module.
Symbol ALIAS is the alias to use, or nil.
DOC is the documentation string to use for the minor mode.
ENABLE-BODY is a list of expressions used to enable the mode.
DISABLE-BODY is a list of expressions used to disable the mode.
If LOCAL-P is non-nil, the mode will be created as a buffer-local
mode, rather than a global one.

This will define a minor mode called erc-NAME-mode, possibly
an alias erc-ALIAS-mode, as well as the helper functions
erc-NAME-enable, and erc-NAME-disable.

With LOCAL-P, these helpers take on an optional argument that,
when non-nil, causes them to act on all buffers of a connection.
This feature is mainly intended for interactive use and does not
carry over to their respective minor-mode toggles.  Beware that
for global modules, these helpers and toggles all mutate
`erc-modules'.

Example:

  ;;;###autoload(autoload \\='erc-replace-mode \"erc-replace\")
  (define-erc-module replace nil
    \"This mode replaces incoming text according to `erc-replace-alist'.\"
    ((add-hook \\='erc-insert-modify-hook
               #\\='erc-replace-insert))
    ((remove-hook \\='erc-insert-modify-hook
                  #\\='erc-replace-insert)))"
  (declare (doc-string 3) (indent defun))
  (let* ((sn (symbol-name name))
         (mode (intern (format "erc-%s-mode" (downcase sn))))
         (enable (intern (format "erc-%s-enable" (downcase sn))))
         (disable (intern (format "erc-%s-disable" (downcase sn)))))
    `(progn
       (define-minor-mode
         ,mode
         ,(erc--fill-module-docstring (format "Toggle ERC %s mode.
With a prefix argument ARG, enable %s if ARG is positive,
and disable it otherwise.  If called from Lisp, enable the mode
if ARG is omitted or nil.
\n%s" name name doc))
         :global ,(not local-p)
         :group (erc--find-group ',name ,(and alias (list 'quote alias)))
         ,@(unless local-p `(:require ',(erc--find-feature name alias)))
         ,@(unless local-p `(:type ,(erc--prepare-custom-module-type name)))
         (if ,mode
             (,enable)
           (,disable)))
       ,(erc--assemble-toggle local-p name enable mode t enable-body)
       ,(erc--assemble-toggle local-p name disable mode nil disable-body)
       ,@(and-let* ((alias)
                    ((not (eq name alias)))
                    (aname (intern (format "erc-%s-mode"
                                           (downcase (symbol-name alias))))))
           `((defalias ',aname #',mode)
             (put ',aname 'erc-module ',(erc--normalize-module-symbol name))))
       (put ',mode 'erc-module ',(erc--normalize-module-symbol name))
       ;; For find-function and find-variable.
       (put ',mode    'definition-name ',name)
       (put ',enable  'definition-name ',name)
       (put ',disable 'definition-name ',name))))

(defmacro erc-with-buffer (spec &rest body)
  "Execute BODY in the buffer associated with SPEC.

SPEC should have the form

 (TARGET [PROCESS])

If TARGET is a buffer, use it.  Otherwise, use the buffer
matching TARGET in the process specified by PROCESS.

If PROCESS is nil, use the current `erc-server-process'.
See `erc-get-buffer' for details.

See also `with-current-buffer'.

\(fn (TARGET [PROCESS]) BODY...)"
  (declare (indent 1) (debug ((form &optional form) body)))
  (let ((buf (make-symbol "buf"))
        (proc (make-symbol "proc"))
        (target (make-symbol "target"))
        (process (make-symbol "process")))
    `(let* ((,target ,(car spec))
            (,process ,(cadr spec))
            (,buf (if (bufferp ,target)
                      ,target
                    (let ((,proc (or ,process
                                     (and (processp erc-server-process)
                                          erc-server-process))))
                      (if (and ,target ,proc)
                          (erc-get-buffer ,target ,proc))))))
       (when (buffer-live-p ,buf)
         (with-current-buffer ,buf
           ,@body)))))

(defmacro erc-with-server-buffer (&rest body)
  "Execute BODY in the current ERC server buffer.
If no server buffer exists, return nil."
  (declare (indent 0) (debug (body)))
  (let ((varp (and (symbolp (car body))
                   (not (cdr body))
                   (special-variable-p (car body))))
        (buffer (make-symbol "buffer")))
    `(when-let* (((processp erc-server-process))
                 (,buffer (process-buffer erc-server-process))
                 ((buffer-live-p ,buffer)))
       ,(if varp
            `(buffer-local-value ',(car body) ,buffer)
          `(with-current-buffer ,buffer
             ,@body)))))

(defmacro erc-with-all-buffers-of-server (process pred &rest forms)
  "Execute FORMS in all buffers which have same process as this server.
FORMS will be evaluated in all buffers having the process PROCESS and
where PRED matches or in all buffers of the server process if PRED is
nil."
  (declare (indent 2) (debug (form form body)))
  (macroexp-let2 nil pred pred
    `(erc-buffer-filter (lambda ()
                          (when (or (not ,pred) (funcall ,pred))
                            ,@forms))
                        ,process)))

(defun erc-log-aux (string)
  "Do the debug logging of STRING."
  (let ((cb (current-buffer))
        (point 1)
        (was-eob nil)
        (session-buffer (erc-server-buffer)))
    (if session-buffer
        (progn
          (set-buffer session-buffer)
          (if (not (and erc-dbuf (bufferp erc-dbuf) (buffer-live-p erc-dbuf)))
              (progn
                (setq erc-dbuf (get-buffer-create
                                (concat "*ERC-DEBUG: "
                                        erc-session-server "*")))))
          (set-buffer erc-dbuf)
          (setq point (point))
          (setq was-eob (eobp))
          (goto-char (point-max))
          (insert (concat "** " string "\n"))
          (if was-eob (goto-char (point-max))
            (goto-char point))
          (set-buffer cb))
      (message "ERC: ** %s" string))))

(define-inline erc-log (string)
  "Logs STRING if logging is on (see `erc-log-p')."
  (inline-quote
   (when erc-log-p
     (erc-log-aux ,string))))

(defun erc-downcase (string)
  "Return a downcased copy of STRING with properties.
Use the CASEMAPPING ISUPPORT parameter to determine the style."
  (with-case-table (pcase (erc--get-isupport-entry 'CASEMAPPING 'single)
                     ("ascii" ascii-case-table)
                     ("rfc1459-strict" erc--casemapping-rfc1459-strict)
                     (_ erc--casemapping-rfc1459))
    (downcase string)))

(define-inline erc-get-channel-user (nick)
  "Find NICK in the current buffer's `erc-channel-users' hash table."
  (inline-quote (gethash (erc-downcase ,nick) erc-channel-users)))

(define-inline erc-get-server-user (nick)
  "Find NICK in the current server's `erc-server-users' hash table."
  (inline-letevals (nick)
    (inline-quote (erc-with-server-buffer
                    (gethash (erc-downcase ,nick) erc-server-users)))))

(provide 'erc-common)

;;; erc-common.el ends here
