;;; tramp-container.el --- Tramp integration for Docker-like containers  -*- lexical-binding: t; -*-

;; Copyright © 2022-2024 Free Software Foundation, Inc.

;; Author: Brian Cully <bjc@kublai.com>
;; Keywords: comm, processes
;; Package: tramp

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

;; Allows Tramp access to environments provided by Docker and similar
;; programs.
;;
;; ## Usage
;;
;; Open a file on a running Docker container:
;;
;;     C-x C-f /docker:USER@CONTAINER:/path/to/file
;;
;; or Podman:
;;
;;     C-x C-f /podman:USER@CONTAINER:/path/to/file
;;
;; Where:
;;     USER          is the user on the container to connect as (optional).
;;     CONTAINER     is the container to connect to.
;;
;;
;;
;; Open file in a Kubernetes container:
;;
;;     C-x C-f /kubernetes:[CONTAINER.]POD:/path/to/file
;;
;; Where:
;;     POD           is the pod to connect to.
;;     CONTAINER     is the container to connect to (optional).
;;		     By default, the first container in that pod will
;;		     be used.
;;
;; Completion for POD and accessing it operate in the current
;; namespace, use this command to change it:
;;
;; "kubectl config set-context --current --namespace=<name>"
;;
;;
;;
;; Open a file on an existing Toolbox container:
;;
;;     C-x C-f /toolbox:CONTAINER:/path/to/file
;;
;; Where:
;;     CONTAINER     is the container to connect to (optional).
;;
;; If the container is not running, it is started.  If no container is
;; specified, the default Toolbox container is used.
;;
;;
;;
;; Open a file on a running Flatpak sandbox:
;;
;;     C-x C-f /flatpak:SANDBOX:/path/to/file
;;
;; Where:
;;     SANDBOX	is the running sandbox to connect to.
;;		It could be an application ID, an instance ID, or a PID.

;;; Code:

(require 'tramp)

;;;###tramp-autoload
(defcustom tramp-docker-program "docker"
  "Name of the Docker client program."
  :group 'tramp
  :version "29.1"
  :type '(choice (const "docker")
                 (string)))

;;;###tramp-autoload
(defcustom tramp-podman-program "podman"
  "Name of the Podman client program."
  :group 'tramp
  :version "29.1"
  :type '(choice (const "podman")
                 (string)))

;;;###tramp-autoload
(defcustom tramp-kubernetes-program "kubectl"
  "Name of the Kubernetes client program."
  :group 'tramp
  :version "29.1"
  :type '(choice (const "kubectl")
                 (string)))

(defcustom tramp-kubernetes-context nil
  "Context of Kubernetes.
If it is nil, the default context will be used."
  :group 'tramp
  :version "30.1"
  :type '(choice (const :tag "Use default" nil)
                 (string)))

(defcustom tramp-kubernetes-namespace "default"
  "Namespace of Kubernetes."
  :group 'tramp
  :version "30.1"
  :type 'string)

;;;###tramp-autoload
(defcustom tramp-toolbox-program "toolbox"
  "Name of the Toolbox client program."
  :group 'tramp
  :version "30.1"
  :type '(choice (const "toolbox")
                 (string)))

;;;###tramp-autoload
(defcustom tramp-flatpak-program "flatpak"
  "Name of the Flatpak client program."
  :group 'tramp
  :version "30.1"
  :type '(choice (const "flatpak")
                 (string)))

;;;###tramp-autoload
(defconst tramp-docker-method "docker"
  "Tramp method name to use to connect to Docker containers.")

;;;###tramp-autoload
(defconst tramp-podman-method "podman"
  "Tramp method name to use to connect to Podman containers.")

;;;###tramp-autoload
(defconst tramp-kubernetes-method "kubernetes"
  "Tramp method name to use to connect to Kubernetes containers.")

;;;###tramp-autoload
(defconst tramp-toolbox-method "toolbox"
  "Tramp method name to use to connect to Toolbox containers.")

;;;###tramp-autoload
(defconst tramp-flatpak-method "flatpak"
  "Tramp method name to use to connect to Flatpak sandboxes.")

;;;###tramp-autoload
(defmacro tramp-skeleton-completion-function (method &rest body)
  "Skeleton for `tramp-*-completion-function' with multi-hop support.
BODY is the backend specific code."
  (declare (indent 1) (debug t))
  `(let* ((default-directory
	   (or (and (member ,method tramp-completion-multi-hop-methods)
		    tramp--last-hop-directory)
	       tramp-compat-temporary-file-directory))
	  (program (let ((tramp-verbose 0))
		     (tramp-get-method-parameter
		      (make-tramp-file-name :method ,method)
		      'tramp-login-program)))
	  (vec (when (tramp-tramp-file-p default-directory)
		 (tramp-dissect-file-name default-directory)))
	  non-essential)
     ;; We don't use connection properties, because this information
     ;; shouldn't be kept persistently.
     (with-tramp-file-property
	 vec (concat "/" ,method ":") "user-host-completions"
       ,@body)))

;;;###tramp-autoload
(defun tramp-container--completion-function (method)
  "List running containers available for connection.
METHOD is the Tramp method to be used for \"ps\", either
`tramp-docker-method' or `tramp-podman-method'.

This function is used by `tramp-set-completion-function', please
see its function help for a description of the format."
  (tramp-skeleton-completion-function method
    (when-let ((raw-list
		(shell-command-to-string
		 (concat program " ps --format '{{.ID}}\t{{.Names}}'")))
	       (lines (split-string raw-list "\n" 'omit))
	       (names
		(mapcar
		 (lambda (line)
		   (when (string-match
			  (rx bol (group (1+ nonl))
			      "\t" (? (group (1+ nonl))) eol)
			  line)
		     (or (match-string 2 line) (match-string 1 line))))
		 lines)))
      (mapcar (lambda (name) (list nil name)) (delq nil names)))))

;;;###tramp-autoload
(defun tramp-kubernetes--completion-function (method)
  "List Kubernetes pods available for connection.

This function is used by `tramp-set-completion-function', please
see its function help for a description of the format."
  (tramp-skeleton-completion-function method
    (when-let ((raw-list
		(shell-command-to-string
		 (concat
		  program " "
		  (tramp-kubernetes--context-namespace vec)
                  " get pods --no-headers"
		  ;; We separate pods by "|".  Inside a pod, its name
		  ;; is separated from the containers by ":".
		  ;; Containers are separated by ",".
		  " -o jsonpath='{range .items[*]}{\"|\"}{.metadata.name}"
		  "{\":\"}{range .spec.containers[*]}{.name}{\",\"}"
		  "{end}{end}'")))
               (lines (split-string raw-list "|" 'omit)))
      (let (names)
	(dolist (line lines)
	  (setq line (split-string line ":" 'omit))
	  ;; Pod name.
	  (push (car line) names)
	  ;; Container names.
	  (dolist (elt (split-string (cadr line) "," 'omit))
	    (push (concat elt "." (car line)) names)))
	(mapcar (lambda (name) (list nil name)) (delq nil names))))))

(defconst tramp-kubernetes--host-name-regexp
  (rx (? (group (regexp tramp-host-regexp)) ".")
      (group (regexp tramp-host-regexp)))
  "The CONTAINER.POD syntax of kubernetes host names in Tramp.")

;;;###tramp-autoload
(defun tramp-kubernetes--container (vec)
  "Extract the container name from a kubernetes host name in VEC."
  (or (let ((host (tramp-file-name-host vec)))
	(and (string-match tramp-kubernetes--host-name-regexp host)
	     (match-string 1 host)))
      ""))

;;;###tramp-autoload
(defun tramp-kubernetes--pod (vec)
  "Extract the pod name from a kubernetes host name in VEC."
  (or (let ((host (tramp-file-name-host vec)))
	(and (string-match tramp-kubernetes--host-name-regexp host)
	     (match-string 2 host)))
      ""))

;; We must change `vec' and `default-directory' to the previous hop,
;; in order to run `process-file' in a proper environment.
(defmacro tramp-skeleton-kubernetes-vector (vec &rest body)
  "Skeleton for `tramp-kubernetes--current-context*' with multi-hop support.
BODY is the backend specific code."
  (declare (indent 1) (debug t))
  `(let* ((vec
	   (cond
	    ((null ,vec) tramp-null-hop)
	    ((equal (tramp-file-name-method ,vec) tramp-kubernetes-method)
	     (if (tramp-file-name-hop ,vec)
		 (tramp-dissect-hop-name (tramp-file-name-hop ,vec))
	       tramp-null-hop))
	    (t ,vec)))
	  (default-directory
	   (if (equal vec tramp-null-hop)
	       tramp-compat-temporary-file-directory
	     (tramp-make-tramp-file-name vec "/"))))
     ,@body))

(defun tramp-kubernetes--current-context (vec)
  "Return Kubernetes current context.
Obey `tramp-kubernetes-context'"
  (or tramp-kubernetes-context
      (tramp-skeleton-kubernetes-vector vec
	(with-tramp-file-property
	    vec (concat "/" tramp-kubernetes-method ":") "current-context"
	  (with-temp-buffer
	    (when (zerop
		   (process-file
		    tramp-kubernetes-program nil t nil
		    "config" "current-context"))
	      (goto-char (point-min))
	      (buffer-substring (point) (line-end-position))))))))

(defun tramp-kubernetes--current-context-data (vec)
  "Return Kubernetes current context data as JSON string."
  (when-let ((current-context (tramp-kubernetes--current-context vec)))
    (tramp-skeleton-kubernetes-vector vec
      (with-temp-buffer
	(when (zerop
	       (process-file
		tramp-kubernetes-program nil t nil
		"config" "view" "-o"
		(format
		 "jsonpath='{.contexts[?(@.name == \"%s\")]}'" current-context)))
	  (buffer-string))))))

;;;###tramp-autoload
(defun tramp-kubernetes--context-namespace (vec)
  "The kubectl options for context and namespace as string."
  (mapconcat
   #'identity
   `(,(when-let ((context (tramp-kubernetes--current-context vec)))
	(format "--context=%s" context))
     ,(when tramp-kubernetes-namespace
	(format "--namespace=%s" tramp-kubernetes-namespace)))
   " "))

;;;###tramp-autoload
(defun tramp-toolbox--completion-function (method)
  "List Toolbox containers available for connection.

This function is used by `tramp-set-completion-function', please
see its function help for a description of the format."
  (tramp-skeleton-completion-function method
    (when-let ((raw-list (shell-command-to-string (concat program " list -c")))
	       ;; Ignore header line.
               (lines (cdr (split-string raw-list "\n" 'omit)))
               (names (mapcar
		       (lambda (line)
			 (when (string-match
				(rx bol (1+ (not space))
				    (1+ space) (group (1+ (not space))) space)
				line)
			   (match-string 1 line)))
                       lines)))
      (mapcar (lambda (name) (list nil name)) (delq nil names)))))

;;;###tramp-autoload
(defun tramp-flatpak--completion-function (method)
  "List Flatpak sandboxes available for connection.
It returns application IDs or, in case there is no application
ID, instance IDs.

This function is used by `tramp-set-completion-function', please
see its function help for a description of the format."
  (tramp-skeleton-completion-function method
    (when-let ((raw-list
		(shell-command-to-string
		 ;; Ignore header line.
		 (concat program " ps --columns=instance,application | cat -")))
               (lines (split-string raw-list "\n" 'omit))
               (names (mapcar
		       (lambda (line)
			 (when (string-match
				(rx bol (* space) (group (+ (not space)))
				    (? (+ space) (group (+ (not space)))) eol)
				line)
			   (or (match-string 2 line) (match-string 1 line))))
                       lines)))
      (mapcar (lambda (name) (list nil name)) (delq nil names)))))

;;;###tramp-autoload
(defvar tramp-default-remote-shell) ;; Silence byte compiler.

;;;###tramp-autoload
(tramp--with-startup
 (add-to-list 'tramp-methods
              `(,tramp-docker-method
                (tramp-login-program ,tramp-docker-program)
                (tramp-login-args (("exec")
                                   ("-it")
                                   ("-u" "%u")
                                   ("%h")
			           ("%l")))
		(tramp-direct-async (,tramp-default-remote-shell "-c"))
                (tramp-remote-shell ,tramp-default-remote-shell)
                (tramp-remote-shell-login ("-l"))
                (tramp-remote-shell-args ("-i" "-c"))))

 (add-to-list 'tramp-methods
              `(,tramp-podman-method
                (tramp-login-program ,tramp-podman-program)
                (tramp-login-args (("exec")
                                   ("-it")
                                   ("-u" "%u")
                                   ("%h")
			           ("%l")))
		(tramp-direct-async (,tramp-default-remote-shell "-c"))
                (tramp-remote-shell ,tramp-default-remote-shell)
                (tramp-remote-shell-login ("-l"))
                (tramp-remote-shell-args ("-i" "-c"))))

 (add-to-list 'tramp-methods
              `(,tramp-kubernetes-method
                (tramp-login-program ,tramp-kubernetes-program)
                (tramp-login-args (("%x") ; context and namespace.
				   ("exec")
                                   ("-c" "%a") ; container.
                                   ("%h")
                                   ("-it")
                                   ("--")
			           ("%l")))
		(tramp-direct-async (,tramp-default-remote-shell "-c"))
                (tramp-remote-shell ,tramp-default-remote-shell)
                (tramp-remote-shell-login ("-l"))
                (tramp-remote-shell-args ("-i" "-c"))))

 (add-to-list 'tramp-methods
	      `(,tramp-toolbox-method
		(tramp-login-program ,tramp-toolbox-program)
		(tramp-login-args (("run")
				   ("-c" "%h")
				   ("%l")))
		(tramp-direct-async (,tramp-default-remote-shell "-c"))
		(tramp-remote-shell ,tramp-default-remote-shell)
		(tramp-remote-shell-login ("-l"))
		(tramp-remote-shell-args ("-c"))))

 (add-to-list 'tramp-default-host-alist `(,tramp-toolbox-method nil ""))

 (add-to-list 'tramp-methods
	      `(,tramp-flatpak-method
		(tramp-login-program ,tramp-flatpak-program)
		(tramp-login-args (("enter")
				   ("%h")
				   ("%l")))
		(tramp-direct-async (,tramp-default-remote-shell "-c"))
		(tramp-remote-shell ,tramp-default-remote-shell)
		(tramp-remote-shell-login ("-l"))
		(tramp-remote-shell-args ("-c"))))

 (tramp-set-completion-function
  tramp-docker-method
  `((tramp-container--completion-function ,tramp-docker-method)))

 (tramp-set-completion-function
  tramp-podman-method
  `((tramp-container--completion-function ,tramp-podman-method)))

 (tramp-set-completion-function
  tramp-kubernetes-method
  `((tramp-kubernetes--completion-function ,tramp-kubernetes-method)))

 (tramp-set-completion-function
  tramp-toolbox-method
  `((tramp-toolbox--completion-function ,tramp-toolbox-method)))

 (tramp-set-completion-function
  tramp-flatpak-method
  `((tramp-flatpak--completion-function ,tramp-flatpak-method)))

 (add-to-list 'tramp-completion-multi-hop-methods tramp-docker-method)
 (add-to-list 'tramp-completion-multi-hop-methods tramp-podman-method)
 (add-to-list 'tramp-completion-multi-hop-methods tramp-kubernetes-method)
 (add-to-list 'tramp-completion-multi-hop-methods tramp-toolbox-method)
 (add-to-list 'tramp-completion-multi-hop-methods tramp-flatpak-method)

 ;; Default connection-local variables for Tramp.

 (defconst tramp-kubernetes-connection-local-default-variables
   '((tramp-config-check . tramp-kubernetes--current-context-data)
     ;; This variable will be eval'ed in `tramp-expand-args'.
     (tramp-extra-expand-args
      . (?a (tramp-kubernetes--container (car tramp-current-connection))
	 ?h (tramp-kubernetes--pod (car tramp-current-connection))
	 ?x (tramp-kubernetes--context-namespace
	     (car tramp-current-connection)))))
   "Default connection-local variables for remote kubernetes connections.")

 (connection-local-set-profile-variables
  'tramp-kubernetes-connection-local-default-profile
  tramp-kubernetes-connection-local-default-variables)

 (connection-local-set-profiles
  `(:application tramp :protocol ,tramp-kubernetes-method)
  'tramp-kubernetes-connection-local-default-profile)

 (defconst tramp-flatpak-connection-local-default-variables
   `((tramp-remote-path  . ,(cons "/app/bin" tramp-remote-path)))
   "Default connection-local variables for remote flatpak connections.")

 (connection-local-set-profile-variables
  'tramp-flatpak-connection-local-default-profile
  tramp-flatpak-connection-local-default-variables)

 (connection-local-set-profiles
  `(:application tramp :protocol ,tramp-flatpak-method)
  'tramp-flatpak-connection-local-default-profile))

(add-hook 'tramp-unload-hook
	  (lambda ()
	    (unload-feature 'tramp-container 'force)))

(provide 'tramp-container)

;;; tramp-container.el ends here
