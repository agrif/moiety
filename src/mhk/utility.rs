use super::MhkError;

pub async fn deserialize_from<'a, R, T>(reader: &'a R, pos: &'a mut u64) -> Result<T, MhkError> where R: crate::AsyncRead, T: serde::de::DeserializeOwned {
    let size = std::mem::size_of::<T>();
    let mut buf = vec![0u8; size];
    await!(reader.read_exact_at(*pos, &mut buf))?;
    let data = bincode::config().big_endian().deserialize(buf.as_mut())?;
    *pos += size as u64;
    Ok(data)
}

pub async fn deserialize_vec_from<'a, R, T>(reader: &'a R, pos: &'a mut u64, count: usize) -> Result<Vec<T>, MhkError> where R: crate::AsyncRead, T: serde::de::DeserializeOwned {
    let mut config = bincode::config();
    config.big_endian();

    let size = std::mem::size_of::<T>();
    let mut buf = vec![0u8; count * size];
    await!(reader.read_exact_at(*pos, &mut buf))?;
    let mut cursor = std::io::Cursor::new(buf);
    let mut ret = Vec::with_capacity(count);
    for _ in 0..count {
        ret.push(config.deserialize_from(&mut cursor)?);
    }
    *pos += (count * size) as u64;
    Ok(ret)
}

pub async fn deserialize_u16_table_from<'a, R, T>(reader: &'a R, pos: &'a mut u64) -> Result<Vec<T>, MhkError> where R: crate::AsyncRead, T: serde::de::DeserializeOwned {
    await!(deserialize_table_from::<u16, R, T>(reader, pos))
}

pub async fn deserialize_u32_table_from<'a, R, T>(reader: &'a R, pos: &'a mut u64) -> Result<Vec<T>, MhkError> where R: crate::AsyncRead, T: serde::de::DeserializeOwned {
    await!(deserialize_table_from::<u32, R, T>(reader, pos))
}

pub async fn deserialize_table_from<'a, S, R, T>(reader: &'a R, pos: &'a mut u64) -> Result<Vec<T>, MhkError> where S: Into<u64> + serde::de::DeserializeOwned, R: crate::AsyncRead, T: serde::de::DeserializeOwned {
    let count: S = await!(deserialize_from(reader, pos))?;
    await!(deserialize_vec_from(reader, pos, count.into() as usize))
}
