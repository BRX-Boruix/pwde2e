# pwde2e

BORUIX 的验收程序：在真实系统上验证按名与按 id 查询用户账户的完整链路。

[English](README.en.md)

## 测什么

标准 C 库提供按名与按 id 查询用户的接口。本程序作为独立进程，验证从账户表到查询结果的整条
链路：

- 账户表不存在时，查询返回「没有这个用户」，不伪造账户
- 写入一张含三个账户的表后，按名查询返回正确的 uid、gid、名字与家目录
- 按 id 查询同样成立，且与按名查询互为逆操作
- 家目录与账户表中登记的一致

全部通过时逐项打印 `OK` 并以 0 退出。

## 退出码

- `0`——全部通过
- `1` 到 `16`——失败项编号，编号对应输出中 `FAIL(n)` 的位置

## 构建

```bash
cargo build --release
```

## 文件结构

```
pwde2e/
├── Cargo.toml    # 包定义
├── build.rs      # 注入链接脚本
├── linker.ld     # 用户态段布局
└── src/
    └── main.rs   # 账户表写入与查询判定
```

## 相关项目

- [`libc`](https://github.com/BRX-Boruix/libc) —— 被验收的查询接口
- [`userd`](https://github.com/BRX-Boruix/userd) —— 账户守护进程
- [`selftest`](https://github.com/BRX-Boruix/selftest) —— 以独立进程方式拉起本程序

## 许可

MIT License，版权归 Yang Borui 所有。详见 [LICENSE](LICENSE)。
