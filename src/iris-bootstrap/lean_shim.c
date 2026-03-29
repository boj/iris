// C shim for Lean FFI — wraps Lean runtime calls for Rust.
#include <lean/lean.h>
#include <pthread.h>
#include <string.h>

extern void lean_initialize_runtime_module(void);
extern void lean_init_task_manager(void);
extern void lean_io_mark_end_initialization(void);
extern lean_obj_res initialize_iris_x2dkernel_IrisKernel_FFI(uint8_t builtin, lean_obj_arg world);

// The @[export] functions — scalar returns
extern uint8_t iris_check_cost_leq(b_lean_obj_arg input);
extern uint8_t iris_eval_lia(b_lean_obj_arg formula_ba, b_lean_obj_arg env_ba);
extern uint8_t iris_type_check_node(uint8_t kind_tag);

// The @[export] functions — ByteArray returns for kernel rules
extern lean_obj_res iris_kernel_assume(b_lean_obj_arg data);
extern lean_obj_res iris_kernel_intro(b_lean_obj_arg data);
extern lean_obj_res iris_kernel_elim(b_lean_obj_arg data);
extern lean_obj_res iris_kernel_refl(b_lean_obj_arg data);
extern lean_obj_res iris_kernel_symm(b_lean_obj_arg data);
extern lean_obj_res iris_kernel_trans(b_lean_obj_arg data);
extern lean_obj_res iris_kernel_congr(b_lean_obj_arg data);
extern lean_obj_res iris_kernel_type_check_node_full(b_lean_obj_arg data);
extern lean_obj_res iris_kernel_cost_subsume(b_lean_obj_arg data);
extern lean_obj_res iris_kernel_cost_leq_rule(b_lean_obj_arg data);
extern lean_obj_res iris_kernel_refine_intro(b_lean_obj_arg data);
extern lean_obj_res iris_kernel_refine_elim(b_lean_obj_arg data);
extern lean_obj_res iris_kernel_nat_ind(b_lean_obj_arg data);
extern lean_obj_res iris_kernel_structural_ind(b_lean_obj_arg data);
extern lean_obj_res iris_kernel_let_bind(b_lean_obj_arg data);
extern lean_obj_res iris_kernel_match_elim(b_lean_obj_arg data);
extern lean_obj_res iris_kernel_fold_rule(b_lean_obj_arg data);
extern lean_obj_res iris_kernel_type_abst(b_lean_obj_arg data);
extern lean_obj_res iris_kernel_type_app(b_lean_obj_arg data);
extern lean_obj_res iris_kernel_guard_rule(b_lean_obj_arg data);

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

/* Generic wrapper for kernel rule FFI calls that take a ByteArray
 * and return a ByteArray.
 *
 * Parameters:
 *   rule_fn  — function pointer to the Lean-exported rule
 *   data     — raw byte input from Rust
 *   len      — input length
 *   out_data — [out] pointer to output bytes (caller must free with iris_lean_free_bytes)
 *   out_len  — [out] output length
 *
 * Returns:
 *   0 on success (out_data/out_len set)
 *   1 if not initialized
 *   2 if buffer too large
 */
int iris_kernel_rule_bytes(
    lean_obj_res (*rule_fn)(b_lean_obj_arg),
    const uint8_t* data, size_t len,
    uint8_t** out_data, size_t* out_len)
{
    if (iris_lean_initialized != 1) return 1;
    if (len > IRIS_MAX_COST_BOUND_BYTES) return 2;

    lean_obj_res arr = lean_alloc_sarray(1, len, len);
    if (len > 0) {
        memcpy(lean_sarray_cptr(arr), data, len);
    }

    lean_obj_res result = rule_fn(arr);
    lean_dec_ref(arr);

    /* The result is a Lean ByteArray (scalar array of UInt8). */
    size_t result_len = lean_sarray_size(result);
    uint8_t* result_ptr = (uint8_t*)lean_sarray_cptr(result);

    /* Copy result to a C-allocated buffer so Rust can manage it. */
    *out_len = result_len;
    *out_data = (uint8_t*)malloc(result_len);
    if (*out_data != NULL && result_len > 0) {
        memcpy(*out_data, result_ptr, result_len);
    }

    lean_dec_ref(result);
    return 0;
}

/* Free bytes allocated by iris_kernel_rule_bytes. */
void iris_lean_free_bytes(uint8_t* ptr) {
    free(ptr);
}

/* Convenience wrappers for each rule. */
#define DEFINE_RULE_WRAPPER(name) \
    int name##_bytes(const uint8_t* data, size_t len, \
                     uint8_t** out_data, size_t* out_len) { \
        return iris_kernel_rule_bytes(name, data, len, out_data, out_len); \
    }

DEFINE_RULE_WRAPPER(iris_kernel_assume)
DEFINE_RULE_WRAPPER(iris_kernel_intro)
DEFINE_RULE_WRAPPER(iris_kernel_elim)
DEFINE_RULE_WRAPPER(iris_kernel_refl)
DEFINE_RULE_WRAPPER(iris_kernel_symm)
DEFINE_RULE_WRAPPER(iris_kernel_trans)
DEFINE_RULE_WRAPPER(iris_kernel_congr)
DEFINE_RULE_WRAPPER(iris_kernel_type_check_node_full)
DEFINE_RULE_WRAPPER(iris_kernel_cost_subsume)
DEFINE_RULE_WRAPPER(iris_kernel_cost_leq_rule)
DEFINE_RULE_WRAPPER(iris_kernel_refine_intro)
DEFINE_RULE_WRAPPER(iris_kernel_refine_elim)
DEFINE_RULE_WRAPPER(iris_kernel_nat_ind)
DEFINE_RULE_WRAPPER(iris_kernel_structural_ind)
DEFINE_RULE_WRAPPER(iris_kernel_let_bind)
DEFINE_RULE_WRAPPER(iris_kernel_match_elim)
DEFINE_RULE_WRAPPER(iris_kernel_fold_rule)
DEFINE_RULE_WRAPPER(iris_kernel_type_abst)
DEFINE_RULE_WRAPPER(iris_kernel_type_app)
DEFINE_RULE_WRAPPER(iris_kernel_guard_rule)

void iris_lean_dec_ref(lean_obj_arg obj) {
    lean_dec_ref(obj);
}
