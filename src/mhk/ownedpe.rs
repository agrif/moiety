use anyhow::Result;
use pelite::pe32::Pe;
use pelite::resources::Name;
use smol::io::Cursor;

pub struct OwnedPe32 {
    _data: Vec<u8>,
    pe_unsafe: pelite::pe32::PeFile<'static>,
}

impl OwnedPe32 {
    pub fn new(data: Vec<u8>) -> Result<Self> {
        let buf: &'static [u8] = unsafe {
            // hella, HELLA unsafe
            // only works because data is a Vec, and so this points
            // to memory that does not move, even if data itself moves
            // we just have to be sure to never mutate data
            // and only provide safe access to pe
            // (see also: owning_ref crate, which barely does not work here)
            std::mem::transmute(data.as_slice())
        };
        Ok(OwnedPe32 {
            pe_unsafe: pelite::pe32::PeFile::from_bytes(buf)?,
            _data: data,
        })
    }

    pub fn pe<'a>(&'a self) -> &'a pelite::pe32::PeFile<'a> {
        &self.pe_unsafe
    }

    pub fn get(&self, typ: &str) -> Option<Vec<u16>> {
        if typ != "tCUR" {
            return None;
        }

        let rsrc = self.pe().resources().ok()?;

        let mut ret = Vec::with_capacity(10);
        for cur in rsrc.group_cursors() {
            let cur = cur.ok()?;
            if let Name::Id(id) = cur.0 {
                ret.push(id as u16);
            }
        }
        Some(ret)
    }

    pub fn open(&self, typ: &str, id: u16) -> Result<Cursor<Vec<u8>>> {
        if typ != "tCUR" {
            anyhow::bail!("bad resource type for exe");
        }

        let rsrc = self.pe().resources()?;

        for cur in rsrc.group_cursors() {
            let cur = cur?;
            if let Name::Id(testid) = cur.0 {
                if id as u32 == testid {
                    let groupid = cur.1.entries()[0].nId;
                    let dataref = cur.1.image(groupid)?;
                    let mut data = Vec::with_capacity(dataref.len());
                    data.extend_from_slice(dataref);
                    return Ok(Cursor::new(data));
                }
            }
        }
        anyhow::bail!("could not find cursor {:?}", id);
    }
}
