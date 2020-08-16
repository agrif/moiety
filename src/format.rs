use anyhow::Result;

#[async_trait::async_trait(?Send)]
pub trait Format<R, I, D> {
    fn extension(&self, _res: &R) -> Option<&str> { None }
    async fn parse(&self, res: &R, input: &mut I) -> Result<D>;
}

#[async_trait::async_trait(?Send)]
pub trait FormatWrite<Fi, R, I, D>: Format<R, I, D>
where
    Fi: Format<R, I, D>,
{
    async fn convert(&self, fmti: &Fi, res: &R, input: &mut I)
                     -> Result<Vec<u8>>;
}

// wrapper type for generic plain old data records
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Record<T>(pub T);

impl<T> std::ops::Deref for Record<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> std::ops::DerefMut for Record<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

// mixed-format
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MixedFormat<B, R> {
    pub bitmap: B,
    pub record: R,
}

#[async_trait::async_trait(?Send)]
impl<Res, I, B, R> Format<Res, I, crate::Bitmap> for MixedFormat<B, R>
where
    B: Format<Res, I, crate::Bitmap>,
{
    fn extension(&self, res: &Res) -> Option<&str> {
        self.bitmap.extension(res)
    }
    async fn parse(&self, res: &Res, input: &mut I) -> Result<crate::Bitmap> {
        self.bitmap.parse(res, input).await
    }
}

#[async_trait::async_trait(?Send)]
impl<Fi, Res, I, B, R> FormatWrite<Fi, Res, I, crate::Bitmap>
    for MixedFormat<B, R>
where
    Fi: Format<Res, I, crate::Bitmap>,
    B: FormatWrite<Fi, Res, I, crate::Bitmap>,
{
    async fn convert(&self, fmti: &Fi, res: &Res, input: &mut I)
                     -> Result<Vec<u8>>
    {
        self.bitmap.convert(fmti, res, input).await
    }
}

#[async_trait::async_trait(?Send)]
impl<Res, I, B, R, T> Format<Res, I, Record<T>> for MixedFormat<B, R>
where
    R: Format<Res, I, Record<T>>,
    T: 'static,
{
    fn extension(&self, res: &Res) -> Option<&str> {
        self.record.extension(res)
    }
    async fn parse(&self, res: &Res, input: &mut I) -> Result<Record<T>> {
        self.record.parse(res, input).await
    }
}

#[async_trait::async_trait(?Send)]
impl<Fi, Res, I, B, R, T> FormatWrite<Fi, Res, I, Record<T>>
    for MixedFormat<B, R>
where
    Fi: Format<Res, I, Record<T>>,
    R: FormatWrite<Fi, Res, I, Record<T>>,
    T: 'static,
{
    async fn convert(&self, fmti: &Fi, res: &Res, input: &mut I)
                     -> Result<Vec<u8>>
    {
        self.record.convert(fmti, res, input).await
    }
}
