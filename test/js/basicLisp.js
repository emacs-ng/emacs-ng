Deno.test({
  name: "lispAllocations",
  fn: () => {
    let plist = lisp.make.plist({ x: 3, y: 4, z: lisp.symbols.qqz });
    let alist = lisp.make.alist({
      s: 5,
      y: "Hello World",
      z: lisp.symbols.qq,
    });

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
  },
});
Deno.test({
  name: "symbols",
  fn: () => {
    let p = lisp.symbols.a;
    let qq = lisp.symbols.qq;
    let qc = lisp.keywords.word;
    lisp.setq(p, "hello");
    lisp.setq(qq, lisp.list(1, 2, 3));
    if (lisp.symbol_value(p) !== "hello") {
      throw new Error("Failure in test lisp.setq");
    }
  },
});
Deno.test({
  name: "basicDefun",
  fn: () => {
    let mutated = 0;
    let myFunc = lisp.defun("hello", (arg, arg2) => {
      mutated = arg;
      return 4;
    });
    lisp.hello(1, 3);

    let myFuncTwo = lisp.defun("helloTwo", "myDocString", (arg) => {
      lisp.hello(1, 2);
      return arg;
    });
    lisp.helloTwo(2);

    let myFuncThree = lisp.defun("helloThree", {
      interactive: true,
      arg: "P\nbbuffer:",
    }, (arg) => {
      lisp.helloTwo(1);
    });
    lisp.helloThree(3);

    let myFuncFour = lisp.defun("helloFour", "HelloFour", {
      interactive: true,
    }, () => {});
    lisp.helloFour();

    let myFuncFive = lisp.defun({
      name: "helloFive",
      docString: "My Cool Function",
      interactive: true,
      args: "p",
      func: (a) => {
        return (a);
      },
    });

    if (lisp.helloFive(1) !== 1) {
      throw new Error("Return Value incorrect for Defun");
    }

    if (mutated !== 1) {
      throw new Error(
        "Failure in test Defun: Mutated value not set from callback",
      );
    }
  },
});
Deno.test({
  name: "pipeProcess",
  fn: () => {
    lisp.make_pipe_process(lisp.keywords.name, "mybuff");
  },
});
Deno.test({
  name: "setqLogic",
  fn: () => {
    lisp.let((a) => {
      if (a !== 3) {
        throw new Error("Arguments do not match");
      }

      lisp.setq(lisp.symbols.bbb, "3");

      if (lisp.symbol_value(lisp.symbols.bbb) !== "3") {
        throw new Error("Value not set within let.");
      }
    }, 3);
  },
});
Deno.test({
  name: "withCurrentBuffer",
  fn: () => {
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

    if (
      lisp.buffer_string() === "Hello" || lisp.buffer_string() !== "World,,,"
    ) {
      throw new Error(
        "with-current-buffer did not properly retain buffer state",
      );
    }
  },
});
Deno.test({
  name: "bufferString",
  fn: () => {
    let executed = false;
    lisp.with_temp_buffer(() => {
      executed = true;
      lisp.insert("XXX12345");
    });

    if (lisp.buffer_string() === "XXX12345") {
      throw new Error(
        "Error in with-temp-buffer test: mutated normal buffer",
      );
    }

    if (!executed) {
      throw new Error("with-temp-buffer failed to execute");
    }
  },
});
Deno.test({
  name: "denoAdditions",
  async fn() {
    const minPowTwo = 2;
    const maxPowTwo = 1024 * 64;

    const END_MARKER = "b";
    const POW_TWO_MARKER = "m";
    const CHAR_MARKER = "a";

    const logIfNotProperEntry = (n, len) => {
      const end = String.fromCharCode(n[len - 1]);
      if (end !== END_MARKER) {
        throw new Error(
          `String for file input size ${len} not properly read, invalid char at position ${n
            .length - 1}, ${END_MARKER} !== ${end}`,
        );
        return;
      }

      for (let k = 0; k < len - 1; ++k) {
        const charAtKPos = String.fromCharCode(n[k]);
        if (k > 1 && (k & (k - 1)) === 0) {
          if (charAtKPos !== POW_TWO_MARKER) {
            throw new Error(
              `String for file input size ${len} not properly read, invalid char at position ${k}, ${POW_TWO_MARKER} !== ${charAtKPos}`,
            );
            return;
          }
        } else if (charAtKPos !== CHAR_MARKER) {
          throw new Error(
            `String for file input size ${len} not properly read, invalid char at position ${k}, ${CHAR_MARKER} !== ${charAtKPos}`,
          );
          return;
        }
      }
    };

    const textWithBufferSize = async (file, bufferSize, maxSize) => {
      await Deno.seek(file.rid, 0, Deno.SeekMode.Start);

      let finalBuffer = null;
      let bytesRead = 0;
      while (bytesRead < maxSize) {
        const slice = new Uint8Array(bufferSize);
        const read = await Deno.read(file.rid, slice);
        if (read === null) {
          break;
        }

        bytesRead += read;
        if (finalBuffer === null) {
          finalBuffer = slice.subarray(0, read);
        } else {
          const len = finalBuffer.length;
          const subslice = slice.subarray(0, read);
          const buffer = new Uint8Array(len + read);
          buffer.set(finalBuffer, 0);
          buffer.set(subslice, len);
          finalBuffer = buffer;
        }
      }

      const textBuffer = finalBuffer.subarray(0, bytesRead);
      const text = decoder.decode(textBuffer);
      logIfNotProperEntry(textBuffer, bytesRead);
    };

    const encoder = new TextEncoder();
    const decoder = new TextDecoder();
    for (let i = minPowTwo; i <= maxPowTwo; i *= 2) {
      const fileName = await Deno.makeTempFile();
      const file = await Deno.open(
        fileName,
        {
          read: true,
          write: true,
          readMightBlock: true,
          writeMightBlock: true,
        },
      );
      let data = "";
      for (let j = 0; j < i - 1; ++j) {
        if (j > 1 && (j & (j - 1)) === 0) {
          data += POW_TWO_MARKER;
        } else {
          data += CHAR_MARKER;
        }
      }

      data += END_MARKER;

      const input = encoder.encode(data);
      await Deno.writeAll(file, input);
      await textWithBufferSize(file, i, i);
      await textWithBufferSize(file, 2 * i, i);
      await textWithBufferSize(file, 300, i);
      await textWithBufferSize(file, 1024 * 16, i);
      await textWithBufferSize(file, (1024 * 16) - 1, i);
      await textWithBufferSize(file, 1024 * 32, i);

      await Deno.close(file.rid);
      await Deno.remove(fileName);
    }
  },
});
