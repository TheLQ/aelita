use crate::{
    CompNodeType, CompressedPaths, HdPathAssociation, ModelFileCompId, ModelFileTreeId,
    ModelLocalTreeId, NewHdPathAssociation, StorDieselResult, StorIdType, StorTransaction,
    storapi_hd_components_get_or_insert,
};
use diesel::{HasQuery, RunQueryDsl};
use std::collections::HashMap;

/// To use AUTO INCREMENT each pass must query the database
/// This is extremely slow with millions of rows
/// So instead calculate IDs locally (synced with db) and use pure-rust model gen
pub fn build_associations_from_compressed(
    conn: &mut StorTransaction,
    compressed: &CompressedPaths,
) -> StorDieselResult<()> {
    let mut database = AssociationCompressed::init(conn)?;
    database.upsert_components(compressed)?;

    let CompNodeType::Dir { children_node_ids } = compressed.nodes()[0].node_type() else {
        unreachable!()
    };
    for root_node_id in children_node_ids {
        let root_node = compressed.node_id(*root_node_id);
        let root_comp = database.components_to_id[root_node.name_from(&compressed)];

        let parent_id = database.association_get_or_insert(NewHdPathAssociation {
            tree_depth: 0,
            parent_id: None,
            component_id: root_comp,
        });

        match root_node.node_type() {
            CompNodeType::Dir { children_node_ids } => {
                // only the 2nd level /a/b has Some(parent_id)
                for child_node_id in children_node_ids {
                    recurse_compressed(&mut database, compressed, parent_id, *child_node_id, 1)
                }
            }
            CompNodeType::File => {
                // adding is enough
            }
            v => todo!("{v:?}"),
        }
    }
    Ok(())
}

fn recurse_compressed(
    database: &mut AssociationCompressed,
    local: &CompressedPaths,
    parent_database: ModelFileTreeId,
    node_local: ModelLocalTreeId,
    tree_depth: u32,
) {
    let node = local.node_id(node_local);
    let db_comp_id = database.components_to_id[node.name_from(&local)];

    let tree_id = database.association_get_or_insert(NewHdPathAssociation {
        tree_depth,
        component_id: db_comp_id,
        parent_id: Some(parent_database),
    });

    match node.node_type() {
        CompNodeType::Dir { children_node_ids } => {
            for id in children_node_ids {
                recurse_compressed(database, local, parent_database, *id, tree_depth + 1)
            }
        }
        CompNodeType::File => {
            // go no further
        }
        v => todo!("{v:?}"),
    }
}

/// Similar to CompressedPaths but Diesel
struct AssociationCompressed<'r, 't> {
    conn: &'r mut StorTransaction<'t>,
    associations: Vec<HdPathAssociation>,
    lookup_by_new: HashMap<NewHdPathAssociation, usize>,
    components_to_id: HashMap<Vec<u8>, ModelFileCompId>,
    new_associations_at: usize,
}

impl<'r, 't> AssociationCompressed<'r, 't> {
    fn init(conn: &'r mut StorTransaction<'t>) -> StorDieselResult<Self> {
        let associations = HdPathAssociation::query().get_results(conn.inner())?;
        for i in 0..associations.len() {
            assert_eq!(associations[i].tree_id.inner_usize(), i);
        }

        let mut lookup_by_new = HashMap::new();
        for (i, assoc) in associations.iter().enumerate() {
            lookup_by_new.insert(NewHdPathAssociation::from_full_ref(&assoc), i);
        }

        let new_associations_at = associations.len();
        Ok(Self {
            conn,
            associations,
            lookup_by_new,
            components_to_id: HashMap::new(),
            new_associations_at,
        })
    }

    fn upsert_components(&mut self, compressed: &CompressedPaths) -> StorDieselResult<()> {
        self.components_to_id = storapi_hd_components_get_or_insert(self.conn, compressed.parts())?;
        Ok(())
    }

    fn association_get_or_insert(&mut self, assoc: NewHdPathAssociation) -> ModelFileTreeId {
        if let Some(index) = self.lookup_by_new.get(&assoc) {
            self.associations[*index].tree_id
        } else {
            let new_id = ModelFileTreeId::new_usize(self.associations.len());
            let assoc = HdPathAssociation::from_partial(assoc, new_id);
            self.lookup_by_new.insert(
                NewHdPathAssociation::from_full_ref(&assoc),
                self.associations.len(),
            );
            self.associations.push(assoc);
            new_id
        }
    }
}
