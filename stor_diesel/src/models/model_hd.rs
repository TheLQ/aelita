use crate::{StorDieselError, StorDieselResult};
use diesel::sql_types::{Integer, Nullable, Text, Unsigned};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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
    p0: Option<u32>,
    p1: Option<u32>,
    p2: Option<u32>,
    p3: Option<u32>,
    p4: Option<u32>,
    p5: Option<u32>,
    p6: Option<u32>,
    p7: Option<u32>,
    p8: Option<u32>,
    p9: Option<u32>,
    p10: Option<u32>,
}

impl HdPathDiesel {
    pub fn from_path(path: &Path, component_to_id: &HashMap<String, u32>) -> Self {
        let mut components = path.iter();
        let mut next_os_str = || {
            components
                .next()
                .map(|v| *component_to_id.get(v.to_str().unwrap()).unwrap())
        };

        // fn next_os_str<'s>(
        //     iter: &mut impl Iterator<Item = &'s OsStr>,
        //     component_to_id: &HashMap<String, u32>,
        // ) -> Option<Vec<u8>> {
        //     iter.next().map(|v| v.as_bytes().to_vec())
        // }

        let new = Self {
            p0: next_os_str(),
            p1: next_os_str(),
            p2: next_os_str(),
            p3: next_os_str(),
            p4: next_os_str(),
            p5: next_os_str(),
            p6: next_os_str(),
            p7: next_os_str(),
            p8: next_os_str(),
            p9: next_os_str(),
            p10: next_os_str(),
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

    pub fn into_array(self) -> [Option<u32>; HD_PATH_DEPTH] {
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
