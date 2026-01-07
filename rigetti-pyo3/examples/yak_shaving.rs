use pyo3::{prelude::*, pymodule, types::PyModule, PyResult, Python};

#[cfg(feature = "stubs")]
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};

use rigetti_pyo3::{create_init_submodule, impl_repr};

#[derive(Debug, thiserror::Error)]
#[error("can't shave a yak with clogged clippers!")]
pub struct CloggedClippersError;

#[derive(Clone, Default, Debug)]
#[cfg_attr(feature = "stubs", gen_stub_pyclass)]
#[pyclass(module = "yak_shaving")]
pub struct Yak {
    is_shaved: bool,
}

impl Yak {
    pub fn new() -> Self {
        Self { is_shaved: false }
    }
}

impl_repr!(Yak);

#[cfg_attr(not(feature = "stubs"), optipy::strip_pyo3(only_stubs))]
#[cfg_attr(feature = "stubs", gen_stub_pymethods)]
#[pymethods]
impl Yak {
    #[new]
    #[pyo3(signature = (is_shaved = false))]
    fn __new__(is_shaved: bool) -> Self {
        Self { is_shaved }
    }

    pub fn is_shaved(&self) -> bool {
        self.is_shaved
    }

    pub fn shave_with<'py>(&mut self, tool: PyCuttingTool<'py>) -> PyResult<()> {
        if self.is_shaved() {
            return Ok(());
        }

        match tool {
            PyCuttingTool::Shears(_) => {
                self.is_shaved = true;
            }
            PyCuttingTool::Clippers(mut clippers) => {
                if clippers.is_clogged {
                    return Err(CloggedClippersError.into());
                }
                clippers.is_clogged = true;
                self.is_shaved = true;
            }
        }

        Ok(())
    }
}

#[derive(Clone, Default)]
#[cfg_attr(feature = "stubs", gen_stub_pyclass)]
#[pyclass(module = "yak_shaving")]
pub struct Clippers {
    is_clogged: bool,
}

#[cfg_attr(not(feature = "stubs"), optipy::strip_pyo3(only_stubs))]
#[cfg_attr(feature = "stubs", gen_stub_pymethods)]
#[pymethods]
impl Clippers {
    #[new]
    pub fn new() -> Self {
        Self { is_clogged: false }
    }

    pub fn unclog(&mut self) {
        self.is_clogged = false;
    }

    pub fn is_clogged(&self) -> bool {
        self.is_clogged
    }
}

#[derive(Clone, Default)]
#[cfg_attr(feature = "stubs", gen_stub_pyclass)]
#[pyclass(module = "yak_shaving")]
pub struct Shears;

#[cfg_attr(feature = "stubs", gen_stub_pymethods)]
#[pymethods]
impl Shears {
    #[new]
    pub fn new() -> Self {
        Self
    }
}

#[derive(Clone, FromPyObject)]
pub enum CuttingTool {
    Shears(Shears),
    Clippers(Clippers),
}

pub enum PyCuttingTool<'a> {
    Shears(PyRefMut<'a, Shears>),
    Clippers(PyRefMut<'a, Clippers>),
}

impl<'py> FromPyObject<'_, 'py> for PyCuttingTool<'py> {
    type Error = pyo3::PyErr;

    fn extract(obj: Borrowed<'_, 'py, PyAny>) -> Result<Self, Self::Error> {
        if let Ok(shears) = obj.cast::<Shears>() {
            return Ok(PyCuttingTool::Shears(shears.borrow_mut()));
        }

        let clippers = obj.cast::<Clippers>()?.borrow_mut();
        Ok(PyCuttingTool::Clippers(clippers))
    }
}

#[cfg(feature = "stubs")]
pyo3_stub_gen::impl_stub_type!(PyCuttingTool<'_> = Shears | Clippers);

mod errors {
    use rigetti_pyo3::{create_exception, exception};

    create_exception!(
        yak_shaving,
        Error,
        pyo3::exceptions::PyException,
        "Base exception type for errors raised by this package."
    );

    exception!(
        crate::CloggedClippersError,
        yak_shaving,
        CloggedClippersError,
        Error,
        "Error raised if clippers are clogged when trying to shave a yak."
    );
}

mod tools {
    use super::{errors, Clippers, Shears};
    use rigetti_pyo3::create_init_submodule;

    create_init_submodule! {
        classes: [ Shears, Clippers ],
        errors: [ errors::CloggedClippersError ],
    }
}

create_init_submodule! {
    classes: [ Yak ],
    errors: [ errors::Error ],
    submodules: [ "tools": tools::init_submodule ],
}

#[pymodule]
#[pyo3(name = "yak_shaving")]
fn yak_shaving(m: &Bound<'_, PyModule>) -> PyResult<()> {
    use pyo3::types::PyStringMethods;
    let py = m.py();
    init_submodule(m.name()?.to_str()?, py, m)
}

#[cfg(feature = "stubs")]
pyo3_stub_gen::define_stub_info_gatherer!(stub_info);

fn main() -> PyResult<()> {
    use pyo3::ffi::c_str;

    pyo3::append_to_inittab!(yak_shaving);
    Python::initialize();

    Python::attach(|py| {
        let code = c_str!(
            r#"
import yak_shaving
from yak_shaving import Yak
from yak_shaving.tools import Clippers, Shears, CloggedClippersError

shears = Shears()
clippers = Clippers()

yak1 = Yak()
yak2 = Yak(False)
yak3 = Yak(is_shaved=True)
yak4 = Yak(False)
yak5 = Yak(is_shaved=False)

yak1.shave_with(shears)
yak2.shave_with(shears)

yak3.shave_with(clippers)
yak4.shave_with(clippers)
yak3.shave_with(clippers)

assert clippers.is_clogged()
try:
    yak5.shave_with(clippers)
except CloggedClippersError:
    pass

assert clippers.is_clogged()
try:
    yak5.shave_with(clippers)
except yak_shaving.Error:
    pass

clippers.unclog()
yak5.shave_with(clippers)

print(f"{yak1=} {yak2=} {yak3=} {yak4=} {yak5=}")
"#
        );

        PyModule::from_code(py, code, c_str!("example.py"), c_str!("example"))?;

        Ok(())
    })
}

#[test]
fn test_yak_shaving_example() {
    main().unwrap()
}
