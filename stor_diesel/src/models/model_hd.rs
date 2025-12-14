use crate::{StorDieselError, StorDieselResult};
use diesel::sql_types::{Nullable, Text};
use serde::{Deserialize, Serialize};

pub const HD_PATH_DEPTH: usize = 9;

#[derive(diesel::QueryableByName, Serialize, Deserialize)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct HdPathDiesel {
    #[diesel(sql_type = Nullable<Text>)]
    p0: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    p1: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    p2: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    p3: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    p4: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    p5: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    p6: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    p7: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    p8: Option<String>,
}

impl HdPathDiesel {
    pub fn unroll_path(self) -> StorDieselResult<Vec<String>> {
        if self.p8.is_some() {
            return Err(StorDieselError::query_fail("row too long"));
        }
        let arr = self.into_array();

        // sanity
        let mut empty_mode = false;
        for component in &arr {
            if empty_mode {
                if component.is_some() {
                    return Err(StorDieselError::query_fail("sanity fail"));
                }
            } else {
                if component.is_none() {
                    empty_mode = true;
                }
            }
        }

        Ok(arr
            .into_iter()
            .take_while(|v| v.is_some())
            .map(|v| v.unwrap())
            .collect())
    }

    fn into_array(self) -> [Option<String>; HD_PATH_DEPTH] {
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
        } = self;
        [p0, p1, p2, p3, p4, p5, p6, p7, p8]
    }
}
