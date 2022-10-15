# Using the power of Deno

## What is Deno?

[Deno](https://deno.land) is a program that is similar to Node.js, except that it is written in Rust. Both Deno and Node.js were created by the same person: Ryan Dhal. While normally you would invoke Deno via the command line, the emacs-ng project has integrated the Deno runtime into the emacs-ng client directly. Deno is powered by v8, Chrome's Open Source JavaScript engine.

A JavaScript engine is more limited than most users realize. For example, utilities like XMLHttpRequest are not provided directly by the JavaScript engine, but are instead provided by a runtime (like your browser). Deno provides interfaces for performing I/O operations like file reads/writes, network operations, and spawning subprocesses.

By default, emacs-ng will allow reading, writing, network, and subprocess operations. In Deno's examples, you may see them advise you to pass flags like "--allow-net". These are not needed when executing emacs-ng code. If you want to globally disable any of the previously mentioned operations, you can run the following lisp prior to executing JavaScript:

`(js-initialize :allow-net nil :allow-read nil :allow-write nil :allow-run nil)`

You can define any combination of the above arguments. See js-initialize's documentation for more information.

# General Documentation / Standard Library

Deno has [excellent documentation](https://deno.land/manual). This guide is to give you a basic familiarity with Deno to help you to quickly write applications, but is FAR from all inclusive.

Deno maintains a powerful standard library at https://deno.land/std@0.83.0 . Importing a Deno std module is simple: just include this in your JavaScript file/buffer:


Credit to https://deno.land/std@0.83.0/fs for the example:
```js
import { copy, copySync } from "https://deno.land/std@0.83.0/fs/mod.ts";

copy("./foo", "./bar"); // returns a promise
copySync("./foo", "./bar"); // void
copySync("./foo", "./existingFolder", { overwrite: true });
```

The first thing you may notice is that we are importing a URL, not a local filepath. Deno allows you to download dependencies from the network. This file will only be downloaded once and compiled once, and it's results will be cached on your local file system. After initial download, you will use your local filesystem's copy instead of using the network.

By default, **all Deno API's are asynchronous**. Almost all async operations have an alternate version that is synchronous. That means that when you make a call to read a file, or walk a directory, it is returning a Promise. Deno has the naming convention that the synchronous versions all end in "<name>Sync".

A common point of confusion with emacs-ng is that when you evaluate a file, a buffer, or even an anonymous block in JavaScript via `(eval-js)`, you are executing your code within a JavaScript module with top level await enabled. What does that mean? *If you invoke await within your top level module, you will block the main thread until completion.*

Looking again with our example above with that in mind:

```js
import { copy } from "https://deno.land/std@0.83.0/fs/mod.ts";

// This block the main thread, including lisp execution, until this action is completed
await copy("./foo", "./bar");

// This does not block JavaScript or Lisp, instead our .then will be executed once
// the async action is complete.
copy("./foo", "./bar").then(() => console.log("Complete"));
```

Remember that async/await in JavaScript is just syntax sugar over Promises. There may be times where you want to use the top level await functionality to block on a promise at a certain time.

## Distribution

Once you have created your great emacs-ng module, how do you distribute it? Normally you would go through a repository like ELPA or MELPLA. While that is still a possibility, you have a third option, which is [Deno's user modules](https://deno.land/x). Navigating to the link below gives you the information on the upload process, but in the author's opinion, it is very simple and streamlined. Once your module is uploaded, you can have your user's include a line similar to this in their init.el

`(eval-js "import 'https://deno.land/x/fuzzy_search@0.3.0/mod-fuzzy.js'")`

Where instead of fuzzy_search@0.3.0/mod-fuzzy.js, you instead have your module version and filename.

## Using emacs as Deno

emacs-ng offers the `deno` function in elisp, which allows users to leverage whatever Deno offers from the command line. For example, you can run Deno's repl (with elisp functions) by running the following:

```bash
emacs --batch --eval '(deno "repl")'
```

You can use Deno's formatter by running


```bash
emacs --batch --eval '(deno "fmt")'
```

You could even run a script via

```bash
emacs --batch --eval '(deno "run" "--allow-read" "test.ts")'
```

NOTE: You need to specify read/write/etc. permissions. Think of this as if you were using emacs AS Deno.

The Deno function takes the exact same flags as the Deno application. It's designed for use in batch mode on the command line, however it can also be used in regular elisp.

## Where to go next

We don't want to duplicate Deno's excellent documentation, so it's recommended you [read their manual](https://deno.land/manual) for their standard library, and their examples.
