export function basicLisp() {
    return Promise.resolve()
	.test(() => {
	    let plist = lisp.make.plist({x: 3, y: 4});
	    let alist = lisp.make.alist({s: 5, y: "Hello World"});
	})
	.test(() => {
	    let p = lisp.symbols.a;
	    let qq = lisp.symbols.qq;
	    lisp.setq(p, "hello");
	    lisp.setq(qq, lisp.list(1, 2, 3));
	    if (lisp.symbol_value(p) !== 'hello') {
		throw new Error("Failure in test lisp.setq");
	    }
	})
	.test(() => {
	    let mutated = 0;
	    let myFunc = lisp.defun("hello", (arg) => {
		mutated = arg;
	    });
	    lisp.hello(1);

	    let myFuncTwo = lisp.defun("helloTwo", "myDocString", (arg) => { });
	    lisp.helloTwo(2);

	    let myFuncThree = lisp.defun("helloThree", {interactive: true, arg: "P\nbbuffer:"}, (arg) => { });
	    lisp.helloThree(3);

	    let myFuncFour = lisp.defun("helloFour", "HelloFour", {interactive: true}, () => { });
	    lisp.helloFour();

	    if (mutated !== 1) {
		throw new Error("Failure in test Defun: Mutated value not set from callback");
	    }
	})
	.test(() => {
	    lisp.make_pipe_process(lisp.keywords.name, "mybuff");
	})
	.test(() => {
	    lisp.let((a) => {
		if (a !== 3) {
		    throw new Error("Arguments do not match");
		}

		lisp.setq(lisp.symbols.a, "3");

		if (lisp.symbol_value(lisp.symbols.a) !== "3") {
		    throw new Error("Value not set within let.");
		}
	    }, 3);

	    if (lisp.symbol_value(lisp.symbols.a) === "3") {
		throw new Error("Scope leaking from let statement");
	    }
	})
	.test(() => {
	    let buf = lisp.get_buffer_create("mybuff");
	    lisp.with_current_buffer(buf, (buffer) => {
		lisp.insert("Hello");
		if (lisp.buffer_string() !== "Hello") {
		    throw new Error("Error in current buffer implementation");
		}
	    });
	})
	.test(() => {
	    lisp.with_temp_buffer(() => {
		lisp.insert("XXX12345");
	    });

	    if (lisp.buffer_string() === "XXX12345") {
		throw new Error("Error in with-temp-buffer test: mutated normal buffer");
	    }
	});
}
