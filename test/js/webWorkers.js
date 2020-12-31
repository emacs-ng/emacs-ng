export function webWorkers() {
    return Promise.resolve()
	.test(() => {
	    let worker = new Worker(new URL("webWorkerModule.js", import.meta.url).href,
				    { type: "module",
				      deno: true,
				    });

	    return new Promise((resolve, reject) => {
		let timer = setTimeout(() => { reject("Test failure due to timeout") }, 5000);
		worker.onmessage = function (e) {
		    clearTimeout(timer);
		    const { filename } = e.data;
		    if (filename !== 'foo') {
			reject("WebWorker async test failed, incorrect data returned");
		    }

		    resolve();
		};



		worker.postMessage({ filename: "./log.txt" });
	    });
	});

}
