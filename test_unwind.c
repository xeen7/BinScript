#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>

struct _Unwind_Exception {
    uint64_t exception_class;
    void (*exception_cleanup)(uint32_t, struct _Unwind_Exception*);
    uint64_t private_1;
    uint64_t private_2;
};

extern uint32_t _Unwind_RaiseException(struct _Unwind_Exception*);

int main() {
    struct _Unwind_Exception* ex = malloc(sizeof(struct _Unwind_Exception));
    ex->exception_class = 0x42696E5363727074; // 'BinScr\x70\x74'
    ex->exception_cleanup = 0;
    
    printf("Calling _Unwind_RaiseException...\n");
    uint32_t res = _Unwind_RaiseException(ex);
    printf("Failed with code: %d\n", res);
    return 0;
}
