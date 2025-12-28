use crate::api::common::check_insert_num_rows;
use crate::err::StorDieselErrorKind;
use crate::{
    CompressedPaths, HdPathAssociation, NewHdPathAssociation, StorDieselResult, StorTransaction,
    schema,
};
use diesel::{HasQuery, RunQueryDsl};
use itertools::Itertools;
use rayon::iter::ParallelIterator;
use rayon::prelude::IntoParallelIterator;
use std::collections::{HashMap, HashSet};
use xana_commons_rs::num_format_re::ToFormattedString;
use xana_commons_rs::tracing_re::{debug, error, info, trace};
use xana_commons_rs::{BasicWatch, CommaJoiner, CrashErrKind, LOCALE};

/// Rest thy tired SQL brain in favor of olde expressive Rust
pub struct HdAssociationsBuilder<'t, 'e> {
    conn: &'t mut StorTransaction<'e>,
    associations: Vec<HdPathAssociation>,
    key_to_associations: HashMap<NewHdPathAssociation, usize>,
    remain: Vec<RemainingPath>,
    next_id: u32,
    new_ids_start: u32,
}

impl<'t, 'e> HdAssociationsBuilder<'t, 'e> {
    pub fn build(
        conn: &'t mut StorTransaction<'e>,
        compressed_paths: CompressedPaths,
        component_to_id: &HashMap<String, u32>,
    ) -> StorDieselResult<()> {
        let watch = BasicWatch::start();
        // dump the entire associations table
        let mut associations = HdPathAssociation::query().get_results(conn.inner())?;
        // associations.insert(
        //     0,
        //     HdPathAssociation {
        //         tree_id: u32::MAX - 100,
        //         component_id: u32::MAX - 100,
        //         parent_id: None,
        //         tree_depth: u32::MAX - 100,
        //     },
        // );

        // umm
        // for (i, association) in associations.iter().enumerate() {
        //     if i != 0 {
        //         assert_eq!(i, association.tree_id as usize);
        //     }
        // }
        let next_id = u32::try_from(associations.len()).unwrap();

        let mut builder = Self {
            conn,
            associations: vec![],
            key_to_associations: HashMap::new(),
            remain: build_remaining(compressed_paths, component_to_id),
            next_id,
            new_ids_start: next_id,
        };
        builder.start()?;
        let associations_len = builder.associations.len();
        builder.insert()?;

        info!("Build {associations_len} in {watch}");

        Ok(())
    }

    fn start(&mut self) -> StorDieselResult<()> {
        let mut loops = 0;
        while !self.remain.is_empty() {
            info!("iteration {loops}");
            self.iteration()?;
            loops += 1;
        }
        Ok(())
    }

    fn iteration(&mut self) -> StorDieselResult<()> {
        let watch = BasicWatch::start();

        let mut remain = Default::default();
        std::mem::swap(&mut self.remain, &mut remain);
        let (remain, queued_keys_joined): (Vec<_>, HashSet<_>) = remain
            .into_par_iter()
            .filter_map(|mut v| {
                let mut queued_keys = Vec::new();
                while !v.path_by_comp_id_reversed.is_empty() {
                    let key;
                    if let Some(parent_association_index) = v.resolved_associations.last() {
                        let parent_association: &HdPathAssociation =
                            &self.associations[*parent_association_index];
                        key = NewHdPathAssociation {
                            parent_id: Some(parent_association.tree_id),
                            component_id: *v.path_by_comp_id_reversed.last().unwrap(),
                            tree_depth: parent_association.tree_depth + 1,
                        };
                    } else {
                        // first component
                        key = NewHdPathAssociation {
                            parent_id: None,
                            component_id: *v.path_by_comp_id_reversed.last().unwrap(),
                            tree_depth: 0,
                        };
                    }

                    if let Some(existing) = self.key_to_associations.get(&key) {
                        v.resolved_associations.push(*existing);
                        v.path_by_comp_id_reversed.pop();
                    } else {
                        queued_keys.push(key);
                        break;
                    }
                }

                if v.path_by_comp_id_reversed.is_empty() {
                    assert!(queued_keys.is_empty());
                    None
                } else {
                    Some((v, queued_keys))
                }
            })
            .unzip();
        trace!("remain retain check in {watch}");
        self.remain = remain;
        let queued_keys = queued_keys_joined
            .into_iter()
            .flatten()
            .collect::<HashSet<_>>();
        trace!("remain retain check in {watch}");

        trace!("queued {} keys", queued_keys.len());

        if self.remain.is_empty() {
            trace!("exit on empty queue");
            assert!(queued_keys.is_empty());
        }

        // if queued_keys.len() > 0 {
        //     for key in queued_keys.iter().take(5) {
        //         debug!("queued {key:?}")
        //     }
        //     // panic!("wut?")
        // }

        // todo delete? chunk
        let watch = BasicWatch::start();
        // let query_input = queued_keys.into_iter().collect::<Vec<_>>();
        // const CHUNK_SIZE: usize = 5_000_000;
        //
        // let chunks = query_input
        //     .into_iter()
        //     .chunks(CHUNK_SIZE)
        //     .into_iter()
        //     .map(|v| v.collect())
        //     .collect::<Vec<_>>();
        // debug!("chunk-ification associations in {watch}");
        //
        // let watch = BasicWatch::start();
        // let total_chunks = chunks.len() - 1;
        // for (chunk_i, chunk) in chunks.into_iter().enumerate() {
        //     trace!("select chunk {chunk_i} of {total_chunks}");
        //     // self.upsert_associations(chunk)?;
        //
        // }
        // todo instead? fake insert new input
        for new in queued_keys {
            let index = self.associations.len();
            let assoc = HdPathAssociation::from_partial(new, self.next_id);
            self.next_id += 1;
            self.key_to_associations
                .insert(NewHdPathAssociation::from_full_ref(&assoc), index);
            self.associations.push(assoc);
        }
        debug!("upsert associations in {watch}");

        Ok(())
    }

    // fn upsert_associations(&mut self, input: Vec<NewHdPathAssociation>) -> StorDieselResult<()> {
    //     assert!(!input.is_empty());
    //     // let mut query_builder = "\
    //     //     SELECT p.tree_id, p.component_id, p.parent_id, p.tree_depth \
    //     //     FROM `hd1_files_parents` p \
    //     //     WHERE "
    //     //     .to_string();
    //     //
    //     // for NewHdPathAssociation {
    //     //     component_id,
    //     //     parent_id,
    //     //     tree_depth,
    //     // } in &input
    //     // {
    //     //     query_builder.push_str(&format!(
    //     //         "(component_id={component_id} AND tree_depth={tree_depth} AND parent_id{})OR",
    //     //         if let Some(parent_id) = parent_id {
    //     //             format!("={parent_id}")
    //     //         } else {
    //     //             " IS NULL".to_string()
    //     //         }
    //     //     ));
    //     // }
    //     // query_builder.truncate(query_builder.len() - 2);
    //     // let rows =
    //     //     diesel::sql_query(query_builder).get_results::<HdPathAssociation>(self.conn.inner())?;
    //
    //     // for v in rows {
    //     //     let pos = input.iter().position(
    //     //         |NewHdPathAssociation {
    //     //              component_id,
    //     //              parent_id,
    //     //              tree_depth,
    //     //          }| {
    //     //             v.component_id == *component_id
    //     //                 && v.parent_id == *parent_id
    //     //                 && v.tree_depth == *tree_depth
    //     //         },
    //     //     );
    //     //     if let Some(pos) = pos {
    //     //         input.remove(pos);
    //     //     }
    //     // }
    //
    //     // input.retain(
    //     //     |NewHdPathAssociation {
    //     //          component_id,
    //     //          parent_id,
    //     //          tree_depth,
    //     //      }| {
    //     //         rows.iter()
    //     //             .filter(|v| {
    //     //                 v.component_id == *component_id
    //     //                     && v.parent_id == *parent_id
    //     //                     && v.tree_depth == *tree_depth
    //     //             })
    //     //             .next()
    //     //             .is_none()
    //     //     },
    //     // );
    //
    //     // fake insert new input
    //     for new in input {
    //         let index = self.associations.len();
    //         let assoc = HdPathAssociation::from_partial(new, self.next_id);
    //         self.next_id += 1;
    //         self.key_to_associations
    //             .insert(NewHdPathAssociation::from_full_ref(&assoc), index);
    //         self.associations.push(assoc);
    //     }
    //
    //     Ok(())
    // }

    fn insert(mut self) -> StorDieselResult<()> {
        let new = self
            .associations
            .drain(self.new_ids_start as usize..)
            .collect::<Vec<_>>();
        let expected_len = new.len();
        trace!(
            "inserting {} rows",
            expected_len.to_formatted_string(&LOCALE)
        );
        let watch = BasicWatch::start();

        trace!("starting checks");
        for i in 0..5 {
            let exist = new.iter().filter(|v| v.tree_id == i).collect_vec();
            if exist.len() > 1 {
                for entry in &exist {
                    error!("entry {entry:?}");
                }
                panic!("uhhh {}", exist.len());
            }
        }

        let mut total_rows = 0usize;
        //
        let chunks = new.chunks(15_000_000);
        let total_chunks = chunks.len();
        for (i, chunk) in chunks.into_iter().enumerate() {
            let chunk_len = chunk.len();
            trace!("inserting chunk {i} of {total_chunks}");
            // let rows = diesel::insert_into(schema::hd1_files_parents::table)
            //     .values(chunk)
            //     .execute(self.conn.inner());

            let values = chunk
                .iter()
                .map(
                    |HdPathAssociation {
                         tree_id,
                         tree_depth,
                         component_id,
                         parent_id,
                     }| {
                        format!(
                            "({tree_id},{tree_depth},{component_id},{})",
                            if let Some(parent_id) = parent_id {
                                parent_id.to_string()
                            } else {
                                "NULL".to_string()
                            }
                        )
                    },
                )
                .collect::<CommaJoiner>();
            let rows = diesel::sql_query(format!(
                "INSERT INTO `hd1_files_parents` \
                 (tree_id,tree_depth,component_id,parent_id) \
                 VALUES {values}"
            ))
            .execute(self.conn.inner());

            if let Ok(rows) = rows {
                total_rows += rows;
                if rows != chunk_len {
                    return Err(StorDieselErrorKind::HdPathsInsertLen
                        .build_message(format!("actual {rows} expected {chunk_len}")));
                }
            }
            check_insert_num_rows(rows, chunk_len)?;
        }
        trace!(
            "inserted {} rows in {watch}",
            total_rows.to_formatted_string(&LOCALE)
        );

        Ok(())
    }
}

fn build_remaining(
    compressed_paths: CompressedPaths,
    component_to_id: &HashMap<String, u32>,
) -> Vec<RemainingPath> {
    let watch = BasicWatch::start();
    let mut remaining_components: Vec<RemainingPath> = Vec::new();
    let (indexed_parts, indexed_paths) = compressed_paths.inner();
    for path in indexed_paths {
        let mut path_by_comp_id: Vec<u32> = path
            .iter()
            .map(|v| {
                let component_str = &indexed_parts[usize::try_from(*v).unwrap()];
                component_to_id[component_str]
            })
            .collect();
        path_by_comp_id.reverse();
        remaining_components.push(RemainingPath {
            resolved_associations: Vec::new(),
            path_by_comp_id_reversed: path_by_comp_id,
        });
    }
    info!(
        "Build {} remaining in {watch}",
        remaining_components.len().to_formatted_string(&LOCALE)
    );
    remaining_components
}

struct RemainingPath {
    resolved_associations: Vec<usize>,
    path_by_comp_id_reversed: Vec<u32>,
}
