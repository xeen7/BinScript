async function asyncPush(arr: any[], item: any) {
    arr.push(item);
    return arr;
}

export async function main() {
    let localArr = [];
    let localItem = { id: 100 };
    
    await asyncPush(localArr, localItem);
    
    __bs_print_rc_stats();
    return localArr.length;
}
