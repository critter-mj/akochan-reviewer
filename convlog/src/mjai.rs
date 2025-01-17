use crate::pai::Pai;

use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize};

/// Describes an event in mjlog format.
///
/// Note that this is a simplified version of mjlog, but it is sufficient for
/// akochan to read.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum Event {
    None,

    StartGame {
        kyoku_first: u8,
        aka_flag: bool,
        names: [String; 4],
    },
    StartKyoku {
        #[serde(deserialize_with = "Pai::deserialize_mjai_str")]
        bakaze: Pai,
        #[serde(deserialize_with = "Pai::deserialize_mjai_str")]
        dora_marker: Pai,
        kyoku: u8, // counts from 1
        honba: u8,
        kyotaku: u8,
        oya: u8,
        scores: [i32; 4],
        #[serde(deserialize_with = "deserialize_tehais_from_str")]
        tehais: [[Pai; 13]; 4],
    },

    Tsumo {
        actor: u8,
        #[serde(deserialize_with = "Pai::deserialize_mjai_str")]
        pai: Pai,
    },
    Dahai {
        actor: u8,
        #[serde(deserialize_with = "Pai::deserialize_mjai_str")]
        pai: Pai,
        tsumogiri: bool,
    },

    Chi {
        actor: u8,
        target: u8,
        #[serde(deserialize_with = "Pai::deserialize_mjai_str")]
        pai: Pai,
        consumed: Consumed2,
    },
    Pon {
        actor: u8,
        target: u8,
        #[serde(deserialize_with = "Pai::deserialize_mjai_str")]
        pai: Pai,
        consumed: Consumed2,
    },
    Daiminkan {
        actor: u8,
        target: u8,
        #[serde(deserialize_with = "Pai::deserialize_mjai_str")]
        pai: Pai,
        consumed: Consumed3,
    },
    Kakan {
        actor: u8,
        #[serde(deserialize_with = "Pai::deserialize_mjai_str")]
        pai: Pai,
        consumed: Consumed3,
    },
    Ankan {
        actor: u8,
        consumed: Consumed4,
    },
    Dora {
        #[serde(deserialize_with = "Pai::deserialize_mjai_str")]
        dora_marker: Pai,
    },

    Reach {
        actor: u8,
    },
    ReachAccepted {
        actor: u8,
    },

    Hora {
        actor: u8,
        target: u8,

        // it is an Option because akochan won't send this field, but we need to
        // record the field.
        #[serde(skip_serializing_if = "Option::is_none")]
        deltas: Option<[i32; 4]>,
    },
    Ryukyoku {
        #[serde(skip_serializing_if = "Option::is_none")]
        deltas: Option<[i32; 4]>,
    },

    EndKyoku,
    EndGame,
}

impl Eq for Event {}

macro_rules! make_pai_array_from_string_array {
    ($array:ident, $($index:expr),*) => {
        [$($array[$index].parse().map_err(Error::custom)?),*]
    };
}

macro_rules! build_consumed_struct {
    ($name:ident; $n:expr; $($index:expr),*) => {
        #[derive(Debug, Clone, Copy, Serialize)]
        pub struct $name(pub [Pai; $n]);

        impl PartialEq for $name {
            #[inline]
            fn eq(&self, other: &Self) -> bool {
                $(1u64 << self.0[$index].0)|*
                    == $(1u64 << other.0[$index].0)|*
            }
        }

        impl Eq for $name {}

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de> {
                    let s = <[String; $n]>::deserialize(deserializer)?;
                    Ok($name(make_pai_array_from_string_array!(s, $($index),*)))
                }
        }
    };
}

build_consumed_struct!(Consumed2; 2; 0, 1);
build_consumed_struct!(Consumed3; 3; 0, 1, 2);
build_consumed_struct!(Consumed4; 4; 0, 1, 2, 3);

fn deserialize_tehais_from_str<'de, D>(deserializer: D) -> Result<[[Pai; 13]; 4], D::Error>
where
    D: Deserializer<'de>,
{
    let s = <[[String; 13]; 4]>::deserialize(deserializer)?;
    let (s0, s1, s2, s3) = (&s[0], &s[1], &s[2], &s[3]);
    Ok([
        make_pai_array_from_string_array!(s0, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12),
        make_pai_array_from_string_array!(s1, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12),
        make_pai_array_from_string_array!(s2, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12),
        make_pai_array_from_string_array!(s3, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12),
    ])
}

impl Event {
    #[inline]
    pub fn actor(&self) -> Option<u8> {
        match *self {
            Event::Tsumo { actor, .. }
            | Event::Dahai { actor, .. }
            | Event::Chi { actor, .. }
            | Event::Pon { actor, .. }
            | Event::Daiminkan { actor, .. }
            | Event::Kakan { actor, .. }
            | Event::Ankan { actor, .. }
            | Event::Reach { actor, .. }
            | Event::ReachAccepted { actor, .. }
            | Event::Hora { actor, .. } => Some(actor),
            _ => None,
        }
    }

    #[inline]
    pub fn naki_info(&self) -> Option<(u8, Pai)> {
        match *self {
            Event::Chi { target, pai, .. }
            | Event::Pon { target, pai, .. }
            | Event::Daiminkan { target, pai, .. } => Some((target, pai)),
            _ => None,
        }
    }
}
