use super::*;

impl<DB: HirBag + 'static> AstLowering<DB> {
    pub fn make_literal(&self, literal: Literal) -> HirLiteral {
        match literal {
            Literal::Error => HirLiteral::Error,
            Literal::True => HirLiteral::Int(1, HirISize::U1, HirIntSign::Unsigned),
            Literal::False => HirLiteral::Int(0, HirISize::U1, HirIntSign::Unsigned),
            Literal::String(value) => HirLiteral::String(HirString { value, name: None }),
            Literal::Nat(_) => todo!("lowering nat literals is not yet implemented"),
            Literal::Int8(value, Signed::Signed) => {
                HirLiteral::Int(value as _, HirISize::U8, HirIntSign::Signed)
            }
            Literal::Int8(value, Signed::Unsigned) => {
                HirLiteral::Int(value as _, HirISize::U8, HirIntSign::Unsigned)
            }
            Literal::Int16(value, Signed::Signed) => {
                HirLiteral::Int(value as _, HirISize::U16, HirIntSign::Signed)
            }
            Literal::Int16(value, Signed::Unsigned) => {
                HirLiteral::Int(value as _, HirISize::U16, HirIntSign::Unsigned)
            }
            Literal::Int32(value, Signed::Signed) => {
                HirLiteral::Int(value as _, HirISize::U32, HirIntSign::Signed)
            }
            Literal::Int32(value, Signed::Unsigned) => {
                HirLiteral::Int(value as _, HirISize::U32, HirIntSign::Unsigned)
            }
            Literal::Int64(value, Signed::Signed) => {
                HirLiteral::Int(value as _, HirISize::U64, HirIntSign::Signed)
            }
            Literal::Int64(value, Signed::Unsigned) => {
                HirLiteral::Int(value as _, HirISize::U64, HirIntSign::Unsigned)
            }
            Literal::Int128(value, Signed::Signed) => {
                HirLiteral::Int(value as _, HirISize::U128, HirIntSign::Signed)
            }
            Literal::Int128(value, Signed::Unsigned) => {
                HirLiteral::Int(value as _, HirISize::U128, HirIntSign::Unsigned)
            }
            Literal::Float32(value) => {
                let s = value.clone().to_string();

                let mut split = s.split('.');
                let integer = split.next().unwrap().parse::<usize>().unwrap();
                let decimal = split.next().unwrap_or("0").parse::<usize>().unwrap();

                HirLiteral::Decimal(HirFSize::F64, HirDecimal { integer, decimal })
            }
            Literal::Float64(value) => {
                let s = value.clone().to_string();

                let mut split = s.split('.');
                let integer = split.next().unwrap().parse::<usize>().unwrap();
                let decimal = split.next().unwrap_or("0").parse::<usize>().unwrap();

                HirLiteral::Decimal(HirFSize::F64, HirDecimal { integer, decimal })
            }
        }
    }
}
