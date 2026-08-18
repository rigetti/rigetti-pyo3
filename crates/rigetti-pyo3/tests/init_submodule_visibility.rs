//! `create_init_submodule!` generates a `pub(crate)` `init_submodule` by default, which cannot be
//! called from another crate. Declaring it `pub` lets a dependent crate register this module's
//! classes into its own module tree — the mechanism behind re-exporting a foreign extension
//! module's types under a local name.

use rigetti_pyo3::pyo3::{Bound, PyResult, Python, types::PyModule};

type InitSubmodule = fn(&str, Python<'_>, &Bound<'_, PyModule>) -> PyResult<()>;

mod default_visibility {
    rigetti_pyo3::create_init_submodule! {}
}

mod public_visibility {
    rigetti_pyo3::create_init_submodule! {
        pub,
    }
}

mod documented_public_visibility {
    rigetti_pyo3::create_init_submodule! {
        pub,
        /// Doc comments still apply after the visibility.
    }
}

/// The generated function must be reachable through the module's public interface.
#[test]
fn public_init_submodule_is_callable_as_a_public_item() {
    let _: InitSubmodule = public_visibility::init_submodule;
    let _: InitSubmodule = documented_public_visibility::init_submodule;

    // The default arm must keep its existing visibility, so this crate can still see it too.
    let _: InitSubmodule = default_visibility::init_submodule;
}
