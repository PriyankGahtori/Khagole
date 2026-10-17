//! The full de440s kernel for unit tests (`scripts/fetch-kernels.sh`). Tests that need it
//! are skipped when it is missing, unless `ASTROCELESTE_REQUIRE_KERNEL` is set (as in CI),
//! which makes a missing kernel a failure instead of a silent pass.

use std::path::Path;

use crate::ephemeris::{Kernel, KernelSet, Spk};

pub(crate) fn full_kernel() -> Option<KernelSet> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../kernels/de440s.bsp");
    if !path.exists() {
        assert!(
            std::env::var_os("ASTROCELESTE_REQUIRE_KERNEL").is_none(),
            "{} not found and ASTROCELESTE_REQUIRE_KERNEL is set",
            path.display()
        );
        eprintln!(
            "skipping: {} not found (scripts/fetch-kernels.sh)",
            path.display()
        );
        return None;
    }
    let mut set = KernelSet::new();
    set.push(Kernel::new("de440s.bsp", Spk::open(path).unwrap()).unwrap());
    Some(set)
}
