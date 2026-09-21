//! BORUIX pwde2e：POSIX 账户查询（A2-5 / ADR-040 3.5 G5）的**真实用户态**验收。
//!
//! 经 selftest 的 quick 组以 /programs/pwde2e.elf 拉起（真实 exec 路径）。
//! 与 libccheck 互补：libccheck 在 shell 进程内；本程序是**独立进程**，验证
//! 任意用户程序经 libc 链接调用 getpwnam/getpwuid 这条路径（3.5.4）。
//! 退出码：0 = 全部通过；非零 = 首个失败项编号。

#![no_std]
#![no_main]
extern crate alloc;
use libsys::{close, open, unlink, write, OpenFlags, Permissions, STDOUT};

fn say(parts: &[&[u8]]) {
    for p in parts { let _ = write(STDOUT, p); }
    let _ = write(STDOUT, b"\n");
}

fn cleanup() { let _ = unlink("/config/users.json"); }

#[unsafe(no_mangle)]
pub extern "C" fn user_main(_argc: isize, _argv: *const *const u8) -> i32 {
    say(&[b"[pwde2e] === A2-5 user-space E2E: POSIX account lookup ==="]);
    const TABLE: &str = "/config/users.json";

    // 0. **先**验证"表缺失时如实报错"（写入真表之前，才能证明无伪数据）
    let _ = unlink(TABLE);
    unsafe { libc::pwd::endpwent() };
    let missing = unsafe { libc::pwd::getpwnam(b"alice\0".as_ptr() as *const i8) };
    if !missing.is_null() {
        say(&[b"[pwde2e] FAIL(1): getpwnam returned an account with NO table on disk"]);
        return 1;
    }
    say(&[b"[pwde2e] no table -> getpwnam NULL (no fabricated account) OK"]);

    // 1. 写入已知内容的账户表
    let fd = match unsafe { open(TABLE, OpenFlags::CREATE_OR_TRUNCATE, Permissions::read_write()) } {
        Ok(f) => f,
        Err(_) => { say(&[b"[pwde2e] FAIL(2): cannot create table"]); return 2; }
    };
    let body: &[u8] = br#"{"users":[{"name":"alice","uid":1000,"gid":1000},{"name":"bob","uid":1001,"gid":1001},{"name":"carol","uid":1002}]}"#;
    if write(fd, body).is_err() {
        say(&[b"[pwde2e] FAIL(3): cannot write table"]);
        let _ = close(fd);
        return 3;
    }
    let _ = close(fd);
    say(&[b"[pwde2e] wrote table: alice(1000:1000) bob(1001:1001) carol(1002)"]);
    unsafe { libc::pwd::endpwent() };

    // 2. getpwnam(alice) 必须返回真 uid/gid
    let pa = unsafe { libc::pwd::getpwnam(b"alice\0".as_ptr() as *const i8) };
    if pa.is_null() {
        say(&[b"[pwde2e] FAIL(4): getpwnam(alice) NULL although table lists alice"]);
        cleanup(); return 4;
    }
    let (uid, gid, namelen) = unsafe {
        let a = &*pa;
        let mut n = 0usize;
        while *(a.pw_name as *const u8).add(n) != 0 { n += 1; }
        (a.pw_uid, a.pw_gid, n)
    };
    if uid != 1000 || gid != 1000 {
        say(&[b"[pwde2e] FAIL(5): alice uid/gid != 1000/1000"]);
        cleanup(); return 5;
    }
    if namelen != 5 {
        say(&[b"[pwde2e] FAIL(6): alice pw_name length != 5"]);
        cleanup(); return 6;
    }
    say(&[b"[pwde2e] getpwnam(alice) -> uid=1000 gid=1000 name=alice (real) OK"]);

    let dir_ok = unsafe {
        let d = (*pa).pw_dir as *const u8;
        if d.is_null() { false } else {
            let want = b"/users/alice";
            let mut ok = true;
            for (k, w) in want.iter().enumerate() {
                if *d.add(k) != *w { ok = false; break; }
            }
            ok && *d.add(want.len()) == 0
        }
    };
    if !dir_ok {
        say(&[b"[pwde2e] FAIL(7): pw_dir is not /users/alice"]);
        cleanup(); return 7;
    }
    say(&[b"[pwde2e] pw_dir == /users/alice OK"]);

    // 3. getpwuid(1001) 反查到 bob
    let pb = unsafe { libc::pwd::getpwuid(1001) };
    if pb.is_null() {
        say(&[b"[pwde2e] FAIL(8): getpwuid(1001) NULL although table lists bob"]);
        cleanup(); return 8;
    }
    let bob_ok = unsafe {
        let n = (*pb).pw_name as *const u8;
        !n.is_null() && *n == b'b' && *n.add(1) == b'o' && *n.add(2) == b'b' && *n.add(3) == 0
            && (*pb).pw_uid == 1001 && (*pb).pw_gid == 1001
    };
    if !bob_ok {
        say(&[b"[pwde2e] FAIL(9): getpwuid(1001) did not round-trip to bob(1001:1001)"]);
        cleanup(); return 9;
    }
    say(&[b"[pwde2e] getpwuid(1001) -> name=bob uid=1001 gid=1001 (bidirectional) OK"]);

    // 4. 缺 gid 的条目按 uid 取值
    let pc = unsafe { libc::pwd::getpwuid(1002) };
    if pc.is_null() {
        say(&[b"[pwde2e] FAIL(10): getpwuid(1002) NULL"]);
        cleanup(); return 10;
    }
    let carol_gid = unsafe { (*pc).pw_gid };
    if carol_gid != 1002 {
        say(&[b"[pwde2e] FAIL(11): carol gid != 1002 (uid fallback)"]);
        cleanup(); return 11;
    }
    say(&[b"[pwde2e] entry without gid takes uid (1002) OK"]);

    // 5. 查不存在的名字/uid -> NULL + ENOENT
    unsafe { libc::errno::set_errno(0); }
    let pg = unsafe { libc::pwd::getpwnam(b"nosuchuser\0".as_ptr() as *const i8) };
    let e1 = libc::errno::errno();
    if !pg.is_null() {
        say(&[b"[pwde2e] FAIL(12): unknown name returned non-NULL"]);
        cleanup(); return 12;
    }
    if e1 != libc::errno::ENOENT {
        say(&[b"[pwde2e] FAIL(13): unknown name errno != ENOENT(2)"]);
        cleanup(); return 13;
    }
    let pu = unsafe { libc::pwd::getpwuid(999999) };
    if !pu.is_null() {
        say(&[b"[pwde2e] FAIL(14): unknown uid returned non-NULL"]);
        cleanup(); return 14;
    }
    say(&[b"[pwde2e] unknown name/uid -> NULL + ENOENT OK"]);

    // 6. getpwent 遍历恰好 3 条
    unsafe { libc::pwd::setpwent() };
    let mut count = 0u64;
    loop {
        let e = unsafe { libc::pwd::getpwent() };
        if e.is_null() { break; }
        count += 1;
        if count > 64 { break; }
    }
    if count != 3 {
        say(&[b"[pwde2e] FAIL(15): getpwent yielded != 3 entries"]);
        cleanup(); return 15;
    }
    say(&[b"[pwde2e] getpwent enumerates exactly 3 entries OK"]);

    // 7. 表移除后必须忘记（不返回陈旧数据）
    unsafe { libc::pwd::endpwent() };
    let _ = unlink(TABLE);
    unsafe { libc::pwd::endpwent() };
    let stale = unsafe { libc::pwd::getpwnam(b"alice\0".as_ptr() as *const i8) };
    if !stale.is_null() {
        say(&[b"[pwde2e] FAIL(16): after removing table alice still resolves (stale cache)"]);
        return 16;
    }
    say(&[b"[pwde2e] after table removal -> alice no longer resolves OK"]);

    say(&[b"[pwde2e] === A2-5 PASS ==="]);
    0
}
