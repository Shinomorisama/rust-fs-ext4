//! Public full-node fallback on exact independently measured Linux inputs.
use super::*;

#[test]
fn linux_checks_public_full_node_fallback() {
    for bs in [1024, 4096] {
        for journal in [false, true] {
            node_full::profile_named(bs, journal, "link-node-driver-source");
            let base = fs_ext4_test_support::temp_dir()
                .join(format!("link-node-driver-source-{bs}-{journal}"));
            let source = base.with_extension("node-full.img");
            let linux = observe(&base.with_extension("node-grown.img"));
            for capi in [false, true] {
                driver_emission::emit(
                    &source,
                    &linux,
                    capi,
                    bs,
                    journal,
                    driver_emission::Mode::NodeFallback,
                );
            }
        }
    }
}
