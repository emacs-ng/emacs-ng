;;; erc-button.el --- A way of buttonizing certain things in ERC buffers  -*- lexical-binding:t -*-

;; Copyright (C) 1996-2004, 2006-2023 Free Software Foundation, Inc.

;; Author: Mario Lang <mlang@delysid.org>
;; Maintainer: Amin Bandali <bandali@gnu.org>, F. Jason Park <jp@neverwas.me>
;; Keywords: comm, irc, button, url, regexp
;; URL: https://www.emacswiki.org/emacs/ErcButton

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

;; Heavily borrowed from gnus-art.el.  Thanks to the original authors.
;; This buttonizes nicks and other stuff to make it all clickable.
;; To enable, add to your init file:
;; (require 'erc-button)
;; (erc-button-mode 1)
;;
;; Todo:
;; * Rewrite all this to do the same, but use button.el.  Why?
;; button.el is much faster, and much more elegant, and solves the
;; problem we get with large buffers and a large erc-button-marker-list.


;;; Code:

(require 'erc)
(require 'wid-edit)
(require 'erc-fill)
(require 'browse-url)

;;; Minor Mode

(defgroup erc-button nil
  "Define how text can be turned into clickable buttons."
  :group 'erc)

;;;###autoload(autoload 'erc-button-mode "erc-button" nil t)
(define-erc-module button nil
  "This mode buttonizes all messages according to `erc-button-alist'."
  ((erc-button--check-nicknames-entry)
   (add-hook 'erc-insert-modify-hook #'erc-button-add-buttons 'append)
   (add-hook 'erc-send-modify-hook #'erc-button-add-buttons 'append)
   (add-hook 'erc--tab-functions #'erc-button-next)
   (erc--modify-local-map t "<backtab>" #'erc-button-previous))
  ((remove-hook 'erc-insert-modify-hook #'erc-button-add-buttons)
   (remove-hook 'erc-send-modify-hook #'erc-button-add-buttons)
   (remove-hook 'erc--tab-functions #'erc-button-next)
   (erc--modify-local-map nil "<backtab>" #'erc-button-previous)))

;;; Variables

(defface erc-button '((t :weight bold))
  "ERC button face."
  :group 'erc-faces)

(defcustom erc-button-face 'erc-button
  "Face used for highlighting buttons in ERC buffers.

A button is a piece of text that you can activate by pressing
\\`RET' or `mouse-2' above it.  See also `erc-button-keymap'."
  :type 'face
  :group 'erc-faces)

(defcustom erc-button-nickname-face 'erc-nick-default-face
  "Face used for ERC nickname buttons."
  :type 'face
  :group 'erc-faces)

(defcustom erc-button-mouse-face 'highlight
  "Face used for mouse highlighting in ERC buffers.

Buttons will be displayed in this face when the mouse cursor is
above them."
  :type 'face
  :group 'erc-faces)

(defcustom erc-button-url-regexp browse-url-button-regexp
  "Regular expression that matches URLs."
  :version "27.1"
  :type 'regexp)

(defcustom erc-button-wrap-long-urls nil
  "If non-nil, \"long\" URLs matching `erc-button-url-regexp' will be wrapped.

If this variable is a number, consider URLs longer than its value to
be \"long\".  If t, URLs will be considered \"long\" if they are
longer than `erc-fill-column'."
  :type '(choice integer boolean))

(defcustom erc-button-buttonize-nicks t
  "Flag indicating whether nicks should be buttonized or not."
  :type 'boolean)

(defcustom erc-button-rfc-url "https://tools.ietf.org/html/rfc%s"
  "URL used to browse RFC references.
%s is replaced by the number."
  :type 'string
  :version "28.1")

(define-obsolete-variable-alias 'erc-button-google-url
  'erc-button-search-url "27.1")

(defcustom erc-button-search-url "https://duckduckgo.com/?q=%s"
  "URL used to search for a term.
%s is replaced by the search string."
  :version "28.1"
  :type 'string)

(defcustom erc-button-alist
  ;; Since the callback is only executed when the user is clicking on
  ;; a button, it makes no sense to optimize performance by
  ;; bytecompiling lambdas in this alist.  On the other hand, it makes
  ;; things hard to maintain.
  '((nicknames 0 erc-button-buttonize-nicks erc-nick-popup 0)
    (erc-button-url-regexp 0 t browse-url-button-open-url 0)
;;; ("(\\(\\([^~\n \t@][^\n \t@]*\\)@\\([a-zA-Z0-9.:-]+\\)\\)" 1 t finger 2 3)
    ;; emacs internal
    ("[`‘]\\([a-zA-Z][-a-zA-Z_0-9!*<=>+]+\\)['’]"
     1 t erc-button-describe-symbol 1)
    ;; pseudo links
    ("\\(?:\\bInfo: ?\\|(info \\)[\"]\\(([^\"]+\\)[\"])?" 0 t info 1)
    ("\\b\\(Ward\\|Wiki\\|WardsWiki\\|TheWiki\\):\\([A-Z][a-z]+\\([A-Z][a-z]+\\)+\\)"
     0 t (lambda (page)
           (browse-url (concat "http://c2.com/cgi-bin/wiki?" page)))
     2)
    ("EmacsWiki:\\([A-Z][a-z]+\\([A-Z][a-z]+\\)+\\)" 0 t erc-browse-emacswiki 1)
    ("Lisp:\\([a-zA-Z.+-]+\\)" 0 t erc-browse-emacswiki-lisp 1)
    ("\\bGoogle:\\([^ \t\n\r\f]+\\)"
     0 t (lambda (keywords)
           (browse-url (format erc-button-search-url keywords)))
     1)
    ("\\brfc[#: ]?\\([0-9]+\\)"
     0 t (lambda (num)
           (browse-url (format erc-button-rfc-url num)))
     1)
    ;; other
    ("\\s-\\(@\\([0-9][0-9][0-9]\\)\\)" 1 t erc-button-beats-to-time 2))
  "Alist of regexps matching buttons in ERC buffers.
Each entry has the form (REGEXP BUTTON FORM CALLBACK PAR...), where

REGEXP is the string matching text around the button or a symbol
  indicating a variable holding that string, or a list of
  strings, or an alist with the strings in the car.  Note that
  entries in lists or alists are considered to be nicks or other
  complete words.  Therefore they are enclosed in \\< and \\>
  while searching.  REGEXP can also be the symbol
  `nicknames', which matches the nickname of any user on the
  current server.

BUTTON is the number of the regexp grouping actually matching the
  button.  This is ignored if REGEXP is `nicknames'.

FORM is either a boolean or a special variable whose value must
  be non-nil for the button to be added.  When REGEXP is the
  special symbol `nicknames', FORM must be the symbol
  `erc-button-buttonize-nicks'.  Anything else is deprecated.
  For all other entries, FORM can also be a function to call in
  place of `erc-button-add-button' with the exact same arguments.
  When FORM is also a special variable, ERC disregards the
  variable and calls the function.

CALLBACK is the function to call when the user push this button.
  CALLBACK can also be a symbol.  Its variable value will be used
  as the callback function.

PAR is a number of a regexp grouping whose text will be passed to
  CALLBACK.  There can be several PAR arguments.  If REGEXP is
  `nicknames', these are ignored, and CALLBACK will be called with
  the nickname matched as the argument."
  :package-version '(ERC . "5.6") ; FIXME sync on release
  :type '(repeat
          (list :tag "Button"
                (choice :tag "Matches"
                        regexp
                        (variable :tag "Variable containing regexp")
                        (const :tag "Nicknames" nicknames))
                (integer :tag "Number of the regexp section that matches")
                (choice :tag "When to buttonize"
                        (const :tag "Always" t)
                        (sexp :tag "Only when this evaluates to non-nil"))
                (function :tag "Function to call when button is pressed")
                (repeat :tag "Sections of regexp to send to the function"
                        :inline t
                        (integer :tag "Regexp section number")))))

(defcustom erc-emacswiki-url "https://www.emacswiki.org/emacs/"
  "URL of the EmacsWiki website."
  :type 'string
  :version "28.1")

(defcustom erc-emacswiki-lisp-url "https://www.emacswiki.org/elisp/"
  "URL of the EmacsWiki ELisp area."
  :type 'string)

(defvar erc-button-keymap
  (let ((map (make-sparse-keymap)))
    (define-key map (kbd "RET") #'erc-button-press-button)
    (define-key map (kbd "<mouse-2>") #'erc-button-click-button)
    (define-key map (kbd "TAB") #'erc-button-next)
    (define-key map (kbd "<backtab>") #'erc-button-previous)
    (define-key map [follow-link] 'mouse-face)
    (set-keymap-parent map erc-mode-map)
    map)
  "Local keymap for ERC buttons.")

(defvar erc-button-syntax-table
  (let ((table (make-syntax-table)))
    (modify-syntax-entry ?\[ "w" table)
    (modify-syntax-entry ?\] "w" table)
    (modify-syntax-entry ?\{ "w" table)
    (modify-syntax-entry ?\} "w" table)
    (modify-syntax-entry ?` "w" table)
    (modify-syntax-entry ?^ "w" table)
    (modify-syntax-entry ?- "w" table)
    (modify-syntax-entry ?_ "w" table)
    (modify-syntax-entry ?| "w" table)
    (modify-syntax-entry ?\\ "w" table)
    table)
  "Syntax table used when buttonizing messages.
This syntax table should make all the valid nick characters word
constituents.")

(defvar erc-button-keys-added nil
  "Internal variable used to keep track of whether we've added the
global-level ERC button keys yet.")

;; Maybe deprecate this function and `erc-button-keys-added' if they
;; continue to go unused for a another version (currently 5.6).
(defun erc-button-setup ()
  "Add ERC mode-level button movement keys.  This is only done once."
  ;; Add keys.
  (unless erc-button-keys-added
    (define-key erc-mode-map (kbd "<backtab>") #'erc-button-previous)
    (setq erc-button-keys-added t)))

(defun erc-button-add-buttons ()
  "Find external references in the current buffer and make buttons of them.
\"External references\" are things like URLs, as
specified by `erc-button-alist'."
  (interactive)
  (save-excursion
    (with-syntax-table erc-button-syntax-table
      (let ((buffer-read-only nil)
            (inhibit-field-text-motion t)
            (alist erc-button-alist)
            regexp)
        (erc-button-remove-old-buttons)
        (dolist (entry alist)
          (if (or (eq (car entry) 'nicknames)
                  ;; Old form retained for backward compatibility.
                  (equal (car entry) (quote 'nicknames)))
              (erc-button-add-nickname-buttons entry)
            (progn
              (setq regexp (or (and (stringp (car entry)) (car entry))
                               (and (boundp (car entry))
                                    (symbol-value (car entry)))))
              (cond ((stringp regexp)
                     (erc-button-add-buttons-1 regexp entry))
                    ((and (listp regexp) (stringp (car regexp)))
                     (dolist (r regexp)
                       (erc-button-add-buttons-1
                        (concat "\\<" (regexp-quote r) "\\>")
                        entry)))
                    ((and (listp regexp) (listp (car regexp))
                          (stringp (caar regexp)))
                     (dolist (elem regexp)
                       (erc-button-add-buttons-1
                        (concat "\\<" (regexp-quote (car elem)) "\\>")
                        entry)))))))))))

(defun erc-button--maybe-warn-arbitrary-sexp (form)
  (cl-assert (not (booleanp form))) ; covered by caller
  ;; If a special-variable is also a function, favor the function.
  (cond ((functionp form) form)
        ((and (symbolp form) (special-variable-p form)) (symbol-value form))
        (t (unless (get 'erc-button--maybe-warn-arbitrary-sexp
                        'warned-arbitrary-sexp)
             (put 'erc-button--maybe-warn-arbitrary-sexp
                  'warned-arbitrary-sexp t)
             (lwarn 'erc :warning (concat "Arbitrary sexps for the third FORM"
                                          " slot of `erc-button-alist' entries"
                                          " have been deprecated.")))
           (eval form t))))

(defun erc-button--check-nicknames-entry ()
  ;; This helper exists because the module is defined after its options.
  (when (eq major-mode 'erc-mode)
    (unless (eq (nth 1 (alist-get 'nicknames erc-button-alist))
                'erc-button-buttonize-nicks)
      (erc-button--display-error-notice-with-keys-and-warn
       "Values other than `erc-button-buttonize-nicks' in the third slot of "
       "the `nicknames' entry of `erc-button-alist' are deprecated."))))

(cl-defstruct erc-button--nick
  ( bounds nil :type cons
    ;; Indicates the nick's position in the current message.  BEG is
    ;; normally also point.
    :documentation "A cons of (BEG . END).")
  ( data nil :type (or null cons)
    ;; When non-nil, the CAR must be a non-casemapped nickname.  For
    ;; compatibility, the CDR should probably be nil, but this may
    ;; have to change eventually.  If non-nil, the entire cons should
    ;; be mutated rather than replaced because it's used as a key in
    ;; hash tables and text-property searches.
    :documentation "A unique cons whose car is a nickname.")
  ( downcased nil :type (or null string)
    :documentation "The case-mapped nickname sans text properties.")
  ( user nil :type (or null erc-server-user)
    ;; Not necessarily present in `erc-server-users'.
    :documentation "A possibly nil or spoofed `erc-server-user'.")
  ( cuser nil :type (or null erc-channel-user)
    ;; The CDR of a value from an `erc-channel-users' table.
    :documentation "A possibly nil `erc-channel-user'.")
  ( erc-button-face erc-button-face :type symbol
    :documentation "Temp `erc-button-face' while buttonizing.")
  ( erc-button-nickname-face erc-button-nickname-face :type symbol
    :documentation "Temp `erc-button-nickname-face' while buttonizing.")
  ( erc-button-mouse-face erc-button-mouse-face :type symbol
    :documentation "Temp `erc-button-mouse-face' while buttonizing."))

;; This variable is intended to serve as a "core" to be wrapped by
;; (built-in) modules during setup.  It's unclear whether
;; `add-function's practice of removing existing advice before
;; re-adding it is desirable when integrating modules since we're
;; mostly concerned with ensuring one "piece" precedes or follows
;; another (specific piece), which may not yet (or ever) be present.

(defvar erc-button--modify-nick-function #'identity
  "Function to possibly modify aspects of nick being buttonized.
Called with one argument, an `erc-button--nick' object, or nil.
The function should return the same (or similar) object when
buttonizing ought to proceed and nil otherwise.  While running,
all faces defined in `erc-button' are bound temporarily and can
be updated at will.")

(defvar-local erc-button--phantom-users nil)

(defvar erc-button--fallback-user-function #'ignore
  "Function to determine `erc-server-user' if not found in the usual places.
Called with DOWNCASED-NICK, NICK, and NICK-BOUNDS when
`erc-button-add-nickname-buttons' cannot find a user object for
DOWNCASED-NICK in `erc-channel-users' or `erc-server-users'.")

(defun erc-button--add-phantom-speaker (downcased nuh _parsed)
  "Stash fictitious `erc-server-user' while processing \"PRIVMSG\".
Expect DOWNCASED to be the downcased nickname, NUH to be a triple
of (NICK LOGIN HOST), and parsed to be an `erc-response' object."
  (pcase-let* ((`(,nick ,login ,host) nuh)
               (user (or (gethash downcased erc-button--phantom-users)
                         (make-erc-server-user
                          :nickname nick
                          :host (and (not (string-empty-p host)) host)
                          :login (and (not (string-empty-p login)) login)))))
    (list (puthash downcased user erc-button--phantom-users))))

(defun erc-button--get-phantom-user (down _word _bounds)
  (gethash down erc-button--phantom-users))

;; In the future, we'll most likely create temporary
;; `erc-channel-users' tables during BATCH chathistory playback, thus
;; obviating the need for this mode entirely.
(define-minor-mode erc-button--phantom-users-mode
  "Minor mode to recognize unknown speakers.
Expect to be used by module setup code for creating placeholder
users on the fly during history playback.  Treat an unknown
\"PRIVMSG\" speaker, like \"<bob>\", as if they previously
appeared in a prior \"353\" message and are thus a known member
of the channel.  However, don't bother creating an actual
`erc-channel-user' object because their status prefix is unknown.
Instead, just spoof an `erc-server-user' and stash it during
\"PRIVMSG\" handling via `erc--user-from-nick-function' and
retrieve it during buttonizing via
`erc-button--fallback-user-function'."
  :interactive nil
  (if erc-button--phantom-users-mode
      (progn
        (add-function :after-until (local 'erc--user-from-nick-function)
                      #'erc-button--add-phantom-speaker '((depth . -50)))
        (add-function :after-until (local 'erc-button--fallback-user-function)
                      #'erc-button--get-phantom-user '((depth . 50)))
        (setq erc-button--phantom-users (make-hash-table :test #'equal)))
    (remove-function (local 'erc--user-from-nick-function)
                     #'erc-button--add-phantom-speaker)
    (remove-function (local 'erc-button--fallback-user-function)
                     #'erc-button--get-phantom-user)
    (kill-local-variable 'erc-nicks--phantom-users)))

(defun erc-button-add-nickname-buttons (entry)
  "Search through the buffer for nicknames, and add buttons."
  (let ((form (nth 2 entry))
        (fun (nth 3 entry))
        bounds word)
    (when (eq form 'erc-button-buttonize-nicks)
      (setq form (and (symbol-value form) erc-button--modify-nick-function)))
    (when (or (functionp form)
              (eq t form)
              (and form (erc-button--maybe-warn-arbitrary-sexp form)))
      (goto-char (point-min))
      (while (erc-forward-word)
        (when (setq bounds (erc-bounds-of-word-at-point))
          (setq word (buffer-substring-no-properties
                      (car bounds) (cdr bounds)))
          (let* ((erc-button-face erc-button-face)
                 (erc-button-mouse-face erc-button-mouse-face)
                 (erc-button-nickname-face erc-button-nickname-face)
                 (down (erc-downcase word))
                 (cuser (and erc-channel-users
                             (gethash down erc-channel-users)))
                 (user (or (and cuser (car cuser))
                           (and erc-server-users
                                (gethash down erc-server-users))
                           (funcall erc-button--fallback-user-function
                                    down word bounds)))
                 (data (list word)))
            (when (or (not (functionp form))
                      (and-let* ((user)
                                 (obj (funcall form (make-erc-button--nick
                                                     :bounds bounds :data data
                                                     :downcased down :user user
                                                     :cuser (cdr cuser)))))
                        (setq bounds (erc-button--nick-bounds obj)
                              data (erc-button--nick-data obj)
                              erc-button-mouse-face
                              (erc-button--nick-erc-button-mouse-face obj)
                              erc-button-nickname-face
                              (erc-button--nick-erc-button-nickname-face obj)
                              erc-button-face
                              (erc-button--nick-erc-button-face obj))))
              (erc-button-add-button (car bounds) (cdr bounds)
                                     fun t data))))))))

(defun erc-button-add-buttons-1 (regexp entry)
  "Search through the buffer for matches to ENTRY and add buttons."
  (goto-char (point-min))
  (let (buttonizer)
    (while
        (and (re-search-forward regexp nil t)
             (or buttonizer
                 (setq buttonizer
                       (and-let*
                           ((raw-form (nth 2 entry))
                            (res (or (eq t raw-form)
                                     (erc-button--maybe-warn-arbitrary-sexp
                                      raw-form))))
                         (if (functionp res) res #'erc-button-add-button)))))
      (let ((start (match-beginning (nth 1 entry)))
            (end (match-end (nth 1 entry)))
            (fun (nth 3 entry))
            (data (mapcar #'match-string-no-properties (nthcdr 4 entry))))
        (funcall buttonizer start end fun nil data regexp)))))

(defun erc-button-remove-old-buttons ()
  "Remove all existing buttons.
This is called with narrowing in effect, just before the text is
buttonized again.  Removing a button means to remove all the properties
that `erc-button-add-button' adds, except for the face."
  (remove-text-properties
   (point-min) (point-max)
   '(erc-callback nil
                  erc-data nil
                  mouse-face nil
                  keymap nil)))

(defun erc-button-add-button (from to fun nick-p &optional data regexp)
  "Create a button between FROM and TO with callback FUN and data DATA.
NICK-P specifies if this is a nickname button.
REGEXP is the regular expression which matched for this button."
  ;; Really nasty hack to <URL: > ise urls, and line-wrap them if
  ;; they're going to be wider than `erc-fill-column'.
  ;; This could be a lot cleaner, but it works for me -- lawrence.
  (let (fill-column)
    (when (and erc-button-wrap-long-urls
               (string= regexp erc-button-url-regexp)
               (> (- to from)
                  (setq fill-column (- (if (numberp erc-button-wrap-long-urls)
                                           erc-button-wrap-long-urls
                                         erc-fill-column)
                                       (length erc-fill-prefix)))))
      (setq to (prog1 (point-marker) (insert ">"))
            from (prog2 (goto-char from) (point-marker) (insert "<URL: ")))
      (let ((pos (copy-marker from)))
        (while (> (- to pos) fill-column)
          (goto-char (+ pos fill-column))
          (insert "\n" erc-fill-prefix) ; This ought to figure out
                                        ; what type of filling we're
                                        ; doing, and indent accordingly.
          (move-marker pos (point))))))
  (if nick-p
      (when erc-button-nickname-face
        (erc-button-add-face from to erc-button-nickname-face))
    (when erc-button-face
      (erc-button-add-face from to erc-button-face)))
  (add-text-properties
   from to
   (nconc (and erc-button-mouse-face
               (list 'mouse-face erc-button-mouse-face))
          (list 'erc-callback fun)
          (list 'keymap erc-button-keymap)
          (list 'rear-nonsticky t)
          (and data (list 'erc-data data)))))

(defun erc-button-add-face (from to face)
  "Add FACE to the region between FROM and TO."
  ;; If we just use `add-text-property', then this will overwrite any
  ;; face text property already used for the button.  It will not be
  ;; merged correctly.  If we use overlays, then redisplay will be
  ;; very slow with lots of buttons.  This is why we manually merge
  ;; face text properties.
  (let ((old (erc-list (get-text-property from 'font-lock-face)))
        (pos from)
        (end (next-single-property-change from 'font-lock-face nil to))
        new)
    ;; old is the face at pos, in list form.  It is nil if there is no
    ;; face at pos.  If nil, the new face is FACE.  If not nil, the
    ;; new face is a list containing FACE and the old stuff.  end is
    ;; where this face changes.
    (while (< pos to)
      (setq new (if old (cons face old) face))
      (put-text-property pos end 'font-lock-face new)
      (setq pos end
            old (erc-list (get-text-property pos 'font-lock-face))
            end (next-single-property-change pos 'font-lock-face nil to)))))

;; widget-button-click calls with two args, we ignore the first.
;; Since Emacs runs this directly, rather than with
;; widget-button-click, we need to fake an extra arg in the
;; interactive spec.
(defun erc-button-click-button (_ignore event)
  "Call `erc-button-press-button'."
  (interactive "P\ne")
  (save-excursion
    (mouse-set-point event)
    (erc-button-press-button)))

(defun erc-button-press-button (&rest _ignore)
  "Check text at point for a callback function.
If the text at point has a `erc-callback' property,
call it with the value of the `erc-data' text property."
  (declare (advertised-calling-convention () "28.1"))
  (interactive)
  (let* ((data (get-text-property (point) 'erc-data))
         (fun (get-text-property (point) 'erc-callback)))
    (unless fun
      (message "No button at point"))
    (when (and fun (symbolp fun) (not (fboundp fun)))
      (error "Function %S is not bound" fun))
    (apply fun data)))

(defun erc-button-next-function ()
  "Pseudo completion function that actually jumps to the next button.
For use on `completion-at-point-functions'."
  (declare (obsolete erc-nickserv-identify "30.1"))
  ;; FIXME: This is an abuse of completion-at-point-functions.
  (when (< (point) (erc-beg-of-input-line))
    (let ((start (point)))
      (lambda ()
        (let ((here start))
          ;; FIXME: Use next-single-property-change.
          (while (and (get-text-property here 'erc-callback)
                      (not (= here (point-max))))
            (setq here (1+ here)))
          (while (not (or (get-text-property here 'erc-callback)
                          (= here (point-max))))
            (setq here (1+ here)))
          (if (< here (point-max))
              (goto-char here)
            (error "No next button"))
          t)))))

(defvar erc-button--prev-next-predicate-functions
  '(erc-button--end-of-button-p)
  "Abnormal hook whose members can return non-nil to continue searching.
Otherwise, if all members return nil, point will stay at the
current button.  Called with a single arg, a buffer position
greater than `point-min' with a text property of `erc-callback'.")

(defun erc-button--end-of-button-p (point)
  (get-text-property (1- point) 'erc-callback))

(defun erc--button-next (arg)
  (let* ((nextp (prog1 (>= arg 1) (setq arg (max 1 (abs arg)))))
         (search-fn (if nextp
                        #'next-single-char-property-change
                      #'previous-single-char-property-change))
         (start (point))
         (p start))
    (while (progn
             ;; Break out of current search context.
             (when-let ((low (max (point-min) (1- (pos-bol))))
                        (high (min (point-max) (1+ (pos-eol))))
                        (prop (get-text-property p 'erc-callback))
                        (q (if nextp
                               (text-property-not-all p high
                                                      'erc-callback prop)
                             (funcall search-fn p 'erc-callback nil low)))
                        ((< low q high)))
               (setq p q))
             ;; Assume that buttons occur frequently enough that
             ;; omitting LIMIT is acceptable.
             (while
                 (and (setq p (funcall search-fn p 'erc-callback))
                      (if nextp (< p erc-insert-marker) (/= p (point-min)))
                      (run-hook-with-args-until-success
                       'erc-button--prev-next-predicate-functions p)))
             (and arg
                  (< (point-min) p erc-insert-marker)
                  (goto-char p)
                  (not (zerop (cl-decf arg))))))
    (when (= (point) start)
      (user-error (if nextp "No next button" "No previous button")))
    t))

(defun erc-button-next (&optional arg)
  "Go to the ARGth next button."
  (declare (advertised-calling-convention (arg) "30.1"))
  (interactive "p")
  (setq arg (pcase arg ((pred listp) (prefix-numeric-value arg)) (_ arg)))
  (erc--button-next arg))

(defun erc-button-previous (&optional arg)
  "Go to ARGth previous button."
  (declare (advertised-calling-convention (arg) "30.1"))
  (interactive "p")
  (setq arg (pcase arg ((pred listp) (prefix-numeric-value arg)) (_ arg)))
  (erc--button-next (- arg)))

(defun erc-button-previous-of-nick (arg)
  "Go to ARGth previous button for nick at point."
  (interactive "p")
  (if-let* ((prop (get-text-property (point) 'erc-data))
            (erc-button--prev-next-predicate-functions
             (cons (lambda (p)
                     (not (equal (get-text-property p 'erc-data) prop)))
                   erc-button--prev-next-predicate-functions)))
      (erc--button-next (- arg))
    (user-error "No nick at point")))

(defun erc-browse-emacswiki (thing)
  "Browse to THING in the emacs-wiki."
  (browse-url (concat erc-emacswiki-url thing)))

(defun erc-browse-emacswiki-lisp (thing)
  "Browse to THING in the emacs-wiki elisp area."
  (browse-url (concat erc-emacswiki-lisp-url thing)))

;;; Nickname buttons:

(defcustom erc-nick-popup-alist
  '(("DeOp"  . (erc-cmd-DEOP nick))
    ("Kick"  . (erc-cmd-KICK (concat nick " "
                                     (read-from-minibuffer
                                      (concat "Kick " nick ", reason: ")))))
    ("Msg"   . (erc-cmd-MSG (concat nick " "
                                    (read-from-minibuffer
                                     (concat "Message to " nick ": ")))))
    ("Op"    . (erc-cmd-OP nick))
    ("Query" . (erc-cmd-QUERY nick))
    ("Whois" . (erc-cmd-WHOIS nick))
    ("Lastlog" . (erc-cmd-LASTLOG nick)))
  "An alist of possible actions to take on a nickname.
An entry looks like (\"Action\" . SEXP) where SEXP is evaluated with
the variable `nick' bound to the nick in question.

Examples:
 (\"DebianDB\" .
  (shell-command
   (format
    \"ldapsearch -x -P 2 -h db.debian.org -b dc=debian,dc=org ircnick=%s\"
    nick)))"
  :type '(repeat (cons (string :tag "Op")
                       sexp)))

(defun erc-nick-popup (nick)
  (let* ((completion-ignore-case t)
         (action (completing-read (format-message
                                   "What action to take on `%s'? " nick)
                                  erc-nick-popup-alist))
         (code (cdr (assoc action erc-nick-popup-alist))))
    (when code
      (erc-set-active-buffer (current-buffer))
      (eval code `((nick . ,nick))))))

;;; Callback functions
(defun erc-button-describe-symbol (symbol-name)
  "Describe SYMBOL-NAME.
Use `describe-function' for functions, `describe-variable' for variables,
and `apropos' for other symbols."
  (let ((symbol (intern-soft symbol-name)))
    (cond ((and symbol (fboundp symbol))
           (describe-function symbol))
          ((and symbol (boundp symbol))
           (describe-variable symbol))
          (t (apropos symbol-name)))))

(defun erc-button-beats-to-time (beats)
  "Display BEATS in a readable time format."
  (let* ((seconds (- (* (string-to-number beats) 86.4)
                     3600
                     (- (car (current-time-zone)))))
         (hours (mod (floor seconds 3600) 24))
         (minutes (mod (round seconds 60) 60)))
    (message "@%s is %d:%02d local time"
             beats hours minutes)))

(defun erc-button--display-error-with-buttons
    (from to fun nick-p &optional data regexp)
  "Replace command in region with keys and return new bounds"
  (let* ((o (buffer-substring from to))
         (s (substitute-command-keys o))
         (erc-button-face (and (equal o s) erc-button-face)))
    (delete-region from to)
    (insert s)
    (erc-button-add-button from (point) fun nick-p data regexp)))

;;;###autoload
(defun erc-button--display-error-notice-with-keys (&optional parsed buffer
                                                             &rest strings)
  "Add help keys to STRINGS for configuration-related admonishments.
Return inserted result.  Expect PARSED to be an `erc-response'
object, a string, or nil.  Expect BUFFER to be a buffer, a string,
or nil.  As a special case, allow PARSED to be a buffer as long
as BUFFER is a string or nil.  If STRINGS contains any trailing
non-strings, concatenate leading string members before applying
`format'.  Otherwise, just concatenate everything."
  (when (stringp buffer)
    (push buffer strings)
    (setq buffer nil))
  (when (stringp parsed)
    (push parsed strings)
    (setq parsed nil))
  (when (bufferp parsed)
    (cl-assert (null buffer))
    (setq buffer parsed
          parsed nil))
  (let* ((op (if (seq-every-p #'stringp (cdr strings))
                 #'concat
               (let ((head (pop strings)))
                 (while (stringp (car strings))
                   (setq head (concat head (pop strings))))
                 (push head strings))
               #'format))
         (string (apply op strings))
         (erc-insert-post-hook
          (cons (lambda ()
                  (setq string (buffer-substring (point-min)
                                                 (1- (point-max)))))
                erc-insert-post-hook))
         (erc-button-alist
          `((,(rx "\\[" (group (+ (not "]"))) "]") 0
             erc-button--display-error-with-buttons
             erc-button-describe-symbol 1)
            ,@erc-button-alist)))
    (erc-display-message parsed '(notice error) (or buffer 'active) string)
    string))

;;;###autoload
(defun erc-button--display-error-notice-with-keys-and-warn (&rest args)
  "Like `erc-button--display-error-notice-with-keys' but also warn."
  (let ((string (apply #'erc-button--display-error-notice-with-keys args)))
    (with-temp-buffer
      (insert string)
      (goto-char (point-min))
      (with-syntax-table lisp-mode-syntax-table
        (skip-syntax-forward "^-"))
      (forward-char)
      (display-warning
       'erc (buffer-substring-no-properties (point) (point-max))))))

(provide 'erc-button)

;;; erc-button.el ends here
;; Local Variables:
;; generated-autoload-file: "erc-loaddefs.el"
;; End:
