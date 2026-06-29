class Widget {
    onClick: any;
    count: number;
    constructor() {
        this.count = 0;
        this.onClick = () => {
            this.count = this.count + 1;
        };
    }
}
function leak() {
    let w = new Widget();
    w.onClick();
}
for (let i = 0; i < 3; i++) {
    leak();
}
