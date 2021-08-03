# Tests

## Running tests

Run elisp and Rust tests in top level directory. If run in a
subdirectory, only run the tests in that directory.

    make check Run all tests as defined in the directory. Expensive tests are suppressed. The result of the tests for .el is stored in .log.

    make check-maybe Like "make check", but run only the tests for files that have been modified since the last build.

## Writing tests

### Elisp

For elisp testing, remacs uses ert.

Add new tests to test/rust_src/src/<filename>-tests.el. There are good
examples in the directory to follow. In general, there should be at
least one test function for each Rust function. This function should
be a 'smoke' test. Does the Rust call succeed for common values? Does
it fail for common values? More complex tests or tests that involve
several lisp functions should be defined in a function named after
what the test is testing.

As an example here is how the `if` function is tested:

```elisp
(ert-deftest eval-tests--if-base ()
  "Check (if) base cases"
  (should-error (eval '(if)) :type 'wrong-number-of-arguments)
  (should (eq (if t 'a) 'a))
  (should (eq (if t 'a 'b) 'a))
  (should (eq (if nil 'a) nil))
  (should (eq (if nil 'a 'b) 'b))
  (should (eq (if t 'a (error "Not evaluated!")) 'a))
  (should (eq (if nil (error "Not evaluated!") 'a) 'a)))

(ert-deftest eval-tests--if-dot-string ()
  "Check that Emacs rejects (if . \"string\")."
  (should-error (eval '(if . "abc")) :type 'wrong-type-argument)
  (let ((if-tail (list '(setcdr if-tail "abc") t)))
    (should-error (eval (cons 'if if-tail))))
  (let ((if-tail (list '(progn (setcdr if-tail "abc") nil) t)))
    (should-error (eval (cons 'if if-tail)))))
```

### Rust

```rust
#[cfg(test)]
use std::cmp::max;

#[test]
fn test_lisp_float_size() { let double_size =
    mem::size_of::<EmacsDouble>(); let ptr_size =
    mem::size_of::<*const Lisp_Float>();

    assert!(mem::size_of::<Lisp_Float>() == max(double_size,
ptr_size)); }
```

## Running code formatters

### Rust

Run `cargo fmt` in the `rust_src/` folder and format all crates in the workspace.

```sh
make rustfmt

```
