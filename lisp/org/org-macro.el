;;; org-macro.el --- Macro Replacement Code for Org  -*- lexical-binding: t; -*-

;; Copyright (C) 2013-2020 Free Software Foundation, Inc.

;; Author: Nicolas Goaziou <n.goaziou@gmail.com>
;; Keywords: outlines, hypermedia, calendar, wp

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

;; Macros are expanded with `org-macro-replace-all', which relies
;; internally on `org-macro-expand'.

;; Default templates for expansion are stored in the buffer-local
;; variable `org-macro-templates'.  This variable is updated by
;; `org-macro-initialize-templates', which recursively calls
;; `org-macro--collect-macros' in order to read setup files.

;; Argument in macros are separated with commas. Proper escaping rules
;; are implemented in `org-macro-escape-arguments' and arguments can
;; be extracted from a string with `org-macro-extract-arguments'.

;; Along with macros defined through #+MACRO: keyword, default
;; templates include the following hard-coded macros:
;;   {{{time(format-string)}}},
;;   {{{property(node-property)}}},
;;   {{{input-file}}},
;;   {{{modification-time(format-string)}}},
;;   {{{n(counter,action}}}.

;; Upon exporting, "ox.el" will also provide {{{author}}}, {{{date}}},
;; {{{email}}} and {{{title}}} macros.

;;; Code:
(require 'cl-lib)
(require 'org-macs)
(require 'org-compat)

(declare-function org-element-at-point "org-element" ())
(declare-function org-element-context "org-element" (&optional element))
(declare-function org-element-copy "org-element" (datum))
(declare-function org-element-macro-parser "org-element" ())
(declare-function org-element-parse-secondary-string "org-element" (string restriction &optional parent))
(declare-function org-element-property "org-element" (property element))
(declare-function org-element-restriction "org-element" (element))
(declare-function org-element-type "org-element" (element))
(declare-function org-entry-get "org" (pom property &optional inherit literal-nil))
(declare-function org-file-contents "org" (file &optional noerror nocache))
(declare-function org-file-url-p "org" (file))
(declare-function org-in-commented-heading-p "org" (&optional no-inheritance))
(declare-function org-link-search "ol" (s &optional avoid-pos stealth))
(declare-function org-mode "org" ())
(declare-function vc-backend "vc-hooks" (f))
(declare-function vc-call "vc-hooks" (fun file &rest args) t)
(declare-function vc-exec-after "vc-dispatcher" (code))

(defvar org-link-search-must-match-exact-headline)

;;; Variables

(defvar-local org-macro-templates nil
  "Alist containing all macro templates in current buffer.
Associations are in the shape of (NAME . TEMPLATE) where NAME
stands for macro's name and template for its replacement value,
both as strings.  This is an internal variable.  Do not set it
directly, use instead:

  #+MACRO: name template")

;;; Functions

(defun org-macro--set-template (name value templates)
  "Set template for the macro NAME.
VALUE is the template of the macro.  The new value override the
previous one, unless VALUE is nil.  TEMPLATES is the list of
templates.  Return the updated list."
  (when value
    (let ((old-definition (assoc name templates)))
      (if old-definition
	  (setcdr old-definition value)
	(push (cons name value) templates))))
  templates)

(defun org-macro--collect-macros (&optional files templates)
  "Collect macro definitions in current buffer and setup files.
Return an alist containing all macro templates found.

FILES is a list of setup files names read so far, used to avoid
circular dependencies.  TEMPLATES is the alist collected so far.
The two arguments are used in recursive calls."
  (let ((case-fold-search t))
    (org-with-point-at 1
      (while (re-search-forward "^[ \t]*#\\+\\(MACRO\\|SETUPFILE\\):" nil t)
	(let ((element (org-element-at-point)))
	  (when (eq (org-element-type element) 'keyword)
	    (let ((val (org-element-property :value element)))
	      (if (equal "MACRO" (org-element-property :key element))
		  ;; Install macro in TEMPLATES.
		  (when (string-match "^\\(\\S-+\\)[ \t]*" val)
		    (let ((name (match-string 1 val))
			  (value (substring val (match-end 0))))
		      (setq templates
			    (org-macro--set-template name value templates))))
		;; Enter setup file.
		(let* ((uri (org-strip-quotes val))
		       (uri-is-url (org-file-url-p uri))
		       (uri (if uri-is-url
				uri
			      (expand-file-name uri))))
		  ;; Avoid circular dependencies.
		  (unless (member uri files)
		    (with-temp-buffer
		      (unless uri-is-url
			(setq default-directory (file-name-directory uri)))
		      (org-mode)
		      (insert (org-file-contents uri 'noerror))
		      (setq templates
			    (org-macro--collect-macros
			     (cons uri files) templates)))))))))))
    (let ((macros `(("author" . ,(org-macro--find-keyword-value "AUTHOR"))
		    ("email" . ,(org-macro--find-keyword-value "EMAIL"))
		    ("title" . ,(org-macro--find-keyword-value "TITLE" t))
		    ("date" . ,(org-macro--find-date)))))
      (pcase-dolist (`(,name . ,value) macros)
	(setq templates (org-macro--set-template name value templates))))
    templates))

(defun org-macro-initialize-templates ()
  "Collect macro templates defined in current buffer.

Templates are stored in buffer-local variable
`org-macro-templates'.

In addition to buffer-defined macros, the function installs the
following ones: \"n\", \"author\", \"email\", \"keyword\",
\"time\", \"property\", and, if the buffer is associated to
a file, \"input-file\" and \"modification-time\"."
  (require 'org-element)
  (org-macro--counter-initialize)	;for "n" macro
  (setq org-macro-templates
	(nconc
	 ;; Install user-defined macros.
	 (org-macro--collect-macros)
	 ;; Install file-specific macros.
	 (let ((visited-file (buffer-file-name (buffer-base-buffer))))
	   (and visited-file
		(file-exists-p visited-file)
		(list
		 `("input-file" . ,(file-name-nondirectory visited-file))
		 `("modification-time" .
		   ,(format "(eval
\(format-time-string $1
                     (or (and (org-string-nw-p $2)
                              (org-macro--vc-modified-time %s))
                     '%s)))"
			    (prin1-to-string visited-file)
			    (prin1-to-string
			     (file-attribute-modification-time
			      (file-attributes visited-file))))))))
	 ;; Install generic macros.
	 (list
	  '("n" . "(eval (org-macro--counter-increment $1 $2))")
	  '("keyword" . "(eval (org-macro--find-keyword-value $1))")
	  '("time" . "(eval (format-time-string $1))")
	  '("property" . "(eval (org-macro--get-property $1 $2))")))))

(defun org-macro-expand (macro templates)
  "Return expanded MACRO, as a string.
MACRO is an object, obtained, for example, with
`org-element-context'.  TEMPLATES is an alist of templates used
for expansion.  See `org-macro-templates' for a buffer-local
default value.  Return nil if no template was found."
  (let ((template
	 ;; Macro names are case-insensitive.
	 (cdr (assoc-string (org-element-property :key macro) templates t))))
    (when template
      (let* ((eval? (string-match-p "\\`(eval\\>" template))
	     (value
	      (replace-regexp-in-string
	       "\\$[0-9]+"
	       (lambda (m)
		 (let ((arg (or (nth (1- (string-to-number (substring m 1)))
				     (org-element-property :args macro))
				;; No argument: remove place-holder.
				"")))
		   ;; `eval' implies arguments are strings.
		   (if eval? (format "%S" arg) arg)))
	       template nil 'literal)))
        (when eval?
          (setq value (eval (condition-case nil (read value)
			      (error (debug))))))
        ;; Force return value to be a string.
        (format "%s" (or value ""))))))

(defun org-macro-replace-all (templates &optional keywords)
  "Replace all macros in current buffer by their expansion.

TEMPLATES is an alist of templates used for expansion.  See
`org-macro-templates' for a buffer-local default value.

Optional argument KEYWORDS, when non-nil is a list of keywords,
as strings, where macro expansion is allowed.

Return an error if a macro in the buffer cannot be associated to
a definition in TEMPLATES."
  (org-with-wide-buffer
   (goto-char (point-min))
   (let ((properties-regexp (format "\\`EXPORT_%s\\+?\\'"
				    (regexp-opt keywords)))
	 record)
     (while (re-search-forward "{{{[-A-Za-z0-9_]" nil t)
       (unless (save-match-data (org-in-commented-heading-p))
	 (let* ((datum (save-match-data (org-element-context)))
		(type (org-element-type datum))
		(macro
		 (cond
		  ((eq type 'macro) datum)
		  ;; In parsed keywords and associated node
		  ;; properties, force macro recognition.
		  ((or (and (eq type 'keyword)
			    (member (org-element-property :key datum) keywords))
		       (and (eq type 'node-property)
			    (string-match-p properties-regexp
					    (org-element-property :key datum))))
		   (save-excursion
		     (goto-char (match-beginning 0))
		     (org-element-macro-parser))))))
	   (when macro
	     (let* ((key (org-element-property :key macro))
		    (value (org-macro-expand macro templates))
		    (begin (org-element-property :begin macro))
		    (signature (list begin
				     macro
				     (org-element-property :args macro))))
	       ;; Avoid circular dependencies by checking if the same
	       ;; macro with the same arguments is expanded at the
	       ;; same position twice.
	       (cond ((member signature record)
		      (error "Circular macro expansion: %s" key))
		     (value
		      (push signature record)
		      (delete-region
		       begin
		       ;; Preserve white spaces after the macro.
		       (progn (goto-char (org-element-property :end macro))
			      (skip-chars-backward " \t")
			      (point)))
		      ;; Leave point before replacement in case of
		      ;; recursive expansions.
		      (save-excursion (insert value)))
		     ;; Special "results" macro: if it is not defined,
		     ;; simply leave it as-is.  It will be expanded in
		     ;; a second phase.
		     ((equal key "results"))
		     (t
		      (error "Undefined Org macro: %s; aborting"
			     (org-element-property :key macro))))))))))))

(defun org-macro-escape-arguments (&rest args)
  "Build macro's arguments string from ARGS.
ARGS are strings.  Return value is a string with arguments
properly escaped and separated with commas.  This is the opposite
of `org-macro-extract-arguments'."
  (let ((s ""))
    (dolist (arg (reverse args) (substring s 1))
      (setq s
	    (concat
	     ","
	     (replace-regexp-in-string
	      "\\(\\\\*\\),"
	      (lambda (m)
		(concat (make-string (1+ (* 2 (length (match-string 1 m)))) ?\\)
			","))
	      ;; If a non-terminal argument ends on backslashes, make
	      ;; sure to also escape them as they will be followed by
	      ;; a comma.
	      (concat arg (and (not (equal s ""))
			       (string-match "\\\\+\\'" arg)
			       (match-string 0 arg)))
	      nil t)
	     s)))))

(defun org-macro-extract-arguments (s)
  "Extract macro arguments from string S.
S is a string containing comma separated values properly escaped.
Return a list of arguments, as strings.  This is the opposite of
`org-macro-escape-arguments'."
  ;; Do not use `org-split-string' since empty strings are
  ;; meaningful here.
  (split-string
   (replace-regexp-in-string
    "\\(\\\\*\\),"
    (lambda (str)
      (let ((len (length (match-string 1 str))))
	(concat (make-string (/ len 2) ?\\)
		(if (zerop (mod len 2)) "\000" ","))))
    s nil t)
   "\000"))


;;; Helper functions and variables for internal macros

(defun org-macro--get-property (property location)
  "Find PROPERTY's value at LOCATION.
PROPERTY is a string.  LOCATION is a search string, as expected
by `org-link-search', or the empty string."
  (save-excursion
    (when (org-string-nw-p location)
      (condition-case _
	  (let ((org-link-search-must-match-exact-headline t))
	    (org-link-search location nil t))
        (error
	 (error "Macro property failed: cannot find location %s" location))))
    (org-entry-get nil property 'selective)))

(defun org-macro--find-keyword-value (name &optional collect)
  "Find value for keyword NAME in current buffer.
Return value associated to the keywords named after NAME, as
a string, or nil.  When optional argument COLLECT is non-nil,
concatenate values, separated with a space, from various keywords
in the buffer."
  (org-with-point-at 1
    (let ((regexp (format "^[ \t]*#\\+%s:" (regexp-quote name)))
	  (case-fold-search t)
	  (result nil))
      (catch :exit
	(while (re-search-forward regexp nil t)
	  (let ((element (org-element-at-point)))
	    (when (eq 'keyword (org-element-type element))
	      (let ((value (org-element-property :value element)))
		(if (not collect) (throw :exit value)
		  (setq result (concat result " " value)))))))
	(and result (org-trim result))))))

(defun org-macro--find-date ()
  "Find value for DATE in current buffer.
Return value as a string."
  (let* ((value (org-macro--find-keyword-value "DATE"))
	 (date (org-element-parse-secondary-string
		value (org-element-restriction 'keyword))))
    (if (and (consp date)
	     (not (cdr date))
	     (eq 'timestamp (org-element-type (car date))))
	(format "(eval (if (org-string-nw-p $1) %s %S))"
		(format "(org-timestamp-format '%S $1)"
			(org-element-copy (car date)))
		value)
      value)))

(defun org-macro--vc-modified-time (file)
  (save-window-excursion
    (when (vc-backend file)
      (let ((buf (get-buffer-create " *org-vc*"))
	    (case-fold-search t)
	    date)
	(unwind-protect
	    (progn
	      (vc-call print-log file buf nil nil 1)
	      (with-current-buffer buf
		(vc-exec-after
		 (lambda ()
		   (goto-char (point-min))
		   (when (re-search-forward "Date:?[ \t]*" nil t)
		     (let ((time (parse-time-string
				  (buffer-substring
				   (point) (line-end-position)))))
		       (when (cl-some #'identity time)
			 (setq date (apply #'encode-time time))))))))
	      (let ((proc (get-buffer-process buf)))
		(while (and proc (accept-process-output proc .5 nil t)))))
	  (kill-buffer buf))
	date))))

(defvar org-macro--counter-table nil
  "Hash table containing counter value per name.")

(defun org-macro--counter-initialize ()
  "Initialize `org-macro--counter-table'."
  (setq org-macro--counter-table (make-hash-table :test #'equal)))

(defun org-macro--counter-increment (name &optional action)
  "Increment counter NAME.
NAME is a string identifying the counter.

When non-nil, optional argument ACTION is a string.

If the string is \"-\", keep the NAME counter at its current
value, i.e. do not increment.

If the string represents an integer, set the counter to this number.

Any other non-empty string resets the counter to 1."
  (let ((name-trimmed (org-trim name))
        (action-trimmed (when (org-string-nw-p action)
                          (org-trim action))))
    (puthash name-trimmed
             (cond ((not (org-string-nw-p action-trimmed))
                    (1+ (gethash name-trimmed org-macro--counter-table 0)))
                   ((string= "-" action-trimmed)
                    (gethash name-trimmed org-macro--counter-table 1))
                   ((string-match-p "\\`[0-9]+\\'" action-trimmed)
                    (string-to-number action-trimmed))
                   (t 1))
             org-macro--counter-table)))


(provide 'org-macro)
;;; org-macro.el ends here
