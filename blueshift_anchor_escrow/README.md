# Task 3. Anchor托管

1. 新建项目

```
anchor init blueshift_anchor_escrow
cd blueshift_anchor_escrow
cargo add anchor-lang --features init-if-needed
cargo add anchor-spl
```

2. 打开programs/blueshift_anchor_escrow/Cargo.toml，在idl-build中添加anchor-spl/idl-build：

```
[features]
idl-build = ["anchor-lang/idl-build", "anchor-spl/idl-build"]
```

3. 项目文件夹结构如下：

```
src
├── instructions
│       ├── make.rs
│       ├── mod.rs
│       ├── refund.rs
│       └── take.rs
├── errors.rs
├── lib.rs
└── state.rs
```

4. 构建项目

```
anchor build
```

#### 问题解决

1. anchor build报错如下：

```
error: failed to download `constant_time_eq v0.4.2`

Caused by:
  unable to get packages from source

Caused by:
  failed to parse manifest at `/root/.cargo/registry/src/index.crates.io-6f17d22bba15001f/constant_time_eq-0.4.2/Cargo.toml`

Caused by:
  feature `edition2024` is required

  The package requires the Cargo feature called `edition2024`, but that feature is not stabilized in this version of Cargo (1.84.0 (12fe57a9d 2025-04-07)).
  Consider trying a more recent nightly release.
  See https://doc.rust-lang.org/nightly/cargo/reference/unstable.html#edition-2024 for more information about the status of this feature.
```

解决方法：

```
# 降级blake3以避免引⼊constant_time_eq 0.4.2 (需要 Rust 2024 edition)
cargo update -p blake3 --precise 1.5.5
```