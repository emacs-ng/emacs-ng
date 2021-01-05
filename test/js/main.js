import { helloWorld } from "./helloWorld.js";
import { basicLisp } from "./basicLisp.js";
import { webWorkers } from "./webWorkers.js";
import { webAsm } from "./webAsm.js";
import { basicTyping } from "./basicTyping.ts";

let counter = 1;
Promise.prototype.test = function(f) {
    return this.then(f)
	.then((result) => {
	    counter += 1;
	    console.log("Passed Test ... " + counter);
	    return result;
	});

};

Promise.all([
    helloWorld(),
    basicLisp(),
    webWorkers(),
    webAsm(),
    basicTyping(),
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
