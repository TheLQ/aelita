use crate::{StorDieselError, StorDieselResult};
use diesel::sql_types::{Nullable, Text};
use serde::{Deserialize, Serialize};
use std::ffi::{OsStr, OsString};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

pub const HD_PATH_DEPTH: usize = 11;

// #[derive(diesel::Insertable, Serialize, Deserialize)]
// #[diesel(table_name = crate::schema_temp::fast_hd_components)]
// #[diesel(check_for_backend(diesel::mysql::Mysql))]
// pub struct FastHdComponent {
//     pub(crate) value: Vec<u8>,
// }

#[derive(diesel::HasQuery, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::hd1_files_parents)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct HdPathAssociation {
    id: u32,
    parent_id: Option<u32>,
}

#[derive(diesel::Insertable, Serialize, Deserialize, Eq, PartialEq, Hash)]
#[diesel(table_name = crate::schema::hd1_files_parents)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct NewHdPathAssociation {
    pub id: u32,
    pub parent_id: u32,
}

#[derive(diesel::HasQuery, diesel::Insertable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema_temp::fast_hd_paths)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct HdPathDiesel {
    p0: Option<Vec<u8>>,
    p1: Option<Vec<u8>>,
    p2: Option<Vec<u8>>,
    p3: Option<Vec<u8>>,
    p4: Option<Vec<u8>>,
    p5: Option<Vec<u8>>,
    p6: Option<Vec<u8>>,
    p7: Option<Vec<u8>>,
    p8: Option<Vec<u8>>,
    p9: Option<Vec<u8>>,
    p10: Option<Vec<u8>>,
}

impl HdPathDiesel {
    pub fn from_path(path: &Path) -> Self {
        fn next_os_str<'s>(iter: &mut impl Iterator<Item = &'s OsStr>) -> Option<Vec<u8>> {
            iter.next().map(|v| v.as_bytes().to_vec())
        }

        let mut components = path.iter();
        let new = Self {
            p0: next_os_str(&mut components),
            p1: next_os_str(&mut components),
            p2: next_os_str(&mut components),
            p3: next_os_str(&mut components),
            p4: next_os_str(&mut components),
            p5: next_os_str(&mut components),
            p6: next_os_str(&mut components),
            p7: next_os_str(&mut components),
            p8: next_os_str(&mut components),
            p9: next_os_str(&mut components),
            p10: next_os_str(&mut components),
        };
        assert_eq!(components.next(), None, "path too long");
        new
    }

    // pub fn unroll_path(self) -> StorDieselResult<Vec<String>> {
    //     if self.p8.is_some() {
    //         return Err(StorDieselError::query_fail("row too long"));
    //     }
    //     let arr = self.into_array();
    //
    //     // sanity
    //     let mut empty_mode = false;
    //     for component in &arr {
    //         if empty_mode {
    //             if component.is_some() {
    //                 return Err(StorDieselError::query_fail("sanity fail"));
    //             }
    //         } else {
    //             if component.is_none() {
    //                 empty_mode = true;
    //             }
    //         }
    //     }
    //
    //     Ok(arr
    //         .into_iter()
    //         .take_while(|v| v.is_some())
    //         .map(|v| v.unwrap())
    //         .collect())
    // }

    pub fn into_array(self) -> [Option<Vec<u8>>; HD_PATH_DEPTH] {
        let Self {
            p0,
            p1,
            p2,
            p3,
            p4,
            p5,
            p6,
            p7,
            p8,
            p9,
            p10,
        } = self;
        [p0, p1, p2, p3, p4, p5, p6, p7, p8, p9, p10]
    }
}
