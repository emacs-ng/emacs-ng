# JavaScript FAQ

## Is JavaScript supposed to replace elisp?

The answer here is a loud NO. The maintainers love elisp and we will never remove elisp functionality from emacs-ng. JavaScript/TypeScript are peer languages in the emacs-ng ecosystem, meaning that if new maintainers want to write a package fully in JS/TS, they have that option. They have the full lisp API for interacting with the editor available to them.

## How should I use JS/TS as an existing package maintainer?

If you have a large elisp package, our guidance is not that you should rewrite your entire package in JS/TS. Instead, we encourage package maintainers to explore using JS/TS's Async I/O and Threading capabilities to improve performance their hot code paths on emacs-ng. Using `(featurep 'emacs-ng)`, you can include an import statement for a JS/TS package that defun's functions for you to use. Our [Getting Started Guide](./getting-started.md) is a good place to start.

## How does adding JS/TS affect your ability to merge future emacs improvements?

JS/TS is almost a completely additive layer, we have made extremely minimal C changes. As of writing this FAQ, we have only made a small edit to a single line of C to support JS/TS. WebRender, while still in development, has also made minimal C changes. We have the ability to cleanly merge upstream patches without conflict. emacs-ng is based off of the `native-comp` branch of emacs, and regularly merges in the latest from that branch. emacs-ng can be compiled with nativecomp using `./configure --with-native-compilation`.

## How does JS/TS running affect performance?

JS/TS is a "you pay for what you use" system. The JS runtime starts uninitialized, and will not be initialized until you run JavaScript. The JS/TS event loop only runs when you have a pending async operation, including timer callbacks. If you don't have any pending promises/callbacks, the event loop isn't running. This means that impact is proportional to how you use the runtime.

## If I'm chasing performance, why not just write a C/Rust module?

JS/TS features faster iteration speed and easier distribution for you as the developer. In addition, your users get a greater deal of freedom and customization for your package because it's all in the scripting layer, as opposed to a binary blob they would have to recompile if they wanted to edit the behavior of the code. Instead of building your .so, uploading to a package repository, and dealing with user complaints when that .so isn't loaded properly, you can distribute your script files and still get a considerable performance increase.

## Will you provide TypeScript definitions for elisp functions?

We do not currently offer that, but it is planned work. We welcome contributions for that effort.

## emacs-ng seems to be using a large amount of virtual memory?

This is due to Tokio/v8 loading up a potentially large number of worker threads based on your core count. You will notice that real memory committed goes up very little from initializing the runtime. In standard emacs, we have observed emacs sitting at about 700K rss vs 900K rss with the JS runtime initialized. Overall, the real memory overhead of initializing is not as bad as the virtual commitment makes it seem.

## I have an existing node package I want to use, but import isn't working

We use the [Deno Framework](https://deno.land) for our JavaScript. Deno uses actual ES6 imports, and not node's commonJS require syntax. You will need to use Deno's compatability module to use require syntax. See https://github.com/denoland/deno/tree/master/std/node
