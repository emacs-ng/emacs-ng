(() => {
    let global = (1,eval)('this');
    let __weak = [];
    let finalize = global.finalize;
    delete global.finalize;
    let lisp_json = global.lisp_json;
    delete global.lisp_json;

    global.errorFuncs = {
	eval_js: true,
	eval_js_file: true,
	recursive_edit: true,
    };

    const __functions = [];

    global.__invoke = function (idx) {
	let modargs = [];
	for (let i = 1; i < arguments.length; ++i) {
	    const result = arguments[i];
            if (is_proxy(result)) {
                result.json = () => {
                    return JSON.parse(lisp_json(result));
                };

		modargs.push(result);
	    } else {
		modargs.push(JSON.parse(arguments[i]));
	    }
	}
	const retval = __functions[idx].apply(this, modargs);
	if (is_proxy) {
	    return retval;
	} else {
	    return JSON.stringify(retval);
	}
    };

    const specialForms = {
	hashtable: (a) => json_lisp(JSON.stringify(a), 0),
	alist: (a) => json_lisp(JSON.stringify(a), 1),
	plist: (a) => json_lisp(JSON.stringify(a), 2),
	array: (a) => json_lisp(JSON.stringify(a), 3),
	list: (a) => json_lisp(JSON.stringify(a), 4),
    };

    const argsLists = [
	() => lisp.list(),
	() => lisp.list(lisp.q.a),
	() => lisp.list(lisp.q.a, lisp.q.b),
	() => lisp.list(lisp.q.a, lisp.q.b, lisp.q.c),
	() => lisp.list(lisp.q.a, lisp.q.b, lisp.q.c, lisp.q.d),
	() => lisp.list(lisp.q.a, lisp.q.b, lisp.q.c, lisp.q.d, lisp.q.e),
	() => lisp.list(lisp.q.a, lisp.q.b, lisp.q.c, lisp.q.d, lisp.q.e. lisp.q.f),
	() => lisp.list(lisp.q.a, lisp.q.b, lisp.q.c, lisp.q.d, lisp.q.e. lisp.q.f, lisp.q.g),
    ];

    const invokeLists = [
	(len) => lisp.list(lisp.q.js__reenter, len),
	(len) => lisp.list(lisp.q.js__reenter, len, lisp.q.a),
	(len) => lisp.list(lisp.q.js__reenter, len, lisp.q.a, lisp.q.b),
	(len) => lisp.list(lisp.q.js__reenter, len, lisp.q.a, lisp.q.b, lisp.q.c),
	(len) => lisp.list(lisp.q.js__reenter, len, lisp.q.a, lisp.q.b, lisp.q.c, lisp.q.d),
	(len) => lisp.list(lisp.q.js__reenter, len, lisp.q.a, lisp.q.b, lisp.q.c, lisp.q.d, lisp.q.e),
	(len) => lisp.list(lisp.q.js__reenter, len, lisp.q.a, lisp.q.b, lisp.q.c, lisp.q.d, lisp.q.e, lisp.q.f),
	(len) => lisp.list(lisp.q.js__reenter, len, lisp.q.a, lisp.q.b, lisp.q.c, lisp.q.d, lisp.q.e, lisp.q.f, lisp.q.g),

    ];

    // Hold on you fool, why not use FinalizerRegistry, it
    // was made for this! That API does not work in Deno
    // at this time, due to their handling of the DefaultPlatform
    // Due to this, I opt'd to use weakrefs in a map. Its nice
    // because I just need to sync that map with a lisp gc root
    // and my job is done.
    // @TODO either make that time for sync customizable
    // or explore better options than hardcoding 10s.
    setInterval(() => {
        const nw = [];
        const args = [];
        __weak.forEach((e) => {
            let x = e.deref();
            if (x) {
                nw.push(e);
                args.push(x);
            }
            finalize.apply(this, args);
        });
        __weak = nw;
    }, 10000);

    global.lisp = new Proxy({}, {
        get: function(o, k) {
	    if (errorFuncs[k]) {
		throw new Error("Attempting to call non-supported function via javascript invokation (" + k + ")");
	    }

	    if (k === 'symbols' || k === 'q') {
		return new Proxy({}, {
		    get: function(o, k) {
			return lisp.intern(k.replaceAll('_', '-'));
		    }
		});

	    }

	    if (k === 'make') {
		return specialForms;
	    }

	    if (k === 'setq') {
		return function () {
		    let newArgs = [lisp.q.setq];
		    for (let i = 0; i < arguments.length; ++i) {
			newArgs.push(arguments[i]);
		    }

		    return lisp.eval(lisp.list.apply(this, newArgs));
		}
	    }

	    if (k === 'defun') {
		return function () {
		    let args = arguments;
		    if (args.length === 2) {
			let name = args[0];
			const func = args[1];
			if (typeof name === 'string') {
			    name = lisp.intern(name);
			}
			const numArgs = func.length;
			const argList = argsLists[numArgs];
			const invokes = invokeLists[numArgs];
			const len = __functions.length;

			lisp.eval(lisp.list(lisp.q.defun, name, argList(), invokes(len)));
			__functions.push(func);
		    } else if (args.length === 3) {
			let name = args[0];
			let second = args[1];
			const func = args[2];
			if (typeof name === 'string') {
			    name = lisp.intern(name);
			}

			if (second.interactive) {
			    if (second.arg) {
				second = lisp.list(lisp.q.interactive, second.arg);
			    } else {
				second = lisp.list(lisp.q.interactive);
			    }
			}

			const numArgs = func.length;
			const argList = argsLists[numArgs];
			const invokes = invokeLists[numArgs];
			const len = __functions.length;

			lisp.eval(lisp.list(lisp.q.defun, name, argList(), second, invokes(len)));
			__functions.push(func);
		    } else if (args.length === 4) {
			let name = args[0];
			let docstring = args[1];
			let interactive = args[2];
			const func = args[3];
			if (typeof name === 'string') {
			    name = lisp.intern(name);
			}

			if (interactive.interactive) {
			    if (interactive.arg) {
				interactive = lisp.list(lisp.q.interactive, interactive.arg);
			    } else {
				interactive = lisp.list(lisp.q.interactive);
			    }
			}

			const numArgs = func.length;
			const argList = argsLists[numArgs];
			const invokes = invokeLists[numArgs];
			const len = __functions.length;

			lisp.eval(lisp.list(lisp.q.defun, name, argList(), docstring, interactive, invokes(len)));
			__functions.push(func);

		    }

		}
	    }

            return function() {
                const modargs = [k.replaceAll('-', '_')];
                for (let i = 0; i < arguments.length; ++i) {
		    if (typeof arguments[i] === 'function') {
			const len = __functions.length;
			const numArgs = arguments[i].length;
			const args = argsLists[numArgs];
			const invokes = invokeLists[numArgs];
			// @TODO the invokation statement needs multiple variants
			// for varargs.
			const lambda = lisp.list(lisp.q.lambda, args(), invokes(len));
			__functions.push(arguments[i]);
			modargs.push(lambda);
		    } else if (is_proxy(arguments[i])) {
                        modargs.push(arguments[i]);
                    } else {
                        modargs.push(JSON.stringify(arguments[i]));
                    }
                }

                let result = lisp_invoke.apply(this, modargs);
                let retval = null;
                if (is_proxy(result)) {
                    result.json = () => {
                        return JSON.parse(lisp_json(result));
                    };

                    __weak.push(new WeakRef(result));
                    retval = result;
                } else {
                    retval = JSON.parse(result);
                }

                return retval;
            }

        }});
})();
