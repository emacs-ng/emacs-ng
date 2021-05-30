# Advanced Features

This section assumes you have completed "Getting Started" and "Using Deno" and have a basic familiarization with the elisp JavaScript API.

## WebWorkers

One of the big draws of emacs-ng is parallel scripting. You may or may not be familiar with the fact that vanilla emacs had lisp threads that you could create via `(make-thread FUNC)`. Lisp threads were concurrent, but not parallel. In addition to that limitation, lisp threads that were not the 'main thread' would only execute at specific times during emacs event loop.

WebWorkers give you parallelism. Under the hood, they are spinning up a Rust std::thread (in C terms, this is analogous to a pthread). Due to this, there is a limited interface to exchange data between threads.

## Building up the WebWorker

We will start by creating the JavaScript our WebWorker will execute. Create a file named "web-worker.js" and insert the following:

```js
self.onmessage = (input) => {
    let { message } = input.data;
    message += " And My Axe ";
    self.postMessage({ message });
    self.close();
};
```

WebWorkers communicate via two special functions, postMessage and onmessage. onmessage will be called once our parent thread sends us a message, while we can use postMessage to send data back to the main thread. This example is appending a string to its input and handing that back to the main thread. We also called the `close` function, meaning that this WebWorker is "one and done", and will shut down once it has performed this operation a single time. If we did not call close, the worker would stay alive and await additional messages.

## Using our WebWorker

Create a new file with the following code:

```ts
declare var lisp: any;

const worker = new Worker(new URL("web-worker.js", import.meta.url).href,
	{
		type: "module",
		deno: true,
	});

worker.onmessage = (output) => {
	const { message } = output.data;
	// This is safe because our callback is back
	// on the main thread.
	lisp.print(message);
}

worker.postMessage({ message: "You have my sword .... " });
```

This code spins up our WebWorker and passes it a message. Running this code with `eval-ts-buffer` should print a reference to a cool little obscure movie in your minibuffer.

## Using Deno with WebWorkers

WebWorkers do not have access to elisp functions - you will notice that if you attempt to use the `lisp` object you will get an error that it is not defined. However, WebWorkers do have full access to Deno.

The recommended usage for WebWorkers is to

1. Identify the slow or blocking portion of your existing or new elisp code
2. Write your webworker to perform the operation and data manipulation and translate the results to a format that elisp will be able to use. I.e. if you want to walk a directory and return all files ending in "foo", perform that logic and construct the array on your WebWorker, and send that array to your main thread via sendMessage. Once your mainThread receives that information, you can call your elisp code to display it.

It's recommended to avoid WebWorkers for pure I/O or subprocess operations. Instead, use Deno's built in async I/O capabilities [outlined in their documentation](https://deno.land/manual@v1.6.3/examples). You should use WebWorkers when you have significant calculation or operation to perform on the returned data. An example would be: Don't use a webworker to just run `git status`. You may want to use a WebWorker if you are running git status, opening each file, and performing complex logic on each line of the file, and then returning a list of strings back to lisp for display.

You can see their example for [spawning a subprocess](https://deno.land/manual@v1.6.3/examples/subprocess) to see what the platform is capable of. This can allow for things normally handled by tramp -> you could ssh into a box, get a list of files, and then actually perform scripting logic on that result prior to passing it back to lisp for display. Current elisp can handle about half of that, however once ssh, or whatever process returns those results, any logic on the subprocesses output will be blocking the editor.

You can also reference [this module](https://github.com/DavidDeSimone/ng-fuzzy-search) as another example.
