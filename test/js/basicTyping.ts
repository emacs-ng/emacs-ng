export function basicTyping() {
    return Promise.resolve()
	.then(() => {
	    let x: string = 'hello';
	    let y: number = 4;
	    let lambda = (x: number, y: string): number => {
		return 4;
	    };

	    lambda(y, x);
	});
}
