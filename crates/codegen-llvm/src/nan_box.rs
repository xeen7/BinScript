//! NaN-boxing constants and helper methods.
//!
//! Every JS value is encoded as a 64-bit integer.
//! - If the upper 16 bits are NOT `0xFFF1..0xFFFA`, the 64 bits are a raw
//!   IEEE-754 `f64`.
//! - Otherwise the upper 16 bits encode a *tag* and the lower 48 bits are
//!   a payload (pointer, integer, or unused).
//!
//! Tag table (upper 16 bits):
//!
//! | Tag      | Meaning               |
//! |----------|-----------------------|
//! | `0xFFF1` | `undefined`           |
//! | `0xFFF2` | `null`                |
//! | `0xFFF3` | `false`               |
//! | `0xFFF4` | `true`                |
//! | `0xFFF5` | `int32` (lower 32b)   |
//! | `0xFFF6` | `object` pointer      |
//! | `0xFFF7` | `string` pointer      |

use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::types::{FloatType, IntType};
use inkwell::values::{FloatValue, IntValue, PointerValue};
use inkwell::{FloatPredicate, IntPredicate};

// Tag constants — upper 16 bits of the NaN-boxed i64.
pub const TAG_UNDEFINED: u64 = 0xFFF1_0000_0000_0000;
pub const TAG_NULL: u64      = 0xFFF2_0000_0000_0000;
pub const TAG_FALSE: u64     = 0xFFF3_0000_0000_0000;
pub const TAG_TRUE: u64      = 0xFFF4_0000_0000_0000;
pub const TAG_INT32: u64     = 0xFFF5_0000_0000_0000;
pub const TAG_OBJECT: u64    = 0xFFF6_0000_0000_0000;
pub const TAG_STRING: u64    = 0xFFF7_0000_0000_0000;
pub const TAG_SYMBOL: u64    = 0xFFF8_0000_0000_0000;
pub const TAG_CLOSURE: u64   = 0xFFF9_0000_0000_0000;
pub const TAG_GENERATOR: u64 = 0xFFFA_0000_0000_0000;

/// Mask to isolate the upper 16 bits.
pub const TOP16_MASK: u64    = 0xFFFF_0000_0000_0000;
/// Mask to extract the lower 48-bit payload.
pub const PAYLOAD_MASK: u64  = 0x0000_FFFF_FFFF_FFFF;

/// Minimum tag value — anything with upper 16 bits >= this is tagged.
pub const TAG_MIN: u64       = 0xFFF1;

pub struct NanBoxHelper<'ctx> {
    pub i64_ty: IntType<'ctx>,
    pub f64_ty: FloatType<'ctx>,
    #[allow(dead_code)]
    pub i1_ty: IntType<'ctx>,
}

impl<'ctx> NanBoxHelper<'ctx> {
    pub fn new(
        _ctx: &'ctx Context,
        i64_ty: IntType<'ctx>,
        f64_ty: FloatType<'ctx>,
        i1_ty: IntType<'ctx>,
    ) -> Self {
        Self { i64_ty, f64_ty, i1_ty }
    }

    // ── compile-time constants ─────────────────────────────────────────────

    pub fn const_number(&self, n: f64) -> IntValue<'ctx> {
        self.i64_ty.const_int(n.to_bits(), false)
    }

    pub fn const_bool(&self, b: bool) -> IntValue<'ctx> {
        self.i64_ty.const_int(if b { TAG_TRUE } else { TAG_FALSE }, false)
    }

    pub fn const_null(&self) -> IntValue<'ctx> {
        self.i64_ty.const_int(TAG_NULL, false)
    }

    pub fn const_undefined(&self) -> IntValue<'ctx> {
        self.i64_ty.const_int(TAG_UNDEFINED, false)
    }

    // ── boxing ─────────────────────────────────────────────────────────────

    pub fn box_number(&self, builder: &Builder<'ctx>, f: FloatValue<'ctx>) -> IntValue<'ctx> {
        builder.build_bit_cast(f, self.i64_ty, "box_num")
            .unwrap()
            .into_int_value()
    }

    pub fn box_bool(&self, builder: &Builder<'ctx>, b: IntValue<'ctx>) -> IntValue<'ctx> {
        let t = self.i64_ty.const_int(TAG_TRUE, false);
        let f = self.i64_ty.const_int(TAG_FALSE, false);
        builder.build_select(b, t, f, "box_bool").unwrap().into_int_value()
    }

    pub fn box_string_ptr(&self, builder: &Builder<'ctx>, ptr: PointerValue<'ctx>) -> IntValue<'ctx> {
        let pi = builder.build_ptr_to_int(ptr, self.i64_ty, "ptr2int").unwrap();
        let tag = self.i64_ty.const_int(TAG_STRING, false);
        builder.build_or(pi, tag, "box_str").unwrap()
    }

    // ── unboxing ───────────────────────────────────────────────────────────

    pub fn unbox_number(&self, builder: &Builder<'ctx>, v: IntValue<'ctx>) -> FloatValue<'ctx> {
        builder.build_bit_cast(v, self.f64_ty, "unbox_num")
            .unwrap()
            .into_float_value()
    }

    // ── type checks ────────────────────────────────────────────────────────

    pub fn is_heap_pointer(&self, builder: &Builder<'ctx>, v: IntValue<'ctx>) -> IntValue<'ctx> {
        let shifted = builder
            .build_right_shift(v, self.i64_ty.const_int(48, false), false, "t16")
            .unwrap();
        let tag_obj = self.i64_ty.const_int(0xFFF6, false);
        let is_obj = builder.build_int_compare(IntPredicate::EQ, shifted, tag_obj, "is_obj").unwrap();
        
        let tag_closure = self.i64_ty.const_int(0xFFF9, false);
        let is_closure = builder.build_int_compare(IntPredicate::EQ, shifted, tag_closure, "is_closure").unwrap();

        let tag_gen = self.i64_ty.const_int(0xFFFA, false);
        let is_gen = builder.build_int_compare(IntPredicate::EQ, shifted, tag_gen, "is_gen").unwrap();
        
        let tag_array = self.i64_ty.const_int(0xFFFB, false);
        let is_array = builder.build_int_compare(IntPredicate::EQ, shifted, tag_array, "is_array").unwrap();
        
        let is_obj_or_closure = builder.build_or(is_obj, is_closure, "is_obj_or_closure").unwrap();
        let is_gen_or_array = builder.build_or(is_gen, is_array, "is_gen_or_array").unwrap();
        builder.build_or(is_obj_or_closure, is_gen_or_array, "is_heap_ptr").unwrap()
    }

    /// Returns an `i1` that is true when `v` is a plain f64 number
    /// (i.e. upper 16 bits < TAG_MIN).
    #[allow(dead_code)]
    pub fn is_number(&self, builder: &Builder<'ctx>, v: IntValue<'ctx>) -> IntValue<'ctx> {
        let shifted = builder
            .build_right_shift(v, self.i64_ty.const_int(48, false), false, "top16")
            .unwrap();
        let tag_min = self.i64_ty.const_int(TAG_MIN, false);
        builder
            .build_int_compare(IntPredicate::ULT, shifted, tag_min, "is_num")
            .unwrap()
    }

    /// JS truthiness: `false`, `null`, `undefined`, `0`, `NaN`, `""` are falsy.
    /// Everything else is truthy.
    pub fn is_truthy(&self, builder: &Builder<'ctx>, v: IntValue<'ctx>) -> IntValue<'ctx> {
        // Check upper 16 bits
        let shifted = builder
            .build_right_shift(v, self.i64_ty.const_int(48, false), false, "t16")
            .unwrap();
        let tag_min = self.i64_ty.const_int(TAG_MIN, false);
        let is_tagged = builder
            .build_int_compare(IntPredicate::UGE, shifted, tag_min, "is_tagged")
            .unwrap();

        // --- Tagged path: truthy if tag > 0xFFF4 (i.e. not undef/null/false) ---
        // Specifically: TAG_TRUE(0xFFF4), TAG_INT32(5), TAG_OBJECT(6), TAG_STRING(7) are truthy.
        // TAG_UNDEFINED(1), TAG_NULL(2), TAG_FALSE(3) are falsy.
        let truthy_tag_min = self.i64_ty.const_int(0xFFF4, false);
        let tag_truthy = builder
            .build_int_compare(IntPredicate::UGE, shifted, truthy_tag_min, "tag_truthy")
            .unwrap();

        // --- Number path: truthy if != 0.0 and not NaN ---
        let as_f64 = builder.build_bit_cast(v, self.f64_ty, "asf64").unwrap().into_float_value();
        let zero = self.f64_ty.const_float(0.0);
        let num_truthy = builder
            .build_float_compare(FloatPredicate::ONE, as_f64, zero, "num_truthy")
            .unwrap();

        // Select based on whether it's tagged
        builder
            .build_select(is_tagged, tag_truthy, num_truthy, "truthy")
            .unwrap()
            .into_int_value()
    }
}
