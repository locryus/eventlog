#![allow(dead_code)]
#![allow(renamed_and_removed_lints)]
#![cfg_attr(feature = "cargo-clippy", allow(unreadable_literal))]

// build.rs generates a rust snippet with constants from res/eventmsgs.h into
// res/eventmsgs.rs.
include!("../res/eventmsgs.rs");
