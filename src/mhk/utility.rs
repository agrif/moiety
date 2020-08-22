use anyhow::Result;
use bincode::Options;
use smol::io::{AsyncRead, AsyncReadExt};

pub async fn deserialize_from<'a, R, T>(
    reader: &'a mut R
) -> Result<T>
where
    R: AsyncRead + Unpin,
    T: serde::de::DeserializeOwned,
{
    let size = std::mem::size_of::<T>();
    let mut buf = vec![0u8; size];
    reader.read_exact(&mut buf).await?;
    let data = bincode::options().with_big_endian().with_fixint_encoding()
        .deserialize(buf.as_mut())?;
    Ok(data)
}

pub async fn deserialize_le_from<'a, R, T>(
    reader: &'a mut R
) -> Result<T>
where
    R: AsyncRead + Unpin,
    T: serde::de::DeserializeOwned,
{
    let size = std::mem::size_of::<T>();
    let mut buf = vec![0u8; size];
    reader.read_exact(&mut buf).await?;
    let data = bincode::options().with_little_endian().with_fixint_encoding()
        .deserialize(buf.as_mut())?;
    Ok(data)
}

pub async fn deserialize_vec_from<'a, R, T>(
    reader: &'a mut R,
    count: usize,
) -> Result<Vec<T>>
where
    R: AsyncRead + Unpin,
    T: serde::de::DeserializeOwned,
{
    let size = std::mem::size_of::<T>();
    let mut buf = vec![0u8; count * size];
    reader.read_exact(&mut buf).await?;
    let mut cursor = std::io::Cursor::new(buf);
    let mut ret = Vec::with_capacity(count);
    for _ in 0..count {
        ret.push(bincode::options().with_big_endian().with_fixint_encoding()
                 .deserialize_from(&mut cursor)?);
    }
    Ok(ret)
}

pub async fn deserialize_u16_table_from<'a, R, T>(
    reader: &'a mut R,
) -> Result<Vec<T>>
where
    R: AsyncRead + Unpin,
    T: serde::de::DeserializeOwned,
{
    deserialize_table_from::<u16, R, T>(reader).await
}

pub async fn deserialize_u32_table_from<'a, R, T>(
    reader: &'a mut R,
) -> Result<Vec<T>>
where
    R: AsyncRead + Unpin,
    T: serde::de::DeserializeOwned,
{
    deserialize_table_from::<u32, R, T>(reader).await
}

pub async fn deserialize_table_from<'a, S, R, T>(
    reader: &'a mut R,
) -> Result<Vec<T>>
where
    S: Into<u64> + serde::de::DeserializeOwned,
    R: AsyncRead + Unpin,
    T: serde::de::DeserializeOwned,
{
    let count: S = deserialize_from(reader).await?;
    deserialize_vec_from(reader, count.into() as usize).await
}
