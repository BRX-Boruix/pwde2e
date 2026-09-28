# pwde2e

BORUIX 的**端到端验收程序**，验证 POSIX 账户查询接口在真实用户态进程内可用。

[English](README.en.md)

## 测什么

程序以**独立进程**运行，经标准 C 接口调用账户查询函数，共 16 项检查：

| 检查 | 内容 |
| --- | --- |
| 无账户表时查询 | 返回空指针，**不伪造账户** |
| `getpwnam` | 按名查 `alice`，返回真实 UID / GID / 名长 / 家目录 |
| `getpwuid` | 按 UID 反查 `bob`，双向一致 |
| UID 回退 | `carol` 未声明 GID 时，GID 回退为其 UID |
| 未命中 | 查不存在的名字 / UID 返回空指针，且 `errno` 为 `ENOENT` |
| `getpwent` | 遍历得到恰好 3 条记录 |
| 缓存陈旧 | 表被移除后，`alice` 不应仍能查到 |

账户数据来自 `/config/users.json`，测试夹具为 3 个账户：`alice`(1000:1000)、`bob`(1001:1001)、
`carol`(1002)。

## 为什么需要独立程序

另有在 `shell` 进程内完成的账户接口检查。本程序是**独立进程**，验证的是**任意用户程序经 C 库链接
调用这些函数**这条路径——包含真实的进程启动、链接与接口穿越。

## 用法

由系统自检的快速组以真实 `exec` 路径拉起，部署位置为 `/programs/pwde2e.elf`。

## 退出码

| 退出码 | 含义 |
| --- | --- |
| `0` | 全部 16 项通过 |
| `1`–`16` | 首个失败项编号 |

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
    └── main.rs   # 16 项验收检查
```

## 相关项目

- [`libc`](https://github.com/BRX-Boruix/libc) —— 提供账户查询函数的 C 接口实现
- [`selftest`](https://github.com/BRX-Boruix/selftest) —— 拉起本程序并判定其退出码
- [`userd`](https://github.com/BRX-Boruix/userd) —— 账户守护进程，维护家目录与身份投影

## 许可

MIT License，版权归 Yang Borui 所有。详见 [LICENSE](LICENSE)。
