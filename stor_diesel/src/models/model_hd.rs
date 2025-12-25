use crate::StorDieselResult;
use crate::err::StorDieselErrorKind;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::{Component, Path};
use xana_commons_rs::CrashErrKind;

pub const HD_PATH_DEPTH: usize = 11;

#[derive(diesel::HasQuery, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::hd1_files_parents)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct HdPathAssociation {
    pub tree_id: u32,
    pub component_id: u32,
    pub parent_id: Option<u32>,
    pub tree_depth: u32,
}

#[derive(diesel::Insertable, Serialize, Deserialize, Eq, PartialEq, Hash)]
#[diesel(table_name = crate::schema::hd1_files_parents)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct NewHdPathAssociation {
    pub component_id: u32,
    pub parent_id: u32,
    pub tree_depth: u32,
}

#[derive(diesel::Insertable, Serialize, Deserialize, Eq, PartialEq, Hash)]
#[diesel(table_name = crate::schema::hd1_files_parents)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct NewHdPathAssociationRoot {
    pub component_id: u32,
    pub tree_depth: u32,
}

#[derive(diesel::HasQuery, diesel::Insertable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::hd1_files_paths)]
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
    pub fn from_path(
        path: &Path,
        component_to_id: &HashMap<String, u32>,
    ) -> StorDieselResult<Self> {
        let component_ids = path_components(path, |c| {
            let key = c.to_str().unwrap();
            *component_to_id.get(key).unwrap()
        })?;
        let mut iter = component_ids.into_iter();

        let new = Self {
            p0: iter.next(),
            p1: iter.next(),
            p2: iter.next(),
            p3: iter.next(),
            p4: iter.next(),
            p5: iter.next(),
            p6: iter.next(),
            p7: iter.next(),
            p8: iter.next(),
            p9: iter.next(),
            p10: iter.next(),
        };
        assert_eq!(iter.next(), None, "path too long");
        Ok(new)
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

pub fn path_components<'p, R>(
    path: &'p Path,
    map: impl Fn(&'p OsStr) -> R,
) -> StorDieselResult<Vec<R>> {
    let mut component_strs = Vec::new();
    let mut components = path.components();
    let Some(root) = components.next() else {
        return Err(StorDieselErrorKind::EmptyPath.build_message(path.display()));
    };
    if root != Component::RootDir {
        return Err(StorDieselErrorKind::PathNotAbsolute.build_message(path.display()));
    }
    for component in components {
        let os_str = match component {
            Component::Normal(v) => v,
            _unknown => {
                return Err(StorDieselErrorKind::PathWeird.build_message(path.display()));
            }
        };
        component_strs.push(map(os_str));
    }

    Ok(component_strs)
}

#[cfg(test)]
mod test {
    use crate::path_components;
    use std::path::Path;

    #[test]
    fn is_component() {
        let path = Path::new("/foo/bar");
        let comp = path_components(path, |o| o.to_str().unwrap()).unwrap();
        assert_eq!(comp, vec!["foo", "bar"]);

        let path = Path::new("/");
        let comp = path_components(path, |o| o.to_str().unwrap()).unwrap();
        assert!(comp.is_empty());
    }
}
