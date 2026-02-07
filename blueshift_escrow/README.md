# Task 5. Pinocchio托管

1. 创建项目

```
# create workspace
cargo new blueshift_escrow --lib --edition 2021
cd blueshift_escrow

cargo add pinocchio pinocchio-system pinocchio-token pinocchio-associated-token-account
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