export function basicLisp() {
    return Promise.resolve()
	.test(() => {
	    let plist = lisp.make.plist({x: 3, y: 4, z: lisp.symbols.qqz});
	    let alist = lisp.make.alist({s: 5, y: "Hello World", z: lisp.symbols.qq});

	    let prop = lisp.plist_get(plist, lisp.keywords.z);
	    if (!lisp.eq(prop, lisp.symbols.qqz)) {
		throw new Error("Failed to insert symbol into plist");
	    }

	    let prop2 = lisp.alist_get(lisp.symbols.s, alist);
	    if (prop2 !== 5) {
		throw new Error("Failed to get proper alist property");
	    }

	    let list = lisp.make.list([1, 2, "foo", lisp.symbols.qqq]);
	    let item = lisp.nth(3, list);
	    if (!lisp.eq(item, lisp.symbols.qqq)) {
		throw new Error("Failed to fetch list item properly");
	    }
	})
	.test(() => {
	    let p = lisp.symbols.a;
	    let qq = lisp.symbols.qq;
	    let qc = lisp.keywords.word;
	    lisp.setq(p, "hello");
	    lisp.setq(qq, lisp.list(1, 2, 3));
	    if (lisp.symbol_value(p) !== 'hello') {
		throw new Error("Failure in test lisp.setq");
	    }
	})
	.test(() => {
	    let mutated = 0;
	    let myFunc = lisp.defun("hello", (arg, arg2) => {
		mutated = arg;
		return 4;
	    });
	    lisp.hello(1, 3);

	    let myFuncTwo = lisp.defun("helloTwo", "myDocString", (arg) => { return arg; });
	    lisp.helloTwo(2);

	    let myFuncThree = lisp.defun("helloThree", {interactive: true, arg: "P\nbbuffer:"}, (arg) => { });
	    lisp.helloThree(3);

	    let myFuncFour = lisp.defun("helloFour", "HelloFour", {interactive: true}, () => { });
	    lisp.helloFour();

	    let myFuncFive = lisp.defun({
		name: "helloFive",
		docString: "My Cool Function",
		interactive: true,
		args: "p",
		func: (a) => { return (a); }
	    });

	    if (lisp.helloFive(1) !== 1) {
		throw new Error("Return Value incorrect for Defun");
	    }

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

		lisp.setq(lisp.symbols.bbb, "3");

		if (lisp.symbol_value(lisp.symbols.bbb) !== "3") {
		    throw new Error("Value not set within let.");
		}
	    }, 3);
	})
	.test(() => {
	    let buf = lisp.get_buffer_create("mybuff");
	    let buf2 = lisp.get_buffer_create("mybuff2");
	    lisp.set_buffer(buf2);
	    lisp.insert("World,,,");

	    let executed = false;
	    lisp.with_current_buffer(buf, () => {
		executed = true;
		lisp.insert("Hello");
		if (lisp.buffer_string() !== "Hello") {
		    throw new Error("Error in current buffer implementation");
		}
	    });

	    if (!executed) {
		throw new Error("with-current-buffer failed to execute");
	    }

	    if (lisp.buffer_string() === "Hello" || lisp.buffer_string() !== "World,,,") {
		throw new Error("with-current-buffer did not properly retain buffer state");
	    }
	})
	.test(() => {
	    let executed = false;
	    lisp.with_temp_buffer(() => {
		executed = true;
		lisp.insert("XXX12345");
	    });

	    if (lisp.buffer_string() === "XXX12345") {
		throw new Error("Error in with-temp-buffer test: mutated normal buffer");
	    }

	    if (!executed) {
		throw new Error("with-temp-buffer failed to execute");
	    }
	});
}
