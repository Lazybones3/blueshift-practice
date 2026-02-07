# Task 6. Pinocchio AMM

1. 创建项目

```
# create workspace
cargo new blueshift_native_amm --lib --edition 2021
cd blueshift_native_amm

cargo add pinocchio pinocchio-system pinocchio-token pinocchio-associated-token-account
cargo add --git="https://github.com/deanmlittle/constant-product-curve" constant-product-curve

cargo add solana-program-log
```

2. 在 Cargo.toml 中声明 crate 类型，以在 target/deploy 中生成部署工件：

```
[lib]
crate-type = ["lib", "cdylib"]
```

3. 编译项目

```
cargo build-sbf
```