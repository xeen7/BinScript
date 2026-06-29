class Resource {
    data: number;
    constructor(data: number) {
        this.data = data;
    }
}

function test_finalizer() {
    let registry = new FinalizationRegistry((heldValue: any) => {
        console.log("Finalizer callback executed with heldValue:", heldValue);
    });

    let res = new Resource(99);
    registry.register(res, "Resource99-HeldValue");

    // res goes out of scope and will be garbage collected
}

test_finalizer();
