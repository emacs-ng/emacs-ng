;;; typescript-ts-mode.el --- tree sitter support for TypeScript  -*- lexical-binding: t; -*-

;; Copyright (C) 2022-2025 Free Software Foundation, Inc.

;; Author     : Theodor Thornhill <theo@thornhill.no>
;; Maintainer : Theodor Thornhill <theo@thornhill.no>
;; Created    : October 2022
;; Keywords   : typescript tsx languages tree-sitter

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

;;; Tree-sitter language versions
;;
;; typescript-ts-mode is known to work with the following languages and version:
;; - tree-sitter-typescript: v0.23.2-2-g8e13e1d
;;
;; We try our best to make builtin modes work with latest grammar
;; versions, so a more recent grammar version has a good chance to work.
;; Send us a bug report if it doesn't.

;;; Commentary:
;;

;;; Code:

(require 'treesit)
(require 'js)
(eval-when-compile (require 'rx))
(require 'c-ts-common) ; For comment indent and filling.
(treesit-declare-unavailable-functions)

(defcustom typescript-ts-mode-indent-offset 2
  "Number of spaces for each indentation step in `typescript-ts-mode'."
  :version "29.1"
  :type 'integer
  :safe 'integerp
  :group 'typescript)

(defface typescript-ts-jsx-tag-face
  '((t . (:inherit font-lock-function-call-face)))
  "Face for HTML tags like <div> and <p> in JSX."
  :group 'typescript)

(defface typescript-ts-jsx-attribute-face
  '((t . (:inherit font-lock-constant-face)))
  "Face for HTML attributes like name and id in JSX."
  :group 'typescript)

(defvar typescript-ts-mode--syntax-table
  (let ((table (make-syntax-table)))
    ;; Taken from the cc-langs version
    (modify-syntax-entry ?_  "_"     table)
    (modify-syntax-entry ?\\ "\\"    table)
    (modify-syntax-entry ?+  "."     table)
    (modify-syntax-entry ?-  "."     table)
    (modify-syntax-entry ?=  "."     table)
    (modify-syntax-entry ?%  "."     table)
    (modify-syntax-entry ?<  "."     table)
    (modify-syntax-entry ?>  "."     table)
    (modify-syntax-entry ?&  "."     table)
    (modify-syntax-entry ?|  "."     table)
    (modify-syntax-entry ?\' "\""    table)
    (modify-syntax-entry ?\240 "."   table)
    (modify-syntax-entry ?/  ". 124b" table)
    (modify-syntax-entry ?*  ". 23"   table)
    (modify-syntax-entry ?\n "> b"  table)
    (modify-syntax-entry ?\^m "> b" table)
    (modify-syntax-entry ?$ "_" table)
    (modify-syntax-entry ?` "\"" table)
    table)
  "Syntax table for `typescript-ts-mode'.")

(define-error 'typescript-ts-mode-wrong-dialect-error
              "Wrong typescript dialect"
              'error)

(defun typescript-ts-mode--check-dialect (dialect)
  (unless (or (eq dialect 'typescript) (eq dialect 'tsx))
    (signal 'typescript-ts-mode-wrong-dialect-error
            (list "Unsupported dialect for typescript-ts-mode supplied" dialect))))

(defun tsx-ts-mode--indent-compatibility-b893426 ()
  "Indent rules helper, to handle different releases of tree-sitter-tsx.
Check if a node type is available, then return the right indent rules."
  ;; handle https://github.com/tree-sitter/tree-sitter-typescript/commit/b893426b82492e59388a326b824a346d829487e8
  (condition-case nil
      (progn (treesit-query-capture 'tsx '((jsx_fragment) @capture))
             `(((match "<" "jsx_fragment") parent 0)
               ((parent-is "jsx_fragment") parent typescript-ts-mode-indent-offset)))
    (treesit-query-error
     `(((match "<" "jsx_text") parent 0)
       ((parent-is "jsx_text") parent typescript-ts-mode-indent-offset)))))

(defun typescript-ts-mode--anchor-decl (_n parent &rest _)
  "Return the position after the declaration keyword before PARENT.

This anchor allows aligning variable_declarators in variable and lexical
declarations, accounting for the length of keyword (var, let, or const)."
  (let* ((declaration (treesit-parent-until
                       parent (rx (or "variable" "lexical") "_declaration") t))
         (decl (treesit-node-child declaration 0)))
    (+ (treesit-node-start declaration)
       (- (treesit-node-end decl) (treesit-node-start decl)))))

(defun typescript-ts-mode--indent-rules (language)
  "Rules used for indentation.
Argument LANGUAGE is either `typescript' or `tsx'."
  (typescript-ts-mode--check-dialect language)
  `((,language
     ((parent-is "program") column-0 0)
     ((node-is "}") parent-bol 0)
     ((node-is ")") parent-bol 0)
     ((node-is "]") parent-bol 0)
     ((node-is ">") parent-bol 0)
     ((and (parent-is "comment") c-ts-common-looking-at-star)
      c-ts-common-comment-start-after-first-star -1)
     ((parent-is "comment") prev-adaptive-prefix 0)
     ((parent-is "ternary_expression") parent-bol typescript-ts-mode-indent-offset)
     ((parent-is "member_expression") parent-bol typescript-ts-mode-indent-offset)
     ((parent-is "named_imports") parent-bol typescript-ts-mode-indent-offset)
     ((parent-is "statement_block") parent-bol typescript-ts-mode-indent-offset)
     ((or (node-is "case")
          (node-is "default"))
      parent-bol typescript-ts-mode-indent-offset)
     ((parent-is "switch_case") parent-bol typescript-ts-mode-indent-offset)
     ((parent-is "switch_default") parent-bol typescript-ts-mode-indent-offset)
     ((parent-is "type_arguments") parent-bol typescript-ts-mode-indent-offset)
     ((parent-is "type_parameters") parent-bol typescript-ts-mode-indent-offset)
     ((parent-is ,(rx (or "variable" "lexical") "_" (or "declaration" "declarator")))
      typescript-ts-mode--anchor-decl 1)
     ((parent-is "arguments") parent-bol typescript-ts-mode-indent-offset)
     ((parent-is "array") parent-bol typescript-ts-mode-indent-offset)
     ((parent-is "formal_parameters") parent-bol typescript-ts-mode-indent-offset)
     ((parent-is "template_string") no-indent) ; Don't indent the string contents.
     ((parent-is "template_substitution") parent-bol typescript-ts-mode-indent-offset)
     ((parent-is "object_pattern") parent-bol typescript-ts-mode-indent-offset)
     ((parent-is "object") parent-bol typescript-ts-mode-indent-offset)
     ((parent-is "object_type") parent-bol typescript-ts-mode-indent-offset)
     ((parent-is "enum_body") parent-bol typescript-ts-mode-indent-offset)
     ((parent-is "class_body") parent-bol typescript-ts-mode-indent-offset)
     ((parent-is "interface_body") parent-bol typescript-ts-mode-indent-offset)
     ((parent-is "arrow_function") parent-bol typescript-ts-mode-indent-offset)
     ((parent-is "parenthesized_expression") parent-bol typescript-ts-mode-indent-offset)
     ((parent-is "binary_expression") parent-bol typescript-ts-mode-indent-offset)
     ((match "while" "do_statement") parent-bol 0)
     ((match "else" "if_statement") parent-bol 0)
     ((parent-is ,(rx (or (seq (or "if" "for" "for_in" "while" "do") "_statement")
                          "else_clause")))
      parent-bol typescript-ts-mode-indent-offset)

     ,@(when (eq language 'tsx)
	 (append (tsx-ts-mode--indent-compatibility-b893426)
		 `(((node-is "jsx_closing_element") parent 0)
		   ((match "jsx_element" "statement") parent typescript-ts-mode-indent-offset)
		   ((parent-is "jsx_element") parent typescript-ts-mode-indent-offset)
		   ((parent-is "jsx_text") parent-bol typescript-ts-mode-indent-offset)
		   ((parent-is "jsx_opening_element") parent typescript-ts-mode-indent-offset)
		   ((parent-is "jsx_expression") parent-bol typescript-ts-mode-indent-offset)
		   ((match "/" "jsx_self_closing_element") parent 0)
		   ((parent-is "jsx_self_closing_element") parent typescript-ts-mode-indent-offset))))
     ;; FIXME(Theo): This no-node catch-all should be removed.  When is it needed?
     (no-node parent-bol 0))))

(defvar typescript-ts-mode--keywords
  '("!" "abstract" "as" "async" "await" "break"
    "case" "catch" "class" "const" "continue" "debugger"
    "declare" "default" "delete" "do" "else" "enum"
    "export" "extends" "finally" "for" "from" "function"
    "get" "if" "implements" "import" "in" "instanceof" "interface" "is" "infer"
    "keyof" "let" "namespace" "new" "of" "private" "protected"
    "public" "readonly" "return" "satisfies" "set" "static" "switch"
    "target" "throw" "try" "type" "typeof" "var" "void"
    "while" "with" "yield")
  "TypeScript keywords for tree-sitter font-locking.")

(defvar typescript-ts-mode--operators
  '("=" "+=" "-=" "*=" "/=" "%=" "**=" "<<=" ">>=" ">>>=" "&=" "^="
    "|=" "&&=" "||=" "??=" "==" "!=" "===" "!==" ">" ">=" "<" "<=" "+"
    "-" "*" "/" "%" "++" "--" "**" "&" "|" "^" "~" "<<" ">>" ">>>"
    "&&" "||" "!" "?.")
  "TypeScript operators for tree-sitter font-locking.")

(defun tsx-ts-mode--font-lock-compatibility-bb1f97b (language)
  "Font lock rules helper, to handle different releases of tree-sitter-tsx.
Check if a node type is available, then return the right font lock rules.
Argument LANGUAGE is either `typescript' or `tsx'."
  ;; handle commit bb1f97b
  ;; Warning: treesitter-query-capture says both node types are valid,
  ;; but then raises an error if the wrong node type is used. So it is
  ;; important to check with the new node type (member_expression)
  ;;
  ;; Later typescript grammar removed support for jsx, so the later
  ;; grammar versions this function just return nil.
  (typescript-ts-mode--check-dialect language)
  (let ((queries-a '((jsx_opening_element
		      [(member_expression (identifier)) (identifier)]
		      @typescript-ts-jsx-tag-face)

	             (jsx_closing_element
		      [(member_expression (identifier)) (identifier)]
		      @typescript-ts-jsx-tag-face)

	             (jsx_self_closing_element
		      [(member_expression (identifier)) (identifier)]
		      @typescript-ts-jsx-tag-face)

                     (jsx_attribute (property_identifier)
                                    @typescript-ts-jsx-attribute-face)))
        (queries-b '((jsx_opening_element
	              [(nested_identifier (identifier)) (identifier)]
	              @typescript-ts-jsx-tag-face)

                     (jsx_closing_element
	              [(nested_identifier (identifier)) (identifier)]
	              @typescript-ts-jsx-tag-face)

                     (jsx_self_closing_element
	              [(nested_identifier (identifier)) (identifier)]
	              @typescript-ts-jsx-tag-face)

                     (jsx_attribute (property_identifier)
                                    @typescript-ts-jsx-attribute-face))))
    (or (ignore-errors
          (treesit-query-compile language queries-a t)
          queries-a)
        (ignore-errors
          (treesit-query-compile language queries-b t)
          queries-b)
        ;; Return a dummy query that doens't do anything, if neither
        ;; query works.
        '("," @_ignore))))

(defun tsx-ts-mode--font-lock-compatibility-function-expression (language)
  "Handle tree-sitter grammar breaking change for `function' expression.

LANGUAGE can be `typescript' or `tsx'.  Starting from version 0.20.4 of the
typescript/tsx grammar, `function' becomes `function_expression'."
  (typescript-ts-mode--check-dialect language)
  (condition-case nil
      (progn (treesit-query-capture language '((function_expression) @cap))
             ;; New version of the grammar
             'function_expression)
    (treesit-query-error
    ;; Old version of the grammar
    'function)))

(defun typescript-ts-mode--font-lock-settings (language)
  "Tree-sitter font-lock settings.
Argument LANGUAGE is either `typescript' or `tsx'."
  (typescript-ts-mode--check-dialect language)
  (let ((func-exp (tsx-ts-mode--font-lock-compatibility-function-expression language)))
    (treesit-font-lock-rules
     :language language
     :feature 'comment
     `([(comment) (hash_bang_line)] @font-lock-comment-face)

     :language language
     :feature 'constant
     `(((identifier) @font-lock-constant-face
        (:match "\\`[A-Z_][0-9A-Z_]*\\'" @font-lock-constant-face))
       [(true) (false) (null)] @font-lock-constant-face)

     :language language
     :feature 'keyword
     `([,@typescript-ts-mode--keywords] @font-lock-keyword-face
       [(this) (super)] @font-lock-keyword-face)

     :language language
     :feature 'string
     `((regex pattern: (regex_pattern)) @font-lock-regexp-face
       (string) @font-lock-string-face
       (template_string) @js--fontify-template-string
       (template_substitution ["${" "}"] @font-lock-misc-punctuation-face))

     :language language
     :override t ;; for functions assigned to variables
     :feature 'declaration
     `((,func-exp
        name: (identifier) @font-lock-function-name-face)
       (function_declaration
        name: (identifier) @font-lock-function-name-face)
       (function_signature
        name: (identifier) @font-lock-function-name-face)

       (method_definition
        name: (property_identifier) @font-lock-function-name-face)
       (method_signature
        name: (property_identifier) @font-lock-function-name-face)
       (required_parameter (identifier) @font-lock-variable-name-face)
       (optional_parameter (identifier) @font-lock-variable-name-face)

       (variable_declarator
        name: (identifier) @font-lock-function-name-face
        value: [(,func-exp) (arrow_function)])

       (variable_declarator
        name: (identifier) @font-lock-variable-name-face)

       (enum_declaration (identifier) @font-lock-type-face)

       (extends_clause value: (identifier) @font-lock-type-face)
       ;; extends React.Component<T>
       (extends_clause value: (member_expression
                               object: (identifier) @font-lock-type-face
                               property: (property_identifier) @font-lock-type-face))

       (arrow_function
        parameter: (identifier) @font-lock-variable-name-face)

       (variable_declarator
        name: (array_pattern
               (identifier)
               (identifier) @font-lock-function-name-face)
        value: (array (number) (,func-exp)))

       (catch_clause
        parameter: (identifier) @font-lock-variable-name-face)

       ;; full module imports
       (import_clause (identifier) @font-lock-variable-name-face)
       ;; named imports with aliasing
       (import_clause (named_imports (import_specifier
                                      alias: (identifier) @font-lock-variable-name-face)))
       ;; named imports without aliasing
       (import_clause (named_imports (import_specifier
                                      !alias
                                      name: (identifier) @font-lock-variable-name-face)))

       ;; full namespace import (* as alias)
       (import_clause (namespace_import (identifier) @font-lock-variable-name-face)))

     :language language
     :feature 'identifier
     `((nested_type_identifier
        module: (identifier) @font-lock-type-face)

       (type_identifier) @font-lock-type-face

       (predefined_type) @font-lock-type-face

       (new_expression
        constructor: (identifier) @font-lock-type-face)

       (enum_body (property_identifier) @font-lock-type-face)

       (enum_assignment name: (property_identifier) @font-lock-type-face)

       (variable_declarator
        name: (identifier) @font-lock-variable-name-face)

       (for_in_statement
        left: (identifier) @font-lock-variable-name-face)

       (arrow_function
        parameters:
        [(_ (identifier) @font-lock-variable-name-face)
         (_ (_ (identifier) @font-lock-variable-name-face))
         (_ (_ (_ (identifier) @font-lock-variable-name-face)))]))

     :language language
     :feature 'property
     `((property_signature
        name: (property_identifier) @font-lock-property-name-face)
       (public_field_definition
        name: (property_identifier) @font-lock-property-name-face)

       (pair key: (property_identifier) @font-lock-property-use-face)

       ((shorthand_property_identifier) @font-lock-property-use-face))

     :language language
     :feature 'expression
     `((assignment_expression
        left: [(identifier) @font-lock-function-name-face
               (member_expression
                property: (property_identifier) @font-lock-function-name-face)]
        right: [(,func-exp) (arrow_function)]))

     :language language
     :feature 'function
     '((call_expression
        function:
        [(identifier) @font-lock-function-call-face
         (member_expression
          property: (property_identifier) @font-lock-function-call-face)]))

     :language language
     :feature 'pattern
     `((pair_pattern
        key: (property_identifier) @font-lock-property-use-face
        value: [(identifier) @font-lock-variable-name-face
                (assignment_pattern left: (identifier) @font-lock-variable-name-face)])

       (array_pattern (identifier) @font-lock-variable-name-face)

       ((shorthand_property_identifier_pattern) @font-lock-variable-name-face))

     :language language
     :feature 'jsx
     (tsx-ts-mode--font-lock-compatibility-bb1f97b language)

     :language language
     :feature 'number
     `((number) @font-lock-number-face
       ((identifier) @font-lock-number-face
        (:match "\\`\\(?:NaN\\|Infinity\\)\\'" @font-lock-number-face)))

     :language language
     :feature 'operator
     `([,@typescript-ts-mode--operators] @font-lock-operator-face
       (ternary_expression ["?" ":"] @font-lock-operator-face))

     :language language
     :feature 'bracket
     '((["(" ")" "[" "]" "{" "}"]) @font-lock-bracket-face)

     :language language
     :feature 'delimiter
     '((["," "." ";" ":"]) @font-lock-delimiter-face)

     :language language
     :feature 'escape-sequence
     :override t
     '((escape_sequence) @font-lock-escape-face))))

(defvar typescript-ts-mode--sentence-nodes
  '("import_statement"
    "debugger_statement"
    "expression_statement"
    "if_statement"
    "switch_statement"
    "for_statement"
    "for_in_statement"
    "while_statement"
    "do_statement"
    "try_statement"
    "with_statement"
    "break_statement"
    "continue_statement"
    "return_statement"
    "throw_statement"
    "empty_statement"
    "labeled_statement"
    "variable_declaration"
    "lexical_declaration"
    "property_signature")
  "Nodes that designate sentences in TypeScript.
See `treesit-thing-settings' for more information.")

(defvar typescript-ts-mode--sexp-nodes
  '("expression"
    "pattern"
    "array"
    "function"
    "string"
    "escape"
    "template"
    "regex"
    "number"
    "identifier"
    "this"
    "super"
    "true"
    "false"
    "null"
    "undefined"
    "arguments"
    "pair")
  "Nodes that designate sexps in TypeScript.
See `treesit-thing-settings' for more information.")

(defvar typescript-ts-mode--list-nodes
  '("export_clause"
    "named_imports"
    "statement_block"
    "_for_header"
    "switch_body"
    "parenthesized_expression"
    "object"
    "object_pattern"
    "array"
    "array_pattern"
    "string"
    "regex"
    "arguments"
    "class_body"
    "formal_parameters"
    "computed_property_name"
    "decorator_parenthesized_expression"
    "enum_body"
    "parenthesized_type"
    "type_arguments"
    "object_type"
    "type_parameters"
    "tuple_type")
  "Nodes that designate lists in TypeScript.
See `treesit-thing-settings' for more information.")

;;;###autoload
(define-derived-mode typescript-ts-base-mode prog-mode "TypeScript"
  "Generic major mode for editing TypeScript.

This mode is intended to be inherited by concrete major modes."
  :group 'typescript
  :syntax-table typescript-ts-mode--syntax-table

  ;; Comments.
  (c-ts-common-comment-setup)

  ;; Electric
  (setq-local electric-indent-chars
              (append "{}():;,<>/" electric-indent-chars))
  (setq-local electric-layout-rules
	      '((?\; . after) (?\{ . after) (?\} . before)))
  ;; Navigation.
  (setq-local treesit-defun-type-regexp
              (regexp-opt '("class_declaration"
                            "method_definition"
                            "function_declaration"
                            "lexical_declaration")))
  (setq-local treesit-defun-name-function #'js--treesit-defun-name)

  (setq-local treesit-thing-settings
              `((typescript
                 (sexp ,(regexp-opt typescript-ts-mode--sexp-nodes 'symbols))
                 (list ,(regexp-opt typescript-ts-mode--list-nodes
                                         'symbols))
                 (sentence ,(regexp-opt
                             typescript-ts-mode--sentence-nodes 'symbols))
                 (text ,(regexp-opt '("comment"
                                      "template_string")
                                    'symbols)))))

  ;; Imenu (same as in `js-ts-mode').
  (setq-local treesit-simple-imenu-settings
              `(("Function" "\\`function_declaration\\'" nil nil)
                ("Variable" "\\`lexical_declaration\\'"
                 js--treesit-valid-imenu-entry nil)
                ("Class" ,(rx bos (or "class_declaration"
                                      "method_definition")
                              eos)
                 nil nil))))

;;;###autoload
(define-derived-mode typescript-ts-mode typescript-ts-base-mode "TypeScript"
  "Major mode for editing TypeScript."
  :group 'typescript
  :syntax-table typescript-ts-mode--syntax-table

  (when (treesit-ready-p 'typescript)
    (setq treesit-primary-parser (treesit-parser-create 'typescript))

    ;; Indent.
    (setq-local treesit-simple-indent-rules
                (typescript-ts-mode--indent-rules 'typescript))

    ;; Font-lock.
    (setq-local treesit-font-lock-settings
                (typescript-ts-mode--font-lock-settings 'typescript))
    (setq-local treesit-font-lock-feature-list
                '((comment declaration)
                  (keyword string escape-sequence)
                  (constant expression identifier number pattern property)
                  (operator function bracket delimiter)))
    (setq-local syntax-propertize-function #'typescript-ts--syntax-propertize)

    (treesit-major-mode-setup)))

(derived-mode-add-parents 'typescript-ts-mode '(typescript-mode))

(if (treesit-ready-p 'typescript)
    (add-to-list 'auto-mode-alist '("\\.ts\\'" . typescript-ts-mode)))

;;;###autoload
(define-derived-mode tsx-ts-mode typescript-ts-base-mode "TypeScript[TSX]"
  "Major mode for editing TSX and JSX documents.

This major mode defines two additional JSX-specific faces:
`typescript-ts-jsx-attribute-face' and
`typescript-ts-jsx-attribute-face' that are used for HTML tags
and attributes, respectively.

The JSX-specific faces are used when `treesit-font-lock-level' is
at least 3 (which is the default value)."
  :group 'typescript
  :syntax-table typescript-ts-mode--syntax-table

  (when (treesit-ready-p 'tsx)
    (setq treesit-primary-parser (treesit-parser-create 'tsx))

    ;; Comments.
    (setq-local comment-start "// ")
    (setq-local comment-end "")
    (setq-local comment-start-skip (rx (or (seq "/" (+ "/"))
                                           (seq "/" (+ "*")))
                                       (* (syntax whitespace))))
    (setq-local comment-end-skip
                (rx (* (syntax whitespace))
                    (group (or (syntax comment-end)
                               (seq (+ "*") "/")))))

    ;; Indent.
    (setq-local treesit-simple-indent-rules
                (typescript-ts-mode--indent-rules 'tsx))

    (setq-local treesit-thing-settings
                `((tsx
                   (sexp ,(regexp-opt
                           (append typescript-ts-mode--sexp-nodes
                                   '("jsx"))))
                   (list ,(concat "^"
                                       (regexp-opt
                                        (append typescript-ts-mode--list-nodes
                                                '(
                                                  "jsx_element"
                                                  "jsx_self_closing_element"
                                                  "jsx_expression")))
                                       "$"))
                   (sentence ,(regexp-opt
                               (append typescript-ts-mode--sentence-nodes
                                       '("jsx_element"
                                         "jsx_self_closing_element"))))
                   (text ,(regexp-opt '("comment"
                                        "template_string"))
                         'symbols))))

    ;; Font-lock.
    (setq-local treesit-font-lock-settings
                (typescript-ts-mode--font-lock-settings 'tsx))
    (setq-local treesit-font-lock-feature-list
                '((comment declaration)
                  (keyword string escape-sequence)
                  (constant expression identifier jsx number pattern property)
                  (function bracket delimiter)))
    (setq-local syntax-propertize-function #'tsx-ts--syntax-propertize)

    (treesit-major-mode-setup)))

(derived-mode-add-parents 'tsx-ts-mode '(tsx-mode))

(defvar typescript-ts--s-p-query
  (when (treesit-available-p)
    (treesit-query-compile 'typescript
                           '(((regex pattern: (regex_pattern) @regexp))))))

(defvar tsx-ts--s-p-query
  (when (treesit-available-p)
    (treesit-query-compile 'tsx
                           '(((regex pattern: (regex_pattern) @regexp))
                             ((jsx_text) @jsx)
                             ((jsx_opening_element) @jsx)
                             ((jsx_closing_element) @jsx)))))

(defun typescript-ts--syntax-propertize (beg end)
  (let ((captures (treesit-query-capture 'typescript typescript-ts--s-p-query beg end)))
    (tsx-ts--syntax-propertize-captures captures)))

(defun tsx-ts--syntax-propertize (beg end)
  (let ((captures (treesit-query-capture 'tsx tsx-ts--s-p-query beg end)))
    (tsx-ts--syntax-propertize-captures captures)))

(defun tsx-ts--syntax-propertize-captures (captures)
  (pcase-dolist (`(,name . ,node) captures)
    (let ((ns (treesit-node-start node))
          (ne (treesit-node-end node)))
      (pcase-exhaustive name
        ('regexp
         (let ((syntax (string-to-syntax "\"/")))
           (cl-decf ns)
           (cl-incf ne)
           (put-text-property ns (1+ ns) 'syntax-table syntax)
           (put-text-property (1- ne) ne 'syntax-table syntax)))
        ;; We put punctuation syntax on all the balanced pair
        ;; characters so they don't mess up syntax-ppss.  We can't put
        ;; string syntax on the whole thing because a) it doesn't work
        ;; if the text is one character long, and b) it interferes
        ;; forward/backward-sexp.
        ('jsx
         (save-excursion
           (goto-char ns)
           (while (re-search-forward (rx (or "{" "}" "[" "]"
                                             "(" ")" "<" ">"))
                                     ne t)
             (put-text-property
              (match-beginning 0) (match-end 0)
              'syntax-table (string-to-syntax
                             (cond
                              ((equal (match-string 0) "<") "(<")
                              ((equal (match-string 0) ">") ")>")
                              (t ".")))))))))))

(if (treesit-ready-p 'tsx)
    (add-to-list 'auto-mode-alist '("\\.tsx\\'" . tsx-ts-mode)))

(provide 'typescript-ts-mode)

;;; typescript-ts-mode.el ends here
