//! Type conversion utilities between Vais types and WIT types

use super::types::WitType;
use vais_types::ResolvedType;

pub fn vais_type_to_wit(ty: &ResolvedType) -> Option<WitType> {
    match ty {
        ResolvedType::Bool => Some(WitType::Bool),
        ResolvedType::U8 => Some(WitType::U8),
        ResolvedType::U16 => Some(WitType::U16),
        ResolvedType::U32 => Some(WitType::U32),
        ResolvedType::U64 => Some(WitType::U64),
        ResolvedType::I8 => Some(WitType::S8),
        ResolvedType::I16 => Some(WitType::S16),
        ResolvedType::I32 => Some(WitType::S32),
        ResolvedType::I64 => Some(WitType::S64),
        ResolvedType::F32 => Some(WitType::F32),
        ResolvedType::F64 => Some(WitType::F64),
        // Vais uses Str for string type
        ResolvedType::Str => Some(WitType::String),
        ResolvedType::Array(inner) => {
            let inner_wit = vais_type_to_wit(inner)?;
            Some(WitType::List(Box::new(inner_wit)))
        }
        ResolvedType::ConstArray { element, .. } => {
            // WIT doesn't have const-sized arrays, map to list
            let inner_wit = vais_type_to_wit(element)?;
            Some(WitType::List(Box::new(inner_wit)))
        }
        ResolvedType::Optional(inner) => {
            let inner_wit = vais_type_to_wit(inner)?;
            Some(WitType::Option_(Box::new(inner_wit)))
        }
        ResolvedType::Result(ok, err) => {
            // Vais Result<T, E>, map to result<T, E>
            let ok_wit = vais_type_to_wit(ok)?;
            let err_wit = vais_type_to_wit(err);
            Some(WitType::Result_ {
                ok: Some(Box::new(ok_wit)),
                err: err_wit.map(Box::new),
            })
        }
        ResolvedType::Tuple(types) => {
            let wit_types: Option<Vec<_>> = types.iter().map(vais_type_to_wit).collect();
            wit_types.map(WitType::Tuple)
        }
        ResolvedType::Named { name, .. } => Some(WitType::Named(name.clone())),
        _ => None, // Other types (pointers, refs, functions, etc.) not directly mappable to WIT
    }
}
