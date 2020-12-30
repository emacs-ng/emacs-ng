import { helloWorld } from "./helloWorld.js";
import { basicLisp } from "./basicLisp.js";
import { webWorkers } from "./webWorkers.js";
import { webAsm } from "./webAsm.js";

let counter = 1;
Promise.prototype.test = (func) => {
    const r = func();
    return Promise.resolve().then(() => r).then(() => {
	console.log("Passed Test ..... " + counter);
	counter += 1;
    });
};

Promise.all([
    helloWorld(),
    basicLisp(),
    webWorkers(),
    webAsm()
])
    .then(() => {
	console.log("JS Tests Complete, No Errors");
	Deno.exit(0);
    })
    .catch(e => {
	console.log(e);
	Deno.exit(1);
    });
