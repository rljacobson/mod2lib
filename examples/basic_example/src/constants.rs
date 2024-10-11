/*!

Some examples of constant atoms. For the typical case when the constant is a newtype of a type that satisfies the type constraints `Debug + Any + PartialEq + Hash`, we can use the `declare_constant_newtype` macro.

*/

use mod2lib::api::atom::{
  ConstantType,
  declare_constant_newtype
};

declare_constant_newtype!(IntegerConstant, isize);
declare_constant_newtype!(StringConstant, String);
declare_constant_newtype!(FloatConstant, f64);
declare_constant_newtype!(ByteConstant, u8);
declare_constant_newtype!(BoolConstant, bool);

