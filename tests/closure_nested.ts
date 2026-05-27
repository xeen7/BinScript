const makeAdder = (x: number) => {
    return (y: number) => {
        return x + y;
    };
};
const add5 = makeAdder(5);
console.log(add5(10));
