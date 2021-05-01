Deno.test({
  name: "reverseProxy",
  fn: () => {
    let b = [[555]];
    let obj = { x: 23, y: b, z: { x: 25 } };

    lisp.defun({
      name: "testReverseProxy",
      func: (item) => {
        if (item !== obj) {
          throw new Error("Reverse Proxy failed...");
        }
      },
    });

    lisp.testReverseProxy(lisp.make.proxy(obj));
  },
});
