# Built-in Emacs Lisp packages

emacs-ng distributed with more built-in Emacs Lisp packages than
upstream GNU Emacs.  For now, we have
[straight.el](https://github.com/raxod502/straight.el/tree/develop)
`08b0ecf` and [use-package](https://github.com/jwiegley/use-package)
`a7422fb` included.

## `straight.el`

We are kind of using `straight.el` as an alternative of `package.el`,
providing `ng-straight-bootstrap-at-startup`, `ng-bootstrap-straight`
(as equipment of `package-enable-at-startup` and
`package-initialize`).

### Usage

Emacs NG automatically bootstraps `straight.el` at startup if
`ng-straight-bootstrap-at-startup` is set to `t`, the default value is
`nil`

You must configure the built-in `straight.el` in the early init file,
as the variable `ng-straight-bootstrap-at-startup` is read before
loading the regular init file. There are some variables you may be
interested in (some of them must be set **before** the bootstrap
process, if they might affect how `straight.el` itself is loaded). You
can find the details from [this
section](https://github.com/raxod502/straight.el/tree/develop#getting-started)
of the `straight.el`'s documentation.

To be compatible with upstream Emacs, you can place the following in
your init-file:

```
(unless (fboundp 'ng-bootstrap-straight)
  (defvar bootstrap-version)
  (let ((bootstrap-file
         (expand-file-name "straight/repos/straight.el/bootstrap.el" user-emacs-directory))
        (bootstrap-version 5))
    (unless (file-exists-p bootstrap-file)
      (with-current-buffer
          (url-retrieve-synchronously
           "https://raw.githubusercontent.com/raxod502/straight.el/develop/install.el"
           'silent 'inhibit-cookies)
        (goto-char (point-max))
        (eval-print-last-sexp)))
    (load bootstrap-file nil 'nomessage)))
```

For more detailed guide, please refer [straight.el's
READMD.md](https://github.com/raxod502/straight.el/blob/develop/README.md).

## `use-package`

There are also
[discussions/efforts](https://github.com/jwiegley/use-package/issues/282)
to include `use-package` into upstream. Until then, we temporally
included it with `emacs-ng`.
