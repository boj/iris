// C shim for Lean FFI — wraps Lean runtime calls for Rust.
#include <lean/lean.h>
#include <pthread.h>
#include <string.h>

extern void lean_initialize_runtime_module(void);
extern void lean_init_task_manager(void);
extern void lean_io_mark_end_initialization(void);
extern lean_obj_res initialize_iris_x2dkernel_IrisKernel_FFI(uint8_t builtin, lean_obj_arg world);

// The @[export] functions — these ARE direct C functions for scalar returns
extern uint8_t iris_check_cost_leq(b_lean_obj_arg input);
extern uint8_t iris_eval_lia(b_lean_obj_arg formula_ba, b_lean_obj_arg env_ba);
extern uint8_t iris_type_check_node(uint8_t kind_tag);

/* Thread-safe initialization state.
 *
 * Issue: the original `static int` flag has no thread safety — two threads
 * calling iris_lean_init() concurrently could both pass the `if` check and
 * initialize the Lean runtime twice, causing undefined behavior.
 *
 * Fix: use pthread_once to guarantee the body runs exactly once, even under
 * concurrent callers. The Rust side also uses std::sync::Once, but we protect
 * the C side independently so the shim is safe regardless of caller context.
 */
static pthread_once_t iris_lean_once = PTHREAD_ONCE_INIT;
static int iris_lean_initialized = 0;

static void iris_lean_init_once(void) {
    lean_initialize_runtime_module();
    lean_init_task_manager();

    lean_obj_res world = lean_io_mk_world();
    lean_obj_res result = initialize_iris_x2dkernel_IrisKernel_FFI(1, world);
    if (lean_io_result_is_error(result)) {
        lean_dec_ref(result);
        iris_lean_initialized = -1;
        return;
    }
    lean_dec_ref(result);
    lean_io_mark_end_initialization();
    iris_lean_initialized = 1;
}

void iris_lean_init(void) {
    pthread_once(&iris_lean_once, iris_lean_init_once);
}

int iris_lean_is_initialized(void) {
    return iris_lean_initialized;
}

/* Maximum buffer size accepted from Rust callers.
 * 1 MiB is well above any realistic encoded CostBound, but prevents a
 * malformed call from triggering an arbitrarily large allocation. */
#define IRIS_MAX_COST_BOUND_BYTES (1024 * 1024)

// Wrapper: create ByteArray, call Lean, return result
uint8_t iris_check_cost_leq_bytes(const uint8_t* data, size_t len) {
    if (iris_lean_initialized != 1) return 2; // not initialized
    /* Bounds check: reject unreasonably large buffers to prevent DoS via
     * enormous allocation or memcpy (issue: C shim no bounds check). */
    if (len > IRIS_MAX_COST_BOUND_BYTES) return 3; // buffer too large
    lean_obj_res arr = lean_alloc_sarray(1, len, len);
    if (len > 0) {
        memcpy(lean_sarray_cptr(arr), data, len);
    }
    uint8_t result = iris_check_cost_leq(arr);
    lean_dec_ref(arr);
    return result;
}

void iris_lean_dec_ref(lean_obj_arg obj) {
    lean_dec_ref(obj);
}
