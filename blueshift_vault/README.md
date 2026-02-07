# Task 4. Pinocchio金库

1. 创建项目

```
cargo new blueshift_vault --lib --edition 2021
cd blueshift_vault
# 安装pinocchio
cargo add pinocchio pinocchio-system

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