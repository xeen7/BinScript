function customPush(arr: any[], item: any) {
    arr.push(item);
}

function nestedPush(arr: any[], item: any) {
    customPush(arr, item);
}

function superNestedPush(arr: any[], item: any) {
    nestedPush(arr, item);
}

export function main() {
    let localArr = [];
    let localItem1 = { id: 1 };
    let localItem2 = { id: 2 };

    // Cross-function aliasing
    let aliasArr = localArr;

    // Test that pushing via nested helper functions retains Owned status
    // since aliasArr is local and flows all the way through
    superNestedPush(aliasArr, localItem1);
    superNestedPush(aliasArr, localItem2);

    let len = aliasArr.length;
    __bs_print_rc_stats();
    return len;
}
