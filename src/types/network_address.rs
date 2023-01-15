use epee_serde::Value;
use serde::{de, ser::SerializeStruct, Deserialize, Serialize};

#[derive(Clone, Serialize, Debug, PartialEq, Eq)]
pub struct IPv4Address {
    m_ip: u32,
    m_port: u16,
}

impl IPv4Address {
    pub fn from_value<E: de::Error>(value: &Value) -> Result<Self, E> {
        let m_ip = get_val_from_map!(value, "m_ip", get_u32, "u32");

        let m_port = get_val_from_map!(value, "m_port", get_u16, "u16");

        Ok(IPv4Address {
            m_ip: *m_ip,
            m_port: *m_port,
        })
    }
}

#[derive(Clone, Serialize, Debug, PartialEq, Eq)]
pub struct IPv6Address {
    addr: [u8; 16],
    m_port: u16,
}

impl IPv6Address {
    pub fn from_value<E: de::Error>(value: &Value) -> Result<Self, E> {
        let addr = get_val_from_map!(value, "addr", get_bytes, "Vec<u8>");

        let m_port = get_val_from_map!(value, "m_port", get_u16, "u16");

        Ok(IPv6Address {
            addr: addr
                .clone()
                .try_into()
                .map_err(|_| E::invalid_length(addr.len(), &"a 16-byte array"))?,
            m_port: *m_port,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NetworkAddress {
    IPv4(IPv4Address),
    IPv6(IPv6Address),
}

impl<'de> Deserialize<'de> for NetworkAddress {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        let addr_type = get_val_from_map!(value, "type", get_u8, "u8");

        Ok(match addr_type {
            1 => NetworkAddress::IPv4(IPv4Address::from_value(get_field_from_map!(value, "addr"))?),
            2 => NetworkAddress::IPv6(IPv6Address::from_value(get_field_from_map!(value, "addr"))?),
            _ => {
                return Err(de::Error::custom(
                    "Network address type currently unsupported",
                ))
            }
        })
    }
}

impl Serialize for NetworkAddress {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("", 2)?;
        match self {
            NetworkAddress::IPv4(v) => {
                state.serialize_field("type", &1_u8)?;
                state.serialize_field("addr", v)?;
            }
            NetworkAddress::IPv6(v) => {
                state.serialize_field("type", &2_u8)?;
                state.serialize_field("addr", v)?;
            }
        }
        state.end()
    }
}
