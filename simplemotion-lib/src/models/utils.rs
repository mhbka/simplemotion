use std::{convert::TryFrom, fmt};
use serde::{
    de::{self, Visitor},
    Deserializer,
};

/// Deserializes a non-negative timeline frame, including exporters that write `1.0`.
pub fn deserialize_timeline_frame<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    struct TimelineFrameVisitor;

    impl<'de> Visitor<'de> for TimelineFrameVisitor {
        type Value = u32;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("an integral timeline frame")
        }

        fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            u32::try_from(value).map_err(E::custom)
        }

        fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            u32::try_from(value).map_err(E::custom)
        }

        fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            if !value.is_finite()
                || value.fract() != 0.0
                || value < 0.0
                || value > u32::MAX as f64
            {
                return Err(E::custom("timeline frame must be a u32"));
            }

            Ok(value as u32)
        }
    }

    deserializer.deserialize_any(TimelineFrameVisitor)
}

/// Deserializes an integral loop period, including the negative no-loop sentinel.
pub fn deserialize_loop_time<'de, D>(deserializer: D) -> Result<i32, D::Error>
where
    D: Deserializer<'de>,
{
    struct LoopTimeVisitor;

    impl<'de> Visitor<'de> for LoopTimeVisitor {
        type Value = i32;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("an integral loop period")
        }

        fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            i32::try_from(value).map_err(E::custom)
        }

        fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            i32::try_from(value).map_err(E::custom)
        }

        fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            if !value.is_finite()
                || value.fract() != 0.0
                || value < i32::MIN as f64
                || value > i32::MAX as f64
            {
                return Err(E::custom("loop period must be an i32"));
            }

            Ok(value as i32)
        }
    }

    deserializer.deserialize_any(LoopTimeVisitor)
}

/// Implement Deserialize for u8 -> enum variants.
#[macro_export]
macro_rules! numeric_type_enum {
    ($name:ident { $($number:literal => $variant:ident),+ $(,)? }) => {
        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                let value = u8::deserialize(deserializer)?;

                Ok(match value {
                    $($number => Self::$variant,)+
                    other => Self::Other(other),
                })
            }
        }
    };
}
