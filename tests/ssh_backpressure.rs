//! Regression test for the "connected but frozen" hang.
//!
//! Reading and writing used to share one `select!` loop. A write parks until the SSH
//! channel's send window reopens, and russh hands *incoming* data to the channel over a
//! bounded queue with a blocking send — so a parked write stopped us draining, the full
//! queue stopped russh's session loop, and the WINDOW_ADJUST that would have reopened
//! the window could never be processed. Both directions stop while the socket stays up.
//!
//! The remote command here is `yes`: it floods output and never reads stdin, so the
//! incoming queue fills while the outgoing window drains — exactly the pair of
//! conditions the old code deadlocked on.
//!
//! Ignored by default: it needs an sshd on 127.0.0.1:1313 that accepts this user's
//! `~/.ssh/id_ed25519`. Run it with
//!   RUSTFLAGS="-L /tmp/linkstubs" cargo test --test ssh_backpressure -- --ignored --nocapture

use std::io::Write;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use leuwi_panjang::ssh::{spawn, SshProfile};

#[test]
#[ignore = "needs a local sshd on :1313 with key auth"]
fn output_keeps_flowing_while_a_large_write_is_in_flight() {
    let home = std::env::var("HOME").expect("HOME");
    let profile = SshProfile {
        label: "loopback".into(),
        host: "127.0.0.1".into(),
        port: 1313,
        user: std::env::var("USER").unwrap_or_else(|_| "hendri".into()),
        key_path: format!("{home}/.ssh/id_ed25519").into(),
        password: String::new(),
        session: "unused".into(),
        startup: Some("yes".into()),
        known_hosts: std::env::temp_dir().join("leuwi-backpressure-known-hosts"),
        cols: 80,
        rows: 24,
    };

    let seen = Arc::new(AtomicUsize::new(0));
    let counter = seen.clone();
    let handle = spawn(profile, move |bytes| {
        counter.fetch_add(bytes.len(), Ordering::Relaxed);
    });

    // Wait for the flood to start, so we know the session is really up.
    let start = Instant::now();
    while seen.load(Ordering::Relaxed) < 100_000 {
        assert!(
            start.elapsed() < Duration::from_secs(20),
            "no output from the remote after 20s ({} bytes) — is sshd on :1313 reachable?",
            seen.load(Ordering::Relaxed)
        );
        std::thread::sleep(Duration::from_millis(100));
    }

    // Push far more than one channel window into a command that never reads stdin.
    let mut writer = handle.writer();
    let chunk = vec![b'x'; 32 * 1024];
    for _ in 0..256 {
        let _ = writer.write_all(&chunk);
    }
    let _ = writer.flush();

    // The point of the test: output must still be arriving *after* those writes have
    // parked. Before the split this is where everything stopped.
    let before = seen.load(Ordering::Relaxed);
    std::thread::sleep(Duration::from_secs(3));
    let after = seen.load(Ordering::Relaxed);
    handle.close();

    assert!(
        after > before,
        "output stalled with a large write in flight: {before} bytes before, {after} after \
         — the read side is blocked behind the write again"
    );
    println!("ok: {} bytes more arrived while the write was parked", after - before);
}
