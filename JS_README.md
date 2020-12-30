This fork of emacs integrates the [Deno Runtime](https://deno.land/). We do this by integrating Chrome's Javascript engine V8, and the deno_core and deno_runtime libraries. deno is dependent on [tokio](https://github.com/tokio-rs/tokio), so we integrate that as well. This code is a strictly additive layer, it changes no elisp functionality, and should be able to merge upstream patches cleanly.

The high level concept here is to allow full control of emacs-ng via the javascript layer. In order to avoid conflict between the elisp and javascript layers, only one scripting engine will be running at a time. elisp is the "authoritative" layer, in that the javascript will invoke elisp functions to interact with the editor. This is done via a special object in javascript called `lisp`. An example of it's usage would be:

``` js
let buffer = lisp.get_buffer_create("mybuf");
```

The lisp object uses reflection to invoke the equivalent of `(get-buffer-create "mybuf")`. In this example, get-buffer-create returns a buffer object. In order to represent this, we use a technique called proxying, in which we give a reference to the elisp object to an internal field of a javascript object. This field is only accessible by the native layer, so it cannot be tampered with in the javascript layer. When javascript calls subsequent functions using `buffer`, a translation layer will extract the elisp object and invoke it on the function.

Primitive data structures will be converted automatically. More complex data structures can be converted to native javascript objects via a call to a special function called `json`

``` js
let qcname = lisp.intern(":name");
let myList = lisp.list(qcname, 3);
console.log(myList); // prints { nativeProxy : true }
console.log(myList.json()); // prints { name: 3 }, defaulting to the assumption this list is a plist
```

It's important to note that once you convert to a native js object via a call to `json`, your mutations do not affect the lisp layer.

``` js
let result = lisp.buffer_string();
result += "my addition"; // This operation did not edit any lisp object.
let myList = lisp.list(qcname, 3);
let jsond = myList.json();
jsond.name = 4; // This did not edit a lisp object.
```

We expose the async IO functionality included with deno. Users can fetch data async from their local file system, or  the network. They can use that data to interact with the editor. An example would be:

``` js
const json = fetch("https://api.github.com/users/denoland")
.then((response) => { return response.json(); });

const txt = Deno.readTextFile("./test.json");

Promise.all([json, text])
    .then((data) => {
        let buffer = lisp.get_buffer_create('hello');
        const current = lisp.current_buffer();
        lisp.set_buffer(buffer);
        lisp.insert(JSON.stringify(data[0]));
        lisp.insert(data[1]);
        console.log(lisp.buffer_string());
        lisp.set_buffer(current);
    });
```
This example assumes you have a json file named `test.json` in your current directory.

Lambdas are converted automatically between the two languages via proxying. You can use  this to set timers, or to add hooks. This example features the lisp.symbols object, which returns a proxy to the quoted key. lisp.setq functions similarly to `(setq ....)`

``` js
lisp.add_hook(lisp.symbols.text_mode_hook, () => {
    lisp.setq(lisp.symbols.foo, "3");
});

// Or set timer with arguments...
lisp.run_with_timer(lisp.symbols.t, 1, (a, b) => {
    console.log('hello ' + a);
    lisp.print(b);
    return 3;
}, 3, lisp.make.alist({x: 3}));
```

The user can also define functions via `defun` that will call back into javascript. Like in normal defun, DOCSTRING and the INTERACTIVE decl are optional. This example features both docstring and interactive

``` js
lisp.defun("my-function", "This is my cool function", {interactive: true, args: "P\nbbuffer"}, (buff) => console.log("Hello Buffer"));
```

We also leverage module loading. Javascript can be invoked anonymously via `(eval-js "....")`, or a module can be loaded via `(eval-js-file "./my-file.js")`. Loading js as a module enables ES6 module imports to be used:

``` js
import { operation } from './ops.js';

const results = operation();
// ...
```

Though tokio is driving the io operations, execution of javascript is strictly controlled by the lisp layer. This is accomplished via a lisp timer. When the timer ticks, we advance the event loop of javascript one iteration.

While a LispObject is being proxy'd by javascript, we add that lisp object to a rooted lisp data structure, and add a special reference to the proxy in javascript called a `WeakRef`. We use `WeakRef`s to manage when javascript objects are no longer used by the javascript runtime and can be removed from our lisp GC root.

When executing javascript, invoking lisp functions may result in lisp throwing errors. For example:

``` js
const cons = lisp.cons(); // oh no, no arguments!
```

In this example, lisp.cons will throw a lisp layer error due to not being provided arguments. That error will be caught by the lisp/javascript intermediate layer, and a javascript exception will be thrown, with the value of (error-message-string). This can be caught at this js layer via a standard try/catch, or using a promise error handler.

``` js
try {
    const cons = lisp.cons(); //will throw
} catch (e) {
    // perform fallback.
}
```

The user can control what permissions javascript is granted when the runtime is initalized by a call to (js-initalize &REST). The user must call this function prior to javascript being executed for it to take effect. Otherwise, the js environment will initialize with default values.

``` lisp
(js-initialize :allow-net nil :allow-read nil :allow-write nil :allow-run nil :js-tick-rate 0.5)
```

This example, the user has denied javascript the ability to access the network, the file system, or spawn processes. They have increased the tick rate to 0.5.

In order to communicate errors to lisp, the user can provide an error handler at first initalization:

``` lisp
(js-initialize ..... :js-error-handler 'handler)
```

If the user does not provide an error handler, the default behavior will be to invoke (error ...) on any uncaught javascript errors.
