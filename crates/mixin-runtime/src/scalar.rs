use crate::{EntityId, Vec2};
use serde::{Deserialize, Serialize, Serializer};
use std::fmt;

macro_rules! scalars {
    ($($type:ident => $rust:ident),+ $(,)?) => {
        #[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
        #[serde(rename_all = "lowercase")]
        pub enum ScalarType {
            Unit,
            #[serde(rename = "entity_id")]
            EntityId,
            #[serde(rename = "option_entity_id")]
            OptionEntityId,
            Vec2,
            $($type),+
        }

        impl fmt::Display for ScalarType {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(match self {
                    Self::Unit => "()",
                    Self::EntityId => "EntityId",
                    Self::OptionEntityId => "Option<EntityId>",
                    Self::Vec2 => "Vec2",
                    $(Self::$type => stringify!($rust)),+
                })
            }
        }

        #[derive(Clone, Copy, Debug, PartialEq, Serialize)]
        #[serde(untagged)]
        pub enum ScalarValue {
            Unit,
            EntityId(#[serde(serialize_with = "serialize_entity_id")] EntityId),
            OptionEntityId(#[serde(serialize_with = "serialize_optional_entity_id")] Option<EntityId>),
            Vec2(#[serde(serialize_with = "serialize_vec2")] Vec2),
            $($type($rust)),+
        }

        impl ScalarValue {
            pub fn kind(self) -> ScalarType {
                match self {
                    Self::Unit => ScalarType::Unit,
                    Self::EntityId(_) => ScalarType::EntityId,
                    Self::OptionEntityId(_) => ScalarType::OptionEntityId,
                    Self::Vec2(_) => ScalarType::Vec2,
                    $(Self::$type(_) => ScalarType::$type),+
                }
            }

            pub fn parse(kind: ScalarType, text: &str) -> crate::Result<Self> {
                match kind {
                    ScalarType::Unit => Err("unit is only supported as a return type".into()),
                    ScalarType::EntityId => {
                        let value = text.parse::<u32>().map_err(|error| format!("invalid EntityId {text:?}: {error}"))?;
                        EntityId::new(value).map(Self::EntityId).ok_or_else(|| "EntityId must be nonzero".into())
                    }
                    ScalarType::OptionEntityId => Err("Option<EntityId> is only supported as a target return type".into()),
                    ScalarType::Vec2 => Err("Vec2 is only supported as a target return type".into()),
                    $(ScalarType::$type => text.parse::<$rust>().map(Self::$type)
                        .map_err(|error| format!("invalid {kind} value {text:?}: {error}"))),+
                }
            }
        }

        impl fmt::Display for ScalarValue {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                match self {
                    Self::Unit => f.write_str("()"),
                    Self::EntityId(value) => write!(f, "EntityId({})", value.get()),
                    Self::OptionEntityId(Some(value)) => write!(f, "Some(EntityId({}))", value.get()),
                    Self::OptionEntityId(None) => f.write_str("None"),
                    Self::Vec2(value) => write!(f, "Vec2({}, {})", value.x, value.y),
                    $(Self::$type(value) => value.fmt(f)),+
                }
            }
        }
    }
}

scalars! {
    Bool => bool,
    I8 => i8, I16 => i16, I32 => i32, I64 => i64, Isize => isize,
    U8 => u8, U16 => u16, U32 => u32, U64 => u64, Usize => usize,
    F32 => f32, F64 => f64,
}

impl ScalarValue {
    pub(crate) fn bits(self) -> u64 {
        match self {
            Self::Unit => 0,
            Self::EntityId(id) => u64::from(id.get()),
            Self::OptionEntityId(id) => u64::from(id.map_or(0, EntityId::get)),
            Self::Vec2(v) => u64::from(v.x.to_bits()) | (u64::from(v.y.to_bits()) << 32),
            Self::Bool(v) => v.into(),
            Self::I8(v) => v as u64,
            Self::I16(v) => v as u64,
            Self::I32(v) => v as u64,
            Self::I64(v) => v as u64,
            Self::Isize(v) => v as u64,
            Self::U8(v) => v.into(),
            Self::U16(v) => v.into(),
            Self::U32(v) => v.into(),
            Self::U64(v) => v,
            Self::Usize(v) => v as u64,
            Self::F32(v) => v.to_bits().into(),
            Self::F64(v) => v.to_bits(),
        }
    }

    pub(crate) fn from_bits(kind: ScalarType, bits: u64) -> crate::Result<Self> {
        Ok(match kind {
            ScalarType::Unit => Self::Unit,
            ScalarType::EntityId => Self::EntityId(
                EntityId::new(entity_bits(bits)?).ok_or("JIT returned a zero EntityId")?,
            ),
            ScalarType::OptionEntityId => Self::OptionEntityId(EntityId::new(entity_bits(bits)?)),
            ScalarType::Vec2 => Self::Vec2(Vec2 {
                x: f32::from_bits(bits as u32),
                y: f32::from_bits((bits >> 32) as u32),
            }),
            ScalarType::Bool => Self::Bool(bits != 0),
            ScalarType::I8 => Self::I8(bits as i8),
            ScalarType::I16 => Self::I16(bits as i16),
            ScalarType::I32 => Self::I32(bits as i32),
            ScalarType::I64 => Self::I64(bits as i64),
            ScalarType::Isize => Self::Isize(bits as isize),
            ScalarType::U8 => Self::U8(bits as u8),
            ScalarType::U16 => Self::U16(bits as u16),
            ScalarType::U32 => Self::U32(bits as u32),
            ScalarType::U64 => Self::U64(bits),
            ScalarType::Usize => Self::Usize(bits as usize),
            ScalarType::F32 => Self::F32(f32::from_bits(bits as u32)),
            ScalarType::F64 => Self::F64(f64::from_bits(bits)),
        })
    }
}

fn entity_bits(bits: u64) -> crate::Result<u32> {
    u32::try_from(bits).map_err(|_| "JIT returned an entity identifier larger than u32".into())
}

fn serialize_entity_id<S: Serializer>(id: &EntityId, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_u32(id.get())
}

fn serialize_optional_entity_id<S: Serializer>(
    id: &Option<EntityId>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    id.map(EntityId::get).serialize(serializer)
}

fn serialize_vec2<S: Serializer>(value: &Vec2, serializer: S) -> Result<S::Ok, S::Error> {
    use serde::ser::SerializeStruct;
    let mut fields = serializer.serialize_struct("Vec2", 2)?;
    fields.serialize_field("x", &value.x)?;
    fields.serialize_field("y", &value.y)?;
    fields.end()
}
