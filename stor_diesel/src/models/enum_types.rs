use crate::models::common::parse_type_checked;
use crate::schema::sql_types::{
    Hd1RootsRtypeEnum, JournalImmutableJournalTypeEnum, SpaceOwnedChildType1Enum,
    SpaceOwnedChildType2Enum, Tor1TorrentsStateEnum,
};
use aelita_xrn::defs::address::XrnType;
use diesel::backend::Backend;
use diesel::deserialize::FromSql;
use diesel::mysql::Mysql;
use diesel::mysql::MysqlValue;
use diesel::serialize::IsNull;
use diesel::serialize::Output;
use diesel::serialize::ToSql;
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::str::FromStr;
use xana_commons_rs::qbittorrent_re::TorrentState;

macro_rules! enum_value {
    ($diesel_type:ident -> $name:ident) => {
        /*
        todo: Assume we can use rust utf8 strings. Mysql only? Then why sqlite has a Binary+Text conversion?
        https://github.com/adwhit/diesel-derive-enum/blob/816ebe062a99056a69a194b4ba15532980558c19/src/lib.rs#L580
        */

        impl FromSql<$diesel_type, Mysql> for $name {
            fn from_sql(input_raw: MysqlValue) -> diesel::deserialize::Result<Self> {
                parse_type_checked(input_raw.as_bytes()).map_err(Into::into)
            }
        }

        impl ToSql<$diesel_type, Mysql> for $name {
            fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Mysql>) -> diesel::serialize::Result {
                let as_str: &str = self.as_ref();
                out.write_all(as_str.as_bytes())?;
                Ok(IsNull::No)
            }
        }
    };
}

#[derive(
    Debug,
    Hash,
    Eq,
    PartialEq,
    Clone,
    diesel::expression::AsExpression,
    diesel::deserialize::FromSqlRow,
    strum::EnumString,
    strum::AsRefStr,
    strum::VariantArray,
    strum::Display,
)]
#[diesel(sql_type = JournalImmutableJournalTypeEnum)]
pub enum ModelJournalTypeName {
    QbGetTorJson1,
    NData1,
    ChangeOp1,
}
enum_value!(JournalImmutableJournalTypeEnum -> ModelJournalTypeName);

#[derive(
    Debug,
    Hash,
    Eq,
    PartialEq,
    Clone,
    diesel::expression::AsExpression,
    diesel::deserialize::FromSqlRow,
    strum::EnumString,
    strum::AsRefStr,
    strum::VariantArray,
    strum::Display,
    Serialize,
    Deserialize,
)]
#[diesel(sql_type = Hd1RootsRtypeEnum)]
pub enum ModelHdRoot {
    ZfsDataset,
    Project,
    Movie,
    Episodes,
}
enum_value!(Hd1RootsRtypeEnum -> ModelHdRoot);

#[derive(
    Debug,
    Hash,
    Eq,
    PartialEq,
    diesel::expression::AsExpression,
    diesel::deserialize::FromSqlRow,
    Serialize,
    Deserialize,
)]
#[diesel(sql_type = Tor1TorrentsStateEnum)]
#[diesel(sql_type = diesel::sql_types::Text)]
#[serde(transparent)]
pub struct ModelTorrentState(TorrentState);
enum_value!(Tor1TorrentsStateEnum -> ModelTorrentState);

impl ModelTorrentState {
    pub fn inner(&self) -> &TorrentState {
        &self.0
    }

    pub fn into_inner(self) -> TorrentState {
        self.0
    }
}

impl From<TorrentState> for ModelTorrentState {
    fn from(value: TorrentState) -> Self {
        Self(value)
    }
}

impl<'t> From<&'t TorrentState> for ModelTorrentState {
    fn from(value: &'t TorrentState) -> Self {
        Self(value.clone())
    }
}

impl AsRef<str> for ModelTorrentState {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

/// strum passthru
impl FromStr for ModelTorrentState {
    type Err = strum::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        TorrentState::from_str(s).map(|inner| Self(inner))
    }
}

/// strum passthru
impl<'s> From<&'s ModelTorrentState> for &'static str {
    fn from(value: &'s ModelTorrentState) -> Self {
        Self::from(&value.0)
    }
}

/// strum passthru
impl From<ModelTorrentState> for &'static str {
    fn from(value: ModelTorrentState) -> Self {
        Self::from(&value.0)
    }
}

#[derive(
    Debug,
    Hash,
    Eq,
    PartialEq,
    diesel::expression::AsExpression,
    diesel::deserialize::FromSqlRow,
    Serialize,
    Deserialize,
)]
#[diesel(sql_type = SpaceOwnedChildType1Enum)]
#[diesel(sql_type = SpaceOwnedChildType2Enum)]
#[diesel(sql_type = diesel::sql_types::Text)]
pub struct AnyEnumToText(String);

impl AnyEnumToText {
    pub fn new(input: impl Into<String>) -> Self {
        Self(input.into())
    }
}

impl AsRef<str> for AnyEnumToText {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl From<XrnType> for AnyEnumToText {
    fn from(value: XrnType) -> Self {
        Self(value.as_ref().to_string())
    }
}

macro_rules! enum_to_string_map {
    ($diesel_type:ident) => {
        impl<DB: Backend> FromSql<$diesel_type, DB> for AnyEnumToText
        where
            String: FromSql<diesel::sql_types::Text, DB>,
        {
            fn from_sql(input_raw: DB::RawValue<'_>) -> diesel::deserialize::Result<Self> {
                let value = <String as FromSql<diesel::sql_types::Text, DB>>::from_sql(input_raw)?;
                Ok(Self(value))
            }
        }

        impl<DB: Backend> ToSql<$diesel_type, DB> for AnyEnumToText
        where
            str: ToSql<diesel::sql_types::Text, DB>,
        {
            fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, DB>) -> diesel::serialize::Result {
                let value: &str = self.0.as_ref();
                <str as ToSql<diesel::sql_types::Text, DB>>::to_sql(&value, out)
            }
        }
    };
}
enum_to_string_map!(SpaceOwnedChildType1Enum);
enum_to_string_map!(SpaceOwnedChildType2Enum);
