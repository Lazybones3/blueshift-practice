use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer};

declare_id!("22222222222222222222222222222222222222222222");

#[program]
pub mod blueshift_anchor_vault {
    use super::*;

    pub fn deposit(ctx: Context<VaultAction>, amount: u64) -> Result<()> {
        // 1.验证金库为空，以防止重复存款
        require_eq!(ctx.accounts.vault.lamports(), 0 , VaultError::VaultAlreadyExists);
        // 2.确保存款金额超过免租金最低限额
        require_gt!(amount, Rent::get()?.minimum_balance(0), VaultError::InvalidAmount);
        // 3.使用 CPI 调用系统程序，将 lamports 从签名者转移到金库
        transfer(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.signer.to_account_info(),
                    to: ctx.accounts.vault.to_account_info(),
                }
            ),
            amount
        )?;
        Ok(())
    }

    pub fn withdraw(ctx: Context<VaultAction>) -> Result<()> {
        // 1.验证保险库中是否有 lamports（不为空）
        require_neq!(ctx.accounts.vault.lamports(), 0, VaultError::InvalidAmount);
        // 2.使用保险库的 PDA 以其自身名义签署转账
        // Create PDA signer seeds
        let signer_key = ctx.accounts.signer.key();
        let signer_seeds = &[b"vault", signer_key.as_ref(), &[ctx.bumps.vault]];
        // 3.将保险库中的所有 lamports 转回到签署者
        // Transfer all lamports from vault to signer
        transfer(
            CpiContext::new_with_signer(
                ctx.accounts.system_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.vault.to_account_info(),
                    to: ctx.accounts.signer.to_account_info(),
                },
                &[&signer_seeds[..]]
            ),
            ctx.accounts.vault.lamports()
        )?;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct VaultAction<'info> {
    // signer是保险库的所有者，也是创建保险库后唯一可以提取lamports的人
    #[account(mut)]
    pub signer: Signer<'info>,
    // vault是一个由种子派生的PDA，用于为签名者存储lamports
    // seeds 和 bumps 定义了如何从种子派生出有效的 PDA
    #[account(
        mut,
        seeds=[b"vault", signer.key().as_ref()],
        bump,
    )]
    pub vault: SystemAccount<'info>,
    // system_program是系统程序账户，因为我们将使用系统程序的转账指令 CPI
    pub system_program: Program<'info, System>,
}

#[error_code]
pub enum VaultError {
    // VaultAlreadyExists用于判断账户中是否已经有lamports，因为这意味着金库已经存在
    #[msg("Vault already exists")]
    VaultAlreadyExists,
    // InvalidAmount用于检查金额是否大于基本账户最低租金
    #[msg("Invalid amount")]
    InvalidAmount,
}