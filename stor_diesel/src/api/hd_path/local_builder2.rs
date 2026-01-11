use crate::{
    CombinedStatAssociation, HdPathAssociation, ModelFileCompId, ModelFileTreeId, ModelLocalTreeId,
    NewHdPathAssociation, ScanStatDiesel, StorDieselResult, StorIdTypeDiesel, StorTransaction,
    components_upsert_cte, components_upsert_select_first, schema,
};
use diesel::{HasQuery, RunQueryDsl};
use std::collections::HashMap;
use xana_commons_rs::num_format_re::ToFormattedString;
use xana_commons_rs::tracing_re::{debug, info, warn};
use xana_commons_rs::{LOCALE, StorIdType};
use xana_fs_indexer_rs::{CompNodeType, CompressedPaths, FsNodeId, ScanStat};

/// To use AUTO INCREMENT each pass must query the database
/// This is extremely slow with millions of rows
/// So instead calculate IDs locally (synced with db) and use pure-rust model gen
pub fn build_associations_from_compressed<'p>(
    conn: &mut StorTransaction,
    compressed: &'p CompressedPaths,
) -> StorDieselResult<Vec<(HdPathAssociation, ScanStat)>> {
    let mut database = AssociationCompressed::init(conn)?;
    database.upsert_components(compressed)?;

    recurse_compressed(&mut database, compressed, None, FsNodeId::new(0), 0);
    info!(
        "total symlink good {} broken {}",
        database.total_symlink_good.to_formatted_string(&LOCALE),
        database.total_symlink_broken.to_formatted_string(&LOCALE)
    );
    let v = database
        .associations
        .drain(database.new_associations_at..)
        .collect::<Vec<_>>();
    Ok(v)
}

fn recurse_compressed(
    database: &mut AssociationCompressed,
    local: &CompressedPaths,
    parent_database: Option<ModelFileTreeId>,
    cur_local: FsNodeId,
    tree_depth: u32,
) {
    let cur_node = local.node_id(cur_local);

    let cur_database;
    let child_tree_depth;
    if cur_local.inner_id() == 0 {
        // listing root itself, which doesn't have a parent
        assert_eq!(parent_database, None);
        cur_database = None;
        child_tree_depth = 0;
    } else {
        let db_comp_id = database.components_to_id[cur_node.name_from(&local)];
        cur_database = Some(database.association_get_or_insert(
            NewHdPathAssociation {
                tree_depth,
                component_id: db_comp_id,
                parent_id: parent_database,
            },
            cur_node.stat().clone(),
        ));
        child_tree_depth = tree_depth + 1;
    }

    match cur_node.node_type() {
        CompNodeType::Dir { children_node_ids } => {
            for child_local in children_node_ids {
                recurse_compressed(
                    database,
                    local,
                    cur_database,
                    *child_local,
                    child_tree_depth,
                )
            }
        }
        CompNodeType::File => {
            // go no further
        }
        CompNodeType::BrokenSymlink { raw } => {
            database.total_symlink_broken += 1;
        }
        CompNodeType::Symlink { target_node_id } => {
            database.total_symlink_good += 1;
        }
    }
}

/// Similar to CompressedPaths but Diesel
struct AssociationCompressed<'r, 't> {
    conn: &'r mut StorTransaction<'t>,
    associations: Vec<(HdPathAssociation, ScanStat)>,
    lookup_by_new: HashMap<NewHdPathAssociation, usize>,
    components_to_id: HashMap<Vec<u8>, ModelFileCompId>,
    new_associations_at: usize,
    total_symlink_broken: usize,
    total_symlink_good: usize,
}

impl<'r, 't> AssociationCompressed<'r, 't> {
    fn init(conn: &'r mut StorTransaction<'t>) -> StorDieselResult<Self> {
        let associations_diesel = CombinedStatAssociation::query().get_results(conn.inner())?;
        for i in 0..associations_diesel.len() {
            assert_eq!(associations_diesel[i].path.tree_id.inner_usize(), i);
        }

        let mut lookup_by_new = HashMap::new();
        let mut associations = Vec::new();
        for (i, assoc) in associations_diesel.into_iter().enumerate() {
            lookup_by_new.insert(NewHdPathAssociation::from_full_ref(&assoc.path), i);
            associations.push((assoc.path, assoc.stat.into()));
        }

        let new_associations_at = associations.len();
        Ok(Self {
            conn,
            associations,
            lookup_by_new,
            components_to_id: HashMap::new(),
            new_associations_at,
            total_symlink_good: 0,
            total_symlink_broken: 0,
        })
    }

    fn upsert_components(&mut self, compressed: &CompressedPaths) -> StorDieselResult<()> {
        self.components_to_id = components_upsert_cte(self.conn, compressed.parts())?;
        // self.components_to_id = components_upsert_select_first(self.conn, compressed.parts())?
        //     .into_iter()
        //     .map(|(id, comp)| (comp, id))
        //     .collect();
        Ok(())
    }

    fn association_get_or_insert(
        &mut self,
        assoc: NewHdPathAssociation,
        stat: ScanStat,
    ) -> ModelFileTreeId {
        if let Some(index) = self.lookup_by_new.get(&assoc) {
            self.associations[*index].0.tree_id
        } else {
            let new_id = ModelFileTreeId::new_usize(self.associations.len());
            let assoc = HdPathAssociation::from_partial(assoc, new_id);
            self.lookup_by_new.insert(
                NewHdPathAssociation::from_full_ref(&assoc),
                self.associations.len(),
            );
            self.associations.push((assoc, stat));
            new_id
        }
    }
}
