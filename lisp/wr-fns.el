;;; wr-fns.el --- Webrender specific functions       -*- lexical-binding: t; -*-

;; Copyright (C) 2021  Declan Qian

;; Author: Declan Qian <>
;; Keywords: lisp

;; This program is free software; you can redistribute it and/or modify
;; it under the terms of the GNU General Public License as published by
;; the Free Software Foundation, either version 3 of the License, or
;; (at your option) any later version.

;; This program is distributed in the hope that it will be useful,
;; but WITHOUT ANY WARRANTY; without even the implied warranty of
;; MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
;; GNU General Public License for more details.

;; You should have received a copy of the GNU General Public License
;; along with this program.  If not, see <https://www.gnu.org/licenses/>.

;;; Commentary:

;;

;;; Code:

(require 'transient)

(defconst WR-CAPTURE-BITS '(("SCENE" . #x1)
                            ("FRAME" . #x2)
                            ("TILE_CACHE" . #x4)
                            ("EXTERNAL_RESOURCES" . #x8)
                            ("ALL" . #xf)))

(defvar wr-capture--in-progress nil)

(defun wr-capture--in-progress-p ()
  wr-capture--in-progress)

(defun wr-capture--bits-completion-table (completions)
  (lambda (string pred action)
    (if (eq action 'metadata)
        `(metadata (display-sort-function . ,#'identity))
      (complete-with-action action completions string pred))))

(defun wr-capture--bits-format (value &optional key)
  (format "%s(#x%x)" (or key (car (rassoc value WR-CAPTURE-BITS))) value))

(defun wr-capture--bits-reader (prompt initial-input _history)
  "Read a Bit flags."
  (let* ((initial-input (and transient-read-with-initial-input
                             (wr-capture--bits-format initial-input)))
         (sorted-bits (sort
                       (copy-sequence WR-CAPTURE-BITS)
                       (lambda (a b)
                         (> (cdr a) (cdr b)))))
         (choices (mapcar
                   (lambda (element)
                     (let ((key (car element))
                           (value (cdr element)))
                       (wr-capture--bits-format value key)))
                   sorted-bits))
         (choice
          (completing-read
           prompt (wr-capture--bits-completion-table choices)
           nil t initial-input))
         (key (car (split-string choice "(")))
         (value (cdr (assoc key WR-CAPTURE-BITS))))
    value))

(defun wr-capture--generate-capture-path (path)
  "Increment the extension until we find a fresh path"
  (let* ((path (expand-file-name "wr-capture" path))
         (count (string-to-number (or (file-name-extension path) "0"))))
    (progn
      (while (file-exists-p (concat path "." (number-to-string count)))
        (setq count (+ count 1)))
      (concat path "." (number-to-string count)))))

(defun wr-capture--path-reader (prompt initial-input _history)
  "`wr-capture--path-infix' transient infix reader."
  (let* ((initial-input (and transient-read-with-initial-input
                             (file-name-directory initial-input)))
         (path (file-local-name
                (expand-file-name
                 (read-directory-name prompt initial-input))))
         (path (wr-capture--generate-capture-path path)))
    path))

(defclass wr-capture--bits-infix (transient-infix) ()
  "Bit flags for WR stages to store in a capture.")

(defclass wr-capture--path-infix (transient-infix) ()
  "wr capture path infix")

(cl-defmethod transient-format-value ((obj wr-capture--bits-infix))
  (with-slots (value) obj
    (propertize (wr-capture--bits-format value)
                'face 'transient-value)))

(cl-defmethod transient-format-value ((obj wr-capture--path-infix))
  (with-slots (value) obj
    (propertize value
                'face 'transient-value)))

(defun wr-capture--bits-init-value (obj)
  (let* ((hvalue (cadr (oref transient--prefix value)))
         (value (if (and hvalue (numberp hvalue))
                    hvalue
                   #xf)))
    (oset obj value value)))

(defun wr-capture--path-init-value (obj)
  (let* ((hvalue (car (oref transient--prefix value)))
         (storage-path (if (and hvalue (stringp hvalue))
                           (file-name-directory hvalue)
                         default-directory))
         (value (wr-capture--generate-capture-path storage-path)))
    (oset obj value value)))

(transient-define-infix  wr-capture.bits()
  :description "Bit flags for WR stages to store in a capture."
  :argument "bits"
  :init-value #'wr-capture--bits-init-value
  :always-read t
  :reader #'wr-capture--bits-reader
  :class 'wr-capture--bits-infix)

(transient-define-infix  wr-capture.path()
  :description "Save to directory: "
  :argument "path"
  :init-value #'wr-capture--path-init-value
  :reader #'wr-capture--path-reader
  :class 'wr-capture--path-infix
  :always-read t)

;;;###autoload (autoload 'wr-capture "wr-fns" nil t)
(transient-define-prefix wr-capture ()
  ;; Transient dispatcher for WebRender capture infrastructure commands
  ["Arguments"
   :if-not wr-capture--in-progress-p
   ("--" wr-capture.path)
   ("-b" wr-capture.bits)]
  ["Actions"
   :if-not wr-capture--in-progress-p
   ("c" wr-capture-suffix)
   ("s" wr-start-capture-sequence-suffix)]
  ["Actions"
   :if wr-capture--in-progress-p
   ("S" wr-stop-capture-sequence-suffix)]
  (interactive)
  (unless (featurep 'wr) (user-error "Webrender not available"))
  (transient-setup 'wr-capture))

(transient-define-suffix wr-capture-suffix ()
  "Transient suffix to invoke `wr--capture'"
  :description "Capture"
  (interactive)
  (apply #'wr-api-capture (transient-args transient-current-command)))

(transient-define-suffix wr-start-capture-sequence-suffix ()
  "Transient suffix to invoke `wr--start-capture-sequence'"
  :description "Start capture sequence"
  (interactive)
  (apply #'wr-api-capture `(,@(transient-args transient-current-command) 't))
  (setq wr-capture--in-progress t))

(transient-define-suffix wr-stop-capture-sequence-suffix ()
  "Transient suffix to invoke `wr--stop-capture-sequence'"
  :description "Stop capture sequence"
  (interactive)
  (wr-api-stop-capture-sequence)
  (setq wr-capture--in-progress nil))

(provide 'wr-fns)
;;; wr-fns.el ends here
