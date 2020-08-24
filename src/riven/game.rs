use super::{
    Card, Command, Event as ScriptEvent, PictureMeta, RivenFormat, Stack as RivenStack, TBmp,
    TCard, TPlst,
};
use crate::{Context, Event, Game, Record, ResourceMap, Resources, Stack};

use anyhow::Result;

pub struct Riven<M> {
    resources: Resources<M>,
    stack: RivenStack,
    current: Option<CardInfo>,
    stackid: u16,
}

#[derive(Debug)]
struct CardInfo {
    id: u16,
    card: Record<Card>,
    plst: Record<Vec<PictureMeta>>,
}

impl<M> Riven<M>
where
    M: ResourceMap<Stack = RivenStack>,
    M::Format: RivenFormat<M::Handle>,
{
    pub async fn new(map: M) -> Result<Self> {
        Ok(Riven {
            resources: Resources::new(map),
            stack: RivenStack::A,
            current: None,
            stackid: 0,
        })
    }

    pub async fn goto(&mut self, ctx: &mut Context, stack: RivenStack, id: u16) -> Result<()> {
        println!("goto {:?} {:?}", stack, id);
        let cardinfo = CardInfo {
            id,
            card: self.resources.open(stack, TCard, id).await?,
            plst: self.resources.open(stack, TPlst, id).await?,
        };

        if let Some(old) = &self.current {
            if let Some(cmds) = old.card.script.get(&ScriptEvent::CloseCard) {
                let cmds = cmds.clone(); // EWWW
                self.script(ctx, &cmds).await?;
            }
        }

        self.stack = stack;
        self.current = Some(cardinfo);

        if let Some(new) = &self.current {
            // EWWWW
            let loads = new.card.script.get(&ScriptEvent::LoadCard).cloned();
            let opens = new.card.script.get(&ScriptEvent::OpenCard).cloned();

            self.activate_plst(ctx, 1).await?;
            if let Some(cmds) = loads {
                self.script(ctx, &cmds).await?;
            }
            self.update_display(ctx).await?;
            if let Some(cmds) = opens {
                self.script(ctx, &cmds).await?;
            }
        }

        Ok(())
    }

    pub async fn script(&mut self, ctx: &mut Context, commands: &[Command]) -> Result<()> {
        for cmd in commands {
            match cmd {
                Command::ActivatePlst { record } => self.activate_plst(ctx, *record).await?,
                Command::Conditional { var, .. } => println!("conditional stub: {:?}", var),
                c => println!("stub: {:?}", c),
            }
        }
        Ok(())
    }

    pub async fn update_display(&mut self, ctx: &mut Context) -> Result<()> {
        println!("update display");
        if let Some(cur) = &self.current {
            if let Some(cmds) = cur.card.script.get(&ScriptEvent::DisplayUpdate) {
                let cmds = cmds.clone(); // EWWW
                self.script(ctx, &cmds).await?;
            }
        }
        ctx.transition().await?;
        Ok(())
    }

    pub async fn activate_plst(&mut self, ctx: &mut Context, id: u16) -> Result<()> {
        println!("activate plst {:?}", id);
        if let Some(current) = &self.current {
            if let Some(plst) = current.plst.get((id - 1) as usize) {
                let bmp = self
                    .resources
                    .open(self.stack, TBmp, plst.bitmap_id)
                    .await?;
                ctx.draw(&bmp, plst.left, plst.top, plst.right, plst.bottom);
            }
        }
        Ok(())
    }
}

#[async_trait::async_trait(?Send)]
impl<M> Game for Riven<M>
where
    M: ResourceMap<Stack = RivenStack>,
    M::Format: RivenFormat<M::Handle>,
{
    fn window_title(&self) -> &str {
        "Riven: The Sequel to Myst"
    }

    fn window_size(&self) -> (u32, u32) {
        (608, 392)
    }

    async fn start(&mut self, ctx: &mut Context) -> Result<()> {
        self.goto(ctx, self.stack, 1).await?;
        Ok(())
    }

    async fn handle_event(&mut self, ctx: &mut Context, ev: Event) -> Result<bool> {
        match ev {
            Event::Exit => Ok(false),
            Event::MouseDown { .. } => {
                if let Some(current) = &self.current {
                    let mut id = current.id + 1;
                    let mut stack = self.stack;
                    loop {
                        match self.goto(ctx, stack, id).await {
                            Ok(()) => return Ok(true),
                            Err(_) => {
                                self.stackid += 1;
                                let stacks = RivenStack::all();
                                if self.stackid as usize >= stacks.len() {
                                    self.stackid = 0;
                                }
                                stack = stacks[self.stackid as usize];
                                id = 1;
                            }
                        }
                    }
                }
                Ok(true)
            }
            _ => Ok(true),
        }
    }
}
