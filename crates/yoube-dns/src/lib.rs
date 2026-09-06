//! `yoube-dns`: system hosts writer for L1 ad/tracker blocking (spec §7).
//!
//! Leaf crate (depends only on `yoube-error`): [`hosts::DnsBlock`] manages a
//! banner-delimited block inside the OS hosts file, backed up before the
//! first install so `uninstall` restores the original bytes. [`source`]
//! fetches/parses the StevenBlack consolidated list.

pub mod hosts;
pub mod source;

pub use hosts::{BEGIN_MARKER, DnsBlock, END_MARKER, HostsStatus, default_hosts_path};
pub use source::{STEVENBLACK_URL, fetch_hosts, parse_domains};
