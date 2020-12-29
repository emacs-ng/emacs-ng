import { helloWorld } from "./helloWorld.js";
import { basicLisp } from "./basicLisp.js";

let counter = 1;
Promise.prototype.test = (func) => {
    const r = func();
    console.log("Passed Test ..... " + counter);
    counter += 1;
    return Promise.resolve().then(() => r);
};

Promise.all([
    helloWorld(),
    basicLisp()
])
    .then(() => console.log("JS Tests Complete, No Errors"))
    .catch(e => {
	console.log(e);
	Deno.exit(1);
    });
