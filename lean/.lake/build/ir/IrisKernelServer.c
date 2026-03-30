// Lean compiler output
// Module: IrisKernelServer
// Imports: public import Init public import IrisKernel.FFI
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
lean_object* lean_byte_array_copy_slice(lean_object*, lean_object*, lean_object*, lean_object*, lean_object*, uint8_t);
uint8_t lean_uint32_to_uint8(uint32_t);
LEAN_EXPORT uint32_t lp_iris_x2dkernel_readUInt32LE(lean_object*);
LEAN_EXPORT lean_object* _lean_main();
lean_object* iris_kernel_guard_rule(lean_object*);
lean_object* lean_uint32_to_nat(uint32_t);
uint8_t iris_check_cost_leq(lean_object*);
uint32_t lean_uint8_to_uint32(uint8_t);
uint32_t lean_uint32_shift_right(uint32_t, uint32_t);
lean_object* iris_kernel_intro(lean_object*);
lean_object* iris_kernel_congr(lean_object*);
lean_object* iris_kernel_match_elim(lean_object*);
static lean_object* lp_iris_x2dkernel_main___closed__3;
lean_object* iris_kernel_type_app(lean_object*);
lean_object* lean_byte_array_push(lean_object*, uint8_t);
LEAN_EXPORT lean_object* lp_iris_x2dkernel_main___boxed(lean_object*);
lean_object* iris_kernel_type_check_node_full(lean_object*);
lean_object* lean_get_stdout();
LEAN_EXPORT lean_object* lp_iris_x2dkernel___private_Init_While_0__Lean_Loop_forIn_loop___at___00readExact_spec__0___boxed(lean_object*, lean_object*, lean_object*, lean_object*);
size_t lean_usize_of_nat(lean_object*);
LEAN_EXPORT lean_object* lp_iris_x2dkernel_serverLoop___boxed(lean_object*, lean_object*, lean_object*);
static lean_object* lp_iris_x2dkernel_main___closed__0;
static lean_object* lp_iris_x2dkernel_dispatchRule___closed__0;
uint32_t lean_uint32_of_nat(lean_object*);
extern lean_object* l_ByteArray_empty;
lean_object* iris_kernel_structural_ind(lean_object*);
lean_object* iris_kernel_refl(lean_object*);
static lean_object* lp_iris_x2dkernel_main___closed__1;
lean_object* iris_kernel_let_bind(lean_object*);
lean_object* iris_kernel_nat_ind(lean_object*);
lean_object* lean_get_stdin();
lean_object* iris_kernel_refine_intro(lean_object*);
lean_object* iris_kernel_fold_rule(lean_object*);
LEAN_EXPORT lean_object* lp_iris_x2dkernel_readExact___boxed(lean_object*, lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_iris_x2dkernel_serverLoop(lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_iris_x2dkernel___private_Init_While_0__Lean_Loop_forIn_loop___at___00readExact_spec__0(lean_object*, lean_object*, lean_object*);
LEAN_EXPORT lean_object* lp_iris_x2dkernel_dispatchRule(uint8_t, lean_object*);
lean_object* iris_kernel_cost_leq_rule(lean_object*);
LEAN_EXPORT lean_object* lp_iris_x2dkernel_readUInt32LE___boxed(lean_object*);
uint8_t lean_nat_dec_eq(lean_object*, lean_object*);
uint8_t lean_nat_dec_lt(lean_object*, lean_object*);
lean_object* iris_kernel_elim(lean_object*);
lean_object* iris_kernel_symm(lean_object*);
lean_object* iris_kernel_cost_subsume(lean_object*);
LEAN_EXPORT lean_object* lp_iris_x2dkernel_readExact(lean_object*, lean_object*);
uint32_t lean_uint32_lor(uint32_t, uint32_t);
uint32_t lean_uint32_shift_left(uint32_t, uint32_t);
static lean_object* lp_iris_x2dkernel___private_Init_While_0__Lean_Loop_forIn_loop___at___00readExact_spec__0___closed__0;
lean_object* lean_nat_sub(lean_object*, lean_object*);
lean_object* iris_kernel_type_abst(lean_object*);
uint8_t lean_byte_array_get(lean_object*, lean_object*);
lean_object* lean_uint8_to_nat(uint8_t);
lean_object* iris_kernel_assume(lean_object*);
static lean_object* lp_iris_x2dkernel_main___closed__2;
LEAN_EXPORT lean_object* lp_iris_x2dkernel_writeUInt32LE(uint32_t);
lean_object* lp_iris_x2dkernel_IrisKernel_FFI_encodeFailure(uint8_t);
lean_object* iris_kernel_refine_elim(lean_object*);
lean_object* lean_byte_array_size(lean_object*);
LEAN_EXPORT lean_object* lp_iris_x2dkernel_writeUInt32LE___boxed(lean_object*);
lean_object* iris_kernel_trans(lean_object*);
uint8_t lean_uint8_dec_eq(uint8_t, uint8_t);
LEAN_EXPORT lean_object* lp_iris_x2dkernel_dispatchRule___boxed(lean_object*, lean_object*);
static lean_object* _init_lp_iris_x2dkernel___private_Init_While_0__Lean_Loop_forIn_loop___at___00readExact_spec__0___closed__0() {
_start:
{
lean_object* x_1; lean_object* x_2; 
x_1 = lean_box(0);
x_2 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_2, 0, x_1);
return x_2;
}
}
LEAN_EXPORT lean_object* lp_iris_x2dkernel___private_Init_While_0__Lean_Loop_forIn_loop___at___00readExact_spec__0(lean_object* x_1, lean_object* x_2, lean_object* x_3) {
_start:
{
uint8_t x_5; 
x_5 = !lean_is_exclusive(x_3);
if (x_5 == 0)
{
lean_object* x_6; lean_object* x_7; uint8_t x_8; 
x_6 = lean_ctor_get(x_3, 1);
x_7 = lean_ctor_get(x_3, 0);
lean_dec(x_7);
x_8 = !lean_is_exclusive(x_6);
if (x_8 == 0)
{
lean_object* x_9; lean_object* x_10; lean_object* x_11; uint8_t x_12; 
x_9 = lean_ctor_get(x_6, 0);
x_10 = lean_ctor_get(x_6, 1);
x_11 = lean_unsigned_to_nat(0u);
x_12 = lean_nat_dec_lt(x_11, x_10);
if (x_12 == 0)
{
lean_object* x_13; 
lean_dec_ref(x_2);
lean_ctor_set(x_3, 0, x_1);
x_13 = lean_alloc_ctor(0, 1, 0);
lean_ctor_set(x_13, 0, x_3);
return x_13;
}
else
{
lean_object* x_14; size_t x_15; lean_object* x_16; lean_object* x_17; 
x_14 = lean_ctor_get(x_2, 1);
x_15 = lean_usize_of_nat(x_10);
x_16 = lean_box_usize(x_15);
lean_inc_ref(x_14);
x_17 = lean_apply_2(x_14, x_16, lean_box(0));
if (lean_obj_tag(x_17) == 0)
{
uint8_t x_18; 
x_18 = !lean_is_exclusive(x_17);
if (x_18 == 0)
{
lean_object* x_19; lean_object* x_20; uint8_t x_21; 
x_19 = lean_ctor_get(x_17, 0);
x_20 = lean_byte_array_size(x_19);
x_21 = lean_nat_dec_eq(x_20, x_11);
if (x_21 == 0)
{
lean_object* x_22; lean_object* x_23; lean_object* x_24; 
lean_free_object(x_17);
x_22 = lean_byte_array_size(x_9);
x_23 = lean_byte_array_copy_slice(x_19, x_11, x_9, x_22, x_20, x_21);
lean_dec(x_19);
x_24 = lean_nat_sub(x_10, x_20);
lean_dec(x_10);
lean_ctor_set(x_6, 1, x_24);
lean_ctor_set(x_6, 0, x_23);
lean_inc(x_1);
lean_ctor_set(x_3, 0, x_1);
goto _start;
}
else
{
lean_object* x_26; 
lean_dec(x_19);
lean_dec_ref(x_2);
lean_dec(x_1);
x_26 = lp_iris_x2dkernel___private_Init_While_0__Lean_Loop_forIn_loop___at___00readExact_spec__0___closed__0;
lean_ctor_set(x_3, 0, x_26);
lean_ctor_set(x_17, 0, x_3);
return x_17;
}
}
else
{
lean_object* x_27; lean_object* x_28; uint8_t x_29; 
x_27 = lean_ctor_get(x_17, 0);
lean_inc(x_27);
lean_dec(x_17);
x_28 = lean_byte_array_size(x_27);
x_29 = lean_nat_dec_eq(x_28, x_11);
if (x_29 == 0)
{
lean_object* x_30; lean_object* x_31; lean_object* x_32; 
x_30 = lean_byte_array_size(x_9);
x_31 = lean_byte_array_copy_slice(x_27, x_11, x_9, x_30, x_28, x_29);
lean_dec(x_27);
x_32 = lean_nat_sub(x_10, x_28);
lean_dec(x_10);
lean_ctor_set(x_6, 1, x_32);
lean_ctor_set(x_6, 0, x_31);
lean_inc(x_1);
lean_ctor_set(x_3, 0, x_1);
goto _start;
}
else
{
lean_object* x_34; lean_object* x_35; 
lean_dec(x_27);
lean_dec_ref(x_2);
lean_dec(x_1);
x_34 = lp_iris_x2dkernel___private_Init_While_0__Lean_Loop_forIn_loop___at___00readExact_spec__0___closed__0;
lean_ctor_set(x_3, 0, x_34);
x_35 = lean_alloc_ctor(0, 1, 0);
lean_ctor_set(x_35, 0, x_3);
return x_35;
}
}
}
else
{
uint8_t x_36; 
lean_free_object(x_6);
lean_dec(x_10);
lean_dec(x_9);
lean_free_object(x_3);
lean_dec_ref(x_2);
lean_dec(x_1);
x_36 = !lean_is_exclusive(x_17);
if (x_36 == 0)
{
return x_17;
}
else
{
lean_object* x_37; lean_object* x_38; 
x_37 = lean_ctor_get(x_17, 0);
lean_inc(x_37);
lean_dec(x_17);
x_38 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_38, 0, x_37);
return x_38;
}
}
}
}
else
{
lean_object* x_39; lean_object* x_40; lean_object* x_41; uint8_t x_42; 
x_39 = lean_ctor_get(x_6, 0);
x_40 = lean_ctor_get(x_6, 1);
lean_inc(x_40);
lean_inc(x_39);
lean_dec(x_6);
x_41 = lean_unsigned_to_nat(0u);
x_42 = lean_nat_dec_lt(x_41, x_40);
if (x_42 == 0)
{
lean_object* x_43; lean_object* x_44; 
lean_dec_ref(x_2);
x_43 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_43, 0, x_39);
lean_ctor_set(x_43, 1, x_40);
lean_ctor_set(x_3, 1, x_43);
lean_ctor_set(x_3, 0, x_1);
x_44 = lean_alloc_ctor(0, 1, 0);
lean_ctor_set(x_44, 0, x_3);
return x_44;
}
else
{
lean_object* x_45; size_t x_46; lean_object* x_47; lean_object* x_48; 
x_45 = lean_ctor_get(x_2, 1);
x_46 = lean_usize_of_nat(x_40);
x_47 = lean_box_usize(x_46);
lean_inc_ref(x_45);
x_48 = lean_apply_2(x_45, x_47, lean_box(0));
if (lean_obj_tag(x_48) == 0)
{
lean_object* x_49; lean_object* x_50; lean_object* x_51; uint8_t x_52; 
x_49 = lean_ctor_get(x_48, 0);
lean_inc(x_49);
if (lean_is_exclusive(x_48)) {
 lean_ctor_release(x_48, 0);
 x_50 = x_48;
} else {
 lean_dec_ref(x_48);
 x_50 = lean_box(0);
}
x_51 = lean_byte_array_size(x_49);
x_52 = lean_nat_dec_eq(x_51, x_41);
if (x_52 == 0)
{
lean_object* x_53; lean_object* x_54; lean_object* x_55; lean_object* x_56; 
lean_dec(x_50);
x_53 = lean_byte_array_size(x_39);
x_54 = lean_byte_array_copy_slice(x_49, x_41, x_39, x_53, x_51, x_52);
lean_dec(x_49);
x_55 = lean_nat_sub(x_40, x_51);
lean_dec(x_40);
x_56 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_56, 0, x_54);
lean_ctor_set(x_56, 1, x_55);
lean_inc(x_1);
lean_ctor_set(x_3, 1, x_56);
lean_ctor_set(x_3, 0, x_1);
goto _start;
}
else
{
lean_object* x_58; lean_object* x_59; lean_object* x_60; 
lean_dec(x_49);
lean_dec_ref(x_2);
lean_dec(x_1);
x_58 = lp_iris_x2dkernel___private_Init_While_0__Lean_Loop_forIn_loop___at___00readExact_spec__0___closed__0;
x_59 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_59, 0, x_39);
lean_ctor_set(x_59, 1, x_40);
lean_ctor_set(x_3, 1, x_59);
lean_ctor_set(x_3, 0, x_58);
if (lean_is_scalar(x_50)) {
 x_60 = lean_alloc_ctor(0, 1, 0);
} else {
 x_60 = x_50;
}
lean_ctor_set(x_60, 0, x_3);
return x_60;
}
}
else
{
lean_object* x_61; lean_object* x_62; lean_object* x_63; 
lean_dec(x_40);
lean_dec(x_39);
lean_free_object(x_3);
lean_dec_ref(x_2);
lean_dec(x_1);
x_61 = lean_ctor_get(x_48, 0);
lean_inc(x_61);
if (lean_is_exclusive(x_48)) {
 lean_ctor_release(x_48, 0);
 x_62 = x_48;
} else {
 lean_dec_ref(x_48);
 x_62 = lean_box(0);
}
if (lean_is_scalar(x_62)) {
 x_63 = lean_alloc_ctor(1, 1, 0);
} else {
 x_63 = x_62;
}
lean_ctor_set(x_63, 0, x_61);
return x_63;
}
}
}
}
else
{
lean_object* x_64; lean_object* x_65; lean_object* x_66; lean_object* x_67; lean_object* x_68; uint8_t x_69; 
x_64 = lean_ctor_get(x_3, 1);
lean_inc(x_64);
lean_dec(x_3);
x_65 = lean_ctor_get(x_64, 0);
lean_inc(x_65);
x_66 = lean_ctor_get(x_64, 1);
lean_inc(x_66);
if (lean_is_exclusive(x_64)) {
 lean_ctor_release(x_64, 0);
 lean_ctor_release(x_64, 1);
 x_67 = x_64;
} else {
 lean_dec_ref(x_64);
 x_67 = lean_box(0);
}
x_68 = lean_unsigned_to_nat(0u);
x_69 = lean_nat_dec_lt(x_68, x_66);
if (x_69 == 0)
{
lean_object* x_70; lean_object* x_71; lean_object* x_72; 
lean_dec_ref(x_2);
if (lean_is_scalar(x_67)) {
 x_70 = lean_alloc_ctor(0, 2, 0);
} else {
 x_70 = x_67;
}
lean_ctor_set(x_70, 0, x_65);
lean_ctor_set(x_70, 1, x_66);
x_71 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_71, 0, x_1);
lean_ctor_set(x_71, 1, x_70);
x_72 = lean_alloc_ctor(0, 1, 0);
lean_ctor_set(x_72, 0, x_71);
return x_72;
}
else
{
lean_object* x_73; size_t x_74; lean_object* x_75; lean_object* x_76; 
x_73 = lean_ctor_get(x_2, 1);
x_74 = lean_usize_of_nat(x_66);
x_75 = lean_box_usize(x_74);
lean_inc_ref(x_73);
x_76 = lean_apply_2(x_73, x_75, lean_box(0));
if (lean_obj_tag(x_76) == 0)
{
lean_object* x_77; lean_object* x_78; lean_object* x_79; uint8_t x_80; 
x_77 = lean_ctor_get(x_76, 0);
lean_inc(x_77);
if (lean_is_exclusive(x_76)) {
 lean_ctor_release(x_76, 0);
 x_78 = x_76;
} else {
 lean_dec_ref(x_76);
 x_78 = lean_box(0);
}
x_79 = lean_byte_array_size(x_77);
x_80 = lean_nat_dec_eq(x_79, x_68);
if (x_80 == 0)
{
lean_object* x_81; lean_object* x_82; lean_object* x_83; lean_object* x_84; lean_object* x_85; 
lean_dec(x_78);
x_81 = lean_byte_array_size(x_65);
x_82 = lean_byte_array_copy_slice(x_77, x_68, x_65, x_81, x_79, x_80);
lean_dec(x_77);
x_83 = lean_nat_sub(x_66, x_79);
lean_dec(x_66);
if (lean_is_scalar(x_67)) {
 x_84 = lean_alloc_ctor(0, 2, 0);
} else {
 x_84 = x_67;
}
lean_ctor_set(x_84, 0, x_82);
lean_ctor_set(x_84, 1, x_83);
lean_inc(x_1);
x_85 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_85, 0, x_1);
lean_ctor_set(x_85, 1, x_84);
x_3 = x_85;
goto _start;
}
else
{
lean_object* x_87; lean_object* x_88; lean_object* x_89; lean_object* x_90; 
lean_dec(x_77);
lean_dec_ref(x_2);
lean_dec(x_1);
x_87 = lp_iris_x2dkernel___private_Init_While_0__Lean_Loop_forIn_loop___at___00readExact_spec__0___closed__0;
if (lean_is_scalar(x_67)) {
 x_88 = lean_alloc_ctor(0, 2, 0);
} else {
 x_88 = x_67;
}
lean_ctor_set(x_88, 0, x_65);
lean_ctor_set(x_88, 1, x_66);
x_89 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_89, 0, x_87);
lean_ctor_set(x_89, 1, x_88);
if (lean_is_scalar(x_78)) {
 x_90 = lean_alloc_ctor(0, 1, 0);
} else {
 x_90 = x_78;
}
lean_ctor_set(x_90, 0, x_89);
return x_90;
}
}
else
{
lean_object* x_91; lean_object* x_92; lean_object* x_93; 
lean_dec(x_67);
lean_dec(x_66);
lean_dec(x_65);
lean_dec_ref(x_2);
lean_dec(x_1);
x_91 = lean_ctor_get(x_76, 0);
lean_inc(x_91);
if (lean_is_exclusive(x_76)) {
 lean_ctor_release(x_76, 0);
 x_92 = x_76;
} else {
 lean_dec_ref(x_76);
 x_92 = lean_box(0);
}
if (lean_is_scalar(x_92)) {
 x_93 = lean_alloc_ctor(1, 1, 0);
} else {
 x_93 = x_92;
}
lean_ctor_set(x_93, 0, x_91);
return x_93;
}
}
}
}
}
LEAN_EXPORT lean_object* lp_iris_x2dkernel___private_Init_While_0__Lean_Loop_forIn_loop___at___00readExact_spec__0___boxed(lean_object* x_1, lean_object* x_2, lean_object* x_3, lean_object* x_4) {
_start:
{
lean_object* x_5; 
x_5 = lp_iris_x2dkernel___private_Init_While_0__Lean_Loop_forIn_loop___at___00readExact_spec__0(x_1, x_2, x_3);
return x_5;
}
}
LEAN_EXPORT lean_object* lp_iris_x2dkernel_readExact(lean_object* x_1, lean_object* x_2) {
_start:
{
lean_object* x_4; lean_object* x_5; lean_object* x_6; lean_object* x_7; lean_object* x_8; 
x_4 = l_ByteArray_empty;
x_5 = lean_box(0);
x_6 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_6, 0, x_4);
lean_ctor_set(x_6, 1, x_2);
x_7 = lean_alloc_ctor(0, 2, 0);
lean_ctor_set(x_7, 0, x_5);
lean_ctor_set(x_7, 1, x_6);
x_8 = lp_iris_x2dkernel___private_Init_While_0__Lean_Loop_forIn_loop___at___00readExact_spec__0(x_5, x_1, x_7);
if (lean_obj_tag(x_8) == 0)
{
uint8_t x_9; 
x_9 = !lean_is_exclusive(x_8);
if (x_9 == 0)
{
lean_object* x_10; lean_object* x_11; lean_object* x_12; 
x_10 = lean_ctor_get(x_8, 0);
x_11 = lean_ctor_get(x_10, 1);
lean_inc(x_11);
x_12 = lean_ctor_get(x_10, 0);
lean_inc(x_12);
lean_dec(x_10);
if (lean_obj_tag(x_12) == 0)
{
lean_object* x_13; lean_object* x_14; 
x_13 = lean_ctor_get(x_11, 0);
lean_inc(x_13);
lean_dec(x_11);
x_14 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_14, 0, x_13);
lean_ctor_set(x_8, 0, x_14);
return x_8;
}
else
{
lean_object* x_15; 
lean_dec(x_11);
x_15 = lean_ctor_get(x_12, 0);
lean_inc(x_15);
lean_dec_ref(x_12);
lean_ctor_set(x_8, 0, x_15);
return x_8;
}
}
else
{
lean_object* x_16; lean_object* x_17; lean_object* x_18; 
x_16 = lean_ctor_get(x_8, 0);
lean_inc(x_16);
lean_dec(x_8);
x_17 = lean_ctor_get(x_16, 1);
lean_inc(x_17);
x_18 = lean_ctor_get(x_16, 0);
lean_inc(x_18);
lean_dec(x_16);
if (lean_obj_tag(x_18) == 0)
{
lean_object* x_19; lean_object* x_20; lean_object* x_21; 
x_19 = lean_ctor_get(x_17, 0);
lean_inc(x_19);
lean_dec(x_17);
x_20 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_20, 0, x_19);
x_21 = lean_alloc_ctor(0, 1, 0);
lean_ctor_set(x_21, 0, x_20);
return x_21;
}
else
{
lean_object* x_22; lean_object* x_23; 
lean_dec(x_17);
x_22 = lean_ctor_get(x_18, 0);
lean_inc(x_22);
lean_dec_ref(x_18);
x_23 = lean_alloc_ctor(0, 1, 0);
lean_ctor_set(x_23, 0, x_22);
return x_23;
}
}
}
else
{
uint8_t x_24; 
x_24 = !lean_is_exclusive(x_8);
if (x_24 == 0)
{
return x_8;
}
else
{
lean_object* x_25; lean_object* x_26; 
x_25 = lean_ctor_get(x_8, 0);
lean_inc(x_25);
lean_dec(x_8);
x_26 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_26, 0, x_25);
return x_26;
}
}
}
}
LEAN_EXPORT lean_object* lp_iris_x2dkernel_readExact___boxed(lean_object* x_1, lean_object* x_2, lean_object* x_3) {
_start:
{
lean_object* x_4; 
x_4 = lp_iris_x2dkernel_readExact(x_1, x_2);
return x_4;
}
}
LEAN_EXPORT uint32_t lp_iris_x2dkernel_readUInt32LE(lean_object* x_1) {
_start:
{
lean_object* x_2; uint8_t x_3; lean_object* x_4; uint8_t x_5; lean_object* x_6; uint8_t x_7; lean_object* x_8; uint8_t x_9; uint32_t x_10; uint32_t x_11; uint32_t x_12; uint32_t x_13; uint32_t x_14; uint32_t x_15; uint32_t x_16; uint32_t x_17; uint32_t x_18; uint32_t x_19; uint32_t x_20; uint32_t x_21; uint32_t x_22; 
x_2 = lean_unsigned_to_nat(0u);
x_3 = lean_byte_array_get(x_1, x_2);
x_4 = lean_unsigned_to_nat(1u);
x_5 = lean_byte_array_get(x_1, x_4);
x_6 = lean_unsigned_to_nat(2u);
x_7 = lean_byte_array_get(x_1, x_6);
x_8 = lean_unsigned_to_nat(3u);
x_9 = lean_byte_array_get(x_1, x_8);
x_10 = lean_uint8_to_uint32(x_3);
x_11 = lean_uint8_to_uint32(x_5);
x_12 = 8;
x_13 = lean_uint32_shift_left(x_11, x_12);
x_14 = lean_uint32_lor(x_10, x_13);
x_15 = lean_uint8_to_uint32(x_7);
x_16 = 16;
x_17 = lean_uint32_shift_left(x_15, x_16);
x_18 = lean_uint32_lor(x_14, x_17);
x_19 = lean_uint8_to_uint32(x_9);
x_20 = 24;
x_21 = lean_uint32_shift_left(x_19, x_20);
x_22 = lean_uint32_lor(x_18, x_21);
return x_22;
}
}
LEAN_EXPORT lean_object* lp_iris_x2dkernel_readUInt32LE___boxed(lean_object* x_1) {
_start:
{
uint32_t x_2; lean_object* x_3; 
x_2 = lp_iris_x2dkernel_readUInt32LE(x_1);
lean_dec_ref(x_1);
x_3 = lean_box_uint32(x_2);
return x_3;
}
}
LEAN_EXPORT lean_object* lp_iris_x2dkernel_writeUInt32LE(uint32_t x_1) {
_start:
{
lean_object* x_2; uint8_t x_3; lean_object* x_4; uint32_t x_5; uint32_t x_6; uint8_t x_7; lean_object* x_8; uint32_t x_9; uint32_t x_10; uint8_t x_11; lean_object* x_12; uint32_t x_13; uint32_t x_14; uint8_t x_15; lean_object* x_16; 
x_2 = l_ByteArray_empty;
x_3 = lean_uint32_to_uint8(x_1);
x_4 = lean_byte_array_push(x_2, x_3);
x_5 = 8;
x_6 = lean_uint32_shift_right(x_1, x_5);
x_7 = lean_uint32_to_uint8(x_6);
x_8 = lean_byte_array_push(x_4, x_7);
x_9 = 16;
x_10 = lean_uint32_shift_right(x_1, x_9);
x_11 = lean_uint32_to_uint8(x_10);
x_12 = lean_byte_array_push(x_8, x_11);
x_13 = 24;
x_14 = lean_uint32_shift_right(x_1, x_13);
x_15 = lean_uint32_to_uint8(x_14);
x_16 = lean_byte_array_push(x_12, x_15);
return x_16;
}
}
LEAN_EXPORT lean_object* lp_iris_x2dkernel_writeUInt32LE___boxed(lean_object* x_1) {
_start:
{
uint32_t x_2; lean_object* x_3; 
x_2 = lean_unbox_uint32(x_1);
lean_dec(x_1);
x_3 = lp_iris_x2dkernel_writeUInt32LE(x_2);
return x_3;
}
}
static lean_object* _init_lp_iris_x2dkernel_dispatchRule___closed__0() {
_start:
{
uint8_t x_1; lean_object* x_2; 
x_1 = 99;
x_2 = lp_iris_x2dkernel_IrisKernel_FFI_encodeFailure(x_1);
return x_2;
}
}
LEAN_EXPORT lean_object* lp_iris_x2dkernel_dispatchRule(uint8_t x_1, lean_object* x_2) {
_start:
{
lean_object* x_3; lean_object* x_4; uint8_t x_5; 
x_3 = lean_uint8_to_nat(x_1);
x_4 = lean_unsigned_to_nat(0u);
x_5 = lean_nat_dec_eq(x_3, x_4);
if (x_5 == 0)
{
lean_object* x_6; uint8_t x_7; 
x_6 = lean_unsigned_to_nat(1u);
x_7 = lean_nat_dec_eq(x_3, x_6);
if (x_7 == 0)
{
lean_object* x_8; uint8_t x_9; 
x_8 = lean_unsigned_to_nat(2u);
x_9 = lean_nat_dec_eq(x_3, x_8);
if (x_9 == 0)
{
lean_object* x_10; uint8_t x_11; 
x_10 = lean_unsigned_to_nat(3u);
x_11 = lean_nat_dec_eq(x_3, x_10);
if (x_11 == 0)
{
lean_object* x_12; uint8_t x_13; 
x_12 = lean_unsigned_to_nat(4u);
x_13 = lean_nat_dec_eq(x_3, x_12);
if (x_13 == 0)
{
lean_object* x_14; uint8_t x_15; 
x_14 = lean_unsigned_to_nat(5u);
x_15 = lean_nat_dec_eq(x_3, x_14);
if (x_15 == 0)
{
lean_object* x_16; uint8_t x_17; 
x_16 = lean_unsigned_to_nat(6u);
x_17 = lean_nat_dec_eq(x_3, x_16);
if (x_17 == 0)
{
lean_object* x_18; uint8_t x_19; 
x_18 = lean_unsigned_to_nat(7u);
x_19 = lean_nat_dec_eq(x_3, x_18);
if (x_19 == 0)
{
lean_object* x_20; uint8_t x_21; 
x_20 = lean_unsigned_to_nat(8u);
x_21 = lean_nat_dec_eq(x_3, x_20);
if (x_21 == 0)
{
lean_object* x_22; uint8_t x_23; 
x_22 = lean_unsigned_to_nat(9u);
x_23 = lean_nat_dec_eq(x_3, x_22);
if (x_23 == 0)
{
lean_object* x_24; uint8_t x_25; 
x_24 = lean_unsigned_to_nat(10u);
x_25 = lean_nat_dec_eq(x_3, x_24);
if (x_25 == 0)
{
lean_object* x_26; uint8_t x_27; 
x_26 = lean_unsigned_to_nat(11u);
x_27 = lean_nat_dec_eq(x_3, x_26);
if (x_27 == 0)
{
lean_object* x_28; uint8_t x_29; 
x_28 = lean_unsigned_to_nat(12u);
x_29 = lean_nat_dec_eq(x_3, x_28);
if (x_29 == 0)
{
lean_object* x_30; uint8_t x_31; 
x_30 = lean_unsigned_to_nat(13u);
x_31 = lean_nat_dec_eq(x_3, x_30);
if (x_31 == 0)
{
lean_object* x_32; uint8_t x_33; 
x_32 = lean_unsigned_to_nat(14u);
x_33 = lean_nat_dec_eq(x_3, x_32);
if (x_33 == 0)
{
lean_object* x_34; uint8_t x_35; 
x_34 = lean_unsigned_to_nat(15u);
x_35 = lean_nat_dec_eq(x_3, x_34);
if (x_35 == 0)
{
lean_object* x_36; uint8_t x_37; 
x_36 = lean_unsigned_to_nat(16u);
x_37 = lean_nat_dec_eq(x_3, x_36);
if (x_37 == 0)
{
lean_object* x_38; uint8_t x_39; 
x_38 = lean_unsigned_to_nat(17u);
x_39 = lean_nat_dec_eq(x_3, x_38);
if (x_39 == 0)
{
lean_object* x_40; uint8_t x_41; 
x_40 = lean_unsigned_to_nat(18u);
x_41 = lean_nat_dec_eq(x_3, x_40);
if (x_41 == 0)
{
lean_object* x_42; uint8_t x_43; 
x_42 = lean_unsigned_to_nat(19u);
x_43 = lean_nat_dec_eq(x_3, x_42);
if (x_43 == 0)
{
lean_object* x_44; uint8_t x_45; 
x_44 = lean_unsigned_to_nat(20u);
x_45 = lean_nat_dec_eq(x_3, x_44);
if (x_45 == 0)
{
lean_object* x_46; 
lean_dec_ref(x_2);
x_46 = lp_iris_x2dkernel_dispatchRule___closed__0;
return x_46;
}
else
{
lean_object* x_47; 
x_47 = iris_kernel_guard_rule(x_2);
return x_47;
}
}
else
{
lean_object* x_48; 
x_48 = iris_kernel_type_app(x_2);
return x_48;
}
}
else
{
lean_object* x_49; 
x_49 = iris_kernel_type_abst(x_2);
return x_49;
}
}
else
{
lean_object* x_50; 
x_50 = iris_kernel_fold_rule(x_2);
return x_50;
}
}
else
{
lean_object* x_51; 
x_51 = iris_kernel_match_elim(x_2);
return x_51;
}
}
else
{
lean_object* x_52; 
x_52 = iris_kernel_let_bind(x_2);
return x_52;
}
}
else
{
lean_object* x_53; 
x_53 = iris_kernel_structural_ind(x_2);
return x_53;
}
}
else
{
lean_object* x_54; 
x_54 = iris_kernel_nat_ind(x_2);
return x_54;
}
}
else
{
lean_object* x_55; 
x_55 = iris_kernel_refine_elim(x_2);
return x_55;
}
}
else
{
lean_object* x_56; 
x_56 = iris_kernel_refine_intro(x_2);
return x_56;
}
}
else
{
lean_object* x_57; 
x_57 = iris_kernel_cost_leq_rule(x_2);
return x_57;
}
}
else
{
lean_object* x_58; 
x_58 = iris_kernel_cost_subsume(x_2);
return x_58;
}
}
else
{
lean_object* x_59; 
x_59 = iris_kernel_type_check_node_full(x_2);
return x_59;
}
}
else
{
lean_object* x_60; 
x_60 = iris_kernel_congr(x_2);
return x_60;
}
}
else
{
lean_object* x_61; 
x_61 = iris_kernel_trans(x_2);
return x_61;
}
}
else
{
lean_object* x_62; 
x_62 = iris_kernel_symm(x_2);
return x_62;
}
}
else
{
lean_object* x_63; 
x_63 = iris_kernel_refl(x_2);
return x_63;
}
}
else
{
lean_object* x_64; 
x_64 = iris_kernel_elim(x_2);
return x_64;
}
}
else
{
lean_object* x_65; 
x_65 = iris_kernel_intro(x_2);
return x_65;
}
}
else
{
lean_object* x_66; 
x_66 = iris_kernel_assume(x_2);
return x_66;
}
}
else
{
uint8_t x_67; lean_object* x_68; lean_object* x_69; 
x_67 = iris_check_cost_leq(x_2);
x_68 = l_ByteArray_empty;
x_69 = lean_byte_array_push(x_68, x_67);
return x_69;
}
}
}
LEAN_EXPORT lean_object* lp_iris_x2dkernel_dispatchRule___boxed(lean_object* x_1, lean_object* x_2) {
_start:
{
uint8_t x_3; lean_object* x_4; 
x_3 = lean_unbox(x_1);
x_4 = lp_iris_x2dkernel_dispatchRule(x_3, x_2);
return x_4;
}
}
LEAN_EXPORT lean_object* lp_iris_x2dkernel_serverLoop(lean_object* x_1, lean_object* x_2) {
_start:
{
lean_object* x_4; lean_object* x_5; 
x_4 = lean_unsigned_to_nat(1u);
lean_inc_ref(x_1);
x_5 = lp_iris_x2dkernel_readExact(x_1, x_4);
if (lean_obj_tag(x_5) == 0)
{
uint8_t x_6; 
x_6 = !lean_is_exclusive(x_5);
if (x_6 == 0)
{
lean_object* x_7; 
x_7 = lean_ctor_get(x_5, 0);
if (lean_obj_tag(x_7) == 1)
{
lean_object* x_8; lean_object* x_9; uint8_t x_10; uint8_t x_11; uint8_t x_12; 
x_8 = lean_ctor_get(x_7, 0);
lean_inc(x_8);
lean_dec_ref(x_7);
x_9 = lean_unsigned_to_nat(0u);
x_10 = lean_byte_array_get(x_8, x_9);
lean_dec(x_8);
x_11 = 255;
x_12 = lean_uint8_dec_eq(x_10, x_11);
if (x_12 == 0)
{
lean_object* x_13; lean_object* x_14; 
lean_free_object(x_5);
x_13 = lean_unsigned_to_nat(4u);
lean_inc_ref(x_1);
x_14 = lp_iris_x2dkernel_readExact(x_1, x_13);
if (lean_obj_tag(x_14) == 0)
{
uint8_t x_15; 
x_15 = !lean_is_exclusive(x_14);
if (x_15 == 0)
{
lean_object* x_16; 
x_16 = lean_ctor_get(x_14, 0);
if (lean_obj_tag(x_16) == 1)
{
lean_object* x_17; uint32_t x_18; lean_object* x_19; lean_object* x_20; 
lean_free_object(x_14);
x_17 = lean_ctor_get(x_16, 0);
lean_inc(x_17);
lean_dec_ref(x_16);
x_18 = lp_iris_x2dkernel_readUInt32LE(x_17);
lean_dec(x_17);
x_19 = lean_uint32_to_nat(x_18);
lean_inc_ref(x_1);
x_20 = lp_iris_x2dkernel_readExact(x_1, x_19);
if (lean_obj_tag(x_20) == 0)
{
uint8_t x_21; 
x_21 = !lean_is_exclusive(x_20);
if (x_21 == 0)
{
lean_object* x_22; 
x_22 = lean_ctor_get(x_20, 0);
if (lean_obj_tag(x_22) == 1)
{
lean_object* x_23; lean_object* x_24; lean_object* x_25; lean_object* x_26; lean_object* x_27; uint32_t x_28; lean_object* x_29; lean_object* x_30; 
lean_free_object(x_20);
x_23 = lean_ctor_get(x_22, 0);
lean_inc(x_23);
lean_dec_ref(x_22);
x_24 = lean_ctor_get(x_2, 0);
x_25 = lean_ctor_get(x_2, 2);
x_26 = lp_iris_x2dkernel_dispatchRule(x_10, x_23);
x_27 = lean_byte_array_size(x_26);
x_28 = lean_uint32_of_nat(x_27);
x_29 = lp_iris_x2dkernel_writeUInt32LE(x_28);
lean_inc_ref(x_25);
x_30 = lean_apply_2(x_25, x_29, lean_box(0));
if (lean_obj_tag(x_30) == 0)
{
lean_object* x_31; 
lean_dec_ref(x_30);
lean_inc_ref(x_25);
x_31 = lean_apply_2(x_25, x_26, lean_box(0));
if (lean_obj_tag(x_31) == 0)
{
lean_object* x_32; 
lean_dec_ref(x_31);
lean_inc_ref(x_24);
x_32 = lean_apply_1(x_24, lean_box(0));
if (lean_obj_tag(x_32) == 0)
{
lean_dec_ref(x_32);
goto _start;
}
else
{
lean_dec_ref(x_2);
lean_dec_ref(x_1);
return x_32;
}
}
else
{
lean_dec_ref(x_2);
lean_dec_ref(x_1);
return x_31;
}
}
else
{
lean_dec_ref(x_26);
lean_dec_ref(x_2);
lean_dec_ref(x_1);
return x_30;
}
}
else
{
lean_object* x_34; 
lean_dec(x_22);
lean_dec_ref(x_2);
lean_dec_ref(x_1);
x_34 = lean_box(0);
lean_ctor_set(x_20, 0, x_34);
return x_20;
}
}
else
{
lean_object* x_35; 
x_35 = lean_ctor_get(x_20, 0);
lean_inc(x_35);
lean_dec(x_20);
if (lean_obj_tag(x_35) == 1)
{
lean_object* x_36; lean_object* x_37; lean_object* x_38; lean_object* x_39; lean_object* x_40; uint32_t x_41; lean_object* x_42; lean_object* x_43; 
x_36 = lean_ctor_get(x_35, 0);
lean_inc(x_36);
lean_dec_ref(x_35);
x_37 = lean_ctor_get(x_2, 0);
x_38 = lean_ctor_get(x_2, 2);
x_39 = lp_iris_x2dkernel_dispatchRule(x_10, x_36);
x_40 = lean_byte_array_size(x_39);
x_41 = lean_uint32_of_nat(x_40);
x_42 = lp_iris_x2dkernel_writeUInt32LE(x_41);
lean_inc_ref(x_38);
x_43 = lean_apply_2(x_38, x_42, lean_box(0));
if (lean_obj_tag(x_43) == 0)
{
lean_object* x_44; 
lean_dec_ref(x_43);
lean_inc_ref(x_38);
x_44 = lean_apply_2(x_38, x_39, lean_box(0));
if (lean_obj_tag(x_44) == 0)
{
lean_object* x_45; 
lean_dec_ref(x_44);
lean_inc_ref(x_37);
x_45 = lean_apply_1(x_37, lean_box(0));
if (lean_obj_tag(x_45) == 0)
{
lean_dec_ref(x_45);
goto _start;
}
else
{
lean_dec_ref(x_2);
lean_dec_ref(x_1);
return x_45;
}
}
else
{
lean_dec_ref(x_2);
lean_dec_ref(x_1);
return x_44;
}
}
else
{
lean_dec_ref(x_39);
lean_dec_ref(x_2);
lean_dec_ref(x_1);
return x_43;
}
}
else
{
lean_object* x_47; lean_object* x_48; 
lean_dec(x_35);
lean_dec_ref(x_2);
lean_dec_ref(x_1);
x_47 = lean_box(0);
x_48 = lean_alloc_ctor(0, 1, 0);
lean_ctor_set(x_48, 0, x_47);
return x_48;
}
}
}
else
{
uint8_t x_49; 
lean_dec_ref(x_2);
lean_dec_ref(x_1);
x_49 = !lean_is_exclusive(x_20);
if (x_49 == 0)
{
return x_20;
}
else
{
lean_object* x_50; lean_object* x_51; 
x_50 = lean_ctor_get(x_20, 0);
lean_inc(x_50);
lean_dec(x_20);
x_51 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_51, 0, x_50);
return x_51;
}
}
}
else
{
lean_object* x_52; 
lean_dec(x_16);
lean_dec_ref(x_2);
lean_dec_ref(x_1);
x_52 = lean_box(0);
lean_ctor_set(x_14, 0, x_52);
return x_14;
}
}
else
{
lean_object* x_53; 
x_53 = lean_ctor_get(x_14, 0);
lean_inc(x_53);
lean_dec(x_14);
if (lean_obj_tag(x_53) == 1)
{
lean_object* x_54; uint32_t x_55; lean_object* x_56; lean_object* x_57; 
x_54 = lean_ctor_get(x_53, 0);
lean_inc(x_54);
lean_dec_ref(x_53);
x_55 = lp_iris_x2dkernel_readUInt32LE(x_54);
lean_dec(x_54);
x_56 = lean_uint32_to_nat(x_55);
lean_inc_ref(x_1);
x_57 = lp_iris_x2dkernel_readExact(x_1, x_56);
if (lean_obj_tag(x_57) == 0)
{
lean_object* x_58; lean_object* x_59; 
x_58 = lean_ctor_get(x_57, 0);
lean_inc(x_58);
if (lean_is_exclusive(x_57)) {
 lean_ctor_release(x_57, 0);
 x_59 = x_57;
} else {
 lean_dec_ref(x_57);
 x_59 = lean_box(0);
}
if (lean_obj_tag(x_58) == 1)
{
lean_object* x_60; lean_object* x_61; lean_object* x_62; lean_object* x_63; lean_object* x_64; uint32_t x_65; lean_object* x_66; lean_object* x_67; 
lean_dec(x_59);
x_60 = lean_ctor_get(x_58, 0);
lean_inc(x_60);
lean_dec_ref(x_58);
x_61 = lean_ctor_get(x_2, 0);
x_62 = lean_ctor_get(x_2, 2);
x_63 = lp_iris_x2dkernel_dispatchRule(x_10, x_60);
x_64 = lean_byte_array_size(x_63);
x_65 = lean_uint32_of_nat(x_64);
x_66 = lp_iris_x2dkernel_writeUInt32LE(x_65);
lean_inc_ref(x_62);
x_67 = lean_apply_2(x_62, x_66, lean_box(0));
if (lean_obj_tag(x_67) == 0)
{
lean_object* x_68; 
lean_dec_ref(x_67);
lean_inc_ref(x_62);
x_68 = lean_apply_2(x_62, x_63, lean_box(0));
if (lean_obj_tag(x_68) == 0)
{
lean_object* x_69; 
lean_dec_ref(x_68);
lean_inc_ref(x_61);
x_69 = lean_apply_1(x_61, lean_box(0));
if (lean_obj_tag(x_69) == 0)
{
lean_dec_ref(x_69);
goto _start;
}
else
{
lean_dec_ref(x_2);
lean_dec_ref(x_1);
return x_69;
}
}
else
{
lean_dec_ref(x_2);
lean_dec_ref(x_1);
return x_68;
}
}
else
{
lean_dec_ref(x_63);
lean_dec_ref(x_2);
lean_dec_ref(x_1);
return x_67;
}
}
else
{
lean_object* x_71; lean_object* x_72; 
lean_dec(x_58);
lean_dec_ref(x_2);
lean_dec_ref(x_1);
x_71 = lean_box(0);
if (lean_is_scalar(x_59)) {
 x_72 = lean_alloc_ctor(0, 1, 0);
} else {
 x_72 = x_59;
}
lean_ctor_set(x_72, 0, x_71);
return x_72;
}
}
else
{
lean_object* x_73; lean_object* x_74; lean_object* x_75; 
lean_dec_ref(x_2);
lean_dec_ref(x_1);
x_73 = lean_ctor_get(x_57, 0);
lean_inc(x_73);
if (lean_is_exclusive(x_57)) {
 lean_ctor_release(x_57, 0);
 x_74 = x_57;
} else {
 lean_dec_ref(x_57);
 x_74 = lean_box(0);
}
if (lean_is_scalar(x_74)) {
 x_75 = lean_alloc_ctor(1, 1, 0);
} else {
 x_75 = x_74;
}
lean_ctor_set(x_75, 0, x_73);
return x_75;
}
}
else
{
lean_object* x_76; lean_object* x_77; 
lean_dec(x_53);
lean_dec_ref(x_2);
lean_dec_ref(x_1);
x_76 = lean_box(0);
x_77 = lean_alloc_ctor(0, 1, 0);
lean_ctor_set(x_77, 0, x_76);
return x_77;
}
}
}
else
{
uint8_t x_78; 
lean_dec_ref(x_2);
lean_dec_ref(x_1);
x_78 = !lean_is_exclusive(x_14);
if (x_78 == 0)
{
return x_14;
}
else
{
lean_object* x_79; lean_object* x_80; 
x_79 = lean_ctor_get(x_14, 0);
lean_inc(x_79);
lean_dec(x_14);
x_80 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_80, 0, x_79);
return x_80;
}
}
}
else
{
lean_object* x_81; 
lean_dec_ref(x_2);
lean_dec_ref(x_1);
x_81 = lean_box(0);
lean_ctor_set(x_5, 0, x_81);
return x_5;
}
}
else
{
lean_object* x_82; 
lean_dec(x_7);
lean_dec_ref(x_2);
lean_dec_ref(x_1);
x_82 = lean_box(0);
lean_ctor_set(x_5, 0, x_82);
return x_5;
}
}
else
{
lean_object* x_83; 
x_83 = lean_ctor_get(x_5, 0);
lean_inc(x_83);
lean_dec(x_5);
if (lean_obj_tag(x_83) == 1)
{
lean_object* x_84; lean_object* x_85; uint8_t x_86; uint8_t x_87; uint8_t x_88; 
x_84 = lean_ctor_get(x_83, 0);
lean_inc(x_84);
lean_dec_ref(x_83);
x_85 = lean_unsigned_to_nat(0u);
x_86 = lean_byte_array_get(x_84, x_85);
lean_dec(x_84);
x_87 = 255;
x_88 = lean_uint8_dec_eq(x_86, x_87);
if (x_88 == 0)
{
lean_object* x_89; lean_object* x_90; 
x_89 = lean_unsigned_to_nat(4u);
lean_inc_ref(x_1);
x_90 = lp_iris_x2dkernel_readExact(x_1, x_89);
if (lean_obj_tag(x_90) == 0)
{
lean_object* x_91; lean_object* x_92; 
x_91 = lean_ctor_get(x_90, 0);
lean_inc(x_91);
if (lean_is_exclusive(x_90)) {
 lean_ctor_release(x_90, 0);
 x_92 = x_90;
} else {
 lean_dec_ref(x_90);
 x_92 = lean_box(0);
}
if (lean_obj_tag(x_91) == 1)
{
lean_object* x_93; uint32_t x_94; lean_object* x_95; lean_object* x_96; 
lean_dec(x_92);
x_93 = lean_ctor_get(x_91, 0);
lean_inc(x_93);
lean_dec_ref(x_91);
x_94 = lp_iris_x2dkernel_readUInt32LE(x_93);
lean_dec(x_93);
x_95 = lean_uint32_to_nat(x_94);
lean_inc_ref(x_1);
x_96 = lp_iris_x2dkernel_readExact(x_1, x_95);
if (lean_obj_tag(x_96) == 0)
{
lean_object* x_97; lean_object* x_98; 
x_97 = lean_ctor_get(x_96, 0);
lean_inc(x_97);
if (lean_is_exclusive(x_96)) {
 lean_ctor_release(x_96, 0);
 x_98 = x_96;
} else {
 lean_dec_ref(x_96);
 x_98 = lean_box(0);
}
if (lean_obj_tag(x_97) == 1)
{
lean_object* x_99; lean_object* x_100; lean_object* x_101; lean_object* x_102; lean_object* x_103; uint32_t x_104; lean_object* x_105; lean_object* x_106; 
lean_dec(x_98);
x_99 = lean_ctor_get(x_97, 0);
lean_inc(x_99);
lean_dec_ref(x_97);
x_100 = lean_ctor_get(x_2, 0);
x_101 = lean_ctor_get(x_2, 2);
x_102 = lp_iris_x2dkernel_dispatchRule(x_86, x_99);
x_103 = lean_byte_array_size(x_102);
x_104 = lean_uint32_of_nat(x_103);
x_105 = lp_iris_x2dkernel_writeUInt32LE(x_104);
lean_inc_ref(x_101);
x_106 = lean_apply_2(x_101, x_105, lean_box(0));
if (lean_obj_tag(x_106) == 0)
{
lean_object* x_107; 
lean_dec_ref(x_106);
lean_inc_ref(x_101);
x_107 = lean_apply_2(x_101, x_102, lean_box(0));
if (lean_obj_tag(x_107) == 0)
{
lean_object* x_108; 
lean_dec_ref(x_107);
lean_inc_ref(x_100);
x_108 = lean_apply_1(x_100, lean_box(0));
if (lean_obj_tag(x_108) == 0)
{
lean_dec_ref(x_108);
goto _start;
}
else
{
lean_dec_ref(x_2);
lean_dec_ref(x_1);
return x_108;
}
}
else
{
lean_dec_ref(x_2);
lean_dec_ref(x_1);
return x_107;
}
}
else
{
lean_dec_ref(x_102);
lean_dec_ref(x_2);
lean_dec_ref(x_1);
return x_106;
}
}
else
{
lean_object* x_110; lean_object* x_111; 
lean_dec(x_97);
lean_dec_ref(x_2);
lean_dec_ref(x_1);
x_110 = lean_box(0);
if (lean_is_scalar(x_98)) {
 x_111 = lean_alloc_ctor(0, 1, 0);
} else {
 x_111 = x_98;
}
lean_ctor_set(x_111, 0, x_110);
return x_111;
}
}
else
{
lean_object* x_112; lean_object* x_113; lean_object* x_114; 
lean_dec_ref(x_2);
lean_dec_ref(x_1);
x_112 = lean_ctor_get(x_96, 0);
lean_inc(x_112);
if (lean_is_exclusive(x_96)) {
 lean_ctor_release(x_96, 0);
 x_113 = x_96;
} else {
 lean_dec_ref(x_96);
 x_113 = lean_box(0);
}
if (lean_is_scalar(x_113)) {
 x_114 = lean_alloc_ctor(1, 1, 0);
} else {
 x_114 = x_113;
}
lean_ctor_set(x_114, 0, x_112);
return x_114;
}
}
else
{
lean_object* x_115; lean_object* x_116; 
lean_dec(x_91);
lean_dec_ref(x_2);
lean_dec_ref(x_1);
x_115 = lean_box(0);
if (lean_is_scalar(x_92)) {
 x_116 = lean_alloc_ctor(0, 1, 0);
} else {
 x_116 = x_92;
}
lean_ctor_set(x_116, 0, x_115);
return x_116;
}
}
else
{
lean_object* x_117; lean_object* x_118; lean_object* x_119; 
lean_dec_ref(x_2);
lean_dec_ref(x_1);
x_117 = lean_ctor_get(x_90, 0);
lean_inc(x_117);
if (lean_is_exclusive(x_90)) {
 lean_ctor_release(x_90, 0);
 x_118 = x_90;
} else {
 lean_dec_ref(x_90);
 x_118 = lean_box(0);
}
if (lean_is_scalar(x_118)) {
 x_119 = lean_alloc_ctor(1, 1, 0);
} else {
 x_119 = x_118;
}
lean_ctor_set(x_119, 0, x_117);
return x_119;
}
}
else
{
lean_object* x_120; lean_object* x_121; 
lean_dec_ref(x_2);
lean_dec_ref(x_1);
x_120 = lean_box(0);
x_121 = lean_alloc_ctor(0, 1, 0);
lean_ctor_set(x_121, 0, x_120);
return x_121;
}
}
else
{
lean_object* x_122; lean_object* x_123; 
lean_dec(x_83);
lean_dec_ref(x_2);
lean_dec_ref(x_1);
x_122 = lean_box(0);
x_123 = lean_alloc_ctor(0, 1, 0);
lean_ctor_set(x_123, 0, x_122);
return x_123;
}
}
}
else
{
uint8_t x_124; 
lean_dec_ref(x_2);
lean_dec_ref(x_1);
x_124 = !lean_is_exclusive(x_5);
if (x_124 == 0)
{
return x_5;
}
else
{
lean_object* x_125; lean_object* x_126; 
x_125 = lean_ctor_get(x_5, 0);
lean_inc(x_125);
lean_dec(x_5);
x_126 = lean_alloc_ctor(1, 1, 0);
lean_ctor_set(x_126, 0, x_125);
return x_126;
}
}
}
}
LEAN_EXPORT lean_object* lp_iris_x2dkernel_serverLoop___boxed(lean_object* x_1, lean_object* x_2, lean_object* x_3) {
_start:
{
lean_object* x_4; 
x_4 = lp_iris_x2dkernel_serverLoop(x_1, x_2);
return x_4;
}
}
static lean_object* _init_lp_iris_x2dkernel_main___closed__0() {
_start:
{
uint8_t x_1; lean_object* x_2; lean_object* x_3; 
x_1 = 73;
x_2 = l_ByteArray_empty;
x_3 = lean_byte_array_push(x_2, x_1);
return x_3;
}
}
static lean_object* _init_lp_iris_x2dkernel_main___closed__1() {
_start:
{
uint8_t x_1; lean_object* x_2; lean_object* x_3; 
x_1 = 82;
x_2 = lp_iris_x2dkernel_main___closed__0;
x_3 = lean_byte_array_push(x_2, x_1);
return x_3;
}
}
static lean_object* _init_lp_iris_x2dkernel_main___closed__2() {
_start:
{
uint8_t x_1; lean_object* x_2; lean_object* x_3; 
x_1 = 73;
x_2 = lp_iris_x2dkernel_main___closed__1;
x_3 = lean_byte_array_push(x_2, x_1);
return x_3;
}
}
static lean_object* _init_lp_iris_x2dkernel_main___closed__3() {
_start:
{
uint8_t x_1; lean_object* x_2; lean_object* x_3; 
x_1 = 83;
x_2 = lp_iris_x2dkernel_main___closed__2;
x_3 = lean_byte_array_push(x_2, x_1);
return x_3;
}
}
LEAN_EXPORT lean_object* _lean_main() {
_start:
{
lean_object* x_2; lean_object* x_3; lean_object* x_4; lean_object* x_5; lean_object* x_6; lean_object* x_7; 
x_2 = lean_get_stdin();
x_3 = lean_get_stdout();
x_4 = lean_ctor_get(x_3, 0);
lean_inc_ref(x_4);
x_5 = lean_ctor_get(x_3, 2);
lean_inc_ref(x_5);
x_6 = lp_iris_x2dkernel_main___closed__3;
x_7 = lean_apply_2(x_5, x_6, lean_box(0));
if (lean_obj_tag(x_7) == 0)
{
lean_object* x_8; 
lean_dec_ref(x_7);
x_8 = lean_apply_1(x_4, lean_box(0));
if (lean_obj_tag(x_8) == 0)
{
lean_object* x_9; 
lean_dec_ref(x_8);
x_9 = lp_iris_x2dkernel_serverLoop(x_2, x_3);
return x_9;
}
else
{
lean_dec_ref(x_3);
lean_dec_ref(x_2);
return x_8;
}
}
else
{
lean_dec_ref(x_4);
lean_dec_ref(x_3);
lean_dec_ref(x_2);
return x_7;
}
}
}
LEAN_EXPORT lean_object* lp_iris_x2dkernel_main___boxed(lean_object* x_1) {
_start:
{
lean_object* x_2; 
x_2 = _lean_main();
return x_2;
}
}
lean_object* initialize_Init(uint8_t builtin);
lean_object* initialize_iris_x2dkernel_IrisKernel_FFI(uint8_t builtin);
static bool _G_initialized = false;
LEAN_EXPORT lean_object* initialize_iris_x2dkernel_IrisKernelServer(uint8_t builtin) {
lean_object * res;
if (_G_initialized) return lean_io_result_mk_ok(lean_box(0));
_G_initialized = true;
res = initialize_Init(builtin);
if (lean_io_result_is_error(res)) return res;
lean_dec_ref(res);
res = initialize_iris_x2dkernel_IrisKernel_FFI(builtin);
if (lean_io_result_is_error(res)) return res;
lean_dec_ref(res);
lp_iris_x2dkernel___private_Init_While_0__Lean_Loop_forIn_loop___at___00readExact_spec__0___closed__0 = _init_lp_iris_x2dkernel___private_Init_While_0__Lean_Loop_forIn_loop___at___00readExact_spec__0___closed__0();
lean_mark_persistent(lp_iris_x2dkernel___private_Init_While_0__Lean_Loop_forIn_loop___at___00readExact_spec__0___closed__0);
lp_iris_x2dkernel_dispatchRule___closed__0 = _init_lp_iris_x2dkernel_dispatchRule___closed__0();
lean_mark_persistent(lp_iris_x2dkernel_dispatchRule___closed__0);
lp_iris_x2dkernel_main___closed__0 = _init_lp_iris_x2dkernel_main___closed__0();
lean_mark_persistent(lp_iris_x2dkernel_main___closed__0);
lp_iris_x2dkernel_main___closed__1 = _init_lp_iris_x2dkernel_main___closed__1();
lean_mark_persistent(lp_iris_x2dkernel_main___closed__1);
lp_iris_x2dkernel_main___closed__2 = _init_lp_iris_x2dkernel_main___closed__2();
lean_mark_persistent(lp_iris_x2dkernel_main___closed__2);
lp_iris_x2dkernel_main___closed__3 = _init_lp_iris_x2dkernel_main___closed__3();
lean_mark_persistent(lp_iris_x2dkernel_main___closed__3);
return lean_io_result_mk_ok(lean_box(0));
}
char ** lean_setup_args(int argc, char ** argv);
void lean_initialize_runtime_module();

  #if defined(WIN32) || defined(_WIN32)
  #include <windows.h>
  #endif

  int main(int argc, char ** argv) {
  #if defined(WIN32) || defined(_WIN32)
  SetErrorMode(SEM_FAILCRITICALERRORS);
  SetConsoleOutputCP(CP_UTF8);
  #endif
  lean_object* in; lean_object* res;
argv = lean_setup_args(argc, argv);
lean_initialize_runtime_module();
lean_set_panic_messages(false);
res = initialize_iris_x2dkernel_IrisKernelServer(1 /* builtin */);
lean_set_panic_messages(true);
lean_io_mark_end_initialization();
if (lean_io_result_is_ok(res)) {
lean_dec_ref(res);
lean_init_task_manager();
res = _lean_main();
}
lean_finalize_task_manager();
if (lean_io_result_is_ok(res)) {
  int ret = 0;
  lean_dec_ref(res);
  return ret;
} else {
  lean_io_result_show_error(res);
  lean_dec_ref(res);
  return 1;
}
}
#ifdef __cplusplus
}
#endif
