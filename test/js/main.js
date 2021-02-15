import { helloWorld } from "./helloWorld.js";
import { basicLisp } from "./basicLisp.js";
import { webWorkers } from "./webWorkers.js";
import { webAsm } from "./webAsm.js";
import { basicTyping } from "./basicTyping.ts";
import { errors } from "./errors.js";

Promise.prototype.test = function(name, f) {
    let now = Date.now();
    return this.then(() => { now = Date.now() })
	.then(f)
	.then((result) => {
	    console.log(`Passed Test ${name} ... (${Date.now() - now} ms)`);
	    return result;
	});

};

Promise.all([
    helloWorld(),
    basicLisp(),
    webWorkers(),
    webAsm(),
    basicTyping(),
    errors(),
])
    .then(() => {
	console.log("JS Tests Complete, No Errors");
	Deno.exit(0);
    })
    .catch(e => {
	console.log(">>> TEST FAILURE <<<");
	console.log(e);
	Deno.exit(1);
    });
