;;; emacs-ng-init.el --- Emacs NG init -*- lexical-binding: t -*-

(require 'emacs-ng)

;;;
(and ng-straight-bootstrap-at-startup
     (not (bound-and-true-p ng--straight-bootstrapped))
     (ng-bootstrap-straight))
