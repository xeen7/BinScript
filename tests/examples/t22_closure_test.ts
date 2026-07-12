export function main() {
    let localArr = [];
    let f = () => {
        localArr.push(1);
    };
    f();
    
    __bs_print_rc_stats();
    return localArr.length;
}
