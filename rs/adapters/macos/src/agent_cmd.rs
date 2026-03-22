//! `clx agent` subcommand — runs the agent within the clx process,
//! inheriting its Accessibility permission.
//!
//! Usage: clx agent --tree
//!        clx agent --exec
//!        clx agent --prompt "click the Issues tab"
//!        clx agent "click the Issues tab"

// Pull in all agent code from the standalone binary source.
// The `main()` at the end is dead code here (shadowed by our pub fn main).
#[allow(dead_code)]
mod inner {
    include!("bin/clx-agent.rs");

    pub fn run(args: &[String]) {
        agent_main(args);
    }
}

pub fn main(args: &[String]) {
    let mut full_args = vec!["clx agent".to_string()];
    full_args.extend_from_slice(args);
    inner::run(&full_args);
}
