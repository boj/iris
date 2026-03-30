// Lean compiler output
// Module: IrisKernel.FFI
// Imports: public import Init public import IrisKernel.Types public import IrisKernel.Eval public import IrisKernel.Kernel
#include <lean/lean.h>
#if defined(__clang__)
#pragma clang diagnostic ignored "-Wunused-parameter"
#pragma clang diagnostic ignored "-Wunused-label"
#elif defined(__GNUC__) && !defined(__CLANG__)
#pragma GCC diagnostic ignored "-Wunused-parameter"
#pragma GCC diagnostic ignored "-Wunused-label"
#pragma GCC diagnostic ignored "-Wunused-but-set-variable"
#endif
#ifdef __cplusplus
extern "C" {
#endif
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_decodeLIAAtom(lean_object*);
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_checkCostLeqFFI___boxed(lean_object*);
LEAN_EXPORT uint8_t iris_type_check_node(uint8_t, uint64_t, uint64_t);
static lean_object* lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__8;
lean_object* lp_iris_x2dkernel_IrisKernel_Kernel_letBind(lean_object*, lean_object*, lean_object*, lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_Cursor_writeUInt8(lean_object*, uint8_t);
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_matchElimFFI_decodeArms(lean_object*, lean_object*, lean_object*);
uint8_t lean_uint32_to_uint8(uint32_t);
lean_object* l_List_lengthTR___redArg(lean_object*);
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readUInt8(lean_object*);
lean_object* lp_iris_x2dkernel_IrisKernel_Kernel_trans__(lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_wrapResult___boxed(lean_object*);
LEAN_EXPORT lean_object* iris_kernel_guard_rule(lean_object*);
LEAN_EXPORT lean_object* lp_iris_x2dkernel_List_foldl___at___00IrisKernel_FFI_encodeContext_spec__0(lean_object*, lean_object*);
lean_object* lean_uint32_to_nat(uint32_t);
lean_object* lp_iris_x2dkernel_IrisKernel_Kernel_typeApp(lean_object*, lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_iris_x2dkernel_List_foldl___at___00IrisKernel_FFI_encodeCostBound_spec__0(lean_object*, lean_object*);
LEAN_EXPORT uint8_t iris_check_cost_leq(lean_object*);
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_decodeBinderId(lean_object*);
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readInt64LE(lean_object*);
uint64_t lean_uint64_of_nat(lean_object*);
uint64_t lean_uint64_lor(uint64_t, uint64_t);
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_decodeTypeDef_decodeTagTypeIdList(lean_object*, lean_object*, lean_object*);
lean_object* lp_iris_x2dkernel_IrisKernel_Kernel_natInd(lean_object*, lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_encodeCostBound(lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_encodeBinderId(lean_object*, lean_object*);
uint32_t lean_uint8_to_uint32(uint8_t);
static lean_object* lp_iris_x2dkernel_IrisKernel_FFI_encodeSuccess___closed__0;
uint32_t lean_uint32_shift_right(uint32_t, uint32_t);
LEAN_EXPORT lean_object* iris_kernel_intro(lean_object*);
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_decodeCostVar(lean_object*);
LEAN_EXPORT lean_object* iris_kernel_congr(lean_object*);
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_decodeContext(lean_object*);
static lean_object* lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__10;
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_encodeNodeId(lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_wrapResult(lean_object*);
uint8_t lp_iris_x2dkernel_IrisKernel_evalLIAFormula(lean_object*, lean_object*);
LEAN_EXPORT lean_object* iris_kernel_match_elim(lean_object*);
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_Cursor_remaining(lean_object*);
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readUInt16LE(lean_object*);
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_Cursor_writeUInt16LE___boxed(lean_object*, lean_object*);
LEAN_EXPORT lean_object* iris_kernel_type_app(lean_object*);
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_encodeBinderId___boxed(lean_object*, lean_object*);
static lean_object* lp_iris_x2dkernel_IrisKernel_FFI_assumeFFI___closed__0;
lean_object* lp_iris_x2dkernel_IrisKernel_Kernel_refineIntro(lean_object*, lean_object*, lean_object*, lean_object*);
lean_object* lean_byte_array_push(lean_object*, uint8_t);
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_encodeFailure___boxed(lean_object*);
LEAN_EXPORT lean_object* iris_kernel_type_check_node_full(lean_object*);
lean_object* lp_iris_x2dkernel_IrisKernel_Kernel_costLeqRule__(lean_object*, lean_object*);
lean_object* lp_iris_x2dkernel_IrisKernel_Kernel_typeAbst(lean_object*, lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_decodeContext_decodeBindings(lean_object*, lean_object*, lean_object*);
static lean_object* lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__4;
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_decodeTypeDef_decodeTypeIdList(lean_object*, lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_iris_x2dkernel_List_foldl___at___00IrisKernel_FFI_encodeContext_spec__0___boxed(lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_Cursor_writeUInt8___boxed(lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_decodeTypeEnv_decodeEntries(lean_object*, lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___boxed(lean_object*);
static lean_object* lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readInt64LE___closed__2;
uint64_t lean_uint64_shift_right(uint64_t, uint64_t);
lean_object* lean_nat_to_int(lean_object*);
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_decodeTag(lean_object*);
uint32_t lean_uint32_of_nat(lean_object*);
extern lean_object* l_ByteArray_empty;
LEAN_EXPORT lean_object* iris_kernel_structural_ind(lean_object*);
uint64_t lean_uint8_to_uint64(uint8_t);
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_encodeCostBound___boxed(lean_object*, lean_object*);
LEAN_EXPORT lean_object* iris_kernel_refl(lean_object*);
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_evalLIAFFI___boxed(lean_object*);
lean_object* l_Int_pow(lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_decodeJudgment(lean_object*);
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_encodeJudgment___boxed(lean_object*, lean_object*);
static lean_object* lp_iris_x2dkernel_IrisKernel_FFI_wrapResult___closed__0;
lean_object* lean_uint64_to_nat(uint64_t);
static lean_object* lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__14;
static lean_object* lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__18;
lean_object* lp_iris_x2dkernel_IrisKernel_Kernel_matchElim(lean_object*, lean_object*, lean_object*);
LEAN_EXPORT lean_object* iris_kernel_let_bind(lean_object*);
uint16_t lean_uint16_shift_right(uint16_t, uint16_t);
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_Cursor_writeUInt32LE(lean_object*, uint32_t);
LEAN_EXPORT lean_object* iris_kernel_nat_ind(lean_object*);
static lean_object* lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__0;
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_encodeSuccess(lean_object*);
uint16_t lean_uint8_to_uint16(uint8_t);
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_structuralIndFFI_decodeCases(lean_object*, lean_object*, lean_object*);
LEAN_EXPORT lean_object* iris_kernel_refine_intro(lean_object*);
static lean_object* lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__6;
static lean_object* lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__5;
uint8_t lean_uint64_to_uint8(uint64_t);
static lean_object* lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__2;
LEAN_EXPORT lean_object* iris_kernel_fold_rule(lean_object*);
uint16_t lean_uint16_of_nat(lean_object*);
lean_object* lp_iris_x2dkernel_IrisKernel_Kernel_structuralInd(lean_object*, lean_object*, lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_Cursor_writeUInt16LE(lean_object*, uint16_t);
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_Cursor_remaining___boxed(lean_object*);
static lean_object* lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__17;
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_Cursor_writeUInt32LE___boxed(lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_encodeContext(lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_typeCheckNodeFFI___boxed(lean_object*, lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_decodeLIAEnv_decodeLIAEnvEntries(lean_object*, lean_object*, lean_object*);
uint8_t lean_uint16_to_uint8(uint16_t);
lean_object* lp_iris_x2dkernel_IrisKernel_Kernel_elim(lean_object*, lean_object*, lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_decodeCostBound_decodeCostBoundList(lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_decodeTypeId(lean_object*);
static lean_object* lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__13;
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_encodeSuccess___boxed(lean_object*);
lean_object* lp_iris_x2dkernel_IrisKernel_Kernel_refineElim(lean_object*, lean_object*);
lean_object* lean_int_sub(lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_encodeTypeId(lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_decodeTypeEnv(lean_object*);
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readUInt64LE(lean_object*);
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_Cursor_create(lean_object*);
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_decodeBoundVar(lean_object*);
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_decodeLIATerm(lean_object*);
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_decodePrimType(lean_object*);
LEAN_EXPORT lean_object* iris_kernel_cost_leq_rule(lean_object*);
static lean_object* lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__15;
static lean_object* lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__19;
uint8_t lp_iris_x2dkernel_IrisKernel_checkCostLeq(lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_decodeCostBound(lean_object*);
lean_object* lp_iris_x2dkernel_IrisKernel_Kernel_symm__(lean_object*, lean_object*, lean_object*);
LEAN_EXPORT uint8_t iris_eval_lia(lean_object*);
uint16_t lean_uint16_lor(uint16_t, uint16_t);
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_decodeTypeDef(lean_object*);
uint8_t lean_nat_dec_eq(lean_object*, lean_object*);
uint8_t lean_nat_dec_lt(lean_object*, lean_object*);
LEAN_EXPORT lean_object* iris_kernel_elim(lean_object*);
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeId(lean_object*);
LEAN_EXPORT lean_object* iris_kernel_symm(lean_object*);
lean_object* lp_iris_x2dkernel_IrisKernel_Kernel_typeCheckNode__(lean_object*, lean_object*, lean_object*, uint8_t, lean_object*);
static lean_object* lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readInt64LE___closed__0;
lean_object* lp_iris_x2dkernel_IrisKernel_Kernel_assume__(lean_object*, lean_object*, lean_object*);
LEAN_EXPORT lean_object* iris_kernel_cost_subsume(lean_object*);
static lean_object* lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__16;
lean_object* lp_iris_x2dkernel_IrisKernel_Kernel_foldRule(lean_object*, lean_object*, lean_object*, lean_object*);
lean_object* lp_iris_x2dkernel_IrisKernel_Kernel_guardRule(lean_object*, lean_object*, lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_typeCheckNodeFFI___redArg___boxed(lean_object*);
LEAN_EXPORT lean_object* lp_iris_x2dkernel_List_foldl___at___00IrisKernel_FFI_encodeCostBound_spec__0___boxed(lean_object*, lean_object*);
lean_object* lean_uint16_to_nat(uint16_t);
uint32_t lean_uint32_lor(uint32_t, uint32_t);
uint32_t lean_uint32_shift_left(uint32_t, uint32_t);
lean_object* l_List_reverse___redArg(lean_object*);
lean_object* lean_nat_sub(lean_object*, lean_object*);
static lean_object* lp_iris_x2dkernel_IrisKernel_FFI_encodeFailure___closed__0;
static lean_object* lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__11;
LEAN_EXPORT lean_object* iris_kernel_type_abst(lean_object*);
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_encodeJudgment(lean_object*, lean_object*);
uint64_t lean_uint64_shift_left(uint64_t, uint64_t);
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_Cursor_writeUInt64LE(lean_object*, uint64_t);
static lean_object* lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__1;
static lean_object* lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__7;
lean_object* lp_iris_x2dkernel_IrisKernel_Kernel_costSubsume__(lean_object*, lean_object*);
static lean_object* lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__9;
static lean_object* lp_iris_x2dkernel_IrisKernel_FFI_typeCheckNodeFullFFI___closed__0;
uint8_t lean_byte_array_get(lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_encodeContext___boxed(lean_object*, lean_object*);
lean_object* lean_uint8_to_nat(uint8_t);
lean_object* lp_iris_x2dkernel_IrisKernel_Kernel_refl__(lean_object*, lean_object*, lean_object*);
LEAN_EXPORT lean_object* iris_kernel_assume(lean_object*);
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_Cursor_writeUInt64LE___boxed(lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_encodeNodeId___boxed(lean_object*, lean_object*);
lean_object* lp_iris_x2dkernel_IrisKernel_Kernel_congr__(lean_object*, lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_encodeTypeId___boxed(lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind(uint8_t);
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_encodeFailure(uint8_t);
uint8_t lean_nat_dec_le(lean_object*, lean_object*);
static lean_object* lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readInt64LE___closed__1;
LEAN_EXPORT uint32_t iris_lean_kernel_version;
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_decodeLIAEnv(lean_object*);
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_decodeCostBound_decodeCostBoundList___boxed(lean_object*, lean_object*);
lean_object* lean_nat_add(lean_object*, lean_object*);
LEAN_EXPORT lean_object* iris_kernel_refine_elim(lean_object*);
lean_object* lp_iris_x2dkernel_IrisKernel_Kernel_intro(lean_object*, lean_object*, lean_object*, lean_object*, lean_object*, lean_object*, lean_object*);
uint16_t lean_uint16_shift_left(uint16_t, uint16_t);
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readUInt32LE(lean_object*);
static lean_object* lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__3;
static lean_object* lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__12;
lean_object* lean_byte_array_size(lean_object*);
LEAN_EXPORT lean_object* iris_kernel_trans(lean_object*);
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_decodeLIAFormula(lean_object*);
LEAN_EXPORT uint8_t lp_iris_x2dkernel_IrisKernel_FFI_typeCheckNodeFFI___redArg(uint8_t);
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_Cursor_create(lean_object* x_1) {
_start:
{
lean_object* x_2; lean_object* x_3; 
x_2 = lean_unsigned_to_nat(0u);
x_3 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_3, 0, x_1);
lean_ctor_set(x_3, 1, x_2);
return x_3;
}
}
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_Cursor_remaining(lean_object* x_1) {
_start:
{
lean_object* x_2; lean_object* x_3; lean_object* x_4; lean_object* x_5; 
x_2 = lean_ctor_get(x_1, 0);
x_3 = lean_ctor_get(x_1, 1);
x_4 = lean_byte_array_size(x_2);
x_5 = lean_nat_sub(x_4, x_3);
return x_5;
}
}
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_Cursor_remaining___boxed(lean_object* x_1) {
_start:
{
lean_object* x_2; 
x_2 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_remaining(x_1);
lean_dec_ref(x_1);
return x_2;
}
}
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readUInt8(lean_object* x_1) {
_start:
{
uint8_t x_2; 
x_2 = !lean_is_exclusive(x_1);
if (x_2 == 0)
{
lean_object* x_3; lean_object* x_4; lean_object* x_5; uint8_t x_6; 
x_3 = lean_ctor_get(x_1, 0);
x_4 = lean_ctor_get(x_1, 1);
x_5 = lean_byte_array_size(x_3);
x_6 = lean_nat_dec_lt(x_4, x_5);
if (x_6 == 0)
{
lean_object* x_7; 
lean_free_object(x_1);
lean_dec(x_4);
lean_dec_ref(x_3);
x_7 = lean_box(0);
return x_7;
}
else
{
uint8_t x_8; lean_object* x_9; lean_object* x_10; lean_object* x_11; lean_object* x_12; lean_object* x_13; 
x_8 = lean_byte_array_get(x_3, x_4);
x_9 = lean_unsigned_to_nat(1u);
x_10 = lean_nat_add(x_4, x_9);
lean_dec(x_4);
lean_ctor_set(x_1, 1, x_10);
x_11 = lean_box(x_8);
x_12 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_12, 0, x_11);
lean_ctor_set(x_12, 1, x_1);
x_13 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_13, 0, x_12);
return x_13;
}
}
else
{
lean_object* x_14; lean_object* x_15; lean_object* x_16; uint8_t x_17; 
x_14 = lean_ctor_get(x_1, 0);
x_15 = lean_ctor_get(x_1, 1);
lean_inc(x_15);
lean_inc(x_14);
lean_dec(x_1);
x_16 = lean_byte_array_size(x_14);
x_17 = lean_nat_dec_lt(x_15, x_16);
if (x_17 == 0)
{
lean_object* x_18; 
lean_dec(x_15);
lean_dec_ref(x_14);
x_18 = lean_box(0);
return x_18;
}
else
{
uint8_t x_19; lean_object* x_20; lean_object* x_21; lean_object* x_22; lean_object* x_23; lean_object* x_24; lean_object* x_25; 
x_19 = lean_byte_array_get(x_14, x_15);
x_20 = lean_unsigned_to_nat(1u);
x_21 = lean_nat_add(x_15, x_20);
lean_dec(x_15);
x_22 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_22, 0, x_14);
lean_ctor_set(x_22, 1, x_21);
x_23 = lean_box(x_19);
x_24 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_24, 0, x_23);
lean_ctor_set(x_24, 1, x_22);
x_25 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_25, 0, x_24);
return x_25;
}
}
}
}
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readUInt16LE(lean_object* x_1) {
_start:
{
uint8_t x_2; 
x_2 = !lean_is_exclusive(x_1);
if (x_2 == 0)
{
lean_object* x_3; lean_object* x_4; lean_object* x_5; lean_object* x_6; lean_object* x_7; uint8_t x_8; 
x_3 = lean_ctor_get(x_1, 0);
x_4 = lean_ctor_get(x_1, 1);
x_5 = lean_unsigned_to_nat(1u);
x_6 = lean_nat_add(x_4, x_5);
x_7 = lean_byte_array_size(x_3);
x_8 = lean_nat_dec_lt(x_6, x_7);
if (x_8 == 0)
{
lean_object* x_9; 
lean_dec(x_6);
lean_free_object(x_1);
lean_dec(x_4);
lean_dec_ref(x_3);
x_9 = lean_box(0);
return x_9;
}
else
{
uint8_t x_10; uint8_t x_11; uint16_t x_12; uint16_t x_13; uint16_t x_14; uint16_t x_15; uint16_t x_16; lean_object* x_17; lean_object* x_18; lean_object* x_19; lean_object* x_20; lean_object* x_21; 
x_10 = lean_byte_array_get(x_3, x_4);
x_11 = lean_byte_array_get(x_3, x_6);
lean_dec(x_6);
x_12 = lean_uint8_to_uint16(x_10);
x_13 = lean_uint8_to_uint16(x_11);
x_14 = 8;
x_15 = lean_uint16_shift_left(x_13, x_14);
x_16 = lean_uint16_lor(x_12, x_15);
x_17 = lean_unsigned_to_nat(2u);
x_18 = lean_nat_add(x_4, x_17);
lean_dec(x_4);
lean_ctor_set(x_1, 1, x_18);
x_19 = lean_box(x_16);
x_20 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_20, 0, x_19);
lean_ctor_set(x_20, 1, x_1);
x_21 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_21, 0, x_20);
return x_21;
}
}
else
{
lean_object* x_22; lean_object* x_23; lean_object* x_24; lean_object* x_25; lean_object* x_26; uint8_t x_27; 
x_22 = lean_ctor_get(x_1, 0);
x_23 = lean_ctor_get(x_1, 1);
lean_inc(x_23);
lean_inc(x_22);
lean_dec(x_1);
x_24 = lean_unsigned_to_nat(1u);
x_25 = lean_nat_add(x_23, x_24);
x_26 = lean_byte_array_size(x_22);
x_27 = lean_nat_dec_lt(x_25, x_26);
if (x_27 == 0)
{
lean_object* x_28; 
lean_dec(x_25);
lean_dec(x_23);
lean_dec_ref(x_22);
x_28 = lean_box(0);
return x_28;
}
else
{
uint8_t x_29; uint8_t x_30; uint16_t x_31; uint16_t x_32; uint16_t x_33; uint16_t x_34; uint16_t x_35; lean_object* x_36; lean_object* x_37; lean_object* x_38; lean_object* x_39; lean_object* x_40; lean_object* x_41; 
x_29 = lean_byte_array_get(x_22, x_23);
x_30 = lean_byte_array_get(x_22, x_25);
lean_dec(x_25);
x_31 = lean_uint8_to_uint16(x_29);
x_32 = lean_uint8_to_uint16(x_30);
x_33 = 8;
x_34 = lean_uint16_shift_left(x_32, x_33);
x_35 = lean_uint16_lor(x_31, x_34);
x_36 = lean_unsigned_to_nat(2u);
x_37 = lean_nat_add(x_23, x_36);
lean_dec(x_23);
x_38 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_38, 0, x_22);
lean_ctor_set(x_38, 1, x_37);
x_39 = lean_box(x_35);
x_40 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_40, 0, x_39);
lean_ctor_set(x_40, 1, x_38);
x_41 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_41, 0, x_40);
return x_41;
}
}
}
}
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readUInt32LE(lean_object* x_1) {
_start:
{
uint8_t x_2; 
x_2 = !lean_is_exclusive(x_1);
if (x_2 == 0)
{
lean_object* x_3; lean_object* x_4; lean_object* x_5; lean_object* x_6; lean_object* x_7; uint8_t x_8; 
x_3 = lean_ctor_get(x_1, 0);
x_4 = lean_ctor_get(x_1, 1);
x_5 = lean_unsigned_to_nat(3u);
x_6 = lean_nat_add(x_4, x_5);
x_7 = lean_byte_array_size(x_3);
x_8 = lean_nat_dec_lt(x_6, x_7);
if (x_8 == 0)
{
lean_object* x_9; 
lean_dec(x_6);
lean_free_object(x_1);
lean_dec(x_4);
lean_dec_ref(x_3);
x_9 = lean_box(0);
return x_9;
}
else
{
uint8_t x_10; lean_object* x_11; lean_object* x_12; uint8_t x_13; lean_object* x_14; lean_object* x_15; uint8_t x_16; uint8_t x_17; uint32_t x_18; uint32_t x_19; uint32_t x_20; uint32_t x_21; uint32_t x_22; uint32_t x_23; uint32_t x_24; uint32_t x_25; uint32_t x_26; uint32_t x_27; uint32_t x_28; uint32_t x_29; uint32_t x_30; lean_object* x_31; lean_object* x_32; lean_object* x_33; lean_object* x_34; lean_object* x_35; 
x_10 = lean_byte_array_get(x_3, x_4);
x_11 = lean_unsigned_to_nat(1u);
x_12 = lean_nat_add(x_4, x_11);
x_13 = lean_byte_array_get(x_3, x_12);
lean_dec(x_12);
x_14 = lean_unsigned_to_nat(2u);
x_15 = lean_nat_add(x_4, x_14);
x_16 = lean_byte_array_get(x_3, x_15);
lean_dec(x_15);
x_17 = lean_byte_array_get(x_3, x_6);
lean_dec(x_6);
x_18 = lean_uint8_to_uint32(x_10);
x_19 = lean_uint8_to_uint32(x_13);
x_20 = 8;
x_21 = lean_uint32_shift_left(x_19, x_20);
x_22 = lean_uint32_lor(x_18, x_21);
x_23 = lean_uint8_to_uint32(x_16);
x_24 = 16;
x_25 = lean_uint32_shift_left(x_23, x_24);
x_26 = lean_uint32_lor(x_22, x_25);
x_27 = lean_uint8_to_uint32(x_17);
x_28 = 24;
x_29 = lean_uint32_shift_left(x_27, x_28);
x_30 = lean_uint32_lor(x_26, x_29);
x_31 = lean_unsigned_to_nat(4u);
x_32 = lean_nat_add(x_4, x_31);
lean_dec(x_4);
lean_ctor_set(x_1, 1, x_32);
x_33 = lean_box_uint32(x_30);
x_34 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_34, 0, x_33);
lean_ctor_set(x_34, 1, x_1);
x_35 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_35, 0, x_34);
return x_35;
}
}
else
{
lean_object* x_36; lean_object* x_37; lean_object* x_38; lean_object* x_39; lean_object* x_40; uint8_t x_41; 
x_36 = lean_ctor_get(x_1, 0);
x_37 = lean_ctor_get(x_1, 1);
lean_inc(x_37);
lean_inc(x_36);
lean_dec(x_1);
x_38 = lean_unsigned_to_nat(3u);
x_39 = lean_nat_add(x_37, x_38);
x_40 = lean_byte_array_size(x_36);
x_41 = lean_nat_dec_lt(x_39, x_40);
if (x_41 == 0)
{
lean_object* x_42; 
lean_dec(x_39);
lean_dec(x_37);
lean_dec_ref(x_36);
x_42 = lean_box(0);
return x_42;
}
else
{
uint8_t x_43; lean_object* x_44; lean_object* x_45; uint8_t x_46; lean_object* x_47; lean_object* x_48; uint8_t x_49; uint8_t x_50; uint32_t x_51; uint32_t x_52; uint32_t x_53; uint32_t x_54; uint32_t x_55; uint32_t x_56; uint32_t x_57; uint32_t x_58; uint32_t x_59; uint32_t x_60; uint32_t x_61; uint32_t x_62; uint32_t x_63; lean_object* x_64; lean_object* x_65; lean_object* x_66; lean_object* x_67; lean_object* x_68; lean_object* x_69; 
x_43 = lean_byte_array_get(x_36, x_37);
x_44 = lean_unsigned_to_nat(1u);
x_45 = lean_nat_add(x_37, x_44);
x_46 = lean_byte_array_get(x_36, x_45);
lean_dec(x_45);
x_47 = lean_unsigned_to_nat(2u);
x_48 = lean_nat_add(x_37, x_47);
x_49 = lean_byte_array_get(x_36, x_48);
lean_dec(x_48);
x_50 = lean_byte_array_get(x_36, x_39);
lean_dec(x_39);
x_51 = lean_uint8_to_uint32(x_43);
x_52 = lean_uint8_to_uint32(x_46);
x_53 = 8;
x_54 = lean_uint32_shift_left(x_52, x_53);
x_55 = lean_uint32_lor(x_51, x_54);
x_56 = lean_uint8_to_uint32(x_49);
x_57 = 16;
x_58 = lean_uint32_shift_left(x_56, x_57);
x_59 = lean_uint32_lor(x_55, x_58);
x_60 = lean_uint8_to_uint32(x_50);
x_61 = 24;
x_62 = lean_uint32_shift_left(x_60, x_61);
x_63 = lean_uint32_lor(x_59, x_62);
x_64 = lean_unsigned_to_nat(4u);
x_65 = lean_nat_add(x_37, x_64);
lean_dec(x_37);
x_66 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_66, 0, x_36);
lean_ctor_set(x_66, 1, x_65);
x_67 = lean_box_uint32(x_63);
x_68 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_68, 0, x_67);
lean_ctor_set(x_68, 1, x_66);
x_69 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_69, 0, x_68);
return x_69;
}
}
}
}
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readUInt64LE(lean_object* x_1) {
_start:
{
uint8_t x_2; 
x_2 = !lean_is_exclusive(x_1);
if (x_2 == 0)
{
lean_object* x_3; lean_object* x_4; lean_object* x_5; lean_object* x_6; lean_object* x_7; uint8_t x_8; 
x_3 = lean_ctor_get(x_1, 0);
x_4 = lean_ctor_get(x_1, 1);
x_5 = lean_unsigned_to_nat(7u);
x_6 = lean_nat_add(x_4, x_5);
x_7 = lean_byte_array_size(x_3);
x_8 = lean_nat_dec_lt(x_6, x_7);
if (x_8 == 0)
{
lean_object* x_9; 
lean_dec(x_6);
lean_free_object(x_1);
lean_dec(x_4);
lean_dec_ref(x_3);
x_9 = lean_box(0);
return x_9;
}
else
{
uint8_t x_10; uint64_t x_11; lean_object* x_12; lean_object* x_13; uint8_t x_14; uint64_t x_15; lean_object* x_16; lean_object* x_17; uint8_t x_18; uint64_t x_19; lean_object* x_20; lean_object* x_21; uint8_t x_22; uint64_t x_23; lean_object* x_24; lean_object* x_25; uint8_t x_26; uint64_t x_27; lean_object* x_28; lean_object* x_29; uint8_t x_30; uint64_t x_31; lean_object* x_32; lean_object* x_33; uint8_t x_34; uint64_t x_35; uint8_t x_36; uint64_t x_37; lean_object* x_38; uint64_t x_39; uint64_t x_40; uint64_t x_41; uint64_t x_42; uint64_t x_43; uint64_t x_44; uint64_t x_45; uint64_t x_46; uint64_t x_47; uint64_t x_48; uint64_t x_49; uint64_t x_50; uint64_t x_51; uint64_t x_52; uint64_t x_53; uint64_t x_54; uint64_t x_55; uint64_t x_56; uint64_t x_57; uint64_t x_58; uint64_t x_59; lean_object* x_60; lean_object* x_61; lean_object* x_62; lean_object* x_63; 
x_10 = lean_byte_array_get(x_3, x_4);
x_11 = lean_uint8_to_uint64(x_10);
x_12 = lean_unsigned_to_nat(1u);
x_13 = lean_nat_add(x_4, x_12);
x_14 = lean_byte_array_get(x_3, x_13);
lean_dec(x_13);
x_15 = lean_uint8_to_uint64(x_14);
x_16 = lean_unsigned_to_nat(2u);
x_17 = lean_nat_add(x_4, x_16);
x_18 = lean_byte_array_get(x_3, x_17);
lean_dec(x_17);
x_19 = lean_uint8_to_uint64(x_18);
x_20 = lean_unsigned_to_nat(3u);
x_21 = lean_nat_add(x_4, x_20);
x_22 = lean_byte_array_get(x_3, x_21);
lean_dec(x_21);
x_23 = lean_uint8_to_uint64(x_22);
x_24 = lean_unsigned_to_nat(4u);
x_25 = lean_nat_add(x_4, x_24);
x_26 = lean_byte_array_get(x_3, x_25);
lean_dec(x_25);
x_27 = lean_uint8_to_uint64(x_26);
x_28 = lean_unsigned_to_nat(5u);
x_29 = lean_nat_add(x_4, x_28);
x_30 = lean_byte_array_get(x_3, x_29);
lean_dec(x_29);
x_31 = lean_uint8_to_uint64(x_30);
x_32 = lean_unsigned_to_nat(6u);
x_33 = lean_nat_add(x_4, x_32);
x_34 = lean_byte_array_get(x_3, x_33);
lean_dec(x_33);
x_35 = lean_uint8_to_uint64(x_34);
x_36 = lean_byte_array_get(x_3, x_6);
lean_dec(x_6);
x_37 = lean_uint8_to_uint64(x_36);
x_38 = lean_unsigned_to_nat(8u);
x_39 = 8;
x_40 = lean_uint64_shift_left(x_15, x_39);
x_41 = lean_uint64_lor(x_11, x_40);
x_42 = 16;
x_43 = lean_uint64_shift_left(x_19, x_42);
x_44 = lean_uint64_lor(x_41, x_43);
x_45 = 24;
x_46 = lean_uint64_shift_left(x_23, x_45);
x_47 = lean_uint64_lor(x_44, x_46);
x_48 = 32;
x_49 = lean_uint64_shift_left(x_27, x_48);
x_50 = lean_uint64_lor(x_47, x_49);
x_51 = 40;
x_52 = lean_uint64_shift_left(x_31, x_51);
x_53 = lean_uint64_lor(x_50, x_52);
x_54 = 48;
x_55 = lean_uint64_shift_left(x_35, x_54);
x_56 = lean_uint64_lor(x_53, x_55);
x_57 = 56;
x_58 = lean_uint64_shift_left(x_37, x_57);
x_59 = lean_uint64_lor(x_56, x_58);
x_60 = lean_nat_add(x_4, x_38);
lean_dec(x_4);
lean_ctor_set(x_1, 1, x_60);
x_61 = lean_box_uint64(x_59);
x_62 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_62, 0, x_61);
lean_ctor_set(x_62, 1, x_1);
x_63 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_63, 0, x_62);
return x_63;
}
}
else
{
lean_object* x_64; lean_object* x_65; lean_object* x_66; lean_object* x_67; lean_object* x_68; uint8_t x_69; 
x_64 = lean_ctor_get(x_1, 0);
x_65 = lean_ctor_get(x_1, 1);
lean_inc(x_65);
lean_inc(x_64);
lean_dec(x_1);
x_66 = lean_unsigned_to_nat(7u);
x_67 = lean_nat_add(x_65, x_66);
x_68 = lean_byte_array_size(x_64);
x_69 = lean_nat_dec_lt(x_67, x_68);
if (x_69 == 0)
{
lean_object* x_70; 
lean_dec(x_67);
lean_dec(x_65);
lean_dec_ref(x_64);
x_70 = lean_box(0);
return x_70;
}
else
{
uint8_t x_71; uint64_t x_72; lean_object* x_73; lean_object* x_74; uint8_t x_75; uint64_t x_76; lean_object* x_77; lean_object* x_78; uint8_t x_79; uint64_t x_80; lean_object* x_81; lean_object* x_82; uint8_t x_83; uint64_t x_84; lean_object* x_85; lean_object* x_86; uint8_t x_87; uint64_t x_88; lean_object* x_89; lean_object* x_90; uint8_t x_91; uint64_t x_92; lean_object* x_93; lean_object* x_94; uint8_t x_95; uint64_t x_96; uint8_t x_97; uint64_t x_98; lean_object* x_99; uint64_t x_100; uint64_t x_101; uint64_t x_102; uint64_t x_103; uint64_t x_104; uint64_t x_105; uint64_t x_106; uint64_t x_107; uint64_t x_108; uint64_t x_109; uint64_t x_110; uint64_t x_111; uint64_t x_112; uint64_t x_113; uint64_t x_114; uint64_t x_115; uint64_t x_116; uint64_t x_117; uint64_t x_118; uint64_t x_119; uint64_t x_120; lean_object* x_121; lean_object* x_122; lean_object* x_123; lean_object* x_124; lean_object* x_125; 
x_71 = lean_byte_array_get(x_64, x_65);
x_72 = lean_uint8_to_uint64(x_71);
x_73 = lean_unsigned_to_nat(1u);
x_74 = lean_nat_add(x_65, x_73);
x_75 = lean_byte_array_get(x_64, x_74);
lean_dec(x_74);
x_76 = lean_uint8_to_uint64(x_75);
x_77 = lean_unsigned_to_nat(2u);
x_78 = lean_nat_add(x_65, x_77);
x_79 = lean_byte_array_get(x_64, x_78);
lean_dec(x_78);
x_80 = lean_uint8_to_uint64(x_79);
x_81 = lean_unsigned_to_nat(3u);
x_82 = lean_nat_add(x_65, x_81);
x_83 = lean_byte_array_get(x_64, x_82);
lean_dec(x_82);
x_84 = lean_uint8_to_uint64(x_83);
x_85 = lean_unsigned_to_nat(4u);
x_86 = lean_nat_add(x_65, x_85);
x_87 = lean_byte_array_get(x_64, x_86);
lean_dec(x_86);
x_88 = lean_uint8_to_uint64(x_87);
x_89 = lean_unsigned_to_nat(5u);
x_90 = lean_nat_add(x_65, x_89);
x_91 = lean_byte_array_get(x_64, x_90);
lean_dec(x_90);
x_92 = lean_uint8_to_uint64(x_91);
x_93 = lean_unsigned_to_nat(6u);
x_94 = lean_nat_add(x_65, x_93);
x_95 = lean_byte_array_get(x_64, x_94);
lean_dec(x_94);
x_96 = lean_uint8_to_uint64(x_95);
x_97 = lean_byte_array_get(x_64, x_67);
lean_dec(x_67);
x_98 = lean_uint8_to_uint64(x_97);
x_99 = lean_unsigned_to_nat(8u);
x_100 = 8;
x_101 = lean_uint64_shift_left(x_76, x_100);
x_102 = lean_uint64_lor(x_72, x_101);
x_103 = 16;
x_104 = lean_uint64_shift_left(x_80, x_103);
x_105 = lean_uint64_lor(x_102, x_104);
x_106 = 24;
x_107 = lean_uint64_shift_left(x_84, x_106);
x_108 = lean_uint64_lor(x_105, x_107);
x_109 = 32;
x_110 = lean_uint64_shift_left(x_88, x_109);
x_111 = lean_uint64_lor(x_108, x_110);
x_112 = 40;
x_113 = lean_uint64_shift_left(x_92, x_112);
x_114 = lean_uint64_lor(x_111, x_113);
x_115 = 48;
x_116 = lean_uint64_shift_left(x_96, x_115);
x_117 = lean_uint64_lor(x_114, x_116);
x_118 = 56;
x_119 = lean_uint64_shift_left(x_98, x_118);
x_120 = lean_uint64_lor(x_117, x_119);
x_121 = lean_nat_add(x_65, x_99);
lean_dec(x_65);
x_122 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_122, 0, x_64);
lean_ctor_set(x_122, 1, x_121);
x_123 = lean_box_uint64(x_120);
x_124 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_124, 0, x_123);
lean_ctor_set(x_124, 1, x_122);
x_125 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_125, 0, x_124);
return x_125;
}
}
}
}
static lean_object* _init_lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readInt64LE___closed__0() {
_start:
{
lean_object* x_1; 
x_1 = lean_cstr_to_nat("9223372036854775808");
return x_1;
}
}
static lean_object* _init_lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readInt64LE___closed__1() {
_start:
{
lean_object* x_1; lean_object* x_2; 
x_1 = lean_unsigned_to_nat(2u);
x_2 = lean_nat_to_int(x_1);
return x_2;
}
}
static lean_object* _init_lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readInt64LE___closed__2() {
_start:
{
lean_object* x_1; lean_object* x_2; lean_object* x_3; 
x_1 = lean_unsigned_to_nat(64u);
x_2 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readInt64LE___closed__1;
x_3 = l_Int_pow(x_2, x_1);
return x_3;
}
}
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readInt64LE(lean_object* x_1) {
_start:
{
lean_object* x_2; 
x_2 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readUInt64LE(x_1);
if (lean_obj_tag(x_2) == 0)
{
lean_object* x_3; 
x_3 = lean_box(0);
return x_3;
}
else
{
lean_object* x_4; lean_object* x_5; lean_object* x_6; lean_object* x_7; lean_object* x_8; lean_object* x_9; uint64_t x_13; lean_object* x_14; lean_object* x_15; uint8_t x_16; 
x_4 = lean_ctor_get(x_2, 0);
lean_inc(x_4);
if (lean_is_exclusive(x_2)) {
 lean_ctor_release(x_2, 0);
 x_5 = x_2;
} else {
 lean_dec_ref(x_2);
 x_5 = lean_box(0);
}
x_6 = lean_ctor_get(x_4, 0);
lean_inc(x_6);
x_7 = lean_ctor_get(x_4, 1);
lean_inc(x_7);
if (lean_is_exclusive(x_4)) {
 lean_ctor_release(x_4, 0);
 lean_ctor_release(x_4, 1);
 x_8 = x_4;
} else {
 lean_dec_ref(x_4);
 x_8 = lean_box(0);
}
x_13 = lean_unbox_uint64(x_6);
lean_dec(x_6);
x_14 = lean_uint64_to_nat(x_13);
x_15 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readInt64LE___closed__0;
x_16 = lean_nat_dec_le(x_15, x_14);
if (x_16 == 0)
{
lean_object* x_17; 
x_17 = lean_nat_to_int(x_14);
x_9 = x_17;
goto block_12;
}
else
{
lean_object* x_18; lean_object* x_19; lean_object* x_20; 
x_18 = lean_nat_to_int(x_14);
x_19 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readInt64LE___closed__2;
x_20 = lean_int_sub(x_18, x_19);
lean_dec(x_18);
x_9 = x_20;
goto block_12;
}
block_12:
{
lean_object* x_10; lean_object* x_11; 
if (lean_is_scalar(x_8)) {
 x_10 = lean_alloc_ctor(0, 2, 0);
} else {
 x_10 = x_8;
}
lean_ctor_set(x_10, 0, x_9);
lean_ctor_set(x_10, 1, x_7);
if (lean_is_scalar(x_5)) {
 x_11 = lean_alloc_ctor(1, 1, 0);
} else {
 x_11 = x_5;
}
lean_ctor_set(x_11, 0, x_10);
return x_11;
}
}
}
}
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_decodeCostBound(lean_object* x_1) {
_start:
{
lean_object* x_2; 
x_2 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readUInt8(x_1);
if (lean_obj_tag(x_2) == 0)
{
lean_object* x_3; 
x_3 = lean_box(0);
return x_3;
}
else
{
uint8_t x_4; 
x_4 = !lean_is_exclusive(x_2);
if (x_4 == 0)
{
lean_object* x_5; uint8_t x_6; 
x_5 = lean_ctor_get(x_2, 0);
x_6 = !lean_is_exclusive(x_5);
if (x_6 == 0)
{
lean_object* x_7; lean_object* x_8; uint8_t x_9; lean_object* x_10; lean_object* x_11; uint8_t x_12; 
x_7 = lean_ctor_get(x_5, 0);
x_8 = lean_ctor_get(x_5, 1);
x_9 = lean_unbox(x_7);
lean_dec(x_7);
x_10 = lean_uint8_to_nat(x_9);
x_11 = lean_unsigned_to_nat(0u);
x_12 = lean_nat_dec_eq(x_10, x_11);
if (x_12 == 0)
{
lean_object* x_13; uint8_t x_14; 
x_13 = lean_unsigned_to_nat(1u);
x_14 = lean_nat_dec_eq(x_10, x_13);
if (x_14 == 0)
{
lean_object* x_15; uint8_t x_16; 
lean_free_object(x_5);
lean_free_object(x_2);
x_15 = lean_unsigned_to_nat(2u);
x_16 = lean_nat_dec_eq(x_10, x_15);
if (x_16 == 0)
{
lean_object* x_17; uint8_t x_18; 
x_17 = lean_unsigned_to_nat(3u);
x_18 = lean_nat_dec_eq(x_10, x_17);
if (x_18 == 0)
{
lean_object* x_19; uint8_t x_20; 
x_19 = lean_unsigned_to_nat(4u);
x_20 = lean_nat_dec_eq(x_10, x_19);
if (x_20 == 0)
{
lean_object* x_21; uint8_t x_22; 
x_21 = lean_unsigned_to_nat(5u);
x_22 = lean_nat_dec_eq(x_10, x_21);
if (x_22 == 0)
{
lean_object* x_23; uint8_t x_24; 
x_23 = lean_unsigned_to_nat(6u);
x_24 = lean_nat_dec_eq(x_10, x_23);
if (x_24 == 0)
{
lean_object* x_25; uint8_t x_26; 
x_25 = lean_unsigned_to_nat(7u);
x_26 = lean_nat_dec_eq(x_10, x_25);
if (x_26 == 0)
{
lean_object* x_27; uint8_t x_28; 
x_27 = lean_unsigned_to_nat(8u);
x_28 = lean_nat_dec_eq(x_10, x_27);
if (x_28 == 0)
{
lean_object* x_29; uint8_t x_30; 
x_29 = lean_unsigned_to_nat(9u);
x_30 = lean_nat_dec_eq(x_10, x_29);
if (x_30 == 0)
{
lean_object* x_31; uint8_t x_32; 
x_31 = lean_unsigned_to_nat(10u);
x_32 = lean_nat_dec_eq(x_10, x_31);
if (x_32 == 0)
{
lean_object* x_33; uint8_t x_34; 
x_33 = lean_unsigned_to_nat(11u);
x_34 = lean_nat_dec_eq(x_10, x_33);
if (x_34 == 0)
{
lean_object* x_35; uint8_t x_36; 
x_35 = lean_unsigned_to_nat(12u);
x_36 = lean_nat_dec_eq(x_10, x_35);
if (x_36 == 0)
{
lean_object* x_37; 
lean_dec(x_8);
x_37 = lean_box(0);
return x_37;
}
else
{
lean_object* x_38; 
x_38 = lp_iris_x2dkernel_IrisKernel_FFI_decodeCostBound(x_8);
if (lean_obj_tag(x_38) == 0)
{
return x_38;
}
else
{
uint8_t x_39; 
x_39 = !lean_is_exclusive(x_38);
if (x_39 == 0)
{
lean_object* x_40; uint8_t x_41; 
x_40 = lean_ctor_get(x_38, 0);
x_41 = !lean_is_exclusive(x_40);
if (x_41 == 0)
{
lean_object* x_42; lean_object* x_43; 
x_42 = lean_ctor_get(x_40, 0);
x_43 = lean_alloc_ctor(12, 1, 0);
lean_ctor_set(x_43, 0, x_42);
lean_ctor_set(x_40, 0, x_43);
return x_38;
}
else
{
lean_object* x_44; lean_object* x_45; lean_object* x_46; lean_object* x_47; 
x_44 = lean_ctor_get(x_40, 0);
x_45 = lean_ctor_get(x_40, 1);
lean_inc(x_45);
lean_inc(x_44);
lean_dec(x_40);
x_46 = lean_alloc_ctor(12, 1, 0);
lean_ctor_set(x_46, 0, x_44);
x_47 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_47, 0, x_46);
lean_ctor_set(x_47, 1, x_45);
lean_ctor_set(x_38, 0, x_47);
return x_38;
}
}
else
{
lean_object* x_48; lean_object* x_49; lean_object* x_50; lean_object* x_51; lean_object* x_52; lean_object* x_53; lean_object* x_54; 
x_48 = lean_ctor_get(x_38, 0);
lean_inc(x_48);
lean_dec(x_38);
x_49 = lean_ctor_get(x_48, 0);
lean_inc(x_49);
x_50 = lean_ctor_get(x_48, 1);
lean_inc(x_50);
if (lean_is_exclusive(x_48)) {
 lean_ctor_release(x_48, 0);
 lean_ctor_release(x_48, 1);
 x_51 = x_48;
} else {
 lean_dec_ref(x_48);
 x_51 = lean_box(0);
}
x_52 = lean_alloc_ctor(12, 1, 0);
lean_ctor_set(x_52, 0, x_49);
if (lean_is_scalar(x_51)) {
 x_53 = lean_alloc_ctor(0, 2, 0);
} else {
 x_53 = x_51;
}
lean_ctor_set(x_53, 0, x_52);
lean_ctor_set(x_53, 1, x_50);
x_54 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_54, 0, x_53);
return x_54;
}
}
}
}
else
{
lean_object* x_55; 
x_55 = lp_iris_x2dkernel_IrisKernel_FFI_decodeCostBound(x_8);
if (lean_obj_tag(x_55) == 0)
{
return x_55;
}
else
{
uint8_t x_56; 
x_56 = !lean_is_exclusive(x_55);
if (x_56 == 0)
{
lean_object* x_57; uint8_t x_58; 
x_57 = lean_ctor_get(x_55, 0);
x_58 = !lean_is_exclusive(x_57);
if (x_58 == 0)
{
lean_object* x_59; lean_object* x_60; 
x_59 = lean_ctor_get(x_57, 0);
x_60 = lean_alloc_ctor(11, 1, 0);
lean_ctor_set(x_60, 0, x_59);
lean_ctor_set(x_57, 0, x_60);
return x_55;
}
else
{
lean_object* x_61; lean_object* x_62; lean_object* x_63; lean_object* x_64; 
x_61 = lean_ctor_get(x_57, 0);
x_62 = lean_ctor_get(x_57, 1);
lean_inc(x_62);
lean_inc(x_61);
lean_dec(x_57);
x_63 = lean_alloc_ctor(11, 1, 0);
lean_ctor_set(x_63, 0, x_61);
x_64 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_64, 0, x_63);
lean_ctor_set(x_64, 1, x_62);
lean_ctor_set(x_55, 0, x_64);
return x_55;
}
}
else
{
lean_object* x_65; lean_object* x_66; lean_object* x_67; lean_object* x_68; lean_object* x_69; lean_object* x_70; lean_object* x_71; 
x_65 = lean_ctor_get(x_55, 0);
lean_inc(x_65);
lean_dec(x_55);
x_66 = lean_ctor_get(x_65, 0);
lean_inc(x_66);
x_67 = lean_ctor_get(x_65, 1);
lean_inc(x_67);
if (lean_is_exclusive(x_65)) {
 lean_ctor_release(x_65, 0);
 lean_ctor_release(x_65, 1);
 x_68 = x_65;
} else {
 lean_dec_ref(x_65);
 x_68 = lean_box(0);
}
x_69 = lean_alloc_ctor(11, 1, 0);
lean_ctor_set(x_69, 0, x_66);
if (lean_is_scalar(x_68)) {
 x_70 = lean_alloc_ctor(0, 2, 0);
} else {
 x_70 = x_68;
}
lean_ctor_set(x_70, 0, x_69);
lean_ctor_set(x_70, 1, x_67);
x_71 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_71, 0, x_70);
return x_71;
}
}
}
}
else
{
lean_object* x_72; 
x_72 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readUInt16LE(x_8);
if (lean_obj_tag(x_72) == 0)
{
lean_object* x_73; 
x_73 = lean_box(0);
return x_73;
}
else
{
uint8_t x_74; 
x_74 = !lean_is_exclusive(x_72);
if (x_74 == 0)
{
lean_object* x_75; lean_object* x_76; lean_object* x_77; uint16_t x_78; lean_object* x_79; lean_object* x_80; 
x_75 = lean_ctor_get(x_72, 0);
x_76 = lean_ctor_get(x_75, 0);
lean_inc(x_76);
x_77 = lean_ctor_get(x_75, 1);
lean_inc(x_77);
lean_dec(x_75);
x_78 = lean_unbox(x_76);
lean_dec(x_76);
x_79 = lean_uint16_to_nat(x_78);
x_80 = lp_iris_x2dkernel_IrisKernel_FFI_decodeCostBound_decodeCostBoundList(x_77, x_79);
if (lean_obj_tag(x_80) == 0)
{
lean_object* x_81; 
lean_free_object(x_72);
x_81 = lean_box(0);
return x_81;
}
else
{
uint8_t x_82; 
x_82 = !lean_is_exclusive(x_80);
if (x_82 == 0)
{
lean_object* x_83; uint8_t x_84; 
x_83 = lean_ctor_get(x_80, 0);
x_84 = !lean_is_exclusive(x_83);
if (x_84 == 0)
{
lean_object* x_85; 
x_85 = lean_ctor_get(x_83, 0);
lean_ctor_set_tag(x_72, 10);
lean_ctor_set(x_72, 0, x_85);
lean_ctor_set(x_83, 0, x_72);
return x_80;
}
else
{
lean_object* x_86; lean_object* x_87; lean_object* x_88; 
x_86 = lean_ctor_get(x_83, 0);
x_87 = lean_ctor_get(x_83, 1);
lean_inc(x_87);
lean_inc(x_86);
lean_dec(x_83);
lean_ctor_set_tag(x_72, 10);
lean_ctor_set(x_72, 0, x_86);
x_88 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_88, 0, x_72);
lean_ctor_set(x_88, 1, x_87);
lean_ctor_set(x_80, 0, x_88);
return x_80;
}
}
else
{
lean_object* x_89; lean_object* x_90; lean_object* x_91; lean_object* x_92; lean_object* x_93; lean_object* x_94; 
x_89 = lean_ctor_get(x_80, 0);
lean_inc(x_89);
lean_dec(x_80);
x_90 = lean_ctor_get(x_89, 0);
lean_inc(x_90);
x_91 = lean_ctor_get(x_89, 1);
lean_inc(x_91);
if (lean_is_exclusive(x_89)) {
 lean_ctor_release(x_89, 0);
 lean_ctor_release(x_89, 1);
 x_92 = x_89;
} else {
 lean_dec_ref(x_89);
 x_92 = lean_box(0);
}
lean_ctor_set_tag(x_72, 10);
lean_ctor_set(x_72, 0, x_90);
if (lean_is_scalar(x_92)) {
 x_93 = lean_alloc_ctor(0, 2, 0);
} else {
 x_93 = x_92;
}
lean_ctor_set(x_93, 0, x_72);
lean_ctor_set(x_93, 1, x_91);
x_94 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_94, 0, x_93);
return x_94;
}
}
}
else
{
lean_object* x_95; lean_object* x_96; lean_object* x_97; uint16_t x_98; lean_object* x_99; lean_object* x_100; 
x_95 = lean_ctor_get(x_72, 0);
lean_inc(x_95);
lean_dec(x_72);
x_96 = lean_ctor_get(x_95, 0);
lean_inc(x_96);
x_97 = lean_ctor_get(x_95, 1);
lean_inc(x_97);
lean_dec(x_95);
x_98 = lean_unbox(x_96);
lean_dec(x_96);
x_99 = lean_uint16_to_nat(x_98);
x_100 = lp_iris_x2dkernel_IrisKernel_FFI_decodeCostBound_decodeCostBoundList(x_97, x_99);
if (lean_obj_tag(x_100) == 0)
{
lean_object* x_101; 
x_101 = lean_box(0);
return x_101;
}
else
{
lean_object* x_102; lean_object* x_103; lean_object* x_104; lean_object* x_105; lean_object* x_106; lean_object* x_107; lean_object* x_108; lean_object* x_109; 
x_102 = lean_ctor_get(x_100, 0);
lean_inc(x_102);
if (lean_is_exclusive(x_100)) {
 lean_ctor_release(x_100, 0);
 x_103 = x_100;
} else {
 lean_dec_ref(x_100);
 x_103 = lean_box(0);
}
x_104 = lean_ctor_get(x_102, 0);
lean_inc(x_104);
x_105 = lean_ctor_get(x_102, 1);
lean_inc(x_105);
if (lean_is_exclusive(x_102)) {
 lean_ctor_release(x_102, 0);
 lean_ctor_release(x_102, 1);
 x_106 = x_102;
} else {
 lean_dec_ref(x_102);
 x_106 = lean_box(0);
}
x_107 = lean_alloc_ctor(10, 1, 0);
lean_ctor_set(x_107, 0, x_104);
if (lean_is_scalar(x_106)) {
 x_108 = lean_alloc_ctor(0, 2, 0);
} else {
 x_108 = x_106;
}
lean_ctor_set(x_108, 0, x_107);
lean_ctor_set(x_108, 1, x_105);
if (lean_is_scalar(x_103)) {
 x_109 = lean_alloc_ctor(1, 1, 0);
} else {
 x_109 = x_103;
}
lean_ctor_set(x_109, 0, x_108);
return x_109;
}
}
}
}
}
else
{
lean_object* x_110; 
x_110 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readUInt16LE(x_8);
if (lean_obj_tag(x_110) == 0)
{
lean_object* x_111; 
x_111 = lean_box(0);
return x_111;
}
else
{
uint8_t x_112; 
x_112 = !lean_is_exclusive(x_110);
if (x_112 == 0)
{
lean_object* x_113; lean_object* x_114; lean_object* x_115; uint16_t x_116; lean_object* x_117; lean_object* x_118; 
x_113 = lean_ctor_get(x_110, 0);
x_114 = lean_ctor_get(x_113, 0);
lean_inc(x_114);
x_115 = lean_ctor_get(x_113, 1);
lean_inc(x_115);
lean_dec(x_113);
x_116 = lean_unbox(x_114);
lean_dec(x_114);
x_117 = lean_uint16_to_nat(x_116);
x_118 = lp_iris_x2dkernel_IrisKernel_FFI_decodeCostBound_decodeCostBoundList(x_115, x_117);
if (lean_obj_tag(x_118) == 0)
{
lean_object* x_119; 
lean_free_object(x_110);
x_119 = lean_box(0);
return x_119;
}
else
{
uint8_t x_120; 
x_120 = !lean_is_exclusive(x_118);
if (x_120 == 0)
{
lean_object* x_121; uint8_t x_122; 
x_121 = lean_ctor_get(x_118, 0);
x_122 = !lean_is_exclusive(x_121);
if (x_122 == 0)
{
lean_object* x_123; 
x_123 = lean_ctor_get(x_121, 0);
lean_ctor_set_tag(x_110, 9);
lean_ctor_set(x_110, 0, x_123);
lean_ctor_set(x_121, 0, x_110);
return x_118;
}
else
{
lean_object* x_124; lean_object* x_125; lean_object* x_126; 
x_124 = lean_ctor_get(x_121, 0);
x_125 = lean_ctor_get(x_121, 1);
lean_inc(x_125);
lean_inc(x_124);
lean_dec(x_121);
lean_ctor_set_tag(x_110, 9);
lean_ctor_set(x_110, 0, x_124);
x_126 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_126, 0, x_110);
lean_ctor_set(x_126, 1, x_125);
lean_ctor_set(x_118, 0, x_126);
return x_118;
}
}
else
{
lean_object* x_127; lean_object* x_128; lean_object* x_129; lean_object* x_130; lean_object* x_131; lean_object* x_132; 
x_127 = lean_ctor_get(x_118, 0);
lean_inc(x_127);
lean_dec(x_118);
x_128 = lean_ctor_get(x_127, 0);
lean_inc(x_128);
x_129 = lean_ctor_get(x_127, 1);
lean_inc(x_129);
if (lean_is_exclusive(x_127)) {
 lean_ctor_release(x_127, 0);
 lean_ctor_release(x_127, 1);
 x_130 = x_127;
} else {
 lean_dec_ref(x_127);
 x_130 = lean_box(0);
}
lean_ctor_set_tag(x_110, 9);
lean_ctor_set(x_110, 0, x_128);
if (lean_is_scalar(x_130)) {
 x_131 = lean_alloc_ctor(0, 2, 0);
} else {
 x_131 = x_130;
}
lean_ctor_set(x_131, 0, x_110);
lean_ctor_set(x_131, 1, x_129);
x_132 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_132, 0, x_131);
return x_132;
}
}
}
else
{
lean_object* x_133; lean_object* x_134; lean_object* x_135; uint16_t x_136; lean_object* x_137; lean_object* x_138; 
x_133 = lean_ctor_get(x_110, 0);
lean_inc(x_133);
lean_dec(x_110);
x_134 = lean_ctor_get(x_133, 0);
lean_inc(x_134);
x_135 = lean_ctor_get(x_133, 1);
lean_inc(x_135);
lean_dec(x_133);
x_136 = lean_unbox(x_134);
lean_dec(x_134);
x_137 = lean_uint16_to_nat(x_136);
x_138 = lp_iris_x2dkernel_IrisKernel_FFI_decodeCostBound_decodeCostBoundList(x_135, x_137);
if (lean_obj_tag(x_138) == 0)
{
lean_object* x_139; 
x_139 = lean_box(0);
return x_139;
}
else
{
lean_object* x_140; lean_object* x_141; lean_object* x_142; lean_object* x_143; lean_object* x_144; lean_object* x_145; lean_object* x_146; lean_object* x_147; 
x_140 = lean_ctor_get(x_138, 0);
lean_inc(x_140);
if (lean_is_exclusive(x_138)) {
 lean_ctor_release(x_138, 0);
 x_141 = x_138;
} else {
 lean_dec_ref(x_138);
 x_141 = lean_box(0);
}
x_142 = lean_ctor_get(x_140, 0);
lean_inc(x_142);
x_143 = lean_ctor_get(x_140, 1);
lean_inc(x_143);
if (lean_is_exclusive(x_140)) {
 lean_ctor_release(x_140, 0);
 lean_ctor_release(x_140, 1);
 x_144 = x_140;
} else {
 lean_dec_ref(x_140);
 x_144 = lean_box(0);
}
x_145 = lean_alloc_ctor(9, 1, 0);
lean_ctor_set(x_145, 0, x_142);
if (lean_is_scalar(x_144)) {
 x_146 = lean_alloc_ctor(0, 2, 0);
} else {
 x_146 = x_144;
}
lean_ctor_set(x_146, 0, x_145);
lean_ctor_set(x_146, 1, x_143);
if (lean_is_scalar(x_141)) {
 x_147 = lean_alloc_ctor(1, 1, 0);
} else {
 x_147 = x_141;
}
lean_ctor_set(x_147, 0, x_146);
return x_147;
}
}
}
}
}
else
{
lean_object* x_148; 
x_148 = lp_iris_x2dkernel_IrisKernel_FFI_decodeCostBound(x_8);
if (lean_obj_tag(x_148) == 0)
{
return x_148;
}
else
{
lean_object* x_149; uint8_t x_150; 
x_149 = lean_ctor_get(x_148, 0);
lean_inc(x_149);
lean_dec_ref(x_148);
x_150 = !lean_is_exclusive(x_149);
if (x_150 == 0)
{
lean_object* x_151; lean_object* x_152; lean_object* x_153; 
x_151 = lean_ctor_get(x_149, 0);
x_152 = lean_ctor_get(x_149, 1);
x_153 = lp_iris_x2dkernel_IrisKernel_FFI_decodeCostBound(x_152);
if (lean_obj_tag(x_153) == 0)
{
lean_free_object(x_149);
lean_dec(x_151);
return x_153;
}
else
{
uint8_t x_154; 
x_154 = !lean_is_exclusive(x_153);
if (x_154 == 0)
{
lean_object* x_155; uint8_t x_156; 
x_155 = lean_ctor_get(x_153, 0);
x_156 = !lean_is_exclusive(x_155);
if (x_156 == 0)
{
lean_object* x_157; 
x_157 = lean_ctor_get(x_155, 0);
lean_ctor_set_tag(x_149, 8);
lean_ctor_set(x_149, 1, x_157);
lean_ctor_set(x_155, 0, x_149);
return x_153;
}
else
{
lean_object* x_158; lean_object* x_159; lean_object* x_160; 
x_158 = lean_ctor_get(x_155, 0);
x_159 = lean_ctor_get(x_155, 1);
lean_inc(x_159);
lean_inc(x_158);
lean_dec(x_155);
lean_ctor_set_tag(x_149, 8);
lean_ctor_set(x_149, 1, x_158);
x_160 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_160, 0, x_149);
lean_ctor_set(x_160, 1, x_159);
lean_ctor_set(x_153, 0, x_160);
return x_153;
}
}
else
{
lean_object* x_161; lean_object* x_162; lean_object* x_163; lean_object* x_164; lean_object* x_165; lean_object* x_166; 
x_161 = lean_ctor_get(x_153, 0);
lean_inc(x_161);
lean_dec(x_153);
x_162 = lean_ctor_get(x_161, 0);
lean_inc(x_162);
x_163 = lean_ctor_get(x_161, 1);
lean_inc(x_163);
if (lean_is_exclusive(x_161)) {
 lean_ctor_release(x_161, 0);
 lean_ctor_release(x_161, 1);
 x_164 = x_161;
} else {
 lean_dec_ref(x_161);
 x_164 = lean_box(0);
}
lean_ctor_set_tag(x_149, 8);
lean_ctor_set(x_149, 1, x_162);
if (lean_is_scalar(x_164)) {
 x_165 = lean_alloc_ctor(0, 2, 0);
} else {
 x_165 = x_164;
}
lean_ctor_set(x_165, 0, x_149);
lean_ctor_set(x_165, 1, x_163);
x_166 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_166, 0, x_165);
return x_166;
}
}
}
else
{
lean_object* x_167; lean_object* x_168; lean_object* x_169; 
x_167 = lean_ctor_get(x_149, 0);
x_168 = lean_ctor_get(x_149, 1);
lean_inc(x_168);
lean_inc(x_167);
lean_dec(x_149);
x_169 = lp_iris_x2dkernel_IrisKernel_FFI_decodeCostBound(x_168);
if (lean_obj_tag(x_169) == 0)
{
lean_dec(x_167);
return x_169;
}
else
{
lean_object* x_170; lean_object* x_171; lean_object* x_172; lean_object* x_173; lean_object* x_174; lean_object* x_175; lean_object* x_176; lean_object* x_177; 
x_170 = lean_ctor_get(x_169, 0);
lean_inc(x_170);
if (lean_is_exclusive(x_169)) {
 lean_ctor_release(x_169, 0);
 x_171 = x_169;
} else {
 lean_dec_ref(x_169);
 x_171 = lean_box(0);
}
x_172 = lean_ctor_get(x_170, 0);
lean_inc(x_172);
x_173 = lean_ctor_get(x_170, 1);
lean_inc(x_173);
if (lean_is_exclusive(x_170)) {
 lean_ctor_release(x_170, 0);
 lean_ctor_release(x_170, 1);
 x_174 = x_170;
} else {
 lean_dec_ref(x_170);
 x_174 = lean_box(0);
}
x_175 = lean_alloc_ctor(8, 2, 0);
lean_ctor_set(x_175, 0, x_167);
lean_ctor_set(x_175, 1, x_172);
if (lean_is_scalar(x_174)) {
 x_176 = lean_alloc_ctor(0, 2, 0);
} else {
 x_176 = x_174;
}
lean_ctor_set(x_176, 0, x_175);
lean_ctor_set(x_176, 1, x_173);
if (lean_is_scalar(x_171)) {
 x_177 = lean_alloc_ctor(1, 1, 0);
} else {
 x_177 = x_171;
}
lean_ctor_set(x_177, 0, x_176);
return x_177;
}
}
}
}
}
else
{
lean_object* x_178; 
x_178 = lp_iris_x2dkernel_IrisKernel_FFI_decodeCostBound(x_8);
if (lean_obj_tag(x_178) == 0)
{
return x_178;
}
else
{
lean_object* x_179; uint8_t x_180; 
x_179 = lean_ctor_get(x_178, 0);
lean_inc(x_179);
lean_dec_ref(x_178);
x_180 = !lean_is_exclusive(x_179);
if (x_180 == 0)
{
lean_object* x_181; lean_object* x_182; lean_object* x_183; 
x_181 = lean_ctor_get(x_179, 0);
x_182 = lean_ctor_get(x_179, 1);
x_183 = lp_iris_x2dkernel_IrisKernel_FFI_decodeCostBound(x_182);
if (lean_obj_tag(x_183) == 0)
{
lean_free_object(x_179);
lean_dec(x_181);
return x_183;
}
else
{
uint8_t x_184; 
x_184 = !lean_is_exclusive(x_183);
if (x_184 == 0)
{
lean_object* x_185; uint8_t x_186; 
x_185 = lean_ctor_get(x_183, 0);
x_186 = !lean_is_exclusive(x_185);
if (x_186 == 0)
{
lean_object* x_187; 
x_187 = lean_ctor_get(x_185, 0);
lean_ctor_set_tag(x_179, 7);
lean_ctor_set(x_179, 1, x_187);
lean_ctor_set(x_185, 0, x_179);
return x_183;
}
else
{
lean_object* x_188; lean_object* x_189; lean_object* x_190; 
x_188 = lean_ctor_get(x_185, 0);
x_189 = lean_ctor_get(x_185, 1);
lean_inc(x_189);
lean_inc(x_188);
lean_dec(x_185);
lean_ctor_set_tag(x_179, 7);
lean_ctor_set(x_179, 1, x_188);
x_190 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_190, 0, x_179);
lean_ctor_set(x_190, 1, x_189);
lean_ctor_set(x_183, 0, x_190);
return x_183;
}
}
else
{
lean_object* x_191; lean_object* x_192; lean_object* x_193; lean_object* x_194; lean_object* x_195; lean_object* x_196; 
x_191 = lean_ctor_get(x_183, 0);
lean_inc(x_191);
lean_dec(x_183);
x_192 = lean_ctor_get(x_191, 0);
lean_inc(x_192);
x_193 = lean_ctor_get(x_191, 1);
lean_inc(x_193);
if (lean_is_exclusive(x_191)) {
 lean_ctor_release(x_191, 0);
 lean_ctor_release(x_191, 1);
 x_194 = x_191;
} else {
 lean_dec_ref(x_191);
 x_194 = lean_box(0);
}
lean_ctor_set_tag(x_179, 7);
lean_ctor_set(x_179, 1, x_192);
if (lean_is_scalar(x_194)) {
 x_195 = lean_alloc_ctor(0, 2, 0);
} else {
 x_195 = x_194;
}
lean_ctor_set(x_195, 0, x_179);
lean_ctor_set(x_195, 1, x_193);
x_196 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_196, 0, x_195);
return x_196;
}
}
}
else
{
lean_object* x_197; lean_object* x_198; lean_object* x_199; 
x_197 = lean_ctor_get(x_179, 0);
x_198 = lean_ctor_get(x_179, 1);
lean_inc(x_198);
lean_inc(x_197);
lean_dec(x_179);
x_199 = lp_iris_x2dkernel_IrisKernel_FFI_decodeCostBound(x_198);
if (lean_obj_tag(x_199) == 0)
{
lean_dec(x_197);
return x_199;
}
else
{
lean_object* x_200; lean_object* x_201; lean_object* x_202; lean_object* x_203; lean_object* x_204; lean_object* x_205; lean_object* x_206; lean_object* x_207; 
x_200 = lean_ctor_get(x_199, 0);
lean_inc(x_200);
if (lean_is_exclusive(x_199)) {
 lean_ctor_release(x_199, 0);
 x_201 = x_199;
} else {
 lean_dec_ref(x_199);
 x_201 = lean_box(0);
}
x_202 = lean_ctor_get(x_200, 0);
lean_inc(x_202);
x_203 = lean_ctor_get(x_200, 1);
lean_inc(x_203);
if (lean_is_exclusive(x_200)) {
 lean_ctor_release(x_200, 0);
 lean_ctor_release(x_200, 1);
 x_204 = x_200;
} else {
 lean_dec_ref(x_200);
 x_204 = lean_box(0);
}
x_205 = lean_alloc_ctor(7, 2, 0);
lean_ctor_set(x_205, 0, x_197);
lean_ctor_set(x_205, 1, x_202);
if (lean_is_scalar(x_204)) {
 x_206 = lean_alloc_ctor(0, 2, 0);
} else {
 x_206 = x_204;
}
lean_ctor_set(x_206, 0, x_205);
lean_ctor_set(x_206, 1, x_203);
if (lean_is_scalar(x_201)) {
 x_207 = lean_alloc_ctor(1, 1, 0);
} else {
 x_207 = x_201;
}
lean_ctor_set(x_207, 0, x_206);
return x_207;
}
}
}
}
}
else
{
lean_object* x_208; 
x_208 = lp_iris_x2dkernel_IrisKernel_FFI_decodeCostBound(x_8);
if (lean_obj_tag(x_208) == 0)
{
return x_208;
}
else
{
lean_object* x_209; uint8_t x_210; 
x_209 = lean_ctor_get(x_208, 0);
lean_inc(x_209);
lean_dec_ref(x_208);
x_210 = !lean_is_exclusive(x_209);
if (x_210 == 0)
{
lean_object* x_211; lean_object* x_212; lean_object* x_213; 
x_211 = lean_ctor_get(x_209, 0);
x_212 = lean_ctor_get(x_209, 1);
x_213 = lp_iris_x2dkernel_IrisKernel_FFI_decodeCostBound(x_212);
if (lean_obj_tag(x_213) == 0)
{
lean_free_object(x_209);
lean_dec(x_211);
return x_213;
}
else
{
uint8_t x_214; 
x_214 = !lean_is_exclusive(x_213);
if (x_214 == 0)
{
lean_object* x_215; uint8_t x_216; 
x_215 = lean_ctor_get(x_213, 0);
x_216 = !lean_is_exclusive(x_215);
if (x_216 == 0)
{
lean_object* x_217; 
x_217 = lean_ctor_get(x_215, 0);
lean_ctor_set_tag(x_209, 6);
lean_ctor_set(x_209, 1, x_217);
lean_ctor_set(x_215, 0, x_209);
return x_213;
}
else
{
lean_object* x_218; lean_object* x_219; lean_object* x_220; 
x_218 = lean_ctor_get(x_215, 0);
x_219 = lean_ctor_get(x_215, 1);
lean_inc(x_219);
lean_inc(x_218);
lean_dec(x_215);
lean_ctor_set_tag(x_209, 6);
lean_ctor_set(x_209, 1, x_218);
x_220 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_220, 0, x_209);
lean_ctor_set(x_220, 1, x_219);
lean_ctor_set(x_213, 0, x_220);
return x_213;
}
}
else
{
lean_object* x_221; lean_object* x_222; lean_object* x_223; lean_object* x_224; lean_object* x_225; lean_object* x_226; 
x_221 = lean_ctor_get(x_213, 0);
lean_inc(x_221);
lean_dec(x_213);
x_222 = lean_ctor_get(x_221, 0);
lean_inc(x_222);
x_223 = lean_ctor_get(x_221, 1);
lean_inc(x_223);
if (lean_is_exclusive(x_221)) {
 lean_ctor_release(x_221, 0);
 lean_ctor_release(x_221, 1);
 x_224 = x_221;
} else {
 lean_dec_ref(x_221);
 x_224 = lean_box(0);
}
lean_ctor_set_tag(x_209, 6);
lean_ctor_set(x_209, 1, x_222);
if (lean_is_scalar(x_224)) {
 x_225 = lean_alloc_ctor(0, 2, 0);
} else {
 x_225 = x_224;
}
lean_ctor_set(x_225, 0, x_209);
lean_ctor_set(x_225, 1, x_223);
x_226 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_226, 0, x_225);
return x_226;
}
}
}
else
{
lean_object* x_227; lean_object* x_228; lean_object* x_229; 
x_227 = lean_ctor_get(x_209, 0);
x_228 = lean_ctor_get(x_209, 1);
lean_inc(x_228);
lean_inc(x_227);
lean_dec(x_209);
x_229 = lp_iris_x2dkernel_IrisKernel_FFI_decodeCostBound(x_228);
if (lean_obj_tag(x_229) == 0)
{
lean_dec(x_227);
return x_229;
}
else
{
lean_object* x_230; lean_object* x_231; lean_object* x_232; lean_object* x_233; lean_object* x_234; lean_object* x_235; lean_object* x_236; lean_object* x_237; 
x_230 = lean_ctor_get(x_229, 0);
lean_inc(x_230);
if (lean_is_exclusive(x_229)) {
 lean_ctor_release(x_229, 0);
 x_231 = x_229;
} else {
 lean_dec_ref(x_229);
 x_231 = lean_box(0);
}
x_232 = lean_ctor_get(x_230, 0);
lean_inc(x_232);
x_233 = lean_ctor_get(x_230, 1);
lean_inc(x_233);
if (lean_is_exclusive(x_230)) {
 lean_ctor_release(x_230, 0);
 lean_ctor_release(x_230, 1);
 x_234 = x_230;
} else {
 lean_dec_ref(x_230);
 x_234 = lean_box(0);
}
x_235 = lean_alloc_ctor(6, 2, 0);
lean_ctor_set(x_235, 0, x_227);
lean_ctor_set(x_235, 1, x_232);
if (lean_is_scalar(x_234)) {
 x_236 = lean_alloc_ctor(0, 2, 0);
} else {
 x_236 = x_234;
}
lean_ctor_set(x_236, 0, x_235);
lean_ctor_set(x_236, 1, x_233);
if (lean_is_scalar(x_231)) {
 x_237 = lean_alloc_ctor(1, 1, 0);
} else {
 x_237 = x_231;
}
lean_ctor_set(x_237, 0, x_236);
return x_237;
}
}
}
}
}
else
{
lean_object* x_238; 
x_238 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readUInt32LE(x_8);
if (lean_obj_tag(x_238) == 0)
{
lean_object* x_239; 
x_239 = lean_box(0);
return x_239;
}
else
{
lean_object* x_240; uint8_t x_241; 
x_240 = lean_ctor_get(x_238, 0);
lean_inc(x_240);
lean_dec_ref(x_238);
x_241 = !lean_is_exclusive(x_240);
if (x_241 == 0)
{
lean_object* x_242; lean_object* x_243; lean_object* x_244; 
x_242 = lean_ctor_get(x_240, 0);
x_243 = lean_ctor_get(x_240, 1);
x_244 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readUInt32LE(x_243);
if (lean_obj_tag(x_244) == 0)
{
lean_object* x_245; 
lean_free_object(x_240);
lean_dec(x_242);
x_245 = lean_box(0);
return x_245;
}
else
{
uint8_t x_246; 
x_246 = !lean_is_exclusive(x_244);
if (x_246 == 0)
{
lean_object* x_247; uint8_t x_248; 
x_247 = lean_ctor_get(x_244, 0);
x_248 = !lean_is_exclusive(x_247);
if (x_248 == 0)
{
lean_object* x_249; uint32_t x_250; lean_object* x_251; uint32_t x_252; lean_object* x_253; 
x_249 = lean_ctor_get(x_247, 0);
x_250 = lean_unbox_uint32(x_242);
lean_dec(x_242);
x_251 = lean_uint32_to_nat(x_250);
x_252 = lean_unbox_uint32(x_249);
lean_dec(x_249);
x_253 = lean_uint32_to_nat(x_252);
lean_ctor_set_tag(x_240, 5);
lean_ctor_set(x_240, 1, x_253);
lean_ctor_set(x_240, 0, x_251);
lean_ctor_set(x_247, 0, x_240);
return x_244;
}
else
{
lean_object* x_254; lean_object* x_255; uint32_t x_256; lean_object* x_257; uint32_t x_258; lean_object* x_259; lean_object* x_260; 
x_254 = lean_ctor_get(x_247, 0);
x_255 = lean_ctor_get(x_247, 1);
lean_inc(x_255);
lean_inc(x_254);
lean_dec(x_247);
x_256 = lean_unbox_uint32(x_242);
lean_dec(x_242);
x_257 = lean_uint32_to_nat(x_256);
x_258 = lean_unbox_uint32(x_254);
lean_dec(x_254);
x_259 = lean_uint32_to_nat(x_258);
lean_ctor_set_tag(x_240, 5);
lean_ctor_set(x_240, 1, x_259);
lean_ctor_set(x_240, 0, x_257);
x_260 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_260, 0, x_240);
lean_ctor_set(x_260, 1, x_255);
lean_ctor_set(x_244, 0, x_260);
return x_244;
}
}
else
{
lean_object* x_261; lean_object* x_262; lean_object* x_263; lean_object* x_264; uint32_t x_265; lean_object* x_266; uint32_t x_267; lean_object* x_268; lean_object* x_269; lean_object* x_270; 
x_261 = lean_ctor_get(x_244, 0);
lean_inc(x_261);
lean_dec(x_244);
x_262 = lean_ctor_get(x_261, 0);
lean_inc(x_262);
x_263 = lean_ctor_get(x_261, 1);
lean_inc(x_263);
if (lean_is_exclusive(x_261)) {
 lean_ctor_release(x_261, 0);
 lean_ctor_release(x_261, 1);
 x_264 = x_261;
} else {
 lean_dec_ref(x_261);
 x_264 = lean_box(0);
}
x_265 = lean_unbox_uint32(x_242);
lean_dec(x_242);
x_266 = lean_uint32_to_nat(x_265);
x_267 = lean_unbox_uint32(x_262);
lean_dec(x_262);
x_268 = lean_uint32_to_nat(x_267);
lean_ctor_set_tag(x_240, 5);
lean_ctor_set(x_240, 1, x_268);
lean_ctor_set(x_240, 0, x_266);
if (lean_is_scalar(x_264)) {
 x_269 = lean_alloc_ctor(0, 2, 0);
} else {
 x_269 = x_264;
}
lean_ctor_set(x_269, 0, x_240);
lean_ctor_set(x_269, 1, x_263);
x_270 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_270, 0, x_269);
return x_270;
}
}
}
else
{
lean_object* x_271; lean_object* x_272; lean_object* x_273; 
x_271 = lean_ctor_get(x_240, 0);
x_272 = lean_ctor_get(x_240, 1);
lean_inc(x_272);
lean_inc(x_271);
lean_dec(x_240);
x_273 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readUInt32LE(x_272);
if (lean_obj_tag(x_273) == 0)
{
lean_object* x_274; 
lean_dec(x_271);
x_274 = lean_box(0);
return x_274;
}
else
{
lean_object* x_275; lean_object* x_276; lean_object* x_277; lean_object* x_278; lean_object* x_279; uint32_t x_280; lean_object* x_281; uint32_t x_282; lean_object* x_283; lean_object* x_284; lean_object* x_285; lean_object* x_286; 
x_275 = lean_ctor_get(x_273, 0);
lean_inc(x_275);
if (lean_is_exclusive(x_273)) {
 lean_ctor_release(x_273, 0);
 x_276 = x_273;
} else {
 lean_dec_ref(x_273);
 x_276 = lean_box(0);
}
x_277 = lean_ctor_get(x_275, 0);
lean_inc(x_277);
x_278 = lean_ctor_get(x_275, 1);
lean_inc(x_278);
if (lean_is_exclusive(x_275)) {
 lean_ctor_release(x_275, 0);
 lean_ctor_release(x_275, 1);
 x_279 = x_275;
} else {
 lean_dec_ref(x_275);
 x_279 = lean_box(0);
}
x_280 = lean_unbox_uint32(x_271);
lean_dec(x_271);
x_281 = lean_uint32_to_nat(x_280);
x_282 = lean_unbox_uint32(x_277);
lean_dec(x_277);
x_283 = lean_uint32_to_nat(x_282);
x_284 = lean_alloc_ctor(5, 2, 0);
lean_ctor_set(x_284, 0, x_281);
lean_ctor_set(x_284, 1, x_283);
if (lean_is_scalar(x_279)) {
 x_285 = lean_alloc_ctor(0, 2, 0);
} else {
 x_285 = x_279;
}
lean_ctor_set(x_285, 0, x_284);
lean_ctor_set(x_285, 1, x_278);
if (lean_is_scalar(x_276)) {
 x_286 = lean_alloc_ctor(1, 1, 0);
} else {
 x_286 = x_276;
}
lean_ctor_set(x_286, 0, x_285);
return x_286;
}
}
}
}
}
else
{
lean_object* x_287; 
x_287 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readUInt32LE(x_8);
if (lean_obj_tag(x_287) == 0)
{
lean_object* x_288; 
x_288 = lean_box(0);
return x_288;
}
else
{
uint8_t x_289; 
x_289 = !lean_is_exclusive(x_287);
if (x_289 == 0)
{
lean_object* x_290; uint8_t x_291; 
x_290 = lean_ctor_get(x_287, 0);
x_291 = !lean_is_exclusive(x_290);
if (x_291 == 0)
{
lean_object* x_292; uint32_t x_293; lean_object* x_294; lean_object* x_295; 
x_292 = lean_ctor_get(x_290, 0);
x_293 = lean_unbox_uint32(x_292);
lean_dec(x_292);
x_294 = lean_uint32_to_nat(x_293);
x_295 = lean_alloc_ctor(4, 1, 0);
lean_ctor_set(x_295, 0, x_294);
lean_ctor_set(x_290, 0, x_295);
return x_287;
}
else
{
lean_object* x_296; lean_object* x_297; uint32_t x_298; lean_object* x_299; lean_object* x_300; lean_object* x_301; 
x_296 = lean_ctor_get(x_290, 0);
x_297 = lean_ctor_get(x_290, 1);
lean_inc(x_297);
lean_inc(x_296);
lean_dec(x_290);
x_298 = lean_unbox_uint32(x_296);
lean_dec(x_296);
x_299 = lean_uint32_to_nat(x_298);
x_300 = lean_alloc_ctor(4, 1, 0);
lean_ctor_set(x_300, 0, x_299);
x_301 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_301, 0, x_300);
lean_ctor_set(x_301, 1, x_297);
lean_ctor_set(x_287, 0, x_301);
return x_287;
}
}
else
{
lean_object* x_302; lean_object* x_303; lean_object* x_304; lean_object* x_305; uint32_t x_306; lean_object* x_307; lean_object* x_308; lean_object* x_309; lean_object* x_310; 
x_302 = lean_ctor_get(x_287, 0);
lean_inc(x_302);
lean_dec(x_287);
x_303 = lean_ctor_get(x_302, 0);
lean_inc(x_303);
x_304 = lean_ctor_get(x_302, 1);
lean_inc(x_304);
if (lean_is_exclusive(x_302)) {
 lean_ctor_release(x_302, 0);
 lean_ctor_release(x_302, 1);
 x_305 = x_302;
} else {
 lean_dec_ref(x_302);
 x_305 = lean_box(0);
}
x_306 = lean_unbox_uint32(x_303);
lean_dec(x_303);
x_307 = lean_uint32_to_nat(x_306);
x_308 = lean_alloc_ctor(4, 1, 0);
lean_ctor_set(x_308, 0, x_307);
if (lean_is_scalar(x_305)) {
 x_309 = lean_alloc_ctor(0, 2, 0);
} else {
 x_309 = x_305;
}
lean_ctor_set(x_309, 0, x_308);
lean_ctor_set(x_309, 1, x_304);
x_310 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_310, 0, x_309);
return x_310;
}
}
}
}
else
{
lean_object* x_311; 
x_311 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readUInt32LE(x_8);
if (lean_obj_tag(x_311) == 0)
{
lean_object* x_312; 
x_312 = lean_box(0);
return x_312;
}
else
{
uint8_t x_313; 
x_313 = !lean_is_exclusive(x_311);
if (x_313 == 0)
{
lean_object* x_314; uint8_t x_315; 
x_314 = lean_ctor_get(x_311, 0);
x_315 = !lean_is_exclusive(x_314);
if (x_315 == 0)
{
lean_object* x_316; uint32_t x_317; lean_object* x_318; lean_object* x_319; 
x_316 = lean_ctor_get(x_314, 0);
x_317 = lean_unbox_uint32(x_316);
lean_dec(x_316);
x_318 = lean_uint32_to_nat(x_317);
x_319 = lean_alloc_ctor(3, 1, 0);
lean_ctor_set(x_319, 0, x_318);
lean_ctor_set(x_314, 0, x_319);
return x_311;
}
else
{
lean_object* x_320; lean_object* x_321; uint32_t x_322; lean_object* x_323; lean_object* x_324; lean_object* x_325; 
x_320 = lean_ctor_get(x_314, 0);
x_321 = lean_ctor_get(x_314, 1);
lean_inc(x_321);
lean_inc(x_320);
lean_dec(x_314);
x_322 = lean_unbox_uint32(x_320);
lean_dec(x_320);
x_323 = lean_uint32_to_nat(x_322);
x_324 = lean_alloc_ctor(3, 1, 0);
lean_ctor_set(x_324, 0, x_323);
x_325 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_325, 0, x_324);
lean_ctor_set(x_325, 1, x_321);
lean_ctor_set(x_311, 0, x_325);
return x_311;
}
}
else
{
lean_object* x_326; lean_object* x_327; lean_object* x_328; lean_object* x_329; uint32_t x_330; lean_object* x_331; lean_object* x_332; lean_object* x_333; lean_object* x_334; 
x_326 = lean_ctor_get(x_311, 0);
lean_inc(x_326);
lean_dec(x_311);
x_327 = lean_ctor_get(x_326, 0);
lean_inc(x_327);
x_328 = lean_ctor_get(x_326, 1);
lean_inc(x_328);
if (lean_is_exclusive(x_326)) {
 lean_ctor_release(x_326, 0);
 lean_ctor_release(x_326, 1);
 x_329 = x_326;
} else {
 lean_dec_ref(x_326);
 x_329 = lean_box(0);
}
x_330 = lean_unbox_uint32(x_327);
lean_dec(x_327);
x_331 = lean_uint32_to_nat(x_330);
x_332 = lean_alloc_ctor(3, 1, 0);
lean_ctor_set(x_332, 0, x_331);
if (lean_is_scalar(x_329)) {
 x_333 = lean_alloc_ctor(0, 2, 0);
} else {
 x_333 = x_329;
}
lean_ctor_set(x_333, 0, x_332);
lean_ctor_set(x_333, 1, x_328);
x_334 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_334, 0, x_333);
return x_334;
}
}
}
}
else
{
lean_object* x_335; 
x_335 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readUInt64LE(x_8);
if (lean_obj_tag(x_335) == 0)
{
lean_object* x_336; 
x_336 = lean_box(0);
return x_336;
}
else
{
uint8_t x_337; 
x_337 = !lean_is_exclusive(x_335);
if (x_337 == 0)
{
lean_object* x_338; uint8_t x_339; 
x_338 = lean_ctor_get(x_335, 0);
x_339 = !lean_is_exclusive(x_338);
if (x_339 == 0)
{
lean_object* x_340; uint64_t x_341; lean_object* x_342; lean_object* x_343; 
x_340 = lean_ctor_get(x_338, 0);
x_341 = lean_unbox_uint64(x_340);
lean_dec(x_340);
x_342 = lean_uint64_to_nat(x_341);
x_343 = lean_alloc_ctor(2, 1, 0);
lean_ctor_set(x_343, 0, x_342);
lean_ctor_set(x_338, 0, x_343);
return x_335;
}
else
{
lean_object* x_344; lean_object* x_345; uint64_t x_346; lean_object* x_347; lean_object* x_348; lean_object* x_349; 
x_344 = lean_ctor_get(x_338, 0);
x_345 = lean_ctor_get(x_338, 1);
lean_inc(x_345);
lean_inc(x_344);
lean_dec(x_338);
x_346 = lean_unbox_uint64(x_344);
lean_dec(x_344);
x_347 = lean_uint64_to_nat(x_346);
x_348 = lean_alloc_ctor(2, 1, 0);
lean_ctor_set(x_348, 0, x_347);
x_349 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_349, 0, x_348);
lean_ctor_set(x_349, 1, x_345);
lean_ctor_set(x_335, 0, x_349);
return x_335;
}
}
else
{
lean_object* x_350; lean_object* x_351; lean_object* x_352; lean_object* x_353; uint64_t x_354; lean_object* x_355; lean_object* x_356; lean_object* x_357; lean_object* x_358; 
x_350 = lean_ctor_get(x_335, 0);
lean_inc(x_350);
lean_dec(x_335);
x_351 = lean_ctor_get(x_350, 0);
lean_inc(x_351);
x_352 = lean_ctor_get(x_350, 1);
lean_inc(x_352);
if (lean_is_exclusive(x_350)) {
 lean_ctor_release(x_350, 0);
 lean_ctor_release(x_350, 1);
 x_353 = x_350;
} else {
 lean_dec_ref(x_350);
 x_353 = lean_box(0);
}
x_354 = lean_unbox_uint64(x_351);
lean_dec(x_351);
x_355 = lean_uint64_to_nat(x_354);
x_356 = lean_alloc_ctor(2, 1, 0);
lean_ctor_set(x_356, 0, x_355);
if (lean_is_scalar(x_353)) {
 x_357 = lean_alloc_ctor(0, 2, 0);
} else {
 x_357 = x_353;
}
lean_ctor_set(x_357, 0, x_356);
lean_ctor_set(x_357, 1, x_352);
x_358 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_358, 0, x_357);
return x_358;
}
}
}
}
else
{
lean_object* x_359; 
x_359 = lean_box(1);
lean_ctor_set(x_5, 0, x_359);
return x_2;
}
}
else
{
lean_object* x_360; 
x_360 = lean_box(0);
lean_ctor_set(x_5, 0, x_360);
return x_2;
}
}
else
{
lean_object* x_361; lean_object* x_362; uint8_t x_363; lean_object* x_364; lean_object* x_365; uint8_t x_366; 
x_361 = lean_ctor_get(x_5, 0);
x_362 = lean_ctor_get(x_5, 1);
lean_inc(x_362);
lean_inc(x_361);
lean_dec(x_5);
x_363 = lean_unbox(x_361);
lean_dec(x_361);
x_364 = lean_uint8_to_nat(x_363);
x_365 = lean_unsigned_to_nat(0u);
x_366 = lean_nat_dec_eq(x_364, x_365);
if (x_366 == 0)
{
lean_object* x_367; uint8_t x_368; 
x_367 = lean_unsigned_to_nat(1u);
x_368 = lean_nat_dec_eq(x_364, x_367);
if (x_368 == 0)
{
lean_object* x_369; uint8_t x_370; 
lean_free_object(x_2);
x_369 = lean_unsigned_to_nat(2u);
x_370 = lean_nat_dec_eq(x_364, x_369);
if (x_370 == 0)
{
lean_object* x_371; uint8_t x_372; 
x_371 = lean_unsigned_to_nat(3u);
x_372 = lean_nat_dec_eq(x_364, x_371);
if (x_372 == 0)
{
lean_object* x_373; uint8_t x_374; 
x_373 = lean_unsigned_to_nat(4u);
x_374 = lean_nat_dec_eq(x_364, x_373);
if (x_374 == 0)
{
lean_object* x_375; uint8_t x_376; 
x_375 = lean_unsigned_to_nat(5u);
x_376 = lean_nat_dec_eq(x_364, x_375);
if (x_376 == 0)
{
lean_object* x_377; uint8_t x_378; 
x_377 = lean_unsigned_to_nat(6u);
x_378 = lean_nat_dec_eq(x_364, x_377);
if (x_378 == 0)
{
lean_object* x_379; uint8_t x_380; 
x_379 = lean_unsigned_to_nat(7u);
x_380 = lean_nat_dec_eq(x_364, x_379);
if (x_380 == 0)
{
lean_object* x_381; uint8_t x_382; 
x_381 = lean_unsigned_to_nat(8u);
x_382 = lean_nat_dec_eq(x_364, x_381);
if (x_382 == 0)
{
lean_object* x_383; uint8_t x_384; 
x_383 = lean_unsigned_to_nat(9u);
x_384 = lean_nat_dec_eq(x_364, x_383);
if (x_384 == 0)
{
lean_object* x_385; uint8_t x_386; 
x_385 = lean_unsigned_to_nat(10u);
x_386 = lean_nat_dec_eq(x_364, x_385);
if (x_386 == 0)
{
lean_object* x_387; uint8_t x_388; 
x_387 = lean_unsigned_to_nat(11u);
x_388 = lean_nat_dec_eq(x_364, x_387);
if (x_388 == 0)
{
lean_object* x_389; uint8_t x_390; 
x_389 = lean_unsigned_to_nat(12u);
x_390 = lean_nat_dec_eq(x_364, x_389);
if (x_390 == 0)
{
lean_object* x_391; 
lean_dec(x_362);
x_391 = lean_box(0);
return x_391;
}
else
{
lean_object* x_392; 
x_392 = lp_iris_x2dkernel_IrisKernel_FFI_decodeCostBound(x_362);
if (lean_obj_tag(x_392) == 0)
{
return x_392;
}
else
{
lean_object* x_393; lean_object* x_394; lean_object* x_395; lean_object* x_396; lean_object* x_397; lean_object* x_398; lean_object* x_399; lean_object* x_400; 
x_393 = lean_ctor_get(x_392, 0);
lean_inc(x_393);
if (lean_is_exclusive(x_392)) {
 lean_ctor_release(x_392, 0);
 x_394 = x_392;
} else {
 lean_dec_ref(x_392);
 x_394 = lean_box(0);
}
x_395 = lean_ctor_get(x_393, 0);
lean_inc(x_395);
x_396 = lean_ctor_get(x_393, 1);
lean_inc(x_396);
if (lean_is_exclusive(x_393)) {
 lean_ctor_release(x_393, 0);
 lean_ctor_release(x_393, 1);
 x_397 = x_393;
} else {
 lean_dec_ref(x_393);
 x_397 = lean_box(0);
}
x_398 = lean_alloc_ctor(12, 1, 0);
lean_ctor_set(x_398, 0, x_395);
if (lean_is_scalar(x_397)) {
 x_399 = lean_alloc_ctor(0, 2, 0);
} else {
 x_399 = x_397;
}
lean_ctor_set(x_399, 0, x_398);
lean_ctor_set(x_399, 1, x_396);
if (lean_is_scalar(x_394)) {
 x_400 = lean_alloc_ctor(1, 1, 0);
} else {
 x_400 = x_394;
}
lean_ctor_set(x_400, 0, x_399);
return x_400;
}
}
}
else
{
lean_object* x_401; 
x_401 = lp_iris_x2dkernel_IrisKernel_FFI_decodeCostBound(x_362);
if (lean_obj_tag(x_401) == 0)
{
return x_401;
}
else
{
lean_object* x_402; lean_object* x_403; lean_object* x_404; lean_object* x_405; lean_object* x_406; lean_object* x_407; lean_object* x_408; lean_object* x_409; 
x_402 = lean_ctor_get(x_401, 0);
lean_inc(x_402);
if (lean_is_exclusive(x_401)) {
 lean_ctor_release(x_401, 0);
 x_403 = x_401;
} else {
 lean_dec_ref(x_401);
 x_403 = lean_box(0);
}
x_404 = lean_ctor_get(x_402, 0);
lean_inc(x_404);
x_405 = lean_ctor_get(x_402, 1);
lean_inc(x_405);
if (lean_is_exclusive(x_402)) {
 lean_ctor_release(x_402, 0);
 lean_ctor_release(x_402, 1);
 x_406 = x_402;
} else {
 lean_dec_ref(x_402);
 x_406 = lean_box(0);
}
x_407 = lean_alloc_ctor(11, 1, 0);
lean_ctor_set(x_407, 0, x_404);
if (lean_is_scalar(x_406)) {
 x_408 = lean_alloc_ctor(0, 2, 0);
} else {
 x_408 = x_406;
}
lean_ctor_set(x_408, 0, x_407);
lean_ctor_set(x_408, 1, x_405);
if (lean_is_scalar(x_403)) {
 x_409 = lean_alloc_ctor(1, 1, 0);
} else {
 x_409 = x_403;
}
lean_ctor_set(x_409, 0, x_408);
return x_409;
}
}
}
else
{
lean_object* x_410; 
x_410 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readUInt16LE(x_362);
if (lean_obj_tag(x_410) == 0)
{
lean_object* x_411; 
x_411 = lean_box(0);
return x_411;
}
else
{
lean_object* x_412; lean_object* x_413; lean_object* x_414; lean_object* x_415; uint16_t x_416; lean_object* x_417; lean_object* x_418; 
x_412 = lean_ctor_get(x_410, 0);
lean_inc(x_412);
if (lean_is_exclusive(x_410)) {
 lean_ctor_release(x_410, 0);
 x_413 = x_410;
} else {
 lean_dec_ref(x_410);
 x_413 = lean_box(0);
}
x_414 = lean_ctor_get(x_412, 0);
lean_inc(x_414);
x_415 = lean_ctor_get(x_412, 1);
lean_inc(x_415);
lean_dec(x_412);
x_416 = lean_unbox(x_414);
lean_dec(x_414);
x_417 = lean_uint16_to_nat(x_416);
x_418 = lp_iris_x2dkernel_IrisKernel_FFI_decodeCostBound_decodeCostBoundList(x_415, x_417);
if (lean_obj_tag(x_418) == 0)
{
lean_object* x_419; 
lean_dec(x_413);
x_419 = lean_box(0);
return x_419;
}
else
{
lean_object* x_420; lean_object* x_421; lean_object* x_422; lean_object* x_423; lean_object* x_424; lean_object* x_425; lean_object* x_426; lean_object* x_427; 
x_420 = lean_ctor_get(x_418, 0);
lean_inc(x_420);
if (lean_is_exclusive(x_418)) {
 lean_ctor_release(x_418, 0);
 x_421 = x_418;
} else {
 lean_dec_ref(x_418);
 x_421 = lean_box(0);
}
x_422 = lean_ctor_get(x_420, 0);
lean_inc(x_422);
x_423 = lean_ctor_get(x_420, 1);
lean_inc(x_423);
if (lean_is_exclusive(x_420)) {
 lean_ctor_release(x_420, 0);
 lean_ctor_release(x_420, 1);
 x_424 = x_420;
} else {
 lean_dec_ref(x_420);
 x_424 = lean_box(0);
}
if (lean_is_scalar(x_413)) {
 x_425 = lean_alloc_ctor(10, 1, 0);
} else {
 x_425 = x_413;
 lean_ctor_set_tag(x_425, 10);
}
lean_ctor_set(x_425, 0, x_422);
if (lean_is_scalar(x_424)) {
 x_426 = lean_alloc_ctor(0, 2, 0);
} else {
 x_426 = x_424;
}
lean_ctor_set(x_426, 0, x_425);
lean_ctor_set(x_426, 1, x_423);
if (lean_is_scalar(x_421)) {
 x_427 = lean_alloc_ctor(1, 1, 0);
} else {
 x_427 = x_421;
}
lean_ctor_set(x_427, 0, x_426);
return x_427;
}
}
}
}
else
{
lean_object* x_428; 
x_428 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readUInt16LE(x_362);
if (lean_obj_tag(x_428) == 0)
{
lean_object* x_429; 
x_429 = lean_box(0);
return x_429;
}
else
{
lean_object* x_430; lean_object* x_431; lean_object* x_432; lean_object* x_433; uint16_t x_434; lean_object* x_435; lean_object* x_436; 
x_430 = lean_ctor_get(x_428, 0);
lean_inc(x_430);
if (lean_is_exclusive(x_428)) {
 lean_ctor_release(x_428, 0);
 x_431 = x_428;
} else {
 lean_dec_ref(x_428);
 x_431 = lean_box(0);
}
x_432 = lean_ctor_get(x_430, 0);
lean_inc(x_432);
x_433 = lean_ctor_get(x_430, 1);
lean_inc(x_433);
lean_dec(x_430);
x_434 = lean_unbox(x_432);
lean_dec(x_432);
x_435 = lean_uint16_to_nat(x_434);
x_436 = lp_iris_x2dkernel_IrisKernel_FFI_decodeCostBound_decodeCostBoundList(x_433, x_435);
if (lean_obj_tag(x_436) == 0)
{
lean_object* x_437; 
lean_dec(x_431);
x_437 = lean_box(0);
return x_437;
}
else
{
lean_object* x_438; lean_object* x_439; lean_object* x_440; lean_object* x_441; lean_object* x_442; lean_object* x_443; lean_object* x_444; lean_object* x_445; 
x_438 = lean_ctor_get(x_436, 0);
lean_inc(x_438);
if (lean_is_exclusive(x_436)) {
 lean_ctor_release(x_436, 0);
 x_439 = x_436;
} else {
 lean_dec_ref(x_436);
 x_439 = lean_box(0);
}
x_440 = lean_ctor_get(x_438, 0);
lean_inc(x_440);
x_441 = lean_ctor_get(x_438, 1);
lean_inc(x_441);
if (lean_is_exclusive(x_438)) {
 lean_ctor_release(x_438, 0);
 lean_ctor_release(x_438, 1);
 x_442 = x_438;
} else {
 lean_dec_ref(x_438);
 x_442 = lean_box(0);
}
if (lean_is_scalar(x_431)) {
 x_443 = lean_alloc_ctor(9, 1, 0);
} else {
 x_443 = x_431;
 lean_ctor_set_tag(x_443, 9);
}
lean_ctor_set(x_443, 0, x_440);
if (lean_is_scalar(x_442)) {
 x_444 = lean_alloc_ctor(0, 2, 0);
} else {
 x_444 = x_442;
}
lean_ctor_set(x_444, 0, x_443);
lean_ctor_set(x_444, 1, x_441);
if (lean_is_scalar(x_439)) {
 x_445 = lean_alloc_ctor(1, 1, 0);
} else {
 x_445 = x_439;
}
lean_ctor_set(x_445, 0, x_444);
return x_445;
}
}
}
}
else
{
lean_object* x_446; 
x_446 = lp_iris_x2dkernel_IrisKernel_FFI_decodeCostBound(x_362);
if (lean_obj_tag(x_446) == 0)
{
return x_446;
}
else
{
lean_object* x_447; lean_object* x_448; lean_object* x_449; lean_object* x_450; lean_object* x_451; 
x_447 = lean_ctor_get(x_446, 0);
lean_inc(x_447);
lean_dec_ref(x_446);
x_448 = lean_ctor_get(x_447, 0);
lean_inc(x_448);
x_449 = lean_ctor_get(x_447, 1);
lean_inc(x_449);
if (lean_is_exclusive(x_447)) {
 lean_ctor_release(x_447, 0);
 lean_ctor_release(x_447, 1);
 x_450 = x_447;
} else {
 lean_dec_ref(x_447);
 x_450 = lean_box(0);
}
x_451 = lp_iris_x2dkernel_IrisKernel_FFI_decodeCostBound(x_449);
if (lean_obj_tag(x_451) == 0)
{
lean_dec(x_450);
lean_dec(x_448);
return x_451;
}
else
{
lean_object* x_452; lean_object* x_453; lean_object* x_454; lean_object* x_455; lean_object* x_456; lean_object* x_457; lean_object* x_458; lean_object* x_459; 
x_452 = lean_ctor_get(x_451, 0);
lean_inc(x_452);
if (lean_is_exclusive(x_451)) {
 lean_ctor_release(x_451, 0);
 x_453 = x_451;
} else {
 lean_dec_ref(x_451);
 x_453 = lean_box(0);
}
x_454 = lean_ctor_get(x_452, 0);
lean_inc(x_454);
x_455 = lean_ctor_get(x_452, 1);
lean_inc(x_455);
if (lean_is_exclusive(x_452)) {
 lean_ctor_release(x_452, 0);
 lean_ctor_release(x_452, 1);
 x_456 = x_452;
} else {
 lean_dec_ref(x_452);
 x_456 = lean_box(0);
}
if (lean_is_scalar(x_450)) {
 x_457 = lean_alloc_ctor(8, 2, 0);
} else {
 x_457 = x_450;
 lean_ctor_set_tag(x_457, 8);
}
lean_ctor_set(x_457, 0, x_448);
lean_ctor_set(x_457, 1, x_454);
if (lean_is_scalar(x_456)) {
 x_458 = lean_alloc_ctor(0, 2, 0);
} else {
 x_458 = x_456;
}
lean_ctor_set(x_458, 0, x_457);
lean_ctor_set(x_458, 1, x_455);
if (lean_is_scalar(x_453)) {
 x_459 = lean_alloc_ctor(1, 1, 0);
} else {
 x_459 = x_453;
}
lean_ctor_set(x_459, 0, x_458);
return x_459;
}
}
}
}
else
{
lean_object* x_460; 
x_460 = lp_iris_x2dkernel_IrisKernel_FFI_decodeCostBound(x_362);
if (lean_obj_tag(x_460) == 0)
{
return x_460;
}
else
{
lean_object* x_461; lean_object* x_462; lean_object* x_463; lean_object* x_464; lean_object* x_465; 
x_461 = lean_ctor_get(x_460, 0);
lean_inc(x_461);
lean_dec_ref(x_460);
x_462 = lean_ctor_get(x_461, 0);
lean_inc(x_462);
x_463 = lean_ctor_get(x_461, 1);
lean_inc(x_463);
if (lean_is_exclusive(x_461)) {
 lean_ctor_release(x_461, 0);
 lean_ctor_release(x_461, 1);
 x_464 = x_461;
} else {
 lean_dec_ref(x_461);
 x_464 = lean_box(0);
}
x_465 = lp_iris_x2dkernel_IrisKernel_FFI_decodeCostBound(x_463);
if (lean_obj_tag(x_465) == 0)
{
lean_dec(x_464);
lean_dec(x_462);
return x_465;
}
else
{
lean_object* x_466; lean_object* x_467; lean_object* x_468; lean_object* x_469; lean_object* x_470; lean_object* x_471; lean_object* x_472; lean_object* x_473; 
x_466 = lean_ctor_get(x_465, 0);
lean_inc(x_466);
if (lean_is_exclusive(x_465)) {
 lean_ctor_release(x_465, 0);
 x_467 = x_465;
} else {
 lean_dec_ref(x_465);
 x_467 = lean_box(0);
}
x_468 = lean_ctor_get(x_466, 0);
lean_inc(x_468);
x_469 = lean_ctor_get(x_466, 1);
lean_inc(x_469);
if (lean_is_exclusive(x_466)) {
 lean_ctor_release(x_466, 0);
 lean_ctor_release(x_466, 1);
 x_470 = x_466;
} else {
 lean_dec_ref(x_466);
 x_470 = lean_box(0);
}
if (lean_is_scalar(x_464)) {
 x_471 = lean_alloc_ctor(7, 2, 0);
} else {
 x_471 = x_464;
 lean_ctor_set_tag(x_471, 7);
}
lean_ctor_set(x_471, 0, x_462);
lean_ctor_set(x_471, 1, x_468);
if (lean_is_scalar(x_470)) {
 x_472 = lean_alloc_ctor(0, 2, 0);
} else {
 x_472 = x_470;
}
lean_ctor_set(x_472, 0, x_471);
lean_ctor_set(x_472, 1, x_469);
if (lean_is_scalar(x_467)) {
 x_473 = lean_alloc_ctor(1, 1, 0);
} else {
 x_473 = x_467;
}
lean_ctor_set(x_473, 0, x_472);
return x_473;
}
}
}
}
else
{
lean_object* x_474; 
x_474 = lp_iris_x2dkernel_IrisKernel_FFI_decodeCostBound(x_362);
if (lean_obj_tag(x_474) == 0)
{
return x_474;
}
else
{
lean_object* x_475; lean_object* x_476; lean_object* x_477; lean_object* x_478; lean_object* x_479; 
x_475 = lean_ctor_get(x_474, 0);
lean_inc(x_475);
lean_dec_ref(x_474);
x_476 = lean_ctor_get(x_475, 0);
lean_inc(x_476);
x_477 = lean_ctor_get(x_475, 1);
lean_inc(x_477);
if (lean_is_exclusive(x_475)) {
 lean_ctor_release(x_475, 0);
 lean_ctor_release(x_475, 1);
 x_478 = x_475;
} else {
 lean_dec_ref(x_475);
 x_478 = lean_box(0);
}
x_479 = lp_iris_x2dkernel_IrisKernel_FFI_decodeCostBound(x_477);
if (lean_obj_tag(x_479) == 0)
{
lean_dec(x_478);
lean_dec(x_476);
return x_479;
}
else
{
lean_object* x_480; lean_object* x_481; lean_object* x_482; lean_object* x_483; lean_object* x_484; lean_object* x_485; lean_object* x_486; lean_object* x_487; 
x_480 = lean_ctor_get(x_479, 0);
lean_inc(x_480);
if (lean_is_exclusive(x_479)) {
 lean_ctor_release(x_479, 0);
 x_481 = x_479;
} else {
 lean_dec_ref(x_479);
 x_481 = lean_box(0);
}
x_482 = lean_ctor_get(x_480, 0);
lean_inc(x_482);
x_483 = lean_ctor_get(x_480, 1);
lean_inc(x_483);
if (lean_is_exclusive(x_480)) {
 lean_ctor_release(x_480, 0);
 lean_ctor_release(x_480, 1);
 x_484 = x_480;
} else {
 lean_dec_ref(x_480);
 x_484 = lean_box(0);
}
if (lean_is_scalar(x_478)) {
 x_485 = lean_alloc_ctor(6, 2, 0);
} else {
 x_485 = x_478;
 lean_ctor_set_tag(x_485, 6);
}
lean_ctor_set(x_485, 0, x_476);
lean_ctor_set(x_485, 1, x_482);
if (lean_is_scalar(x_484)) {
 x_486 = lean_alloc_ctor(0, 2, 0);
} else {
 x_486 = x_484;
}
lean_ctor_set(x_486, 0, x_485);
lean_ctor_set(x_486, 1, x_483);
if (lean_is_scalar(x_481)) {
 x_487 = lean_alloc_ctor(1, 1, 0);
} else {
 x_487 = x_481;
}
lean_ctor_set(x_487, 0, x_486);
return x_487;
}
}
}
}
else
{
lean_object* x_488; 
x_488 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readUInt32LE(x_362);
if (lean_obj_tag(x_488) == 0)
{
lean_object* x_489; 
x_489 = lean_box(0);
return x_489;
}
else
{
lean_object* x_490; lean_object* x_491; lean_object* x_492; lean_object* x_493; lean_object* x_494; 
x_490 = lean_ctor_get(x_488, 0);
lean_inc(x_490);
lean_dec_ref(x_488);
x_491 = lean_ctor_get(x_490, 0);
lean_inc(x_491);
x_492 = lean_ctor_get(x_490, 1);
lean_inc(x_492);
if (lean_is_exclusive(x_490)) {
 lean_ctor_release(x_490, 0);
 lean_ctor_release(x_490, 1);
 x_493 = x_490;
} else {
 lean_dec_ref(x_490);
 x_493 = lean_box(0);
}
x_494 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readUInt32LE(x_492);
if (lean_obj_tag(x_494) == 0)
{
lean_object* x_495; 
lean_dec(x_493);
lean_dec(x_491);
x_495 = lean_box(0);
return x_495;
}
else
{
lean_object* x_496; lean_object* x_497; lean_object* x_498; lean_object* x_499; lean_object* x_500; uint32_t x_501; lean_object* x_502; uint32_t x_503; lean_object* x_504; lean_object* x_505; lean_object* x_506; lean_object* x_507; 
x_496 = lean_ctor_get(x_494, 0);
lean_inc(x_496);
if (lean_is_exclusive(x_494)) {
 lean_ctor_release(x_494, 0);
 x_497 = x_494;
} else {
 lean_dec_ref(x_494);
 x_497 = lean_box(0);
}
x_498 = lean_ctor_get(x_496, 0);
lean_inc(x_498);
x_499 = lean_ctor_get(x_496, 1);
lean_inc(x_499);
if (lean_is_exclusive(x_496)) {
 lean_ctor_release(x_496, 0);
 lean_ctor_release(x_496, 1);
 x_500 = x_496;
} else {
 lean_dec_ref(x_496);
 x_500 = lean_box(0);
}
x_501 = lean_unbox_uint32(x_491);
lean_dec(x_491);
x_502 = lean_uint32_to_nat(x_501);
x_503 = lean_unbox_uint32(x_498);
lean_dec(x_498);
x_504 = lean_uint32_to_nat(x_503);
if (lean_is_scalar(x_493)) {
 x_505 = lean_alloc_ctor(5, 2, 0);
} else {
 x_505 = x_493;
 lean_ctor_set_tag(x_505, 5);
}
lean_ctor_set(x_505, 0, x_502);
lean_ctor_set(x_505, 1, x_504);
if (lean_is_scalar(x_500)) {
 x_506 = lean_alloc_ctor(0, 2, 0);
} else {
 x_506 = x_500;
}
lean_ctor_set(x_506, 0, x_505);
lean_ctor_set(x_506, 1, x_499);
if (lean_is_scalar(x_497)) {
 x_507 = lean_alloc_ctor(1, 1, 0);
} else {
 x_507 = x_497;
}
lean_ctor_set(x_507, 0, x_506);
return x_507;
}
}
}
}
else
{
lean_object* x_508; 
x_508 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readUInt32LE(x_362);
if (lean_obj_tag(x_508) == 0)
{
lean_object* x_509; 
x_509 = lean_box(0);
return x_509;
}
else
{
lean_object* x_510; lean_object* x_511; lean_object* x_512; lean_object* x_513; lean_object* x_514; uint32_t x_515; lean_object* x_516; lean_object* x_517; lean_object* x_518; lean_object* x_519; 
x_510 = lean_ctor_get(x_508, 0);
lean_inc(x_510);
if (lean_is_exclusive(x_508)) {
 lean_ctor_release(x_508, 0);
 x_511 = x_508;
} else {
 lean_dec_ref(x_508);
 x_511 = lean_box(0);
}
x_512 = lean_ctor_get(x_510, 0);
lean_inc(x_512);
x_513 = lean_ctor_get(x_510, 1);
lean_inc(x_513);
if (lean_is_exclusive(x_510)) {
 lean_ctor_release(x_510, 0);
 lean_ctor_release(x_510, 1);
 x_514 = x_510;
} else {
 lean_dec_ref(x_510);
 x_514 = lean_box(0);
}
x_515 = lean_unbox_uint32(x_512);
lean_dec(x_512);
x_516 = lean_uint32_to_nat(x_515);
x_517 = lean_alloc_ctor(4, 1, 0);
lean_ctor_set(x_517, 0, x_516);
if (lean_is_scalar(x_514)) {
 x_518 = lean_alloc_ctor(0, 2, 0);
} else {
 x_518 = x_514;
}
lean_ctor_set(x_518, 0, x_517);
lean_ctor_set(x_518, 1, x_513);
if (lean_is_scalar(x_511)) {
 x_519 = lean_alloc_ctor(1, 1, 0);
} else {
 x_519 = x_511;
}
lean_ctor_set(x_519, 0, x_518);
return x_519;
}
}
}
else
{
lean_object* x_520; 
x_520 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readUInt32LE(x_362);
if (lean_obj_tag(x_520) == 0)
{
lean_object* x_521; 
x_521 = lean_box(0);
return x_521;
}
else
{
lean_object* x_522; lean_object* x_523; lean_object* x_524; lean_object* x_525; lean_object* x_526; uint32_t x_527; lean_object* x_528; lean_object* x_529; lean_object* x_530; lean_object* x_531; 
x_522 = lean_ctor_get(x_520, 0);
lean_inc(x_522);
if (lean_is_exclusive(x_520)) {
 lean_ctor_release(x_520, 0);
 x_523 = x_520;
} else {
 lean_dec_ref(x_520);
 x_523 = lean_box(0);
}
x_524 = lean_ctor_get(x_522, 0);
lean_inc(x_524);
x_525 = lean_ctor_get(x_522, 1);
lean_inc(x_525);
if (lean_is_exclusive(x_522)) {
 lean_ctor_release(x_522, 0);
 lean_ctor_release(x_522, 1);
 x_526 = x_522;
} else {
 lean_dec_ref(x_522);
 x_526 = lean_box(0);
}
x_527 = lean_unbox_uint32(x_524);
lean_dec(x_524);
x_528 = lean_uint32_to_nat(x_527);
x_529 = lean_alloc_ctor(3, 1, 0);
lean_ctor_set(x_529, 0, x_528);
if (lean_is_scalar(x_526)) {
 x_530 = lean_alloc_ctor(0, 2, 0);
} else {
 x_530 = x_526;
}
lean_ctor_set(x_530, 0, x_529);
lean_ctor_set(x_530, 1, x_525);
if (lean_is_scalar(x_523)) {
 x_531 = lean_alloc_ctor(1, 1, 0);
} else {
 x_531 = x_523;
}
lean_ctor_set(x_531, 0, x_530);
return x_531;
}
}
}
else
{
lean_object* x_532; 
x_532 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readUInt64LE(x_362);
if (lean_obj_tag(x_532) == 0)
{
lean_object* x_533; 
x_533 = lean_box(0);
return x_533;
}
else
{
lean_object* x_534; lean_object* x_535; lean_object* x_536; lean_object* x_537; lean_object* x_538; uint64_t x_539; lean_object* x_540; lean_object* x_541; lean_object* x_542; lean_object* x_543; 
x_534 = lean_ctor_get(x_532, 0);
lean_inc(x_534);
if (lean_is_exclusive(x_532)) {
 lean_ctor_release(x_532, 0);
 x_535 = x_532;
} else {
 lean_dec_ref(x_532);
 x_535 = lean_box(0);
}
x_536 = lean_ctor_get(x_534, 0);
lean_inc(x_536);
x_537 = lean_ctor_get(x_534, 1);
lean_inc(x_537);
if (lean_is_exclusive(x_534)) {
 lean_ctor_release(x_534, 0);
 lean_ctor_release(x_534, 1);
 x_538 = x_534;
} else {
 lean_dec_ref(x_534);
 x_538 = lean_box(0);
}
x_539 = lean_unbox_uint64(x_536);
lean_dec(x_536);
x_540 = lean_uint64_to_nat(x_539);
x_541 = lean_alloc_ctor(2, 1, 0);
lean_ctor_set(x_541, 0, x_540);
if (lean_is_scalar(x_538)) {
 x_542 = lean_alloc_ctor(0, 2, 0);
} else {
 x_542 = x_538;
}
lean_ctor_set(x_542, 0, x_541);
lean_ctor_set(x_542, 1, x_537);
if (lean_is_scalar(x_535)) {
 x_543 = lean_alloc_ctor(1, 1, 0);
} else {
 x_543 = x_535;
}
lean_ctor_set(x_543, 0, x_542);
return x_543;
}
}
}
else
{
lean_object* x_544; lean_object* x_545; 
x_544 = lean_box(1);
x_545 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_545, 0, x_544);
lean_ctor_set(x_545, 1, x_362);
lean_ctor_set(x_2, 0, x_545);
return x_2;
}
}
else
{
lean_object* x_546; lean_object* x_547; 
x_546 = lean_box(0);
x_547 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_547, 0, x_546);
lean_ctor_set(x_547, 1, x_362);
lean_ctor_set(x_2, 0, x_547);
return x_2;
}
}
}
else
{
lean_object* x_548; lean_object* x_549; lean_object* x_550; lean_object* x_551; uint8_t x_552; lean_object* x_553; lean_object* x_554; uint8_t x_555; 
x_548 = lean_ctor_get(x_2, 0);
lean_inc(x_548);
lean_dec(x_2);
x_549 = lean_ctor_get(x_548, 0);
lean_inc(x_549);
x_550 = lean_ctor_get(x_548, 1);
lean_inc(x_550);
if (lean_is_exclusive(x_548)) {
 lean_ctor_release(x_548, 0);
 lean_ctor_release(x_548, 1);
 x_551 = x_548;
} else {
 lean_dec_ref(x_548);
 x_551 = lean_box(0);
}
x_552 = lean_unbox(x_549);
lean_dec(x_549);
x_553 = lean_uint8_to_nat(x_552);
x_554 = lean_unsigned_to_nat(0u);
x_555 = lean_nat_dec_eq(x_553, x_554);
if (x_555 == 0)
{
lean_object* x_556; uint8_t x_557; 
x_556 = lean_unsigned_to_nat(1u);
x_557 = lean_nat_dec_eq(x_553, x_556);
if (x_557 == 0)
{
lean_object* x_558; uint8_t x_559; 
lean_dec(x_551);
x_558 = lean_unsigned_to_nat(2u);
x_559 = lean_nat_dec_eq(x_553, x_558);
if (x_559 == 0)
{
lean_object* x_560; uint8_t x_561; 
x_560 = lean_unsigned_to_nat(3u);
x_561 = lean_nat_dec_eq(x_553, x_560);
if (x_561 == 0)
{
lean_object* x_562; uint8_t x_563; 
x_562 = lean_unsigned_to_nat(4u);
x_563 = lean_nat_dec_eq(x_553, x_562);
if (x_563 == 0)
{
lean_object* x_564; uint8_t x_565; 
x_564 = lean_unsigned_to_nat(5u);
x_565 = lean_nat_dec_eq(x_553, x_564);
if (x_565 == 0)
{
lean_object* x_566; uint8_t x_567; 
x_566 = lean_unsigned_to_nat(6u);
x_567 = lean_nat_dec_eq(x_553, x_566);
if (x_567 == 0)
{
lean_object* x_568; uint8_t x_569; 
x_568 = lean_unsigned_to_nat(7u);
x_569 = lean_nat_dec_eq(x_553, x_568);
if (x_569 == 0)
{
lean_object* x_570; uint8_t x_571; 
x_570 = lean_unsigned_to_nat(8u);
x_571 = lean_nat_dec_eq(x_553, x_570);
if (x_571 == 0)
{
lean_object* x_572; uint8_t x_573; 
x_572 = lean_unsigned_to_nat(9u);
x_573 = lean_nat_dec_eq(x_553, x_572);
if (x_573 == 0)
{
lean_object* x_574; uint8_t x_575; 
x_574 = lean_unsigned_to_nat(10u);
x_575 = lean_nat_dec_eq(x_553, x_574);
if (x_575 == 0)
{
lean_object* x_576; uint8_t x_577; 
x_576 = lean_unsigned_to_nat(11u);
x_577 = lean_nat_dec_eq(x_553, x_576);
if (x_577 == 0)
{
lean_object* x_578; uint8_t x_579; 
x_578 = lean_unsigned_to_nat(12u);
x_579 = lean_nat_dec_eq(x_553, x_578);
if (x_579 == 0)
{
lean_object* x_580; 
lean_dec(x_550);
x_580 = lean_box(0);
return x_580;
}
else
{
lean_object* x_581; 
x_581 = lp_iris_x2dkernel_IrisKernel_FFI_decodeCostBound(x_550);
if (lean_obj_tag(x_581) == 0)
{
return x_581;
}
else
{
lean_object* x_582; lean_object* x_583; lean_object* x_584; lean_object* x_585; lean_object* x_586; lean_object* x_587; lean_object* x_588; lean_object* x_589; 
x_582 = lean_ctor_get(x_581, 0);
lean_inc(x_582);
if (lean_is_exclusive(x_581)) {
 lean_ctor_release(x_581, 0);
 x_583 = x_581;
} else {
 lean_dec_ref(x_581);
 x_583 = lean_box(0);
}
x_584 = lean_ctor_get(x_582, 0);
lean_inc(x_584);
x_585 = lean_ctor_get(x_582, 1);
lean_inc(x_585);
if (lean_is_exclusive(x_582)) {
 lean_ctor_release(x_582, 0);
 lean_ctor_release(x_582, 1);
 x_586 = x_582;
} else {
 lean_dec_ref(x_582);
 x_586 = lean_box(0);
}
x_587 = lean_alloc_ctor(12, 1, 0);
lean_ctor_set(x_587, 0, x_584);
if (lean_is_scalar(x_586)) {
 x_588 = lean_alloc_ctor(0, 2, 0);
} else {
 x_588 = x_586;
}
lean_ctor_set(x_588, 0, x_587);
lean_ctor_set(x_588, 1, x_585);
if (lean_is_scalar(x_583)) {
 x_589 = lean_alloc_ctor(1, 1, 0);
} else {
 x_589 = x_583;
}
lean_ctor_set(x_589, 0, x_588);
return x_589;
}
}
}
else
{
lean_object* x_590; 
x_590 = lp_iris_x2dkernel_IrisKernel_FFI_decodeCostBound(x_550);
if (lean_obj_tag(x_590) == 0)
{
return x_590;
}
else
{
lean_object* x_591; lean_object* x_592; lean_object* x_593; lean_object* x_594; lean_object* x_595; lean_object* x_596; lean_object* x_597; lean_object* x_598; 
x_591 = lean_ctor_get(x_590, 0);
lean_inc(x_591);
if (lean_is_exclusive(x_590)) {
 lean_ctor_release(x_590, 0);
 x_592 = x_590;
} else {
 lean_dec_ref(x_590);
 x_592 = lean_box(0);
}
x_593 = lean_ctor_get(x_591, 0);
lean_inc(x_593);
x_594 = lean_ctor_get(x_591, 1);
lean_inc(x_594);
if (lean_is_exclusive(x_591)) {
 lean_ctor_release(x_591, 0);
 lean_ctor_release(x_591, 1);
 x_595 = x_591;
} else {
 lean_dec_ref(x_591);
 x_595 = lean_box(0);
}
x_596 = lean_alloc_ctor(11, 1, 0);
lean_ctor_set(x_596, 0, x_593);
if (lean_is_scalar(x_595)) {
 x_597 = lean_alloc_ctor(0, 2, 0);
} else {
 x_597 = x_595;
}
lean_ctor_set(x_597, 0, x_596);
lean_ctor_set(x_597, 1, x_594);
if (lean_is_scalar(x_592)) {
 x_598 = lean_alloc_ctor(1, 1, 0);
} else {
 x_598 = x_592;
}
lean_ctor_set(x_598, 0, x_597);
return x_598;
}
}
}
else
{
lean_object* x_599; 
x_599 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readUInt16LE(x_550);
if (lean_obj_tag(x_599) == 0)
{
lean_object* x_600; 
x_600 = lean_box(0);
return x_600;
}
else
{
lean_object* x_601; lean_object* x_602; lean_object* x_603; lean_object* x_604; uint16_t x_605; lean_object* x_606; lean_object* x_607; 
x_601 = lean_ctor_get(x_599, 0);
lean_inc(x_601);
if (lean_is_exclusive(x_599)) {
 lean_ctor_release(x_599, 0);
 x_602 = x_599;
} else {
 lean_dec_ref(x_599);
 x_602 = lean_box(0);
}
x_603 = lean_ctor_get(x_601, 0);
lean_inc(x_603);
x_604 = lean_ctor_get(x_601, 1);
lean_inc(x_604);
lean_dec(x_601);
x_605 = lean_unbox(x_603);
lean_dec(x_603);
x_606 = lean_uint16_to_nat(x_605);
x_607 = lp_iris_x2dkernel_IrisKernel_FFI_decodeCostBound_decodeCostBoundList(x_604, x_606);
if (lean_obj_tag(x_607) == 0)
{
lean_object* x_608; 
lean_dec(x_602);
x_608 = lean_box(0);
return x_608;
}
else
{
lean_object* x_609; lean_object* x_610; lean_object* x_611; lean_object* x_612; lean_object* x_613; lean_object* x_614; lean_object* x_615; lean_object* x_616; 
x_609 = lean_ctor_get(x_607, 0);
lean_inc(x_609);
if (lean_is_exclusive(x_607)) {
 lean_ctor_release(x_607, 0);
 x_610 = x_607;
} else {
 lean_dec_ref(x_607);
 x_610 = lean_box(0);
}
x_611 = lean_ctor_get(x_609, 0);
lean_inc(x_611);
x_612 = lean_ctor_get(x_609, 1);
lean_inc(x_612);
if (lean_is_exclusive(x_609)) {
 lean_ctor_release(x_609, 0);
 lean_ctor_release(x_609, 1);
 x_613 = x_609;
} else {
 lean_dec_ref(x_609);
 x_613 = lean_box(0);
}
if (lean_is_scalar(x_602)) {
 x_614 = lean_alloc_ctor(10, 1, 0);
} else {
 x_614 = x_602;
 lean_ctor_set_tag(x_614, 10);
}
lean_ctor_set(x_614, 0, x_611);
if (lean_is_scalar(x_613)) {
 x_615 = lean_alloc_ctor(0, 2, 0);
} else {
 x_615 = x_613;
}
lean_ctor_set(x_615, 0, x_614);
lean_ctor_set(x_615, 1, x_612);
if (lean_is_scalar(x_610)) {
 x_616 = lean_alloc_ctor(1, 1, 0);
} else {
 x_616 = x_610;
}
lean_ctor_set(x_616, 0, x_615);
return x_616;
}
}
}
}
else
{
lean_object* x_617; 
x_617 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readUInt16LE(x_550);
if (lean_obj_tag(x_617) == 0)
{
lean_object* x_618; 
x_618 = lean_box(0);
return x_618;
}
else
{
lean_object* x_619; lean_object* x_620; lean_object* x_621; lean_object* x_622; uint16_t x_623; lean_object* x_624; lean_object* x_625; 
x_619 = lean_ctor_get(x_617, 0);
lean_inc(x_619);
if (lean_is_exclusive(x_617)) {
 lean_ctor_release(x_617, 0);
 x_620 = x_617;
} else {
 lean_dec_ref(x_617);
 x_620 = lean_box(0);
}
x_621 = lean_ctor_get(x_619, 0);
lean_inc(x_621);
x_622 = lean_ctor_get(x_619, 1);
lean_inc(x_622);
lean_dec(x_619);
x_623 = lean_unbox(x_621);
lean_dec(x_621);
x_624 = lean_uint16_to_nat(x_623);
x_625 = lp_iris_x2dkernel_IrisKernel_FFI_decodeCostBound_decodeCostBoundList(x_622, x_624);
if (lean_obj_tag(x_625) == 0)
{
lean_object* x_626; 
lean_dec(x_620);
x_626 = lean_box(0);
return x_626;
}
else
{
lean_object* x_627; lean_object* x_628; lean_object* x_629; lean_object* x_630; lean_object* x_631; lean_object* x_632; lean_object* x_633; lean_object* x_634; 
x_627 = lean_ctor_get(x_625, 0);
lean_inc(x_627);
if (lean_is_exclusive(x_625)) {
 lean_ctor_release(x_625, 0);
 x_628 = x_625;
} else {
 lean_dec_ref(x_625);
 x_628 = lean_box(0);
}
x_629 = lean_ctor_get(x_627, 0);
lean_inc(x_629);
x_630 = lean_ctor_get(x_627, 1);
lean_inc(x_630);
if (lean_is_exclusive(x_627)) {
 lean_ctor_release(x_627, 0);
 lean_ctor_release(x_627, 1);
 x_631 = x_627;
} else {
 lean_dec_ref(x_627);
 x_631 = lean_box(0);
}
if (lean_is_scalar(x_620)) {
 x_632 = lean_alloc_ctor(9, 1, 0);
} else {
 x_632 = x_620;
 lean_ctor_set_tag(x_632, 9);
}
lean_ctor_set(x_632, 0, x_629);
if (lean_is_scalar(x_631)) {
 x_633 = lean_alloc_ctor(0, 2, 0);
} else {
 x_633 = x_631;
}
lean_ctor_set(x_633, 0, x_632);
lean_ctor_set(x_633, 1, x_630);
if (lean_is_scalar(x_628)) {
 x_634 = lean_alloc_ctor(1, 1, 0);
} else {
 x_634 = x_628;
}
lean_ctor_set(x_634, 0, x_633);
return x_634;
}
}
}
}
else
{
lean_object* x_635; 
x_635 = lp_iris_x2dkernel_IrisKernel_FFI_decodeCostBound(x_550);
if (lean_obj_tag(x_635) == 0)
{
return x_635;
}
else
{
lean_object* x_636; lean_object* x_637; lean_object* x_638; lean_object* x_639; lean_object* x_640; 
x_636 = lean_ctor_get(x_635, 0);
lean_inc(x_636);
lean_dec_ref(x_635);
x_637 = lean_ctor_get(x_636, 0);
lean_inc(x_637);
x_638 = lean_ctor_get(x_636, 1);
lean_inc(x_638);
if (lean_is_exclusive(x_636)) {
 lean_ctor_release(x_636, 0);
 lean_ctor_release(x_636, 1);
 x_639 = x_636;
} else {
 lean_dec_ref(x_636);
 x_639 = lean_box(0);
}
x_640 = lp_iris_x2dkernel_IrisKernel_FFI_decodeCostBound(x_638);
if (lean_obj_tag(x_640) == 0)
{
lean_dec(x_639);
lean_dec(x_637);
return x_640;
}
else
{
lean_object* x_641; lean_object* x_642; lean_object* x_643; lean_object* x_644; lean_object* x_645; lean_object* x_646; lean_object* x_647; lean_object* x_648; 
x_641 = lean_ctor_get(x_640, 0);
lean_inc(x_641);
if (lean_is_exclusive(x_640)) {
 lean_ctor_release(x_640, 0);
 x_642 = x_640;
} else {
 lean_dec_ref(x_640);
 x_642 = lean_box(0);
}
x_643 = lean_ctor_get(x_641, 0);
lean_inc(x_643);
x_644 = lean_ctor_get(x_641, 1);
lean_inc(x_644);
if (lean_is_exclusive(x_641)) {
 lean_ctor_release(x_641, 0);
 lean_ctor_release(x_641, 1);
 x_645 = x_641;
} else {
 lean_dec_ref(x_641);
 x_645 = lean_box(0);
}
if (lean_is_scalar(x_639)) {
 x_646 = lean_alloc_ctor(8, 2, 0);
} else {
 x_646 = x_639;
 lean_ctor_set_tag(x_646, 8);
}
lean_ctor_set(x_646, 0, x_637);
lean_ctor_set(x_646, 1, x_643);
if (lean_is_scalar(x_645)) {
 x_647 = lean_alloc_ctor(0, 2, 0);
} else {
 x_647 = x_645;
}
lean_ctor_set(x_647, 0, x_646);
lean_ctor_set(x_647, 1, x_644);
if (lean_is_scalar(x_642)) {
 x_648 = lean_alloc_ctor(1, 1, 0);
} else {
 x_648 = x_642;
}
lean_ctor_set(x_648, 0, x_647);
return x_648;
}
}
}
}
else
{
lean_object* x_649; 
x_649 = lp_iris_x2dkernel_IrisKernel_FFI_decodeCostBound(x_550);
if (lean_obj_tag(x_649) == 0)
{
return x_649;
}
else
{
lean_object* x_650; lean_object* x_651; lean_object* x_652; lean_object* x_653; lean_object* x_654; 
x_650 = lean_ctor_get(x_649, 0);
lean_inc(x_650);
lean_dec_ref(x_649);
x_651 = lean_ctor_get(x_650, 0);
lean_inc(x_651);
x_652 = lean_ctor_get(x_650, 1);
lean_inc(x_652);
if (lean_is_exclusive(x_650)) {
 lean_ctor_release(x_650, 0);
 lean_ctor_release(x_650, 1);
 x_653 = x_650;
} else {
 lean_dec_ref(x_650);
 x_653 = lean_box(0);
}
x_654 = lp_iris_x2dkernel_IrisKernel_FFI_decodeCostBound(x_652);
if (lean_obj_tag(x_654) == 0)
{
lean_dec(x_653);
lean_dec(x_651);
return x_654;
}
else
{
lean_object* x_655; lean_object* x_656; lean_object* x_657; lean_object* x_658; lean_object* x_659; lean_object* x_660; lean_object* x_661; lean_object* x_662; 
x_655 = lean_ctor_get(x_654, 0);
lean_inc(x_655);
if (lean_is_exclusive(x_654)) {
 lean_ctor_release(x_654, 0);
 x_656 = x_654;
} else {
 lean_dec_ref(x_654);
 x_656 = lean_box(0);
}
x_657 = lean_ctor_get(x_655, 0);
lean_inc(x_657);
x_658 = lean_ctor_get(x_655, 1);
lean_inc(x_658);
if (lean_is_exclusive(x_655)) {
 lean_ctor_release(x_655, 0);
 lean_ctor_release(x_655, 1);
 x_659 = x_655;
} else {
 lean_dec_ref(x_655);
 x_659 = lean_box(0);
}
if (lean_is_scalar(x_653)) {
 x_660 = lean_alloc_ctor(7, 2, 0);
} else {
 x_660 = x_653;
 lean_ctor_set_tag(x_660, 7);
}
lean_ctor_set(x_660, 0, x_651);
lean_ctor_set(x_660, 1, x_657);
if (lean_is_scalar(x_659)) {
 x_661 = lean_alloc_ctor(0, 2, 0);
} else {
 x_661 = x_659;
}
lean_ctor_set(x_661, 0, x_660);
lean_ctor_set(x_661, 1, x_658);
if (lean_is_scalar(x_656)) {
 x_662 = lean_alloc_ctor(1, 1, 0);
} else {
 x_662 = x_656;
}
lean_ctor_set(x_662, 0, x_661);
return x_662;
}
}
}
}
else
{
lean_object* x_663; 
x_663 = lp_iris_x2dkernel_IrisKernel_FFI_decodeCostBound(x_550);
if (lean_obj_tag(x_663) == 0)
{
return x_663;
}
else
{
lean_object* x_664; lean_object* x_665; lean_object* x_666; lean_object* x_667; lean_object* x_668; 
x_664 = lean_ctor_get(x_663, 0);
lean_inc(x_664);
lean_dec_ref(x_663);
x_665 = lean_ctor_get(x_664, 0);
lean_inc(x_665);
x_666 = lean_ctor_get(x_664, 1);
lean_inc(x_666);
if (lean_is_exclusive(x_664)) {
 lean_ctor_release(x_664, 0);
 lean_ctor_release(x_664, 1);
 x_667 = x_664;
} else {
 lean_dec_ref(x_664);
 x_667 = lean_box(0);
}
x_668 = lp_iris_x2dkernel_IrisKernel_FFI_decodeCostBound(x_666);
if (lean_obj_tag(x_668) == 0)
{
lean_dec(x_667);
lean_dec(x_665);
return x_668;
}
else
{
lean_object* x_669; lean_object* x_670; lean_object* x_671; lean_object* x_672; lean_object* x_673; lean_object* x_674; lean_object* x_675; lean_object* x_676; 
x_669 = lean_ctor_get(x_668, 0);
lean_inc(x_669);
if (lean_is_exclusive(x_668)) {
 lean_ctor_release(x_668, 0);
 x_670 = x_668;
} else {
 lean_dec_ref(x_668);
 x_670 = lean_box(0);
}
x_671 = lean_ctor_get(x_669, 0);
lean_inc(x_671);
x_672 = lean_ctor_get(x_669, 1);
lean_inc(x_672);
if (lean_is_exclusive(x_669)) {
 lean_ctor_release(x_669, 0);
 lean_ctor_release(x_669, 1);
 x_673 = x_669;
} else {
 lean_dec_ref(x_669);
 x_673 = lean_box(0);
}
if (lean_is_scalar(x_667)) {
 x_674 = lean_alloc_ctor(6, 2, 0);
} else {
 x_674 = x_667;
 lean_ctor_set_tag(x_674, 6);
}
lean_ctor_set(x_674, 0, x_665);
lean_ctor_set(x_674, 1, x_671);
if (lean_is_scalar(x_673)) {
 x_675 = lean_alloc_ctor(0, 2, 0);
} else {
 x_675 = x_673;
}
lean_ctor_set(x_675, 0, x_674);
lean_ctor_set(x_675, 1, x_672);
if (lean_is_scalar(x_670)) {
 x_676 = lean_alloc_ctor(1, 1, 0);
} else {
 x_676 = x_670;
}
lean_ctor_set(x_676, 0, x_675);
return x_676;
}
}
}
}
else
{
lean_object* x_677; 
x_677 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readUInt32LE(x_550);
if (lean_obj_tag(x_677) == 0)
{
lean_object* x_678; 
x_678 = lean_box(0);
return x_678;
}
else
{
lean_object* x_679; lean_object* x_680; lean_object* x_681; lean_object* x_682; lean_object* x_683; 
x_679 = lean_ctor_get(x_677, 0);
lean_inc(x_679);
lean_dec_ref(x_677);
x_680 = lean_ctor_get(x_679, 0);
lean_inc(x_680);
x_681 = lean_ctor_get(x_679, 1);
lean_inc(x_681);
if (lean_is_exclusive(x_679)) {
 lean_ctor_release(x_679, 0);
 lean_ctor_release(x_679, 1);
 x_682 = x_679;
} else {
 lean_dec_ref(x_679);
 x_682 = lean_box(0);
}
x_683 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readUInt32LE(x_681);
if (lean_obj_tag(x_683) == 0)
{
lean_object* x_684; 
lean_dec(x_682);
lean_dec(x_680);
x_684 = lean_box(0);
return x_684;
}
else
{
lean_object* x_685; lean_object* x_686; lean_object* x_687; lean_object* x_688; lean_object* x_689; uint32_t x_690; lean_object* x_691; uint32_t x_692; lean_object* x_693; lean_object* x_694; lean_object* x_695; lean_object* x_696; 
x_685 = lean_ctor_get(x_683, 0);
lean_inc(x_685);
if (lean_is_exclusive(x_683)) {
 lean_ctor_release(x_683, 0);
 x_686 = x_683;
} else {
 lean_dec_ref(x_683);
 x_686 = lean_box(0);
}
x_687 = lean_ctor_get(x_685, 0);
lean_inc(x_687);
x_688 = lean_ctor_get(x_685, 1);
lean_inc(x_688);
if (lean_is_exclusive(x_685)) {
 lean_ctor_release(x_685, 0);
 lean_ctor_release(x_685, 1);
 x_689 = x_685;
} else {
 lean_dec_ref(x_685);
 x_689 = lean_box(0);
}
x_690 = lean_unbox_uint32(x_680);
lean_dec(x_680);
x_691 = lean_uint32_to_nat(x_690);
x_692 = lean_unbox_uint32(x_687);
lean_dec(x_687);
x_693 = lean_uint32_to_nat(x_692);
if (lean_is_scalar(x_682)) {
 x_694 = lean_alloc_ctor(5, 2, 0);
} else {
 x_694 = x_682;
 lean_ctor_set_tag(x_694, 5);
}
lean_ctor_set(x_694, 0, x_691);
lean_ctor_set(x_694, 1, x_693);
if (lean_is_scalar(x_689)) {
 x_695 = lean_alloc_ctor(0, 2, 0);
} else {
 x_695 = x_689;
}
lean_ctor_set(x_695, 0, x_694);
lean_ctor_set(x_695, 1, x_688);
if (lean_is_scalar(x_686)) {
 x_696 = lean_alloc_ctor(1, 1, 0);
} else {
 x_696 = x_686;
}
lean_ctor_set(x_696, 0, x_695);
return x_696;
}
}
}
}
else
{
lean_object* x_697; 
x_697 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readUInt32LE(x_550);
if (lean_obj_tag(x_697) == 0)
{
lean_object* x_698; 
x_698 = lean_box(0);
return x_698;
}
else
{
lean_object* x_699; lean_object* x_700; lean_object* x_701; lean_object* x_702; lean_object* x_703; uint32_t x_704; lean_object* x_705; lean_object* x_706; lean_object* x_707; lean_object* x_708; 
x_699 = lean_ctor_get(x_697, 0);
lean_inc(x_699);
if (lean_is_exclusive(x_697)) {
 lean_ctor_release(x_697, 0);
 x_700 = x_697;
} else {
 lean_dec_ref(x_697);
 x_700 = lean_box(0);
}
x_701 = lean_ctor_get(x_699, 0);
lean_inc(x_701);
x_702 = lean_ctor_get(x_699, 1);
lean_inc(x_702);
if (lean_is_exclusive(x_699)) {
 lean_ctor_release(x_699, 0);
 lean_ctor_release(x_699, 1);
 x_703 = x_699;
} else {
 lean_dec_ref(x_699);
 x_703 = lean_box(0);
}
x_704 = lean_unbox_uint32(x_701);
lean_dec(x_701);
x_705 = lean_uint32_to_nat(x_704);
x_706 = lean_alloc_ctor(4, 1, 0);
lean_ctor_set(x_706, 0, x_705);
if (lean_is_scalar(x_703)) {
 x_707 = lean_alloc_ctor(0, 2, 0);
} else {
 x_707 = x_703;
}
lean_ctor_set(x_707, 0, x_706);
lean_ctor_set(x_707, 1, x_702);
if (lean_is_scalar(x_700)) {
 x_708 = lean_alloc_ctor(1, 1, 0);
} else {
 x_708 = x_700;
}
lean_ctor_set(x_708, 0, x_707);
return x_708;
}
}
}
else
{
lean_object* x_709; 
x_709 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readUInt32LE(x_550);
if (lean_obj_tag(x_709) == 0)
{
lean_object* x_710; 
x_710 = lean_box(0);
return x_710;
}
else
{
lean_object* x_711; lean_object* x_712; lean_object* x_713; lean_object* x_714; lean_object* x_715; uint32_t x_716; lean_object* x_717; lean_object* x_718; lean_object* x_719; lean_object* x_720; 
x_711 = lean_ctor_get(x_709, 0);
lean_inc(x_711);
if (lean_is_exclusive(x_709)) {
 lean_ctor_release(x_709, 0);
 x_712 = x_709;
} else {
 lean_dec_ref(x_709);
 x_712 = lean_box(0);
}
x_713 = lean_ctor_get(x_711, 0);
lean_inc(x_713);
x_714 = lean_ctor_get(x_711, 1);
lean_inc(x_714);
if (lean_is_exclusive(x_711)) {
 lean_ctor_release(x_711, 0);
 lean_ctor_release(x_711, 1);
 x_715 = x_711;
} else {
 lean_dec_ref(x_711);
 x_715 = lean_box(0);
}
x_716 = lean_unbox_uint32(x_713);
lean_dec(x_713);
x_717 = lean_uint32_to_nat(x_716);
x_718 = lean_alloc_ctor(3, 1, 0);
lean_ctor_set(x_718, 0, x_717);
if (lean_is_scalar(x_715)) {
 x_719 = lean_alloc_ctor(0, 2, 0);
} else {
 x_719 = x_715;
}
lean_ctor_set(x_719, 0, x_718);
lean_ctor_set(x_719, 1, x_714);
if (lean_is_scalar(x_712)) {
 x_720 = lean_alloc_ctor(1, 1, 0);
} else {
 x_720 = x_712;
}
lean_ctor_set(x_720, 0, x_719);
return x_720;
}
}
}
else
{
lean_object* x_721; 
x_721 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readUInt64LE(x_550);
if (lean_obj_tag(x_721) == 0)
{
lean_object* x_722; 
x_722 = lean_box(0);
return x_722;
}
else
{
lean_object* x_723; lean_object* x_724; lean_object* x_725; lean_object* x_726; lean_object* x_727; uint64_t x_728; lean_object* x_729; lean_object* x_730; lean_object* x_731; lean_object* x_732; 
x_723 = lean_ctor_get(x_721, 0);
lean_inc(x_723);
if (lean_is_exclusive(x_721)) {
 lean_ctor_release(x_721, 0);
 x_724 = x_721;
} else {
 lean_dec_ref(x_721);
 x_724 = lean_box(0);
}
x_725 = lean_ctor_get(x_723, 0);
lean_inc(x_725);
x_726 = lean_ctor_get(x_723, 1);
lean_inc(x_726);
if (lean_is_exclusive(x_723)) {
 lean_ctor_release(x_723, 0);
 lean_ctor_release(x_723, 1);
 x_727 = x_723;
} else {
 lean_dec_ref(x_723);
 x_727 = lean_box(0);
}
x_728 = lean_unbox_uint64(x_725);
lean_dec(x_725);
x_729 = lean_uint64_to_nat(x_728);
x_730 = lean_alloc_ctor(2, 1, 0);
lean_ctor_set(x_730, 0, x_729);
if (lean_is_scalar(x_727)) {
 x_731 = lean_alloc_ctor(0, 2, 0);
} else {
 x_731 = x_727;
}
lean_ctor_set(x_731, 0, x_730);
lean_ctor_set(x_731, 1, x_726);
if (lean_is_scalar(x_724)) {
 x_732 = lean_alloc_ctor(1, 1, 0);
} else {
 x_732 = x_724;
}
lean_ctor_set(x_732, 0, x_731);
return x_732;
}
}
}
else
{
lean_object* x_733; lean_object* x_734; lean_object* x_735; 
x_733 = lean_box(1);
if (lean_is_scalar(x_551)) {
 x_734 = lean_alloc_ctor(0, 2, 0);
} else {
 x_734 = x_551;
}
lean_ctor_set(x_734, 0, x_733);
lean_ctor_set(x_734, 1, x_550);
x_735 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_735, 0, x_734);
return x_735;
}
}
else
{
lean_object* x_736; lean_object* x_737; lean_object* x_738; 
x_736 = lean_box(0);
if (lean_is_scalar(x_551)) {
 x_737 = lean_alloc_ctor(0, 2, 0);
} else {
 x_737 = x_551;
}
lean_ctor_set(x_737, 0, x_736);
lean_ctor_set(x_737, 1, x_550);
x_738 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_738, 0, x_737);
return x_738;
}
}
}
}
}
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_decodeCostBound_decodeCostBoundList(lean_object* x_1, lean_object* x_2) {
_start:
{
lean_object* x_3; uint8_t x_4; 
x_3 = lean_unsigned_to_nat(0u);
x_4 = lean_nat_dec_eq(x_2, x_3);
if (x_4 == 1)
{
lean_object* x_5; lean_object* x_6; lean_object* x_7; 
x_5 = lean_box(0);
x_6 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_6, 0, x_5);
lean_ctor_set(x_6, 1, x_1);
x_7 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_7, 0, x_6);
return x_7;
}
else
{
lean_object* x_8; 
x_8 = lp_iris_x2dkernel_IrisKernel_FFI_decodeCostBound(x_1);
if (lean_obj_tag(x_8) == 0)
{
lean_object* x_9; 
x_9 = lean_box(0);
return x_9;
}
else
{
lean_object* x_10; uint8_t x_11; 
x_10 = lean_ctor_get(x_8, 0);
lean_inc(x_10);
lean_dec_ref(x_8);
x_11 = !lean_is_exclusive(x_10);
if (x_11 == 0)
{
lean_object* x_12; lean_object* x_13; lean_object* x_14; lean_object* x_15; lean_object* x_16; 
x_12 = lean_ctor_get(x_10, 0);
x_13 = lean_ctor_get(x_10, 1);
x_14 = lean_unsigned_to_nat(1u);
x_15 = lean_nat_sub(x_2, x_14);
x_16 = lp_iris_x2dkernel_IrisKernel_FFI_decodeCostBound_decodeCostBoundList(x_13, x_15);
lean_dec(x_15);
if (lean_obj_tag(x_16) == 0)
{
lean_free_object(x_10);
lean_dec(x_12);
return x_16;
}
else
{
uint8_t x_17; 
x_17 = !lean_is_exclusive(x_16);
if (x_17 == 0)
{
lean_object* x_18; uint8_t x_19; 
x_18 = lean_ctor_get(x_16, 0);
x_19 = !lean_is_exclusive(x_18);
if (x_19 == 0)
{
lean_object* x_20; 
x_20 = lean_ctor_get(x_18, 0);
lean_ctor_set_tag(x_10, 1);
lean_ctor_set(x_10, 1, x_20);
lean_ctor_set(x_18, 0, x_10);
return x_16;
}
else
{
lean_object* x_21; lean_object* x_22; lean_object* x_23; 
x_21 = lean_ctor_get(x_18, 0);
x_22 = lean_ctor_get(x_18, 1);
lean_inc(x_22);
lean_inc(x_21);
lean_dec(x_18);
lean_ctor_set_tag(x_10, 1);
lean_ctor_set(x_10, 1, x_21);
x_23 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_23, 0, x_10);
lean_ctor_set(x_23, 1, x_22);
lean_ctor_set(x_16, 0, x_23);
return x_16;
}
}
else
{
lean_object* x_24; lean_object* x_25; lean_object* x_26; lean_object* x_27; lean_object* x_28; lean_object* x_29; 
x_24 = lean_ctor_get(x_16, 0);
lean_inc(x_24);
lean_dec(x_16);
x_25 = lean_ctor_get(x_24, 0);
lean_inc(x_25);
x_26 = lean_ctor_get(x_24, 1);
lean_inc(x_26);
if (lean_is_exclusive(x_24)) {
 lean_ctor_release(x_24, 0);
 lean_ctor_release(x_24, 1);
 x_27 = x_24;
} else {
 lean_dec_ref(x_24);
 x_27 = lean_box(0);
}
lean_ctor_set_tag(x_10, 1);
lean_ctor_set(x_10, 1, x_25);
if (lean_is_scalar(x_27)) {
 x_28 = lean_alloc_ctor(0, 2, 0);
} else {
 x_28 = x_27;
}
lean_ctor_set(x_28, 0, x_10);
lean_ctor_set(x_28, 1, x_26);
x_29 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_29, 0, x_28);
return x_29;
}
}
}
else
{
lean_object* x_30; lean_object* x_31; lean_object* x_32; lean_object* x_33; lean_object* x_34; 
x_30 = lean_ctor_get(x_10, 0);
x_31 = lean_ctor_get(x_10, 1);
lean_inc(x_31);
lean_inc(x_30);
lean_dec(x_10);
x_32 = lean_unsigned_to_nat(1u);
x_33 = lean_nat_sub(x_2, x_32);
x_34 = lp_iris_x2dkernel_IrisKernel_FFI_decodeCostBound_decodeCostBoundList(x_31, x_33);
lean_dec(x_33);
if (lean_obj_tag(x_34) == 0)
{
lean_dec(x_30);
return x_34;
}
else
{
lean_object* x_35; lean_object* x_36; lean_object* x_37; lean_object* x_38; lean_object* x_39; lean_object* x_40; lean_object* x_41; lean_object* x_42; 
x_35 = lean_ctor_get(x_34, 0);
lean_inc(x_35);
if (lean_is_exclusive(x_34)) {
 lean_ctor_release(x_34, 0);
 x_36 = x_34;
} else {
 lean_dec_ref(x_34);
 x_36 = lean_box(0);
}
x_37 = lean_ctor_get(x_35, 0);
lean_inc(x_37);
x_38 = lean_ctor_get(x_35, 1);
lean_inc(x_38);
if (lean_is_exclusive(x_35)) {
 lean_ctor_release(x_35, 0);
 lean_ctor_release(x_35, 1);
 x_39 = x_35;
} else {
 lean_dec_ref(x_35);
 x_39 = lean_box(0);
}
x_40 = lean_alloc_ctor(1, 2, 0);
lean_ctor_set(x_40, 0, x_30);
lean_ctor_set(x_40, 1, x_37);
if (lean_is_scalar(x_39)) {
 x_41 = lean_alloc_ctor(0, 2, 0);
} else {
 x_41 = x_39;
}
lean_ctor_set(x_41, 0, x_40);
lean_ctor_set(x_41, 1, x_38);
if (lean_is_scalar(x_36)) {
 x_42 = lean_alloc_ctor(1, 1, 0);
} else {
 x_42 = x_36;
}
lean_ctor_set(x_42, 0, x_41);
return x_42;
}
}
}
}
}
}
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_decodeCostBound_decodeCostBoundList___boxed(lean_object* x_1, lean_object* x_2) {
_start:
{
lean_object* x_3; 
x_3 = lp_iris_x2dkernel_IrisKernel_FFI_decodeCostBound_decodeCostBoundList(x_1, x_2);
lean_dec(x_2);
return x_3;
}
}
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeId(lean_object* x_1) {
_start:
{
lean_object* x_2; 
x_2 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readUInt64LE(x_1);
if (lean_obj_tag(x_2) == 0)
{
lean_object* x_3; 
x_3 = lean_box(0);
return x_3;
}
else
{
uint8_t x_4; 
x_4 = !lean_is_exclusive(x_2);
if (x_4 == 0)
{
lean_object* x_5; uint8_t x_6; 
x_5 = lean_ctor_get(x_2, 0);
x_6 = !lean_is_exclusive(x_5);
if (x_6 == 0)
{
lean_object* x_7; uint64_t x_8; lean_object* x_9; 
x_7 = lean_ctor_get(x_5, 0);
x_8 = lean_unbox_uint64(x_7);
lean_dec(x_7);
x_9 = lean_uint64_to_nat(x_8);
lean_ctor_set(x_5, 0, x_9);
return x_2;
}
else
{
lean_object* x_10; lean_object* x_11; uint64_t x_12; lean_object* x_13; lean_object* x_14; 
x_10 = lean_ctor_get(x_5, 0);
x_11 = lean_ctor_get(x_5, 1);
lean_inc(x_11);
lean_inc(x_10);
lean_dec(x_5);
x_12 = lean_unbox_uint64(x_10);
lean_dec(x_10);
x_13 = lean_uint64_to_nat(x_12);
x_14 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_14, 0, x_13);
lean_ctor_set(x_14, 1, x_11);
lean_ctor_set(x_2, 0, x_14);
return x_2;
}
}
else
{
lean_object* x_15; lean_object* x_16; lean_object* x_17; lean_object* x_18; uint64_t x_19; lean_object* x_20; lean_object* x_21; lean_object* x_22; 
x_15 = lean_ctor_get(x_2, 0);
lean_inc(x_15);
lean_dec(x_2);
x_16 = lean_ctor_get(x_15, 0);
lean_inc(x_16);
x_17 = lean_ctor_get(x_15, 1);
lean_inc(x_17);
if (lean_is_exclusive(x_15)) {
 lean_ctor_release(x_15, 0);
 lean_ctor_release(x_15, 1);
 x_18 = x_15;
} else {
 lean_dec_ref(x_15);
 x_18 = lean_box(0);
}
x_19 = lean_unbox_uint64(x_16);
lean_dec(x_16);
x_20 = lean_uint64_to_nat(x_19);
if (lean_is_scalar(x_18)) {
 x_21 = lean_alloc_ctor(0, 2, 0);
} else {
 x_21 = x_18;
}
lean_ctor_set(x_21, 0, x_20);
lean_ctor_set(x_21, 1, x_17);
x_22 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_22, 0, x_21);
return x_22;
}
}
}
}
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_decodeTypeId(lean_object* x_1) {
_start:
{
lean_object* x_2; 
x_2 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readUInt64LE(x_1);
if (lean_obj_tag(x_2) == 0)
{
lean_object* x_3; 
x_3 = lean_box(0);
return x_3;
}
else
{
uint8_t x_4; 
x_4 = !lean_is_exclusive(x_2);
if (x_4 == 0)
{
lean_object* x_5; uint8_t x_6; 
x_5 = lean_ctor_get(x_2, 0);
x_6 = !lean_is_exclusive(x_5);
if (x_6 == 0)
{
lean_object* x_7; uint64_t x_8; lean_object* x_9; 
x_7 = lean_ctor_get(x_5, 0);
x_8 = lean_unbox_uint64(x_7);
lean_dec(x_7);
x_9 = lean_uint64_to_nat(x_8);
lean_ctor_set(x_5, 0, x_9);
return x_2;
}
else
{
lean_object* x_10; lean_object* x_11; uint64_t x_12; lean_object* x_13; lean_object* x_14; 
x_10 = lean_ctor_get(x_5, 0);
x_11 = lean_ctor_get(x_5, 1);
lean_inc(x_11);
lean_inc(x_10);
lean_dec(x_5);
x_12 = lean_unbox_uint64(x_10);
lean_dec(x_10);
x_13 = lean_uint64_to_nat(x_12);
x_14 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_14, 0, x_13);
lean_ctor_set(x_14, 1, x_11);
lean_ctor_set(x_2, 0, x_14);
return x_2;
}
}
else
{
lean_object* x_15; lean_object* x_16; lean_object* x_17; lean_object* x_18; uint64_t x_19; lean_object* x_20; lean_object* x_21; lean_object* x_22; 
x_15 = lean_ctor_get(x_2, 0);
lean_inc(x_15);
lean_dec(x_2);
x_16 = lean_ctor_get(x_15, 0);
lean_inc(x_16);
x_17 = lean_ctor_get(x_15, 1);
lean_inc(x_17);
if (lean_is_exclusive(x_15)) {
 lean_ctor_release(x_15, 0);
 lean_ctor_release(x_15, 1);
 x_18 = x_15;
} else {
 lean_dec_ref(x_15);
 x_18 = lean_box(0);
}
x_19 = lean_unbox_uint64(x_16);
lean_dec(x_16);
x_20 = lean_uint64_to_nat(x_19);
if (lean_is_scalar(x_18)) {
 x_21 = lean_alloc_ctor(0, 2, 0);
} else {
 x_21 = x_18;
}
lean_ctor_set(x_21, 0, x_20);
lean_ctor_set(x_21, 1, x_17);
x_22 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_22, 0, x_21);
return x_22;
}
}
}
}
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_decodeBinderId(lean_object* x_1) {
_start:
{
lean_object* x_2; 
x_2 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readUInt32LE(x_1);
if (lean_obj_tag(x_2) == 0)
{
lean_object* x_3; 
x_3 = lean_box(0);
return x_3;
}
else
{
uint8_t x_4; 
x_4 = !lean_is_exclusive(x_2);
if (x_4 == 0)
{
lean_object* x_5; uint8_t x_6; 
x_5 = lean_ctor_get(x_2, 0);
x_6 = !lean_is_exclusive(x_5);
if (x_6 == 0)
{
lean_object* x_7; uint32_t x_8; lean_object* x_9; 
x_7 = lean_ctor_get(x_5, 0);
x_8 = lean_unbox_uint32(x_7);
lean_dec(x_7);
x_9 = lean_uint32_to_nat(x_8);
lean_ctor_set(x_5, 0, x_9);
return x_2;
}
else
{
lean_object* x_10; lean_object* x_11; uint32_t x_12; lean_object* x_13; lean_object* x_14; 
x_10 = lean_ctor_get(x_5, 0);
x_11 = lean_ctor_get(x_5, 1);
lean_inc(x_11);
lean_inc(x_10);
lean_dec(x_5);
x_12 = lean_unbox_uint32(x_10);
lean_dec(x_10);
x_13 = lean_uint32_to_nat(x_12);
x_14 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_14, 0, x_13);
lean_ctor_set(x_14, 1, x_11);
lean_ctor_set(x_2, 0, x_14);
return x_2;
}
}
else
{
lean_object* x_15; lean_object* x_16; lean_object* x_17; lean_object* x_18; uint32_t x_19; lean_object* x_20; lean_object* x_21; lean_object* x_22; 
x_15 = lean_ctor_get(x_2, 0);
lean_inc(x_15);
lean_dec(x_2);
x_16 = lean_ctor_get(x_15, 0);
lean_inc(x_16);
x_17 = lean_ctor_get(x_15, 1);
lean_inc(x_17);
if (lean_is_exclusive(x_15)) {
 lean_ctor_release(x_15, 0);
 lean_ctor_release(x_15, 1);
 x_18 = x_15;
} else {
 lean_dec_ref(x_15);
 x_18 = lean_box(0);
}
x_19 = lean_unbox_uint32(x_16);
lean_dec(x_16);
x_20 = lean_uint32_to_nat(x_19);
if (lean_is_scalar(x_18)) {
 x_21 = lean_alloc_ctor(0, 2, 0);
} else {
 x_21 = x_18;
}
lean_ctor_set(x_21, 0, x_20);
lean_ctor_set(x_21, 1, x_17);
x_22 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_22, 0, x_21);
return x_22;
}
}
}
}
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_decodeCostVar(lean_object* x_1) {
_start:
{
lean_object* x_2; 
x_2 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readUInt32LE(x_1);
if (lean_obj_tag(x_2) == 0)
{
lean_object* x_3; 
x_3 = lean_box(0);
return x_3;
}
else
{
uint8_t x_4; 
x_4 = !lean_is_exclusive(x_2);
if (x_4 == 0)
{
lean_object* x_5; uint8_t x_6; 
x_5 = lean_ctor_get(x_2, 0);
x_6 = !lean_is_exclusive(x_5);
if (x_6 == 0)
{
lean_object* x_7; uint32_t x_8; lean_object* x_9; 
x_7 = lean_ctor_get(x_5, 0);
x_8 = lean_unbox_uint32(x_7);
lean_dec(x_7);
x_9 = lean_uint32_to_nat(x_8);
lean_ctor_set(x_5, 0, x_9);
return x_2;
}
else
{
lean_object* x_10; lean_object* x_11; uint32_t x_12; lean_object* x_13; lean_object* x_14; 
x_10 = lean_ctor_get(x_5, 0);
x_11 = lean_ctor_get(x_5, 1);
lean_inc(x_11);
lean_inc(x_10);
lean_dec(x_5);
x_12 = lean_unbox_uint32(x_10);
lean_dec(x_10);
x_13 = lean_uint32_to_nat(x_12);
x_14 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_14, 0, x_13);
lean_ctor_set(x_14, 1, x_11);
lean_ctor_set(x_2, 0, x_14);
return x_2;
}
}
else
{
lean_object* x_15; lean_object* x_16; lean_object* x_17; lean_object* x_18; uint32_t x_19; lean_object* x_20; lean_object* x_21; lean_object* x_22; 
x_15 = lean_ctor_get(x_2, 0);
lean_inc(x_15);
lean_dec(x_2);
x_16 = lean_ctor_get(x_15, 0);
lean_inc(x_16);
x_17 = lean_ctor_get(x_15, 1);
lean_inc(x_17);
if (lean_is_exclusive(x_15)) {
 lean_ctor_release(x_15, 0);
 lean_ctor_release(x_15, 1);
 x_18 = x_15;
} else {
 lean_dec_ref(x_15);
 x_18 = lean_box(0);
}
x_19 = lean_unbox_uint32(x_16);
lean_dec(x_16);
x_20 = lean_uint32_to_nat(x_19);
if (lean_is_scalar(x_18)) {
 x_21 = lean_alloc_ctor(0, 2, 0);
} else {
 x_21 = x_18;
}
lean_ctor_set(x_21, 0, x_20);
lean_ctor_set(x_21, 1, x_17);
x_22 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_22, 0, x_21);
return x_22;
}
}
}
}
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_decodeContext_decodeBindings(lean_object* x_1, lean_object* x_2, lean_object* x_3) {
_start:
{
lean_object* x_4; uint8_t x_5; 
x_4 = lean_unsigned_to_nat(0u);
x_5 = lean_nat_dec_eq(x_2, x_4);
if (x_5 == 1)
{
lean_object* x_6; lean_object* x_7; lean_object* x_8; 
lean_dec(x_2);
x_6 = l_List_reverse___redArg(x_3);
x_7 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_7, 0, x_6);
lean_ctor_set(x_7, 1, x_1);
x_8 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_8, 0, x_7);
return x_8;
}
else
{
lean_object* x_9; 
x_9 = lp_iris_x2dkernel_IrisKernel_FFI_decodeBinderId(x_1);
if (lean_obj_tag(x_9) == 0)
{
lean_object* x_10; 
lean_dec(x_3);
lean_dec(x_2);
x_10 = lean_box(0);
return x_10;
}
else
{
lean_object* x_11; uint8_t x_12; 
x_11 = lean_ctor_get(x_9, 0);
lean_inc(x_11);
lean_dec_ref(x_9);
x_12 = !lean_is_exclusive(x_11);
if (x_12 == 0)
{
lean_object* x_13; lean_object* x_14; lean_object* x_15; 
x_13 = lean_ctor_get(x_11, 0);
x_14 = lean_ctor_get(x_11, 1);
x_15 = lp_iris_x2dkernel_IrisKernel_FFI_decodeTypeId(x_14);
if (lean_obj_tag(x_15) == 0)
{
lean_object* x_16; 
lean_free_object(x_11);
lean_dec(x_13);
lean_dec(x_3);
lean_dec(x_2);
x_16 = lean_box(0);
return x_16;
}
else
{
lean_object* x_17; uint8_t x_18; 
x_17 = lean_ctor_get(x_15, 0);
lean_inc(x_17);
lean_dec_ref(x_15);
x_18 = !lean_is_exclusive(x_17);
if (x_18 == 0)
{
lean_object* x_19; lean_object* x_20; lean_object* x_21; lean_object* x_22; 
x_19 = lean_ctor_get(x_17, 0);
x_20 = lean_ctor_get(x_17, 1);
x_21 = lean_unsigned_to_nat(1u);
x_22 = lean_nat_sub(x_2, x_21);
lean_dec(x_2);
lean_ctor_set(x_17, 1, x_19);
lean_ctor_set(x_17, 0, x_13);
lean_ctor_set_tag(x_11, 1);
lean_ctor_set(x_11, 1, x_3);
lean_ctor_set(x_11, 0, x_17);
x_1 = x_20;
x_2 = x_22;
x_3 = x_11;
goto _start;
}
else
{
lean_object* x_24; lean_object* x_25; lean_object* x_26; lean_object* x_27; lean_object* x_28; 
x_24 = lean_ctor_get(x_17, 0);
x_25 = lean_ctor_get(x_17, 1);
lean_inc(x_25);
lean_inc(x_24);
lean_dec(x_17);
x_26 = lean_unsigned_to_nat(1u);
x_27 = lean_nat_sub(x_2, x_26);
lean_dec(x_2);
x_28 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_28, 0, x_13);
lean_ctor_set(x_28, 1, x_24);
lean_ctor_set_tag(x_11, 1);
lean_ctor_set(x_11, 1, x_3);
lean_ctor_set(x_11, 0, x_28);
x_1 = x_25;
x_2 = x_27;
x_3 = x_11;
goto _start;
}
}
}
else
{
lean_object* x_30; lean_object* x_31; lean_object* x_32; 
x_30 = lean_ctor_get(x_11, 0);
x_31 = lean_ctor_get(x_11, 1);
lean_inc(x_31);
lean_inc(x_30);
lean_dec(x_11);
x_32 = lp_iris_x2dkernel_IrisKernel_FFI_decodeTypeId(x_31);
if (lean_obj_tag(x_32) == 0)
{
lean_object* x_33; 
lean_dec(x_30);
lean_dec(x_3);
lean_dec(x_2);
x_33 = lean_box(0);
return x_33;
}
else
{
lean_object* x_34; lean_object* x_35; lean_object* x_36; lean_object* x_37; lean_object* x_38; lean_object* x_39; lean_object* x_40; lean_object* x_41; 
x_34 = lean_ctor_get(x_32, 0);
lean_inc(x_34);
lean_dec_ref(x_32);
x_35 = lean_ctor_get(x_34, 0);
lean_inc(x_35);
x_36 = lean_ctor_get(x_34, 1);
lean_inc(x_36);
if (lean_is_exclusive(x_34)) {
 lean_ctor_release(x_34, 0);
 lean_ctor_release(x_34, 1);
 x_37 = x_34;
} else {
 lean_dec_ref(x_34);
 x_37 = lean_box(0);
}
x_38 = lean_unsigned_to_nat(1u);
x_39 = lean_nat_sub(x_2, x_38);
lean_dec(x_2);
if (lean_is_scalar(x_37)) {
 x_40 = lean_alloc_ctor(0, 2, 0);
} else {
 x_40 = x_37;
}
lean_ctor_set(x_40, 0, x_30);
lean_ctor_set(x_40, 1, x_35);
x_41 = lean_alloc_ctor(1, 2, 0);
lean_ctor_set(x_41, 0, x_40);
lean_ctor_set(x_41, 1, x_3);
x_1 = x_36;
x_2 = x_39;
x_3 = x_41;
goto _start;
}
}
}
}
}
}
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_decodeContext(lean_object* x_1) {
_start:
{
lean_object* x_2; 
x_2 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readUInt16LE(x_1);
if (lean_obj_tag(x_2) == 0)
{
lean_object* x_3; 
x_3 = lean_box(0);
return x_3;
}
else
{
lean_object* x_4; lean_object* x_5; lean_object* x_6; uint16_t x_7; lean_object* x_8; lean_object* x_9; lean_object* x_10; 
x_4 = lean_ctor_get(x_2, 0);
lean_inc(x_4);
lean_dec_ref(x_2);
x_5 = lean_ctor_get(x_4, 0);
lean_inc(x_5);
x_6 = lean_ctor_get(x_4, 1);
lean_inc(x_6);
lean_dec(x_4);
x_7 = lean_unbox(x_5);
lean_dec(x_5);
x_8 = lean_uint16_to_nat(x_7);
x_9 = lean_box(0);
x_10 = lp_iris_x2dkernel_IrisKernel_FFI_decodeContext_decodeBindings(x_6, x_8, x_9);
return x_10;
}
}
}
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_decodePrimType(lean_object* x_1) {
_start:
{
lean_object* x_2; 
x_2 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readUInt8(x_1);
if (lean_obj_tag(x_2) == 0)
{
lean_object* x_3; 
x_3 = lean_box(0);
return x_3;
}
else
{
uint8_t x_4; 
x_4 = !lean_is_exclusive(x_2);
if (x_4 == 0)
{
lean_object* x_5; uint8_t x_6; 
x_5 = lean_ctor_get(x_2, 0);
x_6 = !lean_is_exclusive(x_5);
if (x_6 == 0)
{
lean_object* x_7; lean_object* x_8; uint8_t x_9; lean_object* x_10; lean_object* x_11; uint8_t x_12; 
x_7 = lean_ctor_get(x_5, 0);
x_8 = lean_ctor_get(x_5, 1);
x_9 = lean_unbox(x_7);
lean_dec(x_7);
x_10 = lean_uint8_to_nat(x_9);
x_11 = lean_unsigned_to_nat(0u);
x_12 = lean_nat_dec_eq(x_10, x_11);
if (x_12 == 0)
{
lean_object* x_13; uint8_t x_14; 
x_13 = lean_unsigned_to_nat(1u);
x_14 = lean_nat_dec_eq(x_10, x_13);
if (x_14 == 0)
{
lean_object* x_15; uint8_t x_16; 
x_15 = lean_unsigned_to_nat(2u);
x_16 = lean_nat_dec_eq(x_10, x_15);
if (x_16 == 0)
{
lean_object* x_17; uint8_t x_18; 
x_17 = lean_unsigned_to_nat(3u);
x_18 = lean_nat_dec_eq(x_10, x_17);
if (x_18 == 0)
{
lean_object* x_19; uint8_t x_20; 
x_19 = lean_unsigned_to_nat(4u);
x_20 = lean_nat_dec_eq(x_10, x_19);
if (x_20 == 0)
{
lean_object* x_21; uint8_t x_22; 
x_21 = lean_unsigned_to_nat(5u);
x_22 = lean_nat_dec_eq(x_10, x_21);
if (x_22 == 0)
{
lean_object* x_23; uint8_t x_24; 
x_23 = lean_unsigned_to_nat(6u);
x_24 = lean_nat_dec_eq(x_10, x_23);
if (x_24 == 0)
{
lean_object* x_25; 
lean_free_object(x_5);
lean_dec(x_8);
lean_free_object(x_2);
x_25 = lean_box(0);
return x_25;
}
else
{
uint8_t x_26; lean_object* x_27; 
x_26 = 6;
x_27 = lean_box(x_26);
lean_ctor_set(x_5, 0, x_27);
return x_2;
}
}
else
{
uint8_t x_28; lean_object* x_29; 
x_28 = 5;
x_29 = lean_box(x_28);
lean_ctor_set(x_5, 0, x_29);
return x_2;
}
}
else
{
uint8_t x_30; lean_object* x_31; 
x_30 = 4;
x_31 = lean_box(x_30);
lean_ctor_set(x_5, 0, x_31);
return x_2;
}
}
else
{
uint8_t x_32; lean_object* x_33; 
x_32 = 3;
x_33 = lean_box(x_32);
lean_ctor_set(x_5, 0, x_33);
return x_2;
}
}
else
{
uint8_t x_34; lean_object* x_35; 
x_34 = 2;
x_35 = lean_box(x_34);
lean_ctor_set(x_5, 0, x_35);
return x_2;
}
}
else
{
uint8_t x_36; lean_object* x_37; 
x_36 = 1;
x_37 = lean_box(x_36);
lean_ctor_set(x_5, 0, x_37);
return x_2;
}
}
else
{
uint8_t x_38; lean_object* x_39; 
x_38 = 0;
x_39 = lean_box(x_38);
lean_ctor_set(x_5, 0, x_39);
return x_2;
}
}
else
{
lean_object* x_40; lean_object* x_41; uint8_t x_42; lean_object* x_43; lean_object* x_44; uint8_t x_45; 
x_40 = lean_ctor_get(x_5, 0);
x_41 = lean_ctor_get(x_5, 1);
lean_inc(x_41);
lean_inc(x_40);
lean_dec(x_5);
x_42 = lean_unbox(x_40);
lean_dec(x_40);
x_43 = lean_uint8_to_nat(x_42);
x_44 = lean_unsigned_to_nat(0u);
x_45 = lean_nat_dec_eq(x_43, x_44);
if (x_45 == 0)
{
lean_object* x_46; uint8_t x_47; 
x_46 = lean_unsigned_to_nat(1u);
x_47 = lean_nat_dec_eq(x_43, x_46);
if (x_47 == 0)
{
lean_object* x_48; uint8_t x_49; 
x_48 = lean_unsigned_to_nat(2u);
x_49 = lean_nat_dec_eq(x_43, x_48);
if (x_49 == 0)
{
lean_object* x_50; uint8_t x_51; 
x_50 = lean_unsigned_to_nat(3u);
x_51 = lean_nat_dec_eq(x_43, x_50);
if (x_51 == 0)
{
lean_object* x_52; uint8_t x_53; 
x_52 = lean_unsigned_to_nat(4u);
x_53 = lean_nat_dec_eq(x_43, x_52);
if (x_53 == 0)
{
lean_object* x_54; uint8_t x_55; 
x_54 = lean_unsigned_to_nat(5u);
x_55 = lean_nat_dec_eq(x_43, x_54);
if (x_55 == 0)
{
lean_object* x_56; uint8_t x_57; 
x_56 = lean_unsigned_to_nat(6u);
x_57 = lean_nat_dec_eq(x_43, x_56);
if (x_57 == 0)
{
lean_object* x_58; 
lean_dec(x_41);
lean_free_object(x_2);
x_58 = lean_box(0);
return x_58;
}
else
{
uint8_t x_59; lean_object* x_60; lean_object* x_61; 
x_59 = 6;
x_60 = lean_box(x_59);
x_61 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_61, 0, x_60);
lean_ctor_set(x_61, 1, x_41);
lean_ctor_set(x_2, 0, x_61);
return x_2;
}
}
else
{
uint8_t x_62; lean_object* x_63; lean_object* x_64; 
x_62 = 5;
x_63 = lean_box(x_62);
x_64 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_64, 0, x_63);
lean_ctor_set(x_64, 1, x_41);
lean_ctor_set(x_2, 0, x_64);
return x_2;
}
}
else
{
uint8_t x_65; lean_object* x_66; lean_object* x_67; 
x_65 = 4;
x_66 = lean_box(x_65);
x_67 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_67, 0, x_66);
lean_ctor_set(x_67, 1, x_41);
lean_ctor_set(x_2, 0, x_67);
return x_2;
}
}
else
{
uint8_t x_68; lean_object* x_69; lean_object* x_70; 
x_68 = 3;
x_69 = lean_box(x_68);
x_70 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_70, 0, x_69);
lean_ctor_set(x_70, 1, x_41);
lean_ctor_set(x_2, 0, x_70);
return x_2;
}
}
else
{
uint8_t x_71; lean_object* x_72; lean_object* x_73; 
x_71 = 2;
x_72 = lean_box(x_71);
x_73 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_73, 0, x_72);
lean_ctor_set(x_73, 1, x_41);
lean_ctor_set(x_2, 0, x_73);
return x_2;
}
}
else
{
uint8_t x_74; lean_object* x_75; lean_object* x_76; 
x_74 = 1;
x_75 = lean_box(x_74);
x_76 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_76, 0, x_75);
lean_ctor_set(x_76, 1, x_41);
lean_ctor_set(x_2, 0, x_76);
return x_2;
}
}
else
{
uint8_t x_77; lean_object* x_78; lean_object* x_79; 
x_77 = 0;
x_78 = lean_box(x_77);
x_79 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_79, 0, x_78);
lean_ctor_set(x_79, 1, x_41);
lean_ctor_set(x_2, 0, x_79);
return x_2;
}
}
}
else
{
lean_object* x_80; lean_object* x_81; lean_object* x_82; lean_object* x_83; uint8_t x_84; lean_object* x_85; lean_object* x_86; uint8_t x_87; 
x_80 = lean_ctor_get(x_2, 0);
lean_inc(x_80);
lean_dec(x_2);
x_81 = lean_ctor_get(x_80, 0);
lean_inc(x_81);
x_82 = lean_ctor_get(x_80, 1);
lean_inc(x_82);
if (lean_is_exclusive(x_80)) {
 lean_ctor_release(x_80, 0);
 lean_ctor_release(x_80, 1);
 x_83 = x_80;
} else {
 lean_dec_ref(x_80);
 x_83 = lean_box(0);
}
x_84 = lean_unbox(x_81);
lean_dec(x_81);
x_85 = lean_uint8_to_nat(x_84);
x_86 = lean_unsigned_to_nat(0u);
x_87 = lean_nat_dec_eq(x_85, x_86);
if (x_87 == 0)
{
lean_object* x_88; uint8_t x_89; 
x_88 = lean_unsigned_to_nat(1u);
x_89 = lean_nat_dec_eq(x_85, x_88);
if (x_89 == 0)
{
lean_object* x_90; uint8_t x_91; 
x_90 = lean_unsigned_to_nat(2u);
x_91 = lean_nat_dec_eq(x_85, x_90);
if (x_91 == 0)
{
lean_object* x_92; uint8_t x_93; 
x_92 = lean_unsigned_to_nat(3u);
x_93 = lean_nat_dec_eq(x_85, x_92);
if (x_93 == 0)
{
lean_object* x_94; uint8_t x_95; 
x_94 = lean_unsigned_to_nat(4u);
x_95 = lean_nat_dec_eq(x_85, x_94);
if (x_95 == 0)
{
lean_object* x_96; uint8_t x_97; 
x_96 = lean_unsigned_to_nat(5u);
x_97 = lean_nat_dec_eq(x_85, x_96);
if (x_97 == 0)
{
lean_object* x_98; uint8_t x_99; 
x_98 = lean_unsigned_to_nat(6u);
x_99 = lean_nat_dec_eq(x_85, x_98);
if (x_99 == 0)
{
lean_object* x_100; 
lean_dec(x_83);
lean_dec(x_82);
x_100 = lean_box(0);
return x_100;
}
else
{
uint8_t x_101; lean_object* x_102; lean_object* x_103; lean_object* x_104; 
x_101 = 6;
x_102 = lean_box(x_101);
if (lean_is_scalar(x_83)) {
 x_103 = lean_alloc_ctor(0, 2, 0);
} else {
 x_103 = x_83;
}
lean_ctor_set(x_103, 0, x_102);
lean_ctor_set(x_103, 1, x_82);
x_104 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_104, 0, x_103);
return x_104;
}
}
else
{
uint8_t x_105; lean_object* x_106; lean_object* x_107; lean_object* x_108; 
x_105 = 5;
x_106 = lean_box(x_105);
if (lean_is_scalar(x_83)) {
 x_107 = lean_alloc_ctor(0, 2, 0);
} else {
 x_107 = x_83;
}
lean_ctor_set(x_107, 0, x_106);
lean_ctor_set(x_107, 1, x_82);
x_108 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_108, 0, x_107);
return x_108;
}
}
else
{
uint8_t x_109; lean_object* x_110; lean_object* x_111; lean_object* x_112; 
x_109 = 4;
x_110 = lean_box(x_109);
if (lean_is_scalar(x_83)) {
 x_111 = lean_alloc_ctor(0, 2, 0);
} else {
 x_111 = x_83;
}
lean_ctor_set(x_111, 0, x_110);
lean_ctor_set(x_111, 1, x_82);
x_112 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_112, 0, x_111);
return x_112;
}
}
else
{
uint8_t x_113; lean_object* x_114; lean_object* x_115; lean_object* x_116; 
x_113 = 3;
x_114 = lean_box(x_113);
if (lean_is_scalar(x_83)) {
 x_115 = lean_alloc_ctor(0, 2, 0);
} else {
 x_115 = x_83;
}
lean_ctor_set(x_115, 0, x_114);
lean_ctor_set(x_115, 1, x_82);
x_116 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_116, 0, x_115);
return x_116;
}
}
else
{
uint8_t x_117; lean_object* x_118; lean_object* x_119; lean_object* x_120; 
x_117 = 2;
x_118 = lean_box(x_117);
if (lean_is_scalar(x_83)) {
 x_119 = lean_alloc_ctor(0, 2, 0);
} else {
 x_119 = x_83;
}
lean_ctor_set(x_119, 0, x_118);
lean_ctor_set(x_119, 1, x_82);
x_120 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_120, 0, x_119);
return x_120;
}
}
else
{
uint8_t x_121; lean_object* x_122; lean_object* x_123; lean_object* x_124; 
x_121 = 1;
x_122 = lean_box(x_121);
if (lean_is_scalar(x_83)) {
 x_123 = lean_alloc_ctor(0, 2, 0);
} else {
 x_123 = x_83;
}
lean_ctor_set(x_123, 0, x_122);
lean_ctor_set(x_123, 1, x_82);
x_124 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_124, 0, x_123);
return x_124;
}
}
else
{
uint8_t x_125; lean_object* x_126; lean_object* x_127; lean_object* x_128; 
x_125 = 0;
x_126 = lean_box(x_125);
if (lean_is_scalar(x_83)) {
 x_127 = lean_alloc_ctor(0, 2, 0);
} else {
 x_127 = x_83;
}
lean_ctor_set(x_127, 0, x_126);
lean_ctor_set(x_127, 1, x_82);
x_128 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_128, 0, x_127);
return x_128;
}
}
}
}
}
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_decodeTag(lean_object* x_1) {
_start:
{
lean_object* x_2; 
x_2 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readUInt16LE(x_1);
if (lean_obj_tag(x_2) == 0)
{
lean_object* x_3; 
x_3 = lean_box(0);
return x_3;
}
else
{
uint8_t x_4; 
x_4 = !lean_is_exclusive(x_2);
if (x_4 == 0)
{
lean_object* x_5; uint8_t x_6; 
x_5 = lean_ctor_get(x_2, 0);
x_6 = !lean_is_exclusive(x_5);
if (x_6 == 0)
{
lean_object* x_7; uint16_t x_8; lean_object* x_9; 
x_7 = lean_ctor_get(x_5, 0);
x_8 = lean_unbox(x_7);
lean_dec(x_7);
x_9 = lean_uint16_to_nat(x_8);
lean_ctor_set(x_5, 0, x_9);
return x_2;
}
else
{
lean_object* x_10; lean_object* x_11; uint16_t x_12; lean_object* x_13; lean_object* x_14; 
x_10 = lean_ctor_get(x_5, 0);
x_11 = lean_ctor_get(x_5, 1);
lean_inc(x_11);
lean_inc(x_10);
lean_dec(x_5);
x_12 = lean_unbox(x_10);
lean_dec(x_10);
x_13 = lean_uint16_to_nat(x_12);
x_14 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_14, 0, x_13);
lean_ctor_set(x_14, 1, x_11);
lean_ctor_set(x_2, 0, x_14);
return x_2;
}
}
else
{
lean_object* x_15; lean_object* x_16; lean_object* x_17; lean_object* x_18; uint16_t x_19; lean_object* x_20; lean_object* x_21; lean_object* x_22; 
x_15 = lean_ctor_get(x_2, 0);
lean_inc(x_15);
lean_dec(x_2);
x_16 = lean_ctor_get(x_15, 0);
lean_inc(x_16);
x_17 = lean_ctor_get(x_15, 1);
lean_inc(x_17);
if (lean_is_exclusive(x_15)) {
 lean_ctor_release(x_15, 0);
 lean_ctor_release(x_15, 1);
 x_18 = x_15;
} else {
 lean_dec_ref(x_15);
 x_18 = lean_box(0);
}
x_19 = lean_unbox(x_16);
lean_dec(x_16);
x_20 = lean_uint16_to_nat(x_19);
if (lean_is_scalar(x_18)) {
 x_21 = lean_alloc_ctor(0, 2, 0);
} else {
 x_21 = x_18;
}
lean_ctor_set(x_21, 0, x_20);
lean_ctor_set(x_21, 1, x_17);
x_22 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_22, 0, x_21);
return x_22;
}
}
}
}
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_decodeBoundVar(lean_object* x_1) {
_start:
{
lean_object* x_2; 
x_2 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readUInt32LE(x_1);
if (lean_obj_tag(x_2) == 0)
{
lean_object* x_3; 
x_3 = lean_box(0);
return x_3;
}
else
{
uint8_t x_4; 
x_4 = !lean_is_exclusive(x_2);
if (x_4 == 0)
{
lean_object* x_5; uint8_t x_6; 
x_5 = lean_ctor_get(x_2, 0);
x_6 = !lean_is_exclusive(x_5);
if (x_6 == 0)
{
lean_object* x_7; uint32_t x_8; lean_object* x_9; 
x_7 = lean_ctor_get(x_5, 0);
x_8 = lean_unbox_uint32(x_7);
lean_dec(x_7);
x_9 = lean_uint32_to_nat(x_8);
lean_ctor_set(x_5, 0, x_9);
return x_2;
}
else
{
lean_object* x_10; lean_object* x_11; uint32_t x_12; lean_object* x_13; lean_object* x_14; 
x_10 = lean_ctor_get(x_5, 0);
x_11 = lean_ctor_get(x_5, 1);
lean_inc(x_11);
lean_inc(x_10);
lean_dec(x_5);
x_12 = lean_unbox_uint32(x_10);
lean_dec(x_10);
x_13 = lean_uint32_to_nat(x_12);
x_14 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_14, 0, x_13);
lean_ctor_set(x_14, 1, x_11);
lean_ctor_set(x_2, 0, x_14);
return x_2;
}
}
else
{
lean_object* x_15; lean_object* x_16; lean_object* x_17; lean_object* x_18; uint32_t x_19; lean_object* x_20; lean_object* x_21; lean_object* x_22; 
x_15 = lean_ctor_get(x_2, 0);
lean_inc(x_15);
lean_dec(x_2);
x_16 = lean_ctor_get(x_15, 0);
lean_inc(x_16);
x_17 = lean_ctor_get(x_15, 1);
lean_inc(x_17);
if (lean_is_exclusive(x_15)) {
 lean_ctor_release(x_15, 0);
 lean_ctor_release(x_15, 1);
 x_18 = x_15;
} else {
 lean_dec_ref(x_15);
 x_18 = lean_box(0);
}
x_19 = lean_unbox_uint32(x_16);
lean_dec(x_16);
x_20 = lean_uint32_to_nat(x_19);
if (lean_is_scalar(x_18)) {
 x_21 = lean_alloc_ctor(0, 2, 0);
} else {
 x_21 = x_18;
}
lean_ctor_set(x_21, 0, x_20);
lean_ctor_set(x_21, 1, x_17);
x_22 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_22, 0, x_21);
return x_22;
}
}
}
}
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_decodeTypeDef_decodeTypeIdList(lean_object* x_1, lean_object* x_2, lean_object* x_3) {
_start:
{
lean_object* x_4; uint8_t x_5; 
x_4 = lean_unsigned_to_nat(0u);
x_5 = lean_nat_dec_eq(x_2, x_4);
if (x_5 == 1)
{
lean_object* x_6; lean_object* x_7; lean_object* x_8; 
lean_dec(x_2);
x_6 = l_List_reverse___redArg(x_3);
x_7 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_7, 0, x_6);
lean_ctor_set(x_7, 1, x_1);
x_8 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_8, 0, x_7);
return x_8;
}
else
{
lean_object* x_9; 
x_9 = lp_iris_x2dkernel_IrisKernel_FFI_decodeTypeId(x_1);
if (lean_obj_tag(x_9) == 0)
{
lean_object* x_10; 
lean_dec(x_3);
lean_dec(x_2);
x_10 = lean_box(0);
return x_10;
}
else
{
lean_object* x_11; uint8_t x_12; 
x_11 = lean_ctor_get(x_9, 0);
lean_inc(x_11);
lean_dec_ref(x_9);
x_12 = !lean_is_exclusive(x_11);
if (x_12 == 0)
{
lean_object* x_13; lean_object* x_14; lean_object* x_15; 
x_13 = lean_ctor_get(x_11, 1);
x_14 = lean_unsigned_to_nat(1u);
x_15 = lean_nat_sub(x_2, x_14);
lean_dec(x_2);
lean_ctor_set_tag(x_11, 1);
lean_ctor_set(x_11, 1, x_3);
x_1 = x_13;
x_2 = x_15;
x_3 = x_11;
goto _start;
}
else
{
lean_object* x_17; lean_object* x_18; lean_object* x_19; lean_object* x_20; lean_object* x_21; 
x_17 = lean_ctor_get(x_11, 0);
x_18 = lean_ctor_get(x_11, 1);
lean_inc(x_18);
lean_inc(x_17);
lean_dec(x_11);
x_19 = lean_unsigned_to_nat(1u);
x_20 = lean_nat_sub(x_2, x_19);
lean_dec(x_2);
x_21 = lean_alloc_ctor(1, 2, 0);
lean_ctor_set(x_21, 0, x_17);
lean_ctor_set(x_21, 1, x_3);
x_1 = x_18;
x_2 = x_20;
x_3 = x_21;
goto _start;
}
}
}
}
}
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_decodeTypeDef_decodeTagTypeIdList(lean_object* x_1, lean_object* x_2, lean_object* x_3) {
_start:
{
lean_object* x_4; uint8_t x_5; 
x_4 = lean_unsigned_to_nat(0u);
x_5 = lean_nat_dec_eq(x_2, x_4);
if (x_5 == 1)
{
lean_object* x_6; lean_object* x_7; lean_object* x_8; 
lean_dec(x_2);
x_6 = l_List_reverse___redArg(x_3);
x_7 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_7, 0, x_6);
lean_ctor_set(x_7, 1, x_1);
x_8 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_8, 0, x_7);
return x_8;
}
else
{
lean_object* x_9; 
x_9 = lp_iris_x2dkernel_IrisKernel_FFI_decodeTag(x_1);
if (lean_obj_tag(x_9) == 0)
{
lean_object* x_10; 
lean_dec(x_3);
lean_dec(x_2);
x_10 = lean_box(0);
return x_10;
}
else
{
lean_object* x_11; uint8_t x_12; 
x_11 = lean_ctor_get(x_9, 0);
lean_inc(x_11);
lean_dec_ref(x_9);
x_12 = !lean_is_exclusive(x_11);
if (x_12 == 0)
{
lean_object* x_13; lean_object* x_14; lean_object* x_15; 
x_13 = lean_ctor_get(x_11, 0);
x_14 = lean_ctor_get(x_11, 1);
x_15 = lp_iris_x2dkernel_IrisKernel_FFI_decodeTypeId(x_14);
if (lean_obj_tag(x_15) == 0)
{
lean_object* x_16; 
lean_free_object(x_11);
lean_dec(x_13);
lean_dec(x_3);
lean_dec(x_2);
x_16 = lean_box(0);
return x_16;
}
else
{
lean_object* x_17; uint8_t x_18; 
x_17 = lean_ctor_get(x_15, 0);
lean_inc(x_17);
lean_dec_ref(x_15);
x_18 = !lean_is_exclusive(x_17);
if (x_18 == 0)
{
lean_object* x_19; lean_object* x_20; lean_object* x_21; lean_object* x_22; 
x_19 = lean_ctor_get(x_17, 0);
x_20 = lean_ctor_get(x_17, 1);
x_21 = lean_unsigned_to_nat(1u);
x_22 = lean_nat_sub(x_2, x_21);
lean_dec(x_2);
lean_ctor_set(x_17, 1, x_19);
lean_ctor_set(x_17, 0, x_13);
lean_ctor_set_tag(x_11, 1);
lean_ctor_set(x_11, 1, x_3);
lean_ctor_set(x_11, 0, x_17);
x_1 = x_20;
x_2 = x_22;
x_3 = x_11;
goto _start;
}
else
{
lean_object* x_24; lean_object* x_25; lean_object* x_26; lean_object* x_27; lean_object* x_28; 
x_24 = lean_ctor_get(x_17, 0);
x_25 = lean_ctor_get(x_17, 1);
lean_inc(x_25);
lean_inc(x_24);
lean_dec(x_17);
x_26 = lean_unsigned_to_nat(1u);
x_27 = lean_nat_sub(x_2, x_26);
lean_dec(x_2);
x_28 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_28, 0, x_13);
lean_ctor_set(x_28, 1, x_24);
lean_ctor_set_tag(x_11, 1);
lean_ctor_set(x_11, 1, x_3);
lean_ctor_set(x_11, 0, x_28);
x_1 = x_25;
x_2 = x_27;
x_3 = x_11;
goto _start;
}
}
}
else
{
lean_object* x_30; lean_object* x_31; lean_object* x_32; 
x_30 = lean_ctor_get(x_11, 0);
x_31 = lean_ctor_get(x_11, 1);
lean_inc(x_31);
lean_inc(x_30);
lean_dec(x_11);
x_32 = lp_iris_x2dkernel_IrisKernel_FFI_decodeTypeId(x_31);
if (lean_obj_tag(x_32) == 0)
{
lean_object* x_33; 
lean_dec(x_30);
lean_dec(x_3);
lean_dec(x_2);
x_33 = lean_box(0);
return x_33;
}
else
{
lean_object* x_34; lean_object* x_35; lean_object* x_36; lean_object* x_37; lean_object* x_38; lean_object* x_39; lean_object* x_40; lean_object* x_41; 
x_34 = lean_ctor_get(x_32, 0);
lean_inc(x_34);
lean_dec_ref(x_32);
x_35 = lean_ctor_get(x_34, 0);
lean_inc(x_35);
x_36 = lean_ctor_get(x_34, 1);
lean_inc(x_36);
if (lean_is_exclusive(x_34)) {
 lean_ctor_release(x_34, 0);
 lean_ctor_release(x_34, 1);
 x_37 = x_34;
} else {
 lean_dec_ref(x_34);
 x_37 = lean_box(0);
}
x_38 = lean_unsigned_to_nat(1u);
x_39 = lean_nat_sub(x_2, x_38);
lean_dec(x_2);
if (lean_is_scalar(x_37)) {
 x_40 = lean_alloc_ctor(0, 2, 0);
} else {
 x_40 = x_37;
}
lean_ctor_set(x_40, 0, x_30);
lean_ctor_set(x_40, 1, x_35);
x_41 = lean_alloc_ctor(1, 2, 0);
lean_ctor_set(x_41, 0, x_40);
lean_ctor_set(x_41, 1, x_3);
x_1 = x_36;
x_2 = x_39;
x_3 = x_41;
goto _start;
}
}
}
}
}
}
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_decodeTypeDef(lean_object* x_1) {
_start:
{
lean_object* x_2; 
x_2 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readUInt8(x_1);
if (lean_obj_tag(x_2) == 0)
{
lean_object* x_3; 
x_3 = lean_box(0);
return x_3;
}
else
{
uint8_t x_4; 
x_4 = !lean_is_exclusive(x_2);
if (x_4 == 0)
{
lean_object* x_5; lean_object* x_6; lean_object* x_7; uint8_t x_8; lean_object* x_9; lean_object* x_10; uint8_t x_11; 
x_5 = lean_ctor_get(x_2, 0);
x_6 = lean_ctor_get(x_5, 0);
lean_inc(x_6);
x_7 = lean_ctor_get(x_5, 1);
lean_inc(x_7);
lean_dec(x_5);
x_8 = lean_unbox(x_6);
lean_dec(x_6);
x_9 = lean_uint8_to_nat(x_8);
x_10 = lean_unsigned_to_nat(0u);
x_11 = lean_nat_dec_eq(x_9, x_10);
if (x_11 == 0)
{
lean_object* x_12; uint8_t x_13; 
x_12 = lean_unsigned_to_nat(1u);
x_13 = lean_nat_dec_eq(x_9, x_12);
if (x_13 == 0)
{
lean_object* x_14; uint8_t x_15; 
x_14 = lean_unsigned_to_nat(2u);
x_15 = lean_nat_dec_eq(x_9, x_14);
if (x_15 == 0)
{
lean_object* x_16; uint8_t x_17; 
x_16 = lean_unsigned_to_nat(3u);
x_17 = lean_nat_dec_eq(x_9, x_16);
if (x_17 == 0)
{
lean_object* x_18; uint8_t x_19; 
x_18 = lean_unsigned_to_nat(4u);
x_19 = lean_nat_dec_eq(x_9, x_18);
if (x_19 == 0)
{
lean_object* x_20; uint8_t x_21; 
x_20 = lean_unsigned_to_nat(5u);
x_21 = lean_nat_dec_eq(x_9, x_20);
if (x_21 == 0)
{
lean_object* x_22; uint8_t x_23; 
x_22 = lean_unsigned_to_nat(6u);
x_23 = lean_nat_dec_eq(x_9, x_22);
if (x_23 == 0)
{
lean_object* x_24; uint8_t x_25; 
x_24 = lean_unsigned_to_nat(7u);
x_25 = lean_nat_dec_eq(x_9, x_24);
if (x_25 == 0)
{
lean_object* x_26; uint8_t x_27; 
x_26 = lean_unsigned_to_nat(8u);
x_27 = lean_nat_dec_eq(x_9, x_26);
if (x_27 == 0)
{
lean_object* x_28; uint8_t x_29; 
x_28 = lean_unsigned_to_nat(9u);
x_29 = lean_nat_dec_eq(x_9, x_28);
if (x_29 == 0)
{
lean_object* x_30; uint8_t x_31; 
x_30 = lean_unsigned_to_nat(10u);
x_31 = lean_nat_dec_eq(x_9, x_30);
if (x_31 == 0)
{
lean_object* x_32; 
lean_dec(x_7);
lean_free_object(x_2);
x_32 = lean_box(0);
return x_32;
}
else
{
lean_object* x_33; 
x_33 = lp_iris_x2dkernel_IrisKernel_FFI_decodeTypeId(x_7);
if (lean_obj_tag(x_33) == 0)
{
lean_object* x_34; 
lean_free_object(x_2);
x_34 = lean_box(0);
return x_34;
}
else
{
uint8_t x_35; 
x_35 = !lean_is_exclusive(x_33);
if (x_35 == 0)
{
lean_object* x_36; uint8_t x_37; 
x_36 = lean_ctor_get(x_33, 0);
x_37 = !lean_is_exclusive(x_36);
if (x_37 == 0)
{
lean_object* x_38; 
x_38 = lean_ctor_get(x_36, 0);
lean_ctor_set_tag(x_2, 10);
lean_ctor_set(x_2, 0, x_38);
lean_ctor_set(x_36, 0, x_2);
return x_33;
}
else
{
lean_object* x_39; lean_object* x_40; lean_object* x_41; 
x_39 = lean_ctor_get(x_36, 0);
x_40 = lean_ctor_get(x_36, 1);
lean_inc(x_40);
lean_inc(x_39);
lean_dec(x_36);
lean_ctor_set_tag(x_2, 10);
lean_ctor_set(x_2, 0, x_39);
x_41 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_41, 0, x_2);
lean_ctor_set(x_41, 1, x_40);
lean_ctor_set(x_33, 0, x_41);
return x_33;
}
}
else
{
lean_object* x_42; lean_object* x_43; lean_object* x_44; lean_object* x_45; lean_object* x_46; lean_object* x_47; 
x_42 = lean_ctor_get(x_33, 0);
lean_inc(x_42);
lean_dec(x_33);
x_43 = lean_ctor_get(x_42, 0);
lean_inc(x_43);
x_44 = lean_ctor_get(x_42, 1);
lean_inc(x_44);
if (lean_is_exclusive(x_42)) {
 lean_ctor_release(x_42, 0);
 lean_ctor_release(x_42, 1);
 x_45 = x_42;
} else {
 lean_dec_ref(x_42);
 x_45 = lean_box(0);
}
lean_ctor_set_tag(x_2, 10);
lean_ctor_set(x_2, 0, x_43);
if (lean_is_scalar(x_45)) {
 x_46 = lean_alloc_ctor(0, 2, 0);
} else {
 x_46 = x_45;
}
lean_ctor_set(x_46, 0, x_2);
lean_ctor_set(x_46, 1, x_44);
x_47 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_47, 0, x_46);
return x_47;
}
}
}
}
else
{
lean_object* x_48; 
lean_free_object(x_2);
x_48 = lp_iris_x2dkernel_IrisKernel_FFI_decodeTypeId(x_7);
if (lean_obj_tag(x_48) == 0)
{
lean_object* x_49; 
x_49 = lean_box(0);
return x_49;
}
else
{
lean_object* x_50; uint8_t x_51; 
x_50 = lean_ctor_get(x_48, 0);
lean_inc(x_50);
lean_dec_ref(x_48);
x_51 = !lean_is_exclusive(x_50);
if (x_51 == 0)
{
lean_object* x_52; lean_object* x_53; lean_object* x_54; 
x_52 = lean_ctor_get(x_50, 0);
x_53 = lean_ctor_get(x_50, 1);
x_54 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readUInt64LE(x_53);
if (lean_obj_tag(x_54) == 0)
{
lean_object* x_55; 
lean_free_object(x_50);
lean_dec(x_52);
x_55 = lean_box(0);
return x_55;
}
else
{
uint8_t x_56; 
x_56 = !lean_is_exclusive(x_54);
if (x_56 == 0)
{
lean_object* x_57; uint8_t x_58; 
x_57 = lean_ctor_get(x_54, 0);
x_58 = !lean_is_exclusive(x_57);
if (x_58 == 0)
{
lean_object* x_59; uint64_t x_60; lean_object* x_61; 
x_59 = lean_ctor_get(x_57, 0);
x_60 = lean_unbox_uint64(x_59);
lean_dec(x_59);
x_61 = lean_uint64_to_nat(x_60);
lean_ctor_set_tag(x_50, 9);
lean_ctor_set(x_50, 1, x_61);
lean_ctor_set(x_57, 0, x_50);
return x_54;
}
else
{
lean_object* x_62; lean_object* x_63; uint64_t x_64; lean_object* x_65; lean_object* x_66; 
x_62 = lean_ctor_get(x_57, 0);
x_63 = lean_ctor_get(x_57, 1);
lean_inc(x_63);
lean_inc(x_62);
lean_dec(x_57);
x_64 = lean_unbox_uint64(x_62);
lean_dec(x_62);
x_65 = lean_uint64_to_nat(x_64);
lean_ctor_set_tag(x_50, 9);
lean_ctor_set(x_50, 1, x_65);
x_66 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_66, 0, x_50);
lean_ctor_set(x_66, 1, x_63);
lean_ctor_set(x_54, 0, x_66);
return x_54;
}
}
else
{
lean_object* x_67; lean_object* x_68; lean_object* x_69; lean_object* x_70; uint64_t x_71; lean_object* x_72; lean_object* x_73; lean_object* x_74; 
x_67 = lean_ctor_get(x_54, 0);
lean_inc(x_67);
lean_dec(x_54);
x_68 = lean_ctor_get(x_67, 0);
lean_inc(x_68);
x_69 = lean_ctor_get(x_67, 1);
lean_inc(x_69);
if (lean_is_exclusive(x_67)) {
 lean_ctor_release(x_67, 0);
 lean_ctor_release(x_67, 1);
 x_70 = x_67;
} else {
 lean_dec_ref(x_67);
 x_70 = lean_box(0);
}
x_71 = lean_unbox_uint64(x_68);
lean_dec(x_68);
x_72 = lean_uint64_to_nat(x_71);
lean_ctor_set_tag(x_50, 9);
lean_ctor_set(x_50, 1, x_72);
if (lean_is_scalar(x_70)) {
 x_73 = lean_alloc_ctor(0, 2, 0);
} else {
 x_73 = x_70;
}
lean_ctor_set(x_73, 0, x_50);
lean_ctor_set(x_73, 1, x_69);
x_74 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_74, 0, x_73);
return x_74;
}
}
}
else
{
lean_object* x_75; lean_object* x_76; lean_object* x_77; 
x_75 = lean_ctor_get(x_50, 0);
x_76 = lean_ctor_get(x_50, 1);
lean_inc(x_76);
lean_inc(x_75);
lean_dec(x_50);
x_77 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readUInt64LE(x_76);
if (lean_obj_tag(x_77) == 0)
{
lean_object* x_78; 
lean_dec(x_75);
x_78 = lean_box(0);
return x_78;
}
else
{
lean_object* x_79; lean_object* x_80; lean_object* x_81; lean_object* x_82; lean_object* x_83; uint64_t x_84; lean_object* x_85; lean_object* x_86; lean_object* x_87; lean_object* x_88; 
x_79 = lean_ctor_get(x_77, 0);
lean_inc(x_79);
if (lean_is_exclusive(x_77)) {
 lean_ctor_release(x_77, 0);
 x_80 = x_77;
} else {
 lean_dec_ref(x_77);
 x_80 = lean_box(0);
}
x_81 = lean_ctor_get(x_79, 0);
lean_inc(x_81);
x_82 = lean_ctor_get(x_79, 1);
lean_inc(x_82);
if (lean_is_exclusive(x_79)) {
 lean_ctor_release(x_79, 0);
 lean_ctor_release(x_79, 1);
 x_83 = x_79;
} else {
 lean_dec_ref(x_79);
 x_83 = lean_box(0);
}
x_84 = lean_unbox_uint64(x_81);
lean_dec(x_81);
x_85 = lean_uint64_to_nat(x_84);
x_86 = lean_alloc_ctor(9, 2, 0);
lean_ctor_set(x_86, 0, x_75);
lean_ctor_set(x_86, 1, x_85);
if (lean_is_scalar(x_83)) {
 x_87 = lean_alloc_ctor(0, 2, 0);
} else {
 x_87 = x_83;
}
lean_ctor_set(x_87, 0, x_86);
lean_ctor_set(x_87, 1, x_82);
if (lean_is_scalar(x_80)) {
 x_88 = lean_alloc_ctor(1, 1, 0);
} else {
 x_88 = x_80;
}
lean_ctor_set(x_88, 0, x_87);
return x_88;
}
}
}
}
}
else
{
lean_object* x_89; 
lean_free_object(x_2);
x_89 = lp_iris_x2dkernel_IrisKernel_FFI_decodeBoundVar(x_7);
if (lean_obj_tag(x_89) == 0)
{
lean_object* x_90; 
x_90 = lean_box(0);
return x_90;
}
else
{
lean_object* x_91; uint8_t x_92; 
x_91 = lean_ctor_get(x_89, 0);
lean_inc(x_91);
lean_dec_ref(x_89);
x_92 = !lean_is_exclusive(x_91);
if (x_92 == 0)
{
lean_object* x_93; lean_object* x_94; lean_object* x_95; 
x_93 = lean_ctor_get(x_91, 0);
x_94 = lean_ctor_get(x_91, 1);
x_95 = lp_iris_x2dkernel_IrisKernel_FFI_decodeTypeId(x_94);
if (lean_obj_tag(x_95) == 0)
{
lean_object* x_96; 
lean_free_object(x_91);
lean_dec(x_93);
x_96 = lean_box(0);
return x_96;
}
else
{
uint8_t x_97; 
x_97 = !lean_is_exclusive(x_95);
if (x_97 == 0)
{
lean_object* x_98; uint8_t x_99; 
x_98 = lean_ctor_get(x_95, 0);
x_99 = !lean_is_exclusive(x_98);
if (x_99 == 0)
{
lean_object* x_100; 
x_100 = lean_ctor_get(x_98, 0);
lean_ctor_set_tag(x_91, 8);
lean_ctor_set(x_91, 1, x_100);
lean_ctor_set(x_98, 0, x_91);
return x_95;
}
else
{
lean_object* x_101; lean_object* x_102; lean_object* x_103; 
x_101 = lean_ctor_get(x_98, 0);
x_102 = lean_ctor_get(x_98, 1);
lean_inc(x_102);
lean_inc(x_101);
lean_dec(x_98);
lean_ctor_set_tag(x_91, 8);
lean_ctor_set(x_91, 1, x_101);
x_103 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_103, 0, x_91);
lean_ctor_set(x_103, 1, x_102);
lean_ctor_set(x_95, 0, x_103);
return x_95;
}
}
else
{
lean_object* x_104; lean_object* x_105; lean_object* x_106; lean_object* x_107; lean_object* x_108; lean_object* x_109; 
x_104 = lean_ctor_get(x_95, 0);
lean_inc(x_104);
lean_dec(x_95);
x_105 = lean_ctor_get(x_104, 0);
lean_inc(x_105);
x_106 = lean_ctor_get(x_104, 1);
lean_inc(x_106);
if (lean_is_exclusive(x_104)) {
 lean_ctor_release(x_104, 0);
 lean_ctor_release(x_104, 1);
 x_107 = x_104;
} else {
 lean_dec_ref(x_104);
 x_107 = lean_box(0);
}
lean_ctor_set_tag(x_91, 8);
lean_ctor_set(x_91, 1, x_105);
if (lean_is_scalar(x_107)) {
 x_108 = lean_alloc_ctor(0, 2, 0);
} else {
 x_108 = x_107;
}
lean_ctor_set(x_108, 0, x_91);
lean_ctor_set(x_108, 1, x_106);
x_109 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_109, 0, x_108);
return x_109;
}
}
}
else
{
lean_object* x_110; lean_object* x_111; lean_object* x_112; 
x_110 = lean_ctor_get(x_91, 0);
x_111 = lean_ctor_get(x_91, 1);
lean_inc(x_111);
lean_inc(x_110);
lean_dec(x_91);
x_112 = lp_iris_x2dkernel_IrisKernel_FFI_decodeTypeId(x_111);
if (lean_obj_tag(x_112) == 0)
{
lean_object* x_113; 
lean_dec(x_110);
x_113 = lean_box(0);
return x_113;
}
else
{
lean_object* x_114; lean_object* x_115; lean_object* x_116; lean_object* x_117; lean_object* x_118; lean_object* x_119; lean_object* x_120; lean_object* x_121; 
x_114 = lean_ctor_get(x_112, 0);
lean_inc(x_114);
if (lean_is_exclusive(x_112)) {
 lean_ctor_release(x_112, 0);
 x_115 = x_112;
} else {
 lean_dec_ref(x_112);
 x_115 = lean_box(0);
}
x_116 = lean_ctor_get(x_114, 0);
lean_inc(x_116);
x_117 = lean_ctor_get(x_114, 1);
lean_inc(x_117);
if (lean_is_exclusive(x_114)) {
 lean_ctor_release(x_114, 0);
 lean_ctor_release(x_114, 1);
 x_118 = x_114;
} else {
 lean_dec_ref(x_114);
 x_118 = lean_box(0);
}
x_119 = lean_alloc_ctor(8, 2, 0);
lean_ctor_set(x_119, 0, x_110);
lean_ctor_set(x_119, 1, x_116);
if (lean_is_scalar(x_118)) {
 x_120 = lean_alloc_ctor(0, 2, 0);
} else {
 x_120 = x_118;
}
lean_ctor_set(x_120, 0, x_119);
lean_ctor_set(x_120, 1, x_117);
if (lean_is_scalar(x_115)) {
 x_121 = lean_alloc_ctor(1, 1, 0);
} else {
 x_121 = x_115;
}
lean_ctor_set(x_121, 0, x_120);
return x_121;
}
}
}
}
}
else
{
lean_object* x_122; 
lean_free_object(x_2);
x_122 = lp_iris_x2dkernel_IrisKernel_FFI_decodeTypeId(x_7);
if (lean_obj_tag(x_122) == 0)
{
lean_object* x_123; 
x_123 = lean_box(0);
return x_123;
}
else
{
lean_object* x_124; lean_object* x_125; lean_object* x_126; lean_object* x_127; 
x_124 = lean_ctor_get(x_122, 0);
lean_inc(x_124);
lean_dec_ref(x_122);
x_125 = lean_ctor_get(x_124, 0);
lean_inc(x_125);
x_126 = lean_ctor_get(x_124, 1);
lean_inc(x_126);
lean_dec(x_124);
x_127 = lp_iris_x2dkernel_IrisKernel_FFI_decodeTypeId(x_126);
if (lean_obj_tag(x_127) == 0)
{
lean_object* x_128; 
lean_dec(x_125);
x_128 = lean_box(0);
return x_128;
}
else
{
lean_object* x_129; lean_object* x_130; lean_object* x_131; lean_object* x_132; 
x_129 = lean_ctor_get(x_127, 0);
lean_inc(x_129);
lean_dec_ref(x_127);
x_130 = lean_ctor_get(x_129, 0);
lean_inc(x_130);
x_131 = lean_ctor_get(x_129, 1);
lean_inc(x_131);
lean_dec(x_129);
x_132 = lp_iris_x2dkernel_IrisKernel_FFI_decodeCostBound(x_131);
if (lean_obj_tag(x_132) == 0)
{
lean_object* x_133; 
lean_dec(x_130);
lean_dec(x_125);
x_133 = lean_box(0);
return x_133;
}
else
{
uint8_t x_134; 
x_134 = !lean_is_exclusive(x_132);
if (x_134 == 0)
{
lean_object* x_135; uint8_t x_136; 
x_135 = lean_ctor_get(x_132, 0);
x_136 = !lean_is_exclusive(x_135);
if (x_136 == 0)
{
lean_object* x_137; lean_object* x_138; 
x_137 = lean_ctor_get(x_135, 0);
x_138 = lean_alloc_ctor(7, 3, 0);
lean_ctor_set(x_138, 0, x_125);
lean_ctor_set(x_138, 1, x_130);
lean_ctor_set(x_138, 2, x_137);
lean_ctor_set(x_135, 0, x_138);
return x_132;
}
else
{
lean_object* x_139; lean_object* x_140; lean_object* x_141; lean_object* x_142; 
x_139 = lean_ctor_get(x_135, 0);
x_140 = lean_ctor_get(x_135, 1);
lean_inc(x_140);
lean_inc(x_139);
lean_dec(x_135);
x_141 = lean_alloc_ctor(7, 3, 0);
lean_ctor_set(x_141, 0, x_125);
lean_ctor_set(x_141, 1, x_130);
lean_ctor_set(x_141, 2, x_139);
x_142 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_142, 0, x_141);
lean_ctor_set(x_142, 1, x_140);
lean_ctor_set(x_132, 0, x_142);
return x_132;
}
}
else
{
lean_object* x_143; lean_object* x_144; lean_object* x_145; lean_object* x_146; lean_object* x_147; lean_object* x_148; lean_object* x_149; 
x_143 = lean_ctor_get(x_132, 0);
lean_inc(x_143);
lean_dec(x_132);
x_144 = lean_ctor_get(x_143, 0);
lean_inc(x_144);
x_145 = lean_ctor_get(x_143, 1);
lean_inc(x_145);
if (lean_is_exclusive(x_143)) {
 lean_ctor_release(x_143, 0);
 lean_ctor_release(x_143, 1);
 x_146 = x_143;
} else {
 lean_dec_ref(x_143);
 x_146 = lean_box(0);
}
x_147 = lean_alloc_ctor(7, 3, 0);
lean_ctor_set(x_147, 0, x_125);
lean_ctor_set(x_147, 1, x_130);
lean_ctor_set(x_147, 2, x_144);
if (lean_is_scalar(x_146)) {
 x_148 = lean_alloc_ctor(0, 2, 0);
} else {
 x_148 = x_146;
}
lean_ctor_set(x_148, 0, x_147);
lean_ctor_set(x_148, 1, x_145);
x_149 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_149, 0, x_148);
return x_149;
}
}
}
}
}
}
else
{
lean_object* x_150; 
x_150 = lp_iris_x2dkernel_IrisKernel_FFI_decodeTypeId(x_7);
if (lean_obj_tag(x_150) == 0)
{
lean_object* x_151; 
lean_free_object(x_2);
x_151 = lean_box(0);
return x_151;
}
else
{
uint8_t x_152; 
x_152 = !lean_is_exclusive(x_150);
if (x_152 == 0)
{
lean_object* x_153; uint8_t x_154; 
x_153 = lean_ctor_get(x_150, 0);
x_154 = !lean_is_exclusive(x_153);
if (x_154 == 0)
{
lean_object* x_155; 
x_155 = lean_ctor_get(x_153, 0);
lean_ctor_set_tag(x_2, 6);
lean_ctor_set(x_2, 0, x_155);
lean_ctor_set(x_153, 0, x_2);
return x_150;
}
else
{
lean_object* x_156; lean_object* x_157; lean_object* x_158; 
x_156 = lean_ctor_get(x_153, 0);
x_157 = lean_ctor_get(x_153, 1);
lean_inc(x_157);
lean_inc(x_156);
lean_dec(x_153);
lean_ctor_set_tag(x_2, 6);
lean_ctor_set(x_2, 0, x_156);
x_158 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_158, 0, x_2);
lean_ctor_set(x_158, 1, x_157);
lean_ctor_set(x_150, 0, x_158);
return x_150;
}
}
else
{
lean_object* x_159; lean_object* x_160; lean_object* x_161; lean_object* x_162; lean_object* x_163; lean_object* x_164; 
x_159 = lean_ctor_get(x_150, 0);
lean_inc(x_159);
lean_dec(x_150);
x_160 = lean_ctor_get(x_159, 0);
lean_inc(x_160);
x_161 = lean_ctor_get(x_159, 1);
lean_inc(x_161);
if (lean_is_exclusive(x_159)) {
 lean_ctor_release(x_159, 0);
 lean_ctor_release(x_159, 1);
 x_162 = x_159;
} else {
 lean_dec_ref(x_159);
 x_162 = lean_box(0);
}
lean_ctor_set_tag(x_2, 6);
lean_ctor_set(x_2, 0, x_160);
if (lean_is_scalar(x_162)) {
 x_163 = lean_alloc_ctor(0, 2, 0);
} else {
 x_163 = x_162;
}
lean_ctor_set(x_163, 0, x_2);
lean_ctor_set(x_163, 1, x_161);
x_164 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_164, 0, x_163);
return x_164;
}
}
}
}
else
{
lean_object* x_165; 
lean_free_object(x_2);
x_165 = lp_iris_x2dkernel_IrisKernel_FFI_decodeTypeId(x_7);
if (lean_obj_tag(x_165) == 0)
{
lean_object* x_166; 
x_166 = lean_box(0);
return x_166;
}
else
{
lean_object* x_167; lean_object* x_168; lean_object* x_169; lean_object* x_170; 
x_167 = lean_ctor_get(x_165, 0);
lean_inc(x_167);
lean_dec_ref(x_165);
x_168 = lean_ctor_get(x_167, 0);
lean_inc(x_168);
x_169 = lean_ctor_get(x_167, 1);
lean_inc(x_169);
lean_dec(x_167);
x_170 = lp_iris_x2dkernel_IrisKernel_FFI_decodeTypeId(x_169);
if (lean_obj_tag(x_170) == 0)
{
lean_object* x_171; 
lean_dec(x_168);
x_171 = lean_box(0);
return x_171;
}
else
{
lean_object* x_172; lean_object* x_173; lean_object* x_174; lean_object* x_175; 
x_172 = lean_ctor_get(x_170, 0);
lean_inc(x_172);
lean_dec_ref(x_170);
x_173 = lean_ctor_get(x_172, 0);
lean_inc(x_173);
x_174 = lean_ctor_get(x_172, 1);
lean_inc(x_174);
lean_dec(x_172);
x_175 = lp_iris_x2dkernel_IrisKernel_FFI_decodeCostBound(x_174);
if (lean_obj_tag(x_175) == 0)
{
lean_object* x_176; 
lean_dec(x_173);
lean_dec(x_168);
x_176 = lean_box(0);
return x_176;
}
else
{
uint8_t x_177; 
x_177 = !lean_is_exclusive(x_175);
if (x_177 == 0)
{
lean_object* x_178; uint8_t x_179; 
x_178 = lean_ctor_get(x_175, 0);
x_179 = !lean_is_exclusive(x_178);
if (x_179 == 0)
{
lean_object* x_180; lean_object* x_181; 
x_180 = lean_ctor_get(x_178, 0);
x_181 = lean_alloc_ctor(5, 3, 0);
lean_ctor_set(x_181, 0, x_168);
lean_ctor_set(x_181, 1, x_173);
lean_ctor_set(x_181, 2, x_180);
lean_ctor_set(x_178, 0, x_181);
return x_175;
}
else
{
lean_object* x_182; lean_object* x_183; lean_object* x_184; lean_object* x_185; 
x_182 = lean_ctor_get(x_178, 0);
x_183 = lean_ctor_get(x_178, 1);
lean_inc(x_183);
lean_inc(x_182);
lean_dec(x_178);
x_184 = lean_alloc_ctor(5, 3, 0);
lean_ctor_set(x_184, 0, x_168);
lean_ctor_set(x_184, 1, x_173);
lean_ctor_set(x_184, 2, x_182);
x_185 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_185, 0, x_184);
lean_ctor_set(x_185, 1, x_183);
lean_ctor_set(x_175, 0, x_185);
return x_175;
}
}
else
{
lean_object* x_186; lean_object* x_187; lean_object* x_188; lean_object* x_189; lean_object* x_190; lean_object* x_191; lean_object* x_192; 
x_186 = lean_ctor_get(x_175, 0);
lean_inc(x_186);
lean_dec(x_175);
x_187 = lean_ctor_get(x_186, 0);
lean_inc(x_187);
x_188 = lean_ctor_get(x_186, 1);
lean_inc(x_188);
if (lean_is_exclusive(x_186)) {
 lean_ctor_release(x_186, 0);
 lean_ctor_release(x_186, 1);
 x_189 = x_186;
} else {
 lean_dec_ref(x_186);
 x_189 = lean_box(0);
}
x_190 = lean_alloc_ctor(5, 3, 0);
lean_ctor_set(x_190, 0, x_168);
lean_ctor_set(x_190, 1, x_173);
lean_ctor_set(x_190, 2, x_187);
if (lean_is_scalar(x_189)) {
 x_191 = lean_alloc_ctor(0, 2, 0);
} else {
 x_191 = x_189;
}
lean_ctor_set(x_191, 0, x_190);
lean_ctor_set(x_191, 1, x_188);
x_192 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_192, 0, x_191);
return x_192;
}
}
}
}
}
}
else
{
lean_object* x_193; 
lean_free_object(x_2);
x_193 = lp_iris_x2dkernel_IrisKernel_FFI_decodeBoundVar(x_7);
if (lean_obj_tag(x_193) == 0)
{
lean_object* x_194; 
x_194 = lean_box(0);
return x_194;
}
else
{
lean_object* x_195; uint8_t x_196; 
x_195 = lean_ctor_get(x_193, 0);
lean_inc(x_195);
lean_dec_ref(x_193);
x_196 = !lean_is_exclusive(x_195);
if (x_196 == 0)
{
lean_object* x_197; lean_object* x_198; lean_object* x_199; 
x_197 = lean_ctor_get(x_195, 0);
x_198 = lean_ctor_get(x_195, 1);
x_199 = lp_iris_x2dkernel_IrisKernel_FFI_decodeTypeId(x_198);
if (lean_obj_tag(x_199) == 0)
{
lean_object* x_200; 
lean_free_object(x_195);
lean_dec(x_197);
x_200 = lean_box(0);
return x_200;
}
else
{
uint8_t x_201; 
x_201 = !lean_is_exclusive(x_199);
if (x_201 == 0)
{
lean_object* x_202; uint8_t x_203; 
x_202 = lean_ctor_get(x_199, 0);
x_203 = !lean_is_exclusive(x_202);
if (x_203 == 0)
{
lean_object* x_204; 
x_204 = lean_ctor_get(x_202, 0);
lean_ctor_set_tag(x_195, 4);
lean_ctor_set(x_195, 1, x_204);
lean_ctor_set(x_202, 0, x_195);
return x_199;
}
else
{
lean_object* x_205; lean_object* x_206; lean_object* x_207; 
x_205 = lean_ctor_get(x_202, 0);
x_206 = lean_ctor_get(x_202, 1);
lean_inc(x_206);
lean_inc(x_205);
lean_dec(x_202);
lean_ctor_set_tag(x_195, 4);
lean_ctor_set(x_195, 1, x_205);
x_207 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_207, 0, x_195);
lean_ctor_set(x_207, 1, x_206);
lean_ctor_set(x_199, 0, x_207);
return x_199;
}
}
else
{
lean_object* x_208; lean_object* x_209; lean_object* x_210; lean_object* x_211; lean_object* x_212; lean_object* x_213; 
x_208 = lean_ctor_get(x_199, 0);
lean_inc(x_208);
lean_dec(x_199);
x_209 = lean_ctor_get(x_208, 0);
lean_inc(x_209);
x_210 = lean_ctor_get(x_208, 1);
lean_inc(x_210);
if (lean_is_exclusive(x_208)) {
 lean_ctor_release(x_208, 0);
 lean_ctor_release(x_208, 1);
 x_211 = x_208;
} else {
 lean_dec_ref(x_208);
 x_211 = lean_box(0);
}
lean_ctor_set_tag(x_195, 4);
lean_ctor_set(x_195, 1, x_209);
if (lean_is_scalar(x_211)) {
 x_212 = lean_alloc_ctor(0, 2, 0);
} else {
 x_212 = x_211;
}
lean_ctor_set(x_212, 0, x_195);
lean_ctor_set(x_212, 1, x_210);
x_213 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_213, 0, x_212);
return x_213;
}
}
}
else
{
lean_object* x_214; lean_object* x_215; lean_object* x_216; 
x_214 = lean_ctor_get(x_195, 0);
x_215 = lean_ctor_get(x_195, 1);
lean_inc(x_215);
lean_inc(x_214);
lean_dec(x_195);
x_216 = lp_iris_x2dkernel_IrisKernel_FFI_decodeTypeId(x_215);
if (lean_obj_tag(x_216) == 0)
{
lean_object* x_217; 
lean_dec(x_214);
x_217 = lean_box(0);
return x_217;
}
else
{
lean_object* x_218; lean_object* x_219; lean_object* x_220; lean_object* x_221; lean_object* x_222; lean_object* x_223; lean_object* x_224; lean_object* x_225; 
x_218 = lean_ctor_get(x_216, 0);
lean_inc(x_218);
if (lean_is_exclusive(x_216)) {
 lean_ctor_release(x_216, 0);
 x_219 = x_216;
} else {
 lean_dec_ref(x_216);
 x_219 = lean_box(0);
}
x_220 = lean_ctor_get(x_218, 0);
lean_inc(x_220);
x_221 = lean_ctor_get(x_218, 1);
lean_inc(x_221);
if (lean_is_exclusive(x_218)) {
 lean_ctor_release(x_218, 0);
 lean_ctor_release(x_218, 1);
 x_222 = x_218;
} else {
 lean_dec_ref(x_218);
 x_222 = lean_box(0);
}
x_223 = lean_alloc_ctor(4, 2, 0);
lean_ctor_set(x_223, 0, x_214);
lean_ctor_set(x_223, 1, x_220);
if (lean_is_scalar(x_222)) {
 x_224 = lean_alloc_ctor(0, 2, 0);
} else {
 x_224 = x_222;
}
lean_ctor_set(x_224, 0, x_223);
lean_ctor_set(x_224, 1, x_221);
if (lean_is_scalar(x_219)) {
 x_225 = lean_alloc_ctor(1, 1, 0);
} else {
 x_225 = x_219;
}
lean_ctor_set(x_225, 0, x_224);
return x_225;
}
}
}
}
}
else
{
lean_object* x_226; 
lean_free_object(x_2);
x_226 = lp_iris_x2dkernel_IrisKernel_FFI_decodeBoundVar(x_7);
if (lean_obj_tag(x_226) == 0)
{
lean_object* x_227; 
x_227 = lean_box(0);
return x_227;
}
else
{
lean_object* x_228; uint8_t x_229; 
x_228 = lean_ctor_get(x_226, 0);
lean_inc(x_228);
lean_dec_ref(x_226);
x_229 = !lean_is_exclusive(x_228);
if (x_229 == 0)
{
lean_object* x_230; lean_object* x_231; lean_object* x_232; 
x_230 = lean_ctor_get(x_228, 0);
x_231 = lean_ctor_get(x_228, 1);
x_232 = lp_iris_x2dkernel_IrisKernel_FFI_decodeTypeId(x_231);
if (lean_obj_tag(x_232) == 0)
{
lean_object* x_233; 
lean_free_object(x_228);
lean_dec(x_230);
x_233 = lean_box(0);
return x_233;
}
else
{
uint8_t x_234; 
x_234 = !lean_is_exclusive(x_232);
if (x_234 == 0)
{
lean_object* x_235; uint8_t x_236; 
x_235 = lean_ctor_get(x_232, 0);
x_236 = !lean_is_exclusive(x_235);
if (x_236 == 0)
{
lean_object* x_237; 
x_237 = lean_ctor_get(x_235, 0);
lean_ctor_set_tag(x_228, 3);
lean_ctor_set(x_228, 1, x_237);
lean_ctor_set(x_235, 0, x_228);
return x_232;
}
else
{
lean_object* x_238; lean_object* x_239; lean_object* x_240; 
x_238 = lean_ctor_get(x_235, 0);
x_239 = lean_ctor_get(x_235, 1);
lean_inc(x_239);
lean_inc(x_238);
lean_dec(x_235);
lean_ctor_set_tag(x_228, 3);
lean_ctor_set(x_228, 1, x_238);
x_240 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_240, 0, x_228);
lean_ctor_set(x_240, 1, x_239);
lean_ctor_set(x_232, 0, x_240);
return x_232;
}
}
else
{
lean_object* x_241; lean_object* x_242; lean_object* x_243; lean_object* x_244; lean_object* x_245; lean_object* x_246; 
x_241 = lean_ctor_get(x_232, 0);
lean_inc(x_241);
lean_dec(x_232);
x_242 = lean_ctor_get(x_241, 0);
lean_inc(x_242);
x_243 = lean_ctor_get(x_241, 1);
lean_inc(x_243);
if (lean_is_exclusive(x_241)) {
 lean_ctor_release(x_241, 0);
 lean_ctor_release(x_241, 1);
 x_244 = x_241;
} else {
 lean_dec_ref(x_241);
 x_244 = lean_box(0);
}
lean_ctor_set_tag(x_228, 3);
lean_ctor_set(x_228, 1, x_242);
if (lean_is_scalar(x_244)) {
 x_245 = lean_alloc_ctor(0, 2, 0);
} else {
 x_245 = x_244;
}
lean_ctor_set(x_245, 0, x_228);
lean_ctor_set(x_245, 1, x_243);
x_246 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_246, 0, x_245);
return x_246;
}
}
}
else
{
lean_object* x_247; lean_object* x_248; lean_object* x_249; 
x_247 = lean_ctor_get(x_228, 0);
x_248 = lean_ctor_get(x_228, 1);
lean_inc(x_248);
lean_inc(x_247);
lean_dec(x_228);
x_249 = lp_iris_x2dkernel_IrisKernel_FFI_decodeTypeId(x_248);
if (lean_obj_tag(x_249) == 0)
{
lean_object* x_250; 
lean_dec(x_247);
x_250 = lean_box(0);
return x_250;
}
else
{
lean_object* x_251; lean_object* x_252; lean_object* x_253; lean_object* x_254; lean_object* x_255; lean_object* x_256; lean_object* x_257; lean_object* x_258; 
x_251 = lean_ctor_get(x_249, 0);
lean_inc(x_251);
if (lean_is_exclusive(x_249)) {
 lean_ctor_release(x_249, 0);
 x_252 = x_249;
} else {
 lean_dec_ref(x_249);
 x_252 = lean_box(0);
}
x_253 = lean_ctor_get(x_251, 0);
lean_inc(x_253);
x_254 = lean_ctor_get(x_251, 1);
lean_inc(x_254);
if (lean_is_exclusive(x_251)) {
 lean_ctor_release(x_251, 0);
 lean_ctor_release(x_251, 1);
 x_255 = x_251;
} else {
 lean_dec_ref(x_251);
 x_255 = lean_box(0);
}
x_256 = lean_alloc_ctor(3, 2, 0);
lean_ctor_set(x_256, 0, x_247);
lean_ctor_set(x_256, 1, x_253);
if (lean_is_scalar(x_255)) {
 x_257 = lean_alloc_ctor(0, 2, 0);
} else {
 x_257 = x_255;
}
lean_ctor_set(x_257, 0, x_256);
lean_ctor_set(x_257, 1, x_254);
if (lean_is_scalar(x_252)) {
 x_258 = lean_alloc_ctor(1, 1, 0);
} else {
 x_258 = x_252;
}
lean_ctor_set(x_258, 0, x_257);
return x_258;
}
}
}
}
}
else
{
lean_object* x_259; 
lean_free_object(x_2);
x_259 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readUInt16LE(x_7);
if (lean_obj_tag(x_259) == 0)
{
lean_object* x_260; 
x_260 = lean_box(0);
return x_260;
}
else
{
uint8_t x_261; 
x_261 = !lean_is_exclusive(x_259);
if (x_261 == 0)
{
lean_object* x_262; lean_object* x_263; lean_object* x_264; uint16_t x_265; lean_object* x_266; lean_object* x_267; lean_object* x_268; 
x_262 = lean_ctor_get(x_259, 0);
x_263 = lean_ctor_get(x_262, 0);
lean_inc(x_263);
x_264 = lean_ctor_get(x_262, 1);
lean_inc(x_264);
lean_dec(x_262);
x_265 = lean_unbox(x_263);
lean_dec(x_263);
x_266 = lean_uint16_to_nat(x_265);
x_267 = lean_box(0);
x_268 = lp_iris_x2dkernel_IrisKernel_FFI_decodeTypeDef_decodeTagTypeIdList(x_264, x_266, x_267);
if (lean_obj_tag(x_268) == 0)
{
lean_object* x_269; 
lean_free_object(x_259);
x_269 = lean_box(0);
return x_269;
}
else
{
uint8_t x_270; 
x_270 = !lean_is_exclusive(x_268);
if (x_270 == 0)
{
lean_object* x_271; uint8_t x_272; 
x_271 = lean_ctor_get(x_268, 0);
x_272 = !lean_is_exclusive(x_271);
if (x_272 == 0)
{
lean_object* x_273; 
x_273 = lean_ctor_get(x_271, 0);
lean_ctor_set_tag(x_259, 2);
lean_ctor_set(x_259, 0, x_273);
lean_ctor_set(x_271, 0, x_259);
return x_268;
}
else
{
lean_object* x_274; lean_object* x_275; lean_object* x_276; 
x_274 = lean_ctor_get(x_271, 0);
x_275 = lean_ctor_get(x_271, 1);
lean_inc(x_275);
lean_inc(x_274);
lean_dec(x_271);
lean_ctor_set_tag(x_259, 2);
lean_ctor_set(x_259, 0, x_274);
x_276 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_276, 0, x_259);
lean_ctor_set(x_276, 1, x_275);
lean_ctor_set(x_268, 0, x_276);
return x_268;
}
}
else
{
lean_object* x_277; lean_object* x_278; lean_object* x_279; lean_object* x_280; lean_object* x_281; lean_object* x_282; 
x_277 = lean_ctor_get(x_268, 0);
lean_inc(x_277);
lean_dec(x_268);
x_278 = lean_ctor_get(x_277, 0);
lean_inc(x_278);
x_279 = lean_ctor_get(x_277, 1);
lean_inc(x_279);
if (lean_is_exclusive(x_277)) {
 lean_ctor_release(x_277, 0);
 lean_ctor_release(x_277, 1);
 x_280 = x_277;
} else {
 lean_dec_ref(x_277);
 x_280 = lean_box(0);
}
lean_ctor_set_tag(x_259, 2);
lean_ctor_set(x_259, 0, x_278);
if (lean_is_scalar(x_280)) {
 x_281 = lean_alloc_ctor(0, 2, 0);
} else {
 x_281 = x_280;
}
lean_ctor_set(x_281, 0, x_259);
lean_ctor_set(x_281, 1, x_279);
x_282 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_282, 0, x_281);
return x_282;
}
}
}
else
{
lean_object* x_283; lean_object* x_284; lean_object* x_285; uint16_t x_286; lean_object* x_287; lean_object* x_288; lean_object* x_289; 
x_283 = lean_ctor_get(x_259, 0);
lean_inc(x_283);
lean_dec(x_259);
x_284 = lean_ctor_get(x_283, 0);
lean_inc(x_284);
x_285 = lean_ctor_get(x_283, 1);
lean_inc(x_285);
lean_dec(x_283);
x_286 = lean_unbox(x_284);
lean_dec(x_284);
x_287 = lean_uint16_to_nat(x_286);
x_288 = lean_box(0);
x_289 = lp_iris_x2dkernel_IrisKernel_FFI_decodeTypeDef_decodeTagTypeIdList(x_285, x_287, x_288);
if (lean_obj_tag(x_289) == 0)
{
lean_object* x_290; 
x_290 = lean_box(0);
return x_290;
}
else
{
lean_object* x_291; lean_object* x_292; lean_object* x_293; lean_object* x_294; lean_object* x_295; lean_object* x_296; lean_object* x_297; lean_object* x_298; 
x_291 = lean_ctor_get(x_289, 0);
lean_inc(x_291);
if (lean_is_exclusive(x_289)) {
 lean_ctor_release(x_289, 0);
 x_292 = x_289;
} else {
 lean_dec_ref(x_289);
 x_292 = lean_box(0);
}
x_293 = lean_ctor_get(x_291, 0);
lean_inc(x_293);
x_294 = lean_ctor_get(x_291, 1);
lean_inc(x_294);
if (lean_is_exclusive(x_291)) {
 lean_ctor_release(x_291, 0);
 lean_ctor_release(x_291, 1);
 x_295 = x_291;
} else {
 lean_dec_ref(x_291);
 x_295 = lean_box(0);
}
x_296 = lean_alloc_ctor(2, 1, 0);
lean_ctor_set(x_296, 0, x_293);
if (lean_is_scalar(x_295)) {
 x_297 = lean_alloc_ctor(0, 2, 0);
} else {
 x_297 = x_295;
}
lean_ctor_set(x_297, 0, x_296);
lean_ctor_set(x_297, 1, x_294);
if (lean_is_scalar(x_292)) {
 x_298 = lean_alloc_ctor(1, 1, 0);
} else {
 x_298 = x_292;
}
lean_ctor_set(x_298, 0, x_297);
return x_298;
}
}
}
}
}
else
{
lean_object* x_299; 
lean_free_object(x_2);
x_299 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readUInt16LE(x_7);
if (lean_obj_tag(x_299) == 0)
{
lean_object* x_300; 
x_300 = lean_box(0);
return x_300;
}
else
{
uint8_t x_301; 
x_301 = !lean_is_exclusive(x_299);
if (x_301 == 0)
{
lean_object* x_302; lean_object* x_303; lean_object* x_304; uint16_t x_305; lean_object* x_306; lean_object* x_307; lean_object* x_308; 
x_302 = lean_ctor_get(x_299, 0);
x_303 = lean_ctor_get(x_302, 0);
lean_inc(x_303);
x_304 = lean_ctor_get(x_302, 1);
lean_inc(x_304);
lean_dec(x_302);
x_305 = lean_unbox(x_303);
lean_dec(x_303);
x_306 = lean_uint16_to_nat(x_305);
x_307 = lean_box(0);
x_308 = lp_iris_x2dkernel_IrisKernel_FFI_decodeTypeDef_decodeTypeIdList(x_304, x_306, x_307);
if (lean_obj_tag(x_308) == 0)
{
lean_object* x_309; 
lean_free_object(x_299);
x_309 = lean_box(0);
return x_309;
}
else
{
uint8_t x_310; 
x_310 = !lean_is_exclusive(x_308);
if (x_310 == 0)
{
lean_object* x_311; uint8_t x_312; 
x_311 = lean_ctor_get(x_308, 0);
x_312 = !lean_is_exclusive(x_311);
if (x_312 == 0)
{
lean_object* x_313; 
x_313 = lean_ctor_get(x_311, 0);
lean_ctor_set(x_299, 0, x_313);
lean_ctor_set(x_311, 0, x_299);
return x_308;
}
else
{
lean_object* x_314; lean_object* x_315; lean_object* x_316; 
x_314 = lean_ctor_get(x_311, 0);
x_315 = lean_ctor_get(x_311, 1);
lean_inc(x_315);
lean_inc(x_314);
lean_dec(x_311);
lean_ctor_set(x_299, 0, x_314);
x_316 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_316, 0, x_299);
lean_ctor_set(x_316, 1, x_315);
lean_ctor_set(x_308, 0, x_316);
return x_308;
}
}
else
{
lean_object* x_317; lean_object* x_318; lean_object* x_319; lean_object* x_320; lean_object* x_321; lean_object* x_322; 
x_317 = lean_ctor_get(x_308, 0);
lean_inc(x_317);
lean_dec(x_308);
x_318 = lean_ctor_get(x_317, 0);
lean_inc(x_318);
x_319 = lean_ctor_get(x_317, 1);
lean_inc(x_319);
if (lean_is_exclusive(x_317)) {
 lean_ctor_release(x_317, 0);
 lean_ctor_release(x_317, 1);
 x_320 = x_317;
} else {
 lean_dec_ref(x_317);
 x_320 = lean_box(0);
}
lean_ctor_set(x_299, 0, x_318);
if (lean_is_scalar(x_320)) {
 x_321 = lean_alloc_ctor(0, 2, 0);
} else {
 x_321 = x_320;
}
lean_ctor_set(x_321, 0, x_299);
lean_ctor_set(x_321, 1, x_319);
x_322 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_322, 0, x_321);
return x_322;
}
}
}
else
{
lean_object* x_323; lean_object* x_324; lean_object* x_325; uint16_t x_326; lean_object* x_327; lean_object* x_328; lean_object* x_329; 
x_323 = lean_ctor_get(x_299, 0);
lean_inc(x_323);
lean_dec(x_299);
x_324 = lean_ctor_get(x_323, 0);
lean_inc(x_324);
x_325 = lean_ctor_get(x_323, 1);
lean_inc(x_325);
lean_dec(x_323);
x_326 = lean_unbox(x_324);
lean_dec(x_324);
x_327 = lean_uint16_to_nat(x_326);
x_328 = lean_box(0);
x_329 = lp_iris_x2dkernel_IrisKernel_FFI_decodeTypeDef_decodeTypeIdList(x_325, x_327, x_328);
if (lean_obj_tag(x_329) == 0)
{
lean_object* x_330; 
x_330 = lean_box(0);
return x_330;
}
else
{
lean_object* x_331; lean_object* x_332; lean_object* x_333; lean_object* x_334; lean_object* x_335; lean_object* x_336; lean_object* x_337; lean_object* x_338; 
x_331 = lean_ctor_get(x_329, 0);
lean_inc(x_331);
if (lean_is_exclusive(x_329)) {
 lean_ctor_release(x_329, 0);
 x_332 = x_329;
} else {
 lean_dec_ref(x_329);
 x_332 = lean_box(0);
}
x_333 = lean_ctor_get(x_331, 0);
lean_inc(x_333);
x_334 = lean_ctor_get(x_331, 1);
lean_inc(x_334);
if (lean_is_exclusive(x_331)) {
 lean_ctor_release(x_331, 0);
 lean_ctor_release(x_331, 1);
 x_335 = x_331;
} else {
 lean_dec_ref(x_331);
 x_335 = lean_box(0);
}
x_336 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_336, 0, x_333);
if (lean_is_scalar(x_335)) {
 x_337 = lean_alloc_ctor(0, 2, 0);
} else {
 x_337 = x_335;
}
lean_ctor_set(x_337, 0, x_336);
lean_ctor_set(x_337, 1, x_334);
if (lean_is_scalar(x_332)) {
 x_338 = lean_alloc_ctor(1, 1, 0);
} else {
 x_338 = x_332;
}
lean_ctor_set(x_338, 0, x_337);
return x_338;
}
}
}
}
}
else
{
lean_object* x_339; 
lean_free_object(x_2);
x_339 = lp_iris_x2dkernel_IrisKernel_FFI_decodePrimType(x_7);
if (lean_obj_tag(x_339) == 0)
{
lean_object* x_340; 
x_340 = lean_box(0);
return x_340;
}
else
{
uint8_t x_341; 
x_341 = !lean_is_exclusive(x_339);
if (x_341 == 0)
{
lean_object* x_342; uint8_t x_343; 
x_342 = lean_ctor_get(x_339, 0);
x_343 = !lean_is_exclusive(x_342);
if (x_343 == 0)
{
lean_object* x_344; lean_object* x_345; uint8_t x_346; 
x_344 = lean_ctor_get(x_342, 0);
x_345 = lean_alloc_ctor(0, 0, 1);
x_346 = lean_unbox(x_344);
lean_dec(x_344);
lean_ctor_set_uint8(x_345, 0, x_346);
lean_ctor_set(x_342, 0, x_345);
return x_339;
}
else
{
lean_object* x_347; lean_object* x_348; lean_object* x_349; uint8_t x_350; lean_object* x_351; 
x_347 = lean_ctor_get(x_342, 0);
x_348 = lean_ctor_get(x_342, 1);
lean_inc(x_348);
lean_inc(x_347);
lean_dec(x_342);
x_349 = lean_alloc_ctor(0, 0, 1);
x_350 = lean_unbox(x_347);
lean_dec(x_347);
lean_ctor_set_uint8(x_349, 0, x_350);
x_351 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_351, 0, x_349);
lean_ctor_set(x_351, 1, x_348);
lean_ctor_set(x_339, 0, x_351);
return x_339;
}
}
else
{
lean_object* x_352; lean_object* x_353; lean_object* x_354; lean_object* x_355; lean_object* x_356; uint8_t x_357; lean_object* x_358; lean_object* x_359; 
x_352 = lean_ctor_get(x_339, 0);
lean_inc(x_352);
lean_dec(x_339);
x_353 = lean_ctor_get(x_352, 0);
lean_inc(x_353);
x_354 = lean_ctor_get(x_352, 1);
lean_inc(x_354);
if (lean_is_exclusive(x_352)) {
 lean_ctor_release(x_352, 0);
 lean_ctor_release(x_352, 1);
 x_355 = x_352;
} else {
 lean_dec_ref(x_352);
 x_355 = lean_box(0);
}
x_356 = lean_alloc_ctor(0, 0, 1);
x_357 = lean_unbox(x_353);
lean_dec(x_353);
lean_ctor_set_uint8(x_356, 0, x_357);
if (lean_is_scalar(x_355)) {
 x_358 = lean_alloc_ctor(0, 2, 0);
} else {
 x_358 = x_355;
}
lean_ctor_set(x_358, 0, x_356);
lean_ctor_set(x_358, 1, x_354);
x_359 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_359, 0, x_358);
return x_359;
}
}
}
}
else
{
lean_object* x_360; lean_object* x_361; lean_object* x_362; uint8_t x_363; lean_object* x_364; lean_object* x_365; uint8_t x_366; 
x_360 = lean_ctor_get(x_2, 0);
lean_inc(x_360);
lean_dec(x_2);
x_361 = lean_ctor_get(x_360, 0);
lean_inc(x_361);
x_362 = lean_ctor_get(x_360, 1);
lean_inc(x_362);
lean_dec(x_360);
x_363 = lean_unbox(x_361);
lean_dec(x_361);
x_364 = lean_uint8_to_nat(x_363);
x_365 = lean_unsigned_to_nat(0u);
x_366 = lean_nat_dec_eq(x_364, x_365);
if (x_366 == 0)
{
lean_object* x_367; uint8_t x_368; 
x_367 = lean_unsigned_to_nat(1u);
x_368 = lean_nat_dec_eq(x_364, x_367);
if (x_368 == 0)
{
lean_object* x_369; uint8_t x_370; 
x_369 = lean_unsigned_to_nat(2u);
x_370 = lean_nat_dec_eq(x_364, x_369);
if (x_370 == 0)
{
lean_object* x_371; uint8_t x_372; 
x_371 = lean_unsigned_to_nat(3u);
x_372 = lean_nat_dec_eq(x_364, x_371);
if (x_372 == 0)
{
lean_object* x_373; uint8_t x_374; 
x_373 = lean_unsigned_to_nat(4u);
x_374 = lean_nat_dec_eq(x_364, x_373);
if (x_374 == 0)
{
lean_object* x_375; uint8_t x_376; 
x_375 = lean_unsigned_to_nat(5u);
x_376 = lean_nat_dec_eq(x_364, x_375);
if (x_376 == 0)
{
lean_object* x_377; uint8_t x_378; 
x_377 = lean_unsigned_to_nat(6u);
x_378 = lean_nat_dec_eq(x_364, x_377);
if (x_378 == 0)
{
lean_object* x_379; uint8_t x_380; 
x_379 = lean_unsigned_to_nat(7u);
x_380 = lean_nat_dec_eq(x_364, x_379);
if (x_380 == 0)
{
lean_object* x_381; uint8_t x_382; 
x_381 = lean_unsigned_to_nat(8u);
x_382 = lean_nat_dec_eq(x_364, x_381);
if (x_382 == 0)
{
lean_object* x_383; uint8_t x_384; 
x_383 = lean_unsigned_to_nat(9u);
x_384 = lean_nat_dec_eq(x_364, x_383);
if (x_384 == 0)
{
lean_object* x_385; uint8_t x_386; 
x_385 = lean_unsigned_to_nat(10u);
x_386 = lean_nat_dec_eq(x_364, x_385);
if (x_386 == 0)
{
lean_object* x_387; 
lean_dec(x_362);
x_387 = lean_box(0);
return x_387;
}
else
{
lean_object* x_388; 
x_388 = lp_iris_x2dkernel_IrisKernel_FFI_decodeTypeId(x_362);
if (lean_obj_tag(x_388) == 0)
{
lean_object* x_389; 
x_389 = lean_box(0);
return x_389;
}
else
{
lean_object* x_390; lean_object* x_391; lean_object* x_392; lean_object* x_393; lean_object* x_394; lean_object* x_395; lean_object* x_396; lean_object* x_397; 
x_390 = lean_ctor_get(x_388, 0);
lean_inc(x_390);
if (lean_is_exclusive(x_388)) {
 lean_ctor_release(x_388, 0);
 x_391 = x_388;
} else {
 lean_dec_ref(x_388);
 x_391 = lean_box(0);
}
x_392 = lean_ctor_get(x_390, 0);
lean_inc(x_392);
x_393 = lean_ctor_get(x_390, 1);
lean_inc(x_393);
if (lean_is_exclusive(x_390)) {
 lean_ctor_release(x_390, 0);
 lean_ctor_release(x_390, 1);
 x_394 = x_390;
} else {
 lean_dec_ref(x_390);
 x_394 = lean_box(0);
}
x_395 = lean_alloc_ctor(10, 1, 0);
lean_ctor_set(x_395, 0, x_392);
if (lean_is_scalar(x_394)) {
 x_396 = lean_alloc_ctor(0, 2, 0);
} else {
 x_396 = x_394;
}
lean_ctor_set(x_396, 0, x_395);
lean_ctor_set(x_396, 1, x_393);
if (lean_is_scalar(x_391)) {
 x_397 = lean_alloc_ctor(1, 1, 0);
} else {
 x_397 = x_391;
}
lean_ctor_set(x_397, 0, x_396);
return x_397;
}
}
}
else
{
lean_object* x_398; 
x_398 = lp_iris_x2dkernel_IrisKernel_FFI_decodeTypeId(x_362);
if (lean_obj_tag(x_398) == 0)
{
lean_object* x_399; 
x_399 = lean_box(0);
return x_399;
}
else
{
lean_object* x_400; lean_object* x_401; lean_object* x_402; lean_object* x_403; lean_object* x_404; 
x_400 = lean_ctor_get(x_398, 0);
lean_inc(x_400);
lean_dec_ref(x_398);
x_401 = lean_ctor_get(x_400, 0);
lean_inc(x_401);
x_402 = lean_ctor_get(x_400, 1);
lean_inc(x_402);
if (lean_is_exclusive(x_400)) {
 lean_ctor_release(x_400, 0);
 lean_ctor_release(x_400, 1);
 x_403 = x_400;
} else {
 lean_dec_ref(x_400);
 x_403 = lean_box(0);
}
x_404 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readUInt64LE(x_402);
if (lean_obj_tag(x_404) == 0)
{
lean_object* x_405; 
lean_dec(x_403);
lean_dec(x_401);
x_405 = lean_box(0);
return x_405;
}
else
{
lean_object* x_406; lean_object* x_407; lean_object* x_408; lean_object* x_409; lean_object* x_410; uint64_t x_411; lean_object* x_412; lean_object* x_413; lean_object* x_414; lean_object* x_415; 
x_406 = lean_ctor_get(x_404, 0);
lean_inc(x_406);
if (lean_is_exclusive(x_404)) {
 lean_ctor_release(x_404, 0);
 x_407 = x_404;
} else {
 lean_dec_ref(x_404);
 x_407 = lean_box(0);
}
x_408 = lean_ctor_get(x_406, 0);
lean_inc(x_408);
x_409 = lean_ctor_get(x_406, 1);
lean_inc(x_409);
if (lean_is_exclusive(x_406)) {
 lean_ctor_release(x_406, 0);
 lean_ctor_release(x_406, 1);
 x_410 = x_406;
} else {
 lean_dec_ref(x_406);
 x_410 = lean_box(0);
}
x_411 = lean_unbox_uint64(x_408);
lean_dec(x_408);
x_412 = lean_uint64_to_nat(x_411);
if (lean_is_scalar(x_403)) {
 x_413 = lean_alloc_ctor(9, 2, 0);
} else {
 x_413 = x_403;
 lean_ctor_set_tag(x_413, 9);
}
lean_ctor_set(x_413, 0, x_401);
lean_ctor_set(x_413, 1, x_412);
if (lean_is_scalar(x_410)) {
 x_414 = lean_alloc_ctor(0, 2, 0);
} else {
 x_414 = x_410;
}
lean_ctor_set(x_414, 0, x_413);
lean_ctor_set(x_414, 1, x_409);
if (lean_is_scalar(x_407)) {
 x_415 = lean_alloc_ctor(1, 1, 0);
} else {
 x_415 = x_407;
}
lean_ctor_set(x_415, 0, x_414);
return x_415;
}
}
}
}
else
{
lean_object* x_416; 
x_416 = lp_iris_x2dkernel_IrisKernel_FFI_decodeBoundVar(x_362);
if (lean_obj_tag(x_416) == 0)
{
lean_object* x_417; 
x_417 = lean_box(0);
return x_417;
}
else
{
lean_object* x_418; lean_object* x_419; lean_object* x_420; lean_object* x_421; lean_object* x_422; 
x_418 = lean_ctor_get(x_416, 0);
lean_inc(x_418);
lean_dec_ref(x_416);
x_419 = lean_ctor_get(x_418, 0);
lean_inc(x_419);
x_420 = lean_ctor_get(x_418, 1);
lean_inc(x_420);
if (lean_is_exclusive(x_418)) {
 lean_ctor_release(x_418, 0);
 lean_ctor_release(x_418, 1);
 x_421 = x_418;
} else {
 lean_dec_ref(x_418);
 x_421 = lean_box(0);
}
x_422 = lp_iris_x2dkernel_IrisKernel_FFI_decodeTypeId(x_420);
if (lean_obj_tag(x_422) == 0)
{
lean_object* x_423; 
lean_dec(x_421);
lean_dec(x_419);
x_423 = lean_box(0);
return x_423;
}
else
{
lean_object* x_424; lean_object* x_425; lean_object* x_426; lean_object* x_427; lean_object* x_428; lean_object* x_429; lean_object* x_430; lean_object* x_431; 
x_424 = lean_ctor_get(x_422, 0);
lean_inc(x_424);
if (lean_is_exclusive(x_422)) {
 lean_ctor_release(x_422, 0);
 x_425 = x_422;
} else {
 lean_dec_ref(x_422);
 x_425 = lean_box(0);
}
x_426 = lean_ctor_get(x_424, 0);
lean_inc(x_426);
x_427 = lean_ctor_get(x_424, 1);
lean_inc(x_427);
if (lean_is_exclusive(x_424)) {
 lean_ctor_release(x_424, 0);
 lean_ctor_release(x_424, 1);
 x_428 = x_424;
} else {
 lean_dec_ref(x_424);
 x_428 = lean_box(0);
}
if (lean_is_scalar(x_421)) {
 x_429 = lean_alloc_ctor(8, 2, 0);
} else {
 x_429 = x_421;
 lean_ctor_set_tag(x_429, 8);
}
lean_ctor_set(x_429, 0, x_419);
lean_ctor_set(x_429, 1, x_426);
if (lean_is_scalar(x_428)) {
 x_430 = lean_alloc_ctor(0, 2, 0);
} else {
 x_430 = x_428;
}
lean_ctor_set(x_430, 0, x_429);
lean_ctor_set(x_430, 1, x_427);
if (lean_is_scalar(x_425)) {
 x_431 = lean_alloc_ctor(1, 1, 0);
} else {
 x_431 = x_425;
}
lean_ctor_set(x_431, 0, x_430);
return x_431;
}
}
}
}
else
{
lean_object* x_432; 
x_432 = lp_iris_x2dkernel_IrisKernel_FFI_decodeTypeId(x_362);
if (lean_obj_tag(x_432) == 0)
{
lean_object* x_433; 
x_433 = lean_box(0);
return x_433;
}
else
{
lean_object* x_434; lean_object* x_435; lean_object* x_436; lean_object* x_437; 
x_434 = lean_ctor_get(x_432, 0);
lean_inc(x_434);
lean_dec_ref(x_432);
x_435 = lean_ctor_get(x_434, 0);
lean_inc(x_435);
x_436 = lean_ctor_get(x_434, 1);
lean_inc(x_436);
lean_dec(x_434);
x_437 = lp_iris_x2dkernel_IrisKernel_FFI_decodeTypeId(x_436);
if (lean_obj_tag(x_437) == 0)
{
lean_object* x_438; 
lean_dec(x_435);
x_438 = lean_box(0);
return x_438;
}
else
{
lean_object* x_439; lean_object* x_440; lean_object* x_441; lean_object* x_442; 
x_439 = lean_ctor_get(x_437, 0);
lean_inc(x_439);
lean_dec_ref(x_437);
x_440 = lean_ctor_get(x_439, 0);
lean_inc(x_440);
x_441 = lean_ctor_get(x_439, 1);
lean_inc(x_441);
lean_dec(x_439);
x_442 = lp_iris_x2dkernel_IrisKernel_FFI_decodeCostBound(x_441);
if (lean_obj_tag(x_442) == 0)
{
lean_object* x_443; 
lean_dec(x_440);
lean_dec(x_435);
x_443 = lean_box(0);
return x_443;
}
else
{
lean_object* x_444; lean_object* x_445; lean_object* x_446; lean_object* x_447; lean_object* x_448; lean_object* x_449; lean_object* x_450; lean_object* x_451; 
x_444 = lean_ctor_get(x_442, 0);
lean_inc(x_444);
if (lean_is_exclusive(x_442)) {
 lean_ctor_release(x_442, 0);
 x_445 = x_442;
} else {
 lean_dec_ref(x_442);
 x_445 = lean_box(0);
}
x_446 = lean_ctor_get(x_444, 0);
lean_inc(x_446);
x_447 = lean_ctor_get(x_444, 1);
lean_inc(x_447);
if (lean_is_exclusive(x_444)) {
 lean_ctor_release(x_444, 0);
 lean_ctor_release(x_444, 1);
 x_448 = x_444;
} else {
 lean_dec_ref(x_444);
 x_448 = lean_box(0);
}
x_449 = lean_alloc_ctor(7, 3, 0);
lean_ctor_set(x_449, 0, x_435);
lean_ctor_set(x_449, 1, x_440);
lean_ctor_set(x_449, 2, x_446);
if (lean_is_scalar(x_448)) {
 x_450 = lean_alloc_ctor(0, 2, 0);
} else {
 x_450 = x_448;
}
lean_ctor_set(x_450, 0, x_449);
lean_ctor_set(x_450, 1, x_447);
if (lean_is_scalar(x_445)) {
 x_451 = lean_alloc_ctor(1, 1, 0);
} else {
 x_451 = x_445;
}
lean_ctor_set(x_451, 0, x_450);
return x_451;
}
}
}
}
}
else
{
lean_object* x_452; 
x_452 = lp_iris_x2dkernel_IrisKernel_FFI_decodeTypeId(x_362);
if (lean_obj_tag(x_452) == 0)
{
lean_object* x_453; 
x_453 = lean_box(0);
return x_453;
}
else
{
lean_object* x_454; lean_object* x_455; lean_object* x_456; lean_object* x_457; lean_object* x_458; lean_object* x_459; lean_object* x_460; lean_object* x_461; 
x_454 = lean_ctor_get(x_452, 0);
lean_inc(x_454);
if (lean_is_exclusive(x_452)) {
 lean_ctor_release(x_452, 0);
 x_455 = x_452;
} else {
 lean_dec_ref(x_452);
 x_455 = lean_box(0);
}
x_456 = lean_ctor_get(x_454, 0);
lean_inc(x_456);
x_457 = lean_ctor_get(x_454, 1);
lean_inc(x_457);
if (lean_is_exclusive(x_454)) {
 lean_ctor_release(x_454, 0);
 lean_ctor_release(x_454, 1);
 x_458 = x_454;
} else {
 lean_dec_ref(x_454);
 x_458 = lean_box(0);
}
x_459 = lean_alloc_ctor(6, 1, 0);
lean_ctor_set(x_459, 0, x_456);
if (lean_is_scalar(x_458)) {
 x_460 = lean_alloc_ctor(0, 2, 0);
} else {
 x_460 = x_458;
}
lean_ctor_set(x_460, 0, x_459);
lean_ctor_set(x_460, 1, x_457);
if (lean_is_scalar(x_455)) {
 x_461 = lean_alloc_ctor(1, 1, 0);
} else {
 x_461 = x_455;
}
lean_ctor_set(x_461, 0, x_460);
return x_461;
}
}
}
else
{
lean_object* x_462; 
x_462 = lp_iris_x2dkernel_IrisKernel_FFI_decodeTypeId(x_362);
if (lean_obj_tag(x_462) == 0)
{
lean_object* x_463; 
x_463 = lean_box(0);
return x_463;
}
else
{
lean_object* x_464; lean_object* x_465; lean_object* x_466; lean_object* x_467; 
x_464 = lean_ctor_get(x_462, 0);
lean_inc(x_464);
lean_dec_ref(x_462);
x_465 = lean_ctor_get(x_464, 0);
lean_inc(x_465);
x_466 = lean_ctor_get(x_464, 1);
lean_inc(x_466);
lean_dec(x_464);
x_467 = lp_iris_x2dkernel_IrisKernel_FFI_decodeTypeId(x_466);
if (lean_obj_tag(x_467) == 0)
{
lean_object* x_468; 
lean_dec(x_465);
x_468 = lean_box(0);
return x_468;
}
else
{
lean_object* x_469; lean_object* x_470; lean_object* x_471; lean_object* x_472; 
x_469 = lean_ctor_get(x_467, 0);
lean_inc(x_469);
lean_dec_ref(x_467);
x_470 = lean_ctor_get(x_469, 0);
lean_inc(x_470);
x_471 = lean_ctor_get(x_469, 1);
lean_inc(x_471);
lean_dec(x_469);
x_472 = lp_iris_x2dkernel_IrisKernel_FFI_decodeCostBound(x_471);
if (lean_obj_tag(x_472) == 0)
{
lean_object* x_473; 
lean_dec(x_470);
lean_dec(x_465);
x_473 = lean_box(0);
return x_473;
}
else
{
lean_object* x_474; lean_object* x_475; lean_object* x_476; lean_object* x_477; lean_object* x_478; lean_object* x_479; lean_object* x_480; lean_object* x_481; 
x_474 = lean_ctor_get(x_472, 0);
lean_inc(x_474);
if (lean_is_exclusive(x_472)) {
 lean_ctor_release(x_472, 0);
 x_475 = x_472;
} else {
 lean_dec_ref(x_472);
 x_475 = lean_box(0);
}
x_476 = lean_ctor_get(x_474, 0);
lean_inc(x_476);
x_477 = lean_ctor_get(x_474, 1);
lean_inc(x_477);
if (lean_is_exclusive(x_474)) {
 lean_ctor_release(x_474, 0);
 lean_ctor_release(x_474, 1);
 x_478 = x_474;
} else {
 lean_dec_ref(x_474);
 x_478 = lean_box(0);
}
x_479 = lean_alloc_ctor(5, 3, 0);
lean_ctor_set(x_479, 0, x_465);
lean_ctor_set(x_479, 1, x_470);
lean_ctor_set(x_479, 2, x_476);
if (lean_is_scalar(x_478)) {
 x_480 = lean_alloc_ctor(0, 2, 0);
} else {
 x_480 = x_478;
}
lean_ctor_set(x_480, 0, x_479);
lean_ctor_set(x_480, 1, x_477);
if (lean_is_scalar(x_475)) {
 x_481 = lean_alloc_ctor(1, 1, 0);
} else {
 x_481 = x_475;
}
lean_ctor_set(x_481, 0, x_480);
return x_481;
}
}
}
}
}
else
{
lean_object* x_482; 
x_482 = lp_iris_x2dkernel_IrisKernel_FFI_decodeBoundVar(x_362);
if (lean_obj_tag(x_482) == 0)
{
lean_object* x_483; 
x_483 = lean_box(0);
return x_483;
}
else
{
lean_object* x_484; lean_object* x_485; lean_object* x_486; lean_object* x_487; lean_object* x_488; 
x_484 = lean_ctor_get(x_482, 0);
lean_inc(x_484);
lean_dec_ref(x_482);
x_485 = lean_ctor_get(x_484, 0);
lean_inc(x_485);
x_486 = lean_ctor_get(x_484, 1);
lean_inc(x_486);
if (lean_is_exclusive(x_484)) {
 lean_ctor_release(x_484, 0);
 lean_ctor_release(x_484, 1);
 x_487 = x_484;
} else {
 lean_dec_ref(x_484);
 x_487 = lean_box(0);
}
x_488 = lp_iris_x2dkernel_IrisKernel_FFI_decodeTypeId(x_486);
if (lean_obj_tag(x_488) == 0)
{
lean_object* x_489; 
lean_dec(x_487);
lean_dec(x_485);
x_489 = lean_box(0);
return x_489;
}
else
{
lean_object* x_490; lean_object* x_491; lean_object* x_492; lean_object* x_493; lean_object* x_494; lean_object* x_495; lean_object* x_496; lean_object* x_497; 
x_490 = lean_ctor_get(x_488, 0);
lean_inc(x_490);
if (lean_is_exclusive(x_488)) {
 lean_ctor_release(x_488, 0);
 x_491 = x_488;
} else {
 lean_dec_ref(x_488);
 x_491 = lean_box(0);
}
x_492 = lean_ctor_get(x_490, 0);
lean_inc(x_492);
x_493 = lean_ctor_get(x_490, 1);
lean_inc(x_493);
if (lean_is_exclusive(x_490)) {
 lean_ctor_release(x_490, 0);
 lean_ctor_release(x_490, 1);
 x_494 = x_490;
} else {
 lean_dec_ref(x_490);
 x_494 = lean_box(0);
}
if (lean_is_scalar(x_487)) {
 x_495 = lean_alloc_ctor(4, 2, 0);
} else {
 x_495 = x_487;
 lean_ctor_set_tag(x_495, 4);
}
lean_ctor_set(x_495, 0, x_485);
lean_ctor_set(x_495, 1, x_492);
if (lean_is_scalar(x_494)) {
 x_496 = lean_alloc_ctor(0, 2, 0);
} else {
 x_496 = x_494;
}
lean_ctor_set(x_496, 0, x_495);
lean_ctor_set(x_496, 1, x_493);
if (lean_is_scalar(x_491)) {
 x_497 = lean_alloc_ctor(1, 1, 0);
} else {
 x_497 = x_491;
}
lean_ctor_set(x_497, 0, x_496);
return x_497;
}
}
}
}
else
{
lean_object* x_498; 
x_498 = lp_iris_x2dkernel_IrisKernel_FFI_decodeBoundVar(x_362);
if (lean_obj_tag(x_498) == 0)
{
lean_object* x_499; 
x_499 = lean_box(0);
return x_499;
}
else
{
lean_object* x_500; lean_object* x_501; lean_object* x_502; lean_object* x_503; lean_object* x_504; 
x_500 = lean_ctor_get(x_498, 0);
lean_inc(x_500);
lean_dec_ref(x_498);
x_501 = lean_ctor_get(x_500, 0);
lean_inc(x_501);
x_502 = lean_ctor_get(x_500, 1);
lean_inc(x_502);
if (lean_is_exclusive(x_500)) {
 lean_ctor_release(x_500, 0);
 lean_ctor_release(x_500, 1);
 x_503 = x_500;
} else {
 lean_dec_ref(x_500);
 x_503 = lean_box(0);
}
x_504 = lp_iris_x2dkernel_IrisKernel_FFI_decodeTypeId(x_502);
if (lean_obj_tag(x_504) == 0)
{
lean_object* x_505; 
lean_dec(x_503);
lean_dec(x_501);
x_505 = lean_box(0);
return x_505;
}
else
{
lean_object* x_506; lean_object* x_507; lean_object* x_508; lean_object* x_509; lean_object* x_510; lean_object* x_511; lean_object* x_512; lean_object* x_513; 
x_506 = lean_ctor_get(x_504, 0);
lean_inc(x_506);
if (lean_is_exclusive(x_504)) {
 lean_ctor_release(x_504, 0);
 x_507 = x_504;
} else {
 lean_dec_ref(x_504);
 x_507 = lean_box(0);
}
x_508 = lean_ctor_get(x_506, 0);
lean_inc(x_508);
x_509 = lean_ctor_get(x_506, 1);
lean_inc(x_509);
if (lean_is_exclusive(x_506)) {
 lean_ctor_release(x_506, 0);
 lean_ctor_release(x_506, 1);
 x_510 = x_506;
} else {
 lean_dec_ref(x_506);
 x_510 = lean_box(0);
}
if (lean_is_scalar(x_503)) {
 x_511 = lean_alloc_ctor(3, 2, 0);
} else {
 x_511 = x_503;
 lean_ctor_set_tag(x_511, 3);
}
lean_ctor_set(x_511, 0, x_501);
lean_ctor_set(x_511, 1, x_508);
if (lean_is_scalar(x_510)) {
 x_512 = lean_alloc_ctor(0, 2, 0);
} else {
 x_512 = x_510;
}
lean_ctor_set(x_512, 0, x_511);
lean_ctor_set(x_512, 1, x_509);
if (lean_is_scalar(x_507)) {
 x_513 = lean_alloc_ctor(1, 1, 0);
} else {
 x_513 = x_507;
}
lean_ctor_set(x_513, 0, x_512);
return x_513;
}
}
}
}
else
{
lean_object* x_514; 
x_514 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readUInt16LE(x_362);
if (lean_obj_tag(x_514) == 0)
{
lean_object* x_515; 
x_515 = lean_box(0);
return x_515;
}
else
{
lean_object* x_516; lean_object* x_517; lean_object* x_518; lean_object* x_519; uint16_t x_520; lean_object* x_521; lean_object* x_522; lean_object* x_523; 
x_516 = lean_ctor_get(x_514, 0);
lean_inc(x_516);
if (lean_is_exclusive(x_514)) {
 lean_ctor_release(x_514, 0);
 x_517 = x_514;
} else {
 lean_dec_ref(x_514);
 x_517 = lean_box(0);
}
x_518 = lean_ctor_get(x_516, 0);
lean_inc(x_518);
x_519 = lean_ctor_get(x_516, 1);
lean_inc(x_519);
lean_dec(x_516);
x_520 = lean_unbox(x_518);
lean_dec(x_518);
x_521 = lean_uint16_to_nat(x_520);
x_522 = lean_box(0);
x_523 = lp_iris_x2dkernel_IrisKernel_FFI_decodeTypeDef_decodeTagTypeIdList(x_519, x_521, x_522);
if (lean_obj_tag(x_523) == 0)
{
lean_object* x_524; 
lean_dec(x_517);
x_524 = lean_box(0);
return x_524;
}
else
{
lean_object* x_525; lean_object* x_526; lean_object* x_527; lean_object* x_528; lean_object* x_529; lean_object* x_530; lean_object* x_531; lean_object* x_532; 
x_525 = lean_ctor_get(x_523, 0);
lean_inc(x_525);
if (lean_is_exclusive(x_523)) {
 lean_ctor_release(x_523, 0);
 x_526 = x_523;
} else {
 lean_dec_ref(x_523);
 x_526 = lean_box(0);
}
x_527 = lean_ctor_get(x_525, 0);
lean_inc(x_527);
x_528 = lean_ctor_get(x_525, 1);
lean_inc(x_528);
if (lean_is_exclusive(x_525)) {
 lean_ctor_release(x_525, 0);
 lean_ctor_release(x_525, 1);
 x_529 = x_525;
} else {
 lean_dec_ref(x_525);
 x_529 = lean_box(0);
}
if (lean_is_scalar(x_517)) {
 x_530 = lean_alloc_ctor(2, 1, 0);
} else {
 x_530 = x_517;
 lean_ctor_set_tag(x_530, 2);
}
lean_ctor_set(x_530, 0, x_527);
if (lean_is_scalar(x_529)) {
 x_531 = lean_alloc_ctor(0, 2, 0);
} else {
 x_531 = x_529;
}
lean_ctor_set(x_531, 0, x_530);
lean_ctor_set(x_531, 1, x_528);
if (lean_is_scalar(x_526)) {
 x_532 = lean_alloc_ctor(1, 1, 0);
} else {
 x_532 = x_526;
}
lean_ctor_set(x_532, 0, x_531);
return x_532;
}
}
}
}
else
{
lean_object* x_533; 
x_533 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readUInt16LE(x_362);
if (lean_obj_tag(x_533) == 0)
{
lean_object* x_534; 
x_534 = lean_box(0);
return x_534;
}
else
{
lean_object* x_535; lean_object* x_536; lean_object* x_537; lean_object* x_538; uint16_t x_539; lean_object* x_540; lean_object* x_541; lean_object* x_542; 
x_535 = lean_ctor_get(x_533, 0);
lean_inc(x_535);
if (lean_is_exclusive(x_533)) {
 lean_ctor_release(x_533, 0);
 x_536 = x_533;
} else {
 lean_dec_ref(x_533);
 x_536 = lean_box(0);
}
x_537 = lean_ctor_get(x_535, 0);
lean_inc(x_537);
x_538 = lean_ctor_get(x_535, 1);
lean_inc(x_538);
lean_dec(x_535);
x_539 = lean_unbox(x_537);
lean_dec(x_537);
x_540 = lean_uint16_to_nat(x_539);
x_541 = lean_box(0);
x_542 = lp_iris_x2dkernel_IrisKernel_FFI_decodeTypeDef_decodeTypeIdList(x_538, x_540, x_541);
if (lean_obj_tag(x_542) == 0)
{
lean_object* x_543; 
lean_dec(x_536);
x_543 = lean_box(0);
return x_543;
}
else
{
lean_object* x_544; lean_object* x_545; lean_object* x_546; lean_object* x_547; lean_object* x_548; lean_object* x_549; lean_object* x_550; lean_object* x_551; 
x_544 = lean_ctor_get(x_542, 0);
lean_inc(x_544);
if (lean_is_exclusive(x_542)) {
 lean_ctor_release(x_542, 0);
 x_545 = x_542;
} else {
 lean_dec_ref(x_542);
 x_545 = lean_box(0);
}
x_546 = lean_ctor_get(x_544, 0);
lean_inc(x_546);
x_547 = lean_ctor_get(x_544, 1);
lean_inc(x_547);
if (lean_is_exclusive(x_544)) {
 lean_ctor_release(x_544, 0);
 lean_ctor_release(x_544, 1);
 x_548 = x_544;
} else {
 lean_dec_ref(x_544);
 x_548 = lean_box(0);
}
if (lean_is_scalar(x_536)) {
 x_549 = lean_alloc_ctor(1, 1, 0);
} else {
 x_549 = x_536;
}
lean_ctor_set(x_549, 0, x_546);
if (lean_is_scalar(x_548)) {
 x_550 = lean_alloc_ctor(0, 2, 0);
} else {
 x_550 = x_548;
}
lean_ctor_set(x_550, 0, x_549);
lean_ctor_set(x_550, 1, x_547);
if (lean_is_scalar(x_545)) {
 x_551 = lean_alloc_ctor(1, 1, 0);
} else {
 x_551 = x_545;
}
lean_ctor_set(x_551, 0, x_550);
return x_551;
}
}
}
}
else
{
lean_object* x_552; 
x_552 = lp_iris_x2dkernel_IrisKernel_FFI_decodePrimType(x_362);
if (lean_obj_tag(x_552) == 0)
{
lean_object* x_553; 
x_553 = lean_box(0);
return x_553;
}
else
{
lean_object* x_554; lean_object* x_555; lean_object* x_556; lean_object* x_557; lean_object* x_558; lean_object* x_559; uint8_t x_560; lean_object* x_561; lean_object* x_562; 
x_554 = lean_ctor_get(x_552, 0);
lean_inc(x_554);
if (lean_is_exclusive(x_552)) {
 lean_ctor_release(x_552, 0);
 x_555 = x_552;
} else {
 lean_dec_ref(x_552);
 x_555 = lean_box(0);
}
x_556 = lean_ctor_get(x_554, 0);
lean_inc(x_556);
x_557 = lean_ctor_get(x_554, 1);
lean_inc(x_557);
if (lean_is_exclusive(x_554)) {
 lean_ctor_release(x_554, 0);
 lean_ctor_release(x_554, 1);
 x_558 = x_554;
} else {
 lean_dec_ref(x_554);
 x_558 = lean_box(0);
}
x_559 = lean_alloc_ctor(0, 0, 1);
x_560 = lean_unbox(x_556);
lean_dec(x_556);
lean_ctor_set_uint8(x_559, 0, x_560);
if (lean_is_scalar(x_558)) {
 x_561 = lean_alloc_ctor(0, 2, 0);
} else {
 x_561 = x_558;
}
lean_ctor_set(x_561, 0, x_559);
lean_ctor_set(x_561, 1, x_557);
if (lean_is_scalar(x_555)) {
 x_562 = lean_alloc_ctor(1, 1, 0);
} else {
 x_562 = x_555;
}
lean_ctor_set(x_562, 0, x_561);
return x_562;
}
}
}
}
}
}
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_decodeTypeEnv_decodeEntries(lean_object* x_1, lean_object* x_2, lean_object* x_3) {
_start:
{
lean_object* x_4; uint8_t x_5; 
x_4 = lean_unsigned_to_nat(0u);
x_5 = lean_nat_dec_eq(x_2, x_4);
if (x_5 == 1)
{
lean_object* x_6; lean_object* x_7; lean_object* x_8; 
lean_dec(x_2);
x_6 = l_List_reverse___redArg(x_3);
x_7 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_7, 0, x_6);
lean_ctor_set(x_7, 1, x_1);
x_8 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_8, 0, x_7);
return x_8;
}
else
{
lean_object* x_9; 
x_9 = lp_iris_x2dkernel_IrisKernel_FFI_decodeTypeId(x_1);
if (lean_obj_tag(x_9) == 0)
{
lean_object* x_10; 
lean_dec(x_3);
lean_dec(x_2);
x_10 = lean_box(0);
return x_10;
}
else
{
lean_object* x_11; uint8_t x_12; 
x_11 = lean_ctor_get(x_9, 0);
lean_inc(x_11);
lean_dec_ref(x_9);
x_12 = !lean_is_exclusive(x_11);
if (x_12 == 0)
{
lean_object* x_13; lean_object* x_14; lean_object* x_15; 
x_13 = lean_ctor_get(x_11, 0);
x_14 = lean_ctor_get(x_11, 1);
x_15 = lp_iris_x2dkernel_IrisKernel_FFI_decodeTypeDef(x_14);
if (lean_obj_tag(x_15) == 0)
{
lean_object* x_16; 
lean_free_object(x_11);
lean_dec(x_13);
lean_dec(x_3);
lean_dec(x_2);
x_16 = lean_box(0);
return x_16;
}
else
{
lean_object* x_17; uint8_t x_18; 
x_17 = lean_ctor_get(x_15, 0);
lean_inc(x_17);
lean_dec_ref(x_15);
x_18 = !lean_is_exclusive(x_17);
if (x_18 == 0)
{
lean_object* x_19; lean_object* x_20; lean_object* x_21; lean_object* x_22; 
x_19 = lean_ctor_get(x_17, 0);
x_20 = lean_ctor_get(x_17, 1);
x_21 = lean_unsigned_to_nat(1u);
x_22 = lean_nat_sub(x_2, x_21);
lean_dec(x_2);
lean_ctor_set(x_17, 1, x_19);
lean_ctor_set(x_17, 0, x_13);
lean_ctor_set_tag(x_11, 1);
lean_ctor_set(x_11, 1, x_3);
lean_ctor_set(x_11, 0, x_17);
x_1 = x_20;
x_2 = x_22;
x_3 = x_11;
goto _start;
}
else
{
lean_object* x_24; lean_object* x_25; lean_object* x_26; lean_object* x_27; lean_object* x_28; 
x_24 = lean_ctor_get(x_17, 0);
x_25 = lean_ctor_get(x_17, 1);
lean_inc(x_25);
lean_inc(x_24);
lean_dec(x_17);
x_26 = lean_unsigned_to_nat(1u);
x_27 = lean_nat_sub(x_2, x_26);
lean_dec(x_2);
x_28 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_28, 0, x_13);
lean_ctor_set(x_28, 1, x_24);
lean_ctor_set_tag(x_11, 1);
lean_ctor_set(x_11, 1, x_3);
lean_ctor_set(x_11, 0, x_28);
x_1 = x_25;
x_2 = x_27;
x_3 = x_11;
goto _start;
}
}
}
else
{
lean_object* x_30; lean_object* x_31; lean_object* x_32; 
x_30 = lean_ctor_get(x_11, 0);
x_31 = lean_ctor_get(x_11, 1);
lean_inc(x_31);
lean_inc(x_30);
lean_dec(x_11);
x_32 = lp_iris_x2dkernel_IrisKernel_FFI_decodeTypeDef(x_31);
if (lean_obj_tag(x_32) == 0)
{
lean_object* x_33; 
lean_dec(x_30);
lean_dec(x_3);
lean_dec(x_2);
x_33 = lean_box(0);
return x_33;
}
else
{
lean_object* x_34; lean_object* x_35; lean_object* x_36; lean_object* x_37; lean_object* x_38; lean_object* x_39; lean_object* x_40; lean_object* x_41; 
x_34 = lean_ctor_get(x_32, 0);
lean_inc(x_34);
lean_dec_ref(x_32);
x_35 = lean_ctor_get(x_34, 0);
lean_inc(x_35);
x_36 = lean_ctor_get(x_34, 1);
lean_inc(x_36);
if (lean_is_exclusive(x_34)) {
 lean_ctor_release(x_34, 0);
 lean_ctor_release(x_34, 1);
 x_37 = x_34;
} else {
 lean_dec_ref(x_34);
 x_37 = lean_box(0);
}
x_38 = lean_unsigned_to_nat(1u);
x_39 = lean_nat_sub(x_2, x_38);
lean_dec(x_2);
if (lean_is_scalar(x_37)) {
 x_40 = lean_alloc_ctor(0, 2, 0);
} else {
 x_40 = x_37;
}
lean_ctor_set(x_40, 0, x_30);
lean_ctor_set(x_40, 1, x_35);
x_41 = lean_alloc_ctor(1, 2, 0);
lean_ctor_set(x_41, 0, x_40);
lean_ctor_set(x_41, 1, x_3);
x_1 = x_36;
x_2 = x_39;
x_3 = x_41;
goto _start;
}
}
}
}
}
}
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_decodeTypeEnv(lean_object* x_1) {
_start:
{
lean_object* x_2; 
x_2 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readUInt16LE(x_1);
if (lean_obj_tag(x_2) == 0)
{
lean_object* x_3; 
x_3 = lean_box(0);
return x_3;
}
else
{
lean_object* x_4; lean_object* x_5; lean_object* x_6; uint16_t x_7; lean_object* x_8; lean_object* x_9; lean_object* x_10; 
x_4 = lean_ctor_get(x_2, 0);
lean_inc(x_4);
lean_dec_ref(x_2);
x_5 = lean_ctor_get(x_4, 0);
lean_inc(x_5);
x_6 = lean_ctor_get(x_4, 1);
lean_inc(x_6);
lean_dec(x_4);
x_7 = lean_unbox(x_5);
lean_dec(x_5);
x_8 = lean_uint16_to_nat(x_7);
x_9 = lean_box(0);
x_10 = lp_iris_x2dkernel_IrisKernel_FFI_decodeTypeEnv_decodeEntries(x_6, x_8, x_9);
return x_10;
}
}
}
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_decodeJudgment(lean_object* x_1) {
_start:
{
lean_object* x_2; 
x_2 = lp_iris_x2dkernel_IrisKernel_FFI_decodeContext(x_1);
if (lean_obj_tag(x_2) == 0)
{
lean_object* x_3; 
x_3 = lean_box(0);
return x_3;
}
else
{
lean_object* x_4; lean_object* x_5; lean_object* x_6; lean_object* x_7; 
x_4 = lean_ctor_get(x_2, 0);
lean_inc(x_4);
lean_dec_ref(x_2);
x_5 = lean_ctor_get(x_4, 0);
lean_inc(x_5);
x_6 = lean_ctor_get(x_4, 1);
lean_inc(x_6);
lean_dec(x_4);
x_7 = lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeId(x_6);
if (lean_obj_tag(x_7) == 0)
{
lean_object* x_8; 
lean_dec(x_5);
x_8 = lean_box(0);
return x_8;
}
else
{
lean_object* x_9; lean_object* x_10; lean_object* x_11; lean_object* x_12; 
x_9 = lean_ctor_get(x_7, 0);
lean_inc(x_9);
lean_dec_ref(x_7);
x_10 = lean_ctor_get(x_9, 0);
lean_inc(x_10);
x_11 = lean_ctor_get(x_9, 1);
lean_inc(x_11);
lean_dec(x_9);
x_12 = lp_iris_x2dkernel_IrisKernel_FFI_decodeTypeId(x_11);
if (lean_obj_tag(x_12) == 0)
{
lean_object* x_13; 
lean_dec(x_10);
lean_dec(x_5);
x_13 = lean_box(0);
return x_13;
}
else
{
lean_object* x_14; lean_object* x_15; lean_object* x_16; lean_object* x_17; 
x_14 = lean_ctor_get(x_12, 0);
lean_inc(x_14);
lean_dec_ref(x_12);
x_15 = lean_ctor_get(x_14, 0);
lean_inc(x_15);
x_16 = lean_ctor_get(x_14, 1);
lean_inc(x_16);
lean_dec(x_14);
x_17 = lp_iris_x2dkernel_IrisKernel_FFI_decodeCostBound(x_16);
if (lean_obj_tag(x_17) == 0)
{
lean_object* x_18; 
lean_dec(x_15);
lean_dec(x_10);
lean_dec(x_5);
x_18 = lean_box(0);
return x_18;
}
else
{
uint8_t x_19; 
x_19 = !lean_is_exclusive(x_17);
if (x_19 == 0)
{
lean_object* x_20; uint8_t x_21; 
x_20 = lean_ctor_get(x_17, 0);
x_21 = !lean_is_exclusive(x_20);
if (x_21 == 0)
{
lean_object* x_22; lean_object* x_23; 
x_22 = lean_ctor_get(x_20, 0);
x_23 = lean_alloc_ctor(0, 4, 0);
lean_ctor_set(x_23, 0, x_5);
lean_ctor_set(x_23, 1, x_10);
lean_ctor_set(x_23, 2, x_15);
lean_ctor_set(x_23, 3, x_22);
lean_ctor_set(x_20, 0, x_23);
return x_17;
}
else
{
lean_object* x_24; lean_object* x_25; lean_object* x_26; lean_object* x_27; 
x_24 = lean_ctor_get(x_20, 0);
x_25 = lean_ctor_get(x_20, 1);
lean_inc(x_25);
lean_inc(x_24);
lean_dec(x_20);
x_26 = lean_alloc_ctor(0, 4, 0);
lean_ctor_set(x_26, 0, x_5);
lean_ctor_set(x_26, 1, x_10);
lean_ctor_set(x_26, 2, x_15);
lean_ctor_set(x_26, 3, x_24);
x_27 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_27, 0, x_26);
lean_ctor_set(x_27, 1, x_25);
lean_ctor_set(x_17, 0, x_27);
return x_17;
}
}
else
{
lean_object* x_28; lean_object* x_29; lean_object* x_30; lean_object* x_31; lean_object* x_32; lean_object* x_33; lean_object* x_34; 
x_28 = lean_ctor_get(x_17, 0);
lean_inc(x_28);
lean_dec(x_17);
x_29 = lean_ctor_get(x_28, 0);
lean_inc(x_29);
x_30 = lean_ctor_get(x_28, 1);
lean_inc(x_30);
if (lean_is_exclusive(x_28)) {
 lean_ctor_release(x_28, 0);
 lean_ctor_release(x_28, 1);
 x_31 = x_28;
} else {
 lean_dec_ref(x_28);
 x_31 = lean_box(0);
}
x_32 = lean_alloc_ctor(0, 4, 0);
lean_ctor_set(x_32, 0, x_5);
lean_ctor_set(x_32, 1, x_10);
lean_ctor_set(x_32, 2, x_15);
lean_ctor_set(x_32, 3, x_29);
if (lean_is_scalar(x_31)) {
 x_33 = lean_alloc_ctor(0, 2, 0);
} else {
 x_33 = x_31;
}
lean_ctor_set(x_33, 0, x_32);
lean_ctor_set(x_33, 1, x_30);
x_34 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_34, 0, x_33);
return x_34;
}
}
}
}
}
}
}
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_Cursor_writeUInt8(lean_object* x_1, uint8_t x_2) {
_start:
{
lean_object* x_3; 
x_3 = lean_byte_array_push(x_1, x_2);
return x_3;
}
}
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_Cursor_writeUInt8___boxed(lean_object* x_1, lean_object* x_2) {
_start:
{
uint8_t x_3; lean_object* x_4; 
x_3 = lean_unbox(x_2);
x_4 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_writeUInt8(x_1, x_3);
return x_4;
}
}
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_Cursor_writeUInt16LE(lean_object* x_1, uint16_t x_2) {
_start:
{
uint8_t x_3; lean_object* x_4; uint16_t x_5; uint16_t x_6; uint8_t x_7; lean_object* x_8; 
x_3 = lean_uint16_to_uint8(x_2);
x_4 = lean_byte_array_push(x_1, x_3);
x_5 = 8;
x_6 = lean_uint16_shift_right(x_2, x_5);
x_7 = lean_uint16_to_uint8(x_6);
x_8 = lean_byte_array_push(x_4, x_7);
return x_8;
}
}
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_Cursor_writeUInt16LE___boxed(lean_object* x_1, lean_object* x_2) {
_start:
{
uint16_t x_3; lean_object* x_4; 
x_3 = lean_unbox(x_2);
x_4 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_writeUInt16LE(x_1, x_3);
return x_4;
}
}
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_Cursor_writeUInt32LE(lean_object* x_1, uint32_t x_2) {
_start:
{
uint8_t x_3; lean_object* x_4; uint32_t x_5; uint32_t x_6; uint8_t x_7; lean_object* x_8; uint32_t x_9; uint32_t x_10; uint8_t x_11; lean_object* x_12; uint32_t x_13; uint32_t x_14; uint8_t x_15; lean_object* x_16; 
x_3 = lean_uint32_to_uint8(x_2);
x_4 = lean_byte_array_push(x_1, x_3);
x_5 = 8;
x_6 = lean_uint32_shift_right(x_2, x_5);
x_7 = lean_uint32_to_uint8(x_6);
x_8 = lean_byte_array_push(x_4, x_7);
x_9 = 16;
x_10 = lean_uint32_shift_right(x_2, x_9);
x_11 = lean_uint32_to_uint8(x_10);
x_12 = lean_byte_array_push(x_8, x_11);
x_13 = 24;
x_14 = lean_uint32_shift_right(x_2, x_13);
x_15 = lean_uint32_to_uint8(x_14);
x_16 = lean_byte_array_push(x_12, x_15);
return x_16;
}
}
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_Cursor_writeUInt32LE___boxed(lean_object* x_1, lean_object* x_2) {
_start:
{
uint32_t x_3; lean_object* x_4; 
x_3 = lean_unbox_uint32(x_2);
lean_dec(x_2);
x_4 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_writeUInt32LE(x_1, x_3);
return x_4;
}
}
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_Cursor_writeUInt64LE(lean_object* x_1, uint64_t x_2) {
_start:
{
uint8_t x_3; lean_object* x_4; uint64_t x_5; uint64_t x_6; uint8_t x_7; lean_object* x_8; uint64_t x_9; uint64_t x_10; uint8_t x_11; lean_object* x_12; uint64_t x_13; uint64_t x_14; uint8_t x_15; lean_object* x_16; uint64_t x_17; uint64_t x_18; uint8_t x_19; lean_object* x_20; uint64_t x_21; uint64_t x_22; uint8_t x_23; lean_object* x_24; uint64_t x_25; uint64_t x_26; uint8_t x_27; lean_object* x_28; uint64_t x_29; uint64_t x_30; uint8_t x_31; lean_object* x_32; 
x_3 = lean_uint64_to_uint8(x_2);
x_4 = lean_byte_array_push(x_1, x_3);
x_5 = 8;
x_6 = lean_uint64_shift_right(x_2, x_5);
x_7 = lean_uint64_to_uint8(x_6);
x_8 = lean_byte_array_push(x_4, x_7);
x_9 = 16;
x_10 = lean_uint64_shift_right(x_2, x_9);
x_11 = lean_uint64_to_uint8(x_10);
x_12 = lean_byte_array_push(x_8, x_11);
x_13 = 24;
x_14 = lean_uint64_shift_right(x_2, x_13);
x_15 = lean_uint64_to_uint8(x_14);
x_16 = lean_byte_array_push(x_12, x_15);
x_17 = 32;
x_18 = lean_uint64_shift_right(x_2, x_17);
x_19 = lean_uint64_to_uint8(x_18);
x_20 = lean_byte_array_push(x_16, x_19);
x_21 = 40;
x_22 = lean_uint64_shift_right(x_2, x_21);
x_23 = lean_uint64_to_uint8(x_22);
x_24 = lean_byte_array_push(x_20, x_23);
x_25 = 48;
x_26 = lean_uint64_shift_right(x_2, x_25);
x_27 = lean_uint64_to_uint8(x_26);
x_28 = lean_byte_array_push(x_24, x_27);
x_29 = 56;
x_30 = lean_uint64_shift_right(x_2, x_29);
x_31 = lean_uint64_to_uint8(x_30);
x_32 = lean_byte_array_push(x_28, x_31);
return x_32;
}
}
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_Cursor_writeUInt64LE___boxed(lean_object* x_1, lean_object* x_2) {
_start:
{
uint64_t x_3; lean_object* x_4; 
x_3 = lean_unbox_uint64(x_2);
lean_dec(x_2);
x_4 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_writeUInt64LE(x_1, x_3);
return x_4;
}
}
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_encodeNodeId(lean_object* x_1, lean_object* x_2) {
_start:
{
uint64_t x_3; lean_object* x_4; 
x_3 = lean_uint64_of_nat(x_2);
x_4 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_writeUInt64LE(x_1, x_3);
return x_4;
}
}
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_encodeNodeId___boxed(lean_object* x_1, lean_object* x_2) {
_start:
{
lean_object* x_3; 
x_3 = lp_iris_x2dkernel_IrisKernel_FFI_encodeNodeId(x_1, x_2);
lean_dec(x_2);
return x_3;
}
}
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_encodeTypeId(lean_object* x_1, lean_object* x_2) {
_start:
{
uint64_t x_3; lean_object* x_4; 
x_3 = lean_uint64_of_nat(x_2);
x_4 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_writeUInt64LE(x_1, x_3);
return x_4;
}
}
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_encodeTypeId___boxed(lean_object* x_1, lean_object* x_2) {
_start:
{
lean_object* x_3; 
x_3 = lp_iris_x2dkernel_IrisKernel_FFI_encodeTypeId(x_1, x_2);
lean_dec(x_2);
return x_3;
}
}
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_encodeBinderId(lean_object* x_1, lean_object* x_2) {
_start:
{
uint32_t x_3; lean_object* x_4; 
x_3 = lean_uint32_of_nat(x_2);
x_4 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_writeUInt32LE(x_1, x_3);
return x_4;
}
}
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_encodeBinderId___boxed(lean_object* x_1, lean_object* x_2) {
_start:
{
lean_object* x_3; 
x_3 = lp_iris_x2dkernel_IrisKernel_FFI_encodeBinderId(x_1, x_2);
lean_dec(x_2);
return x_3;
}
}
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_encodeCostBound(lean_object* x_1, lean_object* x_2) {
_start:
{
switch (lean_obj_tag(x_2)) {
case 0:
{
uint8_t x_3; lean_object* x_4; 
x_3 = 0;
x_4 = lean_byte_array_push(x_1, x_3);
return x_4;
}
case 1:
{
uint8_t x_5; lean_object* x_6; 
x_5 = 1;
x_6 = lean_byte_array_push(x_1, x_5);
return x_6;
}
case 2:
{
lean_object* x_7; uint8_t x_8; lean_object* x_9; uint64_t x_10; lean_object* x_11; 
x_7 = lean_ctor_get(x_2, 0);
x_8 = 2;
x_9 = lean_byte_array_push(x_1, x_8);
x_10 = lean_uint64_of_nat(x_7);
x_11 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_writeUInt64LE(x_9, x_10);
return x_11;
}
case 3:
{
lean_object* x_12; uint8_t x_13; lean_object* x_14; uint32_t x_15; lean_object* x_16; 
x_12 = lean_ctor_get(x_2, 0);
x_13 = 3;
x_14 = lean_byte_array_push(x_1, x_13);
x_15 = lean_uint32_of_nat(x_12);
x_16 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_writeUInt32LE(x_14, x_15);
return x_16;
}
case 4:
{
lean_object* x_17; uint8_t x_18; lean_object* x_19; uint32_t x_20; lean_object* x_21; 
x_17 = lean_ctor_get(x_2, 0);
x_18 = 4;
x_19 = lean_byte_array_push(x_1, x_18);
x_20 = lean_uint32_of_nat(x_17);
x_21 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_writeUInt32LE(x_19, x_20);
return x_21;
}
case 5:
{
lean_object* x_22; lean_object* x_23; uint8_t x_24; lean_object* x_25; uint32_t x_26; lean_object* x_27; uint32_t x_28; lean_object* x_29; 
x_22 = lean_ctor_get(x_2, 0);
x_23 = lean_ctor_get(x_2, 1);
x_24 = 5;
x_25 = lean_byte_array_push(x_1, x_24);
x_26 = lean_uint32_of_nat(x_22);
x_27 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_writeUInt32LE(x_25, x_26);
x_28 = lean_uint32_of_nat(x_23);
x_29 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_writeUInt32LE(x_27, x_28);
return x_29;
}
case 6:
{
lean_object* x_30; lean_object* x_31; uint8_t x_32; lean_object* x_33; lean_object* x_34; 
x_30 = lean_ctor_get(x_2, 0);
x_31 = lean_ctor_get(x_2, 1);
x_32 = 6;
x_33 = lean_byte_array_push(x_1, x_32);
x_34 = lp_iris_x2dkernel_IrisKernel_FFI_encodeCostBound(x_33, x_30);
x_1 = x_34;
x_2 = x_31;
goto _start;
}
case 7:
{
lean_object* x_36; lean_object* x_37; uint8_t x_38; lean_object* x_39; lean_object* x_40; 
x_36 = lean_ctor_get(x_2, 0);
x_37 = lean_ctor_get(x_2, 1);
x_38 = 7;
x_39 = lean_byte_array_push(x_1, x_38);
x_40 = lp_iris_x2dkernel_IrisKernel_FFI_encodeCostBound(x_39, x_36);
x_1 = x_40;
x_2 = x_37;
goto _start;
}
case 8:
{
lean_object* x_42; lean_object* x_43; uint8_t x_44; lean_object* x_45; lean_object* x_46; 
x_42 = lean_ctor_get(x_2, 0);
x_43 = lean_ctor_get(x_2, 1);
x_44 = 8;
x_45 = lean_byte_array_push(x_1, x_44);
x_46 = lp_iris_x2dkernel_IrisKernel_FFI_encodeCostBound(x_45, x_42);
x_1 = x_46;
x_2 = x_43;
goto _start;
}
case 9:
{
lean_object* x_48; uint8_t x_49; lean_object* x_50; lean_object* x_51; uint16_t x_52; lean_object* x_53; lean_object* x_54; 
x_48 = lean_ctor_get(x_2, 0);
x_49 = 9;
x_50 = lean_byte_array_push(x_1, x_49);
x_51 = l_List_lengthTR___redArg(x_48);
x_52 = lean_uint16_of_nat(x_51);
lean_dec(x_51);
x_53 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_writeUInt16LE(x_50, x_52);
x_54 = lp_iris_x2dkernel_List_foldl___at___00IrisKernel_FFI_encodeCostBound_spec__0(x_53, x_48);
return x_54;
}
case 10:
{
lean_object* x_55; uint8_t x_56; lean_object* x_57; lean_object* x_58; uint16_t x_59; lean_object* x_60; lean_object* x_61; 
x_55 = lean_ctor_get(x_2, 0);
x_56 = 10;
x_57 = lean_byte_array_push(x_1, x_56);
x_58 = l_List_lengthTR___redArg(x_55);
x_59 = lean_uint16_of_nat(x_58);
lean_dec(x_58);
x_60 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_writeUInt16LE(x_57, x_59);
x_61 = lp_iris_x2dkernel_List_foldl___at___00IrisKernel_FFI_encodeCostBound_spec__0(x_60, x_55);
return x_61;
}
case 11:
{
lean_object* x_62; uint8_t x_63; lean_object* x_64; 
x_62 = lean_ctor_get(x_2, 0);
x_63 = 11;
x_64 = lean_byte_array_push(x_1, x_63);
x_1 = x_64;
x_2 = x_62;
goto _start;
}
default: 
{
lean_object* x_66; uint8_t x_67; lean_object* x_68; 
x_66 = lean_ctor_get(x_2, 0);
x_67 = 12;
x_68 = lean_byte_array_push(x_1, x_67);
x_1 = x_68;
x_2 = x_66;
goto _start;
}
}
}
}
LEAN_EXPORT lean_object* lp_iris_x2dkernel_List_foldl___at___00IrisKernel_FFI_encodeCostBound_spec__0(lean_object* x_1, lean_object* x_2) {
_start:
{
if (lean_obj_tag(x_2) == 0)
{
return x_1;
}
else
{
lean_object* x_3; lean_object* x_4; lean_object* x_5; 
x_3 = lean_ctor_get(x_2, 0);
x_4 = lean_ctor_get(x_2, 1);
x_5 = lp_iris_x2dkernel_IrisKernel_FFI_encodeCostBound(x_1, x_3);
x_1 = x_5;
x_2 = x_4;
goto _start;
}
}
}
LEAN_EXPORT lean_object* lp_iris_x2dkernel_List_foldl___at___00IrisKernel_FFI_encodeCostBound_spec__0___boxed(lean_object* x_1, lean_object* x_2) {
_start:
{
lean_object* x_3; 
x_3 = lp_iris_x2dkernel_List_foldl___at___00IrisKernel_FFI_encodeCostBound_spec__0(x_1, x_2);
lean_dec(x_2);
return x_3;
}
}
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_encodeCostBound___boxed(lean_object* x_1, lean_object* x_2) {
_start:
{
lean_object* x_3; 
x_3 = lp_iris_x2dkernel_IrisKernel_FFI_encodeCostBound(x_1, x_2);
lean_dec(x_2);
return x_3;
}
}
LEAN_EXPORT lean_object* lp_iris_x2dkernel_List_foldl___at___00IrisKernel_FFI_encodeContext_spec__0(lean_object* x_1, lean_object* x_2) {
_start:
{
if (lean_obj_tag(x_2) == 0)
{
return x_1;
}
else
{
lean_object* x_3; lean_object* x_4; lean_object* x_5; lean_object* x_6; lean_object* x_7; lean_object* x_8; 
x_3 = lean_ctor_get(x_2, 0);
x_4 = lean_ctor_get(x_2, 1);
x_5 = lean_ctor_get(x_3, 0);
x_6 = lean_ctor_get(x_3, 1);
x_7 = lp_iris_x2dkernel_IrisKernel_FFI_encodeBinderId(x_1, x_5);
x_8 = lp_iris_x2dkernel_IrisKernel_FFI_encodeTypeId(x_7, x_6);
x_1 = x_8;
x_2 = x_4;
goto _start;
}
}
}
LEAN_EXPORT lean_object* lp_iris_x2dkernel_List_foldl___at___00IrisKernel_FFI_encodeContext_spec__0___boxed(lean_object* x_1, lean_object* x_2) {
_start:
{
lean_object* x_3; 
x_3 = lp_iris_x2dkernel_List_foldl___at___00IrisKernel_FFI_encodeContext_spec__0(x_1, x_2);
lean_dec(x_2);
return x_3;
}
}
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_encodeContext(lean_object* x_1, lean_object* x_2) {
_start:
{
lean_object* x_3; uint16_t x_4; lean_object* x_5; lean_object* x_6; 
x_3 = l_List_lengthTR___redArg(x_2);
x_4 = lean_uint16_of_nat(x_3);
lean_dec(x_3);
x_5 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_writeUInt16LE(x_1, x_4);
x_6 = lp_iris_x2dkernel_List_foldl___at___00IrisKernel_FFI_encodeContext_spec__0(x_5, x_2);
return x_6;
}
}
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_encodeContext___boxed(lean_object* x_1, lean_object* x_2) {
_start:
{
lean_object* x_3; 
x_3 = lp_iris_x2dkernel_IrisKernel_FFI_encodeContext(x_1, x_2);
lean_dec(x_2);
return x_3;
}
}
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_encodeJudgment(lean_object* x_1, lean_object* x_2) {
_start:
{
lean_object* x_3; lean_object* x_4; lean_object* x_5; lean_object* x_6; lean_object* x_7; lean_object* x_8; lean_object* x_9; lean_object* x_10; 
x_3 = lean_ctor_get(x_2, 0);
x_4 = lean_ctor_get(x_2, 1);
x_5 = lean_ctor_get(x_2, 2);
x_6 = lean_ctor_get(x_2, 3);
x_7 = lp_iris_x2dkernel_IrisKernel_FFI_encodeContext(x_1, x_3);
x_8 = lp_iris_x2dkernel_IrisKernel_FFI_encodeNodeId(x_7, x_4);
x_9 = lp_iris_x2dkernel_IrisKernel_FFI_encodeTypeId(x_8, x_5);
x_10 = lp_iris_x2dkernel_IrisKernel_FFI_encodeCostBound(x_9, x_6);
return x_10;
}
}
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_encodeJudgment___boxed(lean_object* x_1, lean_object* x_2) {
_start:
{
lean_object* x_3; 
x_3 = lp_iris_x2dkernel_IrisKernel_FFI_encodeJudgment(x_1, x_2);
lean_dec_ref(x_2);
return x_3;
}
}
static lean_object* _init_lp_iris_x2dkernel_IrisKernel_FFI_encodeSuccess___closed__0() {
_start:
{
uint8_t x_1; lean_object* x_2; lean_object* x_3; 
x_1 = 1;
x_2 = l_ByteArray_empty;
x_3 = lean_byte_array_push(x_2, x_1);
return x_3;
}
}
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_encodeSuccess(lean_object* x_1) {
_start:
{
lean_object* x_2; lean_object* x_3; 
x_2 = lp_iris_x2dkernel_IrisKernel_FFI_encodeSuccess___closed__0;
x_3 = lp_iris_x2dkernel_IrisKernel_FFI_encodeJudgment(x_2, x_1);
return x_3;
}
}
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_encodeSuccess___boxed(lean_object* x_1) {
_start:
{
lean_object* x_2; 
x_2 = lp_iris_x2dkernel_IrisKernel_FFI_encodeSuccess(x_1);
lean_dec_ref(x_1);
return x_2;
}
}
static lean_object* _init_lp_iris_x2dkernel_IrisKernel_FFI_encodeFailure___closed__0() {
_start:
{
uint8_t x_1; lean_object* x_2; lean_object* x_3; 
x_1 = 0;
x_2 = l_ByteArray_empty;
x_3 = lean_byte_array_push(x_2, x_1);
return x_3;
}
}
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_encodeFailure(uint8_t x_1) {
_start:
{
lean_object* x_2; lean_object* x_3; 
x_2 = lp_iris_x2dkernel_IrisKernel_FFI_encodeFailure___closed__0;
x_3 = lean_byte_array_push(x_2, x_1);
return x_3;
}
}
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_encodeFailure___boxed(lean_object* x_1) {
_start:
{
uint8_t x_2; lean_object* x_3; 
x_2 = lean_unbox(x_1);
x_3 = lp_iris_x2dkernel_IrisKernel_FFI_encodeFailure(x_2);
return x_3;
}
}
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_decodeLIAFormula(lean_object* x_1) {
_start:
{
lean_object* x_2; 
x_2 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readUInt8(x_1);
if (lean_obj_tag(x_2) == 0)
{
lean_object* x_3; 
x_3 = lean_box(0);
return x_3;
}
else
{
uint8_t x_4; 
x_4 = !lean_is_exclusive(x_2);
if (x_4 == 0)
{
lean_object* x_5; uint8_t x_6; 
x_5 = lean_ctor_get(x_2, 0);
x_6 = !lean_is_exclusive(x_5);
if (x_6 == 0)
{
lean_object* x_7; lean_object* x_8; uint8_t x_9; lean_object* x_10; lean_object* x_11; uint8_t x_12; 
x_7 = lean_ctor_get(x_5, 0);
x_8 = lean_ctor_get(x_5, 1);
x_9 = lean_unbox(x_7);
lean_dec(x_7);
x_10 = lean_uint8_to_nat(x_9);
x_11 = lean_unsigned_to_nat(0u);
x_12 = lean_nat_dec_eq(x_10, x_11);
if (x_12 == 0)
{
lean_object* x_13; uint8_t x_14; 
x_13 = lean_unsigned_to_nat(1u);
x_14 = lean_nat_dec_eq(x_10, x_13);
if (x_14 == 0)
{
lean_object* x_15; uint8_t x_16; 
lean_free_object(x_5);
lean_free_object(x_2);
x_15 = lean_unsigned_to_nat(2u);
x_16 = lean_nat_dec_eq(x_10, x_15);
if (x_16 == 0)
{
lean_object* x_17; uint8_t x_18; 
x_17 = lean_unsigned_to_nat(3u);
x_18 = lean_nat_dec_eq(x_10, x_17);
if (x_18 == 0)
{
lean_object* x_19; uint8_t x_20; 
x_19 = lean_unsigned_to_nat(4u);
x_20 = lean_nat_dec_eq(x_10, x_19);
if (x_20 == 0)
{
lean_object* x_21; uint8_t x_22; 
x_21 = lean_unsigned_to_nat(5u);
x_22 = lean_nat_dec_eq(x_10, x_21);
if (x_22 == 0)
{
lean_object* x_23; uint8_t x_24; 
x_23 = lean_unsigned_to_nat(6u);
x_24 = lean_nat_dec_eq(x_10, x_23);
if (x_24 == 0)
{
lean_object* x_25; 
lean_dec(x_8);
x_25 = lean_box(0);
return x_25;
}
else
{
lean_object* x_26; 
x_26 = lp_iris_x2dkernel_IrisKernel_FFI_decodeLIAAtom(x_8);
if (lean_obj_tag(x_26) == 0)
{
lean_object* x_27; 
x_27 = lean_box(0);
return x_27;
}
else
{
uint8_t x_28; 
x_28 = !lean_is_exclusive(x_26);
if (x_28 == 0)
{
lean_object* x_29; uint8_t x_30; 
x_29 = lean_ctor_get(x_26, 0);
x_30 = !lean_is_exclusive(x_29);
if (x_30 == 0)
{
lean_object* x_31; lean_object* x_32; 
x_31 = lean_ctor_get(x_29, 0);
x_32 = lean_alloc_ctor(6, 1, 0);
lean_ctor_set(x_32, 0, x_31);
lean_ctor_set(x_29, 0, x_32);
return x_26;
}
else
{
lean_object* x_33; lean_object* x_34; lean_object* x_35; lean_object* x_36; 
x_33 = lean_ctor_get(x_29, 0);
x_34 = lean_ctor_get(x_29, 1);
lean_inc(x_34);
lean_inc(x_33);
lean_dec(x_29);
x_35 = lean_alloc_ctor(6, 1, 0);
lean_ctor_set(x_35, 0, x_33);
x_36 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_36, 0, x_35);
lean_ctor_set(x_36, 1, x_34);
lean_ctor_set(x_26, 0, x_36);
return x_26;
}
}
else
{
lean_object* x_37; lean_object* x_38; lean_object* x_39; lean_object* x_40; lean_object* x_41; lean_object* x_42; lean_object* x_43; 
x_37 = lean_ctor_get(x_26, 0);
lean_inc(x_37);
lean_dec(x_26);
x_38 = lean_ctor_get(x_37, 0);
lean_inc(x_38);
x_39 = lean_ctor_get(x_37, 1);
lean_inc(x_39);
if (lean_is_exclusive(x_37)) {
 lean_ctor_release(x_37, 0);
 lean_ctor_release(x_37, 1);
 x_40 = x_37;
} else {
 lean_dec_ref(x_37);
 x_40 = lean_box(0);
}
x_41 = lean_alloc_ctor(6, 1, 0);
lean_ctor_set(x_41, 0, x_38);
if (lean_is_scalar(x_40)) {
 x_42 = lean_alloc_ctor(0, 2, 0);
} else {
 x_42 = x_40;
}
lean_ctor_set(x_42, 0, x_41);
lean_ctor_set(x_42, 1, x_39);
x_43 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_43, 0, x_42);
return x_43;
}
}
}
}
else
{
lean_object* x_44; 
x_44 = lp_iris_x2dkernel_IrisKernel_FFI_decodeLIAFormula(x_8);
if (lean_obj_tag(x_44) == 0)
{
return x_44;
}
else
{
lean_object* x_45; uint8_t x_46; 
x_45 = lean_ctor_get(x_44, 0);
lean_inc(x_45);
lean_dec_ref(x_44);
x_46 = !lean_is_exclusive(x_45);
if (x_46 == 0)
{
lean_object* x_47; lean_object* x_48; lean_object* x_49; 
x_47 = lean_ctor_get(x_45, 0);
x_48 = lean_ctor_get(x_45, 1);
x_49 = lp_iris_x2dkernel_IrisKernel_FFI_decodeLIAFormula(x_48);
if (lean_obj_tag(x_49) == 0)
{
lean_free_object(x_45);
lean_dec(x_47);
return x_49;
}
else
{
uint8_t x_50; 
x_50 = !lean_is_exclusive(x_49);
if (x_50 == 0)
{
lean_object* x_51; uint8_t x_52; 
x_51 = lean_ctor_get(x_49, 0);
x_52 = !lean_is_exclusive(x_51);
if (x_52 == 0)
{
lean_object* x_53; 
x_53 = lean_ctor_get(x_51, 0);
lean_ctor_set_tag(x_45, 5);
lean_ctor_set(x_45, 1, x_53);
lean_ctor_set(x_51, 0, x_45);
return x_49;
}
else
{
lean_object* x_54; lean_object* x_55; lean_object* x_56; 
x_54 = lean_ctor_get(x_51, 0);
x_55 = lean_ctor_get(x_51, 1);
lean_inc(x_55);
lean_inc(x_54);
lean_dec(x_51);
lean_ctor_set_tag(x_45, 5);
lean_ctor_set(x_45, 1, x_54);
x_56 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_56, 0, x_45);
lean_ctor_set(x_56, 1, x_55);
lean_ctor_set(x_49, 0, x_56);
return x_49;
}
}
else
{
lean_object* x_57; lean_object* x_58; lean_object* x_59; lean_object* x_60; lean_object* x_61; lean_object* x_62; 
x_57 = lean_ctor_get(x_49, 0);
lean_inc(x_57);
lean_dec(x_49);
x_58 = lean_ctor_get(x_57, 0);
lean_inc(x_58);
x_59 = lean_ctor_get(x_57, 1);
lean_inc(x_59);
if (lean_is_exclusive(x_57)) {
 lean_ctor_release(x_57, 0);
 lean_ctor_release(x_57, 1);
 x_60 = x_57;
} else {
 lean_dec_ref(x_57);
 x_60 = lean_box(0);
}
lean_ctor_set_tag(x_45, 5);
lean_ctor_set(x_45, 1, x_58);
if (lean_is_scalar(x_60)) {
 x_61 = lean_alloc_ctor(0, 2, 0);
} else {
 x_61 = x_60;
}
lean_ctor_set(x_61, 0, x_45);
lean_ctor_set(x_61, 1, x_59);
x_62 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_62, 0, x_61);
return x_62;
}
}
}
else
{
lean_object* x_63; lean_object* x_64; lean_object* x_65; 
x_63 = lean_ctor_get(x_45, 0);
x_64 = lean_ctor_get(x_45, 1);
lean_inc(x_64);
lean_inc(x_63);
lean_dec(x_45);
x_65 = lp_iris_x2dkernel_IrisKernel_FFI_decodeLIAFormula(x_64);
if (lean_obj_tag(x_65) == 0)
{
lean_dec(x_63);
return x_65;
}
else
{
lean_object* x_66; lean_object* x_67; lean_object* x_68; lean_object* x_69; lean_object* x_70; lean_object* x_71; lean_object* x_72; lean_object* x_73; 
x_66 = lean_ctor_get(x_65, 0);
lean_inc(x_66);
if (lean_is_exclusive(x_65)) {
 lean_ctor_release(x_65, 0);
 x_67 = x_65;
} else {
 lean_dec_ref(x_65);
 x_67 = lean_box(0);
}
x_68 = lean_ctor_get(x_66, 0);
lean_inc(x_68);
x_69 = lean_ctor_get(x_66, 1);
lean_inc(x_69);
if (lean_is_exclusive(x_66)) {
 lean_ctor_release(x_66, 0);
 lean_ctor_release(x_66, 1);
 x_70 = x_66;
} else {
 lean_dec_ref(x_66);
 x_70 = lean_box(0);
}
x_71 = lean_alloc_ctor(5, 2, 0);
lean_ctor_set(x_71, 0, x_63);
lean_ctor_set(x_71, 1, x_68);
if (lean_is_scalar(x_70)) {
 x_72 = lean_alloc_ctor(0, 2, 0);
} else {
 x_72 = x_70;
}
lean_ctor_set(x_72, 0, x_71);
lean_ctor_set(x_72, 1, x_69);
if (lean_is_scalar(x_67)) {
 x_73 = lean_alloc_ctor(1, 1, 0);
} else {
 x_73 = x_67;
}
lean_ctor_set(x_73, 0, x_72);
return x_73;
}
}
}
}
}
else
{
lean_object* x_74; 
x_74 = lp_iris_x2dkernel_IrisKernel_FFI_decodeLIAFormula(x_8);
if (lean_obj_tag(x_74) == 0)
{
return x_74;
}
else
{
uint8_t x_75; 
x_75 = !lean_is_exclusive(x_74);
if (x_75 == 0)
{
lean_object* x_76; uint8_t x_77; 
x_76 = lean_ctor_get(x_74, 0);
x_77 = !lean_is_exclusive(x_76);
if (x_77 == 0)
{
lean_object* x_78; lean_object* x_79; 
x_78 = lean_ctor_get(x_76, 0);
x_79 = lean_alloc_ctor(4, 1, 0);
lean_ctor_set(x_79, 0, x_78);
lean_ctor_set(x_76, 0, x_79);
return x_74;
}
else
{
lean_object* x_80; lean_object* x_81; lean_object* x_82; lean_object* x_83; 
x_80 = lean_ctor_get(x_76, 0);
x_81 = lean_ctor_get(x_76, 1);
lean_inc(x_81);
lean_inc(x_80);
lean_dec(x_76);
x_82 = lean_alloc_ctor(4, 1, 0);
lean_ctor_set(x_82, 0, x_80);
x_83 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_83, 0, x_82);
lean_ctor_set(x_83, 1, x_81);
lean_ctor_set(x_74, 0, x_83);
return x_74;
}
}
else
{
lean_object* x_84; lean_object* x_85; lean_object* x_86; lean_object* x_87; lean_object* x_88; lean_object* x_89; lean_object* x_90; 
x_84 = lean_ctor_get(x_74, 0);
lean_inc(x_84);
lean_dec(x_74);
x_85 = lean_ctor_get(x_84, 0);
lean_inc(x_85);
x_86 = lean_ctor_get(x_84, 1);
lean_inc(x_86);
if (lean_is_exclusive(x_84)) {
 lean_ctor_release(x_84, 0);
 lean_ctor_release(x_84, 1);
 x_87 = x_84;
} else {
 lean_dec_ref(x_84);
 x_87 = lean_box(0);
}
x_88 = lean_alloc_ctor(4, 1, 0);
lean_ctor_set(x_88, 0, x_85);
if (lean_is_scalar(x_87)) {
 x_89 = lean_alloc_ctor(0, 2, 0);
} else {
 x_89 = x_87;
}
lean_ctor_set(x_89, 0, x_88);
lean_ctor_set(x_89, 1, x_86);
x_90 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_90, 0, x_89);
return x_90;
}
}
}
}
else
{
lean_object* x_91; 
x_91 = lp_iris_x2dkernel_IrisKernel_FFI_decodeLIAFormula(x_8);
if (lean_obj_tag(x_91) == 0)
{
return x_91;
}
else
{
lean_object* x_92; uint8_t x_93; 
x_92 = lean_ctor_get(x_91, 0);
lean_inc(x_92);
lean_dec_ref(x_91);
x_93 = !lean_is_exclusive(x_92);
if (x_93 == 0)
{
lean_object* x_94; lean_object* x_95; lean_object* x_96; 
x_94 = lean_ctor_get(x_92, 0);
x_95 = lean_ctor_get(x_92, 1);
x_96 = lp_iris_x2dkernel_IrisKernel_FFI_decodeLIAFormula(x_95);
if (lean_obj_tag(x_96) == 0)
{
lean_free_object(x_92);
lean_dec(x_94);
return x_96;
}
else
{
uint8_t x_97; 
x_97 = !lean_is_exclusive(x_96);
if (x_97 == 0)
{
lean_object* x_98; uint8_t x_99; 
x_98 = lean_ctor_get(x_96, 0);
x_99 = !lean_is_exclusive(x_98);
if (x_99 == 0)
{
lean_object* x_100; 
x_100 = lean_ctor_get(x_98, 0);
lean_ctor_set_tag(x_92, 3);
lean_ctor_set(x_92, 1, x_100);
lean_ctor_set(x_98, 0, x_92);
return x_96;
}
else
{
lean_object* x_101; lean_object* x_102; lean_object* x_103; 
x_101 = lean_ctor_get(x_98, 0);
x_102 = lean_ctor_get(x_98, 1);
lean_inc(x_102);
lean_inc(x_101);
lean_dec(x_98);
lean_ctor_set_tag(x_92, 3);
lean_ctor_set(x_92, 1, x_101);
x_103 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_103, 0, x_92);
lean_ctor_set(x_103, 1, x_102);
lean_ctor_set(x_96, 0, x_103);
return x_96;
}
}
else
{
lean_object* x_104; lean_object* x_105; lean_object* x_106; lean_object* x_107; lean_object* x_108; lean_object* x_109; 
x_104 = lean_ctor_get(x_96, 0);
lean_inc(x_104);
lean_dec(x_96);
x_105 = lean_ctor_get(x_104, 0);
lean_inc(x_105);
x_106 = lean_ctor_get(x_104, 1);
lean_inc(x_106);
if (lean_is_exclusive(x_104)) {
 lean_ctor_release(x_104, 0);
 lean_ctor_release(x_104, 1);
 x_107 = x_104;
} else {
 lean_dec_ref(x_104);
 x_107 = lean_box(0);
}
lean_ctor_set_tag(x_92, 3);
lean_ctor_set(x_92, 1, x_105);
if (lean_is_scalar(x_107)) {
 x_108 = lean_alloc_ctor(0, 2, 0);
} else {
 x_108 = x_107;
}
lean_ctor_set(x_108, 0, x_92);
lean_ctor_set(x_108, 1, x_106);
x_109 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_109, 0, x_108);
return x_109;
}
}
}
else
{
lean_object* x_110; lean_object* x_111; lean_object* x_112; 
x_110 = lean_ctor_get(x_92, 0);
x_111 = lean_ctor_get(x_92, 1);
lean_inc(x_111);
lean_inc(x_110);
lean_dec(x_92);
x_112 = lp_iris_x2dkernel_IrisKernel_FFI_decodeLIAFormula(x_111);
if (lean_obj_tag(x_112) == 0)
{
lean_dec(x_110);
return x_112;
}
else
{
lean_object* x_113; lean_object* x_114; lean_object* x_115; lean_object* x_116; lean_object* x_117; lean_object* x_118; lean_object* x_119; lean_object* x_120; 
x_113 = lean_ctor_get(x_112, 0);
lean_inc(x_113);
if (lean_is_exclusive(x_112)) {
 lean_ctor_release(x_112, 0);
 x_114 = x_112;
} else {
 lean_dec_ref(x_112);
 x_114 = lean_box(0);
}
x_115 = lean_ctor_get(x_113, 0);
lean_inc(x_115);
x_116 = lean_ctor_get(x_113, 1);
lean_inc(x_116);
if (lean_is_exclusive(x_113)) {
 lean_ctor_release(x_113, 0);
 lean_ctor_release(x_113, 1);
 x_117 = x_113;
} else {
 lean_dec_ref(x_113);
 x_117 = lean_box(0);
}
x_118 = lean_alloc_ctor(3, 2, 0);
lean_ctor_set(x_118, 0, x_110);
lean_ctor_set(x_118, 1, x_115);
if (lean_is_scalar(x_117)) {
 x_119 = lean_alloc_ctor(0, 2, 0);
} else {
 x_119 = x_117;
}
lean_ctor_set(x_119, 0, x_118);
lean_ctor_set(x_119, 1, x_116);
if (lean_is_scalar(x_114)) {
 x_120 = lean_alloc_ctor(1, 1, 0);
} else {
 x_120 = x_114;
}
lean_ctor_set(x_120, 0, x_119);
return x_120;
}
}
}
}
}
else
{
lean_object* x_121; 
x_121 = lp_iris_x2dkernel_IrisKernel_FFI_decodeLIAFormula(x_8);
if (lean_obj_tag(x_121) == 0)
{
return x_121;
}
else
{
lean_object* x_122; uint8_t x_123; 
x_122 = lean_ctor_get(x_121, 0);
lean_inc(x_122);
lean_dec_ref(x_121);
x_123 = !lean_is_exclusive(x_122);
if (x_123 == 0)
{
lean_object* x_124; lean_object* x_125; lean_object* x_126; 
x_124 = lean_ctor_get(x_122, 0);
x_125 = lean_ctor_get(x_122, 1);
x_126 = lp_iris_x2dkernel_IrisKernel_FFI_decodeLIAFormula(x_125);
if (lean_obj_tag(x_126) == 0)
{
lean_free_object(x_122);
lean_dec(x_124);
return x_126;
}
else
{
uint8_t x_127; 
x_127 = !lean_is_exclusive(x_126);
if (x_127 == 0)
{
lean_object* x_128; uint8_t x_129; 
x_128 = lean_ctor_get(x_126, 0);
x_129 = !lean_is_exclusive(x_128);
if (x_129 == 0)
{
lean_object* x_130; 
x_130 = lean_ctor_get(x_128, 0);
lean_ctor_set_tag(x_122, 2);
lean_ctor_set(x_122, 1, x_130);
lean_ctor_set(x_128, 0, x_122);
return x_126;
}
else
{
lean_object* x_131; lean_object* x_132; lean_object* x_133; 
x_131 = lean_ctor_get(x_128, 0);
x_132 = lean_ctor_get(x_128, 1);
lean_inc(x_132);
lean_inc(x_131);
lean_dec(x_128);
lean_ctor_set_tag(x_122, 2);
lean_ctor_set(x_122, 1, x_131);
x_133 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_133, 0, x_122);
lean_ctor_set(x_133, 1, x_132);
lean_ctor_set(x_126, 0, x_133);
return x_126;
}
}
else
{
lean_object* x_134; lean_object* x_135; lean_object* x_136; lean_object* x_137; lean_object* x_138; lean_object* x_139; 
x_134 = lean_ctor_get(x_126, 0);
lean_inc(x_134);
lean_dec(x_126);
x_135 = lean_ctor_get(x_134, 0);
lean_inc(x_135);
x_136 = lean_ctor_get(x_134, 1);
lean_inc(x_136);
if (lean_is_exclusive(x_134)) {
 lean_ctor_release(x_134, 0);
 lean_ctor_release(x_134, 1);
 x_137 = x_134;
} else {
 lean_dec_ref(x_134);
 x_137 = lean_box(0);
}
lean_ctor_set_tag(x_122, 2);
lean_ctor_set(x_122, 1, x_135);
if (lean_is_scalar(x_137)) {
 x_138 = lean_alloc_ctor(0, 2, 0);
} else {
 x_138 = x_137;
}
lean_ctor_set(x_138, 0, x_122);
lean_ctor_set(x_138, 1, x_136);
x_139 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_139, 0, x_138);
return x_139;
}
}
}
else
{
lean_object* x_140; lean_object* x_141; lean_object* x_142; 
x_140 = lean_ctor_get(x_122, 0);
x_141 = lean_ctor_get(x_122, 1);
lean_inc(x_141);
lean_inc(x_140);
lean_dec(x_122);
x_142 = lp_iris_x2dkernel_IrisKernel_FFI_decodeLIAFormula(x_141);
if (lean_obj_tag(x_142) == 0)
{
lean_dec(x_140);
return x_142;
}
else
{
lean_object* x_143; lean_object* x_144; lean_object* x_145; lean_object* x_146; lean_object* x_147; lean_object* x_148; lean_object* x_149; lean_object* x_150; 
x_143 = lean_ctor_get(x_142, 0);
lean_inc(x_143);
if (lean_is_exclusive(x_142)) {
 lean_ctor_release(x_142, 0);
 x_144 = x_142;
} else {
 lean_dec_ref(x_142);
 x_144 = lean_box(0);
}
x_145 = lean_ctor_get(x_143, 0);
lean_inc(x_145);
x_146 = lean_ctor_get(x_143, 1);
lean_inc(x_146);
if (lean_is_exclusive(x_143)) {
 lean_ctor_release(x_143, 0);
 lean_ctor_release(x_143, 1);
 x_147 = x_143;
} else {
 lean_dec_ref(x_143);
 x_147 = lean_box(0);
}
x_148 = lean_alloc_ctor(2, 2, 0);
lean_ctor_set(x_148, 0, x_140);
lean_ctor_set(x_148, 1, x_145);
if (lean_is_scalar(x_147)) {
 x_149 = lean_alloc_ctor(0, 2, 0);
} else {
 x_149 = x_147;
}
lean_ctor_set(x_149, 0, x_148);
lean_ctor_set(x_149, 1, x_146);
if (lean_is_scalar(x_144)) {
 x_150 = lean_alloc_ctor(1, 1, 0);
} else {
 x_150 = x_144;
}
lean_ctor_set(x_150, 0, x_149);
return x_150;
}
}
}
}
}
else
{
lean_object* x_151; 
x_151 = lean_box(1);
lean_ctor_set(x_5, 0, x_151);
return x_2;
}
}
else
{
lean_object* x_152; 
x_152 = lean_box(0);
lean_ctor_set(x_5, 0, x_152);
return x_2;
}
}
else
{
lean_object* x_153; lean_object* x_154; uint8_t x_155; lean_object* x_156; lean_object* x_157; uint8_t x_158; 
x_153 = lean_ctor_get(x_5, 0);
x_154 = lean_ctor_get(x_5, 1);
lean_inc(x_154);
lean_inc(x_153);
lean_dec(x_5);
x_155 = lean_unbox(x_153);
lean_dec(x_153);
x_156 = lean_uint8_to_nat(x_155);
x_157 = lean_unsigned_to_nat(0u);
x_158 = lean_nat_dec_eq(x_156, x_157);
if (x_158 == 0)
{
lean_object* x_159; uint8_t x_160; 
x_159 = lean_unsigned_to_nat(1u);
x_160 = lean_nat_dec_eq(x_156, x_159);
if (x_160 == 0)
{
lean_object* x_161; uint8_t x_162; 
lean_free_object(x_2);
x_161 = lean_unsigned_to_nat(2u);
x_162 = lean_nat_dec_eq(x_156, x_161);
if (x_162 == 0)
{
lean_object* x_163; uint8_t x_164; 
x_163 = lean_unsigned_to_nat(3u);
x_164 = lean_nat_dec_eq(x_156, x_163);
if (x_164 == 0)
{
lean_object* x_165; uint8_t x_166; 
x_165 = lean_unsigned_to_nat(4u);
x_166 = lean_nat_dec_eq(x_156, x_165);
if (x_166 == 0)
{
lean_object* x_167; uint8_t x_168; 
x_167 = lean_unsigned_to_nat(5u);
x_168 = lean_nat_dec_eq(x_156, x_167);
if (x_168 == 0)
{
lean_object* x_169; uint8_t x_170; 
x_169 = lean_unsigned_to_nat(6u);
x_170 = lean_nat_dec_eq(x_156, x_169);
if (x_170 == 0)
{
lean_object* x_171; 
lean_dec(x_154);
x_171 = lean_box(0);
return x_171;
}
else
{
lean_object* x_172; 
x_172 = lp_iris_x2dkernel_IrisKernel_FFI_decodeLIAAtom(x_154);
if (lean_obj_tag(x_172) == 0)
{
lean_object* x_173; 
x_173 = lean_box(0);
return x_173;
}
else
{
lean_object* x_174; lean_object* x_175; lean_object* x_176; lean_object* x_177; lean_object* x_178; lean_object* x_179; lean_object* x_180; lean_object* x_181; 
x_174 = lean_ctor_get(x_172, 0);
lean_inc(x_174);
if (lean_is_exclusive(x_172)) {
 lean_ctor_release(x_172, 0);
 x_175 = x_172;
} else {
 lean_dec_ref(x_172);
 x_175 = lean_box(0);
}
x_176 = lean_ctor_get(x_174, 0);
lean_inc(x_176);
x_177 = lean_ctor_get(x_174, 1);
lean_inc(x_177);
if (lean_is_exclusive(x_174)) {
 lean_ctor_release(x_174, 0);
 lean_ctor_release(x_174, 1);
 x_178 = x_174;
} else {
 lean_dec_ref(x_174);
 x_178 = lean_box(0);
}
x_179 = lean_alloc_ctor(6, 1, 0);
lean_ctor_set(x_179, 0, x_176);
if (lean_is_scalar(x_178)) {
 x_180 = lean_alloc_ctor(0, 2, 0);
} else {
 x_180 = x_178;
}
lean_ctor_set(x_180, 0, x_179);
lean_ctor_set(x_180, 1, x_177);
if (lean_is_scalar(x_175)) {
 x_181 = lean_alloc_ctor(1, 1, 0);
} else {
 x_181 = x_175;
}
lean_ctor_set(x_181, 0, x_180);
return x_181;
}
}
}
else
{
lean_object* x_182; 
x_182 = lp_iris_x2dkernel_IrisKernel_FFI_decodeLIAFormula(x_154);
if (lean_obj_tag(x_182) == 0)
{
return x_182;
}
else
{
lean_object* x_183; lean_object* x_184; lean_object* x_185; lean_object* x_186; lean_object* x_187; 
x_183 = lean_ctor_get(x_182, 0);
lean_inc(x_183);
lean_dec_ref(x_182);
x_184 = lean_ctor_get(x_183, 0);
lean_inc(x_184);
x_185 = lean_ctor_get(x_183, 1);
lean_inc(x_185);
if (lean_is_exclusive(x_183)) {
 lean_ctor_release(x_183, 0);
 lean_ctor_release(x_183, 1);
 x_186 = x_183;
} else {
 lean_dec_ref(x_183);
 x_186 = lean_box(0);
}
x_187 = lp_iris_x2dkernel_IrisKernel_FFI_decodeLIAFormula(x_185);
if (lean_obj_tag(x_187) == 0)
{
lean_dec(x_186);
lean_dec(x_184);
return x_187;
}
else
{
lean_object* x_188; lean_object* x_189; lean_object* x_190; lean_object* x_191; lean_object* x_192; lean_object* x_193; lean_object* x_194; lean_object* x_195; 
x_188 = lean_ctor_get(x_187, 0);
lean_inc(x_188);
if (lean_is_exclusive(x_187)) {
 lean_ctor_release(x_187, 0);
 x_189 = x_187;
} else {
 lean_dec_ref(x_187);
 x_189 = lean_box(0);
}
x_190 = lean_ctor_get(x_188, 0);
lean_inc(x_190);
x_191 = lean_ctor_get(x_188, 1);
lean_inc(x_191);
if (lean_is_exclusive(x_188)) {
 lean_ctor_release(x_188, 0);
 lean_ctor_release(x_188, 1);
 x_192 = x_188;
} else {
 lean_dec_ref(x_188);
 x_192 = lean_box(0);
}
if (lean_is_scalar(x_186)) {
 x_193 = lean_alloc_ctor(5, 2, 0);
} else {
 x_193 = x_186;
 lean_ctor_set_tag(x_193, 5);
}
lean_ctor_set(x_193, 0, x_184);
lean_ctor_set(x_193, 1, x_190);
if (lean_is_scalar(x_192)) {
 x_194 = lean_alloc_ctor(0, 2, 0);
} else {
 x_194 = x_192;
}
lean_ctor_set(x_194, 0, x_193);
lean_ctor_set(x_194, 1, x_191);
if (lean_is_scalar(x_189)) {
 x_195 = lean_alloc_ctor(1, 1, 0);
} else {
 x_195 = x_189;
}
lean_ctor_set(x_195, 0, x_194);
return x_195;
}
}
}
}
else
{
lean_object* x_196; 
x_196 = lp_iris_x2dkernel_IrisKernel_FFI_decodeLIAFormula(x_154);
if (lean_obj_tag(x_196) == 0)
{
return x_196;
}
else
{
lean_object* x_197; lean_object* x_198; lean_object* x_199; lean_object* x_200; lean_object* x_201; lean_object* x_202; lean_object* x_203; lean_object* x_204; 
x_197 = lean_ctor_get(x_196, 0);
lean_inc(x_197);
if (lean_is_exclusive(x_196)) {
 lean_ctor_release(x_196, 0);
 x_198 = x_196;
} else {
 lean_dec_ref(x_196);
 x_198 = lean_box(0);
}
x_199 = lean_ctor_get(x_197, 0);
lean_inc(x_199);
x_200 = lean_ctor_get(x_197, 1);
lean_inc(x_200);
if (lean_is_exclusive(x_197)) {
 lean_ctor_release(x_197, 0);
 lean_ctor_release(x_197, 1);
 x_201 = x_197;
} else {
 lean_dec_ref(x_197);
 x_201 = lean_box(0);
}
x_202 = lean_alloc_ctor(4, 1, 0);
lean_ctor_set(x_202, 0, x_199);
if (lean_is_scalar(x_201)) {
 x_203 = lean_alloc_ctor(0, 2, 0);
} else {
 x_203 = x_201;
}
lean_ctor_set(x_203, 0, x_202);
lean_ctor_set(x_203, 1, x_200);
if (lean_is_scalar(x_198)) {
 x_204 = lean_alloc_ctor(1, 1, 0);
} else {
 x_204 = x_198;
}
lean_ctor_set(x_204, 0, x_203);
return x_204;
}
}
}
else
{
lean_object* x_205; 
x_205 = lp_iris_x2dkernel_IrisKernel_FFI_decodeLIAFormula(x_154);
if (lean_obj_tag(x_205) == 0)
{
return x_205;
}
else
{
lean_object* x_206; lean_object* x_207; lean_object* x_208; lean_object* x_209; lean_object* x_210; 
x_206 = lean_ctor_get(x_205, 0);
lean_inc(x_206);
lean_dec_ref(x_205);
x_207 = lean_ctor_get(x_206, 0);
lean_inc(x_207);
x_208 = lean_ctor_get(x_206, 1);
lean_inc(x_208);
if (lean_is_exclusive(x_206)) {
 lean_ctor_release(x_206, 0);
 lean_ctor_release(x_206, 1);
 x_209 = x_206;
} else {
 lean_dec_ref(x_206);
 x_209 = lean_box(0);
}
x_210 = lp_iris_x2dkernel_IrisKernel_FFI_decodeLIAFormula(x_208);
if (lean_obj_tag(x_210) == 0)
{
lean_dec(x_209);
lean_dec(x_207);
return x_210;
}
else
{
lean_object* x_211; lean_object* x_212; lean_object* x_213; lean_object* x_214; lean_object* x_215; lean_object* x_216; lean_object* x_217; lean_object* x_218; 
x_211 = lean_ctor_get(x_210, 0);
lean_inc(x_211);
if (lean_is_exclusive(x_210)) {
 lean_ctor_release(x_210, 0);
 x_212 = x_210;
} else {
 lean_dec_ref(x_210);
 x_212 = lean_box(0);
}
x_213 = lean_ctor_get(x_211, 0);
lean_inc(x_213);
x_214 = lean_ctor_get(x_211, 1);
lean_inc(x_214);
if (lean_is_exclusive(x_211)) {
 lean_ctor_release(x_211, 0);
 lean_ctor_release(x_211, 1);
 x_215 = x_211;
} else {
 lean_dec_ref(x_211);
 x_215 = lean_box(0);
}
if (lean_is_scalar(x_209)) {
 x_216 = lean_alloc_ctor(3, 2, 0);
} else {
 x_216 = x_209;
 lean_ctor_set_tag(x_216, 3);
}
lean_ctor_set(x_216, 0, x_207);
lean_ctor_set(x_216, 1, x_213);
if (lean_is_scalar(x_215)) {
 x_217 = lean_alloc_ctor(0, 2, 0);
} else {
 x_217 = x_215;
}
lean_ctor_set(x_217, 0, x_216);
lean_ctor_set(x_217, 1, x_214);
if (lean_is_scalar(x_212)) {
 x_218 = lean_alloc_ctor(1, 1, 0);
} else {
 x_218 = x_212;
}
lean_ctor_set(x_218, 0, x_217);
return x_218;
}
}
}
}
else
{
lean_object* x_219; 
x_219 = lp_iris_x2dkernel_IrisKernel_FFI_decodeLIAFormula(x_154);
if (lean_obj_tag(x_219) == 0)
{
return x_219;
}
else
{
lean_object* x_220; lean_object* x_221; lean_object* x_222; lean_object* x_223; lean_object* x_224; 
x_220 = lean_ctor_get(x_219, 0);
lean_inc(x_220);
lean_dec_ref(x_219);
x_221 = lean_ctor_get(x_220, 0);
lean_inc(x_221);
x_222 = lean_ctor_get(x_220, 1);
lean_inc(x_222);
if (lean_is_exclusive(x_220)) {
 lean_ctor_release(x_220, 0);
 lean_ctor_release(x_220, 1);
 x_223 = x_220;
} else {
 lean_dec_ref(x_220);
 x_223 = lean_box(0);
}
x_224 = lp_iris_x2dkernel_IrisKernel_FFI_decodeLIAFormula(x_222);
if (lean_obj_tag(x_224) == 0)
{
lean_dec(x_223);
lean_dec(x_221);
return x_224;
}
else
{
lean_object* x_225; lean_object* x_226; lean_object* x_227; lean_object* x_228; lean_object* x_229; lean_object* x_230; lean_object* x_231; lean_object* x_232; 
x_225 = lean_ctor_get(x_224, 0);
lean_inc(x_225);
if (lean_is_exclusive(x_224)) {
 lean_ctor_release(x_224, 0);
 x_226 = x_224;
} else {
 lean_dec_ref(x_224);
 x_226 = lean_box(0);
}
x_227 = lean_ctor_get(x_225, 0);
lean_inc(x_227);
x_228 = lean_ctor_get(x_225, 1);
lean_inc(x_228);
if (lean_is_exclusive(x_225)) {
 lean_ctor_release(x_225, 0);
 lean_ctor_release(x_225, 1);
 x_229 = x_225;
} else {
 lean_dec_ref(x_225);
 x_229 = lean_box(0);
}
if (lean_is_scalar(x_223)) {
 x_230 = lean_alloc_ctor(2, 2, 0);
} else {
 x_230 = x_223;
 lean_ctor_set_tag(x_230, 2);
}
lean_ctor_set(x_230, 0, x_221);
lean_ctor_set(x_230, 1, x_227);
if (lean_is_scalar(x_229)) {
 x_231 = lean_alloc_ctor(0, 2, 0);
} else {
 x_231 = x_229;
}
lean_ctor_set(x_231, 0, x_230);
lean_ctor_set(x_231, 1, x_228);
if (lean_is_scalar(x_226)) {
 x_232 = lean_alloc_ctor(1, 1, 0);
} else {
 x_232 = x_226;
}
lean_ctor_set(x_232, 0, x_231);
return x_232;
}
}
}
}
else
{
lean_object* x_233; lean_object* x_234; 
x_233 = lean_box(1);
x_234 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_234, 0, x_233);
lean_ctor_set(x_234, 1, x_154);
lean_ctor_set(x_2, 0, x_234);
return x_2;
}
}
else
{
lean_object* x_235; lean_object* x_236; 
x_235 = lean_box(0);
x_236 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_236, 0, x_235);
lean_ctor_set(x_236, 1, x_154);
lean_ctor_set(x_2, 0, x_236);
return x_2;
}
}
}
else
{
lean_object* x_237; lean_object* x_238; lean_object* x_239; lean_object* x_240; uint8_t x_241; lean_object* x_242; lean_object* x_243; uint8_t x_244; 
x_237 = lean_ctor_get(x_2, 0);
lean_inc(x_237);
lean_dec(x_2);
x_238 = lean_ctor_get(x_237, 0);
lean_inc(x_238);
x_239 = lean_ctor_get(x_237, 1);
lean_inc(x_239);
if (lean_is_exclusive(x_237)) {
 lean_ctor_release(x_237, 0);
 lean_ctor_release(x_237, 1);
 x_240 = x_237;
} else {
 lean_dec_ref(x_237);
 x_240 = lean_box(0);
}
x_241 = lean_unbox(x_238);
lean_dec(x_238);
x_242 = lean_uint8_to_nat(x_241);
x_243 = lean_unsigned_to_nat(0u);
x_244 = lean_nat_dec_eq(x_242, x_243);
if (x_244 == 0)
{
lean_object* x_245; uint8_t x_246; 
x_245 = lean_unsigned_to_nat(1u);
x_246 = lean_nat_dec_eq(x_242, x_245);
if (x_246 == 0)
{
lean_object* x_247; uint8_t x_248; 
lean_dec(x_240);
x_247 = lean_unsigned_to_nat(2u);
x_248 = lean_nat_dec_eq(x_242, x_247);
if (x_248 == 0)
{
lean_object* x_249; uint8_t x_250; 
x_249 = lean_unsigned_to_nat(3u);
x_250 = lean_nat_dec_eq(x_242, x_249);
if (x_250 == 0)
{
lean_object* x_251; uint8_t x_252; 
x_251 = lean_unsigned_to_nat(4u);
x_252 = lean_nat_dec_eq(x_242, x_251);
if (x_252 == 0)
{
lean_object* x_253; uint8_t x_254; 
x_253 = lean_unsigned_to_nat(5u);
x_254 = lean_nat_dec_eq(x_242, x_253);
if (x_254 == 0)
{
lean_object* x_255; uint8_t x_256; 
x_255 = lean_unsigned_to_nat(6u);
x_256 = lean_nat_dec_eq(x_242, x_255);
if (x_256 == 0)
{
lean_object* x_257; 
lean_dec(x_239);
x_257 = lean_box(0);
return x_257;
}
else
{
lean_object* x_258; 
x_258 = lp_iris_x2dkernel_IrisKernel_FFI_decodeLIAAtom(x_239);
if (lean_obj_tag(x_258) == 0)
{
lean_object* x_259; 
x_259 = lean_box(0);
return x_259;
}
else
{
lean_object* x_260; lean_object* x_261; lean_object* x_262; lean_object* x_263; lean_object* x_264; lean_object* x_265; lean_object* x_266; lean_object* x_267; 
x_260 = lean_ctor_get(x_258, 0);
lean_inc(x_260);
if (lean_is_exclusive(x_258)) {
 lean_ctor_release(x_258, 0);
 x_261 = x_258;
} else {
 lean_dec_ref(x_258);
 x_261 = lean_box(0);
}
x_262 = lean_ctor_get(x_260, 0);
lean_inc(x_262);
x_263 = lean_ctor_get(x_260, 1);
lean_inc(x_263);
if (lean_is_exclusive(x_260)) {
 lean_ctor_release(x_260, 0);
 lean_ctor_release(x_260, 1);
 x_264 = x_260;
} else {
 lean_dec_ref(x_260);
 x_264 = lean_box(0);
}
x_265 = lean_alloc_ctor(6, 1, 0);
lean_ctor_set(x_265, 0, x_262);
if (lean_is_scalar(x_264)) {
 x_266 = lean_alloc_ctor(0, 2, 0);
} else {
 x_266 = x_264;
}
lean_ctor_set(x_266, 0, x_265);
lean_ctor_set(x_266, 1, x_263);
if (lean_is_scalar(x_261)) {
 x_267 = lean_alloc_ctor(1, 1, 0);
} else {
 x_267 = x_261;
}
lean_ctor_set(x_267, 0, x_266);
return x_267;
}
}
}
else
{
lean_object* x_268; 
x_268 = lp_iris_x2dkernel_IrisKernel_FFI_decodeLIAFormula(x_239);
if (lean_obj_tag(x_268) == 0)
{
return x_268;
}
else
{
lean_object* x_269; lean_object* x_270; lean_object* x_271; lean_object* x_272; lean_object* x_273; 
x_269 = lean_ctor_get(x_268, 0);
lean_inc(x_269);
lean_dec_ref(x_268);
x_270 = lean_ctor_get(x_269, 0);
lean_inc(x_270);
x_271 = lean_ctor_get(x_269, 1);
lean_inc(x_271);
if (lean_is_exclusive(x_269)) {
 lean_ctor_release(x_269, 0);
 lean_ctor_release(x_269, 1);
 x_272 = x_269;
} else {
 lean_dec_ref(x_269);
 x_272 = lean_box(0);
}
x_273 = lp_iris_x2dkernel_IrisKernel_FFI_decodeLIAFormula(x_271);
if (lean_obj_tag(x_273) == 0)
{
lean_dec(x_272);
lean_dec(x_270);
return x_273;
}
else
{
lean_object* x_274; lean_object* x_275; lean_object* x_276; lean_object* x_277; lean_object* x_278; lean_object* x_279; lean_object* x_280; lean_object* x_281; 
x_274 = lean_ctor_get(x_273, 0);
lean_inc(x_274);
if (lean_is_exclusive(x_273)) {
 lean_ctor_release(x_273, 0);
 x_275 = x_273;
} else {
 lean_dec_ref(x_273);
 x_275 = lean_box(0);
}
x_276 = lean_ctor_get(x_274, 0);
lean_inc(x_276);
x_277 = lean_ctor_get(x_274, 1);
lean_inc(x_277);
if (lean_is_exclusive(x_274)) {
 lean_ctor_release(x_274, 0);
 lean_ctor_release(x_274, 1);
 x_278 = x_274;
} else {
 lean_dec_ref(x_274);
 x_278 = lean_box(0);
}
if (lean_is_scalar(x_272)) {
 x_279 = lean_alloc_ctor(5, 2, 0);
} else {
 x_279 = x_272;
 lean_ctor_set_tag(x_279, 5);
}
lean_ctor_set(x_279, 0, x_270);
lean_ctor_set(x_279, 1, x_276);
if (lean_is_scalar(x_278)) {
 x_280 = lean_alloc_ctor(0, 2, 0);
} else {
 x_280 = x_278;
}
lean_ctor_set(x_280, 0, x_279);
lean_ctor_set(x_280, 1, x_277);
if (lean_is_scalar(x_275)) {
 x_281 = lean_alloc_ctor(1, 1, 0);
} else {
 x_281 = x_275;
}
lean_ctor_set(x_281, 0, x_280);
return x_281;
}
}
}
}
else
{
lean_object* x_282; 
x_282 = lp_iris_x2dkernel_IrisKernel_FFI_decodeLIAFormula(x_239);
if (lean_obj_tag(x_282) == 0)
{
return x_282;
}
else
{
lean_object* x_283; lean_object* x_284; lean_object* x_285; lean_object* x_286; lean_object* x_287; lean_object* x_288; lean_object* x_289; lean_object* x_290; 
x_283 = lean_ctor_get(x_282, 0);
lean_inc(x_283);
if (lean_is_exclusive(x_282)) {
 lean_ctor_release(x_282, 0);
 x_284 = x_282;
} else {
 lean_dec_ref(x_282);
 x_284 = lean_box(0);
}
x_285 = lean_ctor_get(x_283, 0);
lean_inc(x_285);
x_286 = lean_ctor_get(x_283, 1);
lean_inc(x_286);
if (lean_is_exclusive(x_283)) {
 lean_ctor_release(x_283, 0);
 lean_ctor_release(x_283, 1);
 x_287 = x_283;
} else {
 lean_dec_ref(x_283);
 x_287 = lean_box(0);
}
x_288 = lean_alloc_ctor(4, 1, 0);
lean_ctor_set(x_288, 0, x_285);
if (lean_is_scalar(x_287)) {
 x_289 = lean_alloc_ctor(0, 2, 0);
} else {
 x_289 = x_287;
}
lean_ctor_set(x_289, 0, x_288);
lean_ctor_set(x_289, 1, x_286);
if (lean_is_scalar(x_284)) {
 x_290 = lean_alloc_ctor(1, 1, 0);
} else {
 x_290 = x_284;
}
lean_ctor_set(x_290, 0, x_289);
return x_290;
}
}
}
else
{
lean_object* x_291; 
x_291 = lp_iris_x2dkernel_IrisKernel_FFI_decodeLIAFormula(x_239);
if (lean_obj_tag(x_291) == 0)
{
return x_291;
}
else
{
lean_object* x_292; lean_object* x_293; lean_object* x_294; lean_object* x_295; lean_object* x_296; 
x_292 = lean_ctor_get(x_291, 0);
lean_inc(x_292);
lean_dec_ref(x_291);
x_293 = lean_ctor_get(x_292, 0);
lean_inc(x_293);
x_294 = lean_ctor_get(x_292, 1);
lean_inc(x_294);
if (lean_is_exclusive(x_292)) {
 lean_ctor_release(x_292, 0);
 lean_ctor_release(x_292, 1);
 x_295 = x_292;
} else {
 lean_dec_ref(x_292);
 x_295 = lean_box(0);
}
x_296 = lp_iris_x2dkernel_IrisKernel_FFI_decodeLIAFormula(x_294);
if (lean_obj_tag(x_296) == 0)
{
lean_dec(x_295);
lean_dec(x_293);
return x_296;
}
else
{
lean_object* x_297; lean_object* x_298; lean_object* x_299; lean_object* x_300; lean_object* x_301; lean_object* x_302; lean_object* x_303; lean_object* x_304; 
x_297 = lean_ctor_get(x_296, 0);
lean_inc(x_297);
if (lean_is_exclusive(x_296)) {
 lean_ctor_release(x_296, 0);
 x_298 = x_296;
} else {
 lean_dec_ref(x_296);
 x_298 = lean_box(0);
}
x_299 = lean_ctor_get(x_297, 0);
lean_inc(x_299);
x_300 = lean_ctor_get(x_297, 1);
lean_inc(x_300);
if (lean_is_exclusive(x_297)) {
 lean_ctor_release(x_297, 0);
 lean_ctor_release(x_297, 1);
 x_301 = x_297;
} else {
 lean_dec_ref(x_297);
 x_301 = lean_box(0);
}
if (lean_is_scalar(x_295)) {
 x_302 = lean_alloc_ctor(3, 2, 0);
} else {
 x_302 = x_295;
 lean_ctor_set_tag(x_302, 3);
}
lean_ctor_set(x_302, 0, x_293);
lean_ctor_set(x_302, 1, x_299);
if (lean_is_scalar(x_301)) {
 x_303 = lean_alloc_ctor(0, 2, 0);
} else {
 x_303 = x_301;
}
lean_ctor_set(x_303, 0, x_302);
lean_ctor_set(x_303, 1, x_300);
if (lean_is_scalar(x_298)) {
 x_304 = lean_alloc_ctor(1, 1, 0);
} else {
 x_304 = x_298;
}
lean_ctor_set(x_304, 0, x_303);
return x_304;
}
}
}
}
else
{
lean_object* x_305; 
x_305 = lp_iris_x2dkernel_IrisKernel_FFI_decodeLIAFormula(x_239);
if (lean_obj_tag(x_305) == 0)
{
return x_305;
}
else
{
lean_object* x_306; lean_object* x_307; lean_object* x_308; lean_object* x_309; lean_object* x_310; 
x_306 = lean_ctor_get(x_305, 0);
lean_inc(x_306);
lean_dec_ref(x_305);
x_307 = lean_ctor_get(x_306, 0);
lean_inc(x_307);
x_308 = lean_ctor_get(x_306, 1);
lean_inc(x_308);
if (lean_is_exclusive(x_306)) {
 lean_ctor_release(x_306, 0);
 lean_ctor_release(x_306, 1);
 x_309 = x_306;
} else {
 lean_dec_ref(x_306);
 x_309 = lean_box(0);
}
x_310 = lp_iris_x2dkernel_IrisKernel_FFI_decodeLIAFormula(x_308);
if (lean_obj_tag(x_310) == 0)
{
lean_dec(x_309);
lean_dec(x_307);
return x_310;
}
else
{
lean_object* x_311; lean_object* x_312; lean_object* x_313; lean_object* x_314; lean_object* x_315; lean_object* x_316; lean_object* x_317; lean_object* x_318; 
x_311 = lean_ctor_get(x_310, 0);
lean_inc(x_311);
if (lean_is_exclusive(x_310)) {
 lean_ctor_release(x_310, 0);
 x_312 = x_310;
} else {
 lean_dec_ref(x_310);
 x_312 = lean_box(0);
}
x_313 = lean_ctor_get(x_311, 0);
lean_inc(x_313);
x_314 = lean_ctor_get(x_311, 1);
lean_inc(x_314);
if (lean_is_exclusive(x_311)) {
 lean_ctor_release(x_311, 0);
 lean_ctor_release(x_311, 1);
 x_315 = x_311;
} else {
 lean_dec_ref(x_311);
 x_315 = lean_box(0);
}
if (lean_is_scalar(x_309)) {
 x_316 = lean_alloc_ctor(2, 2, 0);
} else {
 x_316 = x_309;
 lean_ctor_set_tag(x_316, 2);
}
lean_ctor_set(x_316, 0, x_307);
lean_ctor_set(x_316, 1, x_313);
if (lean_is_scalar(x_315)) {
 x_317 = lean_alloc_ctor(0, 2, 0);
} else {
 x_317 = x_315;
}
lean_ctor_set(x_317, 0, x_316);
lean_ctor_set(x_317, 1, x_314);
if (lean_is_scalar(x_312)) {
 x_318 = lean_alloc_ctor(1, 1, 0);
} else {
 x_318 = x_312;
}
lean_ctor_set(x_318, 0, x_317);
return x_318;
}
}
}
}
else
{
lean_object* x_319; lean_object* x_320; lean_object* x_321; 
x_319 = lean_box(1);
if (lean_is_scalar(x_240)) {
 x_320 = lean_alloc_ctor(0, 2, 0);
} else {
 x_320 = x_240;
}
lean_ctor_set(x_320, 0, x_319);
lean_ctor_set(x_320, 1, x_239);
x_321 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_321, 0, x_320);
return x_321;
}
}
else
{
lean_object* x_322; lean_object* x_323; lean_object* x_324; 
x_322 = lean_box(0);
if (lean_is_scalar(x_240)) {
 x_323 = lean_alloc_ctor(0, 2, 0);
} else {
 x_323 = x_240;
}
lean_ctor_set(x_323, 0, x_322);
lean_ctor_set(x_323, 1, x_239);
x_324 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_324, 0, x_323);
return x_324;
}
}
}
}
}
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_decodeLIATerm(lean_object* x_1) {
_start:
{
lean_object* x_2; 
x_2 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readUInt8(x_1);
if (lean_obj_tag(x_2) == 0)
{
lean_object* x_3; 
x_3 = lean_box(0);
return x_3;
}
else
{
uint8_t x_4; 
x_4 = !lean_is_exclusive(x_2);
if (x_4 == 0)
{
lean_object* x_5; lean_object* x_6; lean_object* x_7; uint8_t x_8; lean_object* x_9; lean_object* x_10; uint8_t x_11; 
x_5 = lean_ctor_get(x_2, 0);
x_6 = lean_ctor_get(x_5, 0);
lean_inc(x_6);
x_7 = lean_ctor_get(x_5, 1);
lean_inc(x_7);
lean_dec(x_5);
x_8 = lean_unbox(x_6);
lean_dec(x_6);
x_9 = lean_uint8_to_nat(x_8);
x_10 = lean_unsigned_to_nat(0u);
x_11 = lean_nat_dec_eq(x_9, x_10);
if (x_11 == 0)
{
lean_object* x_12; uint8_t x_13; 
x_12 = lean_unsigned_to_nat(1u);
x_13 = lean_nat_dec_eq(x_9, x_12);
if (x_13 == 0)
{
lean_object* x_14; uint8_t x_15; 
x_14 = lean_unsigned_to_nat(2u);
x_15 = lean_nat_dec_eq(x_9, x_14);
if (x_15 == 0)
{
lean_object* x_16; uint8_t x_17; 
x_16 = lean_unsigned_to_nat(3u);
x_17 = lean_nat_dec_eq(x_9, x_16);
if (x_17 == 0)
{
lean_object* x_18; uint8_t x_19; 
x_18 = lean_unsigned_to_nat(4u);
x_19 = lean_nat_dec_eq(x_9, x_18);
if (x_19 == 0)
{
lean_object* x_20; uint8_t x_21; 
x_20 = lean_unsigned_to_nat(5u);
x_21 = lean_nat_dec_eq(x_9, x_20);
if (x_21 == 0)
{
lean_object* x_22; uint8_t x_23; 
x_22 = lean_unsigned_to_nat(6u);
x_23 = lean_nat_dec_eq(x_9, x_22);
if (x_23 == 0)
{
lean_object* x_24; uint8_t x_25; 
lean_free_object(x_2);
x_24 = lean_unsigned_to_nat(7u);
x_25 = lean_nat_dec_eq(x_9, x_24);
if (x_25 == 0)
{
lean_object* x_26; uint8_t x_27; 
x_26 = lean_unsigned_to_nat(8u);
x_27 = lean_nat_dec_eq(x_9, x_26);
if (x_27 == 0)
{
lean_object* x_28; 
lean_dec(x_7);
x_28 = lean_box(0);
return x_28;
}
else
{
lean_object* x_29; 
x_29 = lp_iris_x2dkernel_IrisKernel_FFI_decodeLIATerm(x_7);
if (lean_obj_tag(x_29) == 0)
{
return x_29;
}
else
{
lean_object* x_30; uint8_t x_31; 
x_30 = lean_ctor_get(x_29, 0);
lean_inc(x_30);
lean_dec_ref(x_29);
x_31 = !lean_is_exclusive(x_30);
if (x_31 == 0)
{
lean_object* x_32; lean_object* x_33; lean_object* x_34; 
x_32 = lean_ctor_get(x_30, 0);
x_33 = lean_ctor_get(x_30, 1);
x_34 = lp_iris_x2dkernel_IrisKernel_FFI_decodeLIATerm(x_33);
if (lean_obj_tag(x_34) == 0)
{
lean_free_object(x_30);
lean_dec(x_32);
return x_34;
}
else
{
uint8_t x_35; 
x_35 = !lean_is_exclusive(x_34);
if (x_35 == 0)
{
lean_object* x_36; uint8_t x_37; 
x_36 = lean_ctor_get(x_34, 0);
x_37 = !lean_is_exclusive(x_36);
if (x_37 == 0)
{
lean_object* x_38; 
x_38 = lean_ctor_get(x_36, 0);
lean_ctor_set_tag(x_30, 8);
lean_ctor_set(x_30, 1, x_38);
lean_ctor_set(x_36, 0, x_30);
return x_34;
}
else
{
lean_object* x_39; lean_object* x_40; lean_object* x_41; 
x_39 = lean_ctor_get(x_36, 0);
x_40 = lean_ctor_get(x_36, 1);
lean_inc(x_40);
lean_inc(x_39);
lean_dec(x_36);
lean_ctor_set_tag(x_30, 8);
lean_ctor_set(x_30, 1, x_39);
x_41 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_41, 0, x_30);
lean_ctor_set(x_41, 1, x_40);
lean_ctor_set(x_34, 0, x_41);
return x_34;
}
}
else
{
lean_object* x_42; lean_object* x_43; lean_object* x_44; lean_object* x_45; lean_object* x_46; lean_object* x_47; 
x_42 = lean_ctor_get(x_34, 0);
lean_inc(x_42);
lean_dec(x_34);
x_43 = lean_ctor_get(x_42, 0);
lean_inc(x_43);
x_44 = lean_ctor_get(x_42, 1);
lean_inc(x_44);
if (lean_is_exclusive(x_42)) {
 lean_ctor_release(x_42, 0);
 lean_ctor_release(x_42, 1);
 x_45 = x_42;
} else {
 lean_dec_ref(x_42);
 x_45 = lean_box(0);
}
lean_ctor_set_tag(x_30, 8);
lean_ctor_set(x_30, 1, x_43);
if (lean_is_scalar(x_45)) {
 x_46 = lean_alloc_ctor(0, 2, 0);
} else {
 x_46 = x_45;
}
lean_ctor_set(x_46, 0, x_30);
lean_ctor_set(x_46, 1, x_44);
x_47 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_47, 0, x_46);
return x_47;
}
}
}
else
{
lean_object* x_48; lean_object* x_49; lean_object* x_50; 
x_48 = lean_ctor_get(x_30, 0);
x_49 = lean_ctor_get(x_30, 1);
lean_inc(x_49);
lean_inc(x_48);
lean_dec(x_30);
x_50 = lp_iris_x2dkernel_IrisKernel_FFI_decodeLIATerm(x_49);
if (lean_obj_tag(x_50) == 0)
{
lean_dec(x_48);
return x_50;
}
else
{
lean_object* x_51; lean_object* x_52; lean_object* x_53; lean_object* x_54; lean_object* x_55; lean_object* x_56; lean_object* x_57; lean_object* x_58; 
x_51 = lean_ctor_get(x_50, 0);
lean_inc(x_51);
if (lean_is_exclusive(x_50)) {
 lean_ctor_release(x_50, 0);
 x_52 = x_50;
} else {
 lean_dec_ref(x_50);
 x_52 = lean_box(0);
}
x_53 = lean_ctor_get(x_51, 0);
lean_inc(x_53);
x_54 = lean_ctor_get(x_51, 1);
lean_inc(x_54);
if (lean_is_exclusive(x_51)) {
 lean_ctor_release(x_51, 0);
 lean_ctor_release(x_51, 1);
 x_55 = x_51;
} else {
 lean_dec_ref(x_51);
 x_55 = lean_box(0);
}
x_56 = lean_alloc_ctor(8, 2, 0);
lean_ctor_set(x_56, 0, x_48);
lean_ctor_set(x_56, 1, x_53);
if (lean_is_scalar(x_55)) {
 x_57 = lean_alloc_ctor(0, 2, 0);
} else {
 x_57 = x_55;
}
lean_ctor_set(x_57, 0, x_56);
lean_ctor_set(x_57, 1, x_54);
if (lean_is_scalar(x_52)) {
 x_58 = lean_alloc_ctor(1, 1, 0);
} else {
 x_58 = x_52;
}
lean_ctor_set(x_58, 0, x_57);
return x_58;
}
}
}
}
}
else
{
lean_object* x_59; 
x_59 = lp_iris_x2dkernel_IrisKernel_FFI_decodeLIAFormula(x_7);
if (lean_obj_tag(x_59) == 0)
{
lean_object* x_60; 
x_60 = lean_box(0);
return x_60;
}
else
{
lean_object* x_61; lean_object* x_62; lean_object* x_63; lean_object* x_64; 
x_61 = lean_ctor_get(x_59, 0);
lean_inc(x_61);
lean_dec_ref(x_59);
x_62 = lean_ctor_get(x_61, 0);
lean_inc(x_62);
x_63 = lean_ctor_get(x_61, 1);
lean_inc(x_63);
lean_dec(x_61);
x_64 = lp_iris_x2dkernel_IrisKernel_FFI_decodeLIATerm(x_63);
if (lean_obj_tag(x_64) == 0)
{
lean_dec(x_62);
return x_64;
}
else
{
lean_object* x_65; lean_object* x_66; lean_object* x_67; lean_object* x_68; 
x_65 = lean_ctor_get(x_64, 0);
lean_inc(x_65);
lean_dec_ref(x_64);
x_66 = lean_ctor_get(x_65, 0);
lean_inc(x_66);
x_67 = lean_ctor_get(x_65, 1);
lean_inc(x_67);
lean_dec(x_65);
x_68 = lp_iris_x2dkernel_IrisKernel_FFI_decodeLIATerm(x_67);
if (lean_obj_tag(x_68) == 0)
{
lean_dec(x_66);
lean_dec(x_62);
return x_68;
}
else
{
uint8_t x_69; 
x_69 = !lean_is_exclusive(x_68);
if (x_69 == 0)
{
lean_object* x_70; uint8_t x_71; 
x_70 = lean_ctor_get(x_68, 0);
x_71 = !lean_is_exclusive(x_70);
if (x_71 == 0)
{
lean_object* x_72; lean_object* x_73; 
x_72 = lean_ctor_get(x_70, 0);
x_73 = lean_alloc_ctor(7, 3, 0);
lean_ctor_set(x_73, 0, x_62);
lean_ctor_set(x_73, 1, x_66);
lean_ctor_set(x_73, 2, x_72);
lean_ctor_set(x_70, 0, x_73);
return x_68;
}
else
{
lean_object* x_74; lean_object* x_75; lean_object* x_76; lean_object* x_77; 
x_74 = lean_ctor_get(x_70, 0);
x_75 = lean_ctor_get(x_70, 1);
lean_inc(x_75);
lean_inc(x_74);
lean_dec(x_70);
x_76 = lean_alloc_ctor(7, 3, 0);
lean_ctor_set(x_76, 0, x_62);
lean_ctor_set(x_76, 1, x_66);
lean_ctor_set(x_76, 2, x_74);
x_77 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_77, 0, x_76);
lean_ctor_set(x_77, 1, x_75);
lean_ctor_set(x_68, 0, x_77);
return x_68;
}
}
else
{
lean_object* x_78; lean_object* x_79; lean_object* x_80; lean_object* x_81; lean_object* x_82; lean_object* x_83; lean_object* x_84; 
x_78 = lean_ctor_get(x_68, 0);
lean_inc(x_78);
lean_dec(x_68);
x_79 = lean_ctor_get(x_78, 0);
lean_inc(x_79);
x_80 = lean_ctor_get(x_78, 1);
lean_inc(x_80);
if (lean_is_exclusive(x_78)) {
 lean_ctor_release(x_78, 0);
 lean_ctor_release(x_78, 1);
 x_81 = x_78;
} else {
 lean_dec_ref(x_78);
 x_81 = lean_box(0);
}
x_82 = lean_alloc_ctor(7, 3, 0);
lean_ctor_set(x_82, 0, x_62);
lean_ctor_set(x_82, 1, x_66);
lean_ctor_set(x_82, 2, x_79);
if (lean_is_scalar(x_81)) {
 x_83 = lean_alloc_ctor(0, 2, 0);
} else {
 x_83 = x_81;
}
lean_ctor_set(x_83, 0, x_82);
lean_ctor_set(x_83, 1, x_80);
x_84 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_84, 0, x_83);
return x_84;
}
}
}
}
}
}
else
{
lean_object* x_85; 
x_85 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readUInt32LE(x_7);
if (lean_obj_tag(x_85) == 0)
{
lean_object* x_86; 
lean_free_object(x_2);
x_86 = lean_box(0);
return x_86;
}
else
{
uint8_t x_87; 
x_87 = !lean_is_exclusive(x_85);
if (x_87 == 0)
{
lean_object* x_88; uint8_t x_89; 
x_88 = lean_ctor_get(x_85, 0);
x_89 = !lean_is_exclusive(x_88);
if (x_89 == 0)
{
lean_object* x_90; uint32_t x_91; lean_object* x_92; 
x_90 = lean_ctor_get(x_88, 0);
x_91 = lean_unbox_uint32(x_90);
lean_dec(x_90);
x_92 = lean_uint32_to_nat(x_91);
lean_ctor_set_tag(x_2, 6);
lean_ctor_set(x_2, 0, x_92);
lean_ctor_set(x_88, 0, x_2);
return x_85;
}
else
{
lean_object* x_93; lean_object* x_94; uint32_t x_95; lean_object* x_96; lean_object* x_97; 
x_93 = lean_ctor_get(x_88, 0);
x_94 = lean_ctor_get(x_88, 1);
lean_inc(x_94);
lean_inc(x_93);
lean_dec(x_88);
x_95 = lean_unbox_uint32(x_93);
lean_dec(x_93);
x_96 = lean_uint32_to_nat(x_95);
lean_ctor_set_tag(x_2, 6);
lean_ctor_set(x_2, 0, x_96);
x_97 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_97, 0, x_2);
lean_ctor_set(x_97, 1, x_94);
lean_ctor_set(x_85, 0, x_97);
return x_85;
}
}
else
{
lean_object* x_98; lean_object* x_99; lean_object* x_100; lean_object* x_101; uint32_t x_102; lean_object* x_103; lean_object* x_104; lean_object* x_105; 
x_98 = lean_ctor_get(x_85, 0);
lean_inc(x_98);
lean_dec(x_85);
x_99 = lean_ctor_get(x_98, 0);
lean_inc(x_99);
x_100 = lean_ctor_get(x_98, 1);
lean_inc(x_100);
if (lean_is_exclusive(x_98)) {
 lean_ctor_release(x_98, 0);
 lean_ctor_release(x_98, 1);
 x_101 = x_98;
} else {
 lean_dec_ref(x_98);
 x_101 = lean_box(0);
}
x_102 = lean_unbox_uint32(x_99);
lean_dec(x_99);
x_103 = lean_uint32_to_nat(x_102);
lean_ctor_set_tag(x_2, 6);
lean_ctor_set(x_2, 0, x_103);
if (lean_is_scalar(x_101)) {
 x_104 = lean_alloc_ctor(0, 2, 0);
} else {
 x_104 = x_101;
}
lean_ctor_set(x_104, 0, x_2);
lean_ctor_set(x_104, 1, x_100);
x_105 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_105, 0, x_104);
return x_105;
}
}
}
}
else
{
lean_object* x_106; 
x_106 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readUInt32LE(x_7);
if (lean_obj_tag(x_106) == 0)
{
lean_object* x_107; 
lean_free_object(x_2);
x_107 = lean_box(0);
return x_107;
}
else
{
uint8_t x_108; 
x_108 = !lean_is_exclusive(x_106);
if (x_108 == 0)
{
lean_object* x_109; uint8_t x_110; 
x_109 = lean_ctor_get(x_106, 0);
x_110 = !lean_is_exclusive(x_109);
if (x_110 == 0)
{
lean_object* x_111; uint32_t x_112; lean_object* x_113; 
x_111 = lean_ctor_get(x_109, 0);
x_112 = lean_unbox_uint32(x_111);
lean_dec(x_111);
x_113 = lean_uint32_to_nat(x_112);
lean_ctor_set_tag(x_2, 5);
lean_ctor_set(x_2, 0, x_113);
lean_ctor_set(x_109, 0, x_2);
return x_106;
}
else
{
lean_object* x_114; lean_object* x_115; uint32_t x_116; lean_object* x_117; lean_object* x_118; 
x_114 = lean_ctor_get(x_109, 0);
x_115 = lean_ctor_get(x_109, 1);
lean_inc(x_115);
lean_inc(x_114);
lean_dec(x_109);
x_116 = lean_unbox_uint32(x_114);
lean_dec(x_114);
x_117 = lean_uint32_to_nat(x_116);
lean_ctor_set_tag(x_2, 5);
lean_ctor_set(x_2, 0, x_117);
x_118 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_118, 0, x_2);
lean_ctor_set(x_118, 1, x_115);
lean_ctor_set(x_106, 0, x_118);
return x_106;
}
}
else
{
lean_object* x_119; lean_object* x_120; lean_object* x_121; lean_object* x_122; uint32_t x_123; lean_object* x_124; lean_object* x_125; lean_object* x_126; 
x_119 = lean_ctor_get(x_106, 0);
lean_inc(x_119);
lean_dec(x_106);
x_120 = lean_ctor_get(x_119, 0);
lean_inc(x_120);
x_121 = lean_ctor_get(x_119, 1);
lean_inc(x_121);
if (lean_is_exclusive(x_119)) {
 lean_ctor_release(x_119, 0);
 lean_ctor_release(x_119, 1);
 x_122 = x_119;
} else {
 lean_dec_ref(x_119);
 x_122 = lean_box(0);
}
x_123 = lean_unbox_uint32(x_120);
lean_dec(x_120);
x_124 = lean_uint32_to_nat(x_123);
lean_ctor_set_tag(x_2, 5);
lean_ctor_set(x_2, 0, x_124);
if (lean_is_scalar(x_122)) {
 x_125 = lean_alloc_ctor(0, 2, 0);
} else {
 x_125 = x_122;
}
lean_ctor_set(x_125, 0, x_2);
lean_ctor_set(x_125, 1, x_121);
x_126 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_126, 0, x_125);
return x_126;
}
}
}
}
else
{
lean_object* x_127; 
x_127 = lp_iris_x2dkernel_IrisKernel_FFI_decodeLIATerm(x_7);
if (lean_obj_tag(x_127) == 0)
{
lean_free_object(x_2);
return x_127;
}
else
{
uint8_t x_128; 
x_128 = !lean_is_exclusive(x_127);
if (x_128 == 0)
{
lean_object* x_129; uint8_t x_130; 
x_129 = lean_ctor_get(x_127, 0);
x_130 = !lean_is_exclusive(x_129);
if (x_130 == 0)
{
lean_object* x_131; 
x_131 = lean_ctor_get(x_129, 0);
lean_ctor_set_tag(x_2, 4);
lean_ctor_set(x_2, 0, x_131);
lean_ctor_set(x_129, 0, x_2);
return x_127;
}
else
{
lean_object* x_132; lean_object* x_133; lean_object* x_134; 
x_132 = lean_ctor_get(x_129, 0);
x_133 = lean_ctor_get(x_129, 1);
lean_inc(x_133);
lean_inc(x_132);
lean_dec(x_129);
lean_ctor_set_tag(x_2, 4);
lean_ctor_set(x_2, 0, x_132);
x_134 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_134, 0, x_2);
lean_ctor_set(x_134, 1, x_133);
lean_ctor_set(x_127, 0, x_134);
return x_127;
}
}
else
{
lean_object* x_135; lean_object* x_136; lean_object* x_137; lean_object* x_138; lean_object* x_139; lean_object* x_140; 
x_135 = lean_ctor_get(x_127, 0);
lean_inc(x_135);
lean_dec(x_127);
x_136 = lean_ctor_get(x_135, 0);
lean_inc(x_136);
x_137 = lean_ctor_get(x_135, 1);
lean_inc(x_137);
if (lean_is_exclusive(x_135)) {
 lean_ctor_release(x_135, 0);
 lean_ctor_release(x_135, 1);
 x_138 = x_135;
} else {
 lean_dec_ref(x_135);
 x_138 = lean_box(0);
}
lean_ctor_set_tag(x_2, 4);
lean_ctor_set(x_2, 0, x_136);
if (lean_is_scalar(x_138)) {
 x_139 = lean_alloc_ctor(0, 2, 0);
} else {
 x_139 = x_138;
}
lean_ctor_set(x_139, 0, x_2);
lean_ctor_set(x_139, 1, x_137);
x_140 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_140, 0, x_139);
return x_140;
}
}
}
}
else
{
lean_object* x_141; 
lean_free_object(x_2);
x_141 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readInt64LE(x_7);
if (lean_obj_tag(x_141) == 0)
{
lean_object* x_142; 
x_142 = lean_box(0);
return x_142;
}
else
{
lean_object* x_143; uint8_t x_144; 
x_143 = lean_ctor_get(x_141, 0);
lean_inc(x_143);
lean_dec_ref(x_141);
x_144 = !lean_is_exclusive(x_143);
if (x_144 == 0)
{
lean_object* x_145; lean_object* x_146; lean_object* x_147; 
x_145 = lean_ctor_get(x_143, 0);
x_146 = lean_ctor_get(x_143, 1);
x_147 = lp_iris_x2dkernel_IrisKernel_FFI_decodeLIATerm(x_146);
if (lean_obj_tag(x_147) == 0)
{
lean_free_object(x_143);
lean_dec(x_145);
return x_147;
}
else
{
uint8_t x_148; 
x_148 = !lean_is_exclusive(x_147);
if (x_148 == 0)
{
lean_object* x_149; uint8_t x_150; 
x_149 = lean_ctor_get(x_147, 0);
x_150 = !lean_is_exclusive(x_149);
if (x_150 == 0)
{
lean_object* x_151; 
x_151 = lean_ctor_get(x_149, 0);
lean_ctor_set_tag(x_143, 3);
lean_ctor_set(x_143, 1, x_151);
lean_ctor_set(x_149, 0, x_143);
return x_147;
}
else
{
lean_object* x_152; lean_object* x_153; lean_object* x_154; 
x_152 = lean_ctor_get(x_149, 0);
x_153 = lean_ctor_get(x_149, 1);
lean_inc(x_153);
lean_inc(x_152);
lean_dec(x_149);
lean_ctor_set_tag(x_143, 3);
lean_ctor_set(x_143, 1, x_152);
x_154 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_154, 0, x_143);
lean_ctor_set(x_154, 1, x_153);
lean_ctor_set(x_147, 0, x_154);
return x_147;
}
}
else
{
lean_object* x_155; lean_object* x_156; lean_object* x_157; lean_object* x_158; lean_object* x_159; lean_object* x_160; 
x_155 = lean_ctor_get(x_147, 0);
lean_inc(x_155);
lean_dec(x_147);
x_156 = lean_ctor_get(x_155, 0);
lean_inc(x_156);
x_157 = lean_ctor_get(x_155, 1);
lean_inc(x_157);
if (lean_is_exclusive(x_155)) {
 lean_ctor_release(x_155, 0);
 lean_ctor_release(x_155, 1);
 x_158 = x_155;
} else {
 lean_dec_ref(x_155);
 x_158 = lean_box(0);
}
lean_ctor_set_tag(x_143, 3);
lean_ctor_set(x_143, 1, x_156);
if (lean_is_scalar(x_158)) {
 x_159 = lean_alloc_ctor(0, 2, 0);
} else {
 x_159 = x_158;
}
lean_ctor_set(x_159, 0, x_143);
lean_ctor_set(x_159, 1, x_157);
x_160 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_160, 0, x_159);
return x_160;
}
}
}
else
{
lean_object* x_161; lean_object* x_162; lean_object* x_163; 
x_161 = lean_ctor_get(x_143, 0);
x_162 = lean_ctor_get(x_143, 1);
lean_inc(x_162);
lean_inc(x_161);
lean_dec(x_143);
x_163 = lp_iris_x2dkernel_IrisKernel_FFI_decodeLIATerm(x_162);
if (lean_obj_tag(x_163) == 0)
{
lean_dec(x_161);
return x_163;
}
else
{
lean_object* x_164; lean_object* x_165; lean_object* x_166; lean_object* x_167; lean_object* x_168; lean_object* x_169; lean_object* x_170; lean_object* x_171; 
x_164 = lean_ctor_get(x_163, 0);
lean_inc(x_164);
if (lean_is_exclusive(x_163)) {
 lean_ctor_release(x_163, 0);
 x_165 = x_163;
} else {
 lean_dec_ref(x_163);
 x_165 = lean_box(0);
}
x_166 = lean_ctor_get(x_164, 0);
lean_inc(x_166);
x_167 = lean_ctor_get(x_164, 1);
lean_inc(x_167);
if (lean_is_exclusive(x_164)) {
 lean_ctor_release(x_164, 0);
 lean_ctor_release(x_164, 1);
 x_168 = x_164;
} else {
 lean_dec_ref(x_164);
 x_168 = lean_box(0);
}
x_169 = lean_alloc_ctor(3, 2, 0);
lean_ctor_set(x_169, 0, x_161);
lean_ctor_set(x_169, 1, x_166);
if (lean_is_scalar(x_168)) {
 x_170 = lean_alloc_ctor(0, 2, 0);
} else {
 x_170 = x_168;
}
lean_ctor_set(x_170, 0, x_169);
lean_ctor_set(x_170, 1, x_167);
if (lean_is_scalar(x_165)) {
 x_171 = lean_alloc_ctor(1, 1, 0);
} else {
 x_171 = x_165;
}
lean_ctor_set(x_171, 0, x_170);
return x_171;
}
}
}
}
}
else
{
lean_object* x_172; 
lean_free_object(x_2);
x_172 = lp_iris_x2dkernel_IrisKernel_FFI_decodeLIATerm(x_7);
if (lean_obj_tag(x_172) == 0)
{
return x_172;
}
else
{
lean_object* x_173; uint8_t x_174; 
x_173 = lean_ctor_get(x_172, 0);
lean_inc(x_173);
lean_dec_ref(x_172);
x_174 = !lean_is_exclusive(x_173);
if (x_174 == 0)
{
lean_object* x_175; lean_object* x_176; lean_object* x_177; 
x_175 = lean_ctor_get(x_173, 0);
x_176 = lean_ctor_get(x_173, 1);
x_177 = lp_iris_x2dkernel_IrisKernel_FFI_decodeLIATerm(x_176);
if (lean_obj_tag(x_177) == 0)
{
lean_free_object(x_173);
lean_dec(x_175);
return x_177;
}
else
{
uint8_t x_178; 
x_178 = !lean_is_exclusive(x_177);
if (x_178 == 0)
{
lean_object* x_179; uint8_t x_180; 
x_179 = lean_ctor_get(x_177, 0);
x_180 = !lean_is_exclusive(x_179);
if (x_180 == 0)
{
lean_object* x_181; 
x_181 = lean_ctor_get(x_179, 0);
lean_ctor_set_tag(x_173, 2);
lean_ctor_set(x_173, 1, x_181);
lean_ctor_set(x_179, 0, x_173);
return x_177;
}
else
{
lean_object* x_182; lean_object* x_183; lean_object* x_184; 
x_182 = lean_ctor_get(x_179, 0);
x_183 = lean_ctor_get(x_179, 1);
lean_inc(x_183);
lean_inc(x_182);
lean_dec(x_179);
lean_ctor_set_tag(x_173, 2);
lean_ctor_set(x_173, 1, x_182);
x_184 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_184, 0, x_173);
lean_ctor_set(x_184, 1, x_183);
lean_ctor_set(x_177, 0, x_184);
return x_177;
}
}
else
{
lean_object* x_185; lean_object* x_186; lean_object* x_187; lean_object* x_188; lean_object* x_189; lean_object* x_190; 
x_185 = lean_ctor_get(x_177, 0);
lean_inc(x_185);
lean_dec(x_177);
x_186 = lean_ctor_get(x_185, 0);
lean_inc(x_186);
x_187 = lean_ctor_get(x_185, 1);
lean_inc(x_187);
if (lean_is_exclusive(x_185)) {
 lean_ctor_release(x_185, 0);
 lean_ctor_release(x_185, 1);
 x_188 = x_185;
} else {
 lean_dec_ref(x_185);
 x_188 = lean_box(0);
}
lean_ctor_set_tag(x_173, 2);
lean_ctor_set(x_173, 1, x_186);
if (lean_is_scalar(x_188)) {
 x_189 = lean_alloc_ctor(0, 2, 0);
} else {
 x_189 = x_188;
}
lean_ctor_set(x_189, 0, x_173);
lean_ctor_set(x_189, 1, x_187);
x_190 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_190, 0, x_189);
return x_190;
}
}
}
else
{
lean_object* x_191; lean_object* x_192; lean_object* x_193; 
x_191 = lean_ctor_get(x_173, 0);
x_192 = lean_ctor_get(x_173, 1);
lean_inc(x_192);
lean_inc(x_191);
lean_dec(x_173);
x_193 = lp_iris_x2dkernel_IrisKernel_FFI_decodeLIATerm(x_192);
if (lean_obj_tag(x_193) == 0)
{
lean_dec(x_191);
return x_193;
}
else
{
lean_object* x_194; lean_object* x_195; lean_object* x_196; lean_object* x_197; lean_object* x_198; lean_object* x_199; lean_object* x_200; lean_object* x_201; 
x_194 = lean_ctor_get(x_193, 0);
lean_inc(x_194);
if (lean_is_exclusive(x_193)) {
 lean_ctor_release(x_193, 0);
 x_195 = x_193;
} else {
 lean_dec_ref(x_193);
 x_195 = lean_box(0);
}
x_196 = lean_ctor_get(x_194, 0);
lean_inc(x_196);
x_197 = lean_ctor_get(x_194, 1);
lean_inc(x_197);
if (lean_is_exclusive(x_194)) {
 lean_ctor_release(x_194, 0);
 lean_ctor_release(x_194, 1);
 x_198 = x_194;
} else {
 lean_dec_ref(x_194);
 x_198 = lean_box(0);
}
x_199 = lean_alloc_ctor(2, 2, 0);
lean_ctor_set(x_199, 0, x_191);
lean_ctor_set(x_199, 1, x_196);
if (lean_is_scalar(x_198)) {
 x_200 = lean_alloc_ctor(0, 2, 0);
} else {
 x_200 = x_198;
}
lean_ctor_set(x_200, 0, x_199);
lean_ctor_set(x_200, 1, x_197);
if (lean_is_scalar(x_195)) {
 x_201 = lean_alloc_ctor(1, 1, 0);
} else {
 x_201 = x_195;
}
lean_ctor_set(x_201, 0, x_200);
return x_201;
}
}
}
}
}
else
{
lean_object* x_202; 
x_202 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readInt64LE(x_7);
if (lean_obj_tag(x_202) == 0)
{
lean_object* x_203; 
lean_free_object(x_2);
x_203 = lean_box(0);
return x_203;
}
else
{
uint8_t x_204; 
x_204 = !lean_is_exclusive(x_202);
if (x_204 == 0)
{
lean_object* x_205; uint8_t x_206; 
x_205 = lean_ctor_get(x_202, 0);
x_206 = !lean_is_exclusive(x_205);
if (x_206 == 0)
{
lean_object* x_207; 
x_207 = lean_ctor_get(x_205, 0);
lean_ctor_set(x_2, 0, x_207);
lean_ctor_set(x_205, 0, x_2);
return x_202;
}
else
{
lean_object* x_208; lean_object* x_209; lean_object* x_210; 
x_208 = lean_ctor_get(x_205, 0);
x_209 = lean_ctor_get(x_205, 1);
lean_inc(x_209);
lean_inc(x_208);
lean_dec(x_205);
lean_ctor_set(x_2, 0, x_208);
x_210 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_210, 0, x_2);
lean_ctor_set(x_210, 1, x_209);
lean_ctor_set(x_202, 0, x_210);
return x_202;
}
}
else
{
lean_object* x_211; lean_object* x_212; lean_object* x_213; lean_object* x_214; lean_object* x_215; lean_object* x_216; 
x_211 = lean_ctor_get(x_202, 0);
lean_inc(x_211);
lean_dec(x_202);
x_212 = lean_ctor_get(x_211, 0);
lean_inc(x_212);
x_213 = lean_ctor_get(x_211, 1);
lean_inc(x_213);
if (lean_is_exclusive(x_211)) {
 lean_ctor_release(x_211, 0);
 lean_ctor_release(x_211, 1);
 x_214 = x_211;
} else {
 lean_dec_ref(x_211);
 x_214 = lean_box(0);
}
lean_ctor_set(x_2, 0, x_212);
if (lean_is_scalar(x_214)) {
 x_215 = lean_alloc_ctor(0, 2, 0);
} else {
 x_215 = x_214;
}
lean_ctor_set(x_215, 0, x_2);
lean_ctor_set(x_215, 1, x_213);
x_216 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_216, 0, x_215);
return x_216;
}
}
}
}
else
{
lean_object* x_217; 
x_217 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readUInt32LE(x_7);
if (lean_obj_tag(x_217) == 0)
{
lean_object* x_218; 
lean_free_object(x_2);
x_218 = lean_box(0);
return x_218;
}
else
{
uint8_t x_219; 
x_219 = !lean_is_exclusive(x_217);
if (x_219 == 0)
{
lean_object* x_220; uint8_t x_221; 
x_220 = lean_ctor_get(x_217, 0);
x_221 = !lean_is_exclusive(x_220);
if (x_221 == 0)
{
lean_object* x_222; uint32_t x_223; lean_object* x_224; 
x_222 = lean_ctor_get(x_220, 0);
x_223 = lean_unbox_uint32(x_222);
lean_dec(x_222);
x_224 = lean_uint32_to_nat(x_223);
lean_ctor_set_tag(x_2, 0);
lean_ctor_set(x_2, 0, x_224);
lean_ctor_set(x_220, 0, x_2);
return x_217;
}
else
{
lean_object* x_225; lean_object* x_226; uint32_t x_227; lean_object* x_228; lean_object* x_229; 
x_225 = lean_ctor_get(x_220, 0);
x_226 = lean_ctor_get(x_220, 1);
lean_inc(x_226);
lean_inc(x_225);
lean_dec(x_220);
x_227 = lean_unbox_uint32(x_225);
lean_dec(x_225);
x_228 = lean_uint32_to_nat(x_227);
lean_ctor_set_tag(x_2, 0);
lean_ctor_set(x_2, 0, x_228);
x_229 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_229, 0, x_2);
lean_ctor_set(x_229, 1, x_226);
lean_ctor_set(x_217, 0, x_229);
return x_217;
}
}
else
{
lean_object* x_230; lean_object* x_231; lean_object* x_232; lean_object* x_233; uint32_t x_234; lean_object* x_235; lean_object* x_236; lean_object* x_237; 
x_230 = lean_ctor_get(x_217, 0);
lean_inc(x_230);
lean_dec(x_217);
x_231 = lean_ctor_get(x_230, 0);
lean_inc(x_231);
x_232 = lean_ctor_get(x_230, 1);
lean_inc(x_232);
if (lean_is_exclusive(x_230)) {
 lean_ctor_release(x_230, 0);
 lean_ctor_release(x_230, 1);
 x_233 = x_230;
} else {
 lean_dec_ref(x_230);
 x_233 = lean_box(0);
}
x_234 = lean_unbox_uint32(x_231);
lean_dec(x_231);
x_235 = lean_uint32_to_nat(x_234);
lean_ctor_set_tag(x_2, 0);
lean_ctor_set(x_2, 0, x_235);
if (lean_is_scalar(x_233)) {
 x_236 = lean_alloc_ctor(0, 2, 0);
} else {
 x_236 = x_233;
}
lean_ctor_set(x_236, 0, x_2);
lean_ctor_set(x_236, 1, x_232);
x_237 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_237, 0, x_236);
return x_237;
}
}
}
}
else
{
lean_object* x_238; lean_object* x_239; lean_object* x_240; uint8_t x_241; lean_object* x_242; lean_object* x_243; uint8_t x_244; 
x_238 = lean_ctor_get(x_2, 0);
lean_inc(x_238);
lean_dec(x_2);
x_239 = lean_ctor_get(x_238, 0);
lean_inc(x_239);
x_240 = lean_ctor_get(x_238, 1);
lean_inc(x_240);
lean_dec(x_238);
x_241 = lean_unbox(x_239);
lean_dec(x_239);
x_242 = lean_uint8_to_nat(x_241);
x_243 = lean_unsigned_to_nat(0u);
x_244 = lean_nat_dec_eq(x_242, x_243);
if (x_244 == 0)
{
lean_object* x_245; uint8_t x_246; 
x_245 = lean_unsigned_to_nat(1u);
x_246 = lean_nat_dec_eq(x_242, x_245);
if (x_246 == 0)
{
lean_object* x_247; uint8_t x_248; 
x_247 = lean_unsigned_to_nat(2u);
x_248 = lean_nat_dec_eq(x_242, x_247);
if (x_248 == 0)
{
lean_object* x_249; uint8_t x_250; 
x_249 = lean_unsigned_to_nat(3u);
x_250 = lean_nat_dec_eq(x_242, x_249);
if (x_250 == 0)
{
lean_object* x_251; uint8_t x_252; 
x_251 = lean_unsigned_to_nat(4u);
x_252 = lean_nat_dec_eq(x_242, x_251);
if (x_252 == 0)
{
lean_object* x_253; uint8_t x_254; 
x_253 = lean_unsigned_to_nat(5u);
x_254 = lean_nat_dec_eq(x_242, x_253);
if (x_254 == 0)
{
lean_object* x_255; uint8_t x_256; 
x_255 = lean_unsigned_to_nat(6u);
x_256 = lean_nat_dec_eq(x_242, x_255);
if (x_256 == 0)
{
lean_object* x_257; uint8_t x_258; 
x_257 = lean_unsigned_to_nat(7u);
x_258 = lean_nat_dec_eq(x_242, x_257);
if (x_258 == 0)
{
lean_object* x_259; uint8_t x_260; 
x_259 = lean_unsigned_to_nat(8u);
x_260 = lean_nat_dec_eq(x_242, x_259);
if (x_260 == 0)
{
lean_object* x_261; 
lean_dec(x_240);
x_261 = lean_box(0);
return x_261;
}
else
{
lean_object* x_262; 
x_262 = lp_iris_x2dkernel_IrisKernel_FFI_decodeLIATerm(x_240);
if (lean_obj_tag(x_262) == 0)
{
return x_262;
}
else
{
lean_object* x_263; lean_object* x_264; lean_object* x_265; lean_object* x_266; lean_object* x_267; 
x_263 = lean_ctor_get(x_262, 0);
lean_inc(x_263);
lean_dec_ref(x_262);
x_264 = lean_ctor_get(x_263, 0);
lean_inc(x_264);
x_265 = lean_ctor_get(x_263, 1);
lean_inc(x_265);
if (lean_is_exclusive(x_263)) {
 lean_ctor_release(x_263, 0);
 lean_ctor_release(x_263, 1);
 x_266 = x_263;
} else {
 lean_dec_ref(x_263);
 x_266 = lean_box(0);
}
x_267 = lp_iris_x2dkernel_IrisKernel_FFI_decodeLIATerm(x_265);
if (lean_obj_tag(x_267) == 0)
{
lean_dec(x_266);
lean_dec(x_264);
return x_267;
}
else
{
lean_object* x_268; lean_object* x_269; lean_object* x_270; lean_object* x_271; lean_object* x_272; lean_object* x_273; lean_object* x_274; lean_object* x_275; 
x_268 = lean_ctor_get(x_267, 0);
lean_inc(x_268);
if (lean_is_exclusive(x_267)) {
 lean_ctor_release(x_267, 0);
 x_269 = x_267;
} else {
 lean_dec_ref(x_267);
 x_269 = lean_box(0);
}
x_270 = lean_ctor_get(x_268, 0);
lean_inc(x_270);
x_271 = lean_ctor_get(x_268, 1);
lean_inc(x_271);
if (lean_is_exclusive(x_268)) {
 lean_ctor_release(x_268, 0);
 lean_ctor_release(x_268, 1);
 x_272 = x_268;
} else {
 lean_dec_ref(x_268);
 x_272 = lean_box(0);
}
if (lean_is_scalar(x_266)) {
 x_273 = lean_alloc_ctor(8, 2, 0);
} else {
 x_273 = x_266;
 lean_ctor_set_tag(x_273, 8);
}
lean_ctor_set(x_273, 0, x_264);
lean_ctor_set(x_273, 1, x_270);
if (lean_is_scalar(x_272)) {
 x_274 = lean_alloc_ctor(0, 2, 0);
} else {
 x_274 = x_272;
}
lean_ctor_set(x_274, 0, x_273);
lean_ctor_set(x_274, 1, x_271);
if (lean_is_scalar(x_269)) {
 x_275 = lean_alloc_ctor(1, 1, 0);
} else {
 x_275 = x_269;
}
lean_ctor_set(x_275, 0, x_274);
return x_275;
}
}
}
}
else
{
lean_object* x_276; 
x_276 = lp_iris_x2dkernel_IrisKernel_FFI_decodeLIAFormula(x_240);
if (lean_obj_tag(x_276) == 0)
{
lean_object* x_277; 
x_277 = lean_box(0);
return x_277;
}
else
{
lean_object* x_278; lean_object* x_279; lean_object* x_280; lean_object* x_281; 
x_278 = lean_ctor_get(x_276, 0);
lean_inc(x_278);
lean_dec_ref(x_276);
x_279 = lean_ctor_get(x_278, 0);
lean_inc(x_279);
x_280 = lean_ctor_get(x_278, 1);
lean_inc(x_280);
lean_dec(x_278);
x_281 = lp_iris_x2dkernel_IrisKernel_FFI_decodeLIATerm(x_280);
if (lean_obj_tag(x_281) == 0)
{
lean_dec(x_279);
return x_281;
}
else
{
lean_object* x_282; lean_object* x_283; lean_object* x_284; lean_object* x_285; 
x_282 = lean_ctor_get(x_281, 0);
lean_inc(x_282);
lean_dec_ref(x_281);
x_283 = lean_ctor_get(x_282, 0);
lean_inc(x_283);
x_284 = lean_ctor_get(x_282, 1);
lean_inc(x_284);
lean_dec(x_282);
x_285 = lp_iris_x2dkernel_IrisKernel_FFI_decodeLIATerm(x_284);
if (lean_obj_tag(x_285) == 0)
{
lean_dec(x_283);
lean_dec(x_279);
return x_285;
}
else
{
lean_object* x_286; lean_object* x_287; lean_object* x_288; lean_object* x_289; lean_object* x_290; lean_object* x_291; lean_object* x_292; lean_object* x_293; 
x_286 = lean_ctor_get(x_285, 0);
lean_inc(x_286);
if (lean_is_exclusive(x_285)) {
 lean_ctor_release(x_285, 0);
 x_287 = x_285;
} else {
 lean_dec_ref(x_285);
 x_287 = lean_box(0);
}
x_288 = lean_ctor_get(x_286, 0);
lean_inc(x_288);
x_289 = lean_ctor_get(x_286, 1);
lean_inc(x_289);
if (lean_is_exclusive(x_286)) {
 lean_ctor_release(x_286, 0);
 lean_ctor_release(x_286, 1);
 x_290 = x_286;
} else {
 lean_dec_ref(x_286);
 x_290 = lean_box(0);
}
x_291 = lean_alloc_ctor(7, 3, 0);
lean_ctor_set(x_291, 0, x_279);
lean_ctor_set(x_291, 1, x_283);
lean_ctor_set(x_291, 2, x_288);
if (lean_is_scalar(x_290)) {
 x_292 = lean_alloc_ctor(0, 2, 0);
} else {
 x_292 = x_290;
}
lean_ctor_set(x_292, 0, x_291);
lean_ctor_set(x_292, 1, x_289);
if (lean_is_scalar(x_287)) {
 x_293 = lean_alloc_ctor(1, 1, 0);
} else {
 x_293 = x_287;
}
lean_ctor_set(x_293, 0, x_292);
return x_293;
}
}
}
}
}
else
{
lean_object* x_294; 
x_294 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readUInt32LE(x_240);
if (lean_obj_tag(x_294) == 0)
{
lean_object* x_295; 
x_295 = lean_box(0);
return x_295;
}
else
{
lean_object* x_296; lean_object* x_297; lean_object* x_298; lean_object* x_299; lean_object* x_300; uint32_t x_301; lean_object* x_302; lean_object* x_303; lean_object* x_304; lean_object* x_305; 
x_296 = lean_ctor_get(x_294, 0);
lean_inc(x_296);
if (lean_is_exclusive(x_294)) {
 lean_ctor_release(x_294, 0);
 x_297 = x_294;
} else {
 lean_dec_ref(x_294);
 x_297 = lean_box(0);
}
x_298 = lean_ctor_get(x_296, 0);
lean_inc(x_298);
x_299 = lean_ctor_get(x_296, 1);
lean_inc(x_299);
if (lean_is_exclusive(x_296)) {
 lean_ctor_release(x_296, 0);
 lean_ctor_release(x_296, 1);
 x_300 = x_296;
} else {
 lean_dec_ref(x_296);
 x_300 = lean_box(0);
}
x_301 = lean_unbox_uint32(x_298);
lean_dec(x_298);
x_302 = lean_uint32_to_nat(x_301);
x_303 = lean_alloc_ctor(6, 1, 0);
lean_ctor_set(x_303, 0, x_302);
if (lean_is_scalar(x_300)) {
 x_304 = lean_alloc_ctor(0, 2, 0);
} else {
 x_304 = x_300;
}
lean_ctor_set(x_304, 0, x_303);
lean_ctor_set(x_304, 1, x_299);
if (lean_is_scalar(x_297)) {
 x_305 = lean_alloc_ctor(1, 1, 0);
} else {
 x_305 = x_297;
}
lean_ctor_set(x_305, 0, x_304);
return x_305;
}
}
}
else
{
lean_object* x_306; 
x_306 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readUInt32LE(x_240);
if (lean_obj_tag(x_306) == 0)
{
lean_object* x_307; 
x_307 = lean_box(0);
return x_307;
}
else
{
lean_object* x_308; lean_object* x_309; lean_object* x_310; lean_object* x_311; lean_object* x_312; uint32_t x_313; lean_object* x_314; lean_object* x_315; lean_object* x_316; lean_object* x_317; 
x_308 = lean_ctor_get(x_306, 0);
lean_inc(x_308);
if (lean_is_exclusive(x_306)) {
 lean_ctor_release(x_306, 0);
 x_309 = x_306;
} else {
 lean_dec_ref(x_306);
 x_309 = lean_box(0);
}
x_310 = lean_ctor_get(x_308, 0);
lean_inc(x_310);
x_311 = lean_ctor_get(x_308, 1);
lean_inc(x_311);
if (lean_is_exclusive(x_308)) {
 lean_ctor_release(x_308, 0);
 lean_ctor_release(x_308, 1);
 x_312 = x_308;
} else {
 lean_dec_ref(x_308);
 x_312 = lean_box(0);
}
x_313 = lean_unbox_uint32(x_310);
lean_dec(x_310);
x_314 = lean_uint32_to_nat(x_313);
x_315 = lean_alloc_ctor(5, 1, 0);
lean_ctor_set(x_315, 0, x_314);
if (lean_is_scalar(x_312)) {
 x_316 = lean_alloc_ctor(0, 2, 0);
} else {
 x_316 = x_312;
}
lean_ctor_set(x_316, 0, x_315);
lean_ctor_set(x_316, 1, x_311);
if (lean_is_scalar(x_309)) {
 x_317 = lean_alloc_ctor(1, 1, 0);
} else {
 x_317 = x_309;
}
lean_ctor_set(x_317, 0, x_316);
return x_317;
}
}
}
else
{
lean_object* x_318; 
x_318 = lp_iris_x2dkernel_IrisKernel_FFI_decodeLIATerm(x_240);
if (lean_obj_tag(x_318) == 0)
{
return x_318;
}
else
{
lean_object* x_319; lean_object* x_320; lean_object* x_321; lean_object* x_322; lean_object* x_323; lean_object* x_324; lean_object* x_325; lean_object* x_326; 
x_319 = lean_ctor_get(x_318, 0);
lean_inc(x_319);
if (lean_is_exclusive(x_318)) {
 lean_ctor_release(x_318, 0);
 x_320 = x_318;
} else {
 lean_dec_ref(x_318);
 x_320 = lean_box(0);
}
x_321 = lean_ctor_get(x_319, 0);
lean_inc(x_321);
x_322 = lean_ctor_get(x_319, 1);
lean_inc(x_322);
if (lean_is_exclusive(x_319)) {
 lean_ctor_release(x_319, 0);
 lean_ctor_release(x_319, 1);
 x_323 = x_319;
} else {
 lean_dec_ref(x_319);
 x_323 = lean_box(0);
}
x_324 = lean_alloc_ctor(4, 1, 0);
lean_ctor_set(x_324, 0, x_321);
if (lean_is_scalar(x_323)) {
 x_325 = lean_alloc_ctor(0, 2, 0);
} else {
 x_325 = x_323;
}
lean_ctor_set(x_325, 0, x_324);
lean_ctor_set(x_325, 1, x_322);
if (lean_is_scalar(x_320)) {
 x_326 = lean_alloc_ctor(1, 1, 0);
} else {
 x_326 = x_320;
}
lean_ctor_set(x_326, 0, x_325);
return x_326;
}
}
}
else
{
lean_object* x_327; 
x_327 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readInt64LE(x_240);
if (lean_obj_tag(x_327) == 0)
{
lean_object* x_328; 
x_328 = lean_box(0);
return x_328;
}
else
{
lean_object* x_329; lean_object* x_330; lean_object* x_331; lean_object* x_332; lean_object* x_333; 
x_329 = lean_ctor_get(x_327, 0);
lean_inc(x_329);
lean_dec_ref(x_327);
x_330 = lean_ctor_get(x_329, 0);
lean_inc(x_330);
x_331 = lean_ctor_get(x_329, 1);
lean_inc(x_331);
if (lean_is_exclusive(x_329)) {
 lean_ctor_release(x_329, 0);
 lean_ctor_release(x_329, 1);
 x_332 = x_329;
} else {
 lean_dec_ref(x_329);
 x_332 = lean_box(0);
}
x_333 = lp_iris_x2dkernel_IrisKernel_FFI_decodeLIATerm(x_331);
if (lean_obj_tag(x_333) == 0)
{
lean_dec(x_332);
lean_dec(x_330);
return x_333;
}
else
{
lean_object* x_334; lean_object* x_335; lean_object* x_336; lean_object* x_337; lean_object* x_338; lean_object* x_339; lean_object* x_340; lean_object* x_341; 
x_334 = lean_ctor_get(x_333, 0);
lean_inc(x_334);
if (lean_is_exclusive(x_333)) {
 lean_ctor_release(x_333, 0);
 x_335 = x_333;
} else {
 lean_dec_ref(x_333);
 x_335 = lean_box(0);
}
x_336 = lean_ctor_get(x_334, 0);
lean_inc(x_336);
x_337 = lean_ctor_get(x_334, 1);
lean_inc(x_337);
if (lean_is_exclusive(x_334)) {
 lean_ctor_release(x_334, 0);
 lean_ctor_release(x_334, 1);
 x_338 = x_334;
} else {
 lean_dec_ref(x_334);
 x_338 = lean_box(0);
}
if (lean_is_scalar(x_332)) {
 x_339 = lean_alloc_ctor(3, 2, 0);
} else {
 x_339 = x_332;
 lean_ctor_set_tag(x_339, 3);
}
lean_ctor_set(x_339, 0, x_330);
lean_ctor_set(x_339, 1, x_336);
if (lean_is_scalar(x_338)) {
 x_340 = lean_alloc_ctor(0, 2, 0);
} else {
 x_340 = x_338;
}
lean_ctor_set(x_340, 0, x_339);
lean_ctor_set(x_340, 1, x_337);
if (lean_is_scalar(x_335)) {
 x_341 = lean_alloc_ctor(1, 1, 0);
} else {
 x_341 = x_335;
}
lean_ctor_set(x_341, 0, x_340);
return x_341;
}
}
}
}
else
{
lean_object* x_342; 
x_342 = lp_iris_x2dkernel_IrisKernel_FFI_decodeLIATerm(x_240);
if (lean_obj_tag(x_342) == 0)
{
return x_342;
}
else
{
lean_object* x_343; lean_object* x_344; lean_object* x_345; lean_object* x_346; lean_object* x_347; 
x_343 = lean_ctor_get(x_342, 0);
lean_inc(x_343);
lean_dec_ref(x_342);
x_344 = lean_ctor_get(x_343, 0);
lean_inc(x_344);
x_345 = lean_ctor_get(x_343, 1);
lean_inc(x_345);
if (lean_is_exclusive(x_343)) {
 lean_ctor_release(x_343, 0);
 lean_ctor_release(x_343, 1);
 x_346 = x_343;
} else {
 lean_dec_ref(x_343);
 x_346 = lean_box(0);
}
x_347 = lp_iris_x2dkernel_IrisKernel_FFI_decodeLIATerm(x_345);
if (lean_obj_tag(x_347) == 0)
{
lean_dec(x_346);
lean_dec(x_344);
return x_347;
}
else
{
lean_object* x_348; lean_object* x_349; lean_object* x_350; lean_object* x_351; lean_object* x_352; lean_object* x_353; lean_object* x_354; lean_object* x_355; 
x_348 = lean_ctor_get(x_347, 0);
lean_inc(x_348);
if (lean_is_exclusive(x_347)) {
 lean_ctor_release(x_347, 0);
 x_349 = x_347;
} else {
 lean_dec_ref(x_347);
 x_349 = lean_box(0);
}
x_350 = lean_ctor_get(x_348, 0);
lean_inc(x_350);
x_351 = lean_ctor_get(x_348, 1);
lean_inc(x_351);
if (lean_is_exclusive(x_348)) {
 lean_ctor_release(x_348, 0);
 lean_ctor_release(x_348, 1);
 x_352 = x_348;
} else {
 lean_dec_ref(x_348);
 x_352 = lean_box(0);
}
if (lean_is_scalar(x_346)) {
 x_353 = lean_alloc_ctor(2, 2, 0);
} else {
 x_353 = x_346;
 lean_ctor_set_tag(x_353, 2);
}
lean_ctor_set(x_353, 0, x_344);
lean_ctor_set(x_353, 1, x_350);
if (lean_is_scalar(x_352)) {
 x_354 = lean_alloc_ctor(0, 2, 0);
} else {
 x_354 = x_352;
}
lean_ctor_set(x_354, 0, x_353);
lean_ctor_set(x_354, 1, x_351);
if (lean_is_scalar(x_349)) {
 x_355 = lean_alloc_ctor(1, 1, 0);
} else {
 x_355 = x_349;
}
lean_ctor_set(x_355, 0, x_354);
return x_355;
}
}
}
}
else
{
lean_object* x_356; 
x_356 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readInt64LE(x_240);
if (lean_obj_tag(x_356) == 0)
{
lean_object* x_357; 
x_357 = lean_box(0);
return x_357;
}
else
{
lean_object* x_358; lean_object* x_359; lean_object* x_360; lean_object* x_361; lean_object* x_362; lean_object* x_363; lean_object* x_364; lean_object* x_365; 
x_358 = lean_ctor_get(x_356, 0);
lean_inc(x_358);
if (lean_is_exclusive(x_356)) {
 lean_ctor_release(x_356, 0);
 x_359 = x_356;
} else {
 lean_dec_ref(x_356);
 x_359 = lean_box(0);
}
x_360 = lean_ctor_get(x_358, 0);
lean_inc(x_360);
x_361 = lean_ctor_get(x_358, 1);
lean_inc(x_361);
if (lean_is_exclusive(x_358)) {
 lean_ctor_release(x_358, 0);
 lean_ctor_release(x_358, 1);
 x_362 = x_358;
} else {
 lean_dec_ref(x_358);
 x_362 = lean_box(0);
}
x_363 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_363, 0, x_360);
if (lean_is_scalar(x_362)) {
 x_364 = lean_alloc_ctor(0, 2, 0);
} else {
 x_364 = x_362;
}
lean_ctor_set(x_364, 0, x_363);
lean_ctor_set(x_364, 1, x_361);
if (lean_is_scalar(x_359)) {
 x_365 = lean_alloc_ctor(1, 1, 0);
} else {
 x_365 = x_359;
}
lean_ctor_set(x_365, 0, x_364);
return x_365;
}
}
}
else
{
lean_object* x_366; 
x_366 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readUInt32LE(x_240);
if (lean_obj_tag(x_366) == 0)
{
lean_object* x_367; 
x_367 = lean_box(0);
return x_367;
}
else
{
lean_object* x_368; lean_object* x_369; lean_object* x_370; lean_object* x_371; lean_object* x_372; uint32_t x_373; lean_object* x_374; lean_object* x_375; lean_object* x_376; lean_object* x_377; 
x_368 = lean_ctor_get(x_366, 0);
lean_inc(x_368);
if (lean_is_exclusive(x_366)) {
 lean_ctor_release(x_366, 0);
 x_369 = x_366;
} else {
 lean_dec_ref(x_366);
 x_369 = lean_box(0);
}
x_370 = lean_ctor_get(x_368, 0);
lean_inc(x_370);
x_371 = lean_ctor_get(x_368, 1);
lean_inc(x_371);
if (lean_is_exclusive(x_368)) {
 lean_ctor_release(x_368, 0);
 lean_ctor_release(x_368, 1);
 x_372 = x_368;
} else {
 lean_dec_ref(x_368);
 x_372 = lean_box(0);
}
x_373 = lean_unbox_uint32(x_370);
lean_dec(x_370);
x_374 = lean_uint32_to_nat(x_373);
x_375 = lean_alloc_ctor(0, 1, 0);
lean_ctor_set(x_375, 0, x_374);
if (lean_is_scalar(x_372)) {
 x_376 = lean_alloc_ctor(0, 2, 0);
} else {
 x_376 = x_372;
}
lean_ctor_set(x_376, 0, x_375);
lean_ctor_set(x_376, 1, x_371);
if (lean_is_scalar(x_369)) {
 x_377 = lean_alloc_ctor(1, 1, 0);
} else {
 x_377 = x_369;
}
lean_ctor_set(x_377, 0, x_376);
return x_377;
}
}
}
}
}
}
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_decodeLIAAtom(lean_object* x_1) {
_start:
{
lean_object* x_2; 
x_2 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readUInt8(x_1);
if (lean_obj_tag(x_2) == 0)
{
lean_object* x_3; 
x_3 = lean_box(0);
return x_3;
}
else
{
lean_object* x_4; lean_object* x_5; lean_object* x_6; uint8_t x_7; lean_object* x_8; lean_object* x_9; uint8_t x_10; 
x_4 = lean_ctor_get(x_2, 0);
lean_inc(x_4);
lean_dec_ref(x_2);
x_5 = lean_ctor_get(x_4, 0);
lean_inc(x_5);
x_6 = lean_ctor_get(x_4, 1);
lean_inc(x_6);
lean_dec(x_4);
x_7 = lean_unbox(x_5);
lean_dec(x_5);
x_8 = lean_uint8_to_nat(x_7);
x_9 = lean_unsigned_to_nat(0u);
x_10 = lean_nat_dec_eq(x_8, x_9);
if (x_10 == 0)
{
lean_object* x_11; uint8_t x_12; 
x_11 = lean_unsigned_to_nat(1u);
x_12 = lean_nat_dec_eq(x_8, x_11);
if (x_12 == 0)
{
lean_object* x_13; uint8_t x_14; 
x_13 = lean_unsigned_to_nat(2u);
x_14 = lean_nat_dec_eq(x_8, x_13);
if (x_14 == 0)
{
lean_object* x_15; uint8_t x_16; 
x_15 = lean_unsigned_to_nat(3u);
x_16 = lean_nat_dec_eq(x_8, x_15);
if (x_16 == 0)
{
lean_object* x_17; 
lean_dec(x_6);
x_17 = lean_box(0);
return x_17;
}
else
{
lean_object* x_18; 
x_18 = lp_iris_x2dkernel_IrisKernel_FFI_decodeLIATerm(x_6);
if (lean_obj_tag(x_18) == 0)
{
lean_object* x_19; 
x_19 = lean_box(0);
return x_19;
}
else
{
lean_object* x_20; uint8_t x_21; 
x_20 = lean_ctor_get(x_18, 0);
lean_inc(x_20);
lean_dec_ref(x_18);
x_21 = !lean_is_exclusive(x_20);
if (x_21 == 0)
{
lean_object* x_22; lean_object* x_23; lean_object* x_24; 
x_22 = lean_ctor_get(x_20, 0);
x_23 = lean_ctor_get(x_20, 1);
x_24 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readUInt64LE(x_23);
if (lean_obj_tag(x_24) == 0)
{
lean_object* x_25; 
lean_free_object(x_20);
lean_dec(x_22);
x_25 = lean_box(0);
return x_25;
}
else
{
uint8_t x_26; 
x_26 = !lean_is_exclusive(x_24);
if (x_26 == 0)
{
lean_object* x_27; uint8_t x_28; 
x_27 = lean_ctor_get(x_24, 0);
x_28 = !lean_is_exclusive(x_27);
if (x_28 == 0)
{
lean_object* x_29; uint64_t x_30; lean_object* x_31; 
x_29 = lean_ctor_get(x_27, 0);
x_30 = lean_unbox_uint64(x_29);
lean_dec(x_29);
x_31 = lean_uint64_to_nat(x_30);
lean_ctor_set_tag(x_20, 3);
lean_ctor_set(x_20, 1, x_31);
lean_ctor_set(x_27, 0, x_20);
return x_24;
}
else
{
lean_object* x_32; lean_object* x_33; uint64_t x_34; lean_object* x_35; lean_object* x_36; 
x_32 = lean_ctor_get(x_27, 0);
x_33 = lean_ctor_get(x_27, 1);
lean_inc(x_33);
lean_inc(x_32);
lean_dec(x_27);
x_34 = lean_unbox_uint64(x_32);
lean_dec(x_32);
x_35 = lean_uint64_to_nat(x_34);
lean_ctor_set_tag(x_20, 3);
lean_ctor_set(x_20, 1, x_35);
x_36 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_36, 0, x_20);
lean_ctor_set(x_36, 1, x_33);
lean_ctor_set(x_24, 0, x_36);
return x_24;
}
}
else
{
lean_object* x_37; lean_object* x_38; lean_object* x_39; lean_object* x_40; uint64_t x_41; lean_object* x_42; lean_object* x_43; lean_object* x_44; 
x_37 = lean_ctor_get(x_24, 0);
lean_inc(x_37);
lean_dec(x_24);
x_38 = lean_ctor_get(x_37, 0);
lean_inc(x_38);
x_39 = lean_ctor_get(x_37, 1);
lean_inc(x_39);
if (lean_is_exclusive(x_37)) {
 lean_ctor_release(x_37, 0);
 lean_ctor_release(x_37, 1);
 x_40 = x_37;
} else {
 lean_dec_ref(x_37);
 x_40 = lean_box(0);
}
x_41 = lean_unbox_uint64(x_38);
lean_dec(x_38);
x_42 = lean_uint64_to_nat(x_41);
lean_ctor_set_tag(x_20, 3);
lean_ctor_set(x_20, 1, x_42);
if (lean_is_scalar(x_40)) {
 x_43 = lean_alloc_ctor(0, 2, 0);
} else {
 x_43 = x_40;
}
lean_ctor_set(x_43, 0, x_20);
lean_ctor_set(x_43, 1, x_39);
x_44 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_44, 0, x_43);
return x_44;
}
}
}
else
{
lean_object* x_45; lean_object* x_46; lean_object* x_47; 
x_45 = lean_ctor_get(x_20, 0);
x_46 = lean_ctor_get(x_20, 1);
lean_inc(x_46);
lean_inc(x_45);
lean_dec(x_20);
x_47 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readUInt64LE(x_46);
if (lean_obj_tag(x_47) == 0)
{
lean_object* x_48; 
lean_dec(x_45);
x_48 = lean_box(0);
return x_48;
}
else
{
lean_object* x_49; lean_object* x_50; lean_object* x_51; lean_object* x_52; lean_object* x_53; uint64_t x_54; lean_object* x_55; lean_object* x_56; lean_object* x_57; lean_object* x_58; 
x_49 = lean_ctor_get(x_47, 0);
lean_inc(x_49);
if (lean_is_exclusive(x_47)) {
 lean_ctor_release(x_47, 0);
 x_50 = x_47;
} else {
 lean_dec_ref(x_47);
 x_50 = lean_box(0);
}
x_51 = lean_ctor_get(x_49, 0);
lean_inc(x_51);
x_52 = lean_ctor_get(x_49, 1);
lean_inc(x_52);
if (lean_is_exclusive(x_49)) {
 lean_ctor_release(x_49, 0);
 lean_ctor_release(x_49, 1);
 x_53 = x_49;
} else {
 lean_dec_ref(x_49);
 x_53 = lean_box(0);
}
x_54 = lean_unbox_uint64(x_51);
lean_dec(x_51);
x_55 = lean_uint64_to_nat(x_54);
x_56 = lean_alloc_ctor(3, 2, 0);
lean_ctor_set(x_56, 0, x_45);
lean_ctor_set(x_56, 1, x_55);
if (lean_is_scalar(x_53)) {
 x_57 = lean_alloc_ctor(0, 2, 0);
} else {
 x_57 = x_53;
}
lean_ctor_set(x_57, 0, x_56);
lean_ctor_set(x_57, 1, x_52);
if (lean_is_scalar(x_50)) {
 x_58 = lean_alloc_ctor(1, 1, 0);
} else {
 x_58 = x_50;
}
lean_ctor_set(x_58, 0, x_57);
return x_58;
}
}
}
}
}
else
{
lean_object* x_59; 
x_59 = lp_iris_x2dkernel_IrisKernel_FFI_decodeLIATerm(x_6);
if (lean_obj_tag(x_59) == 0)
{
lean_object* x_60; 
x_60 = lean_box(0);
return x_60;
}
else
{
lean_object* x_61; uint8_t x_62; 
x_61 = lean_ctor_get(x_59, 0);
lean_inc(x_61);
lean_dec_ref(x_59);
x_62 = !lean_is_exclusive(x_61);
if (x_62 == 0)
{
lean_object* x_63; lean_object* x_64; lean_object* x_65; 
x_63 = lean_ctor_get(x_61, 0);
x_64 = lean_ctor_get(x_61, 1);
x_65 = lp_iris_x2dkernel_IrisKernel_FFI_decodeLIATerm(x_64);
if (lean_obj_tag(x_65) == 0)
{
lean_object* x_66; 
lean_free_object(x_61);
lean_dec(x_63);
x_66 = lean_box(0);
return x_66;
}
else
{
uint8_t x_67; 
x_67 = !lean_is_exclusive(x_65);
if (x_67 == 0)
{
lean_object* x_68; uint8_t x_69; 
x_68 = lean_ctor_get(x_65, 0);
x_69 = !lean_is_exclusive(x_68);
if (x_69 == 0)
{
lean_object* x_70; 
x_70 = lean_ctor_get(x_68, 0);
lean_ctor_set_tag(x_61, 2);
lean_ctor_set(x_61, 1, x_70);
lean_ctor_set(x_68, 0, x_61);
return x_65;
}
else
{
lean_object* x_71; lean_object* x_72; lean_object* x_73; 
x_71 = lean_ctor_get(x_68, 0);
x_72 = lean_ctor_get(x_68, 1);
lean_inc(x_72);
lean_inc(x_71);
lean_dec(x_68);
lean_ctor_set_tag(x_61, 2);
lean_ctor_set(x_61, 1, x_71);
x_73 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_73, 0, x_61);
lean_ctor_set(x_73, 1, x_72);
lean_ctor_set(x_65, 0, x_73);
return x_65;
}
}
else
{
lean_object* x_74; lean_object* x_75; lean_object* x_76; lean_object* x_77; lean_object* x_78; lean_object* x_79; 
x_74 = lean_ctor_get(x_65, 0);
lean_inc(x_74);
lean_dec(x_65);
x_75 = lean_ctor_get(x_74, 0);
lean_inc(x_75);
x_76 = lean_ctor_get(x_74, 1);
lean_inc(x_76);
if (lean_is_exclusive(x_74)) {
 lean_ctor_release(x_74, 0);
 lean_ctor_release(x_74, 1);
 x_77 = x_74;
} else {
 lean_dec_ref(x_74);
 x_77 = lean_box(0);
}
lean_ctor_set_tag(x_61, 2);
lean_ctor_set(x_61, 1, x_75);
if (lean_is_scalar(x_77)) {
 x_78 = lean_alloc_ctor(0, 2, 0);
} else {
 x_78 = x_77;
}
lean_ctor_set(x_78, 0, x_61);
lean_ctor_set(x_78, 1, x_76);
x_79 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_79, 0, x_78);
return x_79;
}
}
}
else
{
lean_object* x_80; lean_object* x_81; lean_object* x_82; 
x_80 = lean_ctor_get(x_61, 0);
x_81 = lean_ctor_get(x_61, 1);
lean_inc(x_81);
lean_inc(x_80);
lean_dec(x_61);
x_82 = lp_iris_x2dkernel_IrisKernel_FFI_decodeLIATerm(x_81);
if (lean_obj_tag(x_82) == 0)
{
lean_object* x_83; 
lean_dec(x_80);
x_83 = lean_box(0);
return x_83;
}
else
{
lean_object* x_84; lean_object* x_85; lean_object* x_86; lean_object* x_87; lean_object* x_88; lean_object* x_89; lean_object* x_90; lean_object* x_91; 
x_84 = lean_ctor_get(x_82, 0);
lean_inc(x_84);
if (lean_is_exclusive(x_82)) {
 lean_ctor_release(x_82, 0);
 x_85 = x_82;
} else {
 lean_dec_ref(x_82);
 x_85 = lean_box(0);
}
x_86 = lean_ctor_get(x_84, 0);
lean_inc(x_86);
x_87 = lean_ctor_get(x_84, 1);
lean_inc(x_87);
if (lean_is_exclusive(x_84)) {
 lean_ctor_release(x_84, 0);
 lean_ctor_release(x_84, 1);
 x_88 = x_84;
} else {
 lean_dec_ref(x_84);
 x_88 = lean_box(0);
}
x_89 = lean_alloc_ctor(2, 2, 0);
lean_ctor_set(x_89, 0, x_80);
lean_ctor_set(x_89, 1, x_86);
if (lean_is_scalar(x_88)) {
 x_90 = lean_alloc_ctor(0, 2, 0);
} else {
 x_90 = x_88;
}
lean_ctor_set(x_90, 0, x_89);
lean_ctor_set(x_90, 1, x_87);
if (lean_is_scalar(x_85)) {
 x_91 = lean_alloc_ctor(1, 1, 0);
} else {
 x_91 = x_85;
}
lean_ctor_set(x_91, 0, x_90);
return x_91;
}
}
}
}
}
else
{
lean_object* x_92; 
x_92 = lp_iris_x2dkernel_IrisKernel_FFI_decodeLIATerm(x_6);
if (lean_obj_tag(x_92) == 0)
{
lean_object* x_93; 
x_93 = lean_box(0);
return x_93;
}
else
{
lean_object* x_94; uint8_t x_95; 
x_94 = lean_ctor_get(x_92, 0);
lean_inc(x_94);
lean_dec_ref(x_92);
x_95 = !lean_is_exclusive(x_94);
if (x_95 == 0)
{
lean_object* x_96; lean_object* x_97; lean_object* x_98; 
x_96 = lean_ctor_get(x_94, 0);
x_97 = lean_ctor_get(x_94, 1);
x_98 = lp_iris_x2dkernel_IrisKernel_FFI_decodeLIATerm(x_97);
if (lean_obj_tag(x_98) == 0)
{
lean_object* x_99; 
lean_free_object(x_94);
lean_dec(x_96);
x_99 = lean_box(0);
return x_99;
}
else
{
uint8_t x_100; 
x_100 = !lean_is_exclusive(x_98);
if (x_100 == 0)
{
lean_object* x_101; uint8_t x_102; 
x_101 = lean_ctor_get(x_98, 0);
x_102 = !lean_is_exclusive(x_101);
if (x_102 == 0)
{
lean_object* x_103; 
x_103 = lean_ctor_get(x_101, 0);
lean_ctor_set_tag(x_94, 1);
lean_ctor_set(x_94, 1, x_103);
lean_ctor_set(x_101, 0, x_94);
return x_98;
}
else
{
lean_object* x_104; lean_object* x_105; lean_object* x_106; 
x_104 = lean_ctor_get(x_101, 0);
x_105 = lean_ctor_get(x_101, 1);
lean_inc(x_105);
lean_inc(x_104);
lean_dec(x_101);
lean_ctor_set_tag(x_94, 1);
lean_ctor_set(x_94, 1, x_104);
x_106 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_106, 0, x_94);
lean_ctor_set(x_106, 1, x_105);
lean_ctor_set(x_98, 0, x_106);
return x_98;
}
}
else
{
lean_object* x_107; lean_object* x_108; lean_object* x_109; lean_object* x_110; lean_object* x_111; lean_object* x_112; 
x_107 = lean_ctor_get(x_98, 0);
lean_inc(x_107);
lean_dec(x_98);
x_108 = lean_ctor_get(x_107, 0);
lean_inc(x_108);
x_109 = lean_ctor_get(x_107, 1);
lean_inc(x_109);
if (lean_is_exclusive(x_107)) {
 lean_ctor_release(x_107, 0);
 lean_ctor_release(x_107, 1);
 x_110 = x_107;
} else {
 lean_dec_ref(x_107);
 x_110 = lean_box(0);
}
lean_ctor_set_tag(x_94, 1);
lean_ctor_set(x_94, 1, x_108);
if (lean_is_scalar(x_110)) {
 x_111 = lean_alloc_ctor(0, 2, 0);
} else {
 x_111 = x_110;
}
lean_ctor_set(x_111, 0, x_94);
lean_ctor_set(x_111, 1, x_109);
x_112 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_112, 0, x_111);
return x_112;
}
}
}
else
{
lean_object* x_113; lean_object* x_114; lean_object* x_115; 
x_113 = lean_ctor_get(x_94, 0);
x_114 = lean_ctor_get(x_94, 1);
lean_inc(x_114);
lean_inc(x_113);
lean_dec(x_94);
x_115 = lp_iris_x2dkernel_IrisKernel_FFI_decodeLIATerm(x_114);
if (lean_obj_tag(x_115) == 0)
{
lean_object* x_116; 
lean_dec(x_113);
x_116 = lean_box(0);
return x_116;
}
else
{
lean_object* x_117; lean_object* x_118; lean_object* x_119; lean_object* x_120; lean_object* x_121; lean_object* x_122; lean_object* x_123; lean_object* x_124; 
x_117 = lean_ctor_get(x_115, 0);
lean_inc(x_117);
if (lean_is_exclusive(x_115)) {
 lean_ctor_release(x_115, 0);
 x_118 = x_115;
} else {
 lean_dec_ref(x_115);
 x_118 = lean_box(0);
}
x_119 = lean_ctor_get(x_117, 0);
lean_inc(x_119);
x_120 = lean_ctor_get(x_117, 1);
lean_inc(x_120);
if (lean_is_exclusive(x_117)) {
 lean_ctor_release(x_117, 0);
 lean_ctor_release(x_117, 1);
 x_121 = x_117;
} else {
 lean_dec_ref(x_117);
 x_121 = lean_box(0);
}
x_122 = lean_alloc_ctor(1, 2, 0);
lean_ctor_set(x_122, 0, x_113);
lean_ctor_set(x_122, 1, x_119);
if (lean_is_scalar(x_121)) {
 x_123 = lean_alloc_ctor(0, 2, 0);
} else {
 x_123 = x_121;
}
lean_ctor_set(x_123, 0, x_122);
lean_ctor_set(x_123, 1, x_120);
if (lean_is_scalar(x_118)) {
 x_124 = lean_alloc_ctor(1, 1, 0);
} else {
 x_124 = x_118;
}
lean_ctor_set(x_124, 0, x_123);
return x_124;
}
}
}
}
}
else
{
lean_object* x_125; 
x_125 = lp_iris_x2dkernel_IrisKernel_FFI_decodeLIATerm(x_6);
if (lean_obj_tag(x_125) == 0)
{
lean_object* x_126; 
x_126 = lean_box(0);
return x_126;
}
else
{
lean_object* x_127; uint8_t x_128; 
x_127 = lean_ctor_get(x_125, 0);
lean_inc(x_127);
lean_dec_ref(x_125);
x_128 = !lean_is_exclusive(x_127);
if (x_128 == 0)
{
lean_object* x_129; lean_object* x_130; lean_object* x_131; 
x_129 = lean_ctor_get(x_127, 0);
x_130 = lean_ctor_get(x_127, 1);
x_131 = lp_iris_x2dkernel_IrisKernel_FFI_decodeLIATerm(x_130);
if (lean_obj_tag(x_131) == 0)
{
lean_object* x_132; 
lean_free_object(x_127);
lean_dec(x_129);
x_132 = lean_box(0);
return x_132;
}
else
{
uint8_t x_133; 
x_133 = !lean_is_exclusive(x_131);
if (x_133 == 0)
{
lean_object* x_134; uint8_t x_135; 
x_134 = lean_ctor_get(x_131, 0);
x_135 = !lean_is_exclusive(x_134);
if (x_135 == 0)
{
lean_object* x_136; 
x_136 = lean_ctor_get(x_134, 0);
lean_ctor_set(x_127, 1, x_136);
lean_ctor_set(x_134, 0, x_127);
return x_131;
}
else
{
lean_object* x_137; lean_object* x_138; lean_object* x_139; 
x_137 = lean_ctor_get(x_134, 0);
x_138 = lean_ctor_get(x_134, 1);
lean_inc(x_138);
lean_inc(x_137);
lean_dec(x_134);
lean_ctor_set(x_127, 1, x_137);
x_139 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_139, 0, x_127);
lean_ctor_set(x_139, 1, x_138);
lean_ctor_set(x_131, 0, x_139);
return x_131;
}
}
else
{
lean_object* x_140; lean_object* x_141; lean_object* x_142; lean_object* x_143; lean_object* x_144; lean_object* x_145; 
x_140 = lean_ctor_get(x_131, 0);
lean_inc(x_140);
lean_dec(x_131);
x_141 = lean_ctor_get(x_140, 0);
lean_inc(x_141);
x_142 = lean_ctor_get(x_140, 1);
lean_inc(x_142);
if (lean_is_exclusive(x_140)) {
 lean_ctor_release(x_140, 0);
 lean_ctor_release(x_140, 1);
 x_143 = x_140;
} else {
 lean_dec_ref(x_140);
 x_143 = lean_box(0);
}
lean_ctor_set(x_127, 1, x_141);
if (lean_is_scalar(x_143)) {
 x_144 = lean_alloc_ctor(0, 2, 0);
} else {
 x_144 = x_143;
}
lean_ctor_set(x_144, 0, x_127);
lean_ctor_set(x_144, 1, x_142);
x_145 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_145, 0, x_144);
return x_145;
}
}
}
else
{
lean_object* x_146; lean_object* x_147; lean_object* x_148; 
x_146 = lean_ctor_get(x_127, 0);
x_147 = lean_ctor_get(x_127, 1);
lean_inc(x_147);
lean_inc(x_146);
lean_dec(x_127);
x_148 = lp_iris_x2dkernel_IrisKernel_FFI_decodeLIATerm(x_147);
if (lean_obj_tag(x_148) == 0)
{
lean_object* x_149; 
lean_dec(x_146);
x_149 = lean_box(0);
return x_149;
}
else
{
lean_object* x_150; lean_object* x_151; lean_object* x_152; lean_object* x_153; lean_object* x_154; lean_object* x_155; lean_object* x_156; lean_object* x_157; 
x_150 = lean_ctor_get(x_148, 0);
lean_inc(x_150);
if (lean_is_exclusive(x_148)) {
 lean_ctor_release(x_148, 0);
 x_151 = x_148;
} else {
 lean_dec_ref(x_148);
 x_151 = lean_box(0);
}
x_152 = lean_ctor_get(x_150, 0);
lean_inc(x_152);
x_153 = lean_ctor_get(x_150, 1);
lean_inc(x_153);
if (lean_is_exclusive(x_150)) {
 lean_ctor_release(x_150, 0);
 lean_ctor_release(x_150, 1);
 x_154 = x_150;
} else {
 lean_dec_ref(x_150);
 x_154 = lean_box(0);
}
x_155 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_155, 0, x_146);
lean_ctor_set(x_155, 1, x_152);
if (lean_is_scalar(x_154)) {
 x_156 = lean_alloc_ctor(0, 2, 0);
} else {
 x_156 = x_154;
}
lean_ctor_set(x_156, 0, x_155);
lean_ctor_set(x_156, 1, x_153);
if (lean_is_scalar(x_151)) {
 x_157 = lean_alloc_ctor(1, 1, 0);
} else {
 x_157 = x_151;
}
lean_ctor_set(x_157, 0, x_156);
return x_157;
}
}
}
}
}
}
}
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_decodeLIAEnv_decodeLIAEnvEntries(lean_object* x_1, lean_object* x_2, lean_object* x_3) {
_start:
{
lean_object* x_4; uint8_t x_5; 
x_4 = lean_unsigned_to_nat(0u);
x_5 = lean_nat_dec_eq(x_2, x_4);
if (x_5 == 1)
{
lean_object* x_6; lean_object* x_7; lean_object* x_8; 
lean_dec(x_2);
x_6 = l_List_reverse___redArg(x_3);
x_7 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_7, 0, x_6);
lean_ctor_set(x_7, 1, x_1);
x_8 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_8, 0, x_7);
return x_8;
}
else
{
lean_object* x_9; 
x_9 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readUInt32LE(x_1);
if (lean_obj_tag(x_9) == 0)
{
lean_object* x_10; 
lean_dec(x_3);
lean_dec(x_2);
x_10 = lean_box(0);
return x_10;
}
else
{
lean_object* x_11; uint8_t x_12; 
x_11 = lean_ctor_get(x_9, 0);
lean_inc(x_11);
lean_dec_ref(x_9);
x_12 = !lean_is_exclusive(x_11);
if (x_12 == 0)
{
lean_object* x_13; lean_object* x_14; lean_object* x_15; 
x_13 = lean_ctor_get(x_11, 0);
x_14 = lean_ctor_get(x_11, 1);
x_15 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readInt64LE(x_14);
if (lean_obj_tag(x_15) == 0)
{
lean_object* x_16; 
lean_free_object(x_11);
lean_dec(x_13);
lean_dec(x_3);
lean_dec(x_2);
x_16 = lean_box(0);
return x_16;
}
else
{
lean_object* x_17; uint8_t x_18; 
x_17 = lean_ctor_get(x_15, 0);
lean_inc(x_17);
lean_dec_ref(x_15);
x_18 = !lean_is_exclusive(x_17);
if (x_18 == 0)
{
lean_object* x_19; lean_object* x_20; lean_object* x_21; lean_object* x_22; uint32_t x_23; lean_object* x_24; 
x_19 = lean_ctor_get(x_17, 0);
x_20 = lean_ctor_get(x_17, 1);
x_21 = lean_unsigned_to_nat(1u);
x_22 = lean_nat_sub(x_2, x_21);
lean_dec(x_2);
x_23 = lean_unbox_uint32(x_13);
lean_dec(x_13);
x_24 = lean_uint32_to_nat(x_23);
lean_ctor_set(x_17, 1, x_19);
lean_ctor_set(x_17, 0, x_24);
lean_ctor_set_tag(x_11, 1);
lean_ctor_set(x_11, 1, x_3);
lean_ctor_set(x_11, 0, x_17);
x_1 = x_20;
x_2 = x_22;
x_3 = x_11;
goto _start;
}
else
{
lean_object* x_26; lean_object* x_27; lean_object* x_28; lean_object* x_29; uint32_t x_30; lean_object* x_31; lean_object* x_32; 
x_26 = lean_ctor_get(x_17, 0);
x_27 = lean_ctor_get(x_17, 1);
lean_inc(x_27);
lean_inc(x_26);
lean_dec(x_17);
x_28 = lean_unsigned_to_nat(1u);
x_29 = lean_nat_sub(x_2, x_28);
lean_dec(x_2);
x_30 = lean_unbox_uint32(x_13);
lean_dec(x_13);
x_31 = lean_uint32_to_nat(x_30);
x_32 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_32, 0, x_31);
lean_ctor_set(x_32, 1, x_26);
lean_ctor_set_tag(x_11, 1);
lean_ctor_set(x_11, 1, x_3);
lean_ctor_set(x_11, 0, x_32);
x_1 = x_27;
x_2 = x_29;
x_3 = x_11;
goto _start;
}
}
}
else
{
lean_object* x_34; lean_object* x_35; lean_object* x_36; 
x_34 = lean_ctor_get(x_11, 0);
x_35 = lean_ctor_get(x_11, 1);
lean_inc(x_35);
lean_inc(x_34);
lean_dec(x_11);
x_36 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readInt64LE(x_35);
if (lean_obj_tag(x_36) == 0)
{
lean_object* x_37; 
lean_dec(x_34);
lean_dec(x_3);
lean_dec(x_2);
x_37 = lean_box(0);
return x_37;
}
else
{
lean_object* x_38; lean_object* x_39; lean_object* x_40; lean_object* x_41; lean_object* x_42; lean_object* x_43; uint32_t x_44; lean_object* x_45; lean_object* x_46; lean_object* x_47; 
x_38 = lean_ctor_get(x_36, 0);
lean_inc(x_38);
lean_dec_ref(x_36);
x_39 = lean_ctor_get(x_38, 0);
lean_inc(x_39);
x_40 = lean_ctor_get(x_38, 1);
lean_inc(x_40);
if (lean_is_exclusive(x_38)) {
 lean_ctor_release(x_38, 0);
 lean_ctor_release(x_38, 1);
 x_41 = x_38;
} else {
 lean_dec_ref(x_38);
 x_41 = lean_box(0);
}
x_42 = lean_unsigned_to_nat(1u);
x_43 = lean_nat_sub(x_2, x_42);
lean_dec(x_2);
x_44 = lean_unbox_uint32(x_34);
lean_dec(x_34);
x_45 = lean_uint32_to_nat(x_44);
if (lean_is_scalar(x_41)) {
 x_46 = lean_alloc_ctor(0, 2, 0);
} else {
 x_46 = x_41;
}
lean_ctor_set(x_46, 0, x_45);
lean_ctor_set(x_46, 1, x_39);
x_47 = lean_alloc_ctor(1, 2, 0);
lean_ctor_set(x_47, 0, x_46);
lean_ctor_set(x_47, 1, x_3);
x_1 = x_40;
x_2 = x_43;
x_3 = x_47;
goto _start;
}
}
}
}
}
}
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_decodeLIAEnv(lean_object* x_1) {
_start:
{
lean_object* x_2; 
x_2 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readUInt16LE(x_1);
if (lean_obj_tag(x_2) == 0)
{
lean_object* x_3; 
x_3 = lean_box(0);
return x_3;
}
else
{
lean_object* x_4; lean_object* x_5; lean_object* x_6; uint16_t x_7; lean_object* x_8; lean_object* x_9; lean_object* x_10; 
x_4 = lean_ctor_get(x_2, 0);
lean_inc(x_4);
lean_dec_ref(x_2);
x_5 = lean_ctor_get(x_4, 0);
lean_inc(x_5);
x_6 = lean_ctor_get(x_4, 1);
lean_inc(x_6);
lean_dec(x_4);
x_7 = lean_unbox(x_5);
lean_dec(x_5);
x_8 = lean_uint16_to_nat(x_7);
x_9 = lean_box(0);
x_10 = lp_iris_x2dkernel_IrisKernel_FFI_decodeLIAEnv_decodeLIAEnvEntries(x_6, x_8, x_9);
return x_10;
}
}
}
LEAN_EXPORT uint8_t iris_check_cost_leq(lean_object* x_1) {
_start:
{
lean_object* x_2; lean_object* x_3; 
x_2 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_create(x_1);
x_3 = lp_iris_x2dkernel_IrisKernel_FFI_decodeCostBound(x_2);
if (lean_obj_tag(x_3) == 0)
{
uint8_t x_4; 
x_4 = 0;
return x_4;
}
else
{
lean_object* x_5; lean_object* x_6; lean_object* x_7; lean_object* x_8; 
x_5 = lean_ctor_get(x_3, 0);
lean_inc(x_5);
lean_dec_ref(x_3);
x_6 = lean_ctor_get(x_5, 0);
lean_inc(x_6);
x_7 = lean_ctor_get(x_5, 1);
lean_inc(x_7);
lean_dec(x_5);
x_8 = lp_iris_x2dkernel_IrisKernel_FFI_decodeCostBound(x_7);
if (lean_obj_tag(x_8) == 0)
{
uint8_t x_9; 
lean_dec(x_6);
x_9 = 0;
return x_9;
}
else
{
lean_object* x_10; lean_object* x_11; uint8_t x_12; 
x_10 = lean_ctor_get(x_8, 0);
lean_inc(x_10);
lean_dec_ref(x_8);
x_11 = lean_ctor_get(x_10, 0);
lean_inc(x_11);
lean_dec(x_10);
x_12 = lp_iris_x2dkernel_IrisKernel_checkCostLeq(x_6, x_11);
if (x_12 == 0)
{
uint8_t x_13; 
x_13 = 0;
return x_13;
}
else
{
uint8_t x_14; 
x_14 = 1;
return x_14;
}
}
}
}
}
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_checkCostLeqFFI___boxed(lean_object* x_1) {
_start:
{
uint8_t x_2; lean_object* x_3; 
x_2 = iris_check_cost_leq(x_1);
x_3 = lean_box(x_2);
return x_3;
}
}
LEAN_EXPORT uint8_t iris_eval_lia(lean_object* x_1) {
_start:
{
lean_object* x_2; lean_object* x_3; 
x_2 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_create(x_1);
x_3 = lp_iris_x2dkernel_IrisKernel_FFI_decodeLIAFormula(x_2);
if (lean_obj_tag(x_3) == 0)
{
uint8_t x_4; 
x_4 = 0;
return x_4;
}
else
{
lean_object* x_5; lean_object* x_6; lean_object* x_7; lean_object* x_8; 
x_5 = lean_ctor_get(x_3, 0);
lean_inc(x_5);
lean_dec_ref(x_3);
x_6 = lean_ctor_get(x_5, 0);
lean_inc(x_6);
x_7 = lean_ctor_get(x_5, 1);
lean_inc(x_7);
lean_dec(x_5);
x_8 = lp_iris_x2dkernel_IrisKernel_FFI_decodeLIAEnv(x_7);
if (lean_obj_tag(x_8) == 0)
{
uint8_t x_9; 
lean_dec(x_6);
x_9 = 0;
return x_9;
}
else
{
lean_object* x_10; lean_object* x_11; uint8_t x_12; 
x_10 = lean_ctor_get(x_8, 0);
lean_inc(x_10);
lean_dec_ref(x_8);
x_11 = lean_ctor_get(x_10, 0);
lean_inc(x_11);
lean_dec(x_10);
x_12 = lp_iris_x2dkernel_IrisKernel_evalLIAFormula(x_6, x_11);
lean_dec(x_11);
if (x_12 == 0)
{
uint8_t x_13; 
x_13 = 0;
return x_13;
}
else
{
uint8_t x_14; 
x_14 = 1;
return x_14;
}
}
}
}
}
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_evalLIAFFI___boxed(lean_object* x_1) {
_start:
{
uint8_t x_2; lean_object* x_3; 
x_2 = iris_eval_lia(x_1);
x_3 = lean_box(x_2);
return x_3;
}
}
LEAN_EXPORT uint8_t lp_iris_x2dkernel_IrisKernel_FFI_typeCheckNodeFFI___redArg(uint8_t x_1) {
_start:
{
lean_object* x_2; lean_object* x_3; uint8_t x_4; 
x_2 = lean_uint8_to_nat(x_1);
x_3 = lean_unsigned_to_nat(20u);
x_4 = lean_nat_dec_lt(x_2, x_3);
if (x_4 == 0)
{
uint8_t x_5; 
x_5 = 0;
return x_5;
}
else
{
uint8_t x_6; 
x_6 = 1;
return x_6;
}
}
}
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_typeCheckNodeFFI___redArg___boxed(lean_object* x_1) {
_start:
{
uint8_t x_2; uint8_t x_3; lean_object* x_4; 
x_2 = lean_unbox(x_1);
x_3 = lp_iris_x2dkernel_IrisKernel_FFI_typeCheckNodeFFI___redArg(x_2);
x_4 = lean_box(x_3);
return x_4;
}
}
LEAN_EXPORT uint8_t iris_type_check_node(uint8_t x_1, uint64_t x_2, uint64_t x_3) {
_start:
{
uint8_t x_4; 
x_4 = lp_iris_x2dkernel_IrisKernel_FFI_typeCheckNodeFFI___redArg(x_1);
return x_4;
}
}
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_typeCheckNodeFFI___boxed(lean_object* x_1, lean_object* x_2, lean_object* x_3) {
_start:
{
uint8_t x_4; uint64_t x_5; uint64_t x_6; uint8_t x_7; lean_object* x_8; 
x_4 = lean_unbox(x_1);
x_5 = lean_unbox_uint64(x_2);
lean_dec(x_2);
x_6 = lean_unbox_uint64(x_3);
lean_dec(x_3);
x_7 = iris_type_check_node(x_4, x_5, x_6);
x_8 = lean_box(x_7);
return x_8;
}
}
static uint32_t _init_iris_lean_kernel_version() {
_start:
{
uint32_t x_1; 
x_1 = 1230129491;
return x_1;
}
}
static lean_object* _init_lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__0() {
_start:
{
uint8_t x_1; lean_object* x_2; lean_object* x_3; 
x_1 = 19;
x_2 = lean_box(x_1);
x_3 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_3, 0, x_2);
return x_3;
}
}
static lean_object* _init_lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__1() {
_start:
{
uint8_t x_1; lean_object* x_2; lean_object* x_3; 
x_1 = 18;
x_2 = lean_box(x_1);
x_3 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_3, 0, x_2);
return x_3;
}
}
static lean_object* _init_lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__2() {
_start:
{
uint8_t x_1; lean_object* x_2; lean_object* x_3; 
x_1 = 17;
x_2 = lean_box(x_1);
x_3 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_3, 0, x_2);
return x_3;
}
}
static lean_object* _init_lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__3() {
_start:
{
uint8_t x_1; lean_object* x_2; lean_object* x_3; 
x_1 = 16;
x_2 = lean_box(x_1);
x_3 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_3, 0, x_2);
return x_3;
}
}
static lean_object* _init_lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__4() {
_start:
{
uint8_t x_1; lean_object* x_2; lean_object* x_3; 
x_1 = 15;
x_2 = lean_box(x_1);
x_3 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_3, 0, x_2);
return x_3;
}
}
static lean_object* _init_lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__5() {
_start:
{
uint8_t x_1; lean_object* x_2; lean_object* x_3; 
x_1 = 14;
x_2 = lean_box(x_1);
x_3 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_3, 0, x_2);
return x_3;
}
}
static lean_object* _init_lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__6() {
_start:
{
uint8_t x_1; lean_object* x_2; lean_object* x_3; 
x_1 = 13;
x_2 = lean_box(x_1);
x_3 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_3, 0, x_2);
return x_3;
}
}
static lean_object* _init_lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__7() {
_start:
{
uint8_t x_1; lean_object* x_2; lean_object* x_3; 
x_1 = 12;
x_2 = lean_box(x_1);
x_3 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_3, 0, x_2);
return x_3;
}
}
static lean_object* _init_lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__8() {
_start:
{
uint8_t x_1; lean_object* x_2; lean_object* x_3; 
x_1 = 11;
x_2 = lean_box(x_1);
x_3 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_3, 0, x_2);
return x_3;
}
}
static lean_object* _init_lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__9() {
_start:
{
uint8_t x_1; lean_object* x_2; lean_object* x_3; 
x_1 = 10;
x_2 = lean_box(x_1);
x_3 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_3, 0, x_2);
return x_3;
}
}
static lean_object* _init_lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__10() {
_start:
{
uint8_t x_1; lean_object* x_2; lean_object* x_3; 
x_1 = 9;
x_2 = lean_box(x_1);
x_3 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_3, 0, x_2);
return x_3;
}
}
static lean_object* _init_lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__11() {
_start:
{
uint8_t x_1; lean_object* x_2; lean_object* x_3; 
x_1 = 8;
x_2 = lean_box(x_1);
x_3 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_3, 0, x_2);
return x_3;
}
}
static lean_object* _init_lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__12() {
_start:
{
uint8_t x_1; lean_object* x_2; lean_object* x_3; 
x_1 = 7;
x_2 = lean_box(x_1);
x_3 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_3, 0, x_2);
return x_3;
}
}
static lean_object* _init_lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__13() {
_start:
{
uint8_t x_1; lean_object* x_2; lean_object* x_3; 
x_1 = 6;
x_2 = lean_box(x_1);
x_3 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_3, 0, x_2);
return x_3;
}
}
static lean_object* _init_lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__14() {
_start:
{
uint8_t x_1; lean_object* x_2; lean_object* x_3; 
x_1 = 5;
x_2 = lean_box(x_1);
x_3 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_3, 0, x_2);
return x_3;
}
}
static lean_object* _init_lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__15() {
_start:
{
uint8_t x_1; lean_object* x_2; lean_object* x_3; 
x_1 = 4;
x_2 = lean_box(x_1);
x_3 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_3, 0, x_2);
return x_3;
}
}
static lean_object* _init_lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__16() {
_start:
{
uint8_t x_1; lean_object* x_2; lean_object* x_3; 
x_1 = 3;
x_2 = lean_box(x_1);
x_3 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_3, 0, x_2);
return x_3;
}
}
static lean_object* _init_lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__17() {
_start:
{
uint8_t x_1; lean_object* x_2; lean_object* x_3; 
x_1 = 2;
x_2 = lean_box(x_1);
x_3 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_3, 0, x_2);
return x_3;
}
}
static lean_object* _init_lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__18() {
_start:
{
uint8_t x_1; lean_object* x_2; lean_object* x_3; 
x_1 = 1;
x_2 = lean_box(x_1);
x_3 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_3, 0, x_2);
return x_3;
}
}
static lean_object* _init_lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__19() {
_start:
{
uint8_t x_1; lean_object* x_2; lean_object* x_3; 
x_1 = 0;
x_2 = lean_box(x_1);
x_3 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_3, 0, x_2);
return x_3;
}
}
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind(uint8_t x_1) {
_start:
{
lean_object* x_2; lean_object* x_3; uint8_t x_4; 
x_2 = lean_uint8_to_nat(x_1);
x_3 = lean_unsigned_to_nat(0u);
x_4 = lean_nat_dec_eq(x_2, x_3);
if (x_4 == 0)
{
lean_object* x_5; uint8_t x_6; 
x_5 = lean_unsigned_to_nat(1u);
x_6 = lean_nat_dec_eq(x_2, x_5);
if (x_6 == 0)
{
lean_object* x_7; uint8_t x_8; 
x_7 = lean_unsigned_to_nat(2u);
x_8 = lean_nat_dec_eq(x_2, x_7);
if (x_8 == 0)
{
lean_object* x_9; uint8_t x_10; 
x_9 = lean_unsigned_to_nat(3u);
x_10 = lean_nat_dec_eq(x_2, x_9);
if (x_10 == 0)
{
lean_object* x_11; uint8_t x_12; 
x_11 = lean_unsigned_to_nat(4u);
x_12 = lean_nat_dec_eq(x_2, x_11);
if (x_12 == 0)
{
lean_object* x_13; uint8_t x_14; 
x_13 = lean_unsigned_to_nat(5u);
x_14 = lean_nat_dec_eq(x_2, x_13);
if (x_14 == 0)
{
lean_object* x_15; uint8_t x_16; 
x_15 = lean_unsigned_to_nat(6u);
x_16 = lean_nat_dec_eq(x_2, x_15);
if (x_16 == 0)
{
lean_object* x_17; uint8_t x_18; 
x_17 = lean_unsigned_to_nat(7u);
x_18 = lean_nat_dec_eq(x_2, x_17);
if (x_18 == 0)
{
lean_object* x_19; uint8_t x_20; 
x_19 = lean_unsigned_to_nat(8u);
x_20 = lean_nat_dec_eq(x_2, x_19);
if (x_20 == 0)
{
lean_object* x_21; uint8_t x_22; 
x_21 = lean_unsigned_to_nat(9u);
x_22 = lean_nat_dec_eq(x_2, x_21);
if (x_22 == 0)
{
lean_object* x_23; uint8_t x_24; 
x_23 = lean_unsigned_to_nat(10u);
x_24 = lean_nat_dec_eq(x_2, x_23);
if (x_24 == 0)
{
lean_object* x_25; uint8_t x_26; 
x_25 = lean_unsigned_to_nat(11u);
x_26 = lean_nat_dec_eq(x_2, x_25);
if (x_26 == 0)
{
lean_object* x_27; uint8_t x_28; 
x_27 = lean_unsigned_to_nat(12u);
x_28 = lean_nat_dec_eq(x_2, x_27);
if (x_28 == 0)
{
lean_object* x_29; uint8_t x_30; 
x_29 = lean_unsigned_to_nat(13u);
x_30 = lean_nat_dec_eq(x_2, x_29);
if (x_30 == 0)
{
lean_object* x_31; uint8_t x_32; 
x_31 = lean_unsigned_to_nat(14u);
x_32 = lean_nat_dec_eq(x_2, x_31);
if (x_32 == 0)
{
lean_object* x_33; uint8_t x_34; 
x_33 = lean_unsigned_to_nat(15u);
x_34 = lean_nat_dec_eq(x_2, x_33);
if (x_34 == 0)
{
lean_object* x_35; uint8_t x_36; 
x_35 = lean_unsigned_to_nat(16u);
x_36 = lean_nat_dec_eq(x_2, x_35);
if (x_36 == 0)
{
lean_object* x_37; uint8_t x_38; 
x_37 = lean_unsigned_to_nat(17u);
x_38 = lean_nat_dec_eq(x_2, x_37);
if (x_38 == 0)
{
lean_object* x_39; uint8_t x_40; 
x_39 = lean_unsigned_to_nat(18u);
x_40 = lean_nat_dec_eq(x_2, x_39);
if (x_40 == 0)
{
lean_object* x_41; uint8_t x_42; 
x_41 = lean_unsigned_to_nat(19u);
x_42 = lean_nat_dec_eq(x_2, x_41);
if (x_42 == 0)
{
lean_object* x_43; 
x_43 = lean_box(0);
return x_43;
}
else
{
lean_object* x_44; 
x_44 = lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__0;
return x_44;
}
}
else
{
lean_object* x_45; 
x_45 = lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__1;
return x_45;
}
}
else
{
lean_object* x_46; 
x_46 = lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__2;
return x_46;
}
}
else
{
lean_object* x_47; 
x_47 = lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__3;
return x_47;
}
}
else
{
lean_object* x_48; 
x_48 = lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__4;
return x_48;
}
}
else
{
lean_object* x_49; 
x_49 = lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__5;
return x_49;
}
}
else
{
lean_object* x_50; 
x_50 = lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__6;
return x_50;
}
}
else
{
lean_object* x_51; 
x_51 = lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__7;
return x_51;
}
}
else
{
lean_object* x_52; 
x_52 = lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__8;
return x_52;
}
}
else
{
lean_object* x_53; 
x_53 = lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__9;
return x_53;
}
}
else
{
lean_object* x_54; 
x_54 = lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__10;
return x_54;
}
}
else
{
lean_object* x_55; 
x_55 = lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__11;
return x_55;
}
}
else
{
lean_object* x_56; 
x_56 = lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__12;
return x_56;
}
}
else
{
lean_object* x_57; 
x_57 = lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__13;
return x_57;
}
}
else
{
lean_object* x_58; 
x_58 = lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__14;
return x_58;
}
}
else
{
lean_object* x_59; 
x_59 = lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__15;
return x_59;
}
}
else
{
lean_object* x_60; 
x_60 = lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__16;
return x_60;
}
}
else
{
lean_object* x_61; 
x_61 = lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__17;
return x_61;
}
}
else
{
lean_object* x_62; 
x_62 = lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__18;
return x_62;
}
}
else
{
lean_object* x_63; 
x_63 = lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__19;
return x_63;
}
}
}
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___boxed(lean_object* x_1) {
_start:
{
uint8_t x_2; lean_object* x_3; 
x_2 = lean_unbox(x_1);
x_3 = lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind(x_2);
return x_3;
}
}
static lean_object* _init_lp_iris_x2dkernel_IrisKernel_FFI_wrapResult___closed__0() {
_start:
{
uint8_t x_1; lean_object* x_2; 
x_1 = 1;
x_2 = lp_iris_x2dkernel_IrisKernel_FFI_encodeFailure(x_1);
return x_2;
}
}
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_wrapResult(lean_object* x_1) {
_start:
{
if (lean_obj_tag(x_1) == 0)
{
lean_object* x_2; 
x_2 = lp_iris_x2dkernel_IrisKernel_FFI_wrapResult___closed__0;
return x_2;
}
else
{
lean_object* x_3; lean_object* x_4; 
x_3 = lean_ctor_get(x_1, 0);
x_4 = lp_iris_x2dkernel_IrisKernel_FFI_encodeSuccess(x_3);
return x_4;
}
}
}
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_wrapResult___boxed(lean_object* x_1) {
_start:
{
lean_object* x_2; 
x_2 = lp_iris_x2dkernel_IrisKernel_FFI_wrapResult(x_1);
lean_dec(x_1);
return x_2;
}
}
static lean_object* _init_lp_iris_x2dkernel_IrisKernel_FFI_assumeFFI___closed__0() {
_start:
{
uint8_t x_1; lean_object* x_2; 
x_1 = 2;
x_2 = lp_iris_x2dkernel_IrisKernel_FFI_encodeFailure(x_1);
return x_2;
}
}
LEAN_EXPORT lean_object* iris_kernel_assume(lean_object* x_1) {
_start:
{
lean_object* x_4; lean_object* x_5; 
x_4 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_create(x_1);
x_5 = lp_iris_x2dkernel_IrisKernel_FFI_decodeContext(x_4);
if (lean_obj_tag(x_5) == 0)
{
goto block_3;
}
else
{
lean_object* x_6; lean_object* x_7; lean_object* x_8; lean_object* x_9; 
x_6 = lean_ctor_get(x_5, 0);
lean_inc(x_6);
lean_dec_ref(x_5);
x_7 = lean_ctor_get(x_6, 0);
lean_inc(x_7);
x_8 = lean_ctor_get(x_6, 1);
lean_inc(x_8);
lean_dec(x_6);
x_9 = lp_iris_x2dkernel_IrisKernel_FFI_decodeBinderId(x_8);
if (lean_obj_tag(x_9) == 0)
{
lean_dec(x_7);
goto block_3;
}
else
{
lean_object* x_10; lean_object* x_11; lean_object* x_12; lean_object* x_13; 
x_10 = lean_ctor_get(x_9, 0);
lean_inc(x_10);
lean_dec_ref(x_9);
x_11 = lean_ctor_get(x_10, 0);
lean_inc(x_11);
x_12 = lean_ctor_get(x_10, 1);
lean_inc(x_12);
lean_dec(x_10);
x_13 = lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeId(x_12);
if (lean_obj_tag(x_13) == 0)
{
lean_dec(x_11);
lean_dec(x_7);
goto block_3;
}
else
{
lean_object* x_14; lean_object* x_15; lean_object* x_16; lean_object* x_17; 
x_14 = lean_ctor_get(x_13, 0);
lean_inc(x_14);
lean_dec_ref(x_13);
x_15 = lean_ctor_get(x_14, 0);
lean_inc(x_15);
lean_dec(x_14);
x_16 = lp_iris_x2dkernel_IrisKernel_Kernel_assume__(x_7, x_11, x_15);
lean_dec(x_11);
x_17 = lp_iris_x2dkernel_IrisKernel_FFI_wrapResult(x_16);
lean_dec(x_16);
return x_17;
}
}
}
block_3:
{
lean_object* x_2; 
x_2 = lp_iris_x2dkernel_IrisKernel_FFI_assumeFFI___closed__0;
return x_2;
}
}
}
LEAN_EXPORT lean_object* iris_kernel_intro(lean_object* x_1) {
_start:
{
lean_object* x_4; lean_object* x_5; 
x_4 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_create(x_1);
x_5 = lp_iris_x2dkernel_IrisKernel_FFI_decodeTypeEnv(x_4);
if (lean_obj_tag(x_5) == 0)
{
goto block_3;
}
else
{
lean_object* x_6; lean_object* x_7; lean_object* x_8; lean_object* x_9; 
x_6 = lean_ctor_get(x_5, 0);
lean_inc(x_6);
lean_dec_ref(x_5);
x_7 = lean_ctor_get(x_6, 0);
lean_inc(x_7);
x_8 = lean_ctor_get(x_6, 1);
lean_inc(x_8);
lean_dec(x_6);
x_9 = lp_iris_x2dkernel_IrisKernel_FFI_decodeContext(x_8);
if (lean_obj_tag(x_9) == 0)
{
lean_dec(x_7);
goto block_3;
}
else
{
lean_object* x_10; lean_object* x_11; lean_object* x_12; lean_object* x_13; 
x_10 = lean_ctor_get(x_9, 0);
lean_inc(x_10);
lean_dec_ref(x_9);
x_11 = lean_ctor_get(x_10, 0);
lean_inc(x_11);
x_12 = lean_ctor_get(x_10, 1);
lean_inc(x_12);
lean_dec(x_10);
x_13 = lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeId(x_12);
if (lean_obj_tag(x_13) == 0)
{
lean_dec(x_11);
lean_dec(x_7);
goto block_3;
}
else
{
lean_object* x_14; lean_object* x_15; lean_object* x_16; lean_object* x_17; 
x_14 = lean_ctor_get(x_13, 0);
lean_inc(x_14);
lean_dec_ref(x_13);
x_15 = lean_ctor_get(x_14, 0);
lean_inc(x_15);
x_16 = lean_ctor_get(x_14, 1);
lean_inc(x_16);
lean_dec(x_14);
x_17 = lp_iris_x2dkernel_IrisKernel_FFI_decodeBinderId(x_16);
if (lean_obj_tag(x_17) == 0)
{
lean_dec(x_15);
lean_dec(x_11);
lean_dec(x_7);
goto block_3;
}
else
{
lean_object* x_18; lean_object* x_19; lean_object* x_20; lean_object* x_21; 
x_18 = lean_ctor_get(x_17, 0);
lean_inc(x_18);
lean_dec_ref(x_17);
x_19 = lean_ctor_get(x_18, 0);
lean_inc(x_19);
x_20 = lean_ctor_get(x_18, 1);
lean_inc(x_20);
lean_dec(x_18);
x_21 = lp_iris_x2dkernel_IrisKernel_FFI_decodeTypeId(x_20);
if (lean_obj_tag(x_21) == 0)
{
lean_dec(x_19);
lean_dec(x_15);
lean_dec(x_11);
lean_dec(x_7);
goto block_3;
}
else
{
lean_object* x_22; lean_object* x_23; lean_object* x_24; lean_object* x_25; 
x_22 = lean_ctor_get(x_21, 0);
lean_inc(x_22);
lean_dec_ref(x_21);
x_23 = lean_ctor_get(x_22, 0);
lean_inc(x_23);
x_24 = lean_ctor_get(x_22, 1);
lean_inc(x_24);
lean_dec(x_22);
x_25 = lp_iris_x2dkernel_IrisKernel_FFI_decodeJudgment(x_24);
if (lean_obj_tag(x_25) == 0)
{
lean_dec(x_23);
lean_dec(x_19);
lean_dec(x_15);
lean_dec(x_11);
lean_dec(x_7);
goto block_3;
}
else
{
lean_object* x_26; lean_object* x_27; lean_object* x_28; lean_object* x_29; 
x_26 = lean_ctor_get(x_25, 0);
lean_inc(x_26);
lean_dec_ref(x_25);
x_27 = lean_ctor_get(x_26, 0);
lean_inc(x_27);
x_28 = lean_ctor_get(x_26, 1);
lean_inc(x_28);
lean_dec(x_26);
x_29 = lp_iris_x2dkernel_IrisKernel_FFI_decodeTypeId(x_28);
if (lean_obj_tag(x_29) == 0)
{
lean_dec(x_27);
lean_dec(x_23);
lean_dec(x_19);
lean_dec(x_15);
lean_dec(x_11);
lean_dec(x_7);
goto block_3;
}
else
{
lean_object* x_30; lean_object* x_31; lean_object* x_32; lean_object* x_33; 
x_30 = lean_ctor_get(x_29, 0);
lean_inc(x_30);
lean_dec_ref(x_29);
x_31 = lean_ctor_get(x_30, 0);
lean_inc(x_31);
lean_dec(x_30);
x_32 = lp_iris_x2dkernel_IrisKernel_Kernel_intro(x_7, x_11, x_15, x_19, x_23, x_27, x_31);
lean_dec(x_7);
x_33 = lp_iris_x2dkernel_IrisKernel_FFI_wrapResult(x_32);
lean_dec(x_32);
return x_33;
}
}
}
}
}
}
}
block_3:
{
lean_object* x_2; 
x_2 = lp_iris_x2dkernel_IrisKernel_FFI_assumeFFI___closed__0;
return x_2;
}
}
}
LEAN_EXPORT lean_object* iris_kernel_elim(lean_object* x_1) {
_start:
{
lean_object* x_4; lean_object* x_5; 
x_4 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_create(x_1);
x_5 = lp_iris_x2dkernel_IrisKernel_FFI_decodeTypeEnv(x_4);
if (lean_obj_tag(x_5) == 0)
{
goto block_3;
}
else
{
lean_object* x_6; lean_object* x_7; lean_object* x_8; lean_object* x_9; 
x_6 = lean_ctor_get(x_5, 0);
lean_inc(x_6);
lean_dec_ref(x_5);
x_7 = lean_ctor_get(x_6, 0);
lean_inc(x_7);
x_8 = lean_ctor_get(x_6, 1);
lean_inc(x_8);
lean_dec(x_6);
x_9 = lp_iris_x2dkernel_IrisKernel_FFI_decodeJudgment(x_8);
if (lean_obj_tag(x_9) == 0)
{
lean_dec(x_7);
goto block_3;
}
else
{
lean_object* x_10; lean_object* x_11; lean_object* x_12; lean_object* x_13; 
x_10 = lean_ctor_get(x_9, 0);
lean_inc(x_10);
lean_dec_ref(x_9);
x_11 = lean_ctor_get(x_10, 0);
lean_inc(x_11);
x_12 = lean_ctor_get(x_10, 1);
lean_inc(x_12);
lean_dec(x_10);
x_13 = lp_iris_x2dkernel_IrisKernel_FFI_decodeJudgment(x_12);
if (lean_obj_tag(x_13) == 0)
{
lean_dec(x_11);
lean_dec(x_7);
goto block_3;
}
else
{
lean_object* x_14; lean_object* x_15; lean_object* x_16; lean_object* x_17; 
x_14 = lean_ctor_get(x_13, 0);
lean_inc(x_14);
lean_dec_ref(x_13);
x_15 = lean_ctor_get(x_14, 0);
lean_inc(x_15);
x_16 = lean_ctor_get(x_14, 1);
lean_inc(x_16);
lean_dec(x_14);
x_17 = lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeId(x_16);
if (lean_obj_tag(x_17) == 0)
{
lean_dec(x_15);
lean_dec(x_11);
lean_dec(x_7);
goto block_3;
}
else
{
lean_object* x_18; lean_object* x_19; lean_object* x_20; lean_object* x_21; 
x_18 = lean_ctor_get(x_17, 0);
lean_inc(x_18);
lean_dec_ref(x_17);
x_19 = lean_ctor_get(x_18, 0);
lean_inc(x_19);
lean_dec(x_18);
x_20 = lp_iris_x2dkernel_IrisKernel_Kernel_elim(x_7, x_11, x_15, x_19);
lean_dec(x_11);
lean_dec(x_7);
x_21 = lp_iris_x2dkernel_IrisKernel_FFI_wrapResult(x_20);
lean_dec(x_20);
return x_21;
}
}
}
}
block_3:
{
lean_object* x_2; 
x_2 = lp_iris_x2dkernel_IrisKernel_FFI_assumeFFI___closed__0;
return x_2;
}
}
}
LEAN_EXPORT lean_object* iris_kernel_refl(lean_object* x_1) {
_start:
{
lean_object* x_4; lean_object* x_5; 
x_4 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_create(x_1);
x_5 = lp_iris_x2dkernel_IrisKernel_FFI_decodeContext(x_4);
if (lean_obj_tag(x_5) == 0)
{
goto block_3;
}
else
{
lean_object* x_6; lean_object* x_7; lean_object* x_8; lean_object* x_9; 
x_6 = lean_ctor_get(x_5, 0);
lean_inc(x_6);
lean_dec_ref(x_5);
x_7 = lean_ctor_get(x_6, 0);
lean_inc(x_7);
x_8 = lean_ctor_get(x_6, 1);
lean_inc(x_8);
lean_dec(x_6);
x_9 = lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeId(x_8);
if (lean_obj_tag(x_9) == 0)
{
lean_dec(x_7);
goto block_3;
}
else
{
lean_object* x_10; lean_object* x_11; lean_object* x_12; lean_object* x_13; 
x_10 = lean_ctor_get(x_9, 0);
lean_inc(x_10);
lean_dec_ref(x_9);
x_11 = lean_ctor_get(x_10, 0);
lean_inc(x_11);
x_12 = lean_ctor_get(x_10, 1);
lean_inc(x_12);
lean_dec(x_10);
x_13 = lp_iris_x2dkernel_IrisKernel_FFI_decodeTypeId(x_12);
if (lean_obj_tag(x_13) == 0)
{
lean_dec(x_11);
lean_dec(x_7);
goto block_3;
}
else
{
lean_object* x_14; lean_object* x_15; lean_object* x_16; lean_object* x_17; 
x_14 = lean_ctor_get(x_13, 0);
lean_inc(x_14);
lean_dec_ref(x_13);
x_15 = lean_ctor_get(x_14, 0);
lean_inc(x_15);
lean_dec(x_14);
x_16 = lp_iris_x2dkernel_IrisKernel_Kernel_refl__(x_7, x_11, x_15);
x_17 = lp_iris_x2dkernel_IrisKernel_FFI_encodeSuccess(x_16);
lean_dec_ref(x_16);
return x_17;
}
}
}
block_3:
{
lean_object* x_2; 
x_2 = lp_iris_x2dkernel_IrisKernel_FFI_assumeFFI___closed__0;
return x_2;
}
}
}
LEAN_EXPORT lean_object* iris_kernel_symm(lean_object* x_1) {
_start:
{
lean_object* x_4; lean_object* x_5; 
x_4 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_create(x_1);
x_5 = lp_iris_x2dkernel_IrisKernel_FFI_decodeJudgment(x_4);
if (lean_obj_tag(x_5) == 0)
{
goto block_3;
}
else
{
lean_object* x_6; lean_object* x_7; lean_object* x_8; lean_object* x_9; 
x_6 = lean_ctor_get(x_5, 0);
lean_inc(x_6);
lean_dec_ref(x_5);
x_7 = lean_ctor_get(x_6, 0);
lean_inc(x_7);
x_8 = lean_ctor_get(x_6, 1);
lean_inc(x_8);
lean_dec(x_6);
x_9 = lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeId(x_8);
if (lean_obj_tag(x_9) == 0)
{
lean_dec(x_7);
goto block_3;
}
else
{
lean_object* x_10; lean_object* x_11; lean_object* x_12; lean_object* x_13; 
x_10 = lean_ctor_get(x_9, 0);
lean_inc(x_10);
lean_dec_ref(x_9);
x_11 = lean_ctor_get(x_10, 0);
lean_inc(x_11);
x_12 = lean_ctor_get(x_10, 1);
lean_inc(x_12);
lean_dec(x_10);
x_13 = lp_iris_x2dkernel_IrisKernel_FFI_decodeJudgment(x_12);
if (lean_obj_tag(x_13) == 0)
{
lean_dec(x_11);
lean_dec(x_7);
goto block_3;
}
else
{
lean_object* x_14; lean_object* x_15; lean_object* x_16; lean_object* x_17; 
x_14 = lean_ctor_get(x_13, 0);
lean_inc(x_14);
lean_dec_ref(x_13);
x_15 = lean_ctor_get(x_14, 0);
lean_inc(x_15);
lean_dec(x_14);
x_16 = lp_iris_x2dkernel_IrisKernel_Kernel_symm__(x_7, x_11, x_15);
lean_dec(x_15);
x_17 = lp_iris_x2dkernel_IrisKernel_FFI_wrapResult(x_16);
lean_dec(x_16);
return x_17;
}
}
}
block_3:
{
lean_object* x_2; 
x_2 = lp_iris_x2dkernel_IrisKernel_FFI_assumeFFI___closed__0;
return x_2;
}
}
}
LEAN_EXPORT lean_object* iris_kernel_trans(lean_object* x_1) {
_start:
{
lean_object* x_4; lean_object* x_5; 
x_4 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_create(x_1);
x_5 = lp_iris_x2dkernel_IrisKernel_FFI_decodeJudgment(x_4);
if (lean_obj_tag(x_5) == 0)
{
goto block_3;
}
else
{
lean_object* x_6; lean_object* x_7; lean_object* x_8; lean_object* x_9; 
x_6 = lean_ctor_get(x_5, 0);
lean_inc(x_6);
lean_dec_ref(x_5);
x_7 = lean_ctor_get(x_6, 0);
lean_inc(x_7);
x_8 = lean_ctor_get(x_6, 1);
lean_inc(x_8);
lean_dec(x_6);
x_9 = lp_iris_x2dkernel_IrisKernel_FFI_decodeJudgment(x_8);
if (lean_obj_tag(x_9) == 0)
{
lean_dec(x_7);
goto block_3;
}
else
{
lean_object* x_10; lean_object* x_11; lean_object* x_12; lean_object* x_13; 
x_10 = lean_ctor_get(x_9, 0);
lean_inc(x_10);
lean_dec_ref(x_9);
x_11 = lean_ctor_get(x_10, 0);
lean_inc(x_11);
lean_dec(x_10);
x_12 = lp_iris_x2dkernel_IrisKernel_Kernel_trans__(x_7, x_11);
lean_dec(x_7);
x_13 = lp_iris_x2dkernel_IrisKernel_FFI_wrapResult(x_12);
lean_dec(x_12);
return x_13;
}
}
block_3:
{
lean_object* x_2; 
x_2 = lp_iris_x2dkernel_IrisKernel_FFI_assumeFFI___closed__0;
return x_2;
}
}
}
LEAN_EXPORT lean_object* iris_kernel_congr(lean_object* x_1) {
_start:
{
lean_object* x_4; lean_object* x_5; 
x_4 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_create(x_1);
x_5 = lp_iris_x2dkernel_IrisKernel_FFI_decodeJudgment(x_4);
if (lean_obj_tag(x_5) == 0)
{
goto block_3;
}
else
{
lean_object* x_6; lean_object* x_7; lean_object* x_8; lean_object* x_9; 
x_6 = lean_ctor_get(x_5, 0);
lean_inc(x_6);
lean_dec_ref(x_5);
x_7 = lean_ctor_get(x_6, 0);
lean_inc(x_7);
x_8 = lean_ctor_get(x_6, 1);
lean_inc(x_8);
lean_dec(x_6);
x_9 = lp_iris_x2dkernel_IrisKernel_FFI_decodeJudgment(x_8);
if (lean_obj_tag(x_9) == 0)
{
lean_dec(x_7);
goto block_3;
}
else
{
lean_object* x_10; lean_object* x_11; lean_object* x_12; lean_object* x_13; 
x_10 = lean_ctor_get(x_9, 0);
lean_inc(x_10);
lean_dec_ref(x_9);
x_11 = lean_ctor_get(x_10, 0);
lean_inc(x_11);
x_12 = lean_ctor_get(x_10, 1);
lean_inc(x_12);
lean_dec(x_10);
x_13 = lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeId(x_12);
if (lean_obj_tag(x_13) == 0)
{
lean_dec(x_11);
lean_dec(x_7);
goto block_3;
}
else
{
lean_object* x_14; lean_object* x_15; lean_object* x_16; lean_object* x_17; 
x_14 = lean_ctor_get(x_13, 0);
lean_inc(x_14);
lean_dec_ref(x_13);
x_15 = lean_ctor_get(x_14, 0);
lean_inc(x_15);
lean_dec(x_14);
x_16 = lp_iris_x2dkernel_IrisKernel_Kernel_congr__(x_7, x_11, x_15);
lean_dec(x_7);
x_17 = lp_iris_x2dkernel_IrisKernel_FFI_wrapResult(x_16);
lean_dec(x_16);
return x_17;
}
}
}
block_3:
{
lean_object* x_2; 
x_2 = lp_iris_x2dkernel_IrisKernel_FFI_assumeFFI___closed__0;
return x_2;
}
}
}
static lean_object* _init_lp_iris_x2dkernel_IrisKernel_FFI_typeCheckNodeFullFFI___closed__0() {
_start:
{
uint8_t x_1; lean_object* x_2; 
x_1 = 3;
x_2 = lp_iris_x2dkernel_IrisKernel_FFI_encodeFailure(x_1);
return x_2;
}
}
LEAN_EXPORT lean_object* iris_kernel_type_check_node_full(lean_object* x_1) {
_start:
{
lean_object* x_4; lean_object* x_5; 
x_4 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_create(x_1);
x_5 = lp_iris_x2dkernel_IrisKernel_FFI_decodeTypeEnv(x_4);
if (lean_obj_tag(x_5) == 0)
{
goto block_3;
}
else
{
lean_object* x_6; lean_object* x_7; lean_object* x_8; lean_object* x_9; 
x_6 = lean_ctor_get(x_5, 0);
lean_inc(x_6);
lean_dec_ref(x_5);
x_7 = lean_ctor_get(x_6, 0);
lean_inc(x_7);
x_8 = lean_ctor_get(x_6, 1);
lean_inc(x_8);
lean_dec(x_6);
x_9 = lp_iris_x2dkernel_IrisKernel_FFI_decodeContext(x_8);
if (lean_obj_tag(x_9) == 0)
{
lean_dec(x_7);
goto block_3;
}
else
{
lean_object* x_10; lean_object* x_11; lean_object* x_12; lean_object* x_13; 
x_10 = lean_ctor_get(x_9, 0);
lean_inc(x_10);
lean_dec_ref(x_9);
x_11 = lean_ctor_get(x_10, 0);
lean_inc(x_11);
x_12 = lean_ctor_get(x_10, 1);
lean_inc(x_12);
lean_dec(x_10);
x_13 = lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeId(x_12);
if (lean_obj_tag(x_13) == 0)
{
lean_dec(x_11);
lean_dec(x_7);
goto block_3;
}
else
{
lean_object* x_14; lean_object* x_15; lean_object* x_16; lean_object* x_17; 
x_14 = lean_ctor_get(x_13, 0);
lean_inc(x_14);
lean_dec_ref(x_13);
x_15 = lean_ctor_get(x_14, 0);
lean_inc(x_15);
x_16 = lean_ctor_get(x_14, 1);
lean_inc(x_16);
lean_dec(x_14);
x_17 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readUInt8(x_16);
if (lean_obj_tag(x_17) == 0)
{
lean_dec(x_15);
lean_dec(x_11);
lean_dec(x_7);
goto block_3;
}
else
{
lean_object* x_18; lean_object* x_19; lean_object* x_20; uint8_t x_21; lean_object* x_22; 
x_18 = lean_ctor_get(x_17, 0);
lean_inc(x_18);
lean_dec_ref(x_17);
x_19 = lean_ctor_get(x_18, 0);
lean_inc(x_19);
x_20 = lean_ctor_get(x_18, 1);
lean_inc(x_20);
lean_dec(x_18);
x_21 = lean_unbox(x_19);
lean_dec(x_19);
x_22 = lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind(x_21);
if (lean_obj_tag(x_22) == 0)
{
lean_object* x_23; 
lean_dec(x_20);
lean_dec(x_15);
lean_dec(x_11);
lean_dec(x_7);
x_23 = lp_iris_x2dkernel_IrisKernel_FFI_typeCheckNodeFullFFI___closed__0;
return x_23;
}
else
{
lean_object* x_24; lean_object* x_25; 
x_24 = lean_ctor_get(x_22, 0);
lean_inc(x_24);
lean_dec_ref(x_22);
x_25 = lp_iris_x2dkernel_IrisKernel_FFI_decodeTypeId(x_20);
if (lean_obj_tag(x_25) == 0)
{
lean_dec(x_24);
lean_dec(x_15);
lean_dec(x_11);
lean_dec(x_7);
goto block_3;
}
else
{
lean_object* x_26; lean_object* x_27; uint8_t x_28; lean_object* x_29; lean_object* x_30; 
x_26 = lean_ctor_get(x_25, 0);
lean_inc(x_26);
lean_dec_ref(x_25);
x_27 = lean_ctor_get(x_26, 0);
lean_inc(x_27);
lean_dec(x_26);
x_28 = lean_unbox(x_24);
lean_dec(x_24);
x_29 = lp_iris_x2dkernel_IrisKernel_Kernel_typeCheckNode__(x_7, x_11, x_15, x_28, x_27);
x_30 = lp_iris_x2dkernel_IrisKernel_FFI_wrapResult(x_29);
lean_dec(x_29);
return x_30;
}
}
}
}
}
}
block_3:
{
lean_object* x_2; 
x_2 = lp_iris_x2dkernel_IrisKernel_FFI_assumeFFI___closed__0;
return x_2;
}
}
}
LEAN_EXPORT lean_object* iris_kernel_cost_subsume(lean_object* x_1) {
_start:
{
lean_object* x_4; lean_object* x_5; 
x_4 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_create(x_1);
x_5 = lp_iris_x2dkernel_IrisKernel_FFI_decodeJudgment(x_4);
if (lean_obj_tag(x_5) == 0)
{
goto block_3;
}
else
{
lean_object* x_6; lean_object* x_7; lean_object* x_8; lean_object* x_9; 
x_6 = lean_ctor_get(x_5, 0);
lean_inc(x_6);
lean_dec_ref(x_5);
x_7 = lean_ctor_get(x_6, 0);
lean_inc(x_7);
x_8 = lean_ctor_get(x_6, 1);
lean_inc(x_8);
lean_dec(x_6);
x_9 = lp_iris_x2dkernel_IrisKernel_FFI_decodeCostBound(x_8);
if (lean_obj_tag(x_9) == 0)
{
lean_dec(x_7);
goto block_3;
}
else
{
lean_object* x_10; lean_object* x_11; lean_object* x_12; lean_object* x_13; 
x_10 = lean_ctor_get(x_9, 0);
lean_inc(x_10);
lean_dec_ref(x_9);
x_11 = lean_ctor_get(x_10, 0);
lean_inc(x_11);
lean_dec(x_10);
x_12 = lp_iris_x2dkernel_IrisKernel_Kernel_costSubsume__(x_7, x_11);
x_13 = lp_iris_x2dkernel_IrisKernel_FFI_wrapResult(x_12);
lean_dec(x_12);
return x_13;
}
}
block_3:
{
lean_object* x_2; 
x_2 = lp_iris_x2dkernel_IrisKernel_FFI_assumeFFI___closed__0;
return x_2;
}
}
}
LEAN_EXPORT lean_object* iris_kernel_cost_leq_rule(lean_object* x_1) {
_start:
{
lean_object* x_4; lean_object* x_5; 
x_4 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_create(x_1);
x_5 = lp_iris_x2dkernel_IrisKernel_FFI_decodeCostBound(x_4);
if (lean_obj_tag(x_5) == 0)
{
goto block_3;
}
else
{
lean_object* x_6; lean_object* x_7; lean_object* x_8; lean_object* x_9; 
x_6 = lean_ctor_get(x_5, 0);
lean_inc(x_6);
lean_dec_ref(x_5);
x_7 = lean_ctor_get(x_6, 0);
lean_inc(x_7);
x_8 = lean_ctor_get(x_6, 1);
lean_inc(x_8);
lean_dec(x_6);
x_9 = lp_iris_x2dkernel_IrisKernel_FFI_decodeCostBound(x_8);
if (lean_obj_tag(x_9) == 0)
{
lean_dec(x_7);
goto block_3;
}
else
{
lean_object* x_10; lean_object* x_11; lean_object* x_12; lean_object* x_13; 
x_10 = lean_ctor_get(x_9, 0);
lean_inc(x_10);
lean_dec_ref(x_9);
x_11 = lean_ctor_get(x_10, 0);
lean_inc(x_11);
lean_dec(x_10);
x_12 = lp_iris_x2dkernel_IrisKernel_Kernel_costLeqRule__(x_7, x_11);
x_13 = lp_iris_x2dkernel_IrisKernel_FFI_wrapResult(x_12);
lean_dec(x_12);
return x_13;
}
}
block_3:
{
lean_object* x_2; 
x_2 = lp_iris_x2dkernel_IrisKernel_FFI_assumeFFI___closed__0;
return x_2;
}
}
}
LEAN_EXPORT lean_object* iris_kernel_refine_intro(lean_object* x_1) {
_start:
{
lean_object* x_4; lean_object* x_5; 
x_4 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_create(x_1);
x_5 = lp_iris_x2dkernel_IrisKernel_FFI_decodeTypeEnv(x_4);
if (lean_obj_tag(x_5) == 0)
{
goto block_3;
}
else
{
lean_object* x_6; lean_object* x_7; lean_object* x_8; lean_object* x_9; 
x_6 = lean_ctor_get(x_5, 0);
lean_inc(x_6);
lean_dec_ref(x_5);
x_7 = lean_ctor_get(x_6, 0);
lean_inc(x_7);
x_8 = lean_ctor_get(x_6, 1);
lean_inc(x_8);
lean_dec(x_6);
x_9 = lp_iris_x2dkernel_IrisKernel_FFI_decodeJudgment(x_8);
if (lean_obj_tag(x_9) == 0)
{
lean_dec(x_7);
goto block_3;
}
else
{
lean_object* x_10; lean_object* x_11; lean_object* x_12; lean_object* x_13; 
x_10 = lean_ctor_get(x_9, 0);
lean_inc(x_10);
lean_dec_ref(x_9);
x_11 = lean_ctor_get(x_10, 0);
lean_inc(x_11);
x_12 = lean_ctor_get(x_10, 1);
lean_inc(x_12);
lean_dec(x_10);
x_13 = lp_iris_x2dkernel_IrisKernel_FFI_decodeJudgment(x_12);
if (lean_obj_tag(x_13) == 0)
{
lean_dec(x_11);
lean_dec(x_7);
goto block_3;
}
else
{
lean_object* x_14; lean_object* x_15; lean_object* x_16; lean_object* x_17; 
x_14 = lean_ctor_get(x_13, 0);
lean_inc(x_14);
lean_dec_ref(x_13);
x_15 = lean_ctor_get(x_14, 0);
lean_inc(x_15);
x_16 = lean_ctor_get(x_14, 1);
lean_inc(x_16);
lean_dec(x_14);
x_17 = lp_iris_x2dkernel_IrisKernel_FFI_decodeTypeId(x_16);
if (lean_obj_tag(x_17) == 0)
{
lean_dec(x_15);
lean_dec(x_11);
lean_dec(x_7);
goto block_3;
}
else
{
lean_object* x_18; lean_object* x_19; lean_object* x_20; lean_object* x_21; 
x_18 = lean_ctor_get(x_17, 0);
lean_inc(x_18);
lean_dec_ref(x_17);
x_19 = lean_ctor_get(x_18, 0);
lean_inc(x_19);
lean_dec(x_18);
x_20 = lp_iris_x2dkernel_IrisKernel_Kernel_refineIntro(x_7, x_11, x_15, x_19);
lean_dec(x_11);
lean_dec(x_7);
x_21 = lp_iris_x2dkernel_IrisKernel_FFI_wrapResult(x_20);
lean_dec(x_20);
return x_21;
}
}
}
}
block_3:
{
lean_object* x_2; 
x_2 = lp_iris_x2dkernel_IrisKernel_FFI_assumeFFI___closed__0;
return x_2;
}
}
}
LEAN_EXPORT lean_object* iris_kernel_refine_elim(lean_object* x_1) {
_start:
{
lean_object* x_4; lean_object* x_5; 
x_4 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_create(x_1);
x_5 = lp_iris_x2dkernel_IrisKernel_FFI_decodeTypeEnv(x_4);
if (lean_obj_tag(x_5) == 0)
{
goto block_3;
}
else
{
lean_object* x_6; lean_object* x_7; lean_object* x_8; lean_object* x_9; 
x_6 = lean_ctor_get(x_5, 0);
lean_inc(x_6);
lean_dec_ref(x_5);
x_7 = lean_ctor_get(x_6, 0);
lean_inc(x_7);
x_8 = lean_ctor_get(x_6, 1);
lean_inc(x_8);
lean_dec(x_6);
x_9 = lp_iris_x2dkernel_IrisKernel_FFI_decodeJudgment(x_8);
if (lean_obj_tag(x_9) == 0)
{
lean_dec(x_7);
goto block_3;
}
else
{
lean_object* x_10; lean_object* x_11; lean_object* x_12; lean_object* x_13; 
x_10 = lean_ctor_get(x_9, 0);
lean_inc(x_10);
lean_dec_ref(x_9);
x_11 = lean_ctor_get(x_10, 0);
lean_inc(x_11);
lean_dec(x_10);
x_12 = lp_iris_x2dkernel_IrisKernel_Kernel_refineElim(x_7, x_11);
lean_dec(x_7);
x_13 = lp_iris_x2dkernel_IrisKernel_FFI_wrapResult(x_12);
lean_dec(x_12);
return x_13;
}
}
block_3:
{
lean_object* x_2; 
x_2 = lp_iris_x2dkernel_IrisKernel_FFI_assumeFFI___closed__0;
return x_2;
}
}
}
LEAN_EXPORT lean_object* iris_kernel_nat_ind(lean_object* x_1) {
_start:
{
lean_object* x_4; lean_object* x_5; 
x_4 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_create(x_1);
x_5 = lp_iris_x2dkernel_IrisKernel_FFI_decodeJudgment(x_4);
if (lean_obj_tag(x_5) == 0)
{
goto block_3;
}
else
{
lean_object* x_6; lean_object* x_7; lean_object* x_8; lean_object* x_9; 
x_6 = lean_ctor_get(x_5, 0);
lean_inc(x_6);
lean_dec_ref(x_5);
x_7 = lean_ctor_get(x_6, 0);
lean_inc(x_7);
x_8 = lean_ctor_get(x_6, 1);
lean_inc(x_8);
lean_dec(x_6);
x_9 = lp_iris_x2dkernel_IrisKernel_FFI_decodeJudgment(x_8);
if (lean_obj_tag(x_9) == 0)
{
lean_dec(x_7);
goto block_3;
}
else
{
lean_object* x_10; lean_object* x_11; lean_object* x_12; lean_object* x_13; 
x_10 = lean_ctor_get(x_9, 0);
lean_inc(x_10);
lean_dec_ref(x_9);
x_11 = lean_ctor_get(x_10, 0);
lean_inc(x_11);
x_12 = lean_ctor_get(x_10, 1);
lean_inc(x_12);
lean_dec(x_10);
x_13 = lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeId(x_12);
if (lean_obj_tag(x_13) == 0)
{
lean_dec(x_11);
lean_dec(x_7);
goto block_3;
}
else
{
lean_object* x_14; lean_object* x_15; lean_object* x_16; lean_object* x_17; 
x_14 = lean_ctor_get(x_13, 0);
lean_inc(x_14);
lean_dec_ref(x_13);
x_15 = lean_ctor_get(x_14, 0);
lean_inc(x_15);
lean_dec(x_14);
x_16 = lp_iris_x2dkernel_IrisKernel_Kernel_natInd(x_7, x_11, x_15);
lean_dec(x_7);
x_17 = lp_iris_x2dkernel_IrisKernel_FFI_wrapResult(x_16);
lean_dec(x_16);
return x_17;
}
}
}
block_3:
{
lean_object* x_2; 
x_2 = lp_iris_x2dkernel_IrisKernel_FFI_assumeFFI___closed__0;
return x_2;
}
}
}
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_structuralIndFFI_decodeCases(lean_object* x_1, lean_object* x_2, lean_object* x_3) {
_start:
{
lean_object* x_4; uint8_t x_5; 
x_4 = lean_unsigned_to_nat(0u);
x_5 = lean_nat_dec_eq(x_2, x_4);
if (x_5 == 1)
{
lean_object* x_6; lean_object* x_7; lean_object* x_8; 
lean_dec(x_2);
x_6 = l_List_reverse___redArg(x_3);
x_7 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_7, 0, x_6);
lean_ctor_set(x_7, 1, x_1);
x_8 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_8, 0, x_7);
return x_8;
}
else
{
lean_object* x_9; 
x_9 = lp_iris_x2dkernel_IrisKernel_FFI_decodeJudgment(x_1);
if (lean_obj_tag(x_9) == 0)
{
lean_object* x_10; 
lean_dec(x_3);
lean_dec(x_2);
x_10 = lean_box(0);
return x_10;
}
else
{
lean_object* x_11; uint8_t x_12; 
x_11 = lean_ctor_get(x_9, 0);
lean_inc(x_11);
lean_dec_ref(x_9);
x_12 = !lean_is_exclusive(x_11);
if (x_12 == 0)
{
lean_object* x_13; lean_object* x_14; lean_object* x_15; 
x_13 = lean_ctor_get(x_11, 1);
x_14 = lean_unsigned_to_nat(1u);
x_15 = lean_nat_sub(x_2, x_14);
lean_dec(x_2);
lean_ctor_set_tag(x_11, 1);
lean_ctor_set(x_11, 1, x_3);
x_1 = x_13;
x_2 = x_15;
x_3 = x_11;
goto _start;
}
else
{
lean_object* x_17; lean_object* x_18; lean_object* x_19; lean_object* x_20; lean_object* x_21; 
x_17 = lean_ctor_get(x_11, 0);
x_18 = lean_ctor_get(x_11, 1);
lean_inc(x_18);
lean_inc(x_17);
lean_dec(x_11);
x_19 = lean_unsigned_to_nat(1u);
x_20 = lean_nat_sub(x_2, x_19);
lean_dec(x_2);
x_21 = lean_alloc_ctor(1, 2, 0);
lean_ctor_set(x_21, 0, x_17);
lean_ctor_set(x_21, 1, x_3);
x_1 = x_18;
x_2 = x_20;
x_3 = x_21;
goto _start;
}
}
}
}
}
LEAN_EXPORT lean_object* iris_kernel_structural_ind(lean_object* x_1) {
_start:
{
lean_object* x_4; lean_object* x_5; 
x_4 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_create(x_1);
x_5 = lp_iris_x2dkernel_IrisKernel_FFI_decodeTypeEnv(x_4);
if (lean_obj_tag(x_5) == 0)
{
goto block_3;
}
else
{
lean_object* x_6; lean_object* x_7; lean_object* x_8; lean_object* x_9; 
x_6 = lean_ctor_get(x_5, 0);
lean_inc(x_6);
lean_dec_ref(x_5);
x_7 = lean_ctor_get(x_6, 0);
lean_inc(x_7);
x_8 = lean_ctor_get(x_6, 1);
lean_inc(x_8);
lean_dec(x_6);
x_9 = lp_iris_x2dkernel_IrisKernel_FFI_decodeTypeId(x_8);
if (lean_obj_tag(x_9) == 0)
{
lean_dec(x_7);
goto block_3;
}
else
{
lean_object* x_10; lean_object* x_11; lean_object* x_12; lean_object* x_13; 
x_10 = lean_ctor_get(x_9, 0);
lean_inc(x_10);
lean_dec_ref(x_9);
x_11 = lean_ctor_get(x_10, 0);
lean_inc(x_11);
x_12 = lean_ctor_get(x_10, 1);
lean_inc(x_12);
lean_dec(x_10);
x_13 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readUInt16LE(x_12);
if (lean_obj_tag(x_13) == 0)
{
lean_dec(x_11);
lean_dec(x_7);
goto block_3;
}
else
{
lean_object* x_14; lean_object* x_15; lean_object* x_16; uint16_t x_17; lean_object* x_18; lean_object* x_19; lean_object* x_20; 
x_14 = lean_ctor_get(x_13, 0);
lean_inc(x_14);
lean_dec_ref(x_13);
x_15 = lean_ctor_get(x_14, 0);
lean_inc(x_15);
x_16 = lean_ctor_get(x_14, 1);
lean_inc(x_16);
lean_dec(x_14);
x_17 = lean_unbox(x_15);
lean_dec(x_15);
x_18 = lean_uint16_to_nat(x_17);
x_19 = lean_box(0);
x_20 = lp_iris_x2dkernel_IrisKernel_FFI_structuralIndFFI_decodeCases(x_16, x_18, x_19);
if (lean_obj_tag(x_20) == 0)
{
lean_dec(x_11);
lean_dec(x_7);
goto block_3;
}
else
{
lean_object* x_21; lean_object* x_22; lean_object* x_23; lean_object* x_24; 
x_21 = lean_ctor_get(x_20, 0);
lean_inc(x_21);
lean_dec_ref(x_20);
x_22 = lean_ctor_get(x_21, 0);
lean_inc(x_22);
x_23 = lean_ctor_get(x_21, 1);
lean_inc(x_23);
lean_dec(x_21);
x_24 = lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeId(x_23);
if (lean_obj_tag(x_24) == 0)
{
lean_dec(x_22);
lean_dec(x_11);
lean_dec(x_7);
goto block_3;
}
else
{
lean_object* x_25; lean_object* x_26; lean_object* x_27; lean_object* x_28; 
x_25 = lean_ctor_get(x_24, 0);
lean_inc(x_25);
lean_dec_ref(x_24);
x_26 = lean_ctor_get(x_25, 0);
lean_inc(x_26);
lean_dec(x_25);
x_27 = lp_iris_x2dkernel_IrisKernel_Kernel_structuralInd(x_7, x_11, x_22, x_26);
lean_dec(x_11);
lean_dec(x_7);
x_28 = lp_iris_x2dkernel_IrisKernel_FFI_wrapResult(x_27);
lean_dec(x_27);
return x_28;
}
}
}
}
}
block_3:
{
lean_object* x_2; 
x_2 = lp_iris_x2dkernel_IrisKernel_FFI_assumeFFI___closed__0;
return x_2;
}
}
}
LEAN_EXPORT lean_object* iris_kernel_let_bind(lean_object* x_1) {
_start:
{
lean_object* x_4; lean_object* x_5; 
x_4 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_create(x_1);
x_5 = lp_iris_x2dkernel_IrisKernel_FFI_decodeContext(x_4);
if (lean_obj_tag(x_5) == 0)
{
goto block_3;
}
else
{
lean_object* x_6; lean_object* x_7; lean_object* x_8; lean_object* x_9; 
x_6 = lean_ctor_get(x_5, 0);
lean_inc(x_6);
lean_dec_ref(x_5);
x_7 = lean_ctor_get(x_6, 0);
lean_inc(x_7);
x_8 = lean_ctor_get(x_6, 1);
lean_inc(x_8);
lean_dec(x_6);
x_9 = lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeId(x_8);
if (lean_obj_tag(x_9) == 0)
{
lean_dec(x_7);
goto block_3;
}
else
{
lean_object* x_10; lean_object* x_11; lean_object* x_12; lean_object* x_13; 
x_10 = lean_ctor_get(x_9, 0);
lean_inc(x_10);
lean_dec_ref(x_9);
x_11 = lean_ctor_get(x_10, 0);
lean_inc(x_11);
x_12 = lean_ctor_get(x_10, 1);
lean_inc(x_12);
lean_dec(x_10);
x_13 = lp_iris_x2dkernel_IrisKernel_FFI_decodeBinderId(x_12);
if (lean_obj_tag(x_13) == 0)
{
lean_dec(x_11);
lean_dec(x_7);
goto block_3;
}
else
{
lean_object* x_14; lean_object* x_15; lean_object* x_16; lean_object* x_17; 
x_14 = lean_ctor_get(x_13, 0);
lean_inc(x_14);
lean_dec_ref(x_13);
x_15 = lean_ctor_get(x_14, 0);
lean_inc(x_15);
x_16 = lean_ctor_get(x_14, 1);
lean_inc(x_16);
lean_dec(x_14);
x_17 = lp_iris_x2dkernel_IrisKernel_FFI_decodeJudgment(x_16);
if (lean_obj_tag(x_17) == 0)
{
lean_dec(x_15);
lean_dec(x_11);
lean_dec(x_7);
goto block_3;
}
else
{
lean_object* x_18; lean_object* x_19; lean_object* x_20; lean_object* x_21; 
x_18 = lean_ctor_get(x_17, 0);
lean_inc(x_18);
lean_dec_ref(x_17);
x_19 = lean_ctor_get(x_18, 0);
lean_inc(x_19);
x_20 = lean_ctor_get(x_18, 1);
lean_inc(x_20);
lean_dec(x_18);
x_21 = lp_iris_x2dkernel_IrisKernel_FFI_decodeJudgment(x_20);
if (lean_obj_tag(x_21) == 0)
{
lean_dec(x_19);
lean_dec(x_15);
lean_dec(x_11);
lean_dec(x_7);
goto block_3;
}
else
{
lean_object* x_22; lean_object* x_23; lean_object* x_24; lean_object* x_25; 
x_22 = lean_ctor_get(x_21, 0);
lean_inc(x_22);
lean_dec_ref(x_21);
x_23 = lean_ctor_get(x_22, 0);
lean_inc(x_23);
lean_dec(x_22);
x_24 = lp_iris_x2dkernel_IrisKernel_Kernel_letBind(x_7, x_11, x_15, x_19, x_23);
x_25 = lp_iris_x2dkernel_IrisKernel_FFI_wrapResult(x_24);
lean_dec(x_24);
return x_25;
}
}
}
}
}
block_3:
{
lean_object* x_2; 
x_2 = lp_iris_x2dkernel_IrisKernel_FFI_assumeFFI___closed__0;
return x_2;
}
}
}
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_matchElimFFI_decodeArms(lean_object* x_1, lean_object* x_2, lean_object* x_3) {
_start:
{
lean_object* x_4; uint8_t x_5; 
x_4 = lean_unsigned_to_nat(0u);
x_5 = lean_nat_dec_eq(x_2, x_4);
if (x_5 == 1)
{
lean_object* x_6; lean_object* x_7; lean_object* x_8; 
lean_dec(x_2);
x_6 = l_List_reverse___redArg(x_3);
x_7 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_7, 0, x_6);
lean_ctor_set(x_7, 1, x_1);
x_8 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_8, 0, x_7);
return x_8;
}
else
{
lean_object* x_9; 
x_9 = lp_iris_x2dkernel_IrisKernel_FFI_decodeJudgment(x_1);
if (lean_obj_tag(x_9) == 0)
{
lean_object* x_10; 
lean_dec(x_3);
lean_dec(x_2);
x_10 = lean_box(0);
return x_10;
}
else
{
lean_object* x_11; uint8_t x_12; 
x_11 = lean_ctor_get(x_9, 0);
lean_inc(x_11);
lean_dec_ref(x_9);
x_12 = !lean_is_exclusive(x_11);
if (x_12 == 0)
{
lean_object* x_13; lean_object* x_14; lean_object* x_15; 
x_13 = lean_ctor_get(x_11, 1);
x_14 = lean_unsigned_to_nat(1u);
x_15 = lean_nat_sub(x_2, x_14);
lean_dec(x_2);
lean_ctor_set_tag(x_11, 1);
lean_ctor_set(x_11, 1, x_3);
x_1 = x_13;
x_2 = x_15;
x_3 = x_11;
goto _start;
}
else
{
lean_object* x_17; lean_object* x_18; lean_object* x_19; lean_object* x_20; lean_object* x_21; 
x_17 = lean_ctor_get(x_11, 0);
x_18 = lean_ctor_get(x_11, 1);
lean_inc(x_18);
lean_inc(x_17);
lean_dec(x_11);
x_19 = lean_unsigned_to_nat(1u);
x_20 = lean_nat_sub(x_2, x_19);
lean_dec(x_2);
x_21 = lean_alloc_ctor(1, 2, 0);
lean_ctor_set(x_21, 0, x_17);
lean_ctor_set(x_21, 1, x_3);
x_1 = x_18;
x_2 = x_20;
x_3 = x_21;
goto _start;
}
}
}
}
}
LEAN_EXPORT lean_object* iris_kernel_match_elim(lean_object* x_1) {
_start:
{
lean_object* x_4; lean_object* x_5; 
x_4 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_create(x_1);
x_5 = lp_iris_x2dkernel_IrisKernel_FFI_decodeJudgment(x_4);
if (lean_obj_tag(x_5) == 0)
{
goto block_3;
}
else
{
lean_object* x_6; lean_object* x_7; lean_object* x_8; lean_object* x_9; 
x_6 = lean_ctor_get(x_5, 0);
lean_inc(x_6);
lean_dec_ref(x_5);
x_7 = lean_ctor_get(x_6, 0);
lean_inc(x_7);
x_8 = lean_ctor_get(x_6, 1);
lean_inc(x_8);
lean_dec(x_6);
x_9 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readUInt16LE(x_8);
if (lean_obj_tag(x_9) == 0)
{
lean_dec(x_7);
goto block_3;
}
else
{
lean_object* x_10; lean_object* x_11; lean_object* x_12; uint16_t x_13; lean_object* x_14; lean_object* x_15; lean_object* x_16; 
x_10 = lean_ctor_get(x_9, 0);
lean_inc(x_10);
lean_dec_ref(x_9);
x_11 = lean_ctor_get(x_10, 0);
lean_inc(x_11);
x_12 = lean_ctor_get(x_10, 1);
lean_inc(x_12);
lean_dec(x_10);
x_13 = lean_unbox(x_11);
lean_dec(x_11);
x_14 = lean_uint16_to_nat(x_13);
x_15 = lean_box(0);
x_16 = lp_iris_x2dkernel_IrisKernel_FFI_matchElimFFI_decodeArms(x_12, x_14, x_15);
if (lean_obj_tag(x_16) == 0)
{
lean_dec(x_7);
goto block_3;
}
else
{
lean_object* x_17; lean_object* x_18; lean_object* x_19; lean_object* x_20; 
x_17 = lean_ctor_get(x_16, 0);
lean_inc(x_17);
lean_dec_ref(x_16);
x_18 = lean_ctor_get(x_17, 0);
lean_inc(x_18);
x_19 = lean_ctor_get(x_17, 1);
lean_inc(x_19);
lean_dec(x_17);
x_20 = lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeId(x_19);
if (lean_obj_tag(x_20) == 0)
{
lean_dec(x_18);
lean_dec(x_7);
goto block_3;
}
else
{
lean_object* x_21; lean_object* x_22; lean_object* x_23; lean_object* x_24; 
x_21 = lean_ctor_get(x_20, 0);
lean_inc(x_21);
lean_dec_ref(x_20);
x_22 = lean_ctor_get(x_21, 0);
lean_inc(x_22);
lean_dec(x_21);
x_23 = lp_iris_x2dkernel_IrisKernel_Kernel_matchElim(x_7, x_18, x_22);
x_24 = lp_iris_x2dkernel_IrisKernel_FFI_wrapResult(x_23);
lean_dec(x_23);
return x_24;
}
}
}
}
block_3:
{
lean_object* x_2; 
x_2 = lp_iris_x2dkernel_IrisKernel_FFI_assumeFFI___closed__0;
return x_2;
}
}
}
LEAN_EXPORT lean_object* iris_kernel_fold_rule(lean_object* x_1) {
_start:
{
lean_object* x_4; lean_object* x_5; 
x_4 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_create(x_1);
x_5 = lp_iris_x2dkernel_IrisKernel_FFI_decodeJudgment(x_4);
if (lean_obj_tag(x_5) == 0)
{
goto block_3;
}
else
{
lean_object* x_6; lean_object* x_7; lean_object* x_8; lean_object* x_9; 
x_6 = lean_ctor_get(x_5, 0);
lean_inc(x_6);
lean_dec_ref(x_5);
x_7 = lean_ctor_get(x_6, 0);
lean_inc(x_7);
x_8 = lean_ctor_get(x_6, 1);
lean_inc(x_8);
lean_dec(x_6);
x_9 = lp_iris_x2dkernel_IrisKernel_FFI_decodeJudgment(x_8);
if (lean_obj_tag(x_9) == 0)
{
lean_dec(x_7);
goto block_3;
}
else
{
lean_object* x_10; lean_object* x_11; lean_object* x_12; lean_object* x_13; 
x_10 = lean_ctor_get(x_9, 0);
lean_inc(x_10);
lean_dec_ref(x_9);
x_11 = lean_ctor_get(x_10, 0);
lean_inc(x_11);
x_12 = lean_ctor_get(x_10, 1);
lean_inc(x_12);
lean_dec(x_10);
x_13 = lp_iris_x2dkernel_IrisKernel_FFI_decodeJudgment(x_12);
if (lean_obj_tag(x_13) == 0)
{
lean_dec(x_11);
lean_dec(x_7);
goto block_3;
}
else
{
lean_object* x_14; lean_object* x_15; lean_object* x_16; lean_object* x_17; 
x_14 = lean_ctor_get(x_13, 0);
lean_inc(x_14);
lean_dec_ref(x_13);
x_15 = lean_ctor_get(x_14, 0);
lean_inc(x_15);
x_16 = lean_ctor_get(x_14, 1);
lean_inc(x_16);
lean_dec(x_14);
x_17 = lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeId(x_16);
if (lean_obj_tag(x_17) == 0)
{
lean_dec(x_15);
lean_dec(x_11);
lean_dec(x_7);
goto block_3;
}
else
{
lean_object* x_18; lean_object* x_19; lean_object* x_20; lean_object* x_21; 
x_18 = lean_ctor_get(x_17, 0);
lean_inc(x_18);
lean_dec_ref(x_17);
x_19 = lean_ctor_get(x_18, 0);
lean_inc(x_19);
lean_dec(x_18);
x_20 = lp_iris_x2dkernel_IrisKernel_Kernel_foldRule(x_7, x_11, x_15, x_19);
lean_dec(x_15);
lean_dec(x_7);
x_21 = lp_iris_x2dkernel_IrisKernel_FFI_wrapResult(x_20);
lean_dec(x_20);
return x_21;
}
}
}
}
block_3:
{
lean_object* x_2; 
x_2 = lp_iris_x2dkernel_IrisKernel_FFI_assumeFFI___closed__0;
return x_2;
}
}
}
LEAN_EXPORT lean_object* iris_kernel_type_abst(lean_object* x_1) {
_start:
{
lean_object* x_4; lean_object* x_5; 
x_4 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_create(x_1);
x_5 = lp_iris_x2dkernel_IrisKernel_FFI_decodeTypeEnv(x_4);
if (lean_obj_tag(x_5) == 0)
{
goto block_3;
}
else
{
lean_object* x_6; lean_object* x_7; lean_object* x_8; lean_object* x_9; 
x_6 = lean_ctor_get(x_5, 0);
lean_inc(x_6);
lean_dec_ref(x_5);
x_7 = lean_ctor_get(x_6, 0);
lean_inc(x_7);
x_8 = lean_ctor_get(x_6, 1);
lean_inc(x_8);
lean_dec(x_6);
x_9 = lp_iris_x2dkernel_IrisKernel_FFI_decodeJudgment(x_8);
if (lean_obj_tag(x_9) == 0)
{
lean_dec(x_7);
goto block_3;
}
else
{
lean_object* x_10; lean_object* x_11; lean_object* x_12; lean_object* x_13; 
x_10 = lean_ctor_get(x_9, 0);
lean_inc(x_10);
lean_dec_ref(x_9);
x_11 = lean_ctor_get(x_10, 0);
lean_inc(x_11);
x_12 = lean_ctor_get(x_10, 1);
lean_inc(x_12);
lean_dec(x_10);
x_13 = lp_iris_x2dkernel_IrisKernel_FFI_decodeTypeId(x_12);
if (lean_obj_tag(x_13) == 0)
{
lean_dec(x_11);
lean_dec(x_7);
goto block_3;
}
else
{
lean_object* x_14; lean_object* x_15; lean_object* x_16; lean_object* x_17; 
x_14 = lean_ctor_get(x_13, 0);
lean_inc(x_14);
lean_dec_ref(x_13);
x_15 = lean_ctor_get(x_14, 0);
lean_inc(x_15);
lean_dec(x_14);
x_16 = lp_iris_x2dkernel_IrisKernel_Kernel_typeAbst(x_7, x_11, x_15);
x_17 = lp_iris_x2dkernel_IrisKernel_FFI_wrapResult(x_16);
lean_dec(x_16);
return x_17;
}
}
}
block_3:
{
lean_object* x_2; 
x_2 = lp_iris_x2dkernel_IrisKernel_FFI_assumeFFI___closed__0;
return x_2;
}
}
}
LEAN_EXPORT lean_object* iris_kernel_type_app(lean_object* x_1) {
_start:
{
lean_object* x_4; lean_object* x_5; 
x_4 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_create(x_1);
x_5 = lp_iris_x2dkernel_IrisKernel_FFI_decodeTypeEnv(x_4);
if (lean_obj_tag(x_5) == 0)
{
goto block_3;
}
else
{
lean_object* x_6; lean_object* x_7; lean_object* x_8; lean_object* x_9; 
x_6 = lean_ctor_get(x_5, 0);
lean_inc(x_6);
lean_dec_ref(x_5);
x_7 = lean_ctor_get(x_6, 0);
lean_inc(x_7);
x_8 = lean_ctor_get(x_6, 1);
lean_inc(x_8);
lean_dec(x_6);
x_9 = lp_iris_x2dkernel_IrisKernel_FFI_decodeJudgment(x_8);
if (lean_obj_tag(x_9) == 0)
{
lean_dec(x_7);
goto block_3;
}
else
{
lean_object* x_10; lean_object* x_11; lean_object* x_12; lean_object* x_13; 
x_10 = lean_ctor_get(x_9, 0);
lean_inc(x_10);
lean_dec_ref(x_9);
x_11 = lean_ctor_get(x_10, 0);
lean_inc(x_11);
x_12 = lean_ctor_get(x_10, 1);
lean_inc(x_12);
lean_dec(x_10);
x_13 = lp_iris_x2dkernel_IrisKernel_FFI_decodeTypeId(x_12);
if (lean_obj_tag(x_13) == 0)
{
lean_dec(x_11);
lean_dec(x_7);
goto block_3;
}
else
{
lean_object* x_14; lean_object* x_15; lean_object* x_16; lean_object* x_17; 
x_14 = lean_ctor_get(x_13, 0);
lean_inc(x_14);
lean_dec_ref(x_13);
x_15 = lean_ctor_get(x_14, 0);
lean_inc(x_15);
lean_dec(x_14);
x_16 = lp_iris_x2dkernel_IrisKernel_Kernel_typeApp(x_7, x_11, x_15);
x_17 = lp_iris_x2dkernel_IrisKernel_FFI_wrapResult(x_16);
lean_dec(x_16);
return x_17;
}
}
}
block_3:
{
lean_object* x_2; 
x_2 = lp_iris_x2dkernel_IrisKernel_FFI_assumeFFI___closed__0;
return x_2;
}
}
}
LEAN_EXPORT lean_object* iris_kernel_guard_rule(lean_object* x_1) {
_start:
{
lean_object* x_4; lean_object* x_5; 
x_4 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_create(x_1);
x_5 = lp_iris_x2dkernel_IrisKernel_FFI_decodeJudgment(x_4);
if (lean_obj_tag(x_5) == 0)
{
goto block_3;
}
else
{
lean_object* x_6; lean_object* x_7; lean_object* x_8; lean_object* x_9; 
x_6 = lean_ctor_get(x_5, 0);
lean_inc(x_6);
lean_dec_ref(x_5);
x_7 = lean_ctor_get(x_6, 0);
lean_inc(x_7);
x_8 = lean_ctor_get(x_6, 1);
lean_inc(x_8);
lean_dec(x_6);
x_9 = lp_iris_x2dkernel_IrisKernel_FFI_decodeJudgment(x_8);
if (lean_obj_tag(x_9) == 0)
{
lean_dec(x_7);
goto block_3;
}
else
{
lean_object* x_10; lean_object* x_11; lean_object* x_12; lean_object* x_13; 
x_10 = lean_ctor_get(x_9, 0);
lean_inc(x_10);
lean_dec_ref(x_9);
x_11 = lean_ctor_get(x_10, 0);
lean_inc(x_11);
x_12 = lean_ctor_get(x_10, 1);
lean_inc(x_12);
lean_dec(x_10);
x_13 = lp_iris_x2dkernel_IrisKernel_FFI_decodeJudgment(x_12);
if (lean_obj_tag(x_13) == 0)
{
lean_dec(x_11);
lean_dec(x_7);
goto block_3;
}
else
{
lean_object* x_14; lean_object* x_15; lean_object* x_16; lean_object* x_17; 
x_14 = lean_ctor_get(x_13, 0);
lean_inc(x_14);
lean_dec_ref(x_13);
x_15 = lean_ctor_get(x_14, 0);
lean_inc(x_15);
x_16 = lean_ctor_get(x_14, 1);
lean_inc(x_16);
lean_dec(x_14);
x_17 = lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeId(x_16);
if (lean_obj_tag(x_17) == 0)
{
lean_dec(x_15);
lean_dec(x_11);
lean_dec(x_7);
goto block_3;
}
else
{
lean_object* x_18; lean_object* x_19; lean_object* x_20; lean_object* x_21; 
x_18 = lean_ctor_get(x_17, 0);
lean_inc(x_18);
lean_dec_ref(x_17);
x_19 = lean_ctor_get(x_18, 0);
lean_inc(x_19);
lean_dec(x_18);
x_20 = lp_iris_x2dkernel_IrisKernel_Kernel_guardRule(x_7, x_11, x_15, x_19);
lean_dec(x_15);
lean_dec(x_11);
x_21 = lp_iris_x2dkernel_IrisKernel_FFI_wrapResult(x_20);
lean_dec(x_20);
return x_21;
}
}
}
}
block_3:
{
lean_object* x_2; 
x_2 = lp_iris_x2dkernel_IrisKernel_FFI_assumeFFI___closed__0;
return x_2;
}
}
}
lean_object* initialize_Init(uint8_t builtin);
lean_object* initialize_iris_x2dkernel_IrisKernel_Types(uint8_t builtin);
lean_object* initialize_iris_x2dkernel_IrisKernel_Eval(uint8_t builtin);
lean_object* initialize_iris_x2dkernel_IrisKernel_Kernel(uint8_t builtin);
static bool _G_initialized = false;
LEAN_EXPORT lean_object* initialize_iris_x2dkernel_IrisKernel_FFI(uint8_t builtin) {
lean_object * res;
if (_G_initialized) return lean_io_result_mk_ok(lean_box(0));
_G_initialized = true;
res = initialize_Init(builtin);
if (lean_io_result_is_error(res)) return res;
lean_dec_ref(res);
res = initialize_iris_x2dkernel_IrisKernel_Types(builtin);
if (lean_io_result_is_error(res)) return res;
lean_dec_ref(res);
res = initialize_iris_x2dkernel_IrisKernel_Eval(builtin);
if (lean_io_result_is_error(res)) return res;
lean_dec_ref(res);
res = initialize_iris_x2dkernel_IrisKernel_Kernel(builtin);
if (lean_io_result_is_error(res)) return res;
lean_dec_ref(res);
lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readInt64LE___closed__0 = _init_lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readInt64LE___closed__0();
lean_mark_persistent(lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readInt64LE___closed__0);
lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readInt64LE___closed__1 = _init_lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readInt64LE___closed__1();
lean_mark_persistent(lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readInt64LE___closed__1);
lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readInt64LE___closed__2 = _init_lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readInt64LE___closed__2();
lean_mark_persistent(lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readInt64LE___closed__2);
lp_iris_x2dkernel_IrisKernel_FFI_encodeSuccess___closed__0 = _init_lp_iris_x2dkernel_IrisKernel_FFI_encodeSuccess___closed__0();
lean_mark_persistent(lp_iris_x2dkernel_IrisKernel_FFI_encodeSuccess___closed__0);
lp_iris_x2dkernel_IrisKernel_FFI_encodeFailure___closed__0 = _init_lp_iris_x2dkernel_IrisKernel_FFI_encodeFailure___closed__0();
lean_mark_persistent(lp_iris_x2dkernel_IrisKernel_FFI_encodeFailure___closed__0);
iris_lean_kernel_version = _init_iris_lean_kernel_version();
lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__0 = _init_lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__0();
lean_mark_persistent(lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__0);
lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__1 = _init_lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__1();
lean_mark_persistent(lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__1);
lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__2 = _init_lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__2();
lean_mark_persistent(lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__2);
lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__3 = _init_lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__3();
lean_mark_persistent(lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__3);
lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__4 = _init_lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__4();
lean_mark_persistent(lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__4);
lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__5 = _init_lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__5();
lean_mark_persistent(lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__5);
lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__6 = _init_lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__6();
lean_mark_persistent(lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__6);
lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__7 = _init_lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__7();
lean_mark_persistent(lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__7);
lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__8 = _init_lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__8();
lean_mark_persistent(lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__8);
lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__9 = _init_lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__9();
lean_mark_persistent(lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__9);
lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__10 = _init_lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__10();
lean_mark_persistent(lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__10);
lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__11 = _init_lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__11();
lean_mark_persistent(lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__11);
lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__12 = _init_lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__12();
lean_mark_persistent(lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__12);
lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__13 = _init_lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__13();
lean_mark_persistent(lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__13);
lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__14 = _init_lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__14();
lean_mark_persistent(lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__14);
lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__15 = _init_lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__15();
lean_mark_persistent(lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__15);
lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__16 = _init_lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__16();
lean_mark_persistent(lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__16);
lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__17 = _init_lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__17();
lean_mark_persistent(lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__17);
lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__18 = _init_lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__18();
lean_mark_persistent(lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__18);
lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__19 = _init_lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__19();
lean_mark_persistent(lp_iris_x2dkernel_IrisKernel_FFI_decodeNodeKind___closed__19);
lp_iris_x2dkernel_IrisKernel_FFI_wrapResult___closed__0 = _init_lp_iris_x2dkernel_IrisKernel_FFI_wrapResult___closed__0();
lean_mark_persistent(lp_iris_x2dkernel_IrisKernel_FFI_wrapResult___closed__0);
lp_iris_x2dkernel_IrisKernel_FFI_assumeFFI___closed__0 = _init_lp_iris_x2dkernel_IrisKernel_FFI_assumeFFI___closed__0();
lean_mark_persistent(lp_iris_x2dkernel_IrisKernel_FFI_assumeFFI___closed__0);
lp_iris_x2dkernel_IrisKernel_FFI_typeCheckNodeFullFFI___closed__0 = _init_lp_iris_x2dkernel_IrisKernel_FFI_typeCheckNodeFullFFI___closed__0();
lean_mark_persistent(lp_iris_x2dkernel_IrisKernel_FFI_typeCheckNodeFullFFI___closed__0);
return lean_io_result_mk_ok(lean_box(0));
}
#ifdef __cplusplus
}
#endif
