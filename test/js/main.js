import { helloWorld } from "./helloWorld.js";
import { basicLisp } from "./basicLisp.js";

Promise.all([
    helloWorld(),
    basicLisp()
])
    .then(() => console.log("JS Tests Complete, No Errors"))
    .catch(e => {
	console.log(e);
	Deno.exit(1);
    });
