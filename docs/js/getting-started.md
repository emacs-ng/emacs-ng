# Getting Started

This is an introduction for building applications and hacking on emacs-ng using
JavaScript. emacs-ng supports both TypeScript and JavaScript, so these examples
will feature both languages. This guide uses emacs terminology for key binds,
i.e. C-x means holding down control and pressing x. M-x means holding Alt and
pressing x (unless you are on a system where Escape is the 'Meta' key). C-x C-f
means hold down control, press x, keep holding control, press f.

## Requirements

You will need [Rust installed](https://www.rust-lang.org/en-US/install.html).
The file `rust-toolchain` indicates the version that gets installed.  This
happens automatically, so don't override the toolchain manually.  IMPORTANT:
Whenever the toolchain updates, you have to reinstall rustfmt manually.

You also need a working installation of emacs-ng. You can either
[build it] or install a precompiled binary distribution [from our CI].

[build it]: ./building.md
[from our CI]: ./release.md

## Running JavaScript

Type in the following line and press C-j

```lisp
(eval-js "lisp.print('hello world!')")
```

This will display "hello world" in your display area (along with 'nil' - we will
get to that). This is an anonymous javascript evaluation. Before we go further,
let’s make an environment for working with our new scripting language.

We have multiple ways to run JavaScript. Let us open a new *TypeScript* file by
creating a file name "basic.ts" in our current directory. It will look something
like:

```ts
declare var lisp: any;

let x: string = "Hello TypeScript";
lisp.print(x);
```

Now we go back to \*scratch\* and run

```lisp
(eval-js-file "./basic.ts")
```

This is a relative filepath, and it works off of emacs current working directory
(cwd). If you are in doubt to what emacs cwd is, just run the lisp function
`(pwd)` in the lisp scratchpad

You should see "Hello TypeScript" printed. All of the eval-js* functions return
nil. If you want to use a calculated value from JavaScript in Lisp, you should
use `eval-js-literally`. If you are looking for something like
`eval-expression`, which is normally bound to M-:, you should use
`eval-js-expression`. It accepts the same arguments as `eval-expression`, except
with the first argument being a JavaScript expression, and behaves very
similarly to `eval-expression`. It will also inserts the results into `values`
just like `eval-expression`. `eval-js-literally` and `eval-js-expression` do not
work with TypeScript at this time.

## Iteration

Now that we have our TypeScript file, let us get out of lisp and work purely in
TypeScript. Open "basic.ts" by pressing C-x C-f and open "basic.ts". Press M-x
and enter "eval-ts-buffer". This will evaluate the current contents of your
buffer as typescript. You should see "Hello Typescript" print in your
minibuffer. From now on, this will be our preferred way to iterate.

If you do not want to evaluate the entire buffer, you can press C-space,
highlight a region of code, and press M-x eval-ts-region. Try that with this
code and see what happens:

```ts
let y: string = 3;
```

You will see the following error in your minibuffer:

```bash
TS2322 [ERROR]: Type 'number' is not assignable to type 'string'.
let y: string = 3;
    ^
    at file:///home/user/$anon$lisp$91610855413.ts:1:5TS2322
```

It's important to understand that your TypeScript code is compiled prior to
execution - unlike standard JavaScript which is evaluated until you encounter a
runtime error. Within emacs-ng, that means that your code isn't executed if you
have a type error in TypeScript.

If these typescript examples are taking a long time to evaluate, it's likely due
to the processing power required to compile everything. If you want to turn off
TypeScript typechecking for the remainder of the examples, you can run:

```lisp
(js-cleanup)
(js-initialize :no-check t)
```

This code will cleanup your current JS environment and re-initialize it with
TypeScript type checking disabled. If you do not care about the type checking
that TypeScript offers, or your computer struggles with the cost of compiling,
you can add `(js-initialize :no-check t)` to your init.el.

Let's stop printing to the minibuffer, and instead start pushing our results
into buffers. Let's start by something simple: make a network call and dump the
results into a buffer.

## Buffers

First thing first, let's make our network call. In order to do this, we will use
our built-in [Deno APIs](https://deno.land). Deno implements `fetch`, which looks
something like this:

```ts
declare var lisp: any;

fetch("https://api.github.com/users/denoland")
	.then(response => response.json())
	.catch(e => lisp.print(JSON.stringify(e)));
```

`fetch` is a common API [documented
here](https://developer.mozilla.org/en-US/docs/Web/API/Fetch_API). It returns a
[Promise](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise),
which is a tool we will use to manage async I/O operations. Here, the network
call isn't blocking, and it managed by our Rust runtime called Tokio. The
`.then` is saying that once this promise is resolved, execute the next function
in the chain. We have added a `.catch`, which will be executed if fetch errors.

NOTE: If you have an unhandled toplevel promise rejection, the JavaScript
runtime will RESET. You should always have a toplevel promise handler within any
emacs-ng code. We will see the way we can interface with lisp error handling
further on.

However we want to put this response into a buffer. In order to do this, we will
extend our .then chain, like so

```ts
declare var lisp: any;

fetch("https://api.github.com/users/denoland")
	.then(response => response.json())
	.then((data) => {
		const buffer = lisp.get_buffer_create("TypeScript Buffer");
		lisp.with_current_buffer(buffer, () => lisp.insert(JSON.stringify(data)));
	})
	.catch(e => lisp.print(JSON.stringify(e)));
```

Wait for the network call to resolve, and navigate to "TypeScript Buffer" via
C-x b and typing in "TypeScript Buffer", or pressing C-x C-b and selecting our
buffer from the buffer list.

By now, you may be wondering about this `lisp` object, and how we are
able to get references to lisp objects from JavaScript. Our next
example illustrates this.

## Filewatching

Let's write an async filewatch that logs changes to a directory into a buffer,
with a little extra data. In order to do this, we will use Deno's standard
library. Deno has built in functions like `fetch`, along with a robust standard
library that you need to import. That will look like this:

```ts
declare var lisp: any;

// This will allow us to write
const insertIntoTypeScriptBuffer = (str: string) => {
      const buffer = lisp.get_buffer_create("TypeScript Filewatching");
      lisp.with_current_buffer(buffer, () => lisp.insert(`${str}\n`));
};

async function watch(dir: string) {
    const watcher = Deno.watchFs(dir);
    let i = 0;
    for await (const event of watcher) {
	i += 1;
	if (i > 5) break;
	insertIntoTypeScriptBuffer(JSON.stringify(event));
    }
}

watch('.')
```

This example is built to only record 5 events prior to ending itself. You can
write whatever logic you would like for ending your filewatcher.

Running `touch foo.ts` in your current directory should yield something like the
following in the "TypeScript Filewatching" buffer

```json
{"kind":"create","paths":["/home/user/./foo.ts"]}
{"kind":"modify","paths":["/home/user/./foo.ts"]}
{"kind":"access","paths":["/home/user/./foo.ts"]}
```

[Deno has further documentation on
this](https://deno.land/manual@v1.6.3/examples/file_system_events). Note that
these events can differ per operating system.

Key takeaways here - all of the TypeScript written above is executed on
the Main elisp thread - there are no race conditions with lisp here. Even though
the filewatcher is async, it calls back onto the main thread when it has
data. Multithreaded scripting is possible and will be covered later.

## Modules

Now let's look at our tools for importing code. emacs-ng supports [ES6
modules](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Modules). emacs-ng
does not support node's `require` syntax. See the "Using Deno" section for more
information on modules.

Let's create a submodule for our main program. Create a file named "mod-sub.js":

```js
export function generateRandomNumber() {
	return 4;
}
```

Now in our main file, we can add the following to the top of our file:

```ts
import { generateRandomNumber } from "./mod-sub.js";
declare var lisp: any;

lisp.print(generateRandomNumber());
```

Even though our module is TypeScript, we can still import plain old JavaScript.

It's important to note that ES6 modules supposed to be immutable. What does
that mean? If we were to edit mod-js to include the following:

```js
export function generateRandomNumber() {
	return 4;
}

lisp.print(generateRandomNumber());
```

We see that in addition to exporting a function, we execute code with
side-effects (printing). Those side-effects only happen *once*. If I import
mod-sub multiple times, I will only ever see "4" printed once. Another important
note is that **this rule does not apply to any top-level module you
execute**. Meaning that if you call (eval-js-file "./basic.ts") multiple times,
your code is executed every single time, however your dependencies are only
executed once. This is by design.

If you want to break this, you can append a number to your dependency and use
so-called *dynamic importing*, like so:

```js
declare var lisp: any;

let timestamp: number = Date.now();
const { generateRandomNumber } = await import(`./mod-sub.js#${timestamp}`);

lisp.print(generateRandomNumber());
```

This can be useful if you are a module developer and you want to iterate on your
modules within emacs-ng. It is recommended that you do not ship your modules
using this pattern, as it will not cache results properly and lead to a
sub-optimal user experience. Your imports should aim to not have side effects,
and instead should only export functions or variables to be used by your main
module.

The intended use of *Dynamic Importing* is to allow you to have condition
imports, like so:

```ts
if (myCondition) {
     const { func } = await import('example-mod.js');
     func();
}
```

## Lisp Interaction

The lisp object is magic - it has (almost) all lisp functions defined on it,
including any functions defined in your custom packages. If you can invoke it
via `(funcall ....)`, you can call it via the lisp object if you change `-` for
`_`. For example:

```
(with-current-buffer (get-buffer-create "BUFFER") (lambda () (insert "DATA")))
```

becomes

```ts
lisp.with_current_buffer(lisp.get_buffer_create("BUFFER"), () => lisp.insert("DATA"));
```

We have implementations of common macros like `with-current-buffer`. If you find
that a certain common macro doesn't work, you can report it to the project's
maintainers and they will implement it, however what they are doing isn't
magic - they are just calling eval on your whatever macro you want to invoke
from JavaScript.

```ts
lisp.eval(lisp.list(lisp.symbols.with_current_buffer, arg1, arg2));
```

You can override the behavior of the lisp object via the special object
`specialForms`, which looks like

```ts
lisp.specialForms.with_current_buffer = myWithCurrentBufferFunction;
```

This overrides JavaScript's implementation of with_current_buffer without
touching lisp's implementation. Let's discuss working with lisp more:

## Lisp Primitives

Primitives (Number, String, Boolean) are automatically translated when calling
lisp functions

```ts
lisp.my_function(1.0, 2, "MYSTRING", false)
```

If you need to access a symbol or keyword, you will use the symbol keyword
objects

```ts
const mySymbol = lisp.symbols.foo; // foo
const myKeyword = lisp.keywords.foo; // :foo
```

You can create more complex objects via the `make` object

```ts
const hashtable = lisp.make.hashtable({x: 3, y: 4});
const alist = lisp.make.alist({x: lisp.symbols.bar, y: "String"});
const plist = lisp.make.plist({zz: 19, zx: false});

const array = lisp.make.array([1, 2, 3, 4, 5]);
const list = lisp.make.list([1, 5, "String", lisp.symbols.x]);
const lstring = lisp.make.string("MyString");
```


### Errors

If a lisp function would trigger `(error ...)` in lisp, it will throw an error
in javascript. An example:

```js
try {
	lisp.cons(); // No arguments
} catch (e) {
	lisp.print(JSON.stringify(e));
}
```

## Defining Lisp Functions

We can also define functions that can be called via lisp. We will use `defun` to
accomplish this:

```ts
declare var lisp: any;

const insertIntoTypeScriptBuffer = (str: string) => {
      const buffer = lisp.get_buffer_create("TypeScript Filewatching");
      lisp.with_current_buffer(buffer, () => lisp.insert(str));
};

lisp.defun({
	name: "my-function",
	docString: "My Example Function",
	interactive: true,
	args: "MInput",
	func: (str: string) => insertIntoTypeScriptBuffer(str)
});
```

This defines a lisp function named `my-function`. We can call this function from
lisp `(my-function STRING)`, in JavaScript via `lisp.my_function(STRING)`, or
call it interactively. That means that within the editor if we press M-x and
type "my-function", we can invoke the function. It will then perform our
JavaScript action, which is to insert whatever text we enter into our TypeScript
Buffer.

## Conclusion

This covers the basic of calling lisp functions and I/O using Deno. Together
using these tools you can already build powerful apps, or allow emacs-ng to
perform actions. In our next series we will cover more advanced topics like
Threading and WebASM.
