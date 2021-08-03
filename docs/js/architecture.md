# Architecture

The purpose of this document is to outline how emacs-ng's native changes work.

## JavaScript

The majority of JavaScript related code is located in rust_src/src/javascript.rs. There is a lot to unpack on this file, but we will attempt to unpack the core concepts:

### EmacsMainJsRuntime

This is a *thread local* singleton containing the JavaScript runtime and associated state. The two major fields in this struct are the Tokio runtime and the MainWorker. The choice to make this a thread local struct was due to the fact that Emacs has the ability to spawn threads via (make-thread), but v8 isolates do not like being shared between threads without using a specific API. We lean heavily on Deno's MainWorker class, which is also not designed to be shared between threads, so we made the choice to just stay threadlocal. The oddity here is that in the case of calling make-thread, the lisp threads will have their lisp variables will shared, but not their JavaScript variables.

#### Tokio

The Tokio runtime is what is driving all JavaScript Async I/O and timers. Tokio maintains its own threadpool on which tasks are enqueued. This is all "behind the scenes" to any emacs-ng code. The system is designed with the assumption that emacs-ng code will not have to call tokio::spawn or tokio::spawn_blocking.

#### MainWorker

The "MainWorker" is a Deno concept. It encapsulates Deno's module loader, file cache, and most importantly, the "JsRuntime". The JsRuntime encapsules the v8::Isolate, which can be described as the true, actual "JavaScript Runtime". Interfacing with the v8::Isolate is ultimately how JavaScript will be run. There are few instances where we call execute using the isolate directly. The MainWorker has some key restrictions included by design - if you have a top level module promise rejection, the Worker will panic upon the next attempt to execute JavaScript. The worker cannot handle cases where execute is called "deep" within the callstack. Meaning that if I call lisp -> javascript -> lisp -> JavaScript, I cannot depend on the Worker's execute method. We will have an entire section dedicated to that fact.

The EmacsMainJsRuntime is thread local because of the MainWorker - it is not Send nor Sync. The v8::Isolate, which is contained within the MainWorker, cannot be shared between threads by design. A quirk of this is that if you spawn a lisp thread, it has its own JavaScript runtime. This means that while the thread shares lisp globals, it does not share javaScript globals.

### Event Loop

A core concept to JavaScript is the event loop. All JavaScript invocations in emacs-ng start with a call to "run_module". run_module will eventually call into the MainWorkers execute method, which in turn calls into the v8::Isolate's execute method - are you starting to see the pattern?

Once you call run_module, you will begin to execute JavaScript. Doing so may enqueue async events, like setTimeout, fetch, etc. These are all things that Deno calls `async_ops` or `sync_ops`. Deno provides a system to manage ops. We have a policy that emacs-ng will not add ops, or deal with the ops API. If you want to add native functionality to emacs-ng, you should directly bind the function, like with `lisp_callback` and friends.

As these async events execute in the background, we will poll for their completion. In order to integrate with Emacs program loop, we set a timer, calling a function named `js-tick-event-loop`. This function is really just a wrapper around Deno's `poll_event_loop`. For performance reasons, `js-tick-event-loop` can call `poll_event_loop` multiple times. The user does have control of this behavior via `js_set_tick_rate`. In general, we want to give the user full control of emacs-ng, including how the JavaScript environment is configured.

### Proxies and Garbage Collector (GC) Interoperability

JavaScript has a concept of a 'Proxy' object, which we use in emacs-ng, however this section is about our JS <--> elisp marshaling. We refer to that as 'proxying' within internal documentation. 

For example, we will discuss how this code actually works:

```js
const buffer = lisp.get_buffer_create('*foo*');
lisp.set_current_buffer(buffer);
lisp.insert("FOOBAR");
```

#### The Lisp Object

The `lisp` object is a [JavaScript Capital-P Proxy](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Proxy). When you access `lisp.get_buffer_create`, it returns a function that does something like:

```js
(...args) => lisp_invoke('get-buffer-create' ...args);
```

`lisp_invoke` is a native function defined in javascript.rs. It's a wrapper around `ffuncall` with logic for object translation. It will also catch any lisp errors and translates them to JavaScript errors. `lisp_invoke` returns a JavaScript object - but this is a 'special object' in that it may have an internal field. Internal fields are data objects that are contained within JavaScript objects that are not accessible by the user. This is because LispObjects are just pointers - if we let the user alter pointers, they could cause SEGFAULTS or read arbitrary memory.

If we can parse the result of `lisp_invoke` as JSON, we do not proxy it. Instead we return the JavaScript equivalent (i.e. a JS string or number). However, if we cannot convert it to JSON, like in the case of a buffer, we return a proxy. Functions (i.e. `(lambda () (...))`) are another special case where additional logic is employed.

lisp_invoke also works in reverse when being called, it will `unproxy` the arguments it is passed in order to further pass them to `ffuncall`.

We expose a special function called `is_proxy` in order to tell if an object is a proxy.

#### GC Usage

When we create a proxy, we need to properly manage it in the lisp garbage collector. We do not want lisp to GC an object out from underneath us. In order to do this, we need to make two considerations for each direction of JS -> Lisp and Lisp -> JS

To prevent the lisp GC from removing objects that JS has a valid reference to, we include them in a special cons called the `js-retain-map`. The user does not have direct access to this object. Allowing them to access this cons would allow mutation in a way that could lead to use after free bugs.

When a proxy is created in JavaScript, we create a special JavaScript object called a `WeakRef`. This is [documented here](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/WeakRef). Once an object has no outstanding references (besides the WeakRef itself) the WeakRef will return undefined once accessed. We maintain a global array of WeakRefs for all proxies that we sweep every time lisp performs its garbage collection. We map this array to the `js-retain-map`. The end result is that if you have an object in JavaScript that is a proxy, it will always be valid.

elisp does not have proxies, it will only receive valid lisp objects from JavaScript, so this problem does not exist in the opposite direction.

#### Lambda Usage

Users will notice that lambas will auto convert between JS and elisp. Example:

```elisp
lisp.run_with_timer(1, lisp.symbols.nil, () => console.log('This works...'));
```

How does this work under the hood? When we go to create a proxy, we will test if that object is a function. If so, we will include it in another special array for functions. We will then create the following lisp object representing that lambda:

```lisp
(lambda (&REST) (js--reenter INDEX (make-finalizer (lambda () js--clear INDEX)) REST))
```

Index is hard-coded per lambda object, i.e. `js--reenter 1 ...`. Calling this lambda will call js--reenter, which is just calling the function in the array at INDEX. In order to ensure that function will be garbage collected, we add a lisp finalizer, which will clear the lambda from the array upon GC.
