use super::chunks::{
    MHWK, RSRC, TypeTableEntry, ResourceTableEntry,
    FileTableEntry,
};
use super::utility::{
    deserialize_from, deserialize_u16_table_from, deserialize_u32_table_from,
};
use super::error::MhkError;
use super::narrow::Narrow;

use std::collections::HashMap;

use anyhow::Result;
use smol::io::{
    AsyncRead, AsyncSeek, AsyncSeekExt, SeekFrom, BufReader,
};

#[derive(Debug)]
pub struct MhkArchive<R: AsyncRead> {
    pub handle: std::rc::Rc<std::cell::RefCell<(u64, BufReader<R>)>>,
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
    R: AsyncRead + AsyncSeek + Unpin,
{
    pub async fn new(unbuffered_handle: R) -> Result<Self> {
        let mut handle = BufReader::new(unbuffered_handle);

        // do some sanity checks and load in the basic info
        let mhwk: MHWK = deserialize_from(&mut handle).await?;
        if mhwk.signature != "MHWK".as_bytes() {
            anyhow::bail!(MhkError::InvalidFormat("bad signature (MHWK)"));
        }

        let rsrc: RSRC = deserialize_from(&mut handle).await?;
        if rsrc.signature != "RSRC".as_bytes() {
            anyhow::bail!(MhkError::InvalidFormat("bad signature (RSRC)"));
        }

        // go to the resource dir
        handle.seek(SeekFrom::Start(rsrc.resource_dir_offset as u64)).await?;

        // this one is a weirdo, right at the beginning of the resource dir
        let _name_list_offset: u16 = deserialize_from(&mut handle).await?;

        // read in the type table
        let type_table: Vec<TypeTableEntry> =
            deserialize_u16_table_from(&mut handle).await?;

        // go and read the name and resource tables for each type
        // let mut name_tables: Vec<Vec<NameTableEntry>> =
        //     Vec::with_capacity(type_table.len());
        let mut resource_tables: Vec<Vec<ResourceTableEntry>> =
            Vec::with_capacity(type_table.len());
        for entry in &type_table {
            handle.seek(SeekFrom::Start(
                rsrc.resource_dir_offset as u64 +
                    entry.resource_table_offset as u64
            )).await?;
            resource_tables
                .push(deserialize_u16_table_from(&mut handle).await?);
            // handle.seek(SeekFrom::Start(
            //     rsrc.resource_dir_offset as u64 +
            //         entry.name_table_offset as u64
            // )).await?;
            // name_tables.push(deserialize_u16_table_from(&mut handle).await?);
        }

        // go to the file table and read it in
        handle.seek(SeekFrom::Start(
            rsrc.resource_dir_offset as u64 +
                rsrc.file_table_offset as u64
        )).await?;
        let file_table: Vec<FileTableEntry> =
            deserialize_u32_table_from(&mut handle).await?;

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
                    anyhow::bail!(MhkError::InvalidFormat(
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

        let pos = handle.seek(SeekFrom::Current(0)).await?;

        Ok(MhkArchive {
            handle: std::rc::Rc::new(std::cell::RefCell::new((pos, handle))),
            files,
            resources,
        })
    }

    pub fn open(
        &self,
        typ: &str,
        i: u16,
    ) -> Result<Narrow<BufReader<R>>>
    {
        self.resources
            .get(typ)
            .and_then(|e| e.get(&i))
            .and_then(|info| self.files.get(info.file_table_index))
            .map(|info| Narrow::new(self.handle.clone(),
                                    info.offset, info.size))
            .ok_or(MhkError::ResourceNotFound(None, typ.to_owned(), i).into())
    }
}
