// Standalone harness for the SSH backend — compiles ONLY src/ssh.rs (no makepad,
// so it links without the desktop GUI's audio/X libs). This is how the SSH →
// tmux path is verified on a headless box:
//   LEUWI_SSH_HOST=localhost LEUWI_SSH_PORT=1313 \
//     cargo run --bin ssh-smoke -- list
//   ... -- attach main
// The real app reaches the same code through `leuwi-panjang ssh-smoke ...`.
#[path = "../ssh.rs"]
mod ssh;

#[cfg(not(target_os = "android"))]
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let sub = args.get(1).map(|s| s.as_str()).unwrap_or("list");
    let session = args.get(2).map(|s| s.as_str()).unwrap_or("main");
    ssh::smoke(sub, session);
}

// The smoke harness is a desktop dev tool; there is no Android entry point.
#[cfg(target_os = "android")]
fn main() {}
