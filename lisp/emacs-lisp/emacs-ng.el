;;; emacs-ng.el --- Emacs NG related -*- lexical-binding: t -*-

;; `straight.el'
(defvar straight-recipes-gnu-elpa-use-mirror)
(defvar straight-recipes-emacsmirror-use-mirror)
(defvar straight-use-symlinks)
(defvar straight-enable-package-integration)
(defvar straight-enable-use-package-integration)
(defvar straight-recipe-repositories)
(defvar straight-recipes-gnu-elpa-url)
(defvar straight-repository-user)
(defvar straight-repository-branch)
(declare-function straight--reset-caches "straight")
(declare-function straight-use-package-mode "straight")
(declare-function straight-package-neutering-mode "straight")
(declare-function straight-symlink-emulation-mode "straight")
(declare-function straight-watcher-start "straight")
(declare-function straight-live-modifications-mode "straight")
(declare-function straight--modifications "straight")

;;; Customization options

;;;###autoload
(defcustom ng-straight-bootstrap-at-startup nil
  "Whether to bootstrap straight.el when Emacs starts.
If non-nil, straight.el is bootstrapped before reading the init
file (but after reading the early init file).  This means that if
you wish to set this variable, you must do so in the early init
file.  Regardless of the value of this variable, straight.el recipes
are not made available if `user-init-file' is nil (e.g. Emacs
was started with \"-q\").

Even if the value is nil, you can type \\[ng-bootstrap-straight] to
bootstrap straight.el at any time, or you can
call (ng-bootstrap-straight) in your init-file."
  :type 'boolean
  :group 'emacs-ng
  :version "28")

;;;###autoload
(defvar ng--straight-bootstrapped nil
  "Non-nil if `ng-bootstrap-straight' has been run.")

;;;###autoload
(defun ng-bootstrap-straight ()
  "Bootstrap straight.el."

  (interactive)

  (require 'straight)

  (setq ng--straight-bootstrapped t)

  (straight--reset-caches)
  (setq straight-recipe-repositories nil)

  (straight-use-recipes '(org-elpa :local-repo nil))

  (straight-use-recipes '(melpa :type git :host github
                                :repo "melpa/melpa"
                                :build nil))

  (if straight-recipes-gnu-elpa-use-mirror
      (straight-use-recipes
       '(gnu-elpa-mirror :type git :host github
                         :repo "emacs-straight/gnu-elpa-mirror"
                         :build nil))
    (straight-use-recipes `(gnu-elpa :type git
                                     :repo ,straight-recipes-gnu-elpa-url
                                     :local-repo "elpa"
                                     :build nil)))

  (straight-use-recipes '(el-get :type git :host github
                                 :repo "dimitri/el-get"
                                 :build nil))

  (if straight-recipes-emacsmirror-use-mirror
      (straight-use-recipes
       '(emacsmirror-mirror :type git :host github
                            :repo "emacs-straight/emacsmirror-mirror"
                            :build nil))
    (straight-use-recipes '(emacsmirror :type git :host github
                                        :repo "emacsmirror/epkgs"
                                        :nonrecursive t
                                        :build nil)))

  ;; Prefer newly included straight and use-package
  (straight-override-recipe `(straight :type built-in))
  (straight-override-recipe `(use-package :type built-in))

  (if (straight--modifications 'check-on-save)
      (straight-live-modifications-mode +1)
    (straight-live-modifications-mode -1))

  (when (straight--modifications 'watch-files)
    (straight-watcher-start))

  (if straight-use-symlinks
      (straight-symlink-emulation-mode -1)
    (straight-symlink-emulation-mode +1))

  (if straight-enable-package-integration
      (straight-package-neutering-mode +1)
    (straight-package-neutering-mode -1))

  (if straight-enable-use-package-integration
      (straight-use-package-mode +1)
    (straight-use-package-mode -1)))

(when (fboundp 'lsp-make-connection)
 (eval-after-load 'ng-lsp
   '(progn
      (require 'ng-lsp-mode))))

(provide 'emacs-ng)
