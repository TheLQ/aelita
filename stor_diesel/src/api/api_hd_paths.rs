use crate::{
    CompressedPaths, HdPathAssociation, NewHdPathAssociation, StorDieselResult, StorTransaction,
    schema,
};
use diesel::{QueryDsl, RunQueryDsl, dsl};
use indexmap::IndexSet;
use itertools::Itertools;
use std::collections::HashMap;
use xana_commons_rs::num_format_re::ToFormattedString;
use xana_commons_rs::tracing_re::{debug, info, trace};
use xana_commons_rs::{BasicWatch, LOCALE};

/// Rest thy tired SQL brain in favor of olde expressive Rust
pub struct HdAssociationsBuilder<'t, 'e> {
    conn: &'t mut StorTransaction<'e>,
    associations: Vec<HdPathAssociation>,
    key_to_associations: HashMap<NewHdPathAssociation, usize>,
    queued_keys: IndexSet<NewHdPathAssociation>,
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
        let next_id = schema::hd1_files_parents::table
            .select(dsl::max(schema::hd1_files_parents::tree_id))
            .get_result::<Option<u32>>(conn.inner())?
            .unwrap_or(1);

        let mut builder = Self {
            conn,
            associations: vec![HdPathAssociation {
                tree_id: u32::MAX,
                component_id: u32::MAX,
                parent_id: None,
                tree_depth: u32::MAX,
            }],
            key_to_associations: HashMap::new(),
            queued_keys: IndexSet::new(),
            remain: build_remaining(compressed_paths, component_to_id),
            next_id,
            new_ids_start: next_id,
        };
        builder.start()?;
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
        self.remain.retain_mut(|v| {
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
                    v.next_depth += 1;
                } else {
                    self.queued_keys.insert(key);
                    break;
                }
            }

            !v.path_by_comp_id_reversed.is_empty()
        });
        trace!("remain retain check in {watch}");

        if self.queued_keys.is_empty() {
            panic!("wut?");
        }
        trace!("queued {} keys", self.queued_keys.len());

        if self.queued_keys.len() > 0 {
            for key in self.queued_keys.iter().take(5) {
                debug!("queued {key:?}")
            }
            // panic!("wut?")
        }

        let watch = BasicWatch::start();
        let mut query_input = Default::default();
        std::mem::swap(&mut self.queued_keys, &mut query_input);
        let mut query_input = query_input.into_iter().collect::<Vec<_>>();
        const CHUNK_SIZE: usize = 5_000_000;

        // let rem_start = query_input.len() - (query_input.len() % CHUNK_SIZE);
        // let remainder: Vec<NewHdPathAssociation> = query_input.drain(rem_start..).collect();
        // if !remainder.is_empty() {
        //     self.upsert_associations(remainder)?;
        // }

        let chunks = query_input
            .into_iter()
            .chunks(CHUNK_SIZE)
            .into_iter()
            .map(|v| v.collect())
            .collect::<Vec<_>>();
        debug!("chunk-ification associations in {watch}");

        let watch = BasicWatch::start();
        let total_chunks = chunks.len() - 1;
        for (chunk_i, chunk) in chunks.into_iter().enumerate() {
            trace!("select chunk {chunk_i} of {total_chunks}");
            self.upsert_associations(chunk)?;
        }
        debug!("upsert associations in {watch}");

        Ok(())
    }

    fn upsert_associations(
        &mut self,
        mut input: Vec<NewHdPathAssociation>,
    ) -> StorDieselResult<()> {
        assert!(!input.is_empty());
        let mut query_builder = "\
            SELECT p.tree_id, p.component_id, p.parent_id, p.tree_depth \
            FROM `hd1_files_parents` p \
            WHERE "
            .to_string();

        for NewHdPathAssociation {
            component_id,
            parent_id,
            tree_depth,
        } in &input
        {
            query_builder.push_str(&format!(
                "(component_id={component_id} AND tree_depth={tree_depth} AND parent_id{})OR",
                if let Some(parent_id) = parent_id {
                    format!("={parent_id}")
                } else {
                    " IS NULL".to_string()
                }
            ));
        }
        query_builder.truncate(query_builder.len() - 2);
        let rows =
            diesel::sql_query(query_builder).get_results::<HdPathAssociation>(self.conn.inner())?;

        for v in rows {
            let pos = input.iter().position(
                |NewHdPathAssociation {
                     component_id,
                     parent_id,
                     tree_depth,
                 }| {
                    v.component_id == *component_id
                        && v.parent_id == *parent_id
                        && v.tree_depth == *tree_depth
                },
            );
            if let Some(pos) = pos {
                input.remove(pos);
            }
        }

        // input.retain(
        //     |NewHdPathAssociation {
        //          component_id,
        //          parent_id,
        //          tree_depth,
        //      }| {
        //         rows.iter()
        //             .filter(|v| {
        //                 v.component_id == *component_id
        //                     && v.parent_id == *parent_id
        //                     && v.tree_depth == *tree_depth
        //             })
        //             .next()
        //             .is_none()
        //     },
        // );

        // fake insert new input
        for new in input {
            let index = self.associations.len();
            let assoc = HdPathAssociation::from_partial(new, self.next_id);
            self.next_id += 1;
            self.key_to_associations
                .insert(NewHdPathAssociation::from_full_ref(&assoc), index);
            self.associations.push(assoc);
        }

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
            next_depth: 0,
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
    next_depth: u32,
}
