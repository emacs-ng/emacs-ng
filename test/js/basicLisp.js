export function basicLisp() {
    return Promise.resolve()
	.then(() => {
	    let plist = lisp.make.plist({x: 3, y: 4});
	    let alist = lisp.make.alist({s: 5, y: "Hello World"});
	    console.log("Printing results of lisp.make");
	    lisp.print(plist);
	    lisp.print(alist);
	})
	.then(() => {
	    let p = lisp.symbols.a;
	    let qq = lisp.symbols.qq;
	    lisp.setq(p, "hello");
	    lisp.setq(qq, lisp.list(1, 2, 3));
	    if (lisp.symbol_value(p) !== 'hello') {
		throw new Error("Failure in test lisp.setq");
	    }
	})
	.then(() => {
	    let mutated = 0;
	    let myFunc = lisp.defun("hello", (arg) => {
		mutated = arg;
	    });
	    lisp.hello(1);

	    if (mutated !== 1) {
		throw new Error("Failure in test Defun: Mutated value not set from callback");
	    }
	});
}
