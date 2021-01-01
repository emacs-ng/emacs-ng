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
	if (is_proxy(retval)) {
	    return retval;
	} else {
	    return JSON.stringify(retval);
	}
    };

    global.__clear = (idx) => { __functions[idx] = null; };

    const makeFuncs = {
	hashtable: (a) => json_lisp(JSON.stringify(a), 0),
	alist: (a) => json_lisp(JSON.stringify(a), 1),
	plist: (a) => json_lisp(JSON.stringify(a), 2),
	array: (a) => json_lisp(JSON.stringify(a), 3),
	list: (a) => json_lisp(JSON.stringify(a), 4),
    };

    const invokeLists = [
	(len) => lisp.list(lisp.q.js__reenter, len, finalizerLists(len)),
	(len) => lisp.list(lisp.q.js__reenter, len, finalizerLists(len), lisp.q.a),
	(len) => lisp.list(lisp.q.js__reenter, len, finalizerLists(len), lisp.q.a, lisp.q.b),
	(len) => lisp.list(lisp.q.js__reenter, len, finalizerLists(len), lisp.q.a, lisp.q.b, lisp.q.c),
	(len) => lisp.list(lisp.q.js__reenter, len, finalizerLists(len), lisp.q.a, lisp.q.b, lisp.q.c, lisp.q.d),
	(len) => lisp.list(lisp.q.js__reenter, len, finalizerLists(len), lisp.q.a, lisp.q.b, lisp.q.c, lisp.q.d, lisp.q.e),
	(len) => lisp.list(lisp.q.js__reenter, len, finalizerLists(len), lisp.q.a, lisp.q.b, lisp.q.c, lisp.q.d, lisp.q.e, lisp.q.f),
	(len) => lisp.list(lisp.q.js__reenter, len, finalizerLists(len), lisp.q.a, lisp.q.b, lisp.q.c, lisp.q.d, lisp.q.e, lisp.q.f, lisp.q.g),

    ];

    const finalizerLists = (len) => lisp.eval(lisp.list(lisp.q.make_finalizer, lisp.list(lisp.q.lambda, lisp.list(), lisp.list(lisp.q.js__clear, len))));

    // Hold on you fool, why not use FinalizerRegistry, it
    // was made for this! That API does not work in Deno
    // at this time, due to their handling of the DefaultPlatform
    // Due to this, I opt'd to use weakrefs in a map. Its nice
    // because I just need to sync that map with a lisp gc root
    // and my job is done.
    // @TODO either make that time for sync customizable
    // or explore better options than hardcoding 2.5s.
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
    }, 2500);


    // Crossing the JS -> Lisp bridge costs time, which we want to save.
    // We can save time by not crossing the bridge if we cache our symbols in a map.
    // However, this costs memory, AND it can cause issues if the user decides to
    // unintern a symbol, or change the value of Vobarray. In prelim.js, we have
    // a lot of hot fundemental codepaths we want to be fast, so we use the cached
    // version (lisp.q and lisp.k). This is accessible to the user, but not documented on
    // purpose. The user is encouraged to use lisp.symbols and lisp.keywords, which
    // are just wrappers for intern, which will behave as they expect with usage of
    // unintern, or changing the obarray.
    const symbolCache = {};
    const symbolsCached = () => {
	return new Proxy({}, {
	    get: function(o, k) {
		let cached = symbolCache[k] || lisp.intern(k.replaceAll('_', '-'));
		symbolCache[k] = cached;
		return cached;
	    }
	});
    };

    const symbols = () => {
	return new Proxy({}, {
	    get: function(o, k) {
		return lisp.intern(k.replaceAll('_', '-'));
	    }
	});
    };

    const keywordCache = {};
    const keywordsCached = () => {
	return new Proxy({}, {
	    get: function(o, k) {
		const cached = keywordCache[k] || lisp.intern(':' + k.replaceAll('_', '-'));
		keywordCache[k] = cached;
		return cached;
	    }
	});
    };

    const keywords = () => {
	return new Proxy({}, {
	    get: function(o, k) {
		return lisp.intern(':' + k.replaceAll('_', '-'));
	    }
	});
    };

    const quote = (arg) => lisp.list(lisp.q.quote, arg);
    const setq = () => {
	return function () {
	    let newArgs = [lisp.q.setq];
	    for (let i = 0; i < arguments.length; ++i) {
		if (lisp.listp(arguments[i])
		    && !lisp.eq(lisp.q.quote, lisp.car(arguments[i]))) {
		    newArgs.push(quote(arguments[i]));
		} else {
		    newArgs.push(arguments[i]);
		}
	    }

	    return lisp.eval(lisp.list.apply(this, newArgs));
	};
    };

    const defun = () => {
	const makeStatement = (name, docString, interactive, lambda) => {
	    if (typeof name === 'string') {
		name = lisp.intern(name);
	    }

	    const argLen = lambda.length;
	    const argList = argsLists[argLen];
	    const invoke = invokeLists[argLen];
	    const args = [lisp.q.defun, name, argList];
	    if (docString) {
		args.push(docString);
	    }

	    if (interactive) {
		if (interactive.interactive) {
		    if (interactive.args) {
			args.push(lisp.list(lisp.q.interactive, interactive.args));
		    } else {
			args.push(lisp.list(lisp.q.interactive));
		    }
		}
	    }

	    let len = __functions.length;
	    args.push(invoke(len));
	    lisp.eval(lisp.list.apply(this, args));
	    __functions.push(lambda);
	};


	return function () {
	    if (typeof arguments[0] === 'object' && arguments[0].name) {
		const arg = arguments[0];
		return makeStatement(arg.name, arg.docString, { interactive: arg.interactive, args: arg.args}, arg.func);
	    }

	    let args = arguments;
	    if (args.length === 2) {
		return makeStatement(args[0], null, null, args[1]);
	    } else if (args.length === 3) {
		if (args[1].interactive) {
		    return makeStatement(args[0], null, args[1], args[2]);
		} else {
		    return makeStatement(args[0], args[1], null, args[2]);
		}
	    } else if (args.length === 4) {
		return makeStatement(args[0], args[1], args[2], args[3]);
	    }
	};
    };

    const _let = function () {
	return function (lambda) {
	    const args = [];
	    const numArgs = lambda.length;
	    const argList = argsLists[numArgs];
	    const invoke = invokeLists[numArgs];
	    const list = [lisp.q['let'], argList, invoke(numArgs)];

	    for (let i = 1; i < arguments.length; ++i) {
		list.push(arguments[i]);
	    }

	    return lisp.eval(lisp.list.apply(this, args));
	}
    };

    const with_current_buffer = () => {
	return function(bufferOrName, lambda) {
	    if (lambda.length !== 0) {
		throw new Exception("with-current-buffer lambda takes 0 arguments");
	    }

	    const invoke = invokeLists[0];
	    const len = __functions.length;
	    const list = [lisp.q.with_current_buffer, bufferOrName, invoke(len)];
	    __functions.push(lambda);

	    return lisp.eval(lisp.list.apply(this, list));
	}

    };

    const with_temp_buffer = () => {
	return function(lambda) {
	    if (lambda.length !== 0) {
		throw new Exception("with-temp-buffer lambda takes 0 arguments");
	    }

	    const invoke = invokeLists[0];
	    const len = __functions.length;
	    const list = [lisp.q.with_temp_buffer, invoke(len)];
	    __functions.push(lambda);

	    return lisp.eval(lisp.list.apply(this, list));
	}

    };

    const specialForms = {
	make: makeFuncs,
	q: symbolsCached(),
	symbols: symbols(),
	setq: setq(),
	defun: defun(),
	keywords: keywords(),
	k: keywordsCached(),
	"let": _let(),
	with_current_buffer: with_current_buffer(),
	with_temp_buffer: with_temp_buffer(),
	quote: quote,
    };


    global.lisp = new Proxy({}, {
        get: function(o, k) {
	    if (errorFuncs[k]) {
		throw new Error("Attempting to call non-supported function via javascript invokation (" + k + ")");
	    }

	    if (specialForms[k]) {
		return specialForms[k];
	    }

            return function() {
                const modargs = [k.replaceAll('-', '_')];
                for (let i = 0; i < arguments.length; ++i) {
		    if (typeof arguments[i] === 'function') {
			const len = __functions.length;
			const numArgs = arguments[i].length;
			const args = argsLists[numArgs];
			const invokes = invokeLists[numArgs];
			const lambda = lisp.list(lisp.q.lambda, args, invokes(len));
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

    const argsLists = [
	lisp.list(),
	lisp.list(lisp.q.a),
	lisp.list(lisp.q.a, lisp.q.b),
	lisp.list(lisp.q.a, lisp.q.b, lisp.q.c),
	lisp.list(lisp.q.a, lisp.q.b, lisp.q.c, lisp.q.d),
	lisp.list(lisp.q.a, lisp.q.b, lisp.q.c, lisp.q.d, lisp.q.e),
	lisp.list(lisp.q.a, lisp.q.b, lisp.q.c, lisp.q.d, lisp.q.e, lisp.q.f),
	lisp.list(lisp.q.a, lisp.q.b, lisp.q.c, lisp.q.d, lisp.q.e, lisp.q.f, lisp.q.g),
    ];
})();
