/* Generated from bootstrap/interpreter.json */
/* Do not edit — regenerate with iris-c-codegen */

#include "iris_rt.h"

iris_value_t *iris_interpret(iris_value_t *program, iris_value_t *inputs) {
    /* Node 6397819577665799201 (Guard) */
    if (iris_is_truthy(iris_eq(iris_graph_get_kind(program, iris_graph_get_root(program)), iris_int(0)))) {
        /* Node 7348181143289350482 (Guard) */
        if (iris_is_truthy(iris_gt(iris_list_len(iris_graph_outgoing(program, iris_graph_get_root(program))), iris_int(1)))) {
            /* Node 15438238822594501785 (Guard) */
            if (iris_is_truthy(iris_lt(iris_graph_get_prim_op(program, iris_graph_get_root(program)), iris_int(10)))) {
                /* Node 847406437522256212 (Guard) */
                if (iris_is_truthy(iris_eq(iris_graph_get_prim_op(program, iris_graph_get_root(program)), iris_int(0)))) {
                    return iris_add(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs), iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs));
                } else {
                    /* Node 5254895529895305184 (Guard) */
                    if (iris_is_truthy(iris_eq(iris_graph_get_prim_op(program, iris_graph_get_root(program)), iris_int(1)))) {
                        return iris_sub(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs), iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs));
                    } else {
                        /* Node 759622312822243467 (Guard) */
                        if (iris_is_truthy(iris_eq(iris_graph_get_prim_op(program, iris_graph_get_root(program)), iris_int(2)))) {
                            return iris_mul(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs), iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs));
                        } else {
                            /* Node 14219772053884723068 (Guard) */
                            if (iris_is_truthy(iris_eq(iris_graph_get_prim_op(program, iris_graph_get_root(program)), iris_int(3)))) {
                                /* Node 6678574945854958501 (Guard) */
                                if (iris_is_truthy(iris_eq(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_int(0)))) {
                                    return iris_int(0);
                                } else {
                                    return iris_div(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs), iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs));
                                }
                            } else {
                                /* Node 6581287502860085499 (Guard) */
                                if (iris_is_truthy(iris_eq(iris_graph_get_prim_op(program, iris_graph_get_root(program)), iris_int(4)))) {
                                    /* Node 667720204153256056 (Guard) */
                                    if (iris_is_truthy(iris_eq(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_int(0)))) {
                                        return iris_int(0);
                                    } else {
                                        return iris_sub(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs), iris_mul(iris_div(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs), iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs)), iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs)));
                                    }
                                } else {
                                    /* Node 37701106729541718 (Guard) */
                                    if (iris_is_truthy(iris_eq(iris_graph_get_prim_op(program, iris_graph_get_root(program)), iris_int(7)))) {
                                        /* Node 17283128402654903231 (Guard) */
                                        if (iris_is_truthy(iris_lt(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs), iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs)))) {
                                            return iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs);
                                        } else {
                                            return iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs);
                                        }
                                    } else {
                                        /* Node 9912768874850238177 (Guard) */
                                        if (iris_is_truthy(iris_eq(iris_graph_get_prim_op(program, iris_graph_get_root(program)), iris_int(8)))) {
                                            /* Node 7124809665764100277 (Guard) */
                                            if (iris_is_truthy(iris_gt(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs), iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs)))) {
                                                return iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs);
                                            } else {
                                                return iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs);
                                            }
                                        } else {
                                            /* Node 12737220541336813991 (Guard) */
                                            if (iris_is_truthy(iris_eq(iris_graph_get_prim_op(program, iris_graph_get_root(program)), iris_int(9)))) {
                                                /* Node 9622398301786880521 (Guard) */
                                                if (iris_is_truthy(iris_lt(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_int(0)))) {
                                                    return iris_int(0);
                                                } else {
                                                    /* Node 7728613099420246023 (Guard) */
                                                    if (iris_is_truthy(iris_eq(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_int(0)))) {
                                                        return iris_int(1);
                                                    } else {
                                                        /* Node 7323901785270323815 (Guard) */
                                                        if (iris_is_truthy(iris_eq(iris_div(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_int(2)), iris_int(0)))) {
                                                            /* Node 8306638865974969135 (Guard) */
                                                            if (iris_is_truthy(iris_eq(iris_sub(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_mul(iris_div(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_int(2)), iris_int(2))), iris_int(1)))) {
                                                                return iris_mul(iris_int(1), iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs));
                                                            } else {
                                                                return iris_int(1);
                                                            }
                                                        } else {
                                                            /* Node 297011005493917890 (Guard) */
                                                            if (iris_is_truthy(iris_eq(iris_div(iris_div(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_int(2)), iris_int(2)), iris_int(0)))) {
                                                                /* Node 5469743878379171764 (Guard) */
                                                                if (iris_is_truthy(iris_eq(iris_sub(iris_div(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_int(2)), iris_mul(iris_div(iris_div(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_int(2)), iris_int(2)), iris_int(2))), iris_int(1)))) {
                                                                    return iris_mul((iris_is_truthy(iris_eq(iris_sub(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_mul(iris_div(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_int(2)), iris_int(2))), iris_int(1))) ? iris_mul(iris_int(1), iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs)) : iris_int(1)), iris_mul(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs), iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs)));
                                                                } else {
                                                                    /* Node 8306638865974969135 (Guard) */
                                                                    if (iris_is_truthy(iris_eq(iris_sub(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_mul(iris_div(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_int(2)), iris_int(2))), iris_int(1)))) {
                                                                        return iris_mul(iris_int(1), iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs));
                                                                    } else {
                                                                        return iris_int(1);
                                                                    }
                                                                }
                                                            } else {
                                                                /* Node 982671103229182355 (Guard) */
                                                                if (iris_is_truthy(iris_eq(iris_div(iris_div(iris_div(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_int(2)), iris_int(2)), iris_int(2)), iris_int(0)))) {
                                                                    /* Node 4186192556225673833 (Guard) */
                                                                    if (iris_is_truthy(iris_eq(iris_sub(iris_div(iris_div(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_int(2)), iris_int(2)), iris_mul(iris_div(iris_div(iris_div(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_int(2)), iris_int(2)), iris_int(2)), iris_int(2))), iris_int(1)))) {
                                                                        return iris_mul((iris_is_truthy(iris_eq(iris_sub(iris_div(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_int(2)), iris_mul(iris_div(iris_div(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_int(2)), iris_int(2)), iris_int(2))), iris_int(1))) ? iris_mul((iris_is_truthy(iris_eq(iris_sub(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_mul(iris_div(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_int(2)), iris_int(2))), iris_int(1))) ? iris_mul(iris_int(1), iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs)) : iris_int(1)), iris_mul(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs), iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs))) : (iris_is_truthy(iris_eq(iris_sub(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_mul(iris_div(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_int(2)), iris_int(2))), iris_int(1))) ? iris_mul(iris_int(1), iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs)) : iris_int(1))), iris_mul(iris_mul(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs), iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs)), iris_mul(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs), iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs))));
                                                                    } else {
                                                                        /* Node 5469743878379171764 (Guard) */
                                                                        if (iris_is_truthy(iris_eq(iris_sub(iris_div(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_int(2)), iris_mul(iris_div(iris_div(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_int(2)), iris_int(2)), iris_int(2))), iris_int(1)))) {
                                                                            return iris_mul((iris_is_truthy(iris_eq(iris_sub(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_mul(iris_div(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_int(2)), iris_int(2))), iris_int(1))) ? iris_mul(iris_int(1), iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs)) : iris_int(1)), iris_mul(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs), iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs)));
                                                                        } else {
                                                                            /* Node 8306638865974969135 (Guard) */
                                                                            if (iris_is_truthy(iris_eq(iris_sub(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_mul(iris_div(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_int(2)), iris_int(2))), iris_int(1)))) {
                                                                                return iris_mul(iris_int(1), iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs));
                                                                            } else {
                                                                                return iris_int(1);
                                                                            }
                                                                        }
                                                                    }
                                                                } else {
                                                                    /* Node 12058339177145159051 (Guard) */
                                                                    if (iris_is_truthy(iris_eq(iris_div(iris_div(iris_div(iris_div(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_int(2)), iris_int(2)), iris_int(2)), iris_int(2)), iris_int(0)))) {
                                                                        /* Node 12599836267461387759 (Guard) */
                                                                        if (iris_is_truthy(iris_eq(iris_sub(iris_div(iris_div(iris_div(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_int(2)), iris_int(2)), iris_int(2)), iris_mul(iris_div(iris_div(iris_div(iris_div(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_int(2)), iris_int(2)), iris_int(2)), iris_int(2)), iris_int(2))), iris_int(1)))) {
                                                                            return iris_mul((iris_is_truthy(iris_eq(iris_sub(iris_div(iris_div(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_int(2)), iris_int(2)), iris_mul(iris_div(iris_div(iris_div(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_int(2)), iris_int(2)), iris_int(2)), iris_int(2))), iris_int(1))) ? iris_mul((iris_is_truthy(iris_eq(iris_sub(iris_div(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_int(2)), iris_mul(iris_div(iris_div(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_int(2)), iris_int(2)), iris_int(2))), iris_int(1))) ? iris_mul((iris_is_truthy(iris_eq(iris_sub(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_mul(iris_div(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_int(2)), iris_int(2))), iris_int(1))) ? iris_mul(iris_int(1), iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs)) : iris_int(1)), iris_mul(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs), iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs))) : (iris_is_truthy(iris_eq(iris_sub(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_mul(iris_div(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_int(2)), iris_int(2))), iris_int(1))) ? iris_mul(iris_int(1), iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs)) : iris_int(1))), iris_mul(iris_mul(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs), iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs)), iris_mul(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs), iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs)))) : (iris_is_truthy(iris_eq(iris_sub(iris_div(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_int(2)), iris_mul(iris_div(iris_div(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_int(2)), iris_int(2)), iris_int(2))), iris_int(1))) ? iris_mul((iris_is_truthy(iris_eq(iris_sub(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_mul(iris_div(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_int(2)), iris_int(2))), iris_int(1))) ? iris_mul(iris_int(1), iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs)) : iris_int(1)), iris_mul(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs), iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs))) : (iris_is_truthy(iris_eq(iris_sub(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_mul(iris_div(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_int(2)), iris_int(2))), iris_int(1))) ? iris_mul(iris_int(1), iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs)) : iris_int(1)))), iris_mul(iris_mul(iris_mul(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs), iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs)), iris_mul(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs), iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs))), iris_mul(iris_mul(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs), iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs)), iris_mul(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs), iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs)))));
                                                                        } else {
                                                                            /* Node 4186192556225673833 (Guard) */
                                                                            if (iris_is_truthy(iris_eq(iris_sub(iris_div(iris_div(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_int(2)), iris_int(2)), iris_mul(iris_div(iris_div(iris_div(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_int(2)), iris_int(2)), iris_int(2)), iris_int(2))), iris_int(1)))) {
                                                                                return iris_mul((iris_is_truthy(iris_eq(iris_sub(iris_div(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_int(2)), iris_mul(iris_div(iris_div(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_int(2)), iris_int(2)), iris_int(2))), iris_int(1))) ? iris_mul((iris_is_truthy(iris_eq(iris_sub(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_mul(iris_div(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_int(2)), iris_int(2))), iris_int(1))) ? iris_mul(iris_int(1), iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs)) : iris_int(1)), iris_mul(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs), iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs))) : (iris_is_truthy(iris_eq(iris_sub(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_mul(iris_div(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_int(2)), iris_int(2))), iris_int(1))) ? iris_mul(iris_int(1), iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs)) : iris_int(1))), iris_mul(iris_mul(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs), iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs)), iris_mul(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs), iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs))));
                                                                            } else {
                                                                                /* Node 5469743878379171764 (Guard) */
                                                                                if (iris_is_truthy(iris_eq(iris_sub(iris_div(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_int(2)), iris_mul(iris_div(iris_div(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_int(2)), iris_int(2)), iris_int(2))), iris_int(1)))) {
                                                                                    return iris_mul((iris_is_truthy(iris_eq(iris_sub(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_mul(iris_div(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_int(2)), iris_int(2))), iris_int(1))) ? iris_mul(iris_int(1), iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs)) : iris_int(1)), iris_mul(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs), iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs)));
                                                                                } else {
                                                                                    /* Node 8306638865974969135 (Guard) */
                                                                                    if (iris_is_truthy(iris_eq(iris_sub(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_mul(iris_div(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_int(2)), iris_int(2))), iris_int(1)))) {
                                                                                        return iris_mul(iris_int(1), iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs));
                                                                                    } else {
                                                                                        return iris_int(1);
                                                                                    }
                                                                                }
                                                                            }
                                                                        }
                                                                    } else {
                                                                        /* Node 11925754399188782621 (Guard) */
                                                                        if (iris_is_truthy(iris_eq(iris_sub(iris_div(iris_div(iris_div(iris_div(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_int(2)), iris_int(2)), iris_int(2)), iris_int(2)), iris_mul(iris_div(iris_div(iris_div(iris_div(iris_div(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_int(2)), iris_int(2)), iris_int(2)), iris_int(2)), iris_int(2)), iris_int(2))), iris_int(1)))) {
                                                                            return iris_mul((iris_is_truthy(iris_eq(iris_sub(iris_div(iris_div(iris_div(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_int(2)), iris_int(2)), iris_int(2)), iris_mul(iris_div(iris_div(iris_div(iris_div(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_int(2)), iris_int(2)), iris_int(2)), iris_int(2)), iris_int(2))), iris_int(1))) ? iris_mul((iris_is_truthy(iris_eq(iris_sub(iris_div(iris_div(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_int(2)), iris_int(2)), iris_mul(iris_div(iris_div(iris_div(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_int(2)), iris_int(2)), iris_int(2)), iris_int(2))), iris_int(1))) ? iris_mul((iris_is_truthy(iris_eq(iris_sub(iris_div(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_int(2)), iris_mul(iris_div(iris_div(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_int(2)), iris_int(2)), iris_int(2))), iris_int(1))) ? iris_mul((iris_is_truthy(iris_eq(iris_sub(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_mul(iris_div(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_int(2)), iris_int(2))), iris_int(1))) ? iris_mul(iris_int(1), iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs)) : iris_int(1)), iris_mul(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs), iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs))) : (iris_is_truthy(iris_eq(iris_sub(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_mul(iris_div(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_int(2)), iris_int(2))), iris_int(1))) ? iris_mul(iris_int(1), iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs)) : iris_int(1))), iris_mul(iris_mul(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs), iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs)), iris_mul(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs), iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs)))) : (iris_is_truthy(iris_eq(iris_sub(iris_div(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_int(2)), iris_mul(iris_div(iris_div(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_int(2)), iris_int(2)), iris_int(2))), iris_int(1))) ? iris_mul((iris_is_truthy(iris_eq(iris_sub(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_mul(iris_div(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_int(2)), iris_int(2))), iris_int(1))) ? iris_mul(iris_int(1), iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs)) : iris_int(1)), iris_mul(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs), iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs))) : (iris_is_truthy(iris_eq(iris_sub(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_mul(iris_div(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_int(2)), iris_int(2))), iris_int(1))) ? iris_mul(iris_int(1), iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs)) : iris_int(1)))), iris_mul(iris_mul(iris_mul(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs), iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs)), iris_mul(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs), iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs))), iris_mul(iris_mul(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs), iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs)), iris_mul(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs), iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs))))) : (iris_is_truthy(iris_eq(iris_sub(iris_div(iris_div(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_int(2)), iris_int(2)), iris_mul(iris_div(iris_div(iris_div(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_int(2)), iris_int(2)), iris_int(2)), iris_int(2))), iris_int(1))) ? iris_mul((iris_is_truthy(iris_eq(iris_sub(iris_div(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_int(2)), iris_mul(iris_div(iris_div(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_int(2)), iris_int(2)), iris_int(2))), iris_int(1))) ? iris_mul((iris_is_truthy(iris_eq(iris_sub(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_mul(iris_div(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_int(2)), iris_int(2))), iris_int(1))) ? iris_mul(iris_int(1), iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs)) : iris_int(1)), iris_mul(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs), iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs))) : (iris_is_truthy(iris_eq(iris_sub(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_mul(iris_div(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_int(2)), iris_int(2))), iris_int(1))) ? iris_mul(iris_int(1), iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs)) : iris_int(1))), iris_mul(iris_mul(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs), iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs)), iris_mul(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs), iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs)))) : (iris_is_truthy(iris_eq(iris_sub(iris_div(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_int(2)), iris_mul(iris_div(iris_div(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_int(2)), iris_int(2)), iris_int(2))), iris_int(1))) ? iris_mul((iris_is_truthy(iris_eq(iris_sub(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_mul(iris_div(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_int(2)), iris_int(2))), iris_int(1))) ? iris_mul(iris_int(1), iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs)) : iris_int(1)), iris_mul(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs), iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs))) : (iris_is_truthy(iris_eq(iris_sub(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_mul(iris_div(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_int(2)), iris_int(2))), iris_int(1))) ? iris_mul(iris_int(1), iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs)) : iris_int(1))))), iris_mul(iris_mul(iris_mul(iris_mul(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs), iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs)), iris_mul(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs), iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs))), iris_mul(iris_mul(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs), iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs)), iris_mul(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs), iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs)))), iris_mul(iris_mul(iris_mul(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs), iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs)), iris_mul(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs), iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs))), iris_mul(iris_mul(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs), iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs)), iris_mul(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs), iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs))))));
                                                                        } else {
                                                                            /* Node 12599836267461387759 (Guard) */
                                                                            if (iris_is_truthy(iris_eq(iris_sub(iris_div(iris_div(iris_div(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_int(2)), iris_int(2)), iris_int(2)), iris_mul(iris_div(iris_div(iris_div(iris_div(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_int(2)), iris_int(2)), iris_int(2)), iris_int(2)), iris_int(2))), iris_int(1)))) {
                                                                                return iris_mul((iris_is_truthy(iris_eq(iris_sub(iris_div(iris_div(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_int(2)), iris_int(2)), iris_mul(iris_div(iris_div(iris_div(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_int(2)), iris_int(2)), iris_int(2)), iris_int(2))), iris_int(1))) ? iris_mul((iris_is_truthy(iris_eq(iris_sub(iris_div(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_int(2)), iris_mul(iris_div(iris_div(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_int(2)), iris_int(2)), iris_int(2))), iris_int(1))) ? iris_mul((iris_is_truthy(iris_eq(iris_sub(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_mul(iris_div(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_int(2)), iris_int(2))), iris_int(1))) ? iris_mul(iris_int(1), iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs)) : iris_int(1)), iris_mul(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs), iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs))) : (iris_is_truthy(iris_eq(iris_sub(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_mul(iris_div(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_int(2)), iris_int(2))), iris_int(1))) ? iris_mul(iris_int(1), iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs)) : iris_int(1))), iris_mul(iris_mul(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs), iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs)), iris_mul(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs), iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs)))) : (iris_is_truthy(iris_eq(iris_sub(iris_div(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_int(2)), iris_mul(iris_div(iris_div(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_int(2)), iris_int(2)), iris_int(2))), iris_int(1))) ? iris_mul((iris_is_truthy(iris_eq(iris_sub(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_mul(iris_div(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_int(2)), iris_int(2))), iris_int(1))) ? iris_mul(iris_int(1), iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs)) : iris_int(1)), iris_mul(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs), iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs))) : (iris_is_truthy(iris_eq(iris_sub(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_mul(iris_div(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_int(2)), iris_int(2))), iris_int(1))) ? iris_mul(iris_int(1), iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs)) : iris_int(1)))), iris_mul(iris_mul(iris_mul(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs), iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs)), iris_mul(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs), iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs))), iris_mul(iris_mul(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs), iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs)), iris_mul(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs), iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs)))));
                                                                            } else {
                                                                                /* Node 4186192556225673833 (Guard) */
                                                                                if (iris_is_truthy(iris_eq(iris_sub(iris_div(iris_div(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_int(2)), iris_int(2)), iris_mul(iris_div(iris_div(iris_div(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_int(2)), iris_int(2)), iris_int(2)), iris_int(2))), iris_int(1)))) {
                                                                                    return iris_mul((iris_is_truthy(iris_eq(iris_sub(iris_div(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_int(2)), iris_mul(iris_div(iris_div(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_int(2)), iris_int(2)), iris_int(2))), iris_int(1))) ? iris_mul((iris_is_truthy(iris_eq(iris_sub(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_mul(iris_div(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_int(2)), iris_int(2))), iris_int(1))) ? iris_mul(iris_int(1), iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs)) : iris_int(1)), iris_mul(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs), iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs))) : (iris_is_truthy(iris_eq(iris_sub(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_mul(iris_div(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_int(2)), iris_int(2))), iris_int(1))) ? iris_mul(iris_int(1), iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs)) : iris_int(1))), iris_mul(iris_mul(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs), iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs)), iris_mul(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs), iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs))));
                                                                                } else {
                                                                                    /* Node 5469743878379171764 (Guard) */
                                                                                    if (iris_is_truthy(iris_eq(iris_sub(iris_div(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_int(2)), iris_mul(iris_div(iris_div(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_int(2)), iris_int(2)), iris_int(2))), iris_int(1)))) {
                                                                                        return iris_mul((iris_is_truthy(iris_eq(iris_sub(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_mul(iris_div(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_int(2)), iris_int(2))), iris_int(1))) ? iris_mul(iris_int(1), iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs)) : iris_int(1)), iris_mul(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs), iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs)));
                                                                                    } else {
                                                                                        /* Node 8306638865974969135 (Guard) */
                                                                                        if (iris_is_truthy(iris_eq(iris_sub(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_mul(iris_div(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs), iris_int(2)), iris_int(2))), iris_int(1)))) {
                                                                                            return iris_mul(iris_int(1), iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs));
                                                                                        } else {
                                                                                            return iris_int(1);
                                                                                        }
                                                                                    }
                                                                                }
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            } else {
                                                /* Node 14022853554762741850 (Guard) */
                                                if (iris_is_truthy(iris_eq(iris_graph_get_prim_op(program, iris_graph_get_root(program)), iris_int(5)))) {
                                                    return iris_sub(iris_int(0), iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs));
                                                } else {
                                                    /* Node 16170783195928414403 (Guard) */
                                                    if (iris_is_truthy(iris_eq(iris_graph_get_prim_op(program, iris_graph_get_root(program)), iris_int(6)))) {
                                                        /* Node 5500292677411022907 (Guard) */
                                                        if (iris_is_truthy(iris_lt(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs), iris_int(0)))) {
                                                            return iris_sub(iris_int(0), iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs));
                                                        } else {
                                                            return iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs);
                                                        }
                                                    } else {
                                                        return iris_graph_eval(program, inputs);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            } else {
                /* Node 14711076078537531902 (Guard) */
                if (iris_is_truthy(iris_gt(iris_graph_get_prim_op(program, iris_graph_get_root(program)), iris_int(31)))) {
                    /* Node 7591437944546066557 (Guard) */
                    if (iris_is_truthy(iris_lt(iris_graph_get_prim_op(program, iris_graph_get_root(program)), iris_int(38)))) {
                        /* Node 1658543528041261661 (Guard) */
                        if (iris_is_truthy(iris_eq(iris_graph_get_prim_op(program, iris_graph_get_root(program)), iris_int(32)))) {
                            /* Node 3034101205014602107 (Guard) */
                            if (iris_is_truthy(iris_eq(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs), iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs)))) {
                                return iris_int(1);
                            } else {
                                return iris_int(0);
                            }
                        } else {
                            /* Node 8607628140523293779 (Guard) */
                            if (iris_is_truthy(iris_eq(iris_graph_get_prim_op(program, iris_graph_get_root(program)), iris_int(33)))) {
                                /* Node 15353126932450161068 (Guard) */
                                if (iris_is_truthy(iris_eq(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs), iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs)))) {
                                    return iris_int(0);
                                } else {
                                    return iris_int(1);
                                }
                            } else {
                                /* Node 13681752197348407764 (Guard) */
                                if (iris_is_truthy(iris_eq(iris_graph_get_prim_op(program, iris_graph_get_root(program)), iris_int(34)))) {
                                    /* Node 14715987110344714735 (Guard) */
                                    if (iris_is_truthy(iris_lt(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs), iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs)))) {
                                        return iris_int(1);
                                    } else {
                                        return iris_int(0);
                                    }
                                } else {
                                    /* Node 1428228556446781256 (Guard) */
                                    if (iris_is_truthy(iris_eq(iris_graph_get_prim_op(program, iris_graph_get_root(program)), iris_int(35)))) {
                                        /* Node 10950821722264384482 (Guard) */
                                        if (iris_is_truthy(iris_gt(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs), iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs)))) {
                                            return iris_int(1);
                                        } else {
                                            return iris_int(0);
                                        }
                                    } else {
                                        /* Node 10841091013466695784 (Guard) */
                                        if (iris_is_truthy(iris_eq(iris_graph_get_prim_op(program, iris_graph_get_root(program)), iris_int(36)))) {
                                            /* Node 15339408676761165993 (Guard) */
                                            if (iris_is_truthy(iris_lt(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs), iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs)))) {
                                                return iris_int(1);
                                            } else {
                                                /* Node 9866366655149441221 (Guard) */
                                                if (iris_is_truthy(iris_eq(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs), iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs)))) {
                                                    return iris_int(1);
                                                } else {
                                                    return iris_int(0);
                                                }
                                            }
                                        } else {
                                            /* Node 7349882618105606489 (Guard) */
                                            if (iris_is_truthy(iris_eq(iris_graph_get_prim_op(program, iris_graph_get_root(program)), iris_int(37)))) {
                                                /* Node 14944146142046027035 (Guard) */
                                                if (iris_is_truthy(iris_gt(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs), iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs)))) {
                                                    return iris_int(1);
                                                } else {
                                                    /* Node 1885226657094345158 (Guard) */
                                                    if (iris_is_truthy(iris_eq(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs), iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(1))), inputs)))) {
                                                        return iris_int(1);
                                                    } else {
                                                        return iris_int(0);
                                                    }
                                                }
                                            } else {
                                                return iris_graph_eval(program, inputs);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        return iris_graph_eval(program, inputs);
                    }
                } else {
                    return iris_graph_eval(program, inputs);
                }
            }
        } else {
            /* Node 949499296119491328 (Guard) */
            if (iris_is_truthy(iris_eq(iris_list_len(iris_graph_outgoing(program, iris_graph_get_root(program))), iris_int(1)))) {
                /* Node 3617919193432735771 (Guard) */
                if (iris_is_truthy(iris_eq(iris_graph_get_prim_op(program, iris_graph_get_root(program)), iris_int(5)))) {
                    return iris_sub(iris_int(0), iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs));
                } else {
                    /* Node 579399722941095047 (Guard) */
                    if (iris_is_truthy(iris_eq(iris_graph_get_prim_op(program, iris_graph_get_root(program)), iris_int(6)))) {
                        /* Node 8725297170155282876 (Guard) */
                        if (iris_is_truthy(iris_lt(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs), iris_int(0)))) {
                            return iris_sub(iris_int(0), iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs));
                        } else {
                            return iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs);
                        }
                    } else {
                        /* Node 15678121141355902846 (Guard) */
                        if (iris_is_truthy(iris_gt(iris_graph_get_prim_op(program, iris_graph_get_root(program)), iris_int(63)))) {
                            /* Node 11426915708329801472 (Guard) */
                            if (iris_is_truthy(iris_lt(iris_graph_get_prim_op(program, iris_graph_get_root(program)), iris_int(69)))) {
                                /* Node 1803262175905777769 (Guard) */
                                if (iris_is_truthy(iris_eq(iris_graph_get_prim_op(program, iris_graph_get_root(program)), iris_int(68)))) {
                                    /* Node 10873012613320564942 (Guard) */
                                    if (iris_is_truthy(iris_eq(iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs), iris_int(0)))) {
                                        return iris_int(0);
                                    } else {
                                        return iris_int(1);
                                    }
                                } else {
                                    return iris_graph_eval(iris_graph_set_root(program, iris_list_nth(iris_graph_outgoing(program, iris_graph_get_root(program)), iris_int(0))), inputs);
                                }
                            } else {
                                return iris_graph_eval(program, inputs);
                            }
                        } else {
                            return iris_graph_eval(program, inputs);
                        }
                    }
                }
            } else {
                return iris_graph_eval(program, inputs);
            }
        }
    } else {
        /* Node 8224081199128376937 (Guard) */
        if (iris_is_truthy(iris_eq(iris_graph_get_kind(program, iris_graph_get_root(program)), iris_int(5)))) {
            return iris_graph_eval(program, inputs);
        } else {
            /* Node 10174533669715646278 (Guard) */
            if (iris_is_truthy(iris_eq(iris_graph_get_kind(program, iris_graph_get_root(program)), iris_int(1)))) {
                return iris_graph_eval(program, inputs);
            } else {
                /* Node 7470677997057433652 (Guard) */
                if (iris_is_truthy(iris_eq(iris_graph_get_kind(program, iris_graph_get_root(program)), iris_int(2)))) {
                    return iris_graph_eval(program, inputs);
                } else {
                    /* Node 18102236574571657948 (Guard) */
                    if (iris_is_truthy(iris_eq(iris_graph_get_kind(program, iris_graph_get_root(program)), iris_int(3)))) {
                        return iris_graph_eval(program, inputs);
                    } else {
                        /* Node 12303065628584162013 (Guard) */
                        if (iris_is_truthy(iris_eq(iris_graph_get_kind(program, iris_graph_get_root(program)), iris_int(4)))) {
                            return iris_graph_eval(program, inputs);
                        } else {
                            /* Node 4478876409488054070 (Guard) */
                            if (iris_is_truthy(iris_eq(iris_graph_get_kind(program, iris_graph_get_root(program)), iris_int(6)))) {
                                return iris_graph_eval(program, inputs);
                            } else {
                                /* Node 17550646546324492743 (Guard) */
                                if (iris_is_truthy(iris_eq(iris_graph_get_kind(program, iris_graph_get_root(program)), iris_int(7)))) {
                                    return iris_neg(iris_int(1));
                                } else {
                                    /* Node 15152276792198353511 (Guard) */
                                    if (iris_is_truthy(iris_eq(iris_graph_get_kind(program, iris_graph_get_root(program)), iris_int(8)))) {
                                        return iris_graph_eval(program, inputs);
                                    } else {
                                        /* Node 7154817677911913837 (Guard) */
                                        if (iris_is_truthy(iris_eq(iris_graph_get_kind(program, iris_graph_get_root(program)), iris_int(9)))) {
                                            return iris_graph_eval(program, inputs);
                                        } else {
                                            /* Node 15989696222530414987 (Guard) */
                                            if (iris_is_truthy(iris_eq(iris_graph_get_kind(program, iris_graph_get_root(program)), iris_int(10)))) {
                                                return iris_graph_eval(program, inputs);
                                            } else {
                                                /* Node 4546787149409161016 (Guard) */
                                                if (iris_is_truthy(iris_eq(iris_graph_get_kind(program, iris_graph_get_root(program)), iris_int(11)))) {
                                                    return iris_graph_eval(program, inputs);
                                                } else {
                                                    /* Node 15853057743244475095 (Guard) */
                                                    if (iris_is_truthy(iris_eq(iris_graph_get_kind(program, iris_graph_get_root(program)), iris_int(12)))) {
                                                        return iris_graph_eval(program, inputs);
                                                    } else {
                                                        /* Node 17672347902133348804 (Guard) */
                                                        if (iris_is_truthy(iris_eq(iris_graph_get_kind(program, iris_graph_get_root(program)), iris_int(13)))) {
                                                            return iris_graph_eval(program, inputs);
                                                        } else {
                                                            /* Node 11154422202924980821 (Guard) */
                                                            if (iris_is_truthy(iris_eq(iris_graph_get_kind(program, iris_graph_get_root(program)), iris_int(14)))) {
                                                                return iris_graph_eval(program, inputs);
                                                            } else {
                                                                /* Node 15148362811109893154 (Guard) */
                                                                if (iris_is_truthy(iris_eq(iris_graph_get_kind(program, iris_graph_get_root(program)), iris_int(15)))) {
                                                                    return iris_graph_eval(program, inputs);
                                                                } else {
                                                                    /* Node 4406387296577748718 (Guard) */
                                                                    if (iris_is_truthy(iris_eq(iris_graph_get_kind(program, iris_graph_get_root(program)), iris_int(16)))) {
                                                                        return iris_graph_eval(program, inputs);
                                                                    } else {
                                                                        /* Node 5000784544319495265 (Guard) */
                                                                        if (iris_is_truthy(iris_eq(iris_graph_get_kind(program, iris_graph_get_root(program)), iris_int(17)))) {
                                                                            return iris_graph_eval(program, inputs);
                                                                        } else {
                                                                            /* Node 13575236015628884304 (Guard) */
                                                                            if (iris_is_truthy(iris_eq(iris_graph_get_kind(program, iris_graph_get_root(program)), iris_int(18)))) {
                                                                                return iris_graph_eval(program, inputs);
                                                                            } else {
                                                                                /* Node 17014622939366941497 (Guard) */
                                                                                if (iris_is_truthy(iris_eq(iris_graph_get_kind(program, iris_graph_get_root(program)), iris_int(19)))) {
                                                                                    return iris_neg(iris_int(1));
                                                                                } else {
                                                                                    return iris_neg(iris_int(1));
                                                                                }
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
