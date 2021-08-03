# Main Features

## Javascript

This code is a strictly additive layer, it changes no elisp functionality, and should be able to merge upstream patches cleanly. JavaScript tests can be run by building the editor and executing `cd test/js && ../../src/emacs --batch --eval '(deno "test" "--allow-read" "--allow-write" "main.js")'`.

To learn more about JavaScript and TypeScript, check out [Getting Started](./getting-started.md), [Using Deno](./using-deno.md), and [Advanced Features](./adv-features.md)

### Using Async I/O

We expose the async IO functionality included with Deno. Users can fetch data async from their local file system, or the network. They can use that data to interact with the editor. 

An example would be:

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

### WebWorkers and Parallel Scripting

We also support WebWorkers, meaning that you can run JavaScript in separate threads. Note that WebWorkers cannot interact with the lisp VM, however they can use Deno for async I/O. See [Advanced Features](./adv-features.md)

Web Assembly allows you to perform things normally handled by native libraries with easy distribution. Want to manipulate sqlite3? Use the [deno sqlite wasm package](https://deno.land/x/sqlite@v2.3.2/mod.ts)

``` js
import { DB } from "https://deno.land/x/sqlite@v2.3.2/mod.ts";

const db = new DB("test.db");
db.query("CREATE TABLE IF NOT EXISTS people (id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT)");

const name = "David";
db.query("INSERT INTO people (name) VALUES (?)", [name]);
for (const [name] of db.query("SELECT name FROM people")) {
    console.log(name);
}

db.close();
```
