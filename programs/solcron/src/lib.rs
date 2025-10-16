use anchor_lang::prelude::*;

declare_id!("5CENHQo5xhRnAXYjFBZ5D9sjM28w7fNm5iyGptbHzrjo");

#[program]
pub mod solcron {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
