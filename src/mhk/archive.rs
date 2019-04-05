use super::{
    chunks::*,
    utility::*,
    MhkError,
};
use crate::filesystem::{
    AsyncRead,
    Buffered,
    Narrow,
};
use std::collections::HashMap;

#[derive(Debug)]
pub struct MhkArchive<R>
where
    R: AsyncRead,
{
    pub handle: std::rc::Rc<Buffered<R>>,
    pub files: Vec<FileInfo>,
    pub resources: HashMap<String, HashMap<u16, ResourceInfo>>,
}

#[derive(Debug, Clone)]
pub struct FileInfo {
    offset: u64,
    size: u64,
}

#[derive(Debug)]
pub struct ResourceInfo {
    ty: String,
    id: u16,
    file_table_index: usize,
}

impl<R> MhkArchive<R>
where
    R: AsyncRead,
{
    pub async fn new(unbuffered_handle: R) -> Result<Self, MhkError> {
        let handle = unbuffered_handle.buffer(8192);

        // do some sanity checks and load in the basic info
        let mut pos = 0;

        let mhwk: MHWK = await!(deserialize_from(&handle, &mut pos))?;
        if mhwk.signature != "MHWK".as_bytes() {
            return Err(MhkError::InvalidFormat("bad signature (MHWK)"));
        }

        let rsrc: RSRC = await!(deserialize_from(&handle, &mut pos))?;
        if rsrc.signature != "RSRC".as_bytes() {
            return Err(MhkError::InvalidFormat("bad signature (RSRC)"));
        }

        // go to the resource dir
        pos = rsrc.resource_dir_offset as u64;

        // this one is a weirdo, right at the beginning of the resource dir
        let _name_list_offset: u16 =
            await!(deserialize_from(&handle, &mut pos))?;

        // read in the type table
        let type_table: Vec<TypeTableEntry> =
            await!(deserialize_u16_table_from(&handle, &mut pos))?;

        // go and read the name and resource tables for each type
        // let mut name_tables: Vec<Vec<NameTableEntry>> = Vec::with_capacity(type_table.len());
        let mut resource_tables: Vec<Vec<ResourceTableEntry>> =
            Vec::with_capacity(type_table.len());
        for entry in &type_table {
            pos = rsrc.resource_dir_offset as u64
                + entry.resource_table_offset as u64;
            resource_tables
                .push(await!(deserialize_u16_table_from(&handle, &mut pos))?);
            // pos = rsrc.resource_dir_offset as u64 + entry.name_table_offset as u64;
            // name_tables.push(await!(deserialize_u16_table_from(&handle, &mut pos))?);
        }

        // go to the file table and read it in
        pos = rsrc.resource_dir_offset as u64 + rsrc.file_table_offset as u64;
        let file_table: Vec<FileTableEntry> =
            await!(deserialize_u32_table_from(&handle, &mut pos))?;

        // convert the file table into something more useful for us
        let files: Vec<FileInfo> = file_table
            .iter()
            .map(|e| {
                FileInfo {
                    offset: e.offset as u64,
                    size: (e.size_high as u64) << 16 | e.size_low as u64,
                }
            })
            .collect();

        // ok, now we need to construct the resource type / id / names maps
        let mut resources = HashMap::with_capacity(type_table.len());
        for (i, entry) in type_table.iter().enumerate() {
            let ty = std::str::from_utf8(&entry.resource_type)?.to_owned();
            let resource_table = &resource_tables[i];
            // let name_table = &name_tables[i];
            let mut ids = HashMap::with_capacity(resource_table.len());

            for rentry in resource_table {
                if rentry.file_table_index as usize - 1 >= files.len() {
                    return Err(MhkError::InvalidFormat(
                        "bad file table index",
                    ));
                }

                // normally here we'd look for a name, but... no. just no.

                let info = ResourceInfo {
                    ty: ty.clone(),
                    id: rentry.resource_id,
                    file_table_index: rentry.file_table_index as usize - 1,
                };
                ids.insert(rentry.resource_id, info);
            }

            resources.insert(ty, ids);
        }

        Ok(MhkArchive {
            handle: std::rc::Rc::new(handle),
            files,
            resources,
        })
    }

    pub fn open<T>(
        &self,
        typ: T,
        i: u16,
    ) -> Result<Narrow<std::rc::Rc<Buffered<R>>>, MhkError>
    where
        T: crate::ResourceType,
    {
        self.resources
            .get(typ.name())
            .and_then(|e| e.get(&i))
            .and_then(|info| self.files.get(info.file_table_index))
            .map(|info| self.handle.narrow(info.offset, info.size))
            .ok_or(MhkError::ResourceNotFound(None, typ.name(), i))
    }
}
