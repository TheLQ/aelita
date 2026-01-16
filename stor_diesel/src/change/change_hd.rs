use crate::api::api_space_mut::storapi_space_owned_new;
use crate::change::defs::{ChangeContext, Changer};
use crate::err::StorDieselErrorKind;
use crate::{
    DisplayCompPath, ModelFileTreeId, ModelHdRoot, ModelSpaceId, ModelSpaceOwned, NewHdRoot,
    NewModelSpaceName, StorDieselResult, StorIdTypeDiesel, StorTransaction, components_get_bytes,
    components_upsert_cte, convert_comps_to_path, convert_path_to_comps,
    storapi_hd_get_path_by_path, storapi_hd_links_add, storapi_hd_tree_push,
    storapi_hd_tree_push_single, storapi_hdroots_push, storapi_space_get,
    storapi_space_get_ids_by_name,
};
use aelita_xrn::defs::path_xrn::{PathXrn, PathXrnType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use xana_commons_rs::CrashErrKind;
use xana_commons_rs::tracing_re::info;
use xana_fs_indexer_rs::{CompressedPaths, ScanFileTypeWithPath, ScanStat};

#[derive(Debug, Serialize, Deserialize)]
pub struct HdAddPath {
    /// components * path * new_paths
    pub paths: Vec<(ScanFileTypeWithPath, ScanStat)>,
}
impl Changer for HdAddPath {
    type Result = ();

    fn commit_change(
        self,
        conn: &mut StorTransaction,
        _ctx: ChangeContext,
    ) -> StorDieselResult<()> {
        let Self { paths } = self;

        let preview = 5;

        if paths.is_empty() {
            panic!("no empty")
        } else if paths.len() == 1 {
            for path in paths {
                commit_add_path_sql(conn, path)?;
            }
            Ok(())
        } else {
            for (scan_type, _stat) in paths.iter().take(preview) {
                info!("Add path {}", scan_type.path().display());
            }
            if paths.len() >= preview {
                info!("... to len {}", paths.len())
            }
            // todo: this is expensive for 1 path...
            let new_paths = CompressedPaths::from_scan(paths, false)
                .map_err(StorDieselErrorKind::InvalidChangeCompressedPaths.xana_map())?;
            storapi_hd_tree_push(conn, new_paths)
        }
    }
}

fn commit_add_path_sql(
    conn: &mut StorTransaction,
    (scan_type, stat): (ScanFileTypeWithPath, ScanStat),
) -> StorDieselResult<ModelFileTreeId> {
    let path = scan_type.path();
    let path_comps = convert_path_to_comps(path)?;
    components_upsert_cte(conn, &path_comps)?;
    let existing_ids = storapi_hd_get_path_by_path(conn, &path_comps)?;

    if path_comps.len() == existing_ids.len() {
        Err(StorDieselErrorKind::PathAlreadyExists.build_message(path.display()))
    } else if path_comps.len() - 1 == existing_ids.len() {
        // perfect, add a single batch
        let parent = existing_ids.last().unwrap();
        let file_comp = path_comps.last().unwrap();
        let file_id = storapi_hd_tree_push_single(conn, Some(*parent), &[(file_comp, stat)])?;
        // get stats
        Ok(file_id)
    } else {
        // todo adding more than once starts making the SQL equivalent of CompressedPaths
        Err(StorDieselErrorKind::PathFileParentMissing.build_message(path.display()))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HdAddSymlink {
    pub at: Vec<Vec<u8>>,
    pub target: Vec<Vec<u8>>,
}
impl Changer for HdAddSymlink {
    type Result = ();

    fn commit_change(
        self,
        conn: &mut StorTransaction,
        _ctx: ChangeContext,
    ) -> StorDieselResult<()> {
        let Self { at, target } = self;
        info!(
            "Link {} to {}",
            DisplayCompPath(at.as_slice()),
            DisplayCompPath(target.as_slice())
        );
        storapi_hd_links_add(conn, at.as_slice(), target.as_slice())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HdAddRoot {
    pub source: Vec<Vec<u8>>,
    pub description: String,
    pub space_name: String,
    pub root_type: ModelHdRoot,
}
impl Changer for HdAddRoot {
    type Result = ModelSpaceId;

    fn commit_change(
        self,
        conn: &mut StorTransaction,
        ChangeContext { journal_id }: ChangeContext,
    ) -> StorDieselResult<Self::Result> {
        let Self {
            source,
            description,
            space_name,
            root_type,
        } = self;
        info!(
            "Add root {} type {root_type} name {space_name} ({description})",
            DisplayCompPath(source.as_slice()),
        );
        let space_id = storapi_hdroots_push(
            conn,
            NewModelSpaceName {
                journal_id,
                description,
                space_name,
            },
            NewHdRoot { rtype: root_type },
        )?;
        Ok(space_id)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HdAddPathToSpace {
    pub path_to_space_name: Vec<AddPathMeta>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddPathMeta {
    pub path: Vec<Vec<u8>>,
    pub space_name: String,
    pub owned_description: String,
}

impl Changer for HdAddPathToSpace {
    type Result = ();

    fn commit_change(
        self,
        conn: &mut StorTransaction,
        ChangeContext { journal_id }: ChangeContext,
    ) -> StorDieselResult<Self::Result> {
        let Self { path_to_space_name } = self;

        let mut space_names = path_to_space_name
            .iter()
            .map(|meta| meta.space_name.as_ref())
            .collect::<Vec<_>>();
        space_names.sort_unstable();
        space_names.dedup();
        let space_id_by_name = storapi_space_get_ids_by_name(conn, space_names.as_ref())?
            .into_iter()
            .collect::<HashMap<_, _>>();

        for AddPathMeta {
            path,
            space_name,
            owned_description,
        } in path_to_space_name
        {
            let Some(space_id) = space_id_by_name.get(&space_name).cloned() else {
                return Err(StorDieselErrorKind::UnknownComponent.build_message(space_name));
            };

            let path_ids = storapi_hd_get_path_by_path(conn, &path)?;
            let path_id = path_ids.last().unwrap();
            let xrn = PathXrn::new(
                PathXrnType::Fs,
                convert_comps_to_path(&path),
                path_id.inner_id(),
            );

            storapi_space_owned_new(
                conn,
                ModelSpaceOwned {
                    space_id,
                    journal_id,
                    description: Some(owned_description),
                },
                xrn.into(),
            )?;
        }

        Ok(())
    }
}
