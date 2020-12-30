self.onmessage = function(e) {
    self.postMessage({filename: "foo"});
    self.close();
}
