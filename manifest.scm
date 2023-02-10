(concatenate-manifests
 (list (specifications->manifest
        (list "direnv"
	      "git"
	      "git:send-email"
	      "git-cal"
	      "gnupg"

	      ;;x11 deps
	      "libxcursor" "libxrandr" "libxi" "xorg-server-xwayland" "xcb-util"

	      "tree-sitter"

	      "emacs-rustic"
	      "emacs-realgud"
	      "gsettings-desktop-schemas"

	      "rust@nightly-2022-10-24:rust-docs"
	      "rust@nightly-2022-10-24:rust-docs-json-preview"
	      "rust@nightly-2022-10-24:clippy-preview"
	      "rust@nightly-2022-10-24:rust-analyzer-preview"
	      "rust@nightly-2022-10-24:rust-demangler-preview"
	      "rust@nightly-2022-10-24:rustfmt-preview"
	      "rust-cbindgen"
	      "clang"
	      ;; "gcc-toolchain"
	      "gdb"
	      "valgrind"
	      "strace"
	      "glibc:debug"))
       (package->development-manifest
        (specification->package "emacs-next-wr"))))
