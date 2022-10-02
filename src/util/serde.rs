use serde::{de::Visitor, Deserializer, __private::fmt};

trait HexStringOrInt {
    fn to_int(self) -> u32;
}

impl HexStringOrInt for u32 {
    fn to_int(self) -> u32 {
        self
    }
}

impl HexStringOrInt for &str {
    fn to_int(self) -> u32 {
        let raw_bytes = self.trim_start_matches("0x");
        u32::from_str_radix(raw_bytes, 16)
            .expect(&format!("Could not parse hex value \"{raw_bytes}\""))
    }
}

impl HexStringOrInt for String {
    fn to_int(self) -> u32 {
        self.as_str().to_int()
    }
}

pub fn serde_string_or_int<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    struct HexOrInt;

    impl<'de> Visitor<'de> for HexOrInt {
        type Value = u32;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("hex string or integer")
        }

        fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(value.to_int())
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(value.to_int())
        }

        fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            value.try_into().map_err(E::custom)
        }

        fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            value.try_into().map_err(E::custom)
        }
    }

    deserializer.deserialize_any(HexOrInt)
}
