# pwde2e

**简体中文** | [English](#english)

BORUIX 上对 **POSIX 账户查询接口**的端到端验收程序。

它创建一个已知内容的账户表，然后逐项验证查询接口返回的结果是否真实——用户名与用户 ID 能否互相
查到、不存在的账户是否如实返回失败、表被删除后是否会忘记旧数据。

---

## 它验证什么

`getpwnam` / `getpwuid` / `getpwent` 是一组标准的账户查询接口：按名字查用户、按用户 ID 反查
名字、遍历整个表。它们看起来简单，但**容易在"查不到"的时候出错**。

这个程序的核心目的不是确认"能查到存在的用户"——那是显而易见的。它要确认的是**失败路径也是诚实的**：

- 表不存在时，查询必须返回"查不到"，**不能凭空编造出一个账户**
- 查不存在的名字或 ID，必须返回空，并给出正确的错误码
- 表被删除后，之前查过的结果必须被**遗忘**，不能返回陈旧数据

这三条都比"能查到"重要，因为它们决定了程序在数据缺失时的行为是否可信。

## 为什么需要独立进程验收

系统里已经有一个同类的检查命令，但它在 shell 进程内运行。

`pwde2e` 是一个**独立的用户态进程**：它验证的是"任意用户程序经 C 库链接后调用这套接口"这条
路径——链接、符号解析、跨进程调用是否都成立。这是 shell 内检查覆盖不到的。

## 验证项

程序按顺序执行 16 项检查，任何一项失败都立即停止并返回该项编号作为退出码。

| # | 检查 |
| --- | --- |
| 1 | **无表时查询返回空**（不编造账户） |
| 2–3 | 能创建并写入账户表 |
| 4–6 | 按名字查到用户，ID 与名字长度都正确 |
| 7 | 用户的主目录路径正确 |
| 8–9 | 按 ID 反查**能回到同一个用户**（双向一致） |
| 10–11 | 表中缺少组 ID 的条目，按用户 ID 取值 |
| 12–14 | 查不存在的名字/ID 返回空，且错误码为"不存在" |
| 15 | 遍历恰好得到表中全部条目，不多不少 |
| 16 | **表删除后旧数据不再可查**（无陈旧缓存） |

第 1 项和第 16 项是**顺序上的刻意安排**，见下。

## 两处刻意的顺序

**第一项检查必须在写入账户表之前做。** 如果在写表之后才检查"无表时是否返回空"，就无法证明
返回的是空——因为表已经存在了。先确认空状态下的行为，才能证明**没有伪造数据**。

**最后一项检查在删除表之后做。** 它验证的是库不会缓存上一次的结果。如果查询接口把结果留在内存
里，那么即使用户已经被删除，程序仍会认为它存在——这在权限判定上是危险的。

## 退出码

| 退出码 | 含义 |
| --- | --- |
| `0` | 全部通过 |
| `1`–`16` | 首个失败项的编号 |
| 非零 | 失败的**具体位置**可由退出码直接定位，无需翻日志 |

失败时也会打印一行说明，标明是哪一项以及实际观察到的值。

## 清理

程序在结束前会删除它创建的账户表，不留下副作用。中途失败时也会清理。

## 构建

```bash
cargo build --release
```

编译产物部署为 BORUIX 系统中的用户态程序，由自检流程拉起。

## 文件结构

```
pwde2e/
├── Cargo.toml    # 包定义
├── build.rs      # 注入链接脚本
├── linker.ld     # 用户态段布局
└── src/
    └── main.rs   # 16 项检查
```

## 相关项目

- [`libc`](https://github.com/BRX-Boruix/libc) —— 提供 `getpwnam` 等账户查询接口
- [`login`](https://github.com/BRX-Boruix/login) —— 认证时使用账户信息
- [`libsys`](https://github.com/BRX-Boruix/libsys) —— 用户态系统调用封装

## 许可

MIT License，版权归 Yang Borui 所有。详见 [LICENSE](LICENSE)。

---

# English

[简体中文](#pwde2e) | **English**

An end-to-end acceptance program for the **POSIX account lookup interfaces** on BORUIX.

It writes an account table with known contents, then verifies item by item that the lookup interfaces
return the truth — that a name and a user ID resolve to each other, that an absent account fails
honestly, and that a removed table is forgotten rather than cached.

---

## What it verifies

`getpwnam` / `getpwuid` / `getpwent` are the standard account lookup interfaces: look up a user by
name, reverse-look-up a name by user ID, and enumerate the whole table. They look simple, but are
**easy to get wrong precisely when nothing is found**.

The point of this program is not to confirm "an existing user can be found" — that is obvious. It is
to confirm that **the failure paths are honest too**:

- With no table present, a lookup must return "not found" and **must not fabricate an account**
- Looking up a name or ID that does not exist must return nothing, with the correct error code
- After the table is deleted, previously looked-up results must be **forgotten**, not returned stale

All three matter more than "lookup succeeds", because they determine whether the program's behaviour
when data is missing can be trusted.

## Why it needs a separate process

The system already has a check of this kind, but it runs inside the shell process.

`pwde2e` is a **separate user-space process**: it verifies the path "an arbitrary user program
reaches these interfaces through the C library" — that linking, symbol resolution, and the call all
hold. That is what an in-shell check cannot cover.

## The checks

The program performs 16 checks in order, stopping at the first failure and returning its number as the
exit code.

| # | Check |
| --- | --- |
| 1 | **With no table, lookup returns nothing** (no fabricated account) |
| 2–3 | The account table can be created and written |
| 4–6 | A user is found by name, with correct ID and name length |
| 7 | The user's home directory path is correct |
| 8–9 | Reverse lookup by ID **returns the same user** (bidirectional consistency) |
| 10–11 | An entry lacking a group ID falls back to the user ID |
| 12–14 | Unknown name/ID returns nothing, with a "not found" error code |
| 15 | Enumeration yields exactly the table's entries, no more and no fewer |
| 16 | **After the table is deleted, old data no longer resolves** (no stale cache) |

Checks 1 and 16 are **deliberately ordered**, see below.

## Two deliberate orderings

**The first check must run before the table is written.** Were the "no table returns nothing" check
done after writing, its result could not prove anything — the table would already exist. Confirming
the empty state first is what proves **no data is fabricated**.

**The last check runs after the table is deleted.** It verifies that the library does not cache the
previous result. If a lookup kept its result in memory, the program would still believe a deleted
user exists — dangerous where permissions are being decided.

## Exit codes

| Exit code | Meaning |
| --- | --- |
| `0` | Everything passed |
| `1`–`16` | The number of the first failing check |
| | A non-zero exit pinpoints the **exact failure** without reading logs |

On failure it also prints a line stating which check failed and what value was actually observed.

## Cleanup

The program deletes the account table it created before finishing, leaving no side effects. It also
cleans up on an early failure.

## Building

```bash
cargo build --release
```

The artifact is deployed as a user-space program in a BORUIX system and started by the self-test
flow.

## Layout

```
pwde2e/
├── Cargo.toml    # package definition
├── build.rs      # injects the linker script
├── linker.ld     # user-space section layout
└── src/
    └── main.rs   # the 16 checks
```

## Related projects

- [`libc`](https://github.com/BRX-Boruix/libc) — provides `getpwnam` and the other account lookup interfaces
- [`login`](https://github.com/BRX-Boruix/login) — uses account information during authentication
- [`libsys`](https://github.com/BRX-Boruix/libsys) — the user-space syscall wrapper

## License

MIT License, copyright Yang Borui. See [LICENSE](LICENSE).
