;;; erc-stamp.el --- Timestamping for ERC messages  -*- lexical-binding:t -*-

;; Copyright (C) 2002-2004, 2006-2024 Free Software Foundation, Inc.

;; Author: Mario Lang <mlang@delysid.org>
;; Maintainer: Amin Bandali <bandali@gnu.org>, F. Jason Park <jp@neverwas.me>
;; Keywords: comm, timestamp
;; URL: https://www.emacswiki.org/emacs/ErcStamp

;; This file is part of GNU Emacs.

;; GNU Emacs is free software: you can redistribute it and/or modify
;; it under the terms of the GNU General Public License as published by
;; the Free Software Foundation, either version 3 of the License, or
;; (at your option) any later version.

;; GNU Emacs is distributed in the hope that it will be useful,
;; but WITHOUT ANY WARRANTY; without even the implied warranty of
;; MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
;; GNU General Public License for more details.

;; You should have received a copy of the GNU General Public License
;; along with GNU Emacs.  If not, see <https://www.gnu.org/licenses/>.

;;; Commentary:

;; The code contained in this module is responsible for inserting
;; timestamps into ERC buffers.  In order to actually activate this,
;; you must call `erc-timestamp-mode'.

;; You can choose between two different ways of inserting timestamps.
;; Customize `erc-insert-timestamp-function' and
;; `erc-insert-away-timestamp-function'.

;;; Code:

(require 'erc)

(defgroup erc-stamp nil
  "For long conversation on IRC it is sometimes quite
useful to have individual messages timestamp.  This
group provides settings related to the format and display
of timestamp information in `erc-mode' buffer.

For timestamping to be activated, you just need to load `erc-stamp'
in your init file or interactively using `load-library'."
  :group 'erc)

(defcustom erc-timestamp-format "[%H:%M]"
  "If set to a string, messages will be timestamped.
This string is processed using `format-time-string'.
Good examples are \"%T\" and \"%H:%M\".

If nil, timestamping is turned off."
  :type '(choice (const nil)
		 (string)))

(defcustom erc-timestamp-format-left "\n[%a %b %e %Y]\n"
  "Format recognized by `format-time-string' for date stamps.
Only considered when `erc-insert-timestamp-function' is set to
`erc-insert-timestamp-left-and-right'.  Used for displaying date
stamps on their own line, between messages.  ERC inserts this
flavor of stamp as a separate \"pseudo message\", so a final
newline isn't necessary.  For compatibility, only additional
trailing newlines beyond the first become empty lines.  For
example, the default value results in an empty line after the
previous message, followed by the timestamp on its own line,
followed immediately by the next message on the next line.  ERC
expects to display these stamps less frequently, so the
formatting specifiers should reflect that.  To omit these stamps
entirely, use a different `erc-insert-timestamp-function', such
as `erc-timestamp-format-right'.  Note that changing this value
during an ERC session requires cycling `erc-stamp-mode'."
  :type 'string)

(defcustom erc-timestamp-format-right nil
  "If set to a string, messages will be timestamped.
This string is processed using `format-time-string'.
Good examples are \"%T\" and \"%H:%M\".

This timestamp is used for timestamps on the right side of the
screen when `erc-insert-timestamp-function' is set to
`erc-insert-timestamp-left-and-right'.

Unlike `erc-timestamp-format' and `erc-timestamp-format-left', if
the value of this option is nil, it falls back to using the value
of `erc-timestamp-format'."
  :package-version '(ERC . "5.6")
  :type '(choice (const nil)
		 (string)))
(make-obsolete-variable 'erc-timestamp-format-right
                        'erc-timestamp-format "30.1")

(defcustom erc-insert-timestamp-function 'erc-insert-timestamp-left-and-right
  "Function to use to insert timestamps.

It takes a single argument STRING which is the final string
which all text-properties already appended.  This function only cares about
inserting this string at the right position.  Narrowing is in effect
while it is called, so (point-min) and (point-max) determine the region to
operate on.

You will probably want to set
`erc-insert-away-timestamp-function' to the same value."
  :type '(choice (const :tag "Both sides" erc-insert-timestamp-left-and-right)
		 (const :tag "Right" erc-insert-timestamp-right)
		 (const :tag "Left" erc-insert-timestamp-left)
		 function))

(defcustom erc-away-timestamp-format "<%H:%M>"
  "Timestamp format used when marked as being away.

If nil, timestamping is turned off when away unless `erc-timestamp-format'
is set.

If `erc-timestamp-format' is set, this will not be used."
  :type '(choice (const nil)
		 (string)))

(defcustom erc-insert-away-timestamp-function
  #'erc-insert-timestamp-left-and-right
  "Function to use to insert the away timestamp.

See `erc-insert-timestamp-function' for details."
  :type '(choice (const :tag "Both sides" erc-insert-timestamp-left-and-right)
		 (const :tag "Right" erc-insert-timestamp-right)
		 (const :tag "Left" erc-insert-timestamp-left)
		 function))

(defcustom erc-hide-timestamps nil
  "If non-nil, timestamps will be invisible.

This is useful for logging, because, although timestamps will be
hidden, they will still be present in the logs."
  :type 'boolean)

(defcustom erc-echo-timestamps nil
  "If non-nil, print timestamp in the minibuffer when point is moved.
Using this variable, you can turn off normal timestamping,
and simply move point to an irc message to see its timestamp
printed in the minibuffer.  When attempting to enable this option
after `erc-stamp-mode' is already active, you may need to run the
command `erc-show-timestamps' (or `erc-hide-timestamps') in the
appropriate ERC buffer before the change will take effect."
  :type 'boolean)

(defcustom erc-echo-timestamp-format "Timestamped %A, %H:%M:%S"
  "Format string to be used when `erc-echo-timestamps' is non-nil.
This string specifies the format of the timestamp being echoed in
the minibuffer."
  :type '(choice (const :tag "Timestamped Monday, 15:04:05"
                        "Timestamped %A, %H:%M:%S")
                 (const :tag "2006-01-02 15:04:05 MST" "%F %T %Z")
                 string))

(defcustom erc-echo-timestamp-zone nil
  "Default timezone for the option `erc-echo-timestamps'.
Also affects the command `erc-echo-timestamp' (singular).  See
the ZONE parameter of `format-time-string' for a description of
acceptable value types."
  :type '(choice boolean number (const wall) (list number string))
  :package-version '(ERC . "5.6"))

(defcustom erc-timestamp-intangible nil
  "Whether the timestamps should be intangible, i.e. prevent the point
from entering them and instead jump over them."
  :version "24.5"
  :type 'boolean)

(defface erc-timestamp-face '((t :weight bold :foreground "green"))
  "ERC timestamp face."
  :group 'erc-faces)

;; New libraries should only autoload the minor mode for a module's
;; preferred name (rather than its alias).

;;;###autoload(put 'timestamp 'erc--module 'stamp)
;;;###autoload(autoload 'erc-timestamp-mode "erc-stamp" nil t)
(define-erc-module stamp timestamp
  "This mode timestamps messages in the channel buffers."
  ((add-hook 'erc-mode-hook #'erc-stamp--setup)
   (add-hook 'erc-insert-modify-hook #'erc-add-timestamp 70)
   (add-hook 'erc-send-modify-hook #'erc-add-timestamp 70)
   (add-hook 'erc-mode-hook #'erc-stamp--recover-on-reconnect)
   (add-hook 'erc--pre-clear-functions #'erc-stamp--reset-on-clear 40)
   (unless erc--updating-modules-p (erc-buffer-do #'erc-stamp--setup)))
  ((remove-hook 'erc-mode-hook #'erc-munge-invisibility-spec)
   (remove-hook 'erc-insert-modify-hook #'erc-add-timestamp)
   (remove-hook 'erc-send-modify-hook #'erc-add-timestamp)
   (remove-hook 'erc-mode-hook #'erc-stamp--recover-on-reconnect)
   (remove-hook 'erc--pre-clear-functions #'erc-stamp--reset-on-clear)
   (erc-buffer-do #'erc-stamp--setup)))

(defvar erc-stamp--invisible-property nil
  "Existing `invisible' property value and/or symbol `timestamp'.")

(defvar erc-stamp--skip-when-invisible nil
  "Escape hatch for omitting stamps when first char is invisible.")

(defun erc-stamp--recover-on-reconnect ()
  (when-let ((priors (or erc--server-reconnecting erc--target-priors)))
    (dolist (var '(erc-timestamp-last-inserted
                   erc-timestamp-last-inserted-left
                   erc-timestamp-last-inserted-right))
      (when-let (existing (alist-get var priors))
        (set var existing)))))

(defvar erc-stamp--current-time nil
  "The current time when calling `erc-insert-timestamp-function'.
Specifically, this is the same lisp time object used to create
the stamp passed to `erc-insert-timestamp-function'.")

(cl-defgeneric erc-stamp--current-time ()
  "Return a lisp time object to associate with an IRC message.
This becomes the message's `erc--ts' text property."
  (erc-compat--current-lisp-time))

(cl-defmethod erc-stamp--current-time :around ()
  (or erc-stamp--current-time (cl-call-next-method)))

(defvar erc-stamp--skip nil
  "Non-nil means inhibit `erc-add-timestamp' completely.")

(defvar erc-stamp--allow-unmanaged nil
  "Non-nil means run `erc-add-timestamp' almost unconditionally.
This is an unofficial escape hatch for code wanting to use
lower-level message-insertion functions, like `erc-insert-line',
directly.  Third parties needing such functionality should
petition for it via \\[erc-bug].")

(defvar erc-stamp--permanent-cursor-sensor-functions nil
  "Non-nil means add `cursor-sensor-functions' unconditionally.
This is an unofficial escape hatch for code wanting the text
property `cursor-sensor-functions' to always be present,
regardless of the option `erc-echo-timestamps'.  Third parties
needing such pre-5.6 behavior to stick around should make that
known via \\[erc-bug].")

(defun erc-add-timestamp ()
  "Add timestamp and text-properties to message.

This function is meant to be called from `erc-insert-modify-hook'
or `erc-send-modify-hook'."
  (unless (or erc-stamp--skip (and (not erc-stamp--allow-unmanaged)
                                   (null erc--msg-props)))
    (let* ((ct (erc-stamp--current-time))
           (invisible (get-text-property (point-min) 'invisible))
           (erc-stamp--invisible-property
            ;; FIXME on major version bump, make this `erc-' prefixed.
            (if invisible `(timestamp ,@(ensure-list invisible)) 'timestamp))
           (skipp (or (and erc-stamp--skip-when-invisible invisible)
                      (erc--check-msg-prop 'erc--ephemeral)))
           (erc-stamp--current-time ct))
      (when erc--msg-props
        (puthash 'erc--ts ct erc--msg-props))
      (unless skipp
        (funcall erc-insert-timestamp-function
                 (erc-format-timestamp ct erc-timestamp-format)))
      ;; Check `erc-insert-away-timestamp-function' for historical
      ;; reasons even though its Custom :type only allows functions.
      (when (and (not (or skipp erc-timestamp-format))
                 erc-away-timestamp-format
                 (functionp erc-insert-away-timestamp-function)
                 (erc-away-time))
	(funcall erc-insert-away-timestamp-function
		 (erc-format-timestamp ct erc-away-timestamp-format)))
      (when erc-stamp--permanent-cursor-sensor-functions
        (add-text-properties (point-min) (max (point-min) (1- (point-max)))
			   ;; It's important for the function to
			   ;; be different on different entries (bug#22700).
			   (list 'cursor-sensor-functions
                                 ;; Regions are no longer contiguous ^
                                 '(erc--echo-ts-csf) 'erc--ts ct))))))

(defvar-local erc-timestamp-last-window-width nil
  "The width of the last window that showed the current buffer.
his is used by `erc-insert-timestamp-right' when the current
buffer is not shown in any window.")

(defvar-local erc-timestamp-last-inserted nil
  "Last timestamp inserted into the buffer.")

(defvar-local erc-timestamp-last-inserted-left nil
  "Last \"date stamp\" inserted into the left side of the buffer.
Used when `erc-insert-timestamp-function' is set to
`erc-timestamp-left-and-right'.  If the format string specified
by `erc-timestamp-format-left' includes trailing newlines, this
value omits the last one.")

(defvar-local erc-timestamp-last-inserted-right nil
  "Last timestamp inserted into the right side of the buffer.
This is used when `erc-insert-timestamp-function' is set to
`erc-timestamp-left-and-right'")

(defcustom erc-timestamp-only-if-changed-flag t
  "Non-nil means insert timestamp only if its value changed since last insertion.
If `erc-insert-timestamp-function' is `erc-insert-timestamp-left', a
string of spaces which is the same size as the timestamp is added to
the beginning of the line in its place.  If you use
`erc-insert-timestamp-right', nothing gets inserted in place of the
timestamp."
  :type 'boolean)

(defcustom erc-timestamp-right-column nil
  "If non-nil, the column at which the timestamp is inserted,
if the timestamp is to be printed to the right.  If nil,
`erc-insert-timestamp-right' will use other means to determine
the correct column."
  :type '(choice
	  (integer :tag "Column number")
	  (const :tag "Unspecified" nil)))

(defcustom erc-timestamp-use-align-to (and (display-graphic-p) t)
  "If non-nil, use the :align-to display property to align the stamp.
This gives better results when variable-width characters (like
Asian language characters and math symbols) precede a timestamp.

This option only matters when `erc-insert-timestamp-function' is
set to `erc-insert-timestamp-right' or that option's default,
`erc-insert-timestamp-left-and-right'.  If the value is a
positive integer, alignment occurs that many columns from the
right edge.

Enabling this option produces a side effect in that stamps aren't
indented in saved logs.  When its value is an integer, this
option adds a space after the end of a message if the stamp
doesn't already start with one.  And when its value is t, it adds
a single space, unconditionally."
  :type '(choice boolean integer)
  :package-version '(ERC . "5.6"))

(defvar-local erc-stamp--margin-width nil
  "Width in columns of margin for `erc-stamp--display-margin-mode'.
Only consulted when resetting or initializing margin.")

(defvar-local erc-stamp--margin-left-p nil
  "Whether `erc-stamp--display-margin-mode' uses the left margin.
During initialization, the mode respects this variable's existing
value if it already has a local binding.  Otherwise, modules can
bind this to any value while enabling the mode.  If it's nil, ERC
will check to see if `erc-insert-timestamp-function' is
`erc-insert-timestamp-left', interpreting the latter as a non-nil
value.  It'll then coerce any non-nil value to t.")

(defun erc-stamp--init-margins-on-connect (&rest _)
  (let ((existing (if erc-stamp--margin-left-p
                      left-margin-width
                    right-margin-width)))
    (erc-stamp--adjust-margin existing 'resetp)))

(defun erc-stamp--adjust-margin (cols &optional resetp)
  "Adjust managed margin by increment COLS.
With RESETP, set margin's width to COLS.  However, if COLS is
zero, set the width to a non-nil `erc-stamp--margin-width'.
Otherwise, go with the `string-width' of `erc-timestamp-format'.
However, when `erc-stamp--margin-left-p' is non-nil and the
prompt is wider, use its width instead."
  (let* ((leftp erc-stamp--margin-left-p)
         (width
          (if resetp
              (or (and (not (zerop cols)) cols)
                  erc-stamp--margin-width
                  (max (if leftp
                           (cond ((fboundp 'erc-fill--wrap-measure)
                                  (let* ((b erc-insert-marker)
                                         (e (1- erc-input-marker))
                                         (w (erc-fill--wrap-measure b e)))
                                    (/ (if (consp w) (car w) w)
                                       (frame-char-width))))
                                 ((fboundp 'string-pixel-width)
                                  (/ (string-pixel-width (erc-prompt))
                                     (frame-char-width)))
                                 (t (string-width (erc-prompt))))
                         0)
                       (1+ (string-width
                            (or (if leftp
                                    erc-timestamp-last-inserted
                                  erc-timestamp-last-inserted-right)
                                (erc-format-timestamp
                                 (current-time) erc-timestamp-format))))))
            (+ (if leftp left-margin-width right-margin-width) cols))))
    (set (if leftp 'left-margin-width 'right-margin-width) width)
    (when (eq (current-buffer) (window-buffer))
      (set-window-margins nil
                          (if leftp width left-margin-width)
                          (if leftp right-margin-width width)))))

;;;###autoload
(defun erc-stamp-prefix-log-filter (text)
  "Prefix every message in the buffer with a stamp.
Remove trailing stamps as well.  For now, hard code the format to
\"ZNC\"-log style, which is [HH:MM:SS].  Expect to be used as a
`erc-log-filter-function' when `erc-timestamp-use-align-to' is
non-nil."
  (insert text)
  (goto-char (point-min))
  (while
      (progn
        (when-let (((< (point) (pos-eol)))
                   (end (1- (pos-eol)))
                   ((eq 'erc-timestamp (field-at-pos end)))
                   (beg (field-beginning end))
                   ;; Skip a line that's just a timestamp.
                   ((> beg (point))))
          (delete-region beg (1+ end)))
        (when-let (time (erc--get-inserted-msg-prop 'erc--ts))
          (insert (format-time-string "[%H:%M:%S] " time)))
        (zerop (forward-line))))
  "")

;; These are currently extended manually, but we could also bind
;; `text-property-default-nonsticky' and call `insert-and-inherit'
;; instead of `insert', but we'd have to pair the props with differing
;; boolean values for left and right stamps.  Also, since this hook
;; runs last, we can't expect overriding sticky props to be absent,
;; even though, as of 5.6, `front-sticky' is only added by the
;; `readonly' module after hooks run.
(defvar erc-stamp--inherited-props '(line-prefix wrap-prefix)
  "Extant properties at the start of a message inherited by the stamp.")

(defvar-local erc-stamp--skip-left-margin-prompt-p nil
  "Don't display prompt in left margin.")

(declare-function erc--remove-text-properties "erc" (string))

;; Currently, `erc-insert-timestamp-right' hard codes its display
;; property to use `right-margin', and `erc-insert-timestamp-left'
;; does the same for `left-margin'.  However, there's no reason a
;; trailing stamp couldn't be displayed on the left and vice versa.
(define-minor-mode erc-stamp--display-margin-mode
  "Internal minor mode for built-in modules integrating with `stamp'.
Arranges for displaying stamps in a single margin, with the
variable `erc-stamp--margin-left-p' controlling which one.
Provides `erc-stamp--margin-width' and `erc-stamp--adjust-margin'
to help manage the chosen margin's width.  Also removes `display'
properties in killed text to reveal stamps.  The invoking module
should set controlling variables, like `erc-stamp--margin-width'
and `erc-stamp--margin-left-p', before activating the mode."
  :interactive nil
  (if erc-stamp--display-margin-mode
      (progn
        (setq fringes-outside-margins t)
        (when (eq (current-buffer) (window-buffer))
          (set-window-buffer (selected-window) (current-buffer)))
        (setq erc-stamp--margin-left-p (and erc-stamp--margin-left-p t))
        (if (or erc-server-connected (not (functionp erc-prompt)))
            (erc-stamp--init-margins-on-connect)
          (add-hook 'erc-after-connect
                    #'erc-stamp--init-margins-on-connect nil t))
        (add-function :filter-return (local 'filter-buffer-substring-function)
                      #'erc--remove-text-properties)
        (add-hook 'erc--setup-buffer-hook
                  #'erc-stamp--refresh-left-margin-prompt nil t)
        (when (and erc-stamp--margin-left-p
                   (not erc-stamp--skip-left-margin-prompt-p))
          (add-hook 'erc--refresh-prompt-hook
                    #'erc-stamp--display-prompt-in-left-margin nil t)))
    (remove-function (local 'filter-buffer-substring-function)
                     #'erc--remove-text-properties)
    (remove-hook 'erc-after-connect
                 #'erc-stamp--init-margins-on-connect t)
    (remove-hook 'erc--refresh-prompt-hook
                 #'erc-stamp--display-prompt-in-left-margin t)
    (remove-hook 'erc--setup-buffer-hook
                 #'erc-stamp--refresh-left-margin-prompt t)
    (kill-local-variable (if erc-stamp--margin-left-p
                             'left-margin-width
                           'right-margin-width))
    (kill-local-variable 'erc-stamp--skip-left-margin-prompt-p)
    (kill-local-variable 'fringes-outside-margins)
    (kill-local-variable 'erc-stamp--margin-left-p)
    (kill-local-variable 'erc-stamp--margin-width)
    (when (eq (current-buffer) (window-buffer))
      (set-window-margins nil left-margin-width nil)
      (set-window-buffer (selected-window) (current-buffer)))))

(defvar-local erc-stamp--last-prompt nil)

(defun erc-stamp--display-prompt-in-left-margin ()
  "Show prompt in the left margin with padding."
  (when (or (not erc-stamp--last-prompt) (functionp erc-prompt)
            (> (string-width erc-stamp--last-prompt) left-margin-width))
    (let ((s (buffer-substring erc-insert-marker (1- erc-input-marker))))
      ;; Prevent #("abc" n m (display ((...) #("abc" p q (display...))))
      (remove-text-properties 0 (length s) '(display nil) s)
      (when (and erc-stamp--last-prompt
                 (>= (string-width erc-stamp--last-prompt) left-margin-width))
        (let ((sm (truncate-string-to-width s (1- left-margin-width) 0 nil t)))
          ;; This papers over a subtle off-by-1 bug here.
          (unless (equal sm s)
            (setq s (concat sm (substring s -1))))))
      (setq erc-stamp--last-prompt (string-pad s left-margin-width nil t))))
  (put-text-property erc-insert-marker (1- erc-input-marker)
                     'display `((margin left-margin) ,erc-stamp--last-prompt))
  erc-stamp--last-prompt)

(defun erc-stamp--refresh-left-margin-prompt ()
  "Forcefully-recompute display property of prompt in left margin."
  (with-silent-modifications
    (unless (functionp erc-prompt)
      (setq erc-stamp--last-prompt nil))
    (erc--refresh-prompt)))

(cl-defmethod erc--conceal-prompt
  (&context (erc-stamp--display-margin-mode (eql t))
            (erc-stamp--margin-left-p (eql t))
            (erc-stamp--skip-left-margin-prompt-p null))
  (when-let (((null erc--hidden-prompt-overlay))
             (prompt (string-pad erc-prompt-hidden left-margin-width nil 'start))
             (ov (make-overlay erc-insert-marker (1- erc-input-marker)
                               nil 'front-advance)))
    (overlay-put ov 'display `((margin left-margin) ,prompt))
    (setq erc--hidden-prompt-overlay ov)))

(defun erc-insert-timestamp-left (string)
  "Insert timestamps at the beginning of the line."
  (erc--insert-timestamp-left string))

(cl-defmethod erc--insert-timestamp-left (string)
  (goto-char (point-min))
  (let* ((ignore-p (and erc-timestamp-only-if-changed-flag
			(string-equal string erc-timestamp-last-inserted)))
	 (len (length string))
	 (s (if ignore-p (make-string len ? ) string)))
    (unless ignore-p (setq erc-timestamp-last-inserted string))
    (erc-put-text-property 0 len 'field 'erc-timestamp s)
    (erc-put-text-property 0 len 'invisible erc-stamp--invisible-property s)
    (insert s)))

(cl-defmethod erc--insert-timestamp-left
  (string &context (erc-stamp--display-margin-mode (eql t)))
  (unless (and erc-timestamp-only-if-changed-flag
               (string-equal string erc-timestamp-last-inserted))
    (goto-char (point-min))
    (insert-and-inherit (setq erc-timestamp-last-inserted string))
    (dolist (p erc-stamp--inherited-props)
      (when-let ((v (get-text-property (point) p)))
        (put-text-property (point-min) (point) p v)))
    (erc-put-text-property (point-min) (point) 'invisible
                           erc-stamp--invisible-property)
    (put-text-property (point-min) (point) 'field 'erc-timestamp)
    (put-text-property (point-min) (point)
                       'display `((margin left-margin) ,string))))

(defun erc-insert-aligned (string pos)
  "Insert STRING at the POSth column.

If `erc-timestamp-use-align-to' is t, use the :align-to display
property to get to the POSth column."
  (declare (obsolete "inlined and removed from client code path" "30.1"))
  (if (not erc-timestamp-use-align-to)
      (indent-to pos)
    (insert " ")
    (put-text-property (1- (point)) (point) 'display
		       (list 'space ':align-to pos)))
  (insert string))

;; Silence byte-compiler
(defvar erc-fill-column)

(defvar erc-stamp--omit-properties-on-folded-lines nil
  "Skip properties before right stamps occupying their own line.
This escape hatch restores pre-5.6 behavior that left leading
white space alone (unpropertized) for right-sided stamps folded
onto their own line.")

(defun erc-insert-timestamp-right (string)
  "Insert timestamp on the right side of the screen.
STRING is the timestamp to insert.  This function is a possible
value for `erc-insert-timestamp-function'.

If `erc-timestamp-only-if-changed-flag' is nil, a timestamp is
always printed.  If this variable is non-nil, a timestamp is only
printed if it is different from the last.

If `erc-timestamp-right-column' is set, its value will be used as
the column at which the timestamp is to be printed.  If it is
nil, and `erc-fill-mode' is active, then the timestamp will be
printed just before `erc-fill-column'.  Otherwise, if the current
buffer is shown in a window, that window's width is used as the
right boundary.  In case multiple windows show the buffer, the
width of the most recently selected one is used.  If the buffer
is not shown, the timestamp will be printed just before the
window width of the last window that showed it.  If the buffer
was never shown, and `fill-column' is set, it will be printed
just before `fill-column'.  As a last resort, timestamp will be
printed just after each line's text (no alignment)."
  (unless (and erc-timestamp-only-if-changed-flag
	       (string-equal string erc-timestamp-last-inserted))
    (setq erc-timestamp-last-inserted string)
    (goto-char (point-max))
    (forward-char -1)                   ; before the last newline
    (let* ((str-width (string-width string))
           (buffer-invisibility-spec nil) ; `current-column' > 0
           window                  ; used in computation of `pos' only
	   (pos (cond
		 (erc-timestamp-right-column erc-timestamp-right-column)
		 ((and (boundp 'erc-fill-mode)
		       erc-fill-mode
		       (boundp 'erc-fill-column)
		       erc-fill-column)
		  (1+ (- erc-fill-column str-width)))
                 ((setq window (get-buffer-window nil t))
                  (setq erc-timestamp-last-window-width
                        (window-width window))
                  (- erc-timestamp-last-window-width str-width))
                 (erc-timestamp-last-window-width
                  (- erc-timestamp-last-window-width str-width))
		 (fill-column
		  (1+ (- fill-column str-width)))
                 (t (current-column))))
	   (from (point))
	   (col (current-column)))
      ;; The following is a kludge used to calculate whether to move
      ;; to the next line before inserting a stamp.  It allows for
      ;; some margin of error if what is displayed on the line differs
      ;; from the number of characters on the line.
      (setq col (+ col (ceiling (/ (- col (- (point) (line-beginning-position))) 1.6))))
      ;; For compatibility reasons, the `erc-timestamp' field includes
      ;; intervening white space unless a hard break is warranted.
      (pcase erc-timestamp-use-align-to
        ((guard erc-stamp--display-margin-mode)
         (let ((s (propertize (substring-no-properties string)
                              'invisible erc-stamp--invisible-property)))
           (put-text-property 0 (length string) 'display
                              `((margin right-margin) ,s)
                              string)))
        ((and 't (guard (< col pos)))
         (insert " ")
         (put-text-property from (point) 'display `(space :align-to ,pos)))
        ((pred integerp) ; (cl-type (integer 0 *))
         (insert " ")
         (when (eq ?\s (aref string 0))
           (setq string (substring string 1)))
         (let ((s (+ erc-timestamp-use-align-to (string-width string))))
           (put-text-property from (point) 'display
                              `(space :align-to (- right ,s)))))
        ((guard (>= col pos)) (newline) (indent-to pos)
         (when erc-stamp--omit-properties-on-folded-lines (setq from (point))))
        (_ (indent-to pos)))
      (insert string)
      (dolist (p erc-stamp--inherited-props)
        (when-let ((v (get-text-property (1- from) p)))
          (put-text-property from (point) p v)))
      (erc-put-text-property from (point) 'field 'erc-timestamp)
      (erc-put-text-property from (point) 'rear-nonsticky t)
      (erc-put-text-property from (point) 'invisible
                             erc-stamp--invisible-property)
      (when erc-timestamp-intangible
	(erc-put-text-property from (1+ (point)) 'cursor-intangible t)))))

(defvar erc-stamp--insert-date-hook nil
  "Functions appended to send and modify hooks when inserting date stamp.")

(defvar-local erc-stamp--date-format-end nil
  "Tristate value indicating how and whether date stamps have been set up.
A non-nil value means the buffer has been initialized to use date
stamps.  An integer marks the `substring' TO parameter for
truncating `erc-timestamp-format-left' prior to rendering.  A
value of t means the option's value doesn't require trimming.")

(defun erc-stamp--propertize-left-date-stamp ()
  (add-text-properties (point-min) (1- (point-max)) '(field erc-timestamp))
  (erc--hide-message 'timestamp)
  (run-hooks 'erc-stamp--insert-date-hook))

(defun erc-stamp--format-date-stamp (ct)
  "Format left date stamp with `erc-timestamp-format-left'."
  (unless erc-stamp--date-format-end
    ;; Don't add text properties to the trailing newline.
    (setq erc-stamp--date-format-end
          (if (string-suffix-p "\n" erc-timestamp-format-left) -1 t)))
  ;; Ignore existing `invisible' prop value because date stamps should
  ;; never be hideable except via `timestamp'.
  (let (erc-stamp--invisible-property)
    (erc-format-timestamp ct (if (numberp erc-stamp--date-format-end)
                                 (substring erc-timestamp-format-left
                                            0 erc-stamp--date-format-end)
                               erc-timestamp-format-left))))

(defun erc-stamp-inserting-date-stamp-p ()
  "Return non-nil if the narrowed buffer contains a date stamp.
Expect to be called by members of `erc-insert-modify-hook' and
`erc-insert-post-hook' to detect whether the message being
inserted is a date stamp."
  (erc--check-msg-prop 'erc--msg 'datestamp))

;; Calling `erc-display-message' from within a hook it's currently
;; running is roundabout, but it's a definite means of ensuring hooks
;; can act on the date stamp as a standalone message to do things like
;; adjust invisibility props.
(defun erc-stamp--insert-date-stamp-as-phony-message (string)
  (cl-assert (string-empty-p string))
  (setq string erc-timestamp-last-inserted-left)
  (let ((erc-stamp--skip t)
        (erc-insert-modify-hook `(,@erc-insert-modify-hook
                                  erc-stamp--propertize-left-date-stamp))
        (erc--insert-line-function #'insert-before-markers)
        ;; Don't run hooks that aren't expecting a narrowed buffer.
        (erc-insert-pre-hook nil)
        (erc-insert-done-hook nil))
    (erc-display-message nil nil (current-buffer) string)))

(defun erc-stamp--lr-date-on-pre-modify (_)
  (when-let (((not erc-stamp--skip))
             (ct (erc-stamp--current-time))
             (rendered (erc-stamp--format-date-stamp ct))
             ((not (string-equal rendered erc-timestamp-last-inserted-left)))
             (erc-insert-timestamp-function
              #'erc-stamp--insert-date-stamp-as-phony-message))
    (save-excursion
      (save-restriction
        (narrow-to-region (or erc--insert-marker erc-insert-marker)
                          (or erc--insert-marker erc-insert-marker))
        ;; Ensure all hooks, like `erc-stamp--insert-date-hook', only
        ;; see the let-bound value below during `erc-add-timestamp'.
        (setq erc-timestamp-last-inserted-left nil)
        (let* ((aligned (erc-stamp--time-as-day ct))
               (erc-stamp--current-time aligned)
               ;; Forget current `erc--cmd', etc.
               (erc--msg-props (map-into `((erc--msg . datestamp))
                                         'hash-table))
               (erc-timestamp-last-inserted-left rendered)
               erc-timestamp-format erc-away-timestamp-format)
          ;; FIXME delete once convinced adjustment correct.
          (cl-assert (string= rendered
                              (erc-stamp--format-date-stamp aligned)))
          (erc-add-timestamp))
        (setq erc-timestamp-last-inserted-left rendered)))))

;; This minor mode is just a placeholder and currently unhelpful for
;; managing complexity.  A useful version would leave a marker during
;; post-modify hooks and then perform insertions (before markers)
;; during "done" hooks.  This would enable completely decoupling from
;; and possibly deprecating `erc-insert-timestamp-left-and-right'.
;; However, doing this would require expanding the internal API to
;; include insertion and deletion handlers for twiddling and massaging
;; text properties based on context immediately after modifying text
;; earlier in a buffer (away from `erc-insert-marker').  Without such
;; handlers, things like "merged" `fill-wrap' speakers and invisible
;; messages may be damaged by buffer modifications.
(define-minor-mode erc-stamp--date-mode
  "Insert date stamps as standalone messages."
  :interactive nil
  (if erc-stamp--date-mode
      (progn (add-hook 'erc-insert-pre-hook
                       #'erc-stamp--lr-date-on-pre-modify 10 t)
             (add-hook 'erc-pre-send-functions
                       #'erc-stamp--lr-date-on-pre-modify 10 t))
    (kill-local-variable 'erc-timestamp-last-inserted-left)
    (remove-hook 'erc-insert-pre-hook
                 #'erc-stamp--lr-date-on-pre-modify t)
    (remove-hook 'erc-pre-send-functions
                 #'erc-stamp--lr-date-on-pre-modify t)))

(defvar erc-stamp-prepend-date-stamps-p nil
  "When non-nil, date stamps are not independent messages.
This flag restores pre-5.6 behavior in which date stamps formed
the leading portion of affected messages.  Beware that enabling
this degrades the user experience by causing 5.6+ features, like
`fill-wrap', dynamic invisibility, etc., to malfunction.  When
non-nil, none of the newline twiddling mentioned in the doc
string for `erc-timestamp-format-left' occurs.  That is, ERC does
not append or remove trailing newlines.")
(make-obsolete-variable 'erc-stamp-prepend-date-stamps-p
                        "unsupported legacy behavior" "30.1")

(defun erc-insert-timestamp-left-and-right (string)
  "Insert a stamp on either side when it changes.
When the deprecated option `erc-timestamp-format-right' is nil,
use STRING, which originates from `erc-timestamp-format', for the
right-hand stamp.  Use `erc-timestamp-format-left' for formatting
the left-sided \"date stamp,\" and expect it to change less
frequently.  Include all but the final trailing newline present
in the latter (if any) as part of the `erc-timestamp' field.
Allow the stamp's `invisible' property to span that same interval
but also cover the previous newline, in order to satisfy folding
requirements related to `erc-legacy-invisible-bounds-p'.
Additionally, ensure every date stamp is identifiable as such so
that internal modules can easily distinguish between other
left-sided stamps and date stamps inserted by this function."
  (unless (or erc-stamp--date-format-end erc-stamp-prepend-date-stamps-p
              (and (or (null erc-timestamp-format-left)
                       (string-empty-p ; compat
                        (string-trim erc-timestamp-format-left "\n")))
                   (always (erc-stamp--date-mode -1))
                   (setq erc-stamp-prepend-date-stamps-p t)))
    (erc-stamp--date-mode +1)
    ;; Hooks used by ^ are the preferred means of inserting date
    ;; stamps.  But they'll never see this inaugural message, so it
    ;; must be handled specially.
    (let ((erc--insert-marker (point-min-marker))
          (end-marker (point-max-marker)))
      (set-marker-insertion-type erc--insert-marker t)
      (erc-stamp--lr-date-on-pre-modify nil)
      (narrow-to-region erc--insert-marker end-marker)
      (set-marker end-marker nil)
      (set-marker erc--insert-marker nil)))
  (let* ((ct (erc-stamp--current-time))
         (ts-right (with-suppressed-warnings
                       ((obsolete erc-timestamp-format-right))
                     (if erc-timestamp-format-right
                         (erc-format-timestamp ct erc-timestamp-format-right)
                       string))))
    ;; We should arguably be ensuring a trailing newline on legacy
    ;; "prepended" date stamps as well.  However, since this is a
    ;; compatibility oriented code path, and pre-5.6 did no such
    ;; thing, better to punt.
    (when-let ((erc-stamp-prepend-date-stamps-p)
               (ts-left (erc-format-timestamp ct erc-timestamp-format-left))
               ((not (string= ts-left erc-timestamp-last-inserted-left))))
      (goto-char (point-min))
      (erc-put-text-property 0 (length ts-left) 'field 'erc-timestamp ts-left)
      (insert (setq erc-timestamp-last-inserted-left ts-left)))
    ;; insert right timestamp
    (let ((erc-timestamp-only-if-changed-flag t)
	  (erc-timestamp-last-inserted erc-timestamp-last-inserted-right))
      (erc-insert-timestamp-right ts-right)
      (setq erc-timestamp-last-inserted-right ts-right))))

;; for testing: (setq erc-timestamp-only-if-changed-flag nil)
(defvar erc-stamp--tz nil)

;; Unfortunately, cursory measurements show that this function is 10x
;; slower than `erc-format-timestamp', which is perhaps
;; counterintuitive.  Thus, we use the latter for our cache, and
;; perform day alignments via this function only when needed.
(defun erc-stamp--time-as-day (current-time)
  "Discard hour, minute, and second info from timestamp CURRENT-TIME."
  (let* ((current-time-list) ; flag
         (decoded (decode-time current-time erc-stamp--tz)))
    (setf (decoded-time-second decoded) 0
          (decoded-time-minute decoded) 0
          (decoded-time-hour decoded) 0)
    (encode-time decoded))) ; may return an integer

(defun erc-format-timestamp (time format)
  "Return TIME formatted as string according to FORMAT.
Return the empty string if FORMAT is nil."
  (if format
      (let ((ts (format-time-string format time erc-stamp--tz)))
	(erc-put-text-property 0 (length ts)
			       'font-lock-face 'erc-timestamp-face ts)
        (when erc-stamp--invisible-property
          (erc-put-text-property 0 (length ts) 'invisible
                                 erc-stamp--invisible-property ts))
	;; N.B. Later use categories instead of this harmless, but
	;; inelegant, hack. -- BPT
	(and erc-timestamp-intangible
	     (not erc-hide-timestamps)	; bug#11706
	     (erc-put-text-property 0 (length ts) 'cursor-intangible t ts))
	ts)
    ""))

(defvar-local erc-stamp--csf-props-updated-p nil)

;; This function is used to munge `buffer-invisibility-spec' to an
;; appropriate value. Currently, it only handles timestamps, thus its
;; location.  If you add other features which affect invisibility,
;; please modify this function and move it to a more appropriate
;; location.
(defun erc-munge-invisibility-spec ()
  (if erc-timestamp-intangible
      (cursor-intangible-mode +1) ; idempotent
    (when (bound-and-true-p cursor-intangible-mode)
      (cursor-intangible-mode -1)))
  (if erc-echo-timestamps
      (progn
        (unless erc-stamp--permanent-cursor-sensor-functions
          (dolist (hook '(erc-insert-post-hook erc-send-post-hook))
            (add-hook hook #'erc-stamp--add-csf-on-post-modify nil t))
          (erc--restore-initialize-priors erc-stamp-mode
            erc-stamp--csf-props-updated-p nil)
          (unless erc-stamp--csf-props-updated-p
            (setq erc-stamp--csf-props-updated-p t)
            (let ((erc--msg-props (map-into '((erc--ts . t)) 'hash-table)))
              (with-silent-modifications
                (erc--traverse-inserted
                 (point-min) erc-insert-marker
                 #'erc-stamp--add-csf-on-post-modify)))))
        (cursor-sensor-mode +1) ; idempotent
        (when (>= emacs-major-version 29)
          (add-function :before-until (local 'clear-message-function)
                        #'erc-stamp--on-clear-message)))
    (dolist (hook '(erc-insert-post-hook erc-send-post-hook))
      (remove-hook hook #'erc-stamp--add-csf-on-post-modify t))
    (kill-local-variable 'erc-stamp--csf-props-updated-p)
    (when (bound-and-true-p cursor-sensor-mode)
      (cursor-sensor-mode -1))
    (remove-function (local 'clear-message-function)
                     #'erc-stamp--on-clear-message))
  (if erc-hide-timestamps
      (add-to-invisibility-spec 'timestamp)
    (remove-from-invisibility-spec 'timestamp)))

(defun erc-stamp--add-csf-on-post-modify ()
  "Add `cursor-sensor-functions' to narrowed buffer."
  (when (erc--check-msg-prop 'erc--ts)
    (put-text-property (point-min) (1- (point-max))
                       'cursor-sensor-functions '(erc--echo-ts-csf))))

(defun erc-stamp--setup ()
  "Enable or disable buffer-local `erc-stamp-mode' modifications."
  (if erc-stamp-mode
      (erc-munge-invisibility-spec)
    (let (erc-echo-timestamps erc-hide-timestamps erc-timestamp-intangible)
      (erc-munge-invisibility-spec))
    ;; Undo local mods from `erc-insert-timestamp-left-and-right'.
    (erc-stamp--date-mode -1) ; kills `erc-timestamp-last-inserted-left'
    (kill-local-variable 'erc-stamp--last-stamp)
    (kill-local-variable 'erc-timestamp-last-inserted)
    (kill-local-variable 'erc-timestamp-last-inserted-right)
    (kill-local-variable 'erc-stamp--date-format-end)))

(defun erc-hide-timestamps ()
  "Hide timestamp information from display."
  (interactive)
  (setq erc-hide-timestamps t)
  (erc-munge-invisibility-spec))

(defun erc-show-timestamps ()
  "Show timestamp information on display.
This function only works if `erc-timestamp-format' was previously
set, and timestamping is already active."
  (interactive)
  (setq erc-hide-timestamps nil)
  (erc-munge-invisibility-spec))

(defun erc-toggle-timestamps ()
  "Hide or show timestamps in ERC buffers.

Note that timestamps can only be shown for a message using this
function if `erc-timestamp-format' was set and timestamping was
enabled when the message was inserted."
  (interactive)
  (if erc-hide-timestamps
      (setq erc-hide-timestamps nil)
    (setq erc-hide-timestamps t))
  (mapc (lambda (buffer)
	  (with-current-buffer buffer
	    (erc-munge-invisibility-spec)))
	(erc-buffer-list)))

(defvar-local erc-stamp--last-stamp nil)

(defun erc-stamp--on-clear-message (&rest _)
  "Return `dont-clear-message' when operating inside the same stamp."
  (and erc-stamp--last-stamp erc-echo-timestamps
       (eq (erc--get-inserted-msg-prop 'erc--ts) erc-stamp--last-stamp)
       'dont-clear-message))

(defun erc-echo-timestamp (dir stamp &optional zone)
  "Display timestamp of message at point in echo area.
Interactively, interpret a numeric prefix as a ZONE offset in
hours (or seconds, if its abs value is larger than 14), and
interpret a \"raw\" prefix as UTC.  To specify a zone for use
with the option `erc-echo-timestamps', see the companion option
`erc-echo-timestamp-zone'."
  (interactive (list nil (erc--get-inserted-msg-prop 'erc--ts)
                     (pcase current-prefix-arg
                       ((and (pred numberp) v)
                        (if (<= (abs v) 14) (* v 3600) v))
                       (`(,_) t))))
  (if (and stamp (or (null dir) (and erc-echo-timestamps (eq 'entered dir))))
      (progn
        (setq erc-stamp--last-stamp stamp)
        (message (format-time-string erc-echo-timestamp-format
                                     stamp (or zone erc-echo-timestamp-zone))))
    (when (and erc-echo-timestamps (eq 'left dir))
      (setq erc-stamp--last-stamp nil))))

(defun erc--echo-ts-csf (_window _before dir)
  (erc-echo-timestamp dir (erc--get-inserted-msg-prop 'erc--ts)))

(defun erc-stamp--update-saved-position (&rest _)
  (remove-hook 'erc-stamp--insert-date-hook
               #'erc-stamp--update-saved-position t)
  (move-marker erc-last-saved-position (1- (point-max))))

(defun erc-stamp--reset-on-clear (pos)
  "Forget last-inserted stamps when POS is at insert marker."
  (when (= pos (1- erc-insert-marker))
    (when erc-stamp--date-mode
      (add-hook 'erc-stamp--insert-date-hook
                #'erc-stamp--update-saved-position 0 t))
    (setq erc-timestamp-last-inserted nil
          erc-timestamp-last-inserted-left nil
          erc-timestamp-last-inserted-right nil)))

(provide 'erc-stamp)

;;; erc-stamp.el ends here
;;
;; Local Variables:
;; generated-autoload-file: "erc-loaddefs.el"
;; End:
