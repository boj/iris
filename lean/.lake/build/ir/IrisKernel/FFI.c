// Lean compiler output
// Module: IrisKernel.FFI
// Imports: public import Init public import IrisKernel.Types public import IrisKernel.Eval
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
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readUInt8(lean_object*);
lean_object* lean_uint32_to_nat(uint32_t);
LEAN_EXPORT uint8_t iris_check_cost_leq(lean_object*);
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readInt64LE(lean_object*);
uint64_t lean_uint64_lor(uint64_t, uint64_t);
uint32_t lean_uint8_to_uint32(uint8_t);
uint8_t lp_iris_x2dkernel_IrisKernel_evalLIAFormula(lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_Cursor_remaining(lean_object*);
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readUInt16LE(lean_object*);
static lean_object* lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readInt64LE___closed__2;
lean_object* lean_nat_to_int(lean_object*);
uint64_t lean_uint8_to_uint64(uint8_t);
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_evalLIAFFI___boxed(lean_object*);
lean_object* l_Int_pow(lean_object*, lean_object*);
lean_object* lean_uint64_to_nat(uint64_t);
uint16_t lean_uint8_to_uint16(uint8_t);
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_Cursor_remaining___boxed(lean_object*);
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_typeCheckNodeFFI___boxed(lean_object*, lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_decodeLIAEnv_decodeLIAEnvEntries(lean_object*, lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_decodeCostBound_decodeCostBoundList(lean_object*, lean_object*);
lean_object* lean_int_sub(lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readUInt64LE(lean_object*);
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_Cursor_create(lean_object*);
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_decodeLIATerm(lean_object*);
uint8_t lp_iris_x2dkernel_IrisKernel_checkCostLeq(lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_decodeCostBound(lean_object*);
LEAN_EXPORT uint8_t iris_eval_lia(lean_object*);
uint16_t lean_uint16_lor(uint16_t, uint16_t);
uint8_t lean_nat_dec_eq(lean_object*, lean_object*);
uint8_t lean_nat_dec_lt(lean_object*, lean_object*);
static lean_object* lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readInt64LE___closed__0;
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_typeCheckNodeFFI___redArg___boxed(lean_object*);
lean_object* lean_uint16_to_nat(uint16_t);
uint32_t lean_uint32_lor(uint32_t, uint32_t);
uint32_t lean_uint32_shift_left(uint32_t, uint32_t);
lean_object* l_List_reverse___redArg(lean_object*);
lean_object* lean_nat_sub(lean_object*, lean_object*);
uint64_t lean_uint64_shift_left(uint64_t, uint64_t);
uint8_t lean_byte_array_get(lean_object*, lean_object*);
lean_object* lean_uint8_to_nat(uint8_t);
uint8_t lean_nat_dec_le(lean_object*, lean_object*);
static lean_object* lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readInt64LE___closed__1;
LEAN_EXPORT uint32_t iris_lean_kernel_version;
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_decodeLIAEnv(lean_object*);
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_decodeCostBound_decodeCostBoundList___boxed(lean_object*, lean_object*);
lean_object* lean_nat_add(lean_object*, lean_object*);
uint16_t lean_uint16_shift_left(uint16_t, uint16_t);
LEAN_EXPORT lean_object* lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readUInt32LE(lean_object*);
lean_object* lean_byte_array_size(lean_object*);
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
lean_object* x_33; 
lean_dec(x_8);
x_33 = lean_box(0);
return x_33;
}
else
{
lean_object* x_34; 
x_34 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readUInt16LE(x_8);
if (lean_obj_tag(x_34) == 0)
{
lean_object* x_35; 
x_35 = lean_box(0);
return x_35;
}
else
{
uint8_t x_36; 
x_36 = !lean_is_exclusive(x_34);
if (x_36 == 0)
{
lean_object* x_37; lean_object* x_38; lean_object* x_39; uint16_t x_40; lean_object* x_41; lean_object* x_42; 
x_37 = lean_ctor_get(x_34, 0);
x_38 = lean_ctor_get(x_37, 0);
lean_inc(x_38);
x_39 = lean_ctor_get(x_37, 1);
lean_inc(x_39);
lean_dec(x_37);
x_40 = lean_unbox(x_38);
lean_dec(x_38);
x_41 = lean_uint16_to_nat(x_40);
x_42 = lp_iris_x2dkernel_IrisKernel_FFI_decodeCostBound_decodeCostBoundList(x_39, x_41);
if (lean_obj_tag(x_42) == 0)
{
lean_object* x_43; 
lean_free_object(x_34);
x_43 = lean_box(0);
return x_43;
}
else
{
uint8_t x_44; 
x_44 = !lean_is_exclusive(x_42);
if (x_44 == 0)
{
lean_object* x_45; uint8_t x_46; 
x_45 = lean_ctor_get(x_42, 0);
x_46 = !lean_is_exclusive(x_45);
if (x_46 == 0)
{
lean_object* x_47; 
x_47 = lean_ctor_get(x_45, 0);
lean_ctor_set_tag(x_34, 10);
lean_ctor_set(x_34, 0, x_47);
lean_ctor_set(x_45, 0, x_34);
return x_42;
}
else
{
lean_object* x_48; lean_object* x_49; lean_object* x_50; 
x_48 = lean_ctor_get(x_45, 0);
x_49 = lean_ctor_get(x_45, 1);
lean_inc(x_49);
lean_inc(x_48);
lean_dec(x_45);
lean_ctor_set_tag(x_34, 10);
lean_ctor_set(x_34, 0, x_48);
x_50 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_50, 0, x_34);
lean_ctor_set(x_50, 1, x_49);
lean_ctor_set(x_42, 0, x_50);
return x_42;
}
}
else
{
lean_object* x_51; lean_object* x_52; lean_object* x_53; lean_object* x_54; lean_object* x_55; lean_object* x_56; 
x_51 = lean_ctor_get(x_42, 0);
lean_inc(x_51);
lean_dec(x_42);
x_52 = lean_ctor_get(x_51, 0);
lean_inc(x_52);
x_53 = lean_ctor_get(x_51, 1);
lean_inc(x_53);
if (lean_is_exclusive(x_51)) {
 lean_ctor_release(x_51, 0);
 lean_ctor_release(x_51, 1);
 x_54 = x_51;
} else {
 lean_dec_ref(x_51);
 x_54 = lean_box(0);
}
lean_ctor_set_tag(x_34, 10);
lean_ctor_set(x_34, 0, x_52);
if (lean_is_scalar(x_54)) {
 x_55 = lean_alloc_ctor(0, 2, 0);
} else {
 x_55 = x_54;
}
lean_ctor_set(x_55, 0, x_34);
lean_ctor_set(x_55, 1, x_53);
x_56 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_56, 0, x_55);
return x_56;
}
}
}
else
{
lean_object* x_57; lean_object* x_58; lean_object* x_59; uint16_t x_60; lean_object* x_61; lean_object* x_62; 
x_57 = lean_ctor_get(x_34, 0);
lean_inc(x_57);
lean_dec(x_34);
x_58 = lean_ctor_get(x_57, 0);
lean_inc(x_58);
x_59 = lean_ctor_get(x_57, 1);
lean_inc(x_59);
lean_dec(x_57);
x_60 = lean_unbox(x_58);
lean_dec(x_58);
x_61 = lean_uint16_to_nat(x_60);
x_62 = lp_iris_x2dkernel_IrisKernel_FFI_decodeCostBound_decodeCostBoundList(x_59, x_61);
if (lean_obj_tag(x_62) == 0)
{
lean_object* x_63; 
x_63 = lean_box(0);
return x_63;
}
else
{
lean_object* x_64; lean_object* x_65; lean_object* x_66; lean_object* x_67; lean_object* x_68; lean_object* x_69; lean_object* x_70; lean_object* x_71; 
x_64 = lean_ctor_get(x_62, 0);
lean_inc(x_64);
if (lean_is_exclusive(x_62)) {
 lean_ctor_release(x_62, 0);
 x_65 = x_62;
} else {
 lean_dec_ref(x_62);
 x_65 = lean_box(0);
}
x_66 = lean_ctor_get(x_64, 0);
lean_inc(x_66);
x_67 = lean_ctor_get(x_64, 1);
lean_inc(x_67);
if (lean_is_exclusive(x_64)) {
 lean_ctor_release(x_64, 0);
 lean_ctor_release(x_64, 1);
 x_68 = x_64;
} else {
 lean_dec_ref(x_64);
 x_68 = lean_box(0);
}
x_69 = lean_alloc_ctor(10, 1, 0);
lean_ctor_set(x_69, 0, x_66);
if (lean_is_scalar(x_68)) {
 x_70 = lean_alloc_ctor(0, 2, 0);
} else {
 x_70 = x_68;
}
lean_ctor_set(x_70, 0, x_69);
lean_ctor_set(x_70, 1, x_67);
if (lean_is_scalar(x_65)) {
 x_71 = lean_alloc_ctor(1, 1, 0);
} else {
 x_71 = x_65;
}
lean_ctor_set(x_71, 0, x_70);
return x_71;
}
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
lean_ctor_set_tag(x_72, 9);
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
lean_ctor_set_tag(x_72, 9);
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
lean_ctor_set_tag(x_72, 9);
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
x_107 = lean_alloc_ctor(9, 1, 0);
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
x_110 = lp_iris_x2dkernel_IrisKernel_FFI_decodeCostBound(x_8);
if (lean_obj_tag(x_110) == 0)
{
return x_110;
}
else
{
lean_object* x_111; uint8_t x_112; 
x_111 = lean_ctor_get(x_110, 0);
lean_inc(x_111);
lean_dec_ref(x_110);
x_112 = !lean_is_exclusive(x_111);
if (x_112 == 0)
{
lean_object* x_113; lean_object* x_114; lean_object* x_115; 
x_113 = lean_ctor_get(x_111, 0);
x_114 = lean_ctor_get(x_111, 1);
x_115 = lp_iris_x2dkernel_IrisKernel_FFI_decodeCostBound(x_114);
if (lean_obj_tag(x_115) == 0)
{
lean_free_object(x_111);
lean_dec(x_113);
return x_115;
}
else
{
uint8_t x_116; 
x_116 = !lean_is_exclusive(x_115);
if (x_116 == 0)
{
lean_object* x_117; uint8_t x_118; 
x_117 = lean_ctor_get(x_115, 0);
x_118 = !lean_is_exclusive(x_117);
if (x_118 == 0)
{
lean_object* x_119; 
x_119 = lean_ctor_get(x_117, 0);
lean_ctor_set_tag(x_111, 8);
lean_ctor_set(x_111, 1, x_119);
lean_ctor_set(x_117, 0, x_111);
return x_115;
}
else
{
lean_object* x_120; lean_object* x_121; lean_object* x_122; 
x_120 = lean_ctor_get(x_117, 0);
x_121 = lean_ctor_get(x_117, 1);
lean_inc(x_121);
lean_inc(x_120);
lean_dec(x_117);
lean_ctor_set_tag(x_111, 8);
lean_ctor_set(x_111, 1, x_120);
x_122 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_122, 0, x_111);
lean_ctor_set(x_122, 1, x_121);
lean_ctor_set(x_115, 0, x_122);
return x_115;
}
}
else
{
lean_object* x_123; lean_object* x_124; lean_object* x_125; lean_object* x_126; lean_object* x_127; lean_object* x_128; 
x_123 = lean_ctor_get(x_115, 0);
lean_inc(x_123);
lean_dec(x_115);
x_124 = lean_ctor_get(x_123, 0);
lean_inc(x_124);
x_125 = lean_ctor_get(x_123, 1);
lean_inc(x_125);
if (lean_is_exclusive(x_123)) {
 lean_ctor_release(x_123, 0);
 lean_ctor_release(x_123, 1);
 x_126 = x_123;
} else {
 lean_dec_ref(x_123);
 x_126 = lean_box(0);
}
lean_ctor_set_tag(x_111, 8);
lean_ctor_set(x_111, 1, x_124);
if (lean_is_scalar(x_126)) {
 x_127 = lean_alloc_ctor(0, 2, 0);
} else {
 x_127 = x_126;
}
lean_ctor_set(x_127, 0, x_111);
lean_ctor_set(x_127, 1, x_125);
x_128 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_128, 0, x_127);
return x_128;
}
}
}
else
{
lean_object* x_129; lean_object* x_130; lean_object* x_131; 
x_129 = lean_ctor_get(x_111, 0);
x_130 = lean_ctor_get(x_111, 1);
lean_inc(x_130);
lean_inc(x_129);
lean_dec(x_111);
x_131 = lp_iris_x2dkernel_IrisKernel_FFI_decodeCostBound(x_130);
if (lean_obj_tag(x_131) == 0)
{
lean_dec(x_129);
return x_131;
}
else
{
lean_object* x_132; lean_object* x_133; lean_object* x_134; lean_object* x_135; lean_object* x_136; lean_object* x_137; lean_object* x_138; lean_object* x_139; 
x_132 = lean_ctor_get(x_131, 0);
lean_inc(x_132);
if (lean_is_exclusive(x_131)) {
 lean_ctor_release(x_131, 0);
 x_133 = x_131;
} else {
 lean_dec_ref(x_131);
 x_133 = lean_box(0);
}
x_134 = lean_ctor_get(x_132, 0);
lean_inc(x_134);
x_135 = lean_ctor_get(x_132, 1);
lean_inc(x_135);
if (lean_is_exclusive(x_132)) {
 lean_ctor_release(x_132, 0);
 lean_ctor_release(x_132, 1);
 x_136 = x_132;
} else {
 lean_dec_ref(x_132);
 x_136 = lean_box(0);
}
x_137 = lean_alloc_ctor(8, 2, 0);
lean_ctor_set(x_137, 0, x_129);
lean_ctor_set(x_137, 1, x_134);
if (lean_is_scalar(x_136)) {
 x_138 = lean_alloc_ctor(0, 2, 0);
} else {
 x_138 = x_136;
}
lean_ctor_set(x_138, 0, x_137);
lean_ctor_set(x_138, 1, x_135);
if (lean_is_scalar(x_133)) {
 x_139 = lean_alloc_ctor(1, 1, 0);
} else {
 x_139 = x_133;
}
lean_ctor_set(x_139, 0, x_138);
return x_139;
}
}
}
}
}
else
{
lean_object* x_140; 
x_140 = lp_iris_x2dkernel_IrisKernel_FFI_decodeCostBound(x_8);
if (lean_obj_tag(x_140) == 0)
{
return x_140;
}
else
{
lean_object* x_141; uint8_t x_142; 
x_141 = lean_ctor_get(x_140, 0);
lean_inc(x_141);
lean_dec_ref(x_140);
x_142 = !lean_is_exclusive(x_141);
if (x_142 == 0)
{
lean_object* x_143; lean_object* x_144; lean_object* x_145; 
x_143 = lean_ctor_get(x_141, 0);
x_144 = lean_ctor_get(x_141, 1);
x_145 = lp_iris_x2dkernel_IrisKernel_FFI_decodeCostBound(x_144);
if (lean_obj_tag(x_145) == 0)
{
lean_free_object(x_141);
lean_dec(x_143);
return x_145;
}
else
{
uint8_t x_146; 
x_146 = !lean_is_exclusive(x_145);
if (x_146 == 0)
{
lean_object* x_147; uint8_t x_148; 
x_147 = lean_ctor_get(x_145, 0);
x_148 = !lean_is_exclusive(x_147);
if (x_148 == 0)
{
lean_object* x_149; 
x_149 = lean_ctor_get(x_147, 0);
lean_ctor_set_tag(x_141, 7);
lean_ctor_set(x_141, 1, x_149);
lean_ctor_set(x_147, 0, x_141);
return x_145;
}
else
{
lean_object* x_150; lean_object* x_151; lean_object* x_152; 
x_150 = lean_ctor_get(x_147, 0);
x_151 = lean_ctor_get(x_147, 1);
lean_inc(x_151);
lean_inc(x_150);
lean_dec(x_147);
lean_ctor_set_tag(x_141, 7);
lean_ctor_set(x_141, 1, x_150);
x_152 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_152, 0, x_141);
lean_ctor_set(x_152, 1, x_151);
lean_ctor_set(x_145, 0, x_152);
return x_145;
}
}
else
{
lean_object* x_153; lean_object* x_154; lean_object* x_155; lean_object* x_156; lean_object* x_157; lean_object* x_158; 
x_153 = lean_ctor_get(x_145, 0);
lean_inc(x_153);
lean_dec(x_145);
x_154 = lean_ctor_get(x_153, 0);
lean_inc(x_154);
x_155 = lean_ctor_get(x_153, 1);
lean_inc(x_155);
if (lean_is_exclusive(x_153)) {
 lean_ctor_release(x_153, 0);
 lean_ctor_release(x_153, 1);
 x_156 = x_153;
} else {
 lean_dec_ref(x_153);
 x_156 = lean_box(0);
}
lean_ctor_set_tag(x_141, 7);
lean_ctor_set(x_141, 1, x_154);
if (lean_is_scalar(x_156)) {
 x_157 = lean_alloc_ctor(0, 2, 0);
} else {
 x_157 = x_156;
}
lean_ctor_set(x_157, 0, x_141);
lean_ctor_set(x_157, 1, x_155);
x_158 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_158, 0, x_157);
return x_158;
}
}
}
else
{
lean_object* x_159; lean_object* x_160; lean_object* x_161; 
x_159 = lean_ctor_get(x_141, 0);
x_160 = lean_ctor_get(x_141, 1);
lean_inc(x_160);
lean_inc(x_159);
lean_dec(x_141);
x_161 = lp_iris_x2dkernel_IrisKernel_FFI_decodeCostBound(x_160);
if (lean_obj_tag(x_161) == 0)
{
lean_dec(x_159);
return x_161;
}
else
{
lean_object* x_162; lean_object* x_163; lean_object* x_164; lean_object* x_165; lean_object* x_166; lean_object* x_167; lean_object* x_168; lean_object* x_169; 
x_162 = lean_ctor_get(x_161, 0);
lean_inc(x_162);
if (lean_is_exclusive(x_161)) {
 lean_ctor_release(x_161, 0);
 x_163 = x_161;
} else {
 lean_dec_ref(x_161);
 x_163 = lean_box(0);
}
x_164 = lean_ctor_get(x_162, 0);
lean_inc(x_164);
x_165 = lean_ctor_get(x_162, 1);
lean_inc(x_165);
if (lean_is_exclusive(x_162)) {
 lean_ctor_release(x_162, 0);
 lean_ctor_release(x_162, 1);
 x_166 = x_162;
} else {
 lean_dec_ref(x_162);
 x_166 = lean_box(0);
}
x_167 = lean_alloc_ctor(7, 2, 0);
lean_ctor_set(x_167, 0, x_159);
lean_ctor_set(x_167, 1, x_164);
if (lean_is_scalar(x_166)) {
 x_168 = lean_alloc_ctor(0, 2, 0);
} else {
 x_168 = x_166;
}
lean_ctor_set(x_168, 0, x_167);
lean_ctor_set(x_168, 1, x_165);
if (lean_is_scalar(x_163)) {
 x_169 = lean_alloc_ctor(1, 1, 0);
} else {
 x_169 = x_163;
}
lean_ctor_set(x_169, 0, x_168);
return x_169;
}
}
}
}
}
else
{
lean_object* x_170; 
x_170 = lp_iris_x2dkernel_IrisKernel_FFI_decodeCostBound(x_8);
if (lean_obj_tag(x_170) == 0)
{
return x_170;
}
else
{
lean_object* x_171; uint8_t x_172; 
x_171 = lean_ctor_get(x_170, 0);
lean_inc(x_171);
lean_dec_ref(x_170);
x_172 = !lean_is_exclusive(x_171);
if (x_172 == 0)
{
lean_object* x_173; lean_object* x_174; lean_object* x_175; 
x_173 = lean_ctor_get(x_171, 0);
x_174 = lean_ctor_get(x_171, 1);
x_175 = lp_iris_x2dkernel_IrisKernel_FFI_decodeCostBound(x_174);
if (lean_obj_tag(x_175) == 0)
{
lean_free_object(x_171);
lean_dec(x_173);
return x_175;
}
else
{
uint8_t x_176; 
x_176 = !lean_is_exclusive(x_175);
if (x_176 == 0)
{
lean_object* x_177; uint8_t x_178; 
x_177 = lean_ctor_get(x_175, 0);
x_178 = !lean_is_exclusive(x_177);
if (x_178 == 0)
{
lean_object* x_179; 
x_179 = lean_ctor_get(x_177, 0);
lean_ctor_set_tag(x_171, 6);
lean_ctor_set(x_171, 1, x_179);
lean_ctor_set(x_177, 0, x_171);
return x_175;
}
else
{
lean_object* x_180; lean_object* x_181; lean_object* x_182; 
x_180 = lean_ctor_get(x_177, 0);
x_181 = lean_ctor_get(x_177, 1);
lean_inc(x_181);
lean_inc(x_180);
lean_dec(x_177);
lean_ctor_set_tag(x_171, 6);
lean_ctor_set(x_171, 1, x_180);
x_182 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_182, 0, x_171);
lean_ctor_set(x_182, 1, x_181);
lean_ctor_set(x_175, 0, x_182);
return x_175;
}
}
else
{
lean_object* x_183; lean_object* x_184; lean_object* x_185; lean_object* x_186; lean_object* x_187; lean_object* x_188; 
x_183 = lean_ctor_get(x_175, 0);
lean_inc(x_183);
lean_dec(x_175);
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
lean_ctor_set_tag(x_171, 6);
lean_ctor_set(x_171, 1, x_184);
if (lean_is_scalar(x_186)) {
 x_187 = lean_alloc_ctor(0, 2, 0);
} else {
 x_187 = x_186;
}
lean_ctor_set(x_187, 0, x_171);
lean_ctor_set(x_187, 1, x_185);
x_188 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_188, 0, x_187);
return x_188;
}
}
}
else
{
lean_object* x_189; lean_object* x_190; lean_object* x_191; 
x_189 = lean_ctor_get(x_171, 0);
x_190 = lean_ctor_get(x_171, 1);
lean_inc(x_190);
lean_inc(x_189);
lean_dec(x_171);
x_191 = lp_iris_x2dkernel_IrisKernel_FFI_decodeCostBound(x_190);
if (lean_obj_tag(x_191) == 0)
{
lean_dec(x_189);
return x_191;
}
else
{
lean_object* x_192; lean_object* x_193; lean_object* x_194; lean_object* x_195; lean_object* x_196; lean_object* x_197; lean_object* x_198; lean_object* x_199; 
x_192 = lean_ctor_get(x_191, 0);
lean_inc(x_192);
if (lean_is_exclusive(x_191)) {
 lean_ctor_release(x_191, 0);
 x_193 = x_191;
} else {
 lean_dec_ref(x_191);
 x_193 = lean_box(0);
}
x_194 = lean_ctor_get(x_192, 0);
lean_inc(x_194);
x_195 = lean_ctor_get(x_192, 1);
lean_inc(x_195);
if (lean_is_exclusive(x_192)) {
 lean_ctor_release(x_192, 0);
 lean_ctor_release(x_192, 1);
 x_196 = x_192;
} else {
 lean_dec_ref(x_192);
 x_196 = lean_box(0);
}
x_197 = lean_alloc_ctor(6, 2, 0);
lean_ctor_set(x_197, 0, x_189);
lean_ctor_set(x_197, 1, x_194);
if (lean_is_scalar(x_196)) {
 x_198 = lean_alloc_ctor(0, 2, 0);
} else {
 x_198 = x_196;
}
lean_ctor_set(x_198, 0, x_197);
lean_ctor_set(x_198, 1, x_195);
if (lean_is_scalar(x_193)) {
 x_199 = lean_alloc_ctor(1, 1, 0);
} else {
 x_199 = x_193;
}
lean_ctor_set(x_199, 0, x_198);
return x_199;
}
}
}
}
}
else
{
lean_object* x_200; 
x_200 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readUInt32LE(x_8);
if (lean_obj_tag(x_200) == 0)
{
lean_object* x_201; 
x_201 = lean_box(0);
return x_201;
}
else
{
lean_object* x_202; uint8_t x_203; 
x_202 = lean_ctor_get(x_200, 0);
lean_inc(x_202);
lean_dec_ref(x_200);
x_203 = !lean_is_exclusive(x_202);
if (x_203 == 0)
{
lean_object* x_204; lean_object* x_205; lean_object* x_206; 
x_204 = lean_ctor_get(x_202, 0);
x_205 = lean_ctor_get(x_202, 1);
x_206 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readUInt32LE(x_205);
if (lean_obj_tag(x_206) == 0)
{
lean_object* x_207; 
lean_free_object(x_202);
lean_dec(x_204);
x_207 = lean_box(0);
return x_207;
}
else
{
uint8_t x_208; 
x_208 = !lean_is_exclusive(x_206);
if (x_208 == 0)
{
lean_object* x_209; uint8_t x_210; 
x_209 = lean_ctor_get(x_206, 0);
x_210 = !lean_is_exclusive(x_209);
if (x_210 == 0)
{
lean_object* x_211; uint32_t x_212; lean_object* x_213; uint32_t x_214; lean_object* x_215; 
x_211 = lean_ctor_get(x_209, 0);
x_212 = lean_unbox_uint32(x_204);
lean_dec(x_204);
x_213 = lean_uint32_to_nat(x_212);
x_214 = lean_unbox_uint32(x_211);
lean_dec(x_211);
x_215 = lean_uint32_to_nat(x_214);
lean_ctor_set_tag(x_202, 5);
lean_ctor_set(x_202, 1, x_215);
lean_ctor_set(x_202, 0, x_213);
lean_ctor_set(x_209, 0, x_202);
return x_206;
}
else
{
lean_object* x_216; lean_object* x_217; uint32_t x_218; lean_object* x_219; uint32_t x_220; lean_object* x_221; lean_object* x_222; 
x_216 = lean_ctor_get(x_209, 0);
x_217 = lean_ctor_get(x_209, 1);
lean_inc(x_217);
lean_inc(x_216);
lean_dec(x_209);
x_218 = lean_unbox_uint32(x_204);
lean_dec(x_204);
x_219 = lean_uint32_to_nat(x_218);
x_220 = lean_unbox_uint32(x_216);
lean_dec(x_216);
x_221 = lean_uint32_to_nat(x_220);
lean_ctor_set_tag(x_202, 5);
lean_ctor_set(x_202, 1, x_221);
lean_ctor_set(x_202, 0, x_219);
x_222 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_222, 0, x_202);
lean_ctor_set(x_222, 1, x_217);
lean_ctor_set(x_206, 0, x_222);
return x_206;
}
}
else
{
lean_object* x_223; lean_object* x_224; lean_object* x_225; lean_object* x_226; uint32_t x_227; lean_object* x_228; uint32_t x_229; lean_object* x_230; lean_object* x_231; lean_object* x_232; 
x_223 = lean_ctor_get(x_206, 0);
lean_inc(x_223);
lean_dec(x_206);
x_224 = lean_ctor_get(x_223, 0);
lean_inc(x_224);
x_225 = lean_ctor_get(x_223, 1);
lean_inc(x_225);
if (lean_is_exclusive(x_223)) {
 lean_ctor_release(x_223, 0);
 lean_ctor_release(x_223, 1);
 x_226 = x_223;
} else {
 lean_dec_ref(x_223);
 x_226 = lean_box(0);
}
x_227 = lean_unbox_uint32(x_204);
lean_dec(x_204);
x_228 = lean_uint32_to_nat(x_227);
x_229 = lean_unbox_uint32(x_224);
lean_dec(x_224);
x_230 = lean_uint32_to_nat(x_229);
lean_ctor_set_tag(x_202, 5);
lean_ctor_set(x_202, 1, x_230);
lean_ctor_set(x_202, 0, x_228);
if (lean_is_scalar(x_226)) {
 x_231 = lean_alloc_ctor(0, 2, 0);
} else {
 x_231 = x_226;
}
lean_ctor_set(x_231, 0, x_202);
lean_ctor_set(x_231, 1, x_225);
x_232 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_232, 0, x_231);
return x_232;
}
}
}
else
{
lean_object* x_233; lean_object* x_234; lean_object* x_235; 
x_233 = lean_ctor_get(x_202, 0);
x_234 = lean_ctor_get(x_202, 1);
lean_inc(x_234);
lean_inc(x_233);
lean_dec(x_202);
x_235 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readUInt32LE(x_234);
if (lean_obj_tag(x_235) == 0)
{
lean_object* x_236; 
lean_dec(x_233);
x_236 = lean_box(0);
return x_236;
}
else
{
lean_object* x_237; lean_object* x_238; lean_object* x_239; lean_object* x_240; lean_object* x_241; uint32_t x_242; lean_object* x_243; uint32_t x_244; lean_object* x_245; lean_object* x_246; lean_object* x_247; lean_object* x_248; 
x_237 = lean_ctor_get(x_235, 0);
lean_inc(x_237);
if (lean_is_exclusive(x_235)) {
 lean_ctor_release(x_235, 0);
 x_238 = x_235;
} else {
 lean_dec_ref(x_235);
 x_238 = lean_box(0);
}
x_239 = lean_ctor_get(x_237, 0);
lean_inc(x_239);
x_240 = lean_ctor_get(x_237, 1);
lean_inc(x_240);
if (lean_is_exclusive(x_237)) {
 lean_ctor_release(x_237, 0);
 lean_ctor_release(x_237, 1);
 x_241 = x_237;
} else {
 lean_dec_ref(x_237);
 x_241 = lean_box(0);
}
x_242 = lean_unbox_uint32(x_233);
lean_dec(x_233);
x_243 = lean_uint32_to_nat(x_242);
x_244 = lean_unbox_uint32(x_239);
lean_dec(x_239);
x_245 = lean_uint32_to_nat(x_244);
x_246 = lean_alloc_ctor(5, 2, 0);
lean_ctor_set(x_246, 0, x_243);
lean_ctor_set(x_246, 1, x_245);
if (lean_is_scalar(x_241)) {
 x_247 = lean_alloc_ctor(0, 2, 0);
} else {
 x_247 = x_241;
}
lean_ctor_set(x_247, 0, x_246);
lean_ctor_set(x_247, 1, x_240);
if (lean_is_scalar(x_238)) {
 x_248 = lean_alloc_ctor(1, 1, 0);
} else {
 x_248 = x_238;
}
lean_ctor_set(x_248, 0, x_247);
return x_248;
}
}
}
}
}
else
{
lean_object* x_249; 
x_249 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readUInt32LE(x_8);
if (lean_obj_tag(x_249) == 0)
{
lean_object* x_250; 
x_250 = lean_box(0);
return x_250;
}
else
{
uint8_t x_251; 
x_251 = !lean_is_exclusive(x_249);
if (x_251 == 0)
{
lean_object* x_252; uint8_t x_253; 
x_252 = lean_ctor_get(x_249, 0);
x_253 = !lean_is_exclusive(x_252);
if (x_253 == 0)
{
lean_object* x_254; uint32_t x_255; lean_object* x_256; lean_object* x_257; 
x_254 = lean_ctor_get(x_252, 0);
x_255 = lean_unbox_uint32(x_254);
lean_dec(x_254);
x_256 = lean_uint32_to_nat(x_255);
x_257 = lean_alloc_ctor(4, 1, 0);
lean_ctor_set(x_257, 0, x_256);
lean_ctor_set(x_252, 0, x_257);
return x_249;
}
else
{
lean_object* x_258; lean_object* x_259; uint32_t x_260; lean_object* x_261; lean_object* x_262; lean_object* x_263; 
x_258 = lean_ctor_get(x_252, 0);
x_259 = lean_ctor_get(x_252, 1);
lean_inc(x_259);
lean_inc(x_258);
lean_dec(x_252);
x_260 = lean_unbox_uint32(x_258);
lean_dec(x_258);
x_261 = lean_uint32_to_nat(x_260);
x_262 = lean_alloc_ctor(4, 1, 0);
lean_ctor_set(x_262, 0, x_261);
x_263 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_263, 0, x_262);
lean_ctor_set(x_263, 1, x_259);
lean_ctor_set(x_249, 0, x_263);
return x_249;
}
}
else
{
lean_object* x_264; lean_object* x_265; lean_object* x_266; lean_object* x_267; uint32_t x_268; lean_object* x_269; lean_object* x_270; lean_object* x_271; lean_object* x_272; 
x_264 = lean_ctor_get(x_249, 0);
lean_inc(x_264);
lean_dec(x_249);
x_265 = lean_ctor_get(x_264, 0);
lean_inc(x_265);
x_266 = lean_ctor_get(x_264, 1);
lean_inc(x_266);
if (lean_is_exclusive(x_264)) {
 lean_ctor_release(x_264, 0);
 lean_ctor_release(x_264, 1);
 x_267 = x_264;
} else {
 lean_dec_ref(x_264);
 x_267 = lean_box(0);
}
x_268 = lean_unbox_uint32(x_265);
lean_dec(x_265);
x_269 = lean_uint32_to_nat(x_268);
x_270 = lean_alloc_ctor(4, 1, 0);
lean_ctor_set(x_270, 0, x_269);
if (lean_is_scalar(x_267)) {
 x_271 = lean_alloc_ctor(0, 2, 0);
} else {
 x_271 = x_267;
}
lean_ctor_set(x_271, 0, x_270);
lean_ctor_set(x_271, 1, x_266);
x_272 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_272, 0, x_271);
return x_272;
}
}
}
}
else
{
lean_object* x_273; 
x_273 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readUInt32LE(x_8);
if (lean_obj_tag(x_273) == 0)
{
lean_object* x_274; 
x_274 = lean_box(0);
return x_274;
}
else
{
uint8_t x_275; 
x_275 = !lean_is_exclusive(x_273);
if (x_275 == 0)
{
lean_object* x_276; uint8_t x_277; 
x_276 = lean_ctor_get(x_273, 0);
x_277 = !lean_is_exclusive(x_276);
if (x_277 == 0)
{
lean_object* x_278; uint32_t x_279; lean_object* x_280; lean_object* x_281; 
x_278 = lean_ctor_get(x_276, 0);
x_279 = lean_unbox_uint32(x_278);
lean_dec(x_278);
x_280 = lean_uint32_to_nat(x_279);
x_281 = lean_alloc_ctor(3, 1, 0);
lean_ctor_set(x_281, 0, x_280);
lean_ctor_set(x_276, 0, x_281);
return x_273;
}
else
{
lean_object* x_282; lean_object* x_283; uint32_t x_284; lean_object* x_285; lean_object* x_286; lean_object* x_287; 
x_282 = lean_ctor_get(x_276, 0);
x_283 = lean_ctor_get(x_276, 1);
lean_inc(x_283);
lean_inc(x_282);
lean_dec(x_276);
x_284 = lean_unbox_uint32(x_282);
lean_dec(x_282);
x_285 = lean_uint32_to_nat(x_284);
x_286 = lean_alloc_ctor(3, 1, 0);
lean_ctor_set(x_286, 0, x_285);
x_287 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_287, 0, x_286);
lean_ctor_set(x_287, 1, x_283);
lean_ctor_set(x_273, 0, x_287);
return x_273;
}
}
else
{
lean_object* x_288; lean_object* x_289; lean_object* x_290; lean_object* x_291; uint32_t x_292; lean_object* x_293; lean_object* x_294; lean_object* x_295; lean_object* x_296; 
x_288 = lean_ctor_get(x_273, 0);
lean_inc(x_288);
lean_dec(x_273);
x_289 = lean_ctor_get(x_288, 0);
lean_inc(x_289);
x_290 = lean_ctor_get(x_288, 1);
lean_inc(x_290);
if (lean_is_exclusive(x_288)) {
 lean_ctor_release(x_288, 0);
 lean_ctor_release(x_288, 1);
 x_291 = x_288;
} else {
 lean_dec_ref(x_288);
 x_291 = lean_box(0);
}
x_292 = lean_unbox_uint32(x_289);
lean_dec(x_289);
x_293 = lean_uint32_to_nat(x_292);
x_294 = lean_alloc_ctor(3, 1, 0);
lean_ctor_set(x_294, 0, x_293);
if (lean_is_scalar(x_291)) {
 x_295 = lean_alloc_ctor(0, 2, 0);
} else {
 x_295 = x_291;
}
lean_ctor_set(x_295, 0, x_294);
lean_ctor_set(x_295, 1, x_290);
x_296 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_296, 0, x_295);
return x_296;
}
}
}
}
else
{
lean_object* x_297; 
x_297 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readUInt64LE(x_8);
if (lean_obj_tag(x_297) == 0)
{
lean_object* x_298; 
x_298 = lean_box(0);
return x_298;
}
else
{
uint8_t x_299; 
x_299 = !lean_is_exclusive(x_297);
if (x_299 == 0)
{
lean_object* x_300; uint8_t x_301; 
x_300 = lean_ctor_get(x_297, 0);
x_301 = !lean_is_exclusive(x_300);
if (x_301 == 0)
{
lean_object* x_302; uint64_t x_303; lean_object* x_304; lean_object* x_305; 
x_302 = lean_ctor_get(x_300, 0);
x_303 = lean_unbox_uint64(x_302);
lean_dec(x_302);
x_304 = lean_uint64_to_nat(x_303);
x_305 = lean_alloc_ctor(2, 1, 0);
lean_ctor_set(x_305, 0, x_304);
lean_ctor_set(x_300, 0, x_305);
return x_297;
}
else
{
lean_object* x_306; lean_object* x_307; uint64_t x_308; lean_object* x_309; lean_object* x_310; lean_object* x_311; 
x_306 = lean_ctor_get(x_300, 0);
x_307 = lean_ctor_get(x_300, 1);
lean_inc(x_307);
lean_inc(x_306);
lean_dec(x_300);
x_308 = lean_unbox_uint64(x_306);
lean_dec(x_306);
x_309 = lean_uint64_to_nat(x_308);
x_310 = lean_alloc_ctor(2, 1, 0);
lean_ctor_set(x_310, 0, x_309);
x_311 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_311, 0, x_310);
lean_ctor_set(x_311, 1, x_307);
lean_ctor_set(x_297, 0, x_311);
return x_297;
}
}
else
{
lean_object* x_312; lean_object* x_313; lean_object* x_314; lean_object* x_315; uint64_t x_316; lean_object* x_317; lean_object* x_318; lean_object* x_319; lean_object* x_320; 
x_312 = lean_ctor_get(x_297, 0);
lean_inc(x_312);
lean_dec(x_297);
x_313 = lean_ctor_get(x_312, 0);
lean_inc(x_313);
x_314 = lean_ctor_get(x_312, 1);
lean_inc(x_314);
if (lean_is_exclusive(x_312)) {
 lean_ctor_release(x_312, 0);
 lean_ctor_release(x_312, 1);
 x_315 = x_312;
} else {
 lean_dec_ref(x_312);
 x_315 = lean_box(0);
}
x_316 = lean_unbox_uint64(x_313);
lean_dec(x_313);
x_317 = lean_uint64_to_nat(x_316);
x_318 = lean_alloc_ctor(2, 1, 0);
lean_ctor_set(x_318, 0, x_317);
if (lean_is_scalar(x_315)) {
 x_319 = lean_alloc_ctor(0, 2, 0);
} else {
 x_319 = x_315;
}
lean_ctor_set(x_319, 0, x_318);
lean_ctor_set(x_319, 1, x_314);
x_320 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_320, 0, x_319);
return x_320;
}
}
}
}
else
{
lean_object* x_321; 
x_321 = lean_box(1);
lean_ctor_set(x_5, 0, x_321);
return x_2;
}
}
else
{
lean_object* x_322; 
x_322 = lean_box(0);
lean_ctor_set(x_5, 0, x_322);
return x_2;
}
}
else
{
lean_object* x_323; lean_object* x_324; uint8_t x_325; lean_object* x_326; lean_object* x_327; uint8_t x_328; 
x_323 = lean_ctor_get(x_5, 0);
x_324 = lean_ctor_get(x_5, 1);
lean_inc(x_324);
lean_inc(x_323);
lean_dec(x_5);
x_325 = lean_unbox(x_323);
lean_dec(x_323);
x_326 = lean_uint8_to_nat(x_325);
x_327 = lean_unsigned_to_nat(0u);
x_328 = lean_nat_dec_eq(x_326, x_327);
if (x_328 == 0)
{
lean_object* x_329; uint8_t x_330; 
x_329 = lean_unsigned_to_nat(1u);
x_330 = lean_nat_dec_eq(x_326, x_329);
if (x_330 == 0)
{
lean_object* x_331; uint8_t x_332; 
lean_free_object(x_2);
x_331 = lean_unsigned_to_nat(2u);
x_332 = lean_nat_dec_eq(x_326, x_331);
if (x_332 == 0)
{
lean_object* x_333; uint8_t x_334; 
x_333 = lean_unsigned_to_nat(3u);
x_334 = lean_nat_dec_eq(x_326, x_333);
if (x_334 == 0)
{
lean_object* x_335; uint8_t x_336; 
x_335 = lean_unsigned_to_nat(4u);
x_336 = lean_nat_dec_eq(x_326, x_335);
if (x_336 == 0)
{
lean_object* x_337; uint8_t x_338; 
x_337 = lean_unsigned_to_nat(5u);
x_338 = lean_nat_dec_eq(x_326, x_337);
if (x_338 == 0)
{
lean_object* x_339; uint8_t x_340; 
x_339 = lean_unsigned_to_nat(6u);
x_340 = lean_nat_dec_eq(x_326, x_339);
if (x_340 == 0)
{
lean_object* x_341; uint8_t x_342; 
x_341 = lean_unsigned_to_nat(7u);
x_342 = lean_nat_dec_eq(x_326, x_341);
if (x_342 == 0)
{
lean_object* x_343; uint8_t x_344; 
x_343 = lean_unsigned_to_nat(8u);
x_344 = lean_nat_dec_eq(x_326, x_343);
if (x_344 == 0)
{
lean_object* x_345; uint8_t x_346; 
x_345 = lean_unsigned_to_nat(9u);
x_346 = lean_nat_dec_eq(x_326, x_345);
if (x_346 == 0)
{
lean_object* x_347; uint8_t x_348; 
x_347 = lean_unsigned_to_nat(10u);
x_348 = lean_nat_dec_eq(x_326, x_347);
if (x_348 == 0)
{
lean_object* x_349; 
lean_dec(x_324);
x_349 = lean_box(0);
return x_349;
}
else
{
lean_object* x_350; 
x_350 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readUInt16LE(x_324);
if (lean_obj_tag(x_350) == 0)
{
lean_object* x_351; 
x_351 = lean_box(0);
return x_351;
}
else
{
lean_object* x_352; lean_object* x_353; lean_object* x_354; lean_object* x_355; uint16_t x_356; lean_object* x_357; lean_object* x_358; 
x_352 = lean_ctor_get(x_350, 0);
lean_inc(x_352);
if (lean_is_exclusive(x_350)) {
 lean_ctor_release(x_350, 0);
 x_353 = x_350;
} else {
 lean_dec_ref(x_350);
 x_353 = lean_box(0);
}
x_354 = lean_ctor_get(x_352, 0);
lean_inc(x_354);
x_355 = lean_ctor_get(x_352, 1);
lean_inc(x_355);
lean_dec(x_352);
x_356 = lean_unbox(x_354);
lean_dec(x_354);
x_357 = lean_uint16_to_nat(x_356);
x_358 = lp_iris_x2dkernel_IrisKernel_FFI_decodeCostBound_decodeCostBoundList(x_355, x_357);
if (lean_obj_tag(x_358) == 0)
{
lean_object* x_359; 
lean_dec(x_353);
x_359 = lean_box(0);
return x_359;
}
else
{
lean_object* x_360; lean_object* x_361; lean_object* x_362; lean_object* x_363; lean_object* x_364; lean_object* x_365; lean_object* x_366; lean_object* x_367; 
x_360 = lean_ctor_get(x_358, 0);
lean_inc(x_360);
if (lean_is_exclusive(x_358)) {
 lean_ctor_release(x_358, 0);
 x_361 = x_358;
} else {
 lean_dec_ref(x_358);
 x_361 = lean_box(0);
}
x_362 = lean_ctor_get(x_360, 0);
lean_inc(x_362);
x_363 = lean_ctor_get(x_360, 1);
lean_inc(x_363);
if (lean_is_exclusive(x_360)) {
 lean_ctor_release(x_360, 0);
 lean_ctor_release(x_360, 1);
 x_364 = x_360;
} else {
 lean_dec_ref(x_360);
 x_364 = lean_box(0);
}
if (lean_is_scalar(x_353)) {
 x_365 = lean_alloc_ctor(10, 1, 0);
} else {
 x_365 = x_353;
 lean_ctor_set_tag(x_365, 10);
}
lean_ctor_set(x_365, 0, x_362);
if (lean_is_scalar(x_364)) {
 x_366 = lean_alloc_ctor(0, 2, 0);
} else {
 x_366 = x_364;
}
lean_ctor_set(x_366, 0, x_365);
lean_ctor_set(x_366, 1, x_363);
if (lean_is_scalar(x_361)) {
 x_367 = lean_alloc_ctor(1, 1, 0);
} else {
 x_367 = x_361;
}
lean_ctor_set(x_367, 0, x_366);
return x_367;
}
}
}
}
else
{
lean_object* x_368; 
x_368 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readUInt16LE(x_324);
if (lean_obj_tag(x_368) == 0)
{
lean_object* x_369; 
x_369 = lean_box(0);
return x_369;
}
else
{
lean_object* x_370; lean_object* x_371; lean_object* x_372; lean_object* x_373; uint16_t x_374; lean_object* x_375; lean_object* x_376; 
x_370 = lean_ctor_get(x_368, 0);
lean_inc(x_370);
if (lean_is_exclusive(x_368)) {
 lean_ctor_release(x_368, 0);
 x_371 = x_368;
} else {
 lean_dec_ref(x_368);
 x_371 = lean_box(0);
}
x_372 = lean_ctor_get(x_370, 0);
lean_inc(x_372);
x_373 = lean_ctor_get(x_370, 1);
lean_inc(x_373);
lean_dec(x_370);
x_374 = lean_unbox(x_372);
lean_dec(x_372);
x_375 = lean_uint16_to_nat(x_374);
x_376 = lp_iris_x2dkernel_IrisKernel_FFI_decodeCostBound_decodeCostBoundList(x_373, x_375);
if (lean_obj_tag(x_376) == 0)
{
lean_object* x_377; 
lean_dec(x_371);
x_377 = lean_box(0);
return x_377;
}
else
{
lean_object* x_378; lean_object* x_379; lean_object* x_380; lean_object* x_381; lean_object* x_382; lean_object* x_383; lean_object* x_384; lean_object* x_385; 
x_378 = lean_ctor_get(x_376, 0);
lean_inc(x_378);
if (lean_is_exclusive(x_376)) {
 lean_ctor_release(x_376, 0);
 x_379 = x_376;
} else {
 lean_dec_ref(x_376);
 x_379 = lean_box(0);
}
x_380 = lean_ctor_get(x_378, 0);
lean_inc(x_380);
x_381 = lean_ctor_get(x_378, 1);
lean_inc(x_381);
if (lean_is_exclusive(x_378)) {
 lean_ctor_release(x_378, 0);
 lean_ctor_release(x_378, 1);
 x_382 = x_378;
} else {
 lean_dec_ref(x_378);
 x_382 = lean_box(0);
}
if (lean_is_scalar(x_371)) {
 x_383 = lean_alloc_ctor(9, 1, 0);
} else {
 x_383 = x_371;
 lean_ctor_set_tag(x_383, 9);
}
lean_ctor_set(x_383, 0, x_380);
if (lean_is_scalar(x_382)) {
 x_384 = lean_alloc_ctor(0, 2, 0);
} else {
 x_384 = x_382;
}
lean_ctor_set(x_384, 0, x_383);
lean_ctor_set(x_384, 1, x_381);
if (lean_is_scalar(x_379)) {
 x_385 = lean_alloc_ctor(1, 1, 0);
} else {
 x_385 = x_379;
}
lean_ctor_set(x_385, 0, x_384);
return x_385;
}
}
}
}
else
{
lean_object* x_386; 
x_386 = lp_iris_x2dkernel_IrisKernel_FFI_decodeCostBound(x_324);
if (lean_obj_tag(x_386) == 0)
{
return x_386;
}
else
{
lean_object* x_387; lean_object* x_388; lean_object* x_389; lean_object* x_390; lean_object* x_391; 
x_387 = lean_ctor_get(x_386, 0);
lean_inc(x_387);
lean_dec_ref(x_386);
x_388 = lean_ctor_get(x_387, 0);
lean_inc(x_388);
x_389 = lean_ctor_get(x_387, 1);
lean_inc(x_389);
if (lean_is_exclusive(x_387)) {
 lean_ctor_release(x_387, 0);
 lean_ctor_release(x_387, 1);
 x_390 = x_387;
} else {
 lean_dec_ref(x_387);
 x_390 = lean_box(0);
}
x_391 = lp_iris_x2dkernel_IrisKernel_FFI_decodeCostBound(x_389);
if (lean_obj_tag(x_391) == 0)
{
lean_dec(x_390);
lean_dec(x_388);
return x_391;
}
else
{
lean_object* x_392; lean_object* x_393; lean_object* x_394; lean_object* x_395; lean_object* x_396; lean_object* x_397; lean_object* x_398; lean_object* x_399; 
x_392 = lean_ctor_get(x_391, 0);
lean_inc(x_392);
if (lean_is_exclusive(x_391)) {
 lean_ctor_release(x_391, 0);
 x_393 = x_391;
} else {
 lean_dec_ref(x_391);
 x_393 = lean_box(0);
}
x_394 = lean_ctor_get(x_392, 0);
lean_inc(x_394);
x_395 = lean_ctor_get(x_392, 1);
lean_inc(x_395);
if (lean_is_exclusive(x_392)) {
 lean_ctor_release(x_392, 0);
 lean_ctor_release(x_392, 1);
 x_396 = x_392;
} else {
 lean_dec_ref(x_392);
 x_396 = lean_box(0);
}
if (lean_is_scalar(x_390)) {
 x_397 = lean_alloc_ctor(8, 2, 0);
} else {
 x_397 = x_390;
 lean_ctor_set_tag(x_397, 8);
}
lean_ctor_set(x_397, 0, x_388);
lean_ctor_set(x_397, 1, x_394);
if (lean_is_scalar(x_396)) {
 x_398 = lean_alloc_ctor(0, 2, 0);
} else {
 x_398 = x_396;
}
lean_ctor_set(x_398, 0, x_397);
lean_ctor_set(x_398, 1, x_395);
if (lean_is_scalar(x_393)) {
 x_399 = lean_alloc_ctor(1, 1, 0);
} else {
 x_399 = x_393;
}
lean_ctor_set(x_399, 0, x_398);
return x_399;
}
}
}
}
else
{
lean_object* x_400; 
x_400 = lp_iris_x2dkernel_IrisKernel_FFI_decodeCostBound(x_324);
if (lean_obj_tag(x_400) == 0)
{
return x_400;
}
else
{
lean_object* x_401; lean_object* x_402; lean_object* x_403; lean_object* x_404; lean_object* x_405; 
x_401 = lean_ctor_get(x_400, 0);
lean_inc(x_401);
lean_dec_ref(x_400);
x_402 = lean_ctor_get(x_401, 0);
lean_inc(x_402);
x_403 = lean_ctor_get(x_401, 1);
lean_inc(x_403);
if (lean_is_exclusive(x_401)) {
 lean_ctor_release(x_401, 0);
 lean_ctor_release(x_401, 1);
 x_404 = x_401;
} else {
 lean_dec_ref(x_401);
 x_404 = lean_box(0);
}
x_405 = lp_iris_x2dkernel_IrisKernel_FFI_decodeCostBound(x_403);
if (lean_obj_tag(x_405) == 0)
{
lean_dec(x_404);
lean_dec(x_402);
return x_405;
}
else
{
lean_object* x_406; lean_object* x_407; lean_object* x_408; lean_object* x_409; lean_object* x_410; lean_object* x_411; lean_object* x_412; lean_object* x_413; 
x_406 = lean_ctor_get(x_405, 0);
lean_inc(x_406);
if (lean_is_exclusive(x_405)) {
 lean_ctor_release(x_405, 0);
 x_407 = x_405;
} else {
 lean_dec_ref(x_405);
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
if (lean_is_scalar(x_404)) {
 x_411 = lean_alloc_ctor(7, 2, 0);
} else {
 x_411 = x_404;
 lean_ctor_set_tag(x_411, 7);
}
lean_ctor_set(x_411, 0, x_402);
lean_ctor_set(x_411, 1, x_408);
if (lean_is_scalar(x_410)) {
 x_412 = lean_alloc_ctor(0, 2, 0);
} else {
 x_412 = x_410;
}
lean_ctor_set(x_412, 0, x_411);
lean_ctor_set(x_412, 1, x_409);
if (lean_is_scalar(x_407)) {
 x_413 = lean_alloc_ctor(1, 1, 0);
} else {
 x_413 = x_407;
}
lean_ctor_set(x_413, 0, x_412);
return x_413;
}
}
}
}
else
{
lean_object* x_414; 
x_414 = lp_iris_x2dkernel_IrisKernel_FFI_decodeCostBound(x_324);
if (lean_obj_tag(x_414) == 0)
{
return x_414;
}
else
{
lean_object* x_415; lean_object* x_416; lean_object* x_417; lean_object* x_418; lean_object* x_419; 
x_415 = lean_ctor_get(x_414, 0);
lean_inc(x_415);
lean_dec_ref(x_414);
x_416 = lean_ctor_get(x_415, 0);
lean_inc(x_416);
x_417 = lean_ctor_get(x_415, 1);
lean_inc(x_417);
if (lean_is_exclusive(x_415)) {
 lean_ctor_release(x_415, 0);
 lean_ctor_release(x_415, 1);
 x_418 = x_415;
} else {
 lean_dec_ref(x_415);
 x_418 = lean_box(0);
}
x_419 = lp_iris_x2dkernel_IrisKernel_FFI_decodeCostBound(x_417);
if (lean_obj_tag(x_419) == 0)
{
lean_dec(x_418);
lean_dec(x_416);
return x_419;
}
else
{
lean_object* x_420; lean_object* x_421; lean_object* x_422; lean_object* x_423; lean_object* x_424; lean_object* x_425; lean_object* x_426; lean_object* x_427; 
x_420 = lean_ctor_get(x_419, 0);
lean_inc(x_420);
if (lean_is_exclusive(x_419)) {
 lean_ctor_release(x_419, 0);
 x_421 = x_419;
} else {
 lean_dec_ref(x_419);
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
if (lean_is_scalar(x_418)) {
 x_425 = lean_alloc_ctor(6, 2, 0);
} else {
 x_425 = x_418;
 lean_ctor_set_tag(x_425, 6);
}
lean_ctor_set(x_425, 0, x_416);
lean_ctor_set(x_425, 1, x_422);
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
x_428 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readUInt32LE(x_324);
if (lean_obj_tag(x_428) == 0)
{
lean_object* x_429; 
x_429 = lean_box(0);
return x_429;
}
else
{
lean_object* x_430; lean_object* x_431; lean_object* x_432; lean_object* x_433; lean_object* x_434; 
x_430 = lean_ctor_get(x_428, 0);
lean_inc(x_430);
lean_dec_ref(x_428);
x_431 = lean_ctor_get(x_430, 0);
lean_inc(x_431);
x_432 = lean_ctor_get(x_430, 1);
lean_inc(x_432);
if (lean_is_exclusive(x_430)) {
 lean_ctor_release(x_430, 0);
 lean_ctor_release(x_430, 1);
 x_433 = x_430;
} else {
 lean_dec_ref(x_430);
 x_433 = lean_box(0);
}
x_434 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readUInt32LE(x_432);
if (lean_obj_tag(x_434) == 0)
{
lean_object* x_435; 
lean_dec(x_433);
lean_dec(x_431);
x_435 = lean_box(0);
return x_435;
}
else
{
lean_object* x_436; lean_object* x_437; lean_object* x_438; lean_object* x_439; lean_object* x_440; uint32_t x_441; lean_object* x_442; uint32_t x_443; lean_object* x_444; lean_object* x_445; lean_object* x_446; lean_object* x_447; 
x_436 = lean_ctor_get(x_434, 0);
lean_inc(x_436);
if (lean_is_exclusive(x_434)) {
 lean_ctor_release(x_434, 0);
 x_437 = x_434;
} else {
 lean_dec_ref(x_434);
 x_437 = lean_box(0);
}
x_438 = lean_ctor_get(x_436, 0);
lean_inc(x_438);
x_439 = lean_ctor_get(x_436, 1);
lean_inc(x_439);
if (lean_is_exclusive(x_436)) {
 lean_ctor_release(x_436, 0);
 lean_ctor_release(x_436, 1);
 x_440 = x_436;
} else {
 lean_dec_ref(x_436);
 x_440 = lean_box(0);
}
x_441 = lean_unbox_uint32(x_431);
lean_dec(x_431);
x_442 = lean_uint32_to_nat(x_441);
x_443 = lean_unbox_uint32(x_438);
lean_dec(x_438);
x_444 = lean_uint32_to_nat(x_443);
if (lean_is_scalar(x_433)) {
 x_445 = lean_alloc_ctor(5, 2, 0);
} else {
 x_445 = x_433;
 lean_ctor_set_tag(x_445, 5);
}
lean_ctor_set(x_445, 0, x_442);
lean_ctor_set(x_445, 1, x_444);
if (lean_is_scalar(x_440)) {
 x_446 = lean_alloc_ctor(0, 2, 0);
} else {
 x_446 = x_440;
}
lean_ctor_set(x_446, 0, x_445);
lean_ctor_set(x_446, 1, x_439);
if (lean_is_scalar(x_437)) {
 x_447 = lean_alloc_ctor(1, 1, 0);
} else {
 x_447 = x_437;
}
lean_ctor_set(x_447, 0, x_446);
return x_447;
}
}
}
}
else
{
lean_object* x_448; 
x_448 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readUInt32LE(x_324);
if (lean_obj_tag(x_448) == 0)
{
lean_object* x_449; 
x_449 = lean_box(0);
return x_449;
}
else
{
lean_object* x_450; lean_object* x_451; lean_object* x_452; lean_object* x_453; lean_object* x_454; uint32_t x_455; lean_object* x_456; lean_object* x_457; lean_object* x_458; lean_object* x_459; 
x_450 = lean_ctor_get(x_448, 0);
lean_inc(x_450);
if (lean_is_exclusive(x_448)) {
 lean_ctor_release(x_448, 0);
 x_451 = x_448;
} else {
 lean_dec_ref(x_448);
 x_451 = lean_box(0);
}
x_452 = lean_ctor_get(x_450, 0);
lean_inc(x_452);
x_453 = lean_ctor_get(x_450, 1);
lean_inc(x_453);
if (lean_is_exclusive(x_450)) {
 lean_ctor_release(x_450, 0);
 lean_ctor_release(x_450, 1);
 x_454 = x_450;
} else {
 lean_dec_ref(x_450);
 x_454 = lean_box(0);
}
x_455 = lean_unbox_uint32(x_452);
lean_dec(x_452);
x_456 = lean_uint32_to_nat(x_455);
x_457 = lean_alloc_ctor(4, 1, 0);
lean_ctor_set(x_457, 0, x_456);
if (lean_is_scalar(x_454)) {
 x_458 = lean_alloc_ctor(0, 2, 0);
} else {
 x_458 = x_454;
}
lean_ctor_set(x_458, 0, x_457);
lean_ctor_set(x_458, 1, x_453);
if (lean_is_scalar(x_451)) {
 x_459 = lean_alloc_ctor(1, 1, 0);
} else {
 x_459 = x_451;
}
lean_ctor_set(x_459, 0, x_458);
return x_459;
}
}
}
else
{
lean_object* x_460; 
x_460 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readUInt32LE(x_324);
if (lean_obj_tag(x_460) == 0)
{
lean_object* x_461; 
x_461 = lean_box(0);
return x_461;
}
else
{
lean_object* x_462; lean_object* x_463; lean_object* x_464; lean_object* x_465; lean_object* x_466; uint32_t x_467; lean_object* x_468; lean_object* x_469; lean_object* x_470; lean_object* x_471; 
x_462 = lean_ctor_get(x_460, 0);
lean_inc(x_462);
if (lean_is_exclusive(x_460)) {
 lean_ctor_release(x_460, 0);
 x_463 = x_460;
} else {
 lean_dec_ref(x_460);
 x_463 = lean_box(0);
}
x_464 = lean_ctor_get(x_462, 0);
lean_inc(x_464);
x_465 = lean_ctor_get(x_462, 1);
lean_inc(x_465);
if (lean_is_exclusive(x_462)) {
 lean_ctor_release(x_462, 0);
 lean_ctor_release(x_462, 1);
 x_466 = x_462;
} else {
 lean_dec_ref(x_462);
 x_466 = lean_box(0);
}
x_467 = lean_unbox_uint32(x_464);
lean_dec(x_464);
x_468 = lean_uint32_to_nat(x_467);
x_469 = lean_alloc_ctor(3, 1, 0);
lean_ctor_set(x_469, 0, x_468);
if (lean_is_scalar(x_466)) {
 x_470 = lean_alloc_ctor(0, 2, 0);
} else {
 x_470 = x_466;
}
lean_ctor_set(x_470, 0, x_469);
lean_ctor_set(x_470, 1, x_465);
if (lean_is_scalar(x_463)) {
 x_471 = lean_alloc_ctor(1, 1, 0);
} else {
 x_471 = x_463;
}
lean_ctor_set(x_471, 0, x_470);
return x_471;
}
}
}
else
{
lean_object* x_472; 
x_472 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readUInt64LE(x_324);
if (lean_obj_tag(x_472) == 0)
{
lean_object* x_473; 
x_473 = lean_box(0);
return x_473;
}
else
{
lean_object* x_474; lean_object* x_475; lean_object* x_476; lean_object* x_477; lean_object* x_478; uint64_t x_479; lean_object* x_480; lean_object* x_481; lean_object* x_482; lean_object* x_483; 
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
x_479 = lean_unbox_uint64(x_476);
lean_dec(x_476);
x_480 = lean_uint64_to_nat(x_479);
x_481 = lean_alloc_ctor(2, 1, 0);
lean_ctor_set(x_481, 0, x_480);
if (lean_is_scalar(x_478)) {
 x_482 = lean_alloc_ctor(0, 2, 0);
} else {
 x_482 = x_478;
}
lean_ctor_set(x_482, 0, x_481);
lean_ctor_set(x_482, 1, x_477);
if (lean_is_scalar(x_475)) {
 x_483 = lean_alloc_ctor(1, 1, 0);
} else {
 x_483 = x_475;
}
lean_ctor_set(x_483, 0, x_482);
return x_483;
}
}
}
else
{
lean_object* x_484; lean_object* x_485; 
x_484 = lean_box(1);
x_485 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_485, 0, x_484);
lean_ctor_set(x_485, 1, x_324);
lean_ctor_set(x_2, 0, x_485);
return x_2;
}
}
else
{
lean_object* x_486; lean_object* x_487; 
x_486 = lean_box(0);
x_487 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_487, 0, x_486);
lean_ctor_set(x_487, 1, x_324);
lean_ctor_set(x_2, 0, x_487);
return x_2;
}
}
}
else
{
lean_object* x_488; lean_object* x_489; lean_object* x_490; lean_object* x_491; uint8_t x_492; lean_object* x_493; lean_object* x_494; uint8_t x_495; 
x_488 = lean_ctor_get(x_2, 0);
lean_inc(x_488);
lean_dec(x_2);
x_489 = lean_ctor_get(x_488, 0);
lean_inc(x_489);
x_490 = lean_ctor_get(x_488, 1);
lean_inc(x_490);
if (lean_is_exclusive(x_488)) {
 lean_ctor_release(x_488, 0);
 lean_ctor_release(x_488, 1);
 x_491 = x_488;
} else {
 lean_dec_ref(x_488);
 x_491 = lean_box(0);
}
x_492 = lean_unbox(x_489);
lean_dec(x_489);
x_493 = lean_uint8_to_nat(x_492);
x_494 = lean_unsigned_to_nat(0u);
x_495 = lean_nat_dec_eq(x_493, x_494);
if (x_495 == 0)
{
lean_object* x_496; uint8_t x_497; 
x_496 = lean_unsigned_to_nat(1u);
x_497 = lean_nat_dec_eq(x_493, x_496);
if (x_497 == 0)
{
lean_object* x_498; uint8_t x_499; 
lean_dec(x_491);
x_498 = lean_unsigned_to_nat(2u);
x_499 = lean_nat_dec_eq(x_493, x_498);
if (x_499 == 0)
{
lean_object* x_500; uint8_t x_501; 
x_500 = lean_unsigned_to_nat(3u);
x_501 = lean_nat_dec_eq(x_493, x_500);
if (x_501 == 0)
{
lean_object* x_502; uint8_t x_503; 
x_502 = lean_unsigned_to_nat(4u);
x_503 = lean_nat_dec_eq(x_493, x_502);
if (x_503 == 0)
{
lean_object* x_504; uint8_t x_505; 
x_504 = lean_unsigned_to_nat(5u);
x_505 = lean_nat_dec_eq(x_493, x_504);
if (x_505 == 0)
{
lean_object* x_506; uint8_t x_507; 
x_506 = lean_unsigned_to_nat(6u);
x_507 = lean_nat_dec_eq(x_493, x_506);
if (x_507 == 0)
{
lean_object* x_508; uint8_t x_509; 
x_508 = lean_unsigned_to_nat(7u);
x_509 = lean_nat_dec_eq(x_493, x_508);
if (x_509 == 0)
{
lean_object* x_510; uint8_t x_511; 
x_510 = lean_unsigned_to_nat(8u);
x_511 = lean_nat_dec_eq(x_493, x_510);
if (x_511 == 0)
{
lean_object* x_512; uint8_t x_513; 
x_512 = lean_unsigned_to_nat(9u);
x_513 = lean_nat_dec_eq(x_493, x_512);
if (x_513 == 0)
{
lean_object* x_514; uint8_t x_515; 
x_514 = lean_unsigned_to_nat(10u);
x_515 = lean_nat_dec_eq(x_493, x_514);
if (x_515 == 0)
{
lean_object* x_516; 
lean_dec(x_490);
x_516 = lean_box(0);
return x_516;
}
else
{
lean_object* x_517; 
x_517 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readUInt16LE(x_490);
if (lean_obj_tag(x_517) == 0)
{
lean_object* x_518; 
x_518 = lean_box(0);
return x_518;
}
else
{
lean_object* x_519; lean_object* x_520; lean_object* x_521; lean_object* x_522; uint16_t x_523; lean_object* x_524; lean_object* x_525; 
x_519 = lean_ctor_get(x_517, 0);
lean_inc(x_519);
if (lean_is_exclusive(x_517)) {
 lean_ctor_release(x_517, 0);
 x_520 = x_517;
} else {
 lean_dec_ref(x_517);
 x_520 = lean_box(0);
}
x_521 = lean_ctor_get(x_519, 0);
lean_inc(x_521);
x_522 = lean_ctor_get(x_519, 1);
lean_inc(x_522);
lean_dec(x_519);
x_523 = lean_unbox(x_521);
lean_dec(x_521);
x_524 = lean_uint16_to_nat(x_523);
x_525 = lp_iris_x2dkernel_IrisKernel_FFI_decodeCostBound_decodeCostBoundList(x_522, x_524);
if (lean_obj_tag(x_525) == 0)
{
lean_object* x_526; 
lean_dec(x_520);
x_526 = lean_box(0);
return x_526;
}
else
{
lean_object* x_527; lean_object* x_528; lean_object* x_529; lean_object* x_530; lean_object* x_531; lean_object* x_532; lean_object* x_533; lean_object* x_534; 
x_527 = lean_ctor_get(x_525, 0);
lean_inc(x_527);
if (lean_is_exclusive(x_525)) {
 lean_ctor_release(x_525, 0);
 x_528 = x_525;
} else {
 lean_dec_ref(x_525);
 x_528 = lean_box(0);
}
x_529 = lean_ctor_get(x_527, 0);
lean_inc(x_529);
x_530 = lean_ctor_get(x_527, 1);
lean_inc(x_530);
if (lean_is_exclusive(x_527)) {
 lean_ctor_release(x_527, 0);
 lean_ctor_release(x_527, 1);
 x_531 = x_527;
} else {
 lean_dec_ref(x_527);
 x_531 = lean_box(0);
}
if (lean_is_scalar(x_520)) {
 x_532 = lean_alloc_ctor(10, 1, 0);
} else {
 x_532 = x_520;
 lean_ctor_set_tag(x_532, 10);
}
lean_ctor_set(x_532, 0, x_529);
if (lean_is_scalar(x_531)) {
 x_533 = lean_alloc_ctor(0, 2, 0);
} else {
 x_533 = x_531;
}
lean_ctor_set(x_533, 0, x_532);
lean_ctor_set(x_533, 1, x_530);
if (lean_is_scalar(x_528)) {
 x_534 = lean_alloc_ctor(1, 1, 0);
} else {
 x_534 = x_528;
}
lean_ctor_set(x_534, 0, x_533);
return x_534;
}
}
}
}
else
{
lean_object* x_535; 
x_535 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readUInt16LE(x_490);
if (lean_obj_tag(x_535) == 0)
{
lean_object* x_536; 
x_536 = lean_box(0);
return x_536;
}
else
{
lean_object* x_537; lean_object* x_538; lean_object* x_539; lean_object* x_540; uint16_t x_541; lean_object* x_542; lean_object* x_543; 
x_537 = lean_ctor_get(x_535, 0);
lean_inc(x_537);
if (lean_is_exclusive(x_535)) {
 lean_ctor_release(x_535, 0);
 x_538 = x_535;
} else {
 lean_dec_ref(x_535);
 x_538 = lean_box(0);
}
x_539 = lean_ctor_get(x_537, 0);
lean_inc(x_539);
x_540 = lean_ctor_get(x_537, 1);
lean_inc(x_540);
lean_dec(x_537);
x_541 = lean_unbox(x_539);
lean_dec(x_539);
x_542 = lean_uint16_to_nat(x_541);
x_543 = lp_iris_x2dkernel_IrisKernel_FFI_decodeCostBound_decodeCostBoundList(x_540, x_542);
if (lean_obj_tag(x_543) == 0)
{
lean_object* x_544; 
lean_dec(x_538);
x_544 = lean_box(0);
return x_544;
}
else
{
lean_object* x_545; lean_object* x_546; lean_object* x_547; lean_object* x_548; lean_object* x_549; lean_object* x_550; lean_object* x_551; lean_object* x_552; 
x_545 = lean_ctor_get(x_543, 0);
lean_inc(x_545);
if (lean_is_exclusive(x_543)) {
 lean_ctor_release(x_543, 0);
 x_546 = x_543;
} else {
 lean_dec_ref(x_543);
 x_546 = lean_box(0);
}
x_547 = lean_ctor_get(x_545, 0);
lean_inc(x_547);
x_548 = lean_ctor_get(x_545, 1);
lean_inc(x_548);
if (lean_is_exclusive(x_545)) {
 lean_ctor_release(x_545, 0);
 lean_ctor_release(x_545, 1);
 x_549 = x_545;
} else {
 lean_dec_ref(x_545);
 x_549 = lean_box(0);
}
if (lean_is_scalar(x_538)) {
 x_550 = lean_alloc_ctor(9, 1, 0);
} else {
 x_550 = x_538;
 lean_ctor_set_tag(x_550, 9);
}
lean_ctor_set(x_550, 0, x_547);
if (lean_is_scalar(x_549)) {
 x_551 = lean_alloc_ctor(0, 2, 0);
} else {
 x_551 = x_549;
}
lean_ctor_set(x_551, 0, x_550);
lean_ctor_set(x_551, 1, x_548);
if (lean_is_scalar(x_546)) {
 x_552 = lean_alloc_ctor(1, 1, 0);
} else {
 x_552 = x_546;
}
lean_ctor_set(x_552, 0, x_551);
return x_552;
}
}
}
}
else
{
lean_object* x_553; 
x_553 = lp_iris_x2dkernel_IrisKernel_FFI_decodeCostBound(x_490);
if (lean_obj_tag(x_553) == 0)
{
return x_553;
}
else
{
lean_object* x_554; lean_object* x_555; lean_object* x_556; lean_object* x_557; lean_object* x_558; 
x_554 = lean_ctor_get(x_553, 0);
lean_inc(x_554);
lean_dec_ref(x_553);
x_555 = lean_ctor_get(x_554, 0);
lean_inc(x_555);
x_556 = lean_ctor_get(x_554, 1);
lean_inc(x_556);
if (lean_is_exclusive(x_554)) {
 lean_ctor_release(x_554, 0);
 lean_ctor_release(x_554, 1);
 x_557 = x_554;
} else {
 lean_dec_ref(x_554);
 x_557 = lean_box(0);
}
x_558 = lp_iris_x2dkernel_IrisKernel_FFI_decodeCostBound(x_556);
if (lean_obj_tag(x_558) == 0)
{
lean_dec(x_557);
lean_dec(x_555);
return x_558;
}
else
{
lean_object* x_559; lean_object* x_560; lean_object* x_561; lean_object* x_562; lean_object* x_563; lean_object* x_564; lean_object* x_565; lean_object* x_566; 
x_559 = lean_ctor_get(x_558, 0);
lean_inc(x_559);
if (lean_is_exclusive(x_558)) {
 lean_ctor_release(x_558, 0);
 x_560 = x_558;
} else {
 lean_dec_ref(x_558);
 x_560 = lean_box(0);
}
x_561 = lean_ctor_get(x_559, 0);
lean_inc(x_561);
x_562 = lean_ctor_get(x_559, 1);
lean_inc(x_562);
if (lean_is_exclusive(x_559)) {
 lean_ctor_release(x_559, 0);
 lean_ctor_release(x_559, 1);
 x_563 = x_559;
} else {
 lean_dec_ref(x_559);
 x_563 = lean_box(0);
}
if (lean_is_scalar(x_557)) {
 x_564 = lean_alloc_ctor(8, 2, 0);
} else {
 x_564 = x_557;
 lean_ctor_set_tag(x_564, 8);
}
lean_ctor_set(x_564, 0, x_555);
lean_ctor_set(x_564, 1, x_561);
if (lean_is_scalar(x_563)) {
 x_565 = lean_alloc_ctor(0, 2, 0);
} else {
 x_565 = x_563;
}
lean_ctor_set(x_565, 0, x_564);
lean_ctor_set(x_565, 1, x_562);
if (lean_is_scalar(x_560)) {
 x_566 = lean_alloc_ctor(1, 1, 0);
} else {
 x_566 = x_560;
}
lean_ctor_set(x_566, 0, x_565);
return x_566;
}
}
}
}
else
{
lean_object* x_567; 
x_567 = lp_iris_x2dkernel_IrisKernel_FFI_decodeCostBound(x_490);
if (lean_obj_tag(x_567) == 0)
{
return x_567;
}
else
{
lean_object* x_568; lean_object* x_569; lean_object* x_570; lean_object* x_571; lean_object* x_572; 
x_568 = lean_ctor_get(x_567, 0);
lean_inc(x_568);
lean_dec_ref(x_567);
x_569 = lean_ctor_get(x_568, 0);
lean_inc(x_569);
x_570 = lean_ctor_get(x_568, 1);
lean_inc(x_570);
if (lean_is_exclusive(x_568)) {
 lean_ctor_release(x_568, 0);
 lean_ctor_release(x_568, 1);
 x_571 = x_568;
} else {
 lean_dec_ref(x_568);
 x_571 = lean_box(0);
}
x_572 = lp_iris_x2dkernel_IrisKernel_FFI_decodeCostBound(x_570);
if (lean_obj_tag(x_572) == 0)
{
lean_dec(x_571);
lean_dec(x_569);
return x_572;
}
else
{
lean_object* x_573; lean_object* x_574; lean_object* x_575; lean_object* x_576; lean_object* x_577; lean_object* x_578; lean_object* x_579; lean_object* x_580; 
x_573 = lean_ctor_get(x_572, 0);
lean_inc(x_573);
if (lean_is_exclusive(x_572)) {
 lean_ctor_release(x_572, 0);
 x_574 = x_572;
} else {
 lean_dec_ref(x_572);
 x_574 = lean_box(0);
}
x_575 = lean_ctor_get(x_573, 0);
lean_inc(x_575);
x_576 = lean_ctor_get(x_573, 1);
lean_inc(x_576);
if (lean_is_exclusive(x_573)) {
 lean_ctor_release(x_573, 0);
 lean_ctor_release(x_573, 1);
 x_577 = x_573;
} else {
 lean_dec_ref(x_573);
 x_577 = lean_box(0);
}
if (lean_is_scalar(x_571)) {
 x_578 = lean_alloc_ctor(7, 2, 0);
} else {
 x_578 = x_571;
 lean_ctor_set_tag(x_578, 7);
}
lean_ctor_set(x_578, 0, x_569);
lean_ctor_set(x_578, 1, x_575);
if (lean_is_scalar(x_577)) {
 x_579 = lean_alloc_ctor(0, 2, 0);
} else {
 x_579 = x_577;
}
lean_ctor_set(x_579, 0, x_578);
lean_ctor_set(x_579, 1, x_576);
if (lean_is_scalar(x_574)) {
 x_580 = lean_alloc_ctor(1, 1, 0);
} else {
 x_580 = x_574;
}
lean_ctor_set(x_580, 0, x_579);
return x_580;
}
}
}
}
else
{
lean_object* x_581; 
x_581 = lp_iris_x2dkernel_IrisKernel_FFI_decodeCostBound(x_490);
if (lean_obj_tag(x_581) == 0)
{
return x_581;
}
else
{
lean_object* x_582; lean_object* x_583; lean_object* x_584; lean_object* x_585; lean_object* x_586; 
x_582 = lean_ctor_get(x_581, 0);
lean_inc(x_582);
lean_dec_ref(x_581);
x_583 = lean_ctor_get(x_582, 0);
lean_inc(x_583);
x_584 = lean_ctor_get(x_582, 1);
lean_inc(x_584);
if (lean_is_exclusive(x_582)) {
 lean_ctor_release(x_582, 0);
 lean_ctor_release(x_582, 1);
 x_585 = x_582;
} else {
 lean_dec_ref(x_582);
 x_585 = lean_box(0);
}
x_586 = lp_iris_x2dkernel_IrisKernel_FFI_decodeCostBound(x_584);
if (lean_obj_tag(x_586) == 0)
{
lean_dec(x_585);
lean_dec(x_583);
return x_586;
}
else
{
lean_object* x_587; lean_object* x_588; lean_object* x_589; lean_object* x_590; lean_object* x_591; lean_object* x_592; lean_object* x_593; lean_object* x_594; 
x_587 = lean_ctor_get(x_586, 0);
lean_inc(x_587);
if (lean_is_exclusive(x_586)) {
 lean_ctor_release(x_586, 0);
 x_588 = x_586;
} else {
 lean_dec_ref(x_586);
 x_588 = lean_box(0);
}
x_589 = lean_ctor_get(x_587, 0);
lean_inc(x_589);
x_590 = lean_ctor_get(x_587, 1);
lean_inc(x_590);
if (lean_is_exclusive(x_587)) {
 lean_ctor_release(x_587, 0);
 lean_ctor_release(x_587, 1);
 x_591 = x_587;
} else {
 lean_dec_ref(x_587);
 x_591 = lean_box(0);
}
if (lean_is_scalar(x_585)) {
 x_592 = lean_alloc_ctor(6, 2, 0);
} else {
 x_592 = x_585;
 lean_ctor_set_tag(x_592, 6);
}
lean_ctor_set(x_592, 0, x_583);
lean_ctor_set(x_592, 1, x_589);
if (lean_is_scalar(x_591)) {
 x_593 = lean_alloc_ctor(0, 2, 0);
} else {
 x_593 = x_591;
}
lean_ctor_set(x_593, 0, x_592);
lean_ctor_set(x_593, 1, x_590);
if (lean_is_scalar(x_588)) {
 x_594 = lean_alloc_ctor(1, 1, 0);
} else {
 x_594 = x_588;
}
lean_ctor_set(x_594, 0, x_593);
return x_594;
}
}
}
}
else
{
lean_object* x_595; 
x_595 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readUInt32LE(x_490);
if (lean_obj_tag(x_595) == 0)
{
lean_object* x_596; 
x_596 = lean_box(0);
return x_596;
}
else
{
lean_object* x_597; lean_object* x_598; lean_object* x_599; lean_object* x_600; lean_object* x_601; 
x_597 = lean_ctor_get(x_595, 0);
lean_inc(x_597);
lean_dec_ref(x_595);
x_598 = lean_ctor_get(x_597, 0);
lean_inc(x_598);
x_599 = lean_ctor_get(x_597, 1);
lean_inc(x_599);
if (lean_is_exclusive(x_597)) {
 lean_ctor_release(x_597, 0);
 lean_ctor_release(x_597, 1);
 x_600 = x_597;
} else {
 lean_dec_ref(x_597);
 x_600 = lean_box(0);
}
x_601 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readUInt32LE(x_599);
if (lean_obj_tag(x_601) == 0)
{
lean_object* x_602; 
lean_dec(x_600);
lean_dec(x_598);
x_602 = lean_box(0);
return x_602;
}
else
{
lean_object* x_603; lean_object* x_604; lean_object* x_605; lean_object* x_606; lean_object* x_607; uint32_t x_608; lean_object* x_609; uint32_t x_610; lean_object* x_611; lean_object* x_612; lean_object* x_613; lean_object* x_614; 
x_603 = lean_ctor_get(x_601, 0);
lean_inc(x_603);
if (lean_is_exclusive(x_601)) {
 lean_ctor_release(x_601, 0);
 x_604 = x_601;
} else {
 lean_dec_ref(x_601);
 x_604 = lean_box(0);
}
x_605 = lean_ctor_get(x_603, 0);
lean_inc(x_605);
x_606 = lean_ctor_get(x_603, 1);
lean_inc(x_606);
if (lean_is_exclusive(x_603)) {
 lean_ctor_release(x_603, 0);
 lean_ctor_release(x_603, 1);
 x_607 = x_603;
} else {
 lean_dec_ref(x_603);
 x_607 = lean_box(0);
}
x_608 = lean_unbox_uint32(x_598);
lean_dec(x_598);
x_609 = lean_uint32_to_nat(x_608);
x_610 = lean_unbox_uint32(x_605);
lean_dec(x_605);
x_611 = lean_uint32_to_nat(x_610);
if (lean_is_scalar(x_600)) {
 x_612 = lean_alloc_ctor(5, 2, 0);
} else {
 x_612 = x_600;
 lean_ctor_set_tag(x_612, 5);
}
lean_ctor_set(x_612, 0, x_609);
lean_ctor_set(x_612, 1, x_611);
if (lean_is_scalar(x_607)) {
 x_613 = lean_alloc_ctor(0, 2, 0);
} else {
 x_613 = x_607;
}
lean_ctor_set(x_613, 0, x_612);
lean_ctor_set(x_613, 1, x_606);
if (lean_is_scalar(x_604)) {
 x_614 = lean_alloc_ctor(1, 1, 0);
} else {
 x_614 = x_604;
}
lean_ctor_set(x_614, 0, x_613);
return x_614;
}
}
}
}
else
{
lean_object* x_615; 
x_615 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readUInt32LE(x_490);
if (lean_obj_tag(x_615) == 0)
{
lean_object* x_616; 
x_616 = lean_box(0);
return x_616;
}
else
{
lean_object* x_617; lean_object* x_618; lean_object* x_619; lean_object* x_620; lean_object* x_621; uint32_t x_622; lean_object* x_623; lean_object* x_624; lean_object* x_625; lean_object* x_626; 
x_617 = lean_ctor_get(x_615, 0);
lean_inc(x_617);
if (lean_is_exclusive(x_615)) {
 lean_ctor_release(x_615, 0);
 x_618 = x_615;
} else {
 lean_dec_ref(x_615);
 x_618 = lean_box(0);
}
x_619 = lean_ctor_get(x_617, 0);
lean_inc(x_619);
x_620 = lean_ctor_get(x_617, 1);
lean_inc(x_620);
if (lean_is_exclusive(x_617)) {
 lean_ctor_release(x_617, 0);
 lean_ctor_release(x_617, 1);
 x_621 = x_617;
} else {
 lean_dec_ref(x_617);
 x_621 = lean_box(0);
}
x_622 = lean_unbox_uint32(x_619);
lean_dec(x_619);
x_623 = lean_uint32_to_nat(x_622);
x_624 = lean_alloc_ctor(4, 1, 0);
lean_ctor_set(x_624, 0, x_623);
if (lean_is_scalar(x_621)) {
 x_625 = lean_alloc_ctor(0, 2, 0);
} else {
 x_625 = x_621;
}
lean_ctor_set(x_625, 0, x_624);
lean_ctor_set(x_625, 1, x_620);
if (lean_is_scalar(x_618)) {
 x_626 = lean_alloc_ctor(1, 1, 0);
} else {
 x_626 = x_618;
}
lean_ctor_set(x_626, 0, x_625);
return x_626;
}
}
}
else
{
lean_object* x_627; 
x_627 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readUInt32LE(x_490);
if (lean_obj_tag(x_627) == 0)
{
lean_object* x_628; 
x_628 = lean_box(0);
return x_628;
}
else
{
lean_object* x_629; lean_object* x_630; lean_object* x_631; lean_object* x_632; lean_object* x_633; uint32_t x_634; lean_object* x_635; lean_object* x_636; lean_object* x_637; lean_object* x_638; 
x_629 = lean_ctor_get(x_627, 0);
lean_inc(x_629);
if (lean_is_exclusive(x_627)) {
 lean_ctor_release(x_627, 0);
 x_630 = x_627;
} else {
 lean_dec_ref(x_627);
 x_630 = lean_box(0);
}
x_631 = lean_ctor_get(x_629, 0);
lean_inc(x_631);
x_632 = lean_ctor_get(x_629, 1);
lean_inc(x_632);
if (lean_is_exclusive(x_629)) {
 lean_ctor_release(x_629, 0);
 lean_ctor_release(x_629, 1);
 x_633 = x_629;
} else {
 lean_dec_ref(x_629);
 x_633 = lean_box(0);
}
x_634 = lean_unbox_uint32(x_631);
lean_dec(x_631);
x_635 = lean_uint32_to_nat(x_634);
x_636 = lean_alloc_ctor(3, 1, 0);
lean_ctor_set(x_636, 0, x_635);
if (lean_is_scalar(x_633)) {
 x_637 = lean_alloc_ctor(0, 2, 0);
} else {
 x_637 = x_633;
}
lean_ctor_set(x_637, 0, x_636);
lean_ctor_set(x_637, 1, x_632);
if (lean_is_scalar(x_630)) {
 x_638 = lean_alloc_ctor(1, 1, 0);
} else {
 x_638 = x_630;
}
lean_ctor_set(x_638, 0, x_637);
return x_638;
}
}
}
else
{
lean_object* x_639; 
x_639 = lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readUInt64LE(x_490);
if (lean_obj_tag(x_639) == 0)
{
lean_object* x_640; 
x_640 = lean_box(0);
return x_640;
}
else
{
lean_object* x_641; lean_object* x_642; lean_object* x_643; lean_object* x_644; lean_object* x_645; uint64_t x_646; lean_object* x_647; lean_object* x_648; lean_object* x_649; lean_object* x_650; 
x_641 = lean_ctor_get(x_639, 0);
lean_inc(x_641);
if (lean_is_exclusive(x_639)) {
 lean_ctor_release(x_639, 0);
 x_642 = x_639;
} else {
 lean_dec_ref(x_639);
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
x_646 = lean_unbox_uint64(x_643);
lean_dec(x_643);
x_647 = lean_uint64_to_nat(x_646);
x_648 = lean_alloc_ctor(2, 1, 0);
lean_ctor_set(x_648, 0, x_647);
if (lean_is_scalar(x_645)) {
 x_649 = lean_alloc_ctor(0, 2, 0);
} else {
 x_649 = x_645;
}
lean_ctor_set(x_649, 0, x_648);
lean_ctor_set(x_649, 1, x_644);
if (lean_is_scalar(x_642)) {
 x_650 = lean_alloc_ctor(1, 1, 0);
} else {
 x_650 = x_642;
}
lean_ctor_set(x_650, 0, x_649);
return x_650;
}
}
}
else
{
lean_object* x_651; lean_object* x_652; lean_object* x_653; 
x_651 = lean_box(1);
if (lean_is_scalar(x_491)) {
 x_652 = lean_alloc_ctor(0, 2, 0);
} else {
 x_652 = x_491;
}
lean_ctor_set(x_652, 0, x_651);
lean_ctor_set(x_652, 1, x_490);
x_653 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_653, 0, x_652);
return x_653;
}
}
else
{
lean_object* x_654; lean_object* x_655; lean_object* x_656; 
x_654 = lean_box(0);
if (lean_is_scalar(x_491)) {
 x_655 = lean_alloc_ctor(0, 2, 0);
} else {
 x_655 = x_491;
}
lean_ctor_set(x_655, 0, x_654);
lean_ctor_set(x_655, 1, x_490);
x_656 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_656, 0, x_655);
return x_656;
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
lean_object* initialize_Init(uint8_t builtin);
lean_object* initialize_iris_x2dkernel_IrisKernel_Types(uint8_t builtin);
lean_object* initialize_iris_x2dkernel_IrisKernel_Eval(uint8_t builtin);
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
lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readInt64LE___closed__0 = _init_lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readInt64LE___closed__0();
lean_mark_persistent(lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readInt64LE___closed__0);
lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readInt64LE___closed__1 = _init_lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readInt64LE___closed__1();
lean_mark_persistent(lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readInt64LE___closed__1);
lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readInt64LE___closed__2 = _init_lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readInt64LE___closed__2();
lean_mark_persistent(lp_iris_x2dkernel_IrisKernel_FFI_Cursor_readInt64LE___closed__2);
iris_lean_kernel_version = _init_iris_lean_kernel_version();
return lean_io_result_mk_ok(lean_box(0));
}
#ifdef __cplusplus
}
#endif
