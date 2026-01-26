use anchor_lang::prelude::*;

// 我们添加了#[derive(InitSpace)]宏，这样就不需要手动计算这个结构的租金。
#[derive(InitSpace)]
#[account(discriminator = 1)]
pub struct Escrow {
    // seed：在种子派生过程中使用的随机数，因此一个创建者可以使用相同的代币对打开多个托管账户；存储在链上，以便我们始终可以重新派生 PDA。
    pub seed: u64,
    // maker：创建托管账户的钱包；需要用于退款和接收付款。
    pub maker: Pubkey,
    // mint_a 和 mint_b：交换中“给出”和“获取”两侧的 SPL 铸币地址。
    pub mint_a: Pubkey,
    pub mint_b: Pubkey,
    // receive：创建者希望获得的代币 B 的数量。
    pub receive: u64,
    // bump：缓存的 bump 字节；动态派生它会消耗计算资源，因此我们将其保存一次。
    pub bump: u8,
}