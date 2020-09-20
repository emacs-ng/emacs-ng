//! Generic Lisp eval functions

// use std::unreachable;

// use libc::c_void;

// use remacs_macros::lisp_fn;

// use crate::{
//     alloc::purecopy,
//     data::{
//         defalias, default_boundp, fset, indirect_function, indirect_function_lisp, set, set_default,
//     },
//     lisp::is_autoload,
//     lisp::{LispObject, LispSubrRef, SpecbindingRef},
//     lists::{assq, car, cdr, get, memq, nth, put},
//     lists::{LispCons, LispConsCircularChecks, LispConsEndChecks},
//     multibyte::LispStringRef,
//     obarray::loadhist_attach,
//     objects::equal,
//     remacs_sys::specbind_tag,
//     remacs_sys::{
//         backtrace_debug_on_exit, build_string, call_debugger, check_cons_list, do_debug_on_call,
//         do_one_unbind, eval_sub, funcall_lambda, funcall_subr, globals, grow_specpdl,
//         internal_catch, internal_lisp_condition_case, list2, maybe_gc, maybe_quit,
//         record_in_backtrace, record_unwind_save_match_data, signal_or_quit, specbind, COMPILEDP,
//         MODULE_FUNCTIONP,
//     },
//     remacs_sys::{pvec_type, EmacsInt, Lisp_Compiled, Set_Internal_Bind},
//     remacs_sys::{Fapply, Fdefault_value, Fload},
//     remacs_sys::{
//         QCdocumentation, Qautoload, Qclosure, Qerror, Qexit, Qfunction, Qinteractive,
//         Qinteractive_form, Qinternal_interpreter_environment, Qinvalid_function, Qlambda, Qmacro,
//         Qnil, Qrisky_local_variable, Qsetq, Qt, Qunbound, Qvariable_documentation, Qvoid_function,
//     },
//     remacs_sys::{Vautoload_queue, Vrun_hooks},
//     symbols::{fboundp, symbol_function, LispSymbolRef},
//     threads::{c_specpdl_index, ThreadState},
//     vectors::length,
// };


// /* * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * *
//  *   NOTE!!! Every function that can call EVAL must protect its args   *
//  *   and temporaries from garbage collection while it needs them.      *
//  * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * */
// #[no_mangle]
// pub unsafe extern "C" fn record_unwind_protect(
//     function: Option<unsafe extern "C" fn(LispObject)>,
//     arg: LispObject,
// ) {
//     let unwind = (*ThreadState::current_thread().m_specpdl_ptr)
//         .unwind
//         .as_mut();
//     unwind.set_kind(specbind_tag::SPECPDL_UNWIND);
//     unwind.func = function;
//     unwind.arg = arg;
//     grow_specpdl();
// }

// #[no_mangle]
// pub unsafe extern "C" fn record_unwind_protect_ptr(
//     function: Option<unsafe extern "C" fn(*mut c_void)>,
//     arg: *mut c_void,
// ) {
//     let unwind = (*ThreadState::current_thread().m_specpdl_ptr)
//         .unwind_ptr
//         .as_mut();
//     unwind.set_kind(specbind_tag::SPECPDL_UNWIND_PTR);
//     unwind.func = function;
//     unwind.arg = arg;
//     grow_specpdl();
// }

// #[no_mangle]
// pub unsafe extern "C" fn record_unwind_protect_int(
//     function: Option<unsafe extern "C" fn(i32)>,
//     arg: i32,
// ) {
//     let unwind = (*ThreadState::current_thread().m_specpdl_ptr)
//         .unwind_int
//         .as_mut();
//     unwind.set_kind(specbind_tag::SPECPDL_UNWIND_INT);
//     unwind.func = function;
//     unwind.arg = arg;
//     grow_specpdl();
// }

// #[no_mangle]
// pub unsafe extern "C" fn record_unwind_protect_void(function: Option<unsafe extern "C" fn()>) {
//     let unwind = (*ThreadState::current_thread().m_specpdl_ptr)
//         .unwind_void
//         .as_mut();
//     unwind.set_kind(specbind_tag::SPECPDL_UNWIND_VOID);
//     unwind.func = function;
//     grow_specpdl();
// }

// /// Eval args until one of them yields non-nil, then return that value.
// /// The remaining args are not evalled at all.
// /// If all args return nil, return nil.
// /// usage: (or CONDITIONS...)
// #[lisp_fn(min = "0", unevalled = "true")]
// pub fn or(args: LispObject) -> LispObject {
//     eval_and_compare_all(args, Qnil, |a, b| a != b)
// }

// /// Eval args until one of them yields nil, then return nil.
// /// The remaining args are not evalled at all.
// /// If no arg yields nil, return the last arg's value.
// /// usage: (and CONDITIONS...)
// #[lisp_fn(min = "0", unevalled = "true")]
// pub fn and(args: LispObject) -> LispObject {
//     eval_and_compare_all(args, Qt, |a, b| a == b)
// }

// /// Eval each item in ARGS and then compare it using CMP.
// /// INITIAL is returned if the list has no cons cells.
// fn eval_and_compare_all(
//     args: LispObject,
//     initial: LispObject,
//     cmp: impl Fn(LispObject, LispObject) -> bool,
// ) -> LispObject {
//     let mut val = initial;

//     for elt in args.iter_cars(LispConsEndChecks::off, LispConsCircularChecks::off) {
//         val = unsafe { eval_sub(elt) };
//         if cmp(val, Qnil) {
//             break;
//         }
//     }

//     val
// }

// /// If COND yields non-nil, do THEN, else do ELSE...
// /// Returns the value of THEN or the value of the last of the ELSE's.
// /// THEN must be one expression, but ELSE... can be zero or more expressions.
// /// If COND yields nil, and there are no ELSE's, the value is nil.
// /// usage: (if COND THEN ELSE...)
// #[lisp_fn(name = "if", c_name = "if", min = "2", unevalled = "true")]
// pub fn lisp_if(args: LispCons) -> LispObject {
//     let (cond, consq) = args.into();
//     let (then, else_) = consq.into();
//     let result = unsafe { eval_sub(cond) };

//     if result.is_not_nil() {
//         unsafe { eval_sub(then) }
//     } else {
//         progn(else_)
//     }
// }

// /// Try each clause until one succeeds.
// /// Each clause looks like (CONDITION BODY...).  CONDITION is evaluated
// /// and, if the value is non-nil, this clause succeeds:
// /// then the expressions in BODY are evaluated and the last one's
// /// value is the value of the cond-form.
// /// If a clause has one element, as in (CONDITION), then the cond-form
// /// returns CONDITION's value, if that is non-nil.
// /// If no clause succeeds, cond returns nil.
// /// usage: (cond CLAUSES...)
// #[lisp_fn(min = "0", unevalled = "true")]
// pub fn cond(args: LispObject) -> LispObject {
//     let mut val = Qnil;

//     for clause in args.iter_cars(LispConsEndChecks::off, LispConsCircularChecks::off) {
//         let (head, tail) = clause.into();
//         val = unsafe { eval_sub(head) };
//         if val.is_not_nil() {
//             if tail.is_not_nil() {
//                 val = progn(tail);
//             }
//             break;
//         }
//     }

//     val
// }

// /// Eval BODY forms sequentially and return value of last one.
// /// usage: (progn BODY...)
// #[lisp_fn(min = "0", unevalled = "true")]
// pub fn progn(body: LispObject) -> LispObject {
//     body.iter_cars(LispConsEndChecks::off, LispConsCircularChecks::off)
//         .map(|form| unsafe { eval_sub(form) })
//         .last()
//         .into()
// }

// /// Evaluate BODY sequentially, discarding its value.
// #[no_mangle]
// pub extern "C" fn prog_ignore(body: LispObject) {
//     progn(body);
// }

// /// Eval FIRST and BODY sequentially; return value from FIRST.
// /// The value of FIRST is saved during the evaluation of the remaining args,
// /// whose values are discarded.
// /// usage: (prog1 FIRST BODY...)
// #[lisp_fn(min = "1", unevalled = "true")]
// pub fn prog1(args: LispCons) -> LispObject {
//     let (first, body) = args.into();

//     let val = unsafe { eval_sub(first) };
//     progn(body);
//     val
// }

// /// Eval FORM1, FORM2 and BODY sequentially; return value from FORM2.
// /// The value of FORM2 is saved during the evaluation of the
// /// remaining args, whose values are discarded.
// /// usage: (prog2 FORM1 FORM2 BODY...)
// #[lisp_fn(min = "2", unevalled = "true")]
// pub fn prog2(args: LispCons) -> LispObject {
//     let (form1, tail) = args.into();

//     unsafe { eval_sub(form1) };
//     prog1(tail.into())
// }

// /// Set each SYM to the value of its VAL.
// /// The symbols SYM are variables; they are literal (not evaluated).
// /// The values VAL are expressions; they are evaluated.
// /// Thus, (setq x (1+ y)) sets `x' to the value of `(1+ y)'.
// /// The second VAL is not computed until after the first SYM is set, and so on;
// /// each VAL can use the new value of variables set earlier in the `setq'.
// /// The return value of the `setq' form is the value of the last VAL.
// /// usage: (setq [SYM VAL]...)
// #[lisp_fn(min = "0", unevalled = "true")]
// pub fn setq(args: LispObject) -> LispObject {
//     let mut val = args;

//     let mut it = args
//         .iter_cars(LispConsEndChecks::off, LispConsCircularChecks::off)
//         .enumerate();
//     while let Some((nargs, sym)) = it.next() {
//         let (_, arg) = it.next().unwrap_or_else(|| {
//             wrong_number_of_arguments!(Qsetq, nargs + 1);
//         });

//         val = unsafe { eval_sub(arg) };

//         let mut lexical = false;

//         // Like for eval_sub, we do not check declared_special here since
//         // it's been done when let-binding.
//         // N.B. the check against nil is a mere optimization!
//         if unsafe { globals.Vinternal_interpreter_environment.is_not_nil() } && sym.is_symbol() {
//             let binding = assq(sym, unsafe { globals.Vinternal_interpreter_environment });
//             if let Some(binding) = binding.as_cons() {
//                 lexical = true;
//                 binding.set_cdr(val); /* SYM is lexically bound. */
//             }
//         }

//         if !lexical {
//             set(sym.into(), val); /* SYM is dynamically bound. */
//         }
//     }

//     val
// }
// def_lisp_sym!(Qsetq, "setq");

// /// Like `quote', but preferred for objects which are functions.
// /// In byte compilation, `function' causes its argument to be compiled.
// /// `quote' cannot do that.
// /// usage: (function ARG)
// #[lisp_fn(min = "1", unevalled = "true")]
// pub fn function(args: LispCons) -> LispObject {
//     let (quoted, tail) = args.into();

//     if tail.is_not_nil() {
//         wrong_number_of_arguments!(Qfunction, length(args.into()));
//     }

//     if unsafe { globals.Vinternal_interpreter_environment.is_not_nil() } {
//         if let Some((first, mut cdr)) = quoted.into() {
//             if first.eq(Qlambda) {
//                 // This is a lambda expression within a lexical environment;
//                 // return an interpreted closure instead of a simple lambda.

//                 let tmp = cdr
//                     .as_cons()
//                     .and_then(|c| c.cdr().as_cons())
//                     .and_then(|c| c.car().as_cons());
//                 if let Some(cell) = tmp {
//                     let (typ, tail) = cell.into();
//                     if typ.eq(QCdocumentation) {
//                         // Handle the special (:documentation <form>) to build the docstring
//                         // dynamically.

//                         let docstring: LispStringRef = unsafe { eval_sub(car(tail)) }.into();
//                         let (a, b) = cdr.into();
//                         let (_, bd) = b.into();
//                         cdr = (a, (docstring, bd)).into();
//                     }
//                 }

//                 return unsafe {
//                     (Qclosure, (globals.Vinternal_interpreter_environment, cdr)).into()
//                 };
//             }
//         }
//     }

//     // Simply quote the argument.
//     quoted
// }
// def_lisp_sym!(Qfunction, "function");

// /// Make SYMBOL lexically scoped.
// /// Internal function
// #[lisp_fn(name = "internal-make-var-non-special")]
// pub fn make_var_non_special(symbol: LispSymbolRef) -> bool {
//     symbol.set_declared_special(false);
//     true
// }

// /// Return non-nil if SYMBOL's global binding has been declared special.
// /// A special variable is one that will be bound dynamically, even in a
// /// context where binding is lexical by default.
// #[lisp_fn]
// pub fn special_variable_p(symbol: LispSymbolRef) -> bool {
//     symbol.get_declared_special()
// }

// /// Define SYMBOL as a constant variable.
// /// This declares that neither programs nor users should ever change the
// /// value.  This constancy is not actually enforced by Emacs Lisp, but
// /// SYMBOL is marked as a special variable so that it is never lexically
// /// bound.
// ///
// /// The `defconst' form always sets the value of SYMBOL to the result of
// /// evalling INITVALUE.  If SYMBOL is buffer-local, its default value is
// /// what is set; buffer-local values are not affected.  If SYMBOL has a
// /// local binding, then this form sets the local binding's value.
// /// However, you should normally not make local bindings for variables
// /// defined with this form.
// ///
// /// The optional DOCSTRING specifies the variable's documentation string.
// /// usage: (defconst SYMBOL INITVALUE [DOCSTRING])
// #[lisp_fn(min = "2", unevalled = "true")]
// pub fn defconst(args: LispCons) -> LispSymbolRef {
//     let (sym, tail) = args.into();

//     let mut docstring = if cdr(tail).is_not_nil() {
//         if cdr(cdr(tail)).is_not_nil() {
//             error!("Too many arguments");
//         }

//         car(cdr(tail))
//     } else {
//         Qnil
//     };

//     let mut tem = unsafe { eval_sub(car(tail)) };
//     if unsafe { globals.Vpurify_flag }.is_not_nil() {
//         tem = purecopy(tem);
//     }
//     let sym_ref: LispSymbolRef = sym.into();
//     set_default(sym_ref, tem);
//     sym_ref.set_declared_special(true);
//     if docstring.is_not_nil() {
//         if unsafe { globals.Vpurify_flag }.is_not_nil() {
//             docstring = purecopy(docstring);
//         }

//         put(sym_ref, Qvariable_documentation, docstring);
//     }

//     put(sym_ref, Qrisky_local_variable, Qt);
//     loadhist_attach(sym);

//     sym_ref
// }

// // Common code from let and letX.
// // This transforms a binding defined in Lisp into the variable and evaluated value
// // or just the symbol if only a symbol is provided.
// // ((a 1)) -> (a, 1)
// // ((a)) -> (a, nil)
// // ((a (* 5 (+ 2 1)))) -> (a, 15)
// fn let_binding_value(obj: LispObject) -> (LispObject, LispObject) {
//     if obj.is_symbol() {
//         (obj, Qnil)
//     } else {
//         let (front, tail) = obj.into();
//         let (to_eval, tail) = if tail.is_nil() {
//             (Qnil, tail)
//         } else {
//             tail.into()
//         };

//         if tail.is_nil() {
//             (front, unsafe { eval_sub(to_eval) })
//         } else {
//             signal_error("`let' bindings can have only one value-form", obj);
//         }
//     }
// }

// /// Bind variables according to VARLIST then eval BODY.
// /// The value of the last form in BODY is returned.
// /// Each element of VARLIST is a symbol (which is bound to nil)
// /// or a list (SYMBOL VALUEFORM) (which binds SYMBOL to the value of VALUEFORM).
// /// Each VALUEFORM can refer to the symbols already bound by this VARLIST.
// /// usage: (let* VARLIST BODY...)
// #[lisp_fn(name = "let*", min = "1", unevalled = "true")]
// pub fn letX(args: LispCons) -> LispObject {
//     let count = c_specpdl_index();
//     let (varlist, body) = args.into();

//     let lexenv = unsafe { globals.Vinternal_interpreter_environment };

//     for var in varlist.iter_cars(LispConsEndChecks::on, LispConsCircularChecks::off) {
//         unsafe { maybe_quit() };

//         let (var, val) = let_binding_value(var);

//         let mut needs_bind = true;

//         if lexenv.is_not_nil() {
//             if let Some(sym) = var.as_symbol() {
//                 if !sym.get_declared_special() {
//                     let bound = memq(var, unsafe { globals.Vinternal_interpreter_environment })
//                         .is_not_nil();

//                     if !bound {
//                         // Lexically bind VAR by adding it to the interpreter's binding alist.

//                         unsafe {
//                             let newenv =
//                                 ((var, val), globals.Vinternal_interpreter_environment).into();

//                             if globals.Vinternal_interpreter_environment == lexenv {
//                                 // Save the old lexical environment on the specpdl stack,
//                                 // but only for the first lexical binding, since we'll never
//                                 // need to revert to one of the intermediate ones.
//                                 specbind(Qinternal_interpreter_environment, newenv);
//                             } else {
//                                 globals.Vinternal_interpreter_environment = newenv;
//                             }
//                         }

//                         needs_bind = false;
//                     }
//                 }
//             }
//         }

//         // handles both lexenv is nil and the question of already lexically bound
//         if needs_bind {
//             unsafe { specbind(var, val) };
//         }
//     }

//     // The symbols are bound. Now evaluate the body
//     let val = progn(body);

//     unbind_to(count, val)
// }

// /// Bind variables according to VARLIST then eval BODY.
// /// The value of the last form in BODY is returned.
// /// Each element of VARLIST is a symbol (which is bound to nil)
// /// or a list (SYMBOL VALUEFORM) (which binds SYMBOL to the value of VALUEFORM).
// /// All the VALUEFORMs are evalled before any symbols are bound.
// /// usage: (let VARLIST BODY...)
// #[lisp_fn(name = "let", c_name = "let", min = "1", unevalled = "true")]
// pub fn lisp_let(args: LispCons) -> LispObject {
//     let count = c_specpdl_index();
//     let (varlist, body) = args.into();

//     varlist.check_list();

//     let mut lexenv = unsafe { globals.Vinternal_interpreter_environment };

//     for var in varlist.iter_cars(LispConsEndChecks::off, LispConsCircularChecks::off) {
//         let (var, val) = let_binding_value(var);

//         let mut dyn_bind = true;

//         if lexenv.is_not_nil() {
//             if let Some(sym) = var.as_symbol() {
//                 if !sym.get_declared_special() {
//                     let bound = memq(var, unsafe { globals.Vinternal_interpreter_environment })
//                         .is_not_nil();

//                     if !bound {
//                         // Lexically bind VAR by adding it to the lexenv alist.
//                         lexenv = ((var, val), lexenv).into();
//                         dyn_bind = false;
//                     }
//                 }
//             }
//         }

//         // handles both lexenv is nil and the question of already lexically bound
//         if dyn_bind {
//             // Dynamically bind VAR.
//             unsafe { specbind(var, val) };
//         }
//     }

//     unsafe {
//         if lexenv != globals.Vinternal_interpreter_environment {
//             // Instantiate a new lexical environment.
//             specbind(Qinternal_interpreter_environment, lexenv);
//         }
//     }

//     // The symbols are bound. Now evaluate the body
//     let val = progn(body);

//     unbind_to(count, val)
// }

// /// If TEST yields non-nil, eval BODY... and repeat.
// /// The order of execution is thus TEST, BODY, TEST, BODY and so on
// /// until TEST returns nil.
// /// usage: (while TEST BODY...)
// #[lisp_fn(name = "while", c_name = "while", min = "1", unevalled = "true")]
// pub fn lisp_while(args: LispCons) {
//     let (test, body) = args.into();

//     while unsafe { eval_sub(test) }.is_not_nil() {
//         unsafe { maybe_quit() };

//         prog_ignore(body);
//     }
// }

// /// Return result of expanding macros at top level of FORM.
// /// If FORM is not a macro call, it is returned unchanged.
// /// Otherwise, the macro is expanded and the expansion is considered
// /// in place of FORM.  When a non-macro-call results, it is returned.
// ///
// /// The second optional arg ENVIRONMENT specifies an environment of macro
// /// definitions to shadow the loaded ones for use in file byte-compilation.
// #[lisp_fn(min = "1")]
// pub fn macroexpand(mut form: LispObject, environment: LispObject) -> LispObject {
//     while let Some((mut sym, body)) = form.into() {
//         // Come back here each time we expand a macro call,
//         // in case it expands into another macro call.

//         // Set SYM, give DEF and TEM right values in case SYM is not a symbol.
//         let mut def = sym;
//         let mut tem = Qnil;

//         // Trace symbols aliases to other symbols
//         // until we get a symbol that is not an alias.
//         while let Some(sym_ref) = def.as_symbol() {
//             unsafe { maybe_quit() };
//             sym = def;
//             tem = assq(sym, environment);
//             if tem.is_nil() {
//                 def = sym_ref.get_function();
//                 if def.is_not_nil() {
//                     continue;
//                 }
//             }
//             break;
//         }

//         // Right now TEM is the result from SYM in ENVIRONMENT,
//         // and if TEM is nil then DEF is SYM's function definition.
//         let expander = if tem.is_nil() {
//             // SYM is not mentioned in ENVIRONMENT.
//             // Look at its function definition.
//             def = autoload_do_load(def, sym, Qmacro);
//             match def.into() {
//                 Some((func, cdr)) => {
//                     if !func.eq(Qmacro) {
//                         break;
//                     }
//                     cdr
//                 }
//                 None => {
//                     // Not defined or definition not suitable.
//                     break;
//                 }
//             }
//         } else {
//             let (_, next) = tem.into();
//             if next.is_nil() {
//                 break;
//             }
//             next
//         };

//         let newform = apply1(expander, body);
//         if form.eq(newform) {
//             break;
//         } else {
//             form = newform;
//         }
//     }

//     form
// }

// /// Evaluate FORM and return its value.
// /// If LEXICAL is t, evaluate using lexical scoping.
// /// LEXICAL can also be an actual lexical environment, in the form of an
// /// alist mapping symbols to their value.
// #[lisp_fn(min = "1")]
// pub fn eval(form: LispObject, lexical: LispObject) -> LispObject {
//     let count = c_specpdl_index();
//     let value = if lexical.is_list() {
//         lexical
//     } else {
//         list!(Qt)
//     };

//     unsafe {
//         specbind(Qinternal_interpreter_environment, value);
//     }

//     unbind_to(count, unsafe { eval_sub(form) })
// }

// /// Apply fn to arg.
// #[no_mangle]
// pub extern "C" fn apply1(func: LispObject, arg: LispObject) -> LispObject {
//     if arg.is_nil() {
//         call!(func)
//     } else {
//         callN_raw!(Fapply, func, arg)
//     }
// }

// /// Signal `error' with message MSG, and additional arg ARG.
// /// If ARG is not a genuine list, make it a one-element list.
// fn signal_error(msg: &str, arg: LispObject) -> ! {
//     let it = arg.iter_tails(LispConsEndChecks::off, LispConsCircularChecks::safe);
//     let arg = match it.last() {
//         None => list!(arg),
//         Some(_) => arg,
//     };

//     xsignal!(
//         Qerror,
//         (build_string(msg.as_ptr() as *const libc::c_char), arg)
//     );
// }

// /// Non-nil if FUNCTION makes provisions for interactive calling.
// /// This means it contains a description for how to read arguments to give it.
// /// The value is nil for an invalid function or a symbol with no function
// /// definition.
// ///
// /// Interactively callable functions include strings and vectors (treated
// /// as keyboard macros), lambda-expressions that contain a top-level call
// /// to `interactive', autoload definitions made by `autoload' with non-nil
// /// fourth argument, and some of the built-in functions of Lisp.
// ///
// /// Also, a symbol satisfies `commandp' if its function definition does so.
// ///
// /// If the optional argument FOR-CALL-INTERACTIVELY is non-nil,
// /// then strings and vectors are not accepted.
// #[lisp_fn(min = "1")]
// pub fn commandp(function: LispObject, for_call_interactively: bool) -> bool {
//     let mut has_interactive_prop = false;

//     let mut fun = indirect_function(function); // Check cycles.
//     if fun.is_nil() {
//         return false;
//     }

//     // Check an `interactive-form' property if present, analogous to the
//     // function-documentation property.
//     while let Some(sym) = fun.as_symbol() {
//         let tmp = get(sym, Qinteractive_form);
//         if tmp.is_not_nil() {
//             has_interactive_prop = true;
//         }
//         fun = symbol_function(sym);
//     }

//     if let Some(subr) = fun.as_subr() {
//         // Emacs primitives are interactive if their DEFUN specifies an
//         // interactive spec.
//         return !subr.intspec.is_null() || has_interactive_prop;
//     } else if fun.is_string() || fun.is_vector() {
//         // Strings and vectors are keyboard macros.
//         // This check has to occur before the vectorlike check or vectors
//         // will be identified incorrectly.
//         return !for_call_interactively;
//     } else if let Some(vl) = fun.as_vectorlike() {
//         // Bytecode objects are interactive if they are long enough to
//         // have an element whose index is COMPILED_INTERACTIVE, which is
//         // where the interactive spec is stored.
//         return (vl.is_pseudovector(pvec_type::PVEC_COMPILED)
//             && vl.pseudovector_size() > EmacsInt::from(Lisp_Compiled::COMPILED_INTERACTIVE))
//             || has_interactive_prop;
//     } else if let Some((funcar, d)) = fun.into() {
//         // Lists may represent commands.
//         if funcar.eq(Qclosure) {
//             let bound = assq(Qinteractive, cdr(cdr(d)));
//             return bound.is_not_nil() || has_interactive_prop;
//         } else if funcar.eq(Qlambda) {
//             let bound = assq(Qinteractive, cdr(d));
//             return bound.is_not_nil() || has_interactive_prop;
//         } else if funcar.eq(Qautoload) {
//             let value = car(cdr(cdr(d)));
//             return value.is_not_nil() || has_interactive_prop;
//         }
//     }

//     false
// }

// def_lisp_sym!(Qcommandp, "commandp");

// /// Define FUNCTION to autoload from FILE.
// /// FUNCTION is a symbol; FILE is a file name string to pass to `load'.
// /// Third arg DOCSTRING is documentation for the function.
// /// Fourth arg INTERACTIVE if non-nil says function can be called interactively.
// /// Fifth arg TYPE indicates the type of the object:
// ///    nil or omitted says FUNCTION is a function,
// ///    `keymap' says FUNCTION is really a keymap, and
// ///    `macro' or t says FUNCTION is really a macro.
// /// Third through fifth args give info about the real definition.
// /// They default to nil.
// /// If FUNCTION is already defined other than as an autoload,
// /// this does nothing and returns nil.
// #[lisp_fn(min = "2")]
// pub fn autoload(
//     function: LispSymbolRef,
//     file: LispStringRef,
//     mut docstring: LispObject,
//     interactive: LispObject,
//     ty: LispObject,
// ) -> LispObject {
//     // If function is defined and not as an autoload, don't override.
//     if function.get_function().is_not_nil() && !is_autoload(function.get_function()) {
//         return Qnil;
//     }

//     if unsafe { globals.Vpurify_flag.is_not_nil() } && docstring.eq(0) {
//         // `read1' in lread.c has found the docstring starting with "\
//         // and assumed the docstring will be provided by Snarf-documentation, so it
//         // passed us 0 instead.  But that leads to accidental sharing in purecopy's
//         // hash-consing, so we use a (hopefully) unique integer instead.
//         docstring = unsafe { LispObject::from(function).to_fixnum_unchecked() }.into();
//     }

//     defalias(
//         function,
//         list!(Qautoload, file, docstring, interactive, ty),
//         Qnil,
//     )
// }

// def_lisp_sym!(Qautoload, "autoload");

// /// Return t if OBJECT is a function.
// #[lisp_fn(name = "functionp", c_name = "functionp")]
// pub fn functionp_lisp(object: LispObject) -> bool {
//     FUNCTIONP(object)
// }

// #[no_mangle]
// pub extern "C" fn FUNCTIONP(object: LispObject) -> bool {
//     let mut obj = object;

//     if let Some(sym) = obj.as_symbol() {
//         if fboundp(sym) {
//             obj = sym.get_indirect_function();

//             if let Some(cons) = obj.as_cons() {
//                 if cons.car().eq(Qautoload) {
//                     // Autoloaded symbols are functions, except if they load
//                     // macros or keymaps.
//                     let mut it =
//                         cons.iter_tails(LispConsEndChecks::off, LispConsCircularChecks::off);
//                     for _ in 0..4 {
//                         if it.next().is_none() {
//                             break;
//                         }
//                     }

//                     return match it.rest().into() {
//                         None => true,
//                         Some((a, _)) => a.is_nil(),
//                     };
//                 }
//             }
//         }
//     }

//     if let Some(subr) = obj.as_subr() {
//         !subr.is_unevalled()
//     } else if obj.is_byte_code_function() || obj.is_module_function() {
//         true
//     } else if let Some((car, _)) = obj.into() {
//         car.eq(Qlambda) || car.eq(Qclosure)
//     } else {
//         false
//     }
// }

// pub unsafe extern "C" fn un_autoload(oldqueue: LispObject) {
//     // Queue to unwind is current value of Vautoload_queue.
//     // oldqueue is the shadowed value to leave in Vautoload_queue.
//     let queue = Vautoload_queue;
//     Vautoload_queue = oldqueue;

//     for first in queue.iter_cars(LispConsEndChecks::off, LispConsCircularChecks::off) {
//         let (first, second) = first.into();

//         if first.eq(0) {
//             globals.Vfeatures = second;
//         } else {
//             fset(first.into(), second);
//         }
//     }
// }

// // Load an autoloaded function.
// // FUNNAME is the symbol which is the function's name.
// // FUNDEF is the autoload definition (a list).

// /// Load FUNDEF which should be an autoload.
// /// If non-nil, FUNNAME should be the symbol whose function value is FUNDEF,
// /// in which case the function returns the new autoloaded function value.
// /// If equal to `macro', MACRO-ONLY specifies that FUNDEF should only be loaded if
// /// it defines a macro.
// #[lisp_fn(min = "1")]
// pub fn autoload_do_load(
//     fundef: LispObject,
//     funname: LispObject,
//     macro_only: LispObject,
// ) -> LispObject {
//     let count = c_specpdl_index();

//     if !(fundef.is_cons() && car(fundef).eq(Qautoload)) {
//         return fundef;
//     }

//     let kind = nth(4, fundef);
//     if macro_only.eq(Qmacro) && !(kind.eq(Qt) || kind.eq(Qmacro)) {
//         return fundef;
//     }

//     let sym: LispSymbolRef = funname.into();

//     unsafe {
//         // This is to make sure that loadup.el gives a clear picture
//         // of what files are preloaded and when.
//         if globals.Vpurify_flag.is_not_nil() {
//             error!(
//                 "Attempt to autoload {} while preparing to dump",
//                 sym.symbol_name()
//             );
//         }

//         // Preserve the match data.
//         record_unwind_save_match_data();

//         // If autoloading gets an error (which includes the error of failing
//         // to define the function being called), we use Vautoload_queue
//         // to undo function definitions and `provide' calls made by
//         // the function.  We do this in the specific case of autoloading
//         // because autoloading is not an explicit request "load this file",
//         // but rather a request to "call this function".
//         //
//         // The value saved here is to be restored into Vautoload_queue.

//         record_unwind_protect(Some(un_autoload), Vautoload_queue);
//         Vautoload_queue = Qt;
//     }

//     // If `macro_only' is set and fundef isn't a macro, assume this autoload to
//     // be a "best-effort" (e.g. to try and find a compiler macro),
//     // so don't signal an error if autoloading fails.
//     let ignore_errors = if kind.eq(Qt) || kind.eq(Qmacro) {
//         Qnil
//     } else {
//         macro_only
//     };

//     unsafe {
//         Fload(car(cdr(fundef)), ignore_errors, Qt, Qnil, Qt);

//         // Once loading finishes, don't undo it.
//         Vautoload_queue = Qt;
//     }

//     unbind_to(count, Qnil);

//     if funname.is_nil() || ignore_errors.is_not_nil() {
//         Qnil
//     } else {
//         let fun = indirect_function_lisp(funname, Qnil);

//         if equal(fun, fundef) {
//             error!(
//                 "Autoloading file {} failed to define function {}",
//                 car(car(unsafe { globals.Vload_history })),
//                 sym.symbol_name()
//             );
//         } else {
//             fun
//         }
//     }
// }

// /// Run each hook in HOOKS.
// /// Each argument should be a symbol, a hook variable.
// /// These symbols are processed in the order specified.
// /// If a hook symbol has a non-nil value, that value may be a function
// /// or a list of functions to be called to run the hook.
// /// If the value is a function, it is called with no arguments.
// /// If it is a list, the elements are called, in order, with no arguments.
// ///
// /// Major modes should not use this function directly to run their mode
// /// hook; they should use `run-mode-hooks' instead.
// ///
// /// Do not use `make-local-variable' to make a hook variable buffer-local.
// /// Instead, use `add-hook' and specify t for the LOCAL argument.
// /// usage: (run-hooks &rest HOOKS)
// #[lisp_fn]
// pub fn run_hooks(args: &[LispObject]) {
//     for item in args {
//         run_hook(*item);
//     }
// }

// /// Run HOOK with the specified arguments ARGS.
// /// HOOK should be a symbol, a hook variable.  The value of HOOK
// /// may be nil, a function, or a list of functions.  Call each
// /// function in order with arguments ARGS.  The final return value
// /// is unspecified.
// ///
// /// Do not use `make-local-variable' to make a hook variable buffer-local.
// /// Instead, use `add-hook' and specify t for the LOCAL argument.
// /// usage: (run-hook-with-args HOOK &rest ARGS)
// #[lisp_fn(min = "1")]
// pub fn run_hook_with_args(args: &mut [LispObject]) -> LispObject {
//     run_hook_with_args_internal(args, funcall_nil)
// }

// fn funcall_nil(args: &mut [LispObject]) -> LispObject {
//     funcall(args);
//     Qnil
// }

// // NB this one still documents a specific non-nil return value.  (As
// // did run-hook-with-args and run-hook-with-args-until-failure until
// // they were changed in 24.1.)

// /// Run HOOK with the specified arguments ARGS.
// /// HOOK should be a symbol, a hook variable.  The value of HOOK
// /// may be nil, a function, or a list of functions.  Call each
// /// function in order with arguments ARGS, stopping at the first
// /// one that returns non-nil, and return that value.  Otherwise (if
// /// all functions return nil, or if there are no functions to call),
// /// return nil.
// ///
// /// Do not use `make-local-variable' to make a hook variable buffer-local.
// /// Instead, use `add-hook' and specify t for the LOCAL argument.
// /// usage: (run-hook-with-args-until-success HOOK &rest ARGS)
// #[lisp_fn(min = "1")]
// pub fn run_hook_with_args_until_success(args: &mut [LispObject]) -> LispObject {
//     run_hook_with_args_internal(args, funcall)
// }

// fn funcall_not(args: &mut [LispObject]) -> LispObject {
//     funcall(args).is_nil().into()
// }

// /// Run HOOK with the specified arguments ARGS.
// /// HOOK should be a symbol, a hook variable.  The value of HOOK may
// /// be nil, a function, or a list of functions.  Call each function in
// /// order with arguments ARGS, stopping at the first one that returns
// /// nil, and return nil.  Otherwise (if all functions return non-nil,
// /// or if there are no functions to call), return non-nil (do not rely
// /// on the precise return value in this case).
// ///
// /// Do not use `make-local-variable' to make a hook variable buffer-local.
// /// Instead, use `add-hook' and specify t for the LOCAL argument.
// /// usage: (run-hook-with-args-until-failure HOOK &rest ARGS)
// #[lisp_fn(min = "1")]
// pub fn run_hook_with_args_until_failure(args: &mut [LispObject]) -> bool {
//     run_hook_with_args_internal(args, funcall_not).is_nil()
// }

// fn run_hook_wrapped_funcall(args: &mut [LispObject]) -> LispObject {
//     args.swap(0, 1);
//     let ret = funcall(args);
//     args.swap(0, 1);
//     ret
// }

// /// Run HOOK, passing each function through WRAP-FUNCTION.
// /// I.e. instead of calling each function FUN directly with arguments
// /// ARGS, it calls WRAP-FUNCTION with arguments FUN and ARGS.
// ///
// /// As soon as a call to WRAP-FUNCTION returns non-nil,
// /// `run-hook-wrapped' aborts and returns that value.
// /// usage: (run-hook-wrapped HOOK WRAP-FUNCTION &rest ARGS)
// #[lisp_fn(min = "2")]
// pub fn run_hook_wrapped(args: &mut [LispObject]) -> LispObject {
//     run_hook_with_args_internal(args, run_hook_wrapped_funcall)
// }

// /// Run the hook HOOK, giving each function no args.
// #[no_mangle]
// pub extern "C" fn run_hook(hook: LispObject) {
//     run_hook_with_args(&mut [hook]);
// }

// /// ARGS[0] should be a hook symbol.
// /// Call each of the functions in the hook value, passing each of them
// /// as arguments all the rest of ARGS (all NARGS - 1 elements).
// /// FUNCALL specifies how to call each function on the hook.
// fn run_hook_with_args_internal(
//     args: &mut [LispObject],
//     func: fn(&mut [LispObject]) -> LispObject,
// ) -> LispObject {
//     // If we are dying or still initializing,
//     // don't do anything -- it would probably crash if we tried.
//     if unsafe { Vrun_hooks.is_nil() } {
//         return Qnil;
//     }

//     let mut ret = Qnil;
//     let sym = args[0];
//     let val = unsafe { LispSymbolRef::from(sym).find_value() };

//     if val.eq(Qunbound) || val.is_nil() {
//         Qnil
//     } else if !val.is_cons() || FUNCTIONP(val) {
//         args[0] = val;
//         func(args)
//     } else {
//         for item in val.iter_cars(LispConsEndChecks::off, LispConsCircularChecks::off) {
//             if ret.is_not_nil() {
//                 break;
//             }

//             if item.eq(Qt) {
//                 // t indicates this hook has a local binding;
//                 // it means to run the global binding too.
//                 let global_vals = unsafe { Fdefault_value(sym) };
//                 if global_vals.is_nil() {
//                     continue;
//                 }

//                 if !global_vals.is_cons() || car(global_vals).eq(Qlambda) {
//                     args[0] = global_vals;
//                     ret = func(args);
//                 } else {
//                     for gval in
//                         global_vals.iter_cars(LispConsEndChecks::off, LispConsCircularChecks::off)
//                     {
//                         if ret.is_not_nil() {
//                             break;
//                         }

//                         args[0] = gval;
//                         // In a global value, t should not occur. If it does, we
//                         // must ignore it to avoid an endless loop.
//                         if !args[0].eq(Qt) {
//                             ret = func(args);
//                         }
//                     }
//                 }
//             } else {
//                 args[0] = item;
//                 ret = func(args);
//             }
//         }

//         ret
//     }
// }

// enum LispFun {
//     SubrFun(LispSubrRef),
//     LambdaFun(LispObject),
// }

// enum LispFunError {
//     InvalidFun,
//     VoidFun,
// }

// fn resolve_fun(fun: LispObject) -> Result<LispFun, LispFunError> {
//     let original_fun = fun;

//     loop {
//         if original_fun.is_nil() {
//             return Err(LispFunError::VoidFun);
//         }

//         // Optimize for no indirection.
//         let fun = original_fun
//             .as_symbol()
//             .map_or_else(|| original_fun, LispSymbolRef::get_indirect_function);

//         if fun.is_nil() {
//             return Err(LispFunError::VoidFun);
//         }

//         if let Some(f) = fun.as_subr() {
//             return Ok(LispFun::SubrFun(f));
//         }

//         if unsafe { COMPILEDP(fun) || MODULE_FUNCTIONP(fun) } {
//             return Ok(LispFun::LambdaFun(fun));
//         }

//         if let Some((funcar, _)) = fun.into() {
//             if !funcar.is_symbol() {
//                 return Err(LispFunError::InvalidFun);
//             }

//             if funcar.eq(Qlambda) || funcar.eq(Qclosure) {
//                 return Ok(LispFun::LambdaFun(fun));
//             }

//             if funcar.eq(Qautoload) {
//                 unsafe {Fautoload_do_load(fun, original_fun, Qnil)} ;
//                 unsafe { check_cons_list() };
//                 continue;
//             }

//             return Err(LispFunError::InvalidFun);
//         }

//         return Err(LispFunError::InvalidFun);
//     }
// }

// /// Call first argument as a function, passing remaining arguments to it.
// /// Return the value that function returns.
// /// Thus, (funcall \\='cons \\='x \\='y) returns (x . y).
// /// usage: (funcall FUNCTION &rest ARGUMENTS)
// #[allow(unused_assignments)]
// #[lisp_fn(min = "1")]
// pub fn funcall(args: &mut [LispObject]) -> LispObject {
//     unsafe { maybe_quit() };

//     // Increment the lisp eval depth
//     let mut current_thread = ThreadState::current_thread();
//     current_thread.m_lisp_eval_depth += 1;

//     unsafe {
//         if current_thread.m_lisp_eval_depth > globals.max_lisp_eval_depth {
//             if globals.max_lisp_eval_depth < 100 {
//                 globals.max_lisp_eval_depth = 100;
//             }

//             if current_thread.m_lisp_eval_depth > globals.max_lisp_eval_depth {
//                 error!("Lisp nesting exceeds `max-lisp-eval-depth'");
//             }
//         }
//     }

//     // The first element in args is the called function.
//     let numargs = args.len() as isize - 1;

//     let fun = args[0];

//     let fun_args = if numargs > 0 {
//         &mut args[1]
//     } else {
//         ptr::null_mut()
//     };

//     let count = unsafe { record_in_backtrace(fun, fun_args, numargs) };

//     unsafe { maybe_gc() };

//     unsafe {
//         if globals.debug_on_next_call {
//             do_debug_on_call(Qlambda, count);
//         }
//     }

//     unsafe { check_cons_list() };

//     let mut val = Qnil;

//     match resolve_fun(fun) {
//         Ok(LispFun::SubrFun(mut f)) => {
//             val = unsafe { funcall_subr(f.as_mut(), numargs, fun_args) };
//         }
//         Ok(LispFun::LambdaFun(f)) => {
//             val = unsafe { funcall_lambda(f, numargs, fun_args) };
//         }
//         Err(LispFunError::InvalidFun) => {
//             xsignal!(Qinvalid_function, fun);
//         }
//         Err(LispFunError::VoidFun) => {
//             xsignal!(Qvoid_function, fun);
//         }
//     }

//     unsafe { check_cons_list() };

//     current_thread.m_lisp_eval_depth -= 1;

//     unsafe {
//         if backtrace_debug_on_exit(current_thread.m_specpdl.offset(count)) {
//             val = call_debugger(list2(Qexit, val));
//         }

//         current_thread.m_specpdl_ptr = current_thread.m_specpdl_ptr.offset(-1);
//     }

//     val
// }

// /// Pop and execute entries from the unwind-protect stack until the
// /// depth COUNT is reached. Return VALUE.
// #[no_mangle]
// pub extern "C" fn unbind_to(count: libc::ptrdiff_t, value: LispObject) -> LispObject {
//     let mut current_thread = ThreadState::current_thread();

//     unsafe {
//         let quitf = globals.Vquit_flag;

//         globals.Vquit_flag = Qnil;

//         while current_thread.m_specpdl_ptr != current_thread.m_specpdl.offset(count) {
//             // Copy the binding, and decrement specpdl_ptr, before we
//             // do the work to unbind it.  We decrement first so that
//             // an error in unbinding won't try to unbind the same
//             // entry again, and we copy the binding first in case more
//             // bindings are made during some of the code we run.

//             current_thread.m_specpdl_ptr = current_thread.m_specpdl_ptr.offset(-1);

//             let this_binding = current_thread.m_specpdl_ptr;

//             do_one_unbind(this_binding, true, Set_Internal_Bind::SET_INTERNAL_UNBIND);
//         }

//         if globals.Vquit_flag.is_nil() && quitf.is_not_nil() {
//             globals.Vquit_flag = quitf;
//         }
//     }

//     value
// }

// /// Do BODYFORM, protecting with UNWINDFORMS.
// /// If BODYFORM completes normally, its value is returned
// /// after executing the UNWINDFORMS.
// /// If BODYFORM exits nonlocally, the UNWINDFORMS are executed anyway.
// /// usage: (unwind-protect BODYFORM UNWINDFORMS...)
// #[lisp_fn(min = "1", unevalled = "true")]
// pub fn unwind_protect(args: LispCons) -> LispObject {
//     let (bodyform, unwindforms) = args.into();
//     let count = c_specpdl_index();

//     unsafe { record_unwind_protect(Some(prog_ignore), unwindforms) };

//     unbind_to(count, unsafe { eval_sub(bodyform) })
// }

// /// Eval BODY allowing nonlocal exits using `throw'.
// /// TAG is evalled to get the tag to use; it must not be nil.
// /// Then the BODY is executed.
// /// Within BODY, a call to `throw' with the same TAG exits BODY and this `catch'.
// /// If no throw happens, `catch' returns the value of the last BODY form.
// /// If a throw happens, it specifies the value to return from `catch'.
// /// usage: (catch TAG BODY...)
// #[lisp_fn(min = "1", unevalled = "true")]
// pub fn catch(args: LispCons) -> LispObject {
//     let (tag, body) = args.into();

//     let val = unsafe { eval_sub(tag) };

//     unsafe { internal_catch(val, Some(Fprogn), body) }
// }

// /// Signal an error.  Args are ERROR-SYMBOL and associated DATA. This
// /// function does not return.
// ///
// /// An error symbol is a symbol with an `error-conditions' property
// /// that is a list of condition names.  A handler for any of those
// /// names will get to handle this signal.  The symbol `error' should
// /// normally be one of them.
// ///
// /// DATA should be a list.  Its elements are printed as part of the
// /// error message.  See Info anchor `(elisp)Definition of signal' for
// /// some details on how this error message is constructed.
// /// If the signal is handled, DATA is made available to the handler.
// /// See also the function `condition-case'.
// #[lisp_fn]
// pub fn signal_rust(error_symbol: LispObject, data: LispObject) -> ! {
//     #[cfg(test)]
//     {
//         panic!("Fsignal called during tests.");
//     }
//     #[cfg(not(test))]
//     {
//         unsafe { signal_or_quit(error_symbol, data, false) };
//         unreachable!();
//     }
// }

// /// Regain control when an error is signaled.
// /// Executes BODYFORM and returns its value if no error happens.
// /// Each element of HANDLERS looks like (CONDITION-NAME BODY...)
// /// where the BODY is made of Lisp expressions.
// ///
// /// A handler is applicable to an error
// /// if CONDITION-NAME is one of the error's condition names.
// /// If an error happens, the first applicable handler is run.
// ///
// /// The car of a handler may be a list of condition names instead of a
// /// single condition name; then it handles all of them.  If the special
// /// condition name `debug' is present in this list, it allows another
// /// condition in the list to run the debugger if `debug-on-error' and the
// /// other usual mechanisms says it should (otherwise, `condition-case'
// /// suppresses the debugger).
// ///
// /// When a handler handles an error, control returns to the `condition-case'
// /// and it executes the handler's BODY...
// /// with VAR bound to (ERROR-SYMBOL . SIGNAL-DATA) from the error.
// /// \(If VAR is nil, the handler can't access that information.)
// /// Then the value of the last BODY form is returned from the `condition-case'
// /// expression.
// ///
// /// See also the function `signal' for more info.
// /// usage: (condition-case VAR BODYFORM &rest HANDLERS)
// #[lisp_fn(min = "2", unevalled = "true")]
// pub fn condition_case(args: LispCons) -> LispObject {
//     let (var, consq) = args.into();
//     let (bodyform, handlers) = consq.into();
//     unsafe { internal_lisp_condition_case(var, bodyform, handlers) }
// }

// #[no_mangle]
// pub extern "C" fn specpdl_symbol(pdl: SpecbindingRef) -> LispObject {
//     pdl.symbol().into()
// }

// #[no_mangle]
// pub extern "C" fn specpdl_old_value(pdl: SpecbindingRef) -> LispObject {
//     pdl.old_value()
// }

// #[no_mangle]
// pub extern "C" fn set_specpdl_old_value(mut pdl: SpecbindingRef, val: LispObject) {
//     pdl.set_old_value(val)
// }

// #[no_mangle]
// pub extern "C" fn default_toplevel_binding(symbol: LispObject) -> SpecbindingRef {
//     LispSymbolRef::from(symbol).default_toplevel_binding_rust()
// }

// /// Define SYMBOL as a variable, and return SYMBOL.
// /// You are not required to define a variable in order to use it, but
// /// defining it lets you supply an initial value and documentation, which
// /// can be referred to by the Emacs help facilities and other programming
// /// tools.  The `defvar' form also declares the variable as \"special\",
// /// so that it is always dynamically bound even if `lexical-binding' is t.
// ///
// /// If SYMBOL's value is void and the optional argument INITVALUE is
// /// provided, INITVALUE is evaluated and the result used to set SYMBOL's
// /// value.  If SYMBOL is buffer-local, its default value is what is set;
// /// buffer-local values are not affected.  If INITVALUE is missing,
// /// SYMBOL's value is not set.
// ///
// /// If SYMBOL has a local binding, then this form affects the local
// /// binding.  This is usually not what you want.  Thus, if you need to
// /// load a file defining variables, with this form or with `defconst' or
// /// `defcustom', you should always load that file _outside_ any bindings
// /// for these variables.  (`defconst' and `defcustom' behave similarly in
// /// this respect.)
// ///
// /// The optional argument DOCSTRING is a documentation string for the
// /// variable.
// ///
// /// To define a user option, use `defcustom' instead of `defvar'.
// /// usage: (defvar SYMBOL &optional INITVALUE DOCSTRING)
// #[lisp_fn(min = "1", unevalled = "true")]
// pub fn defvar(args: LispCons) -> LispObject {
//     let (sym_obj, tail) = args.into();

//     if let Some(tail) = tail.as_cons() {
//         if tail.length() > 2 {
//             error!("Too many arguments");
//         }

//         let sym: LispSymbolRef = sym_obj.into();
//         let has_default = default_boundp(sym);

//         // Do it before evaluating the initial value, for self-references.
//         sym.set_declared_special(true);

//         if has_default {
//             // Check if there is really a global binding rather than just a let
//             // binding that shadows the global unboundness of the var.
//             let mut binding = sym.default_toplevel_binding_rust();
//             if !binding.is_null() && (binding.old_value() == Qunbound) {
//                 binding.set_old_value(unsafe { eval_sub(tail.car()) });
//             }
//         } else {
//             set_default(sym, unsafe { eval_sub(tail.car()) });
//         }

//         let mut documentation = car(tail.cdr());

//         if documentation.is_not_nil() {
//             if unsafe { globals.Vpurify_flag }.is_not_nil() {
//                 documentation = purecopy(documentation);
//             }
//             put(sym, Qvariable_documentation, documentation);
//         }
//         loadhist_attach(sym_obj);
//     } else if unsafe { globals.Vinternal_interpreter_environment }.is_not_nil()
//         && sym_obj
//             .as_symbol()
//             .map_or(false, |x| !x.get_declared_special())
//     {
//         // A simple (defvar foo) with lexical scoping does "nothing" except
//         // declare that var to be dynamically scoped *locally* (i.e. within
//         // the current file or let-block).
//         unsafe {
//             globals.Vinternal_interpreter_environment =
//                 LispObject::cons(sym_obj, globals.Vinternal_interpreter_environment);
//         }
//     } else {
//         // Simple (defvar <var>) should not count as a definition at all.
//         // It could get in the way of other definitions, and unloading this
//         // package could try to make the variable unbound.
//     }

//     sym_obj
// }

// include!(concat!(env!("OUT_DIR"), "/eval_exports.rs"));
