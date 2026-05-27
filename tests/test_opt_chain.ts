// Optional Chaining Test Suite

let obj = {
    a: {
        b: 42
    },
    greet: function() {
        return "Hello World";
    }
};

let nullObj: any = null;
let undefinedObj: any = undefined;

console.log("Testing optional property access on non-null:");
console.log(obj?.a?.b);

console.log("Testing optional property access on null/undefined:");
console.log(nullObj?.a);
console.log(undefinedObj?.a);

console.log("Testing optional index access:");
let arr = [10, 20];
console.log(arr?.[1]);
let nullArr: any = null;
console.log(nullArr?.[0]);

console.log("Testing optional call:");
console.log(obj.greet?.());
let nullFunc: any = null;
console.log(nullFunc?.());

// Test entire chain short-circuiting: a?.b.c when a is nullish
console.log("Testing entire chain short-circuiting on nullish base:");
let partialNull: any = null;
console.log(partialNull?.b.c); // Should print undefined, not throw!
