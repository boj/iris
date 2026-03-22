/*
 * interpreter.c — CLCU container interpreter
 *
 * Dispatch table with function pointers for each opcode.
 * Implements prefetch on container entry, conditional continuation protocol
 * (SPEC Section 7.4), and core SIMD opcodes with scalar fallbacks.
 *
 * See SPEC.md Sections 9.3, 9.4, 7.4.
 */

#include "clcu.h"

#include <string.h>
#include <math.h>

/* ────────────────────────────────────────────────────────────────────────────
 * Dispatch table — 256-entry function pointer array
 * ──────────────────────────────────────────────────────────────────────────── */
static microop_fn dispatch_table[256];

/* ────────────────────────────────────────────────────────────────────────────
 * Scalar fallback helpers
 *
 * When AVX-512 is not available at compile time, all operations go through
 * scalar int32 arrays of 16 elements (simulating 512-bit ZMM registers).
 * ──────────────────────────────────────────────────────────────────────────── */

#ifdef __AVX512F__

/* ──── AVX-512 opcode implementations ──── */

static void op_nop(exec_state_t *state, uint8_t dst, uint8_t src1,
                   uint8_t src2, uint8_t width, uint8_t mod, uint32_t imm) {
    (void)state; (void)dst; (void)src1; (void)src2;
    (void)width; (void)mod; (void)imm;
}

static void op_vadd(exec_state_t *state, uint8_t dst, uint8_t src1,
                    uint8_t src2, uint8_t width, uint8_t mod, uint32_t imm) {
    (void)width; (void)mod; (void)imm;
    state->zmm[dst] = _mm512_add_epi32(state->zmm[src1], state->zmm[src2]);
}

static void op_vmul(exec_state_t *state, uint8_t dst, uint8_t src1,
                    uint8_t src2, uint8_t width, uint8_t mod, uint32_t imm) {
    (void)width; (void)mod; (void)imm;
    state->zmm[dst] = _mm512_mullo_epi32(state->zmm[src1], state->zmm[src2]);
}

static void op_vfma(exec_state_t *state, uint8_t dst, uint8_t src1,
                    uint8_t src2, uint8_t width, uint8_t mod, uint32_t imm) {
    (void)width; (void)mod; (void)imm;
    /* FMA: dst = src1 * src2 + dst (integer: mullo + add) */
    __m512i product = _mm512_mullo_epi32(state->zmm[src1], state->zmm[src2]);
    state->zmm[dst] = _mm512_add_epi32(product, state->zmm[dst]);
}

static void op_vsub(exec_state_t *state, uint8_t dst, uint8_t src1,
                    uint8_t src2, uint8_t width, uint8_t mod, uint32_t imm) {
    (void)width; (void)mod; (void)imm;
    state->zmm[dst] = _mm512_sub_epi32(state->zmm[src1], state->zmm[src2]);
}

static void op_vcmp(exec_state_t *state, uint8_t dst, uint8_t src1,
                    uint8_t src2, uint8_t width, uint8_t mod, uint32_t imm) {
    (void)width; (void)imm;
    /* dst is treated as mask register index (k0-k7) */
    uint8_t k_idx = dst & 0x7;
    switch (mod) {
    case CMP_EQ:
        state->k[k_idx] = _mm512_cmpeq_epi32_mask(state->zmm[src1], state->zmm[src2]);
        break;
    case CMP_LT:
        state->k[k_idx] = _mm512_cmplt_epi32_mask(state->zmm[src1], state->zmm[src2]);
        break;
    case CMP_LE:
        state->k[k_idx] = _mm512_cmple_epi32_mask(state->zmm[src1], state->zmm[src2]);
        break;
    case CMP_NE:
        state->k[k_idx] = _mm512_cmpneq_epi32_mask(state->zmm[src1], state->zmm[src2]);
        break;
    case CMP_GE:
        state->k[k_idx] = _mm512_cmpge_epi32_mask(state->zmm[src1], state->zmm[src2]);
        break;
    case CMP_GT:
        state->k[k_idx] = _mm512_cmpgt_epi32_mask(state->zmm[src1], state->zmm[src2]);
        break;
    default:
        state->k[k_idx] = _mm512_cmpeq_epi32_mask(state->zmm[src1], state->zmm[src2]);
        break;
    }
}

static void op_vblend(exec_state_t *state, uint8_t dst, uint8_t src1,
                      uint8_t src2, uint8_t width, uint8_t mod, uint32_t imm) {
    (void)width; (void)imm;
    /* Blend src1 and src2 using mask register k[mod & 7] */
    uint8_t k_idx = mod & 0x7;
    state->zmm[dst] = _mm512_mask_blend_epi32(state->k[k_idx],
                                               state->zmm[src2],
                                               state->zmm[src1]);
}

static void op_vload(exec_state_t *state, uint8_t dst, uint8_t src1,
                     uint8_t src2, uint8_t width, uint8_t mod, uint32_t imm) {
    (void)src2; (void)width; (void)mod;
    /* Load from address: base in zmm[src1] lane 0, offset in immediate */
    int32_t base_val;
    _mm512_mask_storeu_epi32(&base_val, 1, state->zmm[src1]);
    const void *addr = (const void *)((uintptr_t)base_val + imm);
    state->zmm[dst] = _mm512_loadu_si512(addr);
}

static void op_vstore(exec_state_t *state, uint8_t dst, uint8_t src1,
                      uint8_t src2, uint8_t width, uint8_t mod, uint32_t imm) {
    (void)src2; (void)width; (void)mod;
    /* Store: base address from zmm[dst] lane 0, data from zmm[src1] */
    int32_t base_val;
    _mm512_mask_storeu_epi32(&base_val, 1, state->zmm[dst]);
    void *addr = (void *)((uintptr_t)base_val + imm);
    _mm512_storeu_si512(addr, state->zmm[src1]);
}

static void op_vreduce(exec_state_t *state, uint8_t dst, uint8_t src1,
                       uint8_t src2, uint8_t width, uint8_t mod, uint32_t imm) {
    (void)src2; (void)width; (void)imm;
    int32_t result;
    switch (mod) {
    case REDUCE_ADD:
        result = _mm512_reduce_add_epi32(state->zmm[src1]);
        break;
    case REDUCE_MIN:
        result = _mm512_reduce_min_epi32(state->zmm[src1]);
        break;
    case REDUCE_MAX:
        result = _mm512_reduce_max_epi32(state->zmm[src1]);
        break;
    case REDUCE_AND:
        result = _mm512_reduce_and_epi32(state->zmm[src1]);
        break;
    case REDUCE_OR:
        result = _mm512_reduce_or_epi32(state->zmm[src1]);
        break;
    case REDUCE_MUL:
        result = _mm512_reduce_mul_epi32(state->zmm[src1]);
        break;
    default:
        result = _mm512_reduce_add_epi32(state->zmm[src1]);
        break;
    }
    state->zmm[dst] = _mm512_set1_epi32(result);
}

static void op_vpermute(exec_state_t *state, uint8_t dst, uint8_t src1,
                        uint8_t src2, uint8_t width, uint8_t mod, uint32_t imm) {
    (void)width; (void)mod; (void)imm;
    /* Permute src1 using indices in src2 */
    state->zmm[dst] = _mm512_permutexvar_epi32(state->zmm[src2], state->zmm[src1]);
}

static void op_vbroadcast(exec_state_t *state, uint8_t dst, uint8_t src1,
                          uint8_t src2, uint8_t width, uint8_t mod, uint32_t imm) {
    (void)src1; (void)src2; (void)width; (void)mod;
    /* Broadcast 24-bit sign-extended immediate to all lanes */
    int32_t val = (int32_t)(imm | ((imm & 0x800000) ? 0xFF000000 : 0));
    state->zmm[dst] = _mm512_set1_epi32(val);
}

static void op_vand(exec_state_t *state, uint8_t dst, uint8_t src1,
                    uint8_t src2, uint8_t width, uint8_t mod, uint32_t imm) {
    (void)width; (void)mod; (void)imm;
    state->zmm[dst] = _mm512_and_si512(state->zmm[src1], state->zmm[src2]);
}

static void op_vor(exec_state_t *state, uint8_t dst, uint8_t src1,
                   uint8_t src2, uint8_t width, uint8_t mod, uint32_t imm) {
    (void)width; (void)mod; (void)imm;
    state->zmm[dst] = _mm512_or_si512(state->zmm[src1], state->zmm[src2]);
}

static void op_vxor(exec_state_t *state, uint8_t dst, uint8_t src1,
                    uint8_t src2, uint8_t width, uint8_t mod, uint32_t imm) {
    (void)width; (void)mod; (void)imm;
    state->zmm[dst] = _mm512_xor_si512(state->zmm[src1], state->zmm[src2]);
}

static void op_vshl(exec_state_t *state, uint8_t dst, uint8_t src1,
                    uint8_t src2, uint8_t width, uint8_t mod, uint32_t imm) {
    (void)width; (void)mod;
    if (imm > 0) {
        /* Shift by immediate */
        state->zmm[dst] = _mm512_slli_epi32(state->zmm[src1], (int)imm);
    } else {
        /* Shift by variable amounts in src2 */
        state->zmm[dst] = _mm512_sllv_epi32(state->zmm[src1], state->zmm[src2]);
    }
}

static void op_vshr(exec_state_t *state, uint8_t dst, uint8_t src1,
                    uint8_t src2, uint8_t width, uint8_t mod, uint32_t imm) {
    (void)width; (void)mod;
    if (imm > 0) {
        state->zmm[dst] = _mm512_srli_epi32(state->zmm[src1], (int)imm);
    } else {
        state->zmm[dst] = _mm512_srlv_epi32(state->zmm[src1], state->zmm[src2]);
    }
}

static void op_vmin(exec_state_t *state, uint8_t dst, uint8_t src1,
                    uint8_t src2, uint8_t width, uint8_t mod, uint32_t imm) {
    (void)width; (void)mod; (void)imm;
    state->zmm[dst] = _mm512_min_epi32(state->zmm[src1], state->zmm[src2]);
}

static void op_vmax(exec_state_t *state, uint8_t dst, uint8_t src1,
                    uint8_t src2, uint8_t width, uint8_t mod, uint32_t imm) {
    (void)width; (void)mod; (void)imm;
    state->zmm[dst] = _mm512_max_epi32(state->zmm[src1], state->zmm[src2]);
}

static void op_vneg(exec_state_t *state, uint8_t dst, uint8_t src1,
                    uint8_t src2, uint8_t width, uint8_t mod, uint32_t imm) {
    (void)src2; (void)width; (void)mod; (void)imm;
    __m512i zero = _mm512_setzero_si512();
    state->zmm[dst] = _mm512_sub_epi32(zero, state->zmm[src1]);
}

static void op_vabs(exec_state_t *state, uint8_t dst, uint8_t src1,
                    uint8_t src2, uint8_t width, uint8_t mod, uint32_t imm) {
    (void)src2; (void)width; (void)mod; (void)imm;
    state->zmm[dst] = _mm512_abs_epi32(state->zmm[src1]);
}

#else /* !__AVX512F__ — scalar fallback */

/* ──── Scalar opcode implementations ──── */

static void op_nop(exec_state_t *state, uint8_t dst, uint8_t src1,
                   uint8_t src2, uint8_t width, uint8_t mod, uint32_t imm) {
    (void)state; (void)dst; (void)src1; (void)src2;
    (void)width; (void)mod; (void)imm;
}

static void op_vadd(exec_state_t *state, uint8_t dst, uint8_t src1,
                    uint8_t src2, uint8_t width, uint8_t mod, uint32_t imm) {
    (void)width; (void)mod; (void)imm;
    for (int i = 0; i < 16; i++)
        state->zmm[dst][i] = state->zmm[src1][i] + state->zmm[src2][i];
}

static void op_vmul(exec_state_t *state, uint8_t dst, uint8_t src1,
                    uint8_t src2, uint8_t width, uint8_t mod, uint32_t imm) {
    (void)width; (void)mod; (void)imm;
    for (int i = 0; i < 16; i++)
        state->zmm[dst][i] = state->zmm[src1][i] * state->zmm[src2][i];
}

static void op_vfma(exec_state_t *state, uint8_t dst, uint8_t src1,
                    uint8_t src2, uint8_t width, uint8_t mod, uint32_t imm) {
    (void)width; (void)mod; (void)imm;
    for (int i = 0; i < 16; i++)
        state->zmm[dst][i] = state->zmm[src1][i] * state->zmm[src2][i] + state->zmm[dst][i];
}

static void op_vsub(exec_state_t *state, uint8_t dst, uint8_t src1,
                    uint8_t src2, uint8_t width, uint8_t mod, uint32_t imm) {
    (void)width; (void)mod; (void)imm;
    for (int i = 0; i < 16; i++)
        state->zmm[dst][i] = state->zmm[src1][i] - state->zmm[src2][i];
}

static void op_vcmp(exec_state_t *state, uint8_t dst, uint8_t src1,
                    uint8_t src2, uint8_t width, uint8_t mod, uint32_t imm) {
    (void)width; (void)imm;
    uint8_t k_idx = dst & 0x7;
    uint16_t mask = 0;
    for (int i = 0; i < 16; i++) {
        int cond = 0;
        switch (mod) {
        case CMP_EQ: cond = (state->zmm[src1][i] == state->zmm[src2][i]); break;
        case CMP_NE: cond = (state->zmm[src1][i] != state->zmm[src2][i]); break;
        case CMP_LT: cond = (state->zmm[src1][i] <  state->zmm[src2][i]); break;
        case CMP_LE: cond = (state->zmm[src1][i] <= state->zmm[src2][i]); break;
        case CMP_GT: cond = (state->zmm[src1][i] >  state->zmm[src2][i]); break;
        case CMP_GE: cond = (state->zmm[src1][i] >= state->zmm[src2][i]); break;
        default:     cond = (state->zmm[src1][i] == state->zmm[src2][i]); break;
        }
        if (cond) mask |= (1u << i);
    }
    state->k[k_idx] = mask;
}

static void op_vblend(exec_state_t *state, uint8_t dst, uint8_t src1,
                      uint8_t src2, uint8_t width, uint8_t mod, uint32_t imm) {
    (void)width; (void)imm;
    uint8_t k_idx = mod & 0x7;
    for (int i = 0; i < 16; i++) {
        state->zmm[dst][i] = (state->k[k_idx] & (1u << i))
                              ? state->zmm[src1][i]
                              : state->zmm[src2][i];
    }
}

static void op_vload(exec_state_t *state, uint8_t dst, uint8_t src1,
                     uint8_t src2, uint8_t width, uint8_t mod, uint32_t imm) {
    (void)src2; (void)width; (void)mod;
    const void *addr = (const void *)((uintptr_t)state->zmm[src1][0] + imm);
    memcpy(state->zmm[dst], addr, 64);
}

static void op_vstore(exec_state_t *state, uint8_t dst, uint8_t src1,
                      uint8_t src2, uint8_t width, uint8_t mod, uint32_t imm) {
    (void)src2; (void)width; (void)mod;
    void *addr = (void *)((uintptr_t)state->zmm[dst][0] + imm);
    memcpy(addr, state->zmm[src1], 64);
}

static void op_vreduce(exec_state_t *state, uint8_t dst, uint8_t src1,
                       uint8_t src2, uint8_t width, uint8_t mod, uint32_t imm) {
    (void)src2; (void)width; (void)imm;
    int32_t result = state->zmm[src1][0];
    for (int i = 1; i < 16; i++) {
        switch (mod) {
        case REDUCE_ADD: result += state->zmm[src1][i]; break;
        case REDUCE_MUL: result *= state->zmm[src1][i]; break;
        case REDUCE_MIN: if (state->zmm[src1][i] < result) result = state->zmm[src1][i]; break;
        case REDUCE_MAX: if (state->zmm[src1][i] > result) result = state->zmm[src1][i]; break;
        case REDUCE_AND: result &= state->zmm[src1][i]; break;
        case REDUCE_OR:  result |= state->zmm[src1][i]; break;
        case REDUCE_XOR: result ^= state->zmm[src1][i]; break;
        default:         result += state->zmm[src1][i]; break;
        }
    }
    for (int i = 0; i < 16; i++)
        state->zmm[dst][i] = result;
}

static void op_vpermute(exec_state_t *state, uint8_t dst, uint8_t src1,
                        uint8_t src2, uint8_t width, uint8_t mod, uint32_t imm) {
    (void)width; (void)mod; (void)imm;
    int32_t tmp[16];
    for (int i = 0; i < 16; i++)
        tmp[i] = state->zmm[src1][state->zmm[src2][i] & 0xF];
    memcpy(state->zmm[dst], tmp, 64);
}

static void op_vbroadcast(exec_state_t *state, uint8_t dst, uint8_t src1,
                          uint8_t src2, uint8_t width, uint8_t mod, uint32_t imm) {
    (void)src1; (void)src2; (void)width; (void)mod;
    int32_t val = (int32_t)(imm | ((imm & 0x800000) ? 0xFF000000 : 0));
    for (int i = 0; i < 16; i++)
        state->zmm[dst][i] = val;
}

static void op_vand(exec_state_t *state, uint8_t dst, uint8_t src1,
                    uint8_t src2, uint8_t width, uint8_t mod, uint32_t imm) {
    (void)width; (void)mod; (void)imm;
    for (int i = 0; i < 16; i++)
        state->zmm[dst][i] = state->zmm[src1][i] & state->zmm[src2][i];
}

static void op_vor(exec_state_t *state, uint8_t dst, uint8_t src1,
                   uint8_t src2, uint8_t width, uint8_t mod, uint32_t imm) {
    (void)width; (void)mod; (void)imm;
    for (int i = 0; i < 16; i++)
        state->zmm[dst][i] = state->zmm[src1][i] | state->zmm[src2][i];
}

static void op_vxor(exec_state_t *state, uint8_t dst, uint8_t src1,
                    uint8_t src2, uint8_t width, uint8_t mod, uint32_t imm) {
    (void)width; (void)mod; (void)imm;
    for (int i = 0; i < 16; i++)
        state->zmm[dst][i] = state->zmm[src1][i] ^ state->zmm[src2][i];
}

static void op_vshl(exec_state_t *state, uint8_t dst, uint8_t src1,
                    uint8_t src2, uint8_t width, uint8_t mod, uint32_t imm) {
    (void)width; (void)mod;
    for (int i = 0; i < 16; i++) {
        int shift = (imm > 0) ? (int)imm : state->zmm[src2][i];
        state->zmm[dst][i] = (shift < 32) ? (state->zmm[src1][i] << shift) : 0;
    }
}

static void op_vshr(exec_state_t *state, uint8_t dst, uint8_t src1,
                    uint8_t src2, uint8_t width, uint8_t mod, uint32_t imm) {
    (void)width; (void)mod;
    for (int i = 0; i < 16; i++) {
        int shift = (imm > 0) ? (int)imm : state->zmm[src2][i];
        state->zmm[dst][i] = (shift < 32)
            ? (int32_t)((uint32_t)state->zmm[src1][i] >> shift)
            : 0;
    }
}

static void op_vmin(exec_state_t *state, uint8_t dst, uint8_t src1,
                    uint8_t src2, uint8_t width, uint8_t mod, uint32_t imm) {
    (void)width; (void)mod; (void)imm;
    for (int i = 0; i < 16; i++)
        state->zmm[dst][i] = (state->zmm[src1][i] < state->zmm[src2][i])
                              ? state->zmm[src1][i] : state->zmm[src2][i];
}

static void op_vmax(exec_state_t *state, uint8_t dst, uint8_t src1,
                    uint8_t src2, uint8_t width, uint8_t mod, uint32_t imm) {
    (void)width; (void)mod; (void)imm;
    for (int i = 0; i < 16; i++)
        state->zmm[dst][i] = (state->zmm[src1][i] > state->zmm[src2][i])
                              ? state->zmm[src1][i] : state->zmm[src2][i];
}

static void op_vneg(exec_state_t *state, uint8_t dst, uint8_t src1,
                    uint8_t src2, uint8_t width, uint8_t mod, uint32_t imm) {
    (void)src2; (void)width; (void)mod; (void)imm;
    for (int i = 0; i < 16; i++)
        state->zmm[dst][i] = -state->zmm[src1][i];
}

static void op_vabs(exec_state_t *state, uint8_t dst, uint8_t src1,
                    uint8_t src2, uint8_t width, uint8_t mod, uint32_t imm) {
    (void)src2; (void)width; (void)mod; (void)imm;
    for (int i = 0; i < 16; i++)
        state->zmm[dst][i] = (state->zmm[src1][i] < 0)
                              ? -state->zmm[src1][i] : state->zmm[src1][i];
}

#endif /* __AVX512F__ */

/* ────────────────────────────────────────────────────────────────────────────
 * Default handler for unimplemented opcodes — treated as NOP
 * ──────────────────────────────────────────────────────────────────────────── */
static void op_unimplemented(exec_state_t *state, uint8_t dst, uint8_t src1,
                             uint8_t src2, uint8_t width, uint8_t mod, uint32_t imm) {
    (void)state; (void)dst; (void)src1; (void)src2;
    (void)width; (void)mod; (void)imm;
}

/* ────────────────────────────────────────────────────────────────────────────
 * interpreter_init — populate the dispatch table
 * ──────────────────────────────────────────────────────────────────────────── */
void interpreter_init(void) {
    /* Fill everything with unimplemented (NOP-like) handler */
    for (int i = 0; i < 256; i++)
        dispatch_table[i] = op_unimplemented;

    /* Arithmetic / vector compute */
    dispatch_table[OP_NOP]        = op_nop;
    dispatch_table[OP_VADD]       = op_vadd;
    dispatch_table[OP_VMUL]       = op_vmul;
    dispatch_table[OP_VFMA]       = op_vfma;
    dispatch_table[OP_VSUB]       = op_vsub;
    dispatch_table[OP_VNEG]       = op_vneg;
    dispatch_table[OP_VABS]       = op_vabs;
    dispatch_table[OP_VMIN]       = op_vmin;
    dispatch_table[OP_VMAX]       = op_vmax;
    dispatch_table[OP_VREDUCE]    = op_vreduce;
    dispatch_table[OP_VBROADCAST] = op_vbroadcast;
    dispatch_table[OP_VPERMUTE]   = op_vpermute;
    dispatch_table[OP_VBLEND]     = op_vblend;

    /* Bitwise / shifts */
    dispatch_table[OP_VAND]       = op_vand;
    dispatch_table[OP_VOR]        = op_vor;
    dispatch_table[OP_VXOR]       = op_vxor;
    dispatch_table[OP_VSHL]       = op_vshl;
    dispatch_table[OP_VSHR]       = op_vshr;

    /* Comparison / mask generation */
    dispatch_table[OP_VCMP]       = op_vcmp;

    /* Memory / data movement */
    dispatch_table[OP_VLOAD]      = op_vload;
    dispatch_table[OP_VSTORE]     = op_vstore;
}

/* ────────────────────────────────────────────────────────────────────────────
 * execute_container — process one 64-byte container
 *
 * 1. Issue PREFETCHT0 on entry if prefetch_valid
 * 2. Decode and dispatch up to 8 micro-ops from payload
 * ──────────────────────────────────────────────────────────────────────────── */
__attribute__((hot))
void execute_container(exec_state_t *state, const clcu_container_t *c) {
    /* Prefetch on container entry */
    if (c->flags & CLCU_FLAG_PREFETCH_VALID) {
        const void *pf_target = (const char *)c + ((int64_t)c->prefetch_target * 64);
        __builtin_prefetch(pf_target, 0, 3);
    }

    /* Decode and execute up to 8 micro-ops from payload */
    const uint8_t *payload = c->payload;
    for (int i = 0; i < CLCU_MAX_MICRO_OPS; i++) {
        const uint8_t *op = payload + (i * CLCU_MICRO_OP_SIZE);
        uint8_t opcode = uop_opcode(op);
        if (opcode == OP_NOP) break;  /* NOP terminates micro-op stream */

        uint8_t  dst   = uop_dst(op);
        uint8_t  src1  = uop_src1(op);
        uint8_t  src2  = uop_src2(op);
        uint8_t  width = uop_width(op);
        uint8_t  mod   = uop_modifier(op);
        uint32_t imm   = uop_immediate(op);

        dispatch_table[opcode](state, dst, src1, src2, width, mod, imm);
    }

    state->containers_executed++;
}

/* ────────────────────────────────────────────────────────────────────────────
 * execute_chain — follow continuation chain with conditional continuation
 *
 * Conditional continuation protocol (SPEC Section 7.4):
 *   When flags.mask_source == 01 (register-based mask):
 *     - Read mask register indexed by (mask_immediate & 0x07)
 *     - If mask is all-zeros: follow next_container (backedge / loop continues)
 *     - If mask is non-zero: follow prefetch_target (forward exit)
 *
 *   When mask_source == 00: unconditional follow next_container
 * ──────────────────────────────────────────────────────────────────────────── */
__attribute__((hot))
void execute_chain(exec_state_t *state, const clcu_container_t *entry) {
    const clcu_container_t *c = entry;

    while (c != NULL) {
        state->pc = c;
        execute_container(state, c);

        /* Check for terminal / yield sentinels */
        if (c->next_container == CLCU_NEXT_TERMINAL) {
            state->status_flags |= EXEC_STATUS_HALTED;
            break;
        }
        if (c->next_container == CLCU_NEXT_YIELD) {
            state->status_flags |= EXEC_STATUS_YIELDED;
            break;
        }

        /* Conditional continuation protocol (Section 7.4) */
        uint8_t mask_source = (c->flags & CLCU_FLAG_MASK_SOURCE_MASK) >> 2;

        if (mask_source == 1) {
            /* Register-based mask: read k[mask_immediate & 7] */
            uint8_t k_idx = c->mask_immediate & 0x07;
            uint16_t mask_val = state->k[k_idx];

            if (mask_val == 0) {
                /* All-zeros: follow next_container (backedge, loop continues) */
                c = (const clcu_container_t *)((const char *)c +
                    (int64_t)c->next_container * 64);
            } else {
                /* Non-zero: follow prefetch_target as alt continuation (forward exit) */
                c = (const clcu_container_t *)((const char *)c +
                    (int64_t)c->prefetch_target * 64);
            }
        } else {
            /* Unconditional continuation: follow next_container */
            if (c->next_container == CLCU_NEXT_SEQUENTIAL) {
                c = c + 1;  /* Next cache line */
            } else {
                c = (const clcu_container_t *)((const char *)c +
                    (int64_t)c->next_container * 64);
            }
        }
    }
}
