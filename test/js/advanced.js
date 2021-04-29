Deno.test({
  name: "reverseProxy",
  fn: () => {
    let b = [[555]];
    let x = lisp.make.proxy({ x: 23, y: b, z: { x: 25 } });

    lisp.defun({
      name: "testReverseProxy",
      func: (item) => {
        if (item !== x) {
          throw new Error("Reverse Proxy failed...");
        }
      },
    });
  },
});
