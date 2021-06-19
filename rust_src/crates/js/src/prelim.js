(() => {
  let global = (1, eval)("this");
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
  const __reverse_proxies = [];

  global.__invoke = function (idx) {
    let modargs = [];
    for (let i = 1; i < arguments.length; ++i) {
      const result = arguments[i];
      if (is_proxy(result)) {
        if (is_reverse_proxy(result)) {
          const idx = unreverse_proxy(result);
          modargs.push(__reverse_proxies[idx]);
        } else {
          result.json = () => {
            return JSON.parse(lisp_json(result));
          };

          modargs.push(result);
        }
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

  global.__clear = (idx) => {
    __functions[idx] = null;
  };

  global.__clear_r = (idx) => {
    __reverse_proxies[idx] = null;
  };

  global.__eval = (str) => {
    const evalResult = (1, eval)(str);
    const retval = processArgs([evalResult]);
    return retval[0];
  };

  const makeHashTable = (a) => {
    let x = lisp.make_hash_table();
    for (k in a) {
      lisp.puthash(k, a[k], x);
    }

    return x;
  };

  const makeAlist = (a) => {
    let result = lisp.q.nil;
    for (k in a) {
      const sym = lisp.intern(k);
      result = lisp.cons(lisp.cons(sym, a[k]), result);
    }

    return lisp.reverse(result);
  };

  const makePlist = (a) => {
    let result = lisp.q.nil;
    for (k in a) {
      const sym = lisp.intern(":" + k);
      result = lisp.cons(sym, result);
      result = lisp.cons(a[k], result);
    }

    return lisp.reverse(result);
  };

  const makeArray = (a) => {
    const len = a.length;
    let v = lisp.make_vector(len, lisp.q.unbound);
    for (let i = 0; i < len; ++i) {
      lisp.aset(v, i, a[i]);
    }

    return v;
  };

  const makeReverseProxy = (a) => {
    const len = __reverse_proxies.length;
    __reverse_proxies.push(a);
    return make_reverse_proxy(len);
  };

  const makeFuncs = {
    hashtable: (a) => makeHashTable(a),
    alist: (a) => makeAlist(a),
    plist: (a) => makePlist(a),
    array: (a) => makeArray(a),
    list: (a) => lisp.list.apply(this, a),
    string: (a) => lisp_string(a),
    proxy: (a) => makeReverseProxy(a),
  };

  const stringToLispCache = {};
  const getOrCacheString = (str) => {
    let value = stringToLispCache[str];
    if (value) {
      let deref = value.deref();
      if (deref) {
        return deref;
      }
    }

    const string = processReturn(lisp_string(str));
    stringToLispCache[str] = new WeakRef(string);
    return string;
  };

  const numCache = {};
  const getOrCacheNum = (num) => {
    let value = numCache[num];
    if (value) {
      let deref = value.deref();
      if (deref) {
        return deref;
      }
    }

    const isInt = num % 1 === 0;
    let v = null;
    if (isInt) {
      v = lisp_fixnum(num);
    } else {
      v = lisp_float(num);
    }

    const result = processReturn(v);
    numCache[num] = new WeakRef(result);
    return result;
  };

  const getLambdaArgs = (len) => {
    if (len === 0) {
      return lisp.q.nil;
    }

    return varArgsList;
  };

  const getLambdaDef = (numArgs, lambda) => {
    const len = __functions.length;
    const result = lisp_make_lambda(len, numArgs);
    __functions.push(lambda);
    return result;
  };

  // Hold on you fool, why not use FinalizerRegistry, it
  // was made for this! That API does not work in Deno
  // at this time, due to their handling of the DefaultPlatform
  // Due to this, I opt'd to use weakrefs in a map. It's nice
  // because I just need to sync that map with a lisp gc root
  // and my job is done.
  // This is called from lisp's native garbage_collect function
  // via the lisp function (js--sweep)
  global.__sweep = () => {
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
  };

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

  const get_or_cache = (k, cache) => {
    const cached = cache[k];
    if (cached) {
      const v = cached.deref();
      if (v) {
        return v;
      }
    }

    const v = _intern(k.replaceAll("_", "-"));
    cache[k] = new WeakRef(v);
    return v;
  };

  const symbolsCached = () => {
    return new Proxy({}, {
      get: function (o, k) {
        return get_or_cache(k, symbolCache);
      },
    });
  };

  const symbols = () => {
    return new Proxy({}, {
      get: function (o, k) {
        return lisp.intern(k.replaceAll("_", "-"));
      },
    });
  };

  const keywordCache = {};
  const keywordsCached = () => {
    return new Proxy({}, {
      get: function (o, k) {
        return get_or_cache(k, keywordCache);
      },
    });
  };

  const keywords = () => {
    return new Proxy({}, {
      get: function (o, k) {
        return lisp.intern(":" + k.replaceAll("_", "-"));
      },
    });
  };

  const quote = (arg) => lisp.list(lisp.q.quote, arg);

  const defun = () => {
    const makeStatement = (name, docString, interactive, lambda) => {
      if (typeof name === "string") {
        name = lisp.intern(name);
      }

      const argLen = lambda.length;
      const argList = getLambdaArgs(argLen);
      const lambdaDef = getLambdaDef(argLen, lambda);
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

      args.push(lambdaDef);
      lisp.eval(lisp.list.apply(this, args));
    };

    return function () {
      if (typeof arguments[0] === "object" && arguments[0].name) {
        const arg = arguments[0];
        return makeStatement(arg.name, arg.docString, {
          interactive: arg.interactive,
          args: arg.args,
        }, arg.func);
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
      const argList = getLambdaArgs(numArgs);
      const lambdaDef = getLambdaDef(numArgs, lambda);
      const list = [lisp.q["let"], argList, lambdaDef];

      for (let i = 1; i < arguments.length; ++i) {
        list.push(arguments[i]);
      }

      return lisp.eval(lisp.list.apply(this, args));
    };
  };

  const with_current_buffer = () => {
    return function (bufferOrName, lambda) {
      if (lambda.length !== 0) {
        throw new Exception("with-current-buffer lambda takes 0 arguments");
      }

      const lambdaDef = getLambdaDef(0, lambda);
      const list = [lisp.q.with_current_buffer, bufferOrName, lambdaDef];

      return lisp.eval(lisp.list.apply(this, list));
    };
  };

  const with_temp_buffer = () => {
    return function (lambda) {
      if (lambda.length !== 0) {
        throw new Exception("with-temp-buffer lambda takes 0 arguments");
      }

      const lambdaDef = getLambdaDef(0, lambda);
      const list = [lisp.q.with_temp_buffer, lambdaDef];

      return lisp.eval(lisp.list.apply(this, list));
    };
  };

  const processReturn = (result, knownProxy) => {
    let retval = null;
    if (knownProxy || is_proxy(result)) {
      result.json = () => {
        return JSON.parse(lisp_json(result));
      };

      __weak.push(new WeakRef(result));
      retval = result;
    } else {
      retval = JSON.parse(result);
    }

    return retval;
  };

  // We do not call getOrCacheString on purpose here
  // in case the user manipulates the string's properties
  // Those properties will be on the cache string.
  const processArgs = (dataArr, container) => {
    const retval = container || [];
    for (let i = 0; i < dataArr.length; ++i) {
      if (typeof dataArr[i] === "function") {
        const numArgs = dataArr[i].length;
        const args = getLambdaArgs(numArgs);
        const lambdaDef = getLambdaDef(numArgs, dataArr[i]);
        const lambda = lisp.list(lisp.q.lambda, args, lambdaDef);
        retval.push(lambda);
      } else if (typeof dataArr[i] === "number") {
        let numProxy = getOrCacheNum(dataArr[i]);
        retval.push(numProxy);
      } else if (is_proxy(dataArr[i])) {
        retval.push(dataArr[i]);
      } else {
        retval.push(JSON.stringify(dataArr[i]));
      }
    }

    return retval;
  };

  const _intern = (arg) => {
    let result = lisp_intern(getOrCacheString(arg)) || {};
    return processReturn(result, true);
  };

  const list = function () {
    if (arguments.length === 0) {
      return lisp.q.nil;
    }

    let args = processArgs(arguments);
    return processReturn(lisp_list.apply(this, args), true);
  };

  const evalForm = (symbol, ...args) => {
    let form = [symbol];
    for (let i = 0; i < args.length; ++i) {
      if (
        lisp.listp(args[i]) &&
        !lisp.eq(lisp.q.quote, lisp.car(args[i]))
      ) {
        form.push(quote(args[i]));
      } else {
        form.push(args[i]);
      }
    }

    return lisp.eval(lisp.list.apply(this, form));
  };

  const setq = (...args) => evalForm(lisp.q.setq, ...args);
  const defvar = (...args) => evalForm(lisp.q.defvar, ...args);
  const define_key = (...args) => evalForm(lisp.q.define_key, ...args);
  const define_minor_mode = (...args) =>
    evalForm(lisp.q.define_minor_mode, ...args);

  const define_derived_mode = (...args) =>
    evalForm(lisp.q.define_derived_mode, ...args);

  const specialForms = {
    make: makeFuncs,
    q: symbolsCached(),
    symbols: symbols(),
    setq,
    defun: defun(),
    keywords: keywords(),
    k: keywordsCached(),
    "let": _let(),
    with_current_buffer: with_current_buffer(),
    with_temp_buffer: with_temp_buffer(),
    quote,
    list,
    defvar,
    define_key,
    define_minor_mode,
    define_derived_mode,
  };

  global.lisp = new Proxy({}, {
    get: function (o, k) {
      if (errorFuncs[k]) {
        throw new Error(
          "Attempting to call non-supported function via javascript invokation (" +
            k + ")",
        );
      }

      if (k === "specialForms") {
        return specialForms;
      }

      if (specialForms[k]) {
        return specialForms[k];
      }

      return function () {
        const modargs = [lisp.q[k.replaceAll("_", "-")]];
        processArgs(arguments, modargs);
        const result = lisp_invoke.apply(this, modargs);
        return processReturn(result);
      };
    },
  });

  const varArgsList = lisp.list(lisp.q["&rest"], lisp.q.alpha);
})();
