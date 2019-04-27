use super::*;
use crate::{
    display::Display,
    game::{
        Event,
        EventPump,
        Game,
    },
    Format,
    FormatFor,
    ResourceMap,
    Resources,
    ResourcesError,
};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Fail)]
pub enum RivenError<M: failure::Fail, F: failure::Fail, D: failure::Fail> {
    #[fail(display = "{}", _0)]
    ResourceError(#[cause] ResourcesError<M, F>),
    #[fail(display = "{}", _0)]
    DisplayError(#[cause] D),
}

pub struct Riven<M, F> {
    resources: Resources<M, F>,
    needs_draw: bool,
    current: u16,
}

impl<M, F> Riven<M, F> {
    pub fn new(resources: Resources<M, F>) -> Self {
        Riven {
            resources,
            needs_draw: true,
            current: 0,
        }
    }
}

impl<'a, M, F, D> Game<'a, D> for Riven<M, F>
where
    D: Display + 'a,
    M: ResourceMap<Stack = Stack> + 'a,
    F: Format<M::Handle> + FormatFor<M::Handle, Resource<Bitmap>> + 'a,
{
    type Error = RivenError<M::Error, F::Error, D::Error>;

    fn start(
        mut self,
        mut pump: EventPump,
        mut display: D,
    ) -> Fut<'a, Result<(), Self::Error>> {
        fut!({
            loop {
                let event = await!(pump.pump());
                match event {
                    Event::Quit => return Ok(()),

                    Event::MouseDown(_x, _y) => {
                        self.current += 1;
                        self.needs_draw = true;
                    },

                    _ => {},
                }

                if self.needs_draw {
                    let raw_bmp = await!(self.resources.open(
                        Stack::J,
                        Resource::TBMP,
                        self.current
                    ))
                    .map_err(RivenError::ResourceError)?;
                    let bmp = await!(display.transfer(&raw_bmp))
                        .map_err(RivenError::DisplayError)?;
                    display.draw(
                        &bmp,
                        0,
                        0,
                        raw_bmp.width as i32,
                        raw_bmp.height as i32,
                    );
                    display.flip();
                    self.needs_draw = false;
                }
            }
        })
    }
}
