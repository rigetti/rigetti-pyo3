use pyo3::{self, pymodule, types::PyModule, PyResult, Python};
pub mod rust {
    pub enum TestEnum {
        One,
        Two,
    }
    #[automatically_derived]
    impl ::core::clone::Clone for TestEnum {
        #[inline]
        fn clone(&self) -> TestEnum {
            *self
        }
    }
    #[automatically_derived]
    impl ::core::marker::Copy for TestEnum {}
    pub enum TestUnionEnum {
        Unit,
        String(String),
    }
    #[automatically_derived]
    impl ::core::clone::Clone for TestUnionEnum {
        #[inline]
        fn clone(&self) -> TestUnionEnum {
            match self {
                TestUnionEnum::Unit => TestUnionEnum::Unit,
                TestUnionEnum::String(__self_0) => {
                    TestUnionEnum::String(::core::clone::Clone::clone(__self_0))
                }
            }
        }
    }
    pub struct TestStruct {
        pub test_enum_unaliased: TestEnum,
        pub test_enum_aliased: TestEnum,
    }
    #[automatically_derived]
    impl ::core::clone::Clone for TestStruct {
        #[inline]
        fn clone(&self) -> TestStruct {
            let _: ::core::clone::AssertParamIsClone<TestEnum>;
            *self
        }
    }
    #[automatically_derived]
    impl ::core::marker::Copy for TestStruct {}
}
pub mod python {
    use super::rust::*;
    use pyo3::pymethods;
    use rigetti_pyo3::{
        create_init_submodule, py_wrap_data_struct, py_wrap_simple_enum,
        py_wrap_union_enum,
    };
    pub(crate) fn init_submodule(
        _name: &str,
        _py: ::rigetti_pyo3::pyo3::Python,
        m: &::rigetti_pyo3::pyo3::types::PyModule,
    ) -> ::rigetti_pyo3::pyo3::PyResult<()> {
        m.add_class::<PyTestEnumUnaliased>()?;
        m.add_class::<PyTestEnumAliased>()?;
        m.add_class::<PyTestStruct>()?;
        m.add_class::<PyTestUnionEnum>()?;
        Ok(())
    }
    #[repr(transparent)]
    #[allow(clippy::use_self)]
    pub struct PyTestUnionEnum(TestUnionEnum);
    #[automatically_derived]
    #[allow(clippy::use_self)]
    impl ::core::clone::Clone for PyTestUnionEnum {
        #[inline]
        fn clone(&self) -> PyTestUnionEnum {
            PyTestUnionEnum(::core::clone::Clone::clone(&self.0))
        }
    }
    const _: () = {
        use ::pyo3 as _pyo3;
        unsafe impl _pyo3::type_object::PyTypeInfo for PyTestUnionEnum {
            type AsRefTarget = _pyo3::PyCell<Self>;
            const NAME: &'static str = "TestUnionEnum";
            const MODULE: ::std::option::Option<&'static str> = ::core::option::Option::None;
            #[inline]
            fn type_object_raw(py: _pyo3::Python<'_>) -> *mut _pyo3::ffi::PyTypeObject {
                <PyTestUnionEnum as _pyo3::impl_::pyclass::PyClassImpl>::lazy_type_object()
                    .get_or_init(py)
                    .as_type_ptr()
            }
        }
        impl _pyo3::PyClass for PyTestUnionEnum {
            type Frozen = _pyo3::pyclass::boolean_struct::False;
        }
        impl<'a, 'py> _pyo3::impl_::extract_argument::PyFunctionArgument<'a, 'py>
        for &'a PyTestUnionEnum {
            type Holder = ::std::option::Option<_pyo3::PyRef<'py, PyTestUnionEnum>>;
            #[inline]
            fn extract(
                obj: &'py _pyo3::PyAny,
                holder: &'a mut Self::Holder,
            ) -> _pyo3::PyResult<Self> {
                _pyo3::impl_::extract_argument::extract_pyclass_ref(obj, holder)
            }
        }
        impl<'a, 'py> _pyo3::impl_::extract_argument::PyFunctionArgument<'a, 'py>
        for &'a mut PyTestUnionEnum {
            type Holder = ::std::option::Option<_pyo3::PyRefMut<'py, PyTestUnionEnum>>;
            #[inline]
            fn extract(
                obj: &'py _pyo3::PyAny,
                holder: &'a mut Self::Holder,
            ) -> _pyo3::PyResult<Self> {
                _pyo3::impl_::extract_argument::extract_pyclass_ref_mut(obj, holder)
            }
        }
        impl _pyo3::IntoPy<_pyo3::PyObject> for PyTestUnionEnum {
            fn into_py(self, py: _pyo3::Python) -> _pyo3::PyObject {
                _pyo3::IntoPy::into_py(_pyo3::Py::new(py, self).unwrap(), py)
            }
        }
        impl _pyo3::impl_::pyclass::PyClassImpl for PyTestUnionEnum {
            const IS_BASETYPE: bool = false;
            const IS_SUBCLASS: bool = false;
            const IS_MAPPING: bool = false;
            const IS_SEQUENCE: bool = false;
            type BaseType = _pyo3::PyAny;
            type ThreadChecker = _pyo3::impl_::pyclass::SendablePyClass<PyTestUnionEnum>;
            type Inventory = Pyo3MethodsInventoryForPyTestUnionEnum;
            type PyClassMutability = <<_pyo3::PyAny as _pyo3::impl_::pyclass::PyClassBaseType>::PyClassMutability as _pyo3::impl_::pycell::PyClassMutability>::MutableChild;
            type Dict = _pyo3::impl_::pyclass::PyClassDummySlot;
            type WeakRef = _pyo3::impl_::pyclass::PyClassDummySlot;
            type BaseNativeType = _pyo3::PyAny;
            fn items_iter() -> _pyo3::impl_::pyclass::PyClassItemsIter {
                use _pyo3::impl_::pyclass::*;
                let collector = PyClassImplCollector::<Self>::new();
                static INTRINSIC_ITEMS: PyClassItems = PyClassItems {
                    methods: &[],
                    slots: &[],
                };
                PyClassItemsIter::new(
                    &INTRINSIC_ITEMS,
                    ::std::boxed::Box::new(
                        ::std::iter::Iterator::map(
                            _pyo3::inventory::iter::<
                                <Self as _pyo3::impl_::pyclass::PyClassImpl>::Inventory,
                            >(),
                            _pyo3::impl_::pyclass::PyClassInventory::items,
                        ),
                    ),
                )
            }
            fn doc(py: _pyo3::Python<'_>) -> _pyo3::PyResult<&'static ::std::ffi::CStr> {
                use _pyo3::impl_::pyclass::*;
                static DOC: _pyo3::once_cell::GILOnceCell<
                    ::std::borrow::Cow<'static, ::std::ffi::CStr>,
                > = _pyo3::once_cell::GILOnceCell::new();
                DOC.get_or_try_init(
                        py,
                        || {
                            let collector = PyClassImplCollector::<Self>::new();
                            build_pyclass_doc(
                                <PyTestUnionEnum as _pyo3::PyTypeInfo>::NAME,
                                "\0",
                                ::std::option::Option::None
                                    .or_else(|| collector.new_text_signature()),
                            )
                        },
                    )
                    .map(::std::ops::Deref::deref)
            }
            fn lazy_type_object() -> &'static _pyo3::impl_::pyclass::LazyTypeObject<
                Self,
            > {
                use _pyo3::impl_::pyclass::LazyTypeObject;
                static TYPE_OBJECT: LazyTypeObject<PyTestUnionEnum> = LazyTypeObject::new();
                &TYPE_OBJECT
            }
        }
        #[doc(hidden)]
        #[allow(non_snake_case)]
        impl PyTestUnionEnum {}
        #[doc(hidden)]
        pub struct Pyo3MethodsInventoryForPyTestUnionEnum {
            items: _pyo3::impl_::pyclass::PyClassItems,
        }
        impl Pyo3MethodsInventoryForPyTestUnionEnum {
            pub const fn new(items: _pyo3::impl_::pyclass::PyClassItems) -> Self {
                Self { items }
            }
        }
        impl _pyo3::impl_::pyclass::PyClassInventory
        for Pyo3MethodsInventoryForPyTestUnionEnum {
            fn items(&self) -> &_pyo3::impl_::pyclass::PyClassItems {
                &self.items
            }
        }
        impl ::inventory::Collect for Pyo3MethodsInventoryForPyTestUnionEnum {
            #[inline]
            fn registry() -> &'static ::inventory::Registry {
                static REGISTRY: ::inventory::Registry = ::inventory::Registry::new();
                &REGISTRY
            }
        }
    };
    impl ::rigetti_pyo3::PyTryFrom<PyTestUnionEnum> for TestUnionEnum {
        fn py_try_from(
            py: ::rigetti_pyo3::pyo3::Python,
            item: &PyTestUnionEnum,
        ) -> ::rigetti_pyo3::pyo3::PyResult<Self> {
            Ok(item.0.clone())
        }
    }
    impl ::rigetti_pyo3::PyTryFrom<::rigetti_pyo3::pyo3::PyAny> for PyTestUnionEnum {
        fn py_try_from(
            py: ::rigetti_pyo3::pyo3::Python,
            item: &::rigetti_pyo3::pyo3::PyAny,
        ) -> ::rigetti_pyo3::pyo3::PyResult<Self> {
            item.extract()
        }
    }
    impl ::rigetti_pyo3::PyTryFrom<PyTestUnionEnum> for PyTestUnionEnum {
        fn py_try_from(
            py: ::rigetti_pyo3::pyo3::Python,
            item: &PyTestUnionEnum,
        ) -> ::rigetti_pyo3::pyo3::PyResult<Self> {
            Ok(item.clone())
        }
    }
    #[allow(clippy::use_self)]
    impl ::rigetti_pyo3::ToPython<PyTestUnionEnum> for TestUnionEnum {
        fn to_python(
            &self,
            py: ::rigetti_pyo3::pyo3::Python<'_>,
        ) -> ::rigetti_pyo3::pyo3::PyResult<PyTestUnionEnum> {
            { Ok(PyTestUnionEnum::from(self.clone())) }
        }
    }
    #[allow(clippy::use_self)]
    impl<'a> ::rigetti_pyo3::ToPython<PyTestUnionEnum> for &'a TestUnionEnum {
        fn to_python(
            &self,
            py: ::rigetti_pyo3::pyo3::Python<'_>,
        ) -> ::rigetti_pyo3::pyo3::PyResult<PyTestUnionEnum> {
            {
                <TestUnionEnum as ::rigetti_pyo3::ToPython<
                    PyTestUnionEnum,
                >>::to_python(*self, py)
            }
        }
    }
    #[allow(clippy::use_self)]
    impl ::rigetti_pyo3::ToPython<::rigetti_pyo3::pyo3::Py<::rigetti_pyo3::pyo3::PyAny>>
    for PyTestUnionEnum {
        fn to_python(
            &self,
            py: ::rigetti_pyo3::pyo3::Python<'_>,
        ) -> ::rigetti_pyo3::pyo3::PyResult<
            ::rigetti_pyo3::pyo3::Py<::rigetti_pyo3::pyo3::PyAny>,
        > {
            { Ok(<Self as ::rigetti_pyo3::pyo3::ToPyObject>::to_object(self, py)) }
        }
    }
    #[allow(clippy::use_self)]
    impl<
        'a,
    > ::rigetti_pyo3::ToPython<::rigetti_pyo3::pyo3::Py<::rigetti_pyo3::pyo3::PyAny>>
    for &'a PyTestUnionEnum {
        fn to_python(
            &self,
            py: ::rigetti_pyo3::pyo3::Python<'_>,
        ) -> ::rigetti_pyo3::pyo3::PyResult<
            ::rigetti_pyo3::pyo3::Py<::rigetti_pyo3::pyo3::PyAny>,
        > {
            {
                <PyTestUnionEnum as ::rigetti_pyo3::ToPython<
                    ::rigetti_pyo3::pyo3::Py<::rigetti_pyo3::pyo3::PyAny>,
                >>::to_python(*self, py)
            }
        }
    }
    impl From<PyTestUnionEnum> for TestUnionEnum {
        fn from(wrapper: PyTestUnionEnum) -> Self {
            wrapper.0
        }
    }
    impl From<TestUnionEnum> for PyTestUnionEnum {
        fn from(inner: TestUnionEnum) -> Self {
            Self(inner)
        }
    }
    impl From<&TestUnionEnum> for PyTestUnionEnum {
        fn from(inner: &TestUnionEnum) -> Self {
            Self(inner.clone())
        }
    }
    impl AsRef<TestUnionEnum> for PyTestUnionEnum {
        fn as_ref(&self) -> &TestUnionEnum {
            &self.0
        }
    }
    impl ::rigetti_pyo3::PyWrapper for PyTestUnionEnum {
        type Inner = TestUnionEnum;
    }
    impl ::rigetti_pyo3::pyo3::conversion::ToPyObject for PyTestUnionEnum {
        fn to_object(
            &self,
            py: ::rigetti_pyo3::pyo3::Python,
        ) -> ::rigetti_pyo3::pyo3::PyObject {
            #[allow(clippy::use_self)]
            const NAME: &'static str = "PyTestUnionEnum";
            let cell = ::rigetti_pyo3::pyo3::PyCell::new(py, self.clone())
                .unwrap_or_else(|err| {
                    {
                        ::core::panicking::panic_fmt(
                            format_args!(
                                "failed to create {0} on Python heap: {1}", NAME, err,
                            ),
                        );
                    }
                });
            ::rigetti_pyo3::pyo3::conversion::ToPyObject::to_object(&cell, py)
        }
    }
    impl AsMut<<PyTestUnionEnum as ::rigetti_pyo3::PyWrapper>::Inner>
    for PyTestUnionEnum {
        fn as_mut(
            &mut self,
        ) -> &mut <PyTestUnionEnum as ::rigetti_pyo3::PyWrapper>::Inner {
            &mut self.0
        }
    }
    impl PyTestUnionEnum {
        ///Create a new [`PyTestUnionEnum`] wrapping a [`TestUnionEnum::Unit`].
        pub fn new_unit() -> Self {
            Self::from(TestUnionEnum::Unit)
        }
    }
    const _: () = {
        use ::pyo3 as _pyo3;
        #[allow(non_upper_case_globals)]
        const _: () = {
            static __INVENTORY: ::inventory::Node = ::inventory::Node {
                value: &{
                    type Inventory = <PyTestUnionEnum as _pyo3::impl_::pyclass::PyClassImpl>::Inventory;
                    Inventory::new(_pyo3::impl_::pyclass::PyClassItems {
                        methods: &[
                            _pyo3::class::PyMethodDefType::Static(
                                _pyo3::impl_::pymethods::PyMethodDef::noargs(
                                        "new_unit\0",
                                        _pyo3::impl_::pymethods::PyCFunction({
                                            unsafe extern "C" fn trampoline(
                                                _slf: *mut _pyo3::ffi::PyObject,
                                                _args: *mut _pyo3::ffi::PyObject,
                                            ) -> *mut _pyo3::ffi::PyObject {
                                                _pyo3::impl_::trampoline::noargs(
                                                    _slf,
                                                    _args,
                                                    PyTestUnionEnum::__pymethod_new_unit__,
                                                )
                                            }
                                            trampoline
                                        }),
                                        "new_unit()\n--\n\nCreate a new [`PyTestUnionEnum`] wrapping a [`TestUnionEnum::Unit`].\u{0}",
                                    )
                                    .flags(_pyo3::ffi::METH_STATIC),
                            ),
                        ],
                        slots: &[],
                    })
                },
                next: ::inventory::core::cell::UnsafeCell::new(
                    ::inventory::core::option::Option::None,
                ),
            };
            #[link_section = ".text.startup"]
            unsafe extern "C" fn __ctor() {
                unsafe {
                    ::inventory::ErasedNode::submit(__INVENTORY.value, &__INVENTORY)
                }
            }
            #[used]
            #[link_section = ".init_array"]
            static __CTOR: unsafe extern "C" fn() = __ctor;
        };
        #[doc(hidden)]
        #[allow(non_snake_case)]
        impl PyTestUnionEnum {
            unsafe fn __pymethod_new_unit__<'py>(
                py: _pyo3::Python<'py>,
                _slf: *mut _pyo3::ffi::PyObject,
            ) -> _pyo3::PyResult<*mut _pyo3::ffi::PyObject> {
                let function = PyTestUnionEnum::new_unit;
                _pyo3::impl_::wrap::OkWrap::wrap(function(), py)
                    .map_err(::core::convert::Into::<_pyo3::PyErr>::into)
                    .map(_pyo3::PyObject::into_ptr)
            }
        }
    };
    impl PyTestUnionEnum {
        ///The Python wrapper for [`TestUnionEnum::String`], creating a [`PyTestUnionEnum`] and taking a Python argument.
        pub fn from_string(
            py: ::rigetti_pyo3::pyo3::Python,
            inner: String,
        ) -> ::rigetti_pyo3::pyo3::PyResult<Self> {
            let inner = &inner;
            { <_ as ::rigetti_pyo3::PyTryFrom<String>>::py_try_from(py, inner) }
                .map(TestUnionEnum::String)
                .map(Self)
        }
    }
    const _: () = {
        use ::pyo3 as _pyo3;
        #[allow(non_upper_case_globals)]
        const _: () = {
            static __INVENTORY: ::inventory::Node = ::inventory::Node {
                value: &{
                    type Inventory = <PyTestUnionEnum as _pyo3::impl_::pyclass::PyClassImpl>::Inventory;
                    Inventory::new(_pyo3::impl_::pyclass::PyClassItems {
                        methods: &[
                            _pyo3::class::PyMethodDefType::Static(
                                _pyo3::impl_::pymethods::PyMethodDef::cfunction_with_keywords(
                                        "from_string\0",
                                        _pyo3::impl_::pymethods::PyCFunctionWithKeywords({
                                            unsafe extern "C" fn trampoline(
                                                _slf: *mut _pyo3::ffi::PyObject,
                                                _args: *mut _pyo3::ffi::PyObject,
                                                _kwargs: *mut _pyo3::ffi::PyObject,
                                            ) -> *mut _pyo3::ffi::PyObject {
                                                _pyo3::impl_::trampoline::cfunction_with_keywords(
                                                    _slf,
                                                    _args,
                                                    _kwargs,
                                                    PyTestUnionEnum::__pymethod_from_string__,
                                                )
                                            }
                                            trampoline
                                        }),
                                        "from_string(inner)\n--\n\nThe Python wrapper for [`TestUnionEnum::String`], creating a [`PyTestUnionEnum`] and taking a Python argument.\u{0}",
                                    )
                                    .flags(_pyo3::ffi::METH_STATIC),
                            ),
                        ],
                        slots: &[],
                    })
                },
                next: ::inventory::core::cell::UnsafeCell::new(
                    ::inventory::core::option::Option::None,
                ),
            };
            #[link_section = ".text.startup"]
            unsafe extern "C" fn __ctor() {
                unsafe {
                    ::inventory::ErasedNode::submit(__INVENTORY.value, &__INVENTORY)
                }
            }
            #[used]
            #[link_section = ".init_array"]
            static __CTOR: unsafe extern "C" fn() = __ctor;
        };
        #[doc(hidden)]
        #[allow(non_snake_case)]
        impl PyTestUnionEnum {
            unsafe fn __pymethod_from_string__<'py>(
                py: _pyo3::Python<'py>,
                _slf: *mut _pyo3::ffi::PyObject,
                _args: *mut _pyo3::ffi::PyObject,
                _kwargs: *mut _pyo3::ffi::PyObject,
            ) -> _pyo3::PyResult<*mut _pyo3::ffi::PyObject> {
                let function = PyTestUnionEnum::from_string;
                const DESCRIPTION: _pyo3::impl_::extract_argument::FunctionDescription = _pyo3::impl_::extract_argument::FunctionDescription {
                    cls_name: ::std::option::Option::Some(
                        <PyTestUnionEnum as _pyo3::type_object::PyTypeInfo>::NAME,
                    ),
                    func_name: "from_string",
                    positional_parameter_names: &["inner"],
                    positional_only_parameters: 0usize,
                    required_positional_parameters: 1usize,
                    keyword_only_parameters: &[],
                };
                let mut output = [::std::option::Option::None; 1usize];
                let (_args, _kwargs) = DESCRIPTION
                    .extract_arguments_tuple_dict::<
                        _pyo3::impl_::extract_argument::NoVarargs,
                        _pyo3::impl_::extract_argument::NoVarkeywords,
                    >(py, _args, _kwargs, &mut output)?;
                _pyo3::impl_::wrap::OkWrap::wrap(
                        function(
                            py,
                            _pyo3::impl_::extract_argument::extract_argument(
                                _pyo3::impl_::extract_argument::unwrap_required_argument(
                                    output[0usize],
                                ),
                                &mut {
                                    _pyo3::impl_::extract_argument::FunctionArgumentHolder::INIT
                                },
                                "inner",
                            )?,
                        ),
                        py,
                    )
                    .map_err(::core::convert::Into::<_pyo3::PyErr>::into)
                    .map(_pyo3::PyObject::into_ptr)
            }
        }
    };
    impl PyTestUnionEnum {
        ///Create a new [`PyTestUnionEnum`] from a Python argument; corresponds to `TestUnionEnum.__new__()` in Python
        pub fn new(
            py: ::rigetti_pyo3::pyo3::Python,
            input: &::rigetti_pyo3::pyo3::PyAny,
        ) -> ::rigetti_pyo3::pyo3::PyResult<Self> {
            if let Ok(inner) = <_ as ::rigetti_pyo3::PyTryFrom<
                ::rigetti_pyo3::pyo3::PyAny,
            >>::py_try_from(py, input) {
                let inner = &inner;
                let converted = {
                    <_ as ::rigetti_pyo3::PyTryFrom<String>>::py_try_from(py, inner)
                };
                if let Ok(item) = converted {
                    return Ok(Self::from(TestUnionEnum::String(item)));
                }
            }
            Err(
                ::rigetti_pyo3::pyo3::exceptions::PyValueError::new_err({
                    let res = ::alloc::fmt::format(
                        format_args!(
                            "could not create {0} from {1}", "PyTestUnionEnum", input
                            .repr() ?,
                        ),
                    );
                    res
                }),
            )
        }
        ///Directly return the Python version of the variant discriminant wrapped by this value; i.e., performs the match `TestUnionEnum::Variant(x) => x` for every variant constructor in [`TestUnionEnum`]
        #[allow(unreachable_code, unreachable_pattern)]
        pub fn inner(
            &self,
            py: ::rigetti_pyo3::pyo3::Python,
        ) -> ::rigetti_pyo3::pyo3::PyResult<
            ::rigetti_pyo3::pyo3::Py<::rigetti_pyo3::pyo3::PyAny>,
        > {
            match &self.0 {
                TestUnionEnum::String(inner) => {
                    Ok(
                        ::rigetti_pyo3::pyo3::conversion::IntoPy::<
                            ::rigetti_pyo3::pyo3::Py<::rigetti_pyo3::pyo3::PyAny>,
                        >::into_py(
                            {
                                let inner: String = ::rigetti_pyo3::ToPython::<
                                    String,
                                >::to_python(&inner, py)?;
                                Ok::<_, ::rigetti_pyo3::pyo3::PyErr>(inner)
                            }?,
                            py,
                        ),
                    )
                }
                _ => {
                    use ::rigetti_pyo3::pyo3::exceptions::PyRuntimeError;
                    Err(
                        PyRuntimeError::new_err(
                            "Enum variant has no inner data or is unimplemented",
                        ),
                    )
                }
            }
        }
        ///Tests if this [`PyTestUnionEnum`] wraps a [`TestUnionEnum::unit`] value
        const fn is_unit(&self) -> bool {
            match &self.0 {
                TestUnionEnum::Unit => true,
                _ => false,
            }
        }
        ///Tests if this [`PyTestUnionEnum`] wraps a [`TestUnionEnum::string`] value
        const fn is_string(&self) -> bool {
            match &self.0 {
                TestUnionEnum::String(_) => true,
                _ => false,
            }
        }
        ///Returns `x` if this [`PyTestUnionEnum`] wraps a `TestUnionEnum::string`(x); otherwise returns (Python) `None`.  On the Rust side, this corresponds to either `Some(x)` or [`None`].
        fn as_string(&self, py: ::rigetti_pyo3::pyo3::Python) -> Option<String> {
            self.to_string(py).ok()
        }
        ///Returns `x` if this [`PyTestUnionEnum`] wraps a `TestUnionEnum::string`(x); otherwise raises a `ValueError`.  On the Rust side, this corresponds to either `Ok(x)` or `Err(...)`.
        fn to_string(
            &self,
            py: ::rigetti_pyo3::pyo3::Python,
        ) -> ::rigetti_pyo3::pyo3::PyResult<String> {
            if let TestUnionEnum::String(inner) = &self.0 {
                {
                    let inner: String = ::rigetti_pyo3::ToPython::<
                        String,
                    >::to_python(&inner, py)?;
                    Ok::<_, ::rigetti_pyo3::pyo3::PyErr>(inner)
                }
            } else {
                Err(
                    ::rigetti_pyo3::pyo3::exceptions::PyValueError::new_err(
                        "expected self to be a string",
                    ),
                )
            }
        }
    }
    const _: () = {
        use ::pyo3 as _pyo3;
        #[allow(non_upper_case_globals)]
        const _: () = {
            static __INVENTORY: ::inventory::Node = ::inventory::Node {
                value: &{
                    type Inventory = <PyTestUnionEnum as _pyo3::impl_::pyclass::PyClassImpl>::Inventory;
                    Inventory::new(_pyo3::impl_::pyclass::PyClassItems {
                        methods: &[
                            _pyo3::class::PyMethodDefType::Method(
                                _pyo3::impl_::pymethods::PyMethodDef::noargs(
                                    "inner\0",
                                    _pyo3::impl_::pymethods::PyCFunction({
                                        unsafe extern "C" fn trampoline(
                                            _slf: *mut _pyo3::ffi::PyObject,
                                            _args: *mut _pyo3::ffi::PyObject,
                                        ) -> *mut _pyo3::ffi::PyObject {
                                            _pyo3::impl_::trampoline::noargs(
                                                _slf,
                                                _args,
                                                PyTestUnionEnum::__pymethod_inner__,
                                            )
                                        }
                                        trampoline
                                    }),
                                    "inner($self)\n--\n\nDirectly return the Python version of the variant discriminant wrapped by this value; i.e., performs the match `TestUnionEnum::Variant(x) => x` for every variant constructor in [`TestUnionEnum`]\u{0}",
                                ),
                            ),
                            _pyo3::class::PyMethodDefType::Method(
                                _pyo3::impl_::pymethods::PyMethodDef::noargs(
                                    "is_unit\0",
                                    _pyo3::impl_::pymethods::PyCFunction({
                                        unsafe extern "C" fn trampoline(
                                            _slf: *mut _pyo3::ffi::PyObject,
                                            _args: *mut _pyo3::ffi::PyObject,
                                        ) -> *mut _pyo3::ffi::PyObject {
                                            _pyo3::impl_::trampoline::noargs(
                                                _slf,
                                                _args,
                                                PyTestUnionEnum::__pymethod_is_unit__,
                                            )
                                        }
                                        trampoline
                                    }),
                                    "is_unit($self)\n--\n\nTests if this [`PyTestUnionEnum`] wraps a [`TestUnionEnum::unit`] value\u{0}",
                                ),
                            ),
                            _pyo3::class::PyMethodDefType::Method(
                                _pyo3::impl_::pymethods::PyMethodDef::noargs(
                                    "is_string\0",
                                    _pyo3::impl_::pymethods::PyCFunction({
                                        unsafe extern "C" fn trampoline(
                                            _slf: *mut _pyo3::ffi::PyObject,
                                            _args: *mut _pyo3::ffi::PyObject,
                                        ) -> *mut _pyo3::ffi::PyObject {
                                            _pyo3::impl_::trampoline::noargs(
                                                _slf,
                                                _args,
                                                PyTestUnionEnum::__pymethod_is_string__,
                                            )
                                        }
                                        trampoline
                                    }),
                                    "is_string($self)\n--\n\nTests if this [`PyTestUnionEnum`] wraps a [`TestUnionEnum::string`] value\u{0}",
                                ),
                            ),
                            _pyo3::class::PyMethodDefType::Method(
                                _pyo3::impl_::pymethods::PyMethodDef::noargs(
                                    "as_string\0",
                                    _pyo3::impl_::pymethods::PyCFunction({
                                        unsafe extern "C" fn trampoline(
                                            _slf: *mut _pyo3::ffi::PyObject,
                                            _args: *mut _pyo3::ffi::PyObject,
                                        ) -> *mut _pyo3::ffi::PyObject {
                                            _pyo3::impl_::trampoline::noargs(
                                                _slf,
                                                _args,
                                                PyTestUnionEnum::__pymethod_as_string__,
                                            )
                                        }
                                        trampoline
                                    }),
                                    "as_string($self)\n--\n\nReturns `x` if this [`PyTestUnionEnum`] wraps a `TestUnionEnum::string`(x); otherwise returns (Python) `None`.  On the Rust side, this corresponds to either `Some(x)` or [`None`].\u{0}",
                                ),
                            ),
                            _pyo3::class::PyMethodDefType::Method(
                                _pyo3::impl_::pymethods::PyMethodDef::noargs(
                                    "to_string\0",
                                    _pyo3::impl_::pymethods::PyCFunction({
                                        unsafe extern "C" fn trampoline(
                                            _slf: *mut _pyo3::ffi::PyObject,
                                            _args: *mut _pyo3::ffi::PyObject,
                                        ) -> *mut _pyo3::ffi::PyObject {
                                            _pyo3::impl_::trampoline::noargs(
                                                _slf,
                                                _args,
                                                PyTestUnionEnum::__pymethod_to_string__,
                                            )
                                        }
                                        trampoline
                                    }),
                                    "to_string($self)\n--\n\nReturns `x` if this [`PyTestUnionEnum`] wraps a `TestUnionEnum::string`(x); otherwise raises a `ValueError`.  On the Rust side, this corresponds to either `Ok(x)` or `Err(...)`.\u{0}",
                                ),
                            ),
                        ],
                        slots: &[
                            _pyo3::ffi::PyType_Slot {
                                slot: _pyo3::ffi::Py_tp_new,
                                pfunc: {
                                    unsafe extern "C" fn trampoline(
                                        subtype: *mut _pyo3::ffi::PyTypeObject,
                                        args: *mut _pyo3::ffi::PyObject,
                                        kwargs: *mut _pyo3::ffi::PyObject,
                                    ) -> *mut _pyo3::ffi::PyObject {
                                        use _pyo3::impl_::pyclass::*;
                                        impl PyClassNewTextSignature<PyTestUnionEnum>
                                        for PyClassImplCollector<PyTestUnionEnum> {
                                            #[inline]
                                            fn new_text_signature(
                                                self,
                                            ) -> ::std::option::Option<&'static str> {
                                                ::std::option::Option::Some("(input)")
                                            }
                                        }
                                        _pyo3::impl_::trampoline::newfunc(
                                            subtype,
                                            args,
                                            kwargs,
                                            PyTestUnionEnum::__pymethod___new____,
                                        )
                                    }
                                    trampoline
                                } as _pyo3::ffi::newfunc as _,
                            },
                        ],
                    })
                },
                next: ::inventory::core::cell::UnsafeCell::new(
                    ::inventory::core::option::Option::None,
                ),
            };
            #[link_section = ".text.startup"]
            unsafe extern "C" fn __ctor() {
                unsafe {
                    ::inventory::ErasedNode::submit(__INVENTORY.value, &__INVENTORY)
                }
            }
            #[used]
            #[link_section = ".init_array"]
            static __CTOR: unsafe extern "C" fn() = __ctor;
        };
        #[doc(hidden)]
        #[allow(non_snake_case)]
        impl PyTestUnionEnum {
            unsafe fn __pymethod___new____(
                py: _pyo3::Python<'_>,
                _slf: *mut _pyo3::ffi::PyTypeObject,
                _args: *mut _pyo3::ffi::PyObject,
                _kwargs: *mut _pyo3::ffi::PyObject,
            ) -> _pyo3::PyResult<*mut _pyo3::ffi::PyObject> {
                use _pyo3::callback::IntoPyCallbackOutput;
                let function = PyTestUnionEnum::new;
                const DESCRIPTION: _pyo3::impl_::extract_argument::FunctionDescription = _pyo3::impl_::extract_argument::FunctionDescription {
                    cls_name: ::std::option::Option::Some(
                        <PyTestUnionEnum as _pyo3::type_object::PyTypeInfo>::NAME,
                    ),
                    func_name: "__new__",
                    positional_parameter_names: &["input"],
                    positional_only_parameters: 0usize,
                    required_positional_parameters: 1usize,
                    keyword_only_parameters: &[],
                };
                let mut output = [::std::option::Option::None; 1usize];
                let (_args, _kwargs) = DESCRIPTION
                    .extract_arguments_tuple_dict::<
                        _pyo3::impl_::extract_argument::NoVarargs,
                        _pyo3::impl_::extract_argument::NoVarkeywords,
                    >(py, _args, _kwargs, &mut output)?;
                let result = PyTestUnionEnum::new(
                    py,
                    _pyo3::impl_::extract_argument::extract_argument(
                        _pyo3::impl_::extract_argument::unwrap_required_argument(
                            output[0usize],
                        ),
                        &mut {
                            _pyo3::impl_::extract_argument::FunctionArgumentHolder::INIT
                        },
                        "input",
                    )?,
                );
                let initializer: _pyo3::PyClassInitializer<PyTestUnionEnum> = result
                    .convert(py)?;
                let cell = initializer.create_cell_from_subtype(py, _slf)?;
                ::std::result::Result::Ok(cell as *mut _pyo3::ffi::PyObject)
            }
            unsafe fn __pymethod_inner__<'py>(
                py: _pyo3::Python<'py>,
                _slf: *mut _pyo3::ffi::PyObject,
            ) -> _pyo3::PyResult<*mut _pyo3::ffi::PyObject> {
                let function = PyTestUnionEnum::inner;
                _pyo3::impl_::wrap::OkWrap::wrap(
                        function(
                            _pyo3::impl_::extract_argument::extract_pyclass_ref::<
                                PyTestUnionEnum,
                            >(
                                py.from_borrowed_ptr::<_pyo3::PyAny>(_slf),
                                &mut {
                                    _pyo3::impl_::extract_argument::FunctionArgumentHolder::INIT
                                },
                            )?,
                            py,
                        ),
                        py,
                    )
                    .map_err(::core::convert::Into::<_pyo3::PyErr>::into)
                    .map(_pyo3::PyObject::into_ptr)
            }
            unsafe fn __pymethod_is_unit__<'py>(
                py: _pyo3::Python<'py>,
                _slf: *mut _pyo3::ffi::PyObject,
            ) -> _pyo3::PyResult<*mut _pyo3::ffi::PyObject> {
                let function = PyTestUnionEnum::is_unit;
                _pyo3::impl_::wrap::OkWrap::wrap(
                        function(
                            _pyo3::impl_::extract_argument::extract_pyclass_ref::<
                                PyTestUnionEnum,
                            >(
                                py.from_borrowed_ptr::<_pyo3::PyAny>(_slf),
                                &mut {
                                    _pyo3::impl_::extract_argument::FunctionArgumentHolder::INIT
                                },
                            )?,
                        ),
                        py,
                    )
                    .map_err(::core::convert::Into::<_pyo3::PyErr>::into)
                    .map(_pyo3::PyObject::into_ptr)
            }
            unsafe fn __pymethod_is_string__<'py>(
                py: _pyo3::Python<'py>,
                _slf: *mut _pyo3::ffi::PyObject,
            ) -> _pyo3::PyResult<*mut _pyo3::ffi::PyObject> {
                let function = PyTestUnionEnum::is_string;
                _pyo3::impl_::wrap::OkWrap::wrap(
                        function(
                            _pyo3::impl_::extract_argument::extract_pyclass_ref::<
                                PyTestUnionEnum,
                            >(
                                py.from_borrowed_ptr::<_pyo3::PyAny>(_slf),
                                &mut {
                                    _pyo3::impl_::extract_argument::FunctionArgumentHolder::INIT
                                },
                            )?,
                        ),
                        py,
                    )
                    .map_err(::core::convert::Into::<_pyo3::PyErr>::into)
                    .map(_pyo3::PyObject::into_ptr)
            }
            unsafe fn __pymethod_as_string__<'py>(
                py: _pyo3::Python<'py>,
                _slf: *mut _pyo3::ffi::PyObject,
            ) -> _pyo3::PyResult<*mut _pyo3::ffi::PyObject> {
                let function = PyTestUnionEnum::as_string;
                _pyo3::impl_::wrap::OkWrap::wrap(
                        function(
                            _pyo3::impl_::extract_argument::extract_pyclass_ref::<
                                PyTestUnionEnum,
                            >(
                                py.from_borrowed_ptr::<_pyo3::PyAny>(_slf),
                                &mut {
                                    _pyo3::impl_::extract_argument::FunctionArgumentHolder::INIT
                                },
                            )?,
                            py,
                        ),
                        py,
                    )
                    .map_err(::core::convert::Into::<_pyo3::PyErr>::into)
                    .map(_pyo3::PyObject::into_ptr)
            }
            unsafe fn __pymethod_to_string__<'py>(
                py: _pyo3::Python<'py>,
                _slf: *mut _pyo3::ffi::PyObject,
            ) -> _pyo3::PyResult<*mut _pyo3::ffi::PyObject> {
                let function = PyTestUnionEnum::to_string;
                _pyo3::impl_::wrap::OkWrap::wrap(
                        function(
                            _pyo3::impl_::extract_argument::extract_pyclass_ref::<
                                PyTestUnionEnum,
                            >(
                                py.from_borrowed_ptr::<_pyo3::PyAny>(_slf),
                                &mut {
                                    _pyo3::impl_::extract_argument::FunctionArgumentHolder::INIT
                                },
                            )?,
                            py,
                        ),
                        py,
                    )
                    .map_err(::core::convert::Into::<_pyo3::PyErr>::into)
                    .map(_pyo3::PyObject::into_ptr)
            }
        }
    };
    pub enum PyTestEnumUnaliased {
        One,
        Two,
    }
    const _: () = {
        use ::pyo3 as _pyo3;
        unsafe impl _pyo3::type_object::PyTypeInfo for PyTestEnumUnaliased {
            type AsRefTarget = _pyo3::PyCell<Self>;
            const NAME: &'static str = "TestEnumUnaliased";
            const MODULE: ::std::option::Option<&'static str> = ::core::option::Option::None;
            #[inline]
            fn type_object_raw(py: _pyo3::Python<'_>) -> *mut _pyo3::ffi::PyTypeObject {
                <PyTestEnumUnaliased as _pyo3::impl_::pyclass::PyClassImpl>::lazy_type_object()
                    .get_or_init(py)
                    .as_type_ptr()
            }
        }
        impl _pyo3::PyClass for PyTestEnumUnaliased {
            type Frozen = _pyo3::pyclass::boolean_struct::False;
        }
        impl<'a, 'py> _pyo3::impl_::extract_argument::PyFunctionArgument<'a, 'py>
        for &'a PyTestEnumUnaliased {
            type Holder = ::std::option::Option<_pyo3::PyRef<'py, PyTestEnumUnaliased>>;
            #[inline]
            fn extract(
                obj: &'py _pyo3::PyAny,
                holder: &'a mut Self::Holder,
            ) -> _pyo3::PyResult<Self> {
                _pyo3::impl_::extract_argument::extract_pyclass_ref(obj, holder)
            }
        }
        impl<'a, 'py> _pyo3::impl_::extract_argument::PyFunctionArgument<'a, 'py>
        for &'a mut PyTestEnumUnaliased {
            type Holder = ::std::option::Option<
                _pyo3::PyRefMut<'py, PyTestEnumUnaliased>,
            >;
            #[inline]
            fn extract(
                obj: &'py _pyo3::PyAny,
                holder: &'a mut Self::Holder,
            ) -> _pyo3::PyResult<Self> {
                _pyo3::impl_::extract_argument::extract_pyclass_ref_mut(obj, holder)
            }
        }
        impl _pyo3::IntoPy<_pyo3::PyObject> for PyTestEnumUnaliased {
            fn into_py(self, py: _pyo3::Python) -> _pyo3::PyObject {
                _pyo3::IntoPy::into_py(_pyo3::Py::new(py, self).unwrap(), py)
            }
        }
        impl _pyo3::impl_::pyclass::PyClassImpl for PyTestEnumUnaliased {
            const IS_BASETYPE: bool = false;
            const IS_SUBCLASS: bool = false;
            const IS_MAPPING: bool = false;
            const IS_SEQUENCE: bool = false;
            type BaseType = _pyo3::PyAny;
            type ThreadChecker = _pyo3::impl_::pyclass::SendablePyClass<
                PyTestEnumUnaliased,
            >;
            type Inventory = Pyo3MethodsInventoryForPyTestEnumUnaliased;
            type PyClassMutability = <<_pyo3::PyAny as _pyo3::impl_::pyclass::PyClassBaseType>::PyClassMutability as _pyo3::impl_::pycell::PyClassMutability>::MutableChild;
            type Dict = _pyo3::impl_::pyclass::PyClassDummySlot;
            type WeakRef = _pyo3::impl_::pyclass::PyClassDummySlot;
            type BaseNativeType = _pyo3::PyAny;
            fn items_iter() -> _pyo3::impl_::pyclass::PyClassItemsIter {
                use _pyo3::impl_::pyclass::*;
                let collector = PyClassImplCollector::<Self>::new();
                static INTRINSIC_ITEMS: PyClassItems = PyClassItems {
                    methods: &[
                        _pyo3::class::PyMethodDefType::ClassAttribute({
                            _pyo3::class::PyClassAttributeDef::new(
                                { "One\0" },
                                _pyo3::impl_::pymethods::PyClassAttributeFactory(
                                    PyTestEnumUnaliased::__pymethod_One__,
                                ),
                            )
                        }),
                        _pyo3::class::PyMethodDefType::ClassAttribute({
                            _pyo3::class::PyClassAttributeDef::new(
                                { "Two\0" },
                                _pyo3::impl_::pymethods::PyClassAttributeFactory(
                                    PyTestEnumUnaliased::__pymethod_Two__,
                                ),
                            )
                        }),
                    ],
                    slots: &[
                        {
                            unsafe extern "C" fn trampoline(
                                _slf: *mut _pyo3::ffi::PyObject,
                            ) -> *mut _pyo3::ffi::PyObject {
                                _pyo3::impl_::trampoline::reprfunc(
                                    _slf,
                                    PyTestEnumUnaliased::__pymethod___default___pyo3__repr______,
                                )
                            }
                            _pyo3::ffi::PyType_Slot {
                                slot: _pyo3::ffi::Py_tp_repr,
                                pfunc: trampoline as _pyo3::ffi::reprfunc as _,
                            }
                        },
                        {
                            unsafe extern "C" fn trampoline(
                                _slf: *mut _pyo3::ffi::PyObject,
                            ) -> *mut _pyo3::ffi::PyObject {
                                _pyo3::impl_::trampoline::unaryfunc(
                                    _slf,
                                    PyTestEnumUnaliased::__pymethod___default___pyo3__int______,
                                )
                            }
                            _pyo3::ffi::PyType_Slot {
                                slot: _pyo3::ffi::Py_nb_int,
                                pfunc: trampoline as _pyo3::ffi::unaryfunc as _,
                            }
                        },
                        {
                            unsafe extern "C" fn trampoline(
                                _slf: *mut _pyo3::ffi::PyObject,
                                arg0: *mut _pyo3::ffi::PyObject,
                                arg1: ::std::os::raw::c_int,
                            ) -> *mut _pyo3::ffi::PyObject {
                                _pyo3::impl_::trampoline::richcmpfunc(
                                    _slf,
                                    arg0,
                                    arg1,
                                    PyTestEnumUnaliased::__pymethod___default___pyo3__richcmp______,
                                )
                            }
                            _pyo3::ffi::PyType_Slot {
                                slot: _pyo3::ffi::Py_tp_richcompare,
                                pfunc: trampoline as _pyo3::ffi::richcmpfunc as _,
                            }
                        },
                    ],
                };
                PyClassItemsIter::new(
                    &INTRINSIC_ITEMS,
                    ::std::boxed::Box::new(
                        ::std::iter::Iterator::map(
                            _pyo3::inventory::iter::<
                                <Self as _pyo3::impl_::pyclass::PyClassImpl>::Inventory,
                            >(),
                            _pyo3::impl_::pyclass::PyClassInventory::items,
                        ),
                    ),
                )
            }
            fn doc(py: _pyo3::Python<'_>) -> _pyo3::PyResult<&'static ::std::ffi::CStr> {
                use _pyo3::impl_::pyclass::*;
                static DOC: _pyo3::once_cell::GILOnceCell<
                    ::std::borrow::Cow<'static, ::std::ffi::CStr>,
                > = _pyo3::once_cell::GILOnceCell::new();
                DOC.get_or_try_init(
                        py,
                        || {
                            let collector = PyClassImplCollector::<Self>::new();
                            build_pyclass_doc(
                                <PyTestEnumUnaliased as _pyo3::PyTypeInfo>::NAME,
                                "\0",
                                ::std::option::Option::None
                                    .or_else(|| collector.new_text_signature()),
                            )
                        },
                    )
                    .map(::std::ops::Deref::deref)
            }
            fn lazy_type_object() -> &'static _pyo3::impl_::pyclass::LazyTypeObject<
                Self,
            > {
                use _pyo3::impl_::pyclass::LazyTypeObject;
                static TYPE_OBJECT: LazyTypeObject<PyTestEnumUnaliased> = LazyTypeObject::new();
                &TYPE_OBJECT
            }
        }
        #[doc(hidden)]
        #[allow(non_snake_case)]
        impl PyTestEnumUnaliased {
            fn __pymethod_One__(
                py: _pyo3::Python<'_>,
            ) -> _pyo3::PyResult<_pyo3::PyObject> {
                ::std::result::Result::Ok(
                    _pyo3::IntoPy::into_py(PyTestEnumUnaliased::One, py),
                )
            }
            fn __pymethod_Two__(
                py: _pyo3::Python<'_>,
            ) -> _pyo3::PyResult<_pyo3::PyObject> {
                ::std::result::Result::Ok(
                    _pyo3::IntoPy::into_py(PyTestEnumUnaliased::Two, py),
                )
            }
            unsafe fn __pymethod___default___pyo3__repr______(
                py: _pyo3::Python<'_>,
                _raw_slf: *mut _pyo3::ffi::PyObject,
            ) -> _pyo3::PyResult<*mut _pyo3::ffi::PyObject> {
                let function = PyTestEnumUnaliased::__pyo3__repr__;
                let _slf = _raw_slf;
                _pyo3::callback::convert(
                    py,
                    PyTestEnumUnaliased::__pyo3__repr__(
                        _pyo3::impl_::extract_argument::extract_pyclass_ref::<
                            PyTestEnumUnaliased,
                        >(
                            py.from_borrowed_ptr::<_pyo3::PyAny>(_slf),
                            &mut {
                                _pyo3::impl_::extract_argument::FunctionArgumentHolder::INIT
                            },
                        )?,
                    ),
                )
            }
            unsafe fn __pymethod___default___pyo3__int______(
                py: _pyo3::Python<'_>,
                _raw_slf: *mut _pyo3::ffi::PyObject,
            ) -> _pyo3::PyResult<*mut _pyo3::ffi::PyObject> {
                let function = PyTestEnumUnaliased::__pyo3__int__;
                let _slf = _raw_slf;
                _pyo3::callback::convert(
                    py,
                    PyTestEnumUnaliased::__pyo3__int__(
                        _pyo3::impl_::extract_argument::extract_pyclass_ref::<
                            PyTestEnumUnaliased,
                        >(
                            py.from_borrowed_ptr::<_pyo3::PyAny>(_slf),
                            &mut {
                                _pyo3::impl_::extract_argument::FunctionArgumentHolder::INIT
                            },
                        )?,
                    ),
                )
            }
            unsafe fn __pymethod___default___pyo3__richcmp______(
                py: _pyo3::Python<'_>,
                _raw_slf: *mut _pyo3::ffi::PyObject,
                arg0: *mut _pyo3::ffi::PyObject,
                arg1: ::std::os::raw::c_int,
            ) -> _pyo3::PyResult<*mut _pyo3::ffi::PyObject> {
                let function = PyTestEnumUnaliased::__pyo3__richcmp__;
                let _slf = _raw_slf;
                _pyo3::callback::convert(
                    py,
                    PyTestEnumUnaliased::__pyo3__richcmp__(
                        match _pyo3::impl_::extract_argument::extract_pyclass_ref::<
                            PyTestEnumUnaliased,
                        >(
                            py.from_borrowed_ptr::<_pyo3::PyAny>(_slf),
                            &mut {
                                _pyo3::impl_::extract_argument::FunctionArgumentHolder::INIT
                            },
                        ) {
                            ::std::result::Result::Ok(value) => value,
                            ::std::result::Result::Err(_) => {
                                return _pyo3::callback::convert(py, py.NotImplemented());
                            }
                        },
                        py,
                        match _pyo3::impl_::extract_argument::extract_argument(
                            py.from_borrowed_ptr::<_pyo3::PyAny>(arg0),
                            &mut {
                                _pyo3::impl_::extract_argument::FunctionArgumentHolder::INIT
                            },
                            "other",
                        ) {
                            ::std::result::Result::Ok(value) => value,
                            ::std::result::Result::Err(_) => {
                                return _pyo3::callback::convert(py, py.NotImplemented());
                            }
                        },
                        match _pyo3::class::basic::CompareOp::from_raw(arg1)
                            .ok_or_else(|| _pyo3::exceptions::PyValueError::new_err(
                                "invalid comparison operator",
                            ))
                        {
                            ::std::result::Result::Ok(value) => value,
                            ::std::result::Result::Err(_) => {
                                return _pyo3::callback::convert(py, py.NotImplemented());
                            }
                        },
                    ),
                )
            }
        }
        #[doc(hidden)]
        pub struct Pyo3MethodsInventoryForPyTestEnumUnaliased {
            items: _pyo3::impl_::pyclass::PyClassItems,
        }
        impl Pyo3MethodsInventoryForPyTestEnumUnaliased {
            pub const fn new(items: _pyo3::impl_::pyclass::PyClassItems) -> Self {
                Self { items }
            }
        }
        impl _pyo3::impl_::pyclass::PyClassInventory
        for Pyo3MethodsInventoryForPyTestEnumUnaliased {
            fn items(&self) -> &_pyo3::impl_::pyclass::PyClassItems {
                &self.items
            }
        }
        impl ::inventory::Collect for Pyo3MethodsInventoryForPyTestEnumUnaliased {
            #[inline]
            fn registry() -> &'static ::inventory::Registry {
                static REGISTRY: ::inventory::Registry = ::inventory::Registry::new();
                &REGISTRY
            }
        }
        #[doc(hidden)]
        #[allow(non_snake_case)]
        impl PyTestEnumUnaliased {
            fn __pyo3__repr__(&self) -> &'static str {
                match self {
                    PyTestEnumUnaliased::One => "TestEnumUnaliased.One",
                    PyTestEnumUnaliased::Two => "TestEnumUnaliased.Two",
                }
            }
            fn __pyo3__int__(&self) -> isize {
                match self {
                    PyTestEnumUnaliased::One => PyTestEnumUnaliased::One as isize,
                    PyTestEnumUnaliased::Two => PyTestEnumUnaliased::Two as isize,
                }
            }
            fn __pyo3__richcmp__(
                &self,
                py: _pyo3::Python,
                other: &_pyo3::PyAny,
                op: _pyo3::basic::CompareOp,
            ) -> _pyo3::PyResult<_pyo3::PyObject> {
                use _pyo3::conversion::ToPyObject;
                use ::core::result::Result::*;
                match op {
                    _pyo3::basic::CompareOp::Eq => {
                        let self_val = self.__pyo3__int__();
                        if let Ok(i) = other.extract::<isize>() {
                            return Ok((self_val == i).to_object(py));
                        }
                        if let Ok(other) = other.extract::<_pyo3::PyRef<Self>>() {
                            return Ok((self_val == other.__pyo3__int__()).to_object(py));
                        }
                        return Ok(py.NotImplemented());
                    }
                    _pyo3::basic::CompareOp::Ne => {
                        let self_val = self.__pyo3__int__();
                        if let Ok(i) = other.extract::<isize>() {
                            return Ok((self_val != i).to_object(py));
                        }
                        if let Ok(other) = other.extract::<_pyo3::PyRef<Self>>() {
                            return Ok((self_val != other.__pyo3__int__()).to_object(py));
                        }
                        return Ok(py.NotImplemented());
                    }
                    _ => Ok(py.NotImplemented()),
                }
            }
        }
    };
    #[automatically_derived]
    impl ::core::marker::Copy for PyTestEnumUnaliased {}
    #[automatically_derived]
    impl ::core::clone::Clone for PyTestEnumUnaliased {
        #[inline]
        fn clone(&self) -> PyTestEnumUnaliased {
            *self
        }
    }
    impl From<PyTestEnumUnaliased> for TestEnum {
        fn from(item: PyTestEnumUnaliased) -> Self {
            match item {
                PyTestEnumUnaliased::One => Self::One,
                PyTestEnumUnaliased::Two => Self::Two,
            }
        }
    }
    impl From<&PyTestEnumUnaliased> for TestEnum {
        fn from(item: &PyTestEnumUnaliased) -> Self {
            Self::from(*item)
        }
    }
    impl From<TestEnum> for PyTestEnumUnaliased {
        fn from(item: TestEnum) -> Self {
            match item {
                TestEnum::One => PyTestEnumUnaliased::One,
                TestEnum::Two => PyTestEnumUnaliased::Two,
            }
        }
    }
    impl From<&TestEnum> for PyTestEnumUnaliased {
        fn from(item: &TestEnum) -> Self {
            Self::from(*item)
        }
    }
    impl ::rigetti_pyo3::PyWrapper for PyTestEnumUnaliased {
        type Inner = TestEnum;
    }
    impl AsRef<TestEnum> for PyTestEnumUnaliased {
        fn as_ref(&self) -> &TestEnum {
            match self {
                PyTestEnumUnaliased::One => &TestEnum::One,
                PyTestEnumUnaliased::Two => &TestEnum::Two,
            }
        }
    }
    impl ::rigetti_pyo3::pyo3::conversion::ToPyObject for PyTestEnumUnaliased {
        fn to_object(
            &self,
            py: ::rigetti_pyo3::pyo3::Python,
        ) -> ::rigetti_pyo3::pyo3::PyObject {
            let cell = ::rigetti_pyo3::pyo3::PyCell::new(py, self.clone())
                .unwrap_or_else(|err| {
                    ::core::panicking::panic_fmt(
                        format_args!(
                            "failed to create {0} on Python heap: {1}",
                            "PyTestEnumUnaliased", err,
                        ),
                    );
                });
            cell.to_object(py)
        }
    }
    #[allow(clippy::use_self)]
    impl ::rigetti_pyo3::ToPython<PyTestEnumUnaliased> for TestEnum {
        fn to_python(
            &self,
            _py: ::rigetti_pyo3::pyo3::Python<'_>,
        ) -> ::rigetti_pyo3::pyo3::PyResult<PyTestEnumUnaliased> {
            { Ok(PyTestEnumUnaliased::from(self)) }
        }
    }
    #[allow(clippy::use_self)]
    impl<'a> ::rigetti_pyo3::ToPython<PyTestEnumUnaliased> for &'a TestEnum {
        fn to_python(
            &self,
            _py: ::rigetti_pyo3::pyo3::Python<'_>,
        ) -> ::rigetti_pyo3::pyo3::PyResult<PyTestEnumUnaliased> {
            {
                <TestEnum as ::rigetti_pyo3::ToPython<
                    PyTestEnumUnaliased,
                >>::to_python(*self, _py)
            }
        }
    }
    #[allow(clippy::use_self)]
    impl ::rigetti_pyo3::PyTryFrom<PyTestEnumUnaliased> for TestEnum {
        fn py_try_from(
            _py: ::rigetti_pyo3::pyo3::Python,
            item: &PyTestEnumUnaliased,
        ) -> ::rigetti_pyo3::pyo3::PyResult<Self> {
            { Ok(*item.as_ref()) }
        }
    }
    pub enum PyTestEnumAliased {
        NONE,
        Two,
    }
    const _: () = {
        use ::pyo3 as _pyo3;
        unsafe impl _pyo3::type_object::PyTypeInfo for PyTestEnumAliased {
            type AsRefTarget = _pyo3::PyCell<Self>;
            const NAME: &'static str = "TestEnumAliased";
            const MODULE: ::std::option::Option<&'static str> = ::core::option::Option::None;
            #[inline]
            fn type_object_raw(py: _pyo3::Python<'_>) -> *mut _pyo3::ffi::PyTypeObject {
                <PyTestEnumAliased as _pyo3::impl_::pyclass::PyClassImpl>::lazy_type_object()
                    .get_or_init(py)
                    .as_type_ptr()
            }
        }
        impl _pyo3::PyClass for PyTestEnumAliased {
            type Frozen = _pyo3::pyclass::boolean_struct::False;
        }
        impl<'a, 'py> _pyo3::impl_::extract_argument::PyFunctionArgument<'a, 'py>
        for &'a PyTestEnumAliased {
            type Holder = ::std::option::Option<_pyo3::PyRef<'py, PyTestEnumAliased>>;
            #[inline]
            fn extract(
                obj: &'py _pyo3::PyAny,
                holder: &'a mut Self::Holder,
            ) -> _pyo3::PyResult<Self> {
                _pyo3::impl_::extract_argument::extract_pyclass_ref(obj, holder)
            }
        }
        impl<'a, 'py> _pyo3::impl_::extract_argument::PyFunctionArgument<'a, 'py>
        for &'a mut PyTestEnumAliased {
            type Holder = ::std::option::Option<_pyo3::PyRefMut<'py, PyTestEnumAliased>>;
            #[inline]
            fn extract(
                obj: &'py _pyo3::PyAny,
                holder: &'a mut Self::Holder,
            ) -> _pyo3::PyResult<Self> {
                _pyo3::impl_::extract_argument::extract_pyclass_ref_mut(obj, holder)
            }
        }
        impl _pyo3::IntoPy<_pyo3::PyObject> for PyTestEnumAliased {
            fn into_py(self, py: _pyo3::Python) -> _pyo3::PyObject {
                _pyo3::IntoPy::into_py(_pyo3::Py::new(py, self).unwrap(), py)
            }
        }
        impl _pyo3::impl_::pyclass::PyClassImpl for PyTestEnumAliased {
            const IS_BASETYPE: bool = false;
            const IS_SUBCLASS: bool = false;
            const IS_MAPPING: bool = false;
            const IS_SEQUENCE: bool = false;
            type BaseType = _pyo3::PyAny;
            type ThreadChecker = _pyo3::impl_::pyclass::SendablePyClass<
                PyTestEnumAliased,
            >;
            type Inventory = Pyo3MethodsInventoryForPyTestEnumAliased;
            type PyClassMutability = <<_pyo3::PyAny as _pyo3::impl_::pyclass::PyClassBaseType>::PyClassMutability as _pyo3::impl_::pycell::PyClassMutability>::MutableChild;
            type Dict = _pyo3::impl_::pyclass::PyClassDummySlot;
            type WeakRef = _pyo3::impl_::pyclass::PyClassDummySlot;
            type BaseNativeType = _pyo3::PyAny;
            fn items_iter() -> _pyo3::impl_::pyclass::PyClassItemsIter {
                use _pyo3::impl_::pyclass::*;
                let collector = PyClassImplCollector::<Self>::new();
                static INTRINSIC_ITEMS: PyClassItems = PyClassItems {
                    methods: &[
                        _pyo3::class::PyMethodDefType::ClassAttribute({
                            _pyo3::class::PyClassAttributeDef::new(
                                { "NONE\0" },
                                _pyo3::impl_::pymethods::PyClassAttributeFactory(
                                    PyTestEnumAliased::__pymethod_NONE__,
                                ),
                            )
                        }),
                        _pyo3::class::PyMethodDefType::ClassAttribute({
                            _pyo3::class::PyClassAttributeDef::new(
                                { "Two\0" },
                                _pyo3::impl_::pymethods::PyClassAttributeFactory(
                                    PyTestEnumAliased::__pymethod_Two__,
                                ),
                            )
                        }),
                    ],
                    slots: &[
                        {
                            unsafe extern "C" fn trampoline(
                                _slf: *mut _pyo3::ffi::PyObject,
                            ) -> *mut _pyo3::ffi::PyObject {
                                _pyo3::impl_::trampoline::reprfunc(
                                    _slf,
                                    PyTestEnumAliased::__pymethod___default___pyo3__repr______,
                                )
                            }
                            _pyo3::ffi::PyType_Slot {
                                slot: _pyo3::ffi::Py_tp_repr,
                                pfunc: trampoline as _pyo3::ffi::reprfunc as _,
                            }
                        },
                        {
                            unsafe extern "C" fn trampoline(
                                _slf: *mut _pyo3::ffi::PyObject,
                            ) -> *mut _pyo3::ffi::PyObject {
                                _pyo3::impl_::trampoline::unaryfunc(
                                    _slf,
                                    PyTestEnumAliased::__pymethod___default___pyo3__int______,
                                )
                            }
                            _pyo3::ffi::PyType_Slot {
                                slot: _pyo3::ffi::Py_nb_int,
                                pfunc: trampoline as _pyo3::ffi::unaryfunc as _,
                            }
                        },
                        {
                            unsafe extern "C" fn trampoline(
                                _slf: *mut _pyo3::ffi::PyObject,
                                arg0: *mut _pyo3::ffi::PyObject,
                                arg1: ::std::os::raw::c_int,
                            ) -> *mut _pyo3::ffi::PyObject {
                                _pyo3::impl_::trampoline::richcmpfunc(
                                    _slf,
                                    arg0,
                                    arg1,
                                    PyTestEnumAliased::__pymethod___default___pyo3__richcmp______,
                                )
                            }
                            _pyo3::ffi::PyType_Slot {
                                slot: _pyo3::ffi::Py_tp_richcompare,
                                pfunc: trampoline as _pyo3::ffi::richcmpfunc as _,
                            }
                        },
                    ],
                };
                PyClassItemsIter::new(
                    &INTRINSIC_ITEMS,
                    ::std::boxed::Box::new(
                        ::std::iter::Iterator::map(
                            _pyo3::inventory::iter::<
                                <Self as _pyo3::impl_::pyclass::PyClassImpl>::Inventory,
                            >(),
                            _pyo3::impl_::pyclass::PyClassInventory::items,
                        ),
                    ),
                )
            }
            fn doc(py: _pyo3::Python<'_>) -> _pyo3::PyResult<&'static ::std::ffi::CStr> {
                use _pyo3::impl_::pyclass::*;
                static DOC: _pyo3::once_cell::GILOnceCell<
                    ::std::borrow::Cow<'static, ::std::ffi::CStr>,
                > = _pyo3::once_cell::GILOnceCell::new();
                DOC.get_or_try_init(
                        py,
                        || {
                            let collector = PyClassImplCollector::<Self>::new();
                            build_pyclass_doc(
                                <PyTestEnumAliased as _pyo3::PyTypeInfo>::NAME,
                                "\0",
                                ::std::option::Option::None
                                    .or_else(|| collector.new_text_signature()),
                            )
                        },
                    )
                    .map(::std::ops::Deref::deref)
            }
            fn lazy_type_object() -> &'static _pyo3::impl_::pyclass::LazyTypeObject<
                Self,
            > {
                use _pyo3::impl_::pyclass::LazyTypeObject;
                static TYPE_OBJECT: LazyTypeObject<PyTestEnumAliased> = LazyTypeObject::new();
                &TYPE_OBJECT
            }
        }
        #[doc(hidden)]
        #[allow(non_snake_case)]
        impl PyTestEnumAliased {
            fn __pymethod_NONE__(
                py: _pyo3::Python<'_>,
            ) -> _pyo3::PyResult<_pyo3::PyObject> {
                ::std::result::Result::Ok(
                    _pyo3::IntoPy::into_py(PyTestEnumAliased::NONE, py),
                )
            }
            fn __pymethod_Two__(
                py: _pyo3::Python<'_>,
            ) -> _pyo3::PyResult<_pyo3::PyObject> {
                ::std::result::Result::Ok(
                    _pyo3::IntoPy::into_py(PyTestEnumAliased::Two, py),
                )
            }
            unsafe fn __pymethod___default___pyo3__repr______(
                py: _pyo3::Python<'_>,
                _raw_slf: *mut _pyo3::ffi::PyObject,
            ) -> _pyo3::PyResult<*mut _pyo3::ffi::PyObject> {
                let function = PyTestEnumAliased::__pyo3__repr__;
                let _slf = _raw_slf;
                _pyo3::callback::convert(
                    py,
                    PyTestEnumAliased::__pyo3__repr__(
                        _pyo3::impl_::extract_argument::extract_pyclass_ref::<
                            PyTestEnumAliased,
                        >(
                            py.from_borrowed_ptr::<_pyo3::PyAny>(_slf),
                            &mut {
                                _pyo3::impl_::extract_argument::FunctionArgumentHolder::INIT
                            },
                        )?,
                    ),
                )
            }
            unsafe fn __pymethod___default___pyo3__int______(
                py: _pyo3::Python<'_>,
                _raw_slf: *mut _pyo3::ffi::PyObject,
            ) -> _pyo3::PyResult<*mut _pyo3::ffi::PyObject> {
                let function = PyTestEnumAliased::__pyo3__int__;
                let _slf = _raw_slf;
                _pyo3::callback::convert(
                    py,
                    PyTestEnumAliased::__pyo3__int__(
                        _pyo3::impl_::extract_argument::extract_pyclass_ref::<
                            PyTestEnumAliased,
                        >(
                            py.from_borrowed_ptr::<_pyo3::PyAny>(_slf),
                            &mut {
                                _pyo3::impl_::extract_argument::FunctionArgumentHolder::INIT
                            },
                        )?,
                    ),
                )
            }
            unsafe fn __pymethod___default___pyo3__richcmp______(
                py: _pyo3::Python<'_>,
                _raw_slf: *mut _pyo3::ffi::PyObject,
                arg0: *mut _pyo3::ffi::PyObject,
                arg1: ::std::os::raw::c_int,
            ) -> _pyo3::PyResult<*mut _pyo3::ffi::PyObject> {
                let function = PyTestEnumAliased::__pyo3__richcmp__;
                let _slf = _raw_slf;
                _pyo3::callback::convert(
                    py,
                    PyTestEnumAliased::__pyo3__richcmp__(
                        match _pyo3::impl_::extract_argument::extract_pyclass_ref::<
                            PyTestEnumAliased,
                        >(
                            py.from_borrowed_ptr::<_pyo3::PyAny>(_slf),
                            &mut {
                                _pyo3::impl_::extract_argument::FunctionArgumentHolder::INIT
                            },
                        ) {
                            ::std::result::Result::Ok(value) => value,
                            ::std::result::Result::Err(_) => {
                                return _pyo3::callback::convert(py, py.NotImplemented());
                            }
                        },
                        py,
                        match _pyo3::impl_::extract_argument::extract_argument(
                            py.from_borrowed_ptr::<_pyo3::PyAny>(arg0),
                            &mut {
                                _pyo3::impl_::extract_argument::FunctionArgumentHolder::INIT
                            },
                            "other",
                        ) {
                            ::std::result::Result::Ok(value) => value,
                            ::std::result::Result::Err(_) => {
                                return _pyo3::callback::convert(py, py.NotImplemented());
                            }
                        },
                        match _pyo3::class::basic::CompareOp::from_raw(arg1)
                            .ok_or_else(|| _pyo3::exceptions::PyValueError::new_err(
                                "invalid comparison operator",
                            ))
                        {
                            ::std::result::Result::Ok(value) => value,
                            ::std::result::Result::Err(_) => {
                                return _pyo3::callback::convert(py, py.NotImplemented());
                            }
                        },
                    ),
                )
            }
        }
        #[doc(hidden)]
        pub struct Pyo3MethodsInventoryForPyTestEnumAliased {
            items: _pyo3::impl_::pyclass::PyClassItems,
        }
        impl Pyo3MethodsInventoryForPyTestEnumAliased {
            pub const fn new(items: _pyo3::impl_::pyclass::PyClassItems) -> Self {
                Self { items }
            }
        }
        impl _pyo3::impl_::pyclass::PyClassInventory
        for Pyo3MethodsInventoryForPyTestEnumAliased {
            fn items(&self) -> &_pyo3::impl_::pyclass::PyClassItems {
                &self.items
            }
        }
        impl ::inventory::Collect for Pyo3MethodsInventoryForPyTestEnumAliased {
            #[inline]
            fn registry() -> &'static ::inventory::Registry {
                static REGISTRY: ::inventory::Registry = ::inventory::Registry::new();
                &REGISTRY
            }
        }
        #[doc(hidden)]
        #[allow(non_snake_case)]
        impl PyTestEnumAliased {
            fn __pyo3__repr__(&self) -> &'static str {
                match self {
                    PyTestEnumAliased::NONE => "TestEnumAliased.NONE",
                    PyTestEnumAliased::Two => "TestEnumAliased.Two",
                }
            }
            fn __pyo3__int__(&self) -> isize {
                match self {
                    PyTestEnumAliased::NONE => PyTestEnumAliased::NONE as isize,
                    PyTestEnumAliased::Two => PyTestEnumAliased::Two as isize,
                }
            }
            fn __pyo3__richcmp__(
                &self,
                py: _pyo3::Python,
                other: &_pyo3::PyAny,
                op: _pyo3::basic::CompareOp,
            ) -> _pyo3::PyResult<_pyo3::PyObject> {
                use _pyo3::conversion::ToPyObject;
                use ::core::result::Result::*;
                match op {
                    _pyo3::basic::CompareOp::Eq => {
                        let self_val = self.__pyo3__int__();
                        if let Ok(i) = other.extract::<isize>() {
                            return Ok((self_val == i).to_object(py));
                        }
                        if let Ok(other) = other.extract::<_pyo3::PyRef<Self>>() {
                            return Ok((self_val == other.__pyo3__int__()).to_object(py));
                        }
                        return Ok(py.NotImplemented());
                    }
                    _pyo3::basic::CompareOp::Ne => {
                        let self_val = self.__pyo3__int__();
                        if let Ok(i) = other.extract::<isize>() {
                            return Ok((self_val != i).to_object(py));
                        }
                        if let Ok(other) = other.extract::<_pyo3::PyRef<Self>>() {
                            return Ok((self_val != other.__pyo3__int__()).to_object(py));
                        }
                        return Ok(py.NotImplemented());
                    }
                    _ => Ok(py.NotImplemented()),
                }
            }
        }
    };
    #[automatically_derived]
    impl ::core::marker::Copy for PyTestEnumAliased {}
    #[automatically_derived]
    impl ::core::clone::Clone for PyTestEnumAliased {
        #[inline]
        fn clone(&self) -> PyTestEnumAliased {
            *self
        }
    }
    impl From<PyTestEnumAliased> for TestEnum {
        fn from(item: PyTestEnumAliased) -> Self {
            match item {
                PyTestEnumAliased::NONE => Self::One,
                PyTestEnumAliased::Two => Self::Two,
            }
        }
    }
    impl From<&PyTestEnumAliased> for TestEnum {
        fn from(item: &PyTestEnumAliased) -> Self {
            Self::from(*item)
        }
    }
    impl From<TestEnum> for PyTestEnumAliased {
        fn from(item: TestEnum) -> Self {
            match item {
                TestEnum::One => PyTestEnumAliased::NONE,
                TestEnum::Two => PyTestEnumAliased::Two,
            }
        }
    }
    impl From<&TestEnum> for PyTestEnumAliased {
        fn from(item: &TestEnum) -> Self {
            Self::from(*item)
        }
    }
    impl ::rigetti_pyo3::PyWrapper for PyTestEnumAliased {
        type Inner = TestEnum;
    }
    impl AsRef<TestEnum> for PyTestEnumAliased {
        fn as_ref(&self) -> &TestEnum {
            match self {
                PyTestEnumAliased::NONE => &TestEnum::One,
                PyTestEnumAliased::Two => &TestEnum::Two,
            }
        }
    }
    impl ::rigetti_pyo3::pyo3::conversion::ToPyObject for PyTestEnumAliased {
        fn to_object(
            &self,
            py: ::rigetti_pyo3::pyo3::Python,
        ) -> ::rigetti_pyo3::pyo3::PyObject {
            let cell = ::rigetti_pyo3::pyo3::PyCell::new(py, self.clone())
                .unwrap_or_else(|err| {
                    ::core::panicking::panic_fmt(
                        format_args!(
                            "failed to create {0} on Python heap: {1}",
                            "PyTestEnumAliased", err,
                        ),
                    );
                });
            cell.to_object(py)
        }
    }
    #[allow(clippy::use_self)]
    impl ::rigetti_pyo3::ToPython<PyTestEnumAliased> for TestEnum {
        fn to_python(
            &self,
            _py: ::rigetti_pyo3::pyo3::Python<'_>,
        ) -> ::rigetti_pyo3::pyo3::PyResult<PyTestEnumAliased> {
            { Ok(PyTestEnumAliased::from(self)) }
        }
    }
    #[allow(clippy::use_self)]
    impl<'a> ::rigetti_pyo3::ToPython<PyTestEnumAliased> for &'a TestEnum {
        fn to_python(
            &self,
            _py: ::rigetti_pyo3::pyo3::Python<'_>,
        ) -> ::rigetti_pyo3::pyo3::PyResult<PyTestEnumAliased> {
            {
                <TestEnum as ::rigetti_pyo3::ToPython<
                    PyTestEnumAliased,
                >>::to_python(*self, _py)
            }
        }
    }
    #[allow(clippy::use_self)]
    impl ::rigetti_pyo3::PyTryFrom<PyTestEnumAliased> for TestEnum {
        fn py_try_from(
            _py: ::rigetti_pyo3::pyo3::Python,
            item: &PyTestEnumAliased,
        ) -> ::rigetti_pyo3::pyo3::PyResult<Self> {
            { Ok(*item.as_ref()) }
        }
    }
    #[repr(transparent)]
    #[allow(clippy::use_self)]
    pub struct PyTestStruct(TestStruct);
    #[automatically_derived]
    #[allow(clippy::use_self)]
    impl ::core::clone::Clone for PyTestStruct {
        #[inline]
        fn clone(&self) -> PyTestStruct {
            PyTestStruct(::core::clone::Clone::clone(&self.0))
        }
    }
    const _: () = {
        use ::pyo3 as _pyo3;
        unsafe impl _pyo3::type_object::PyTypeInfo for PyTestStruct {
            type AsRefTarget = _pyo3::PyCell<Self>;
            const NAME: &'static str = "TestStruct";
            const MODULE: ::std::option::Option<&'static str> = ::core::option::Option::None;
            #[inline]
            fn type_object_raw(py: _pyo3::Python<'_>) -> *mut _pyo3::ffi::PyTypeObject {
                <PyTestStruct as _pyo3::impl_::pyclass::PyClassImpl>::lazy_type_object()
                    .get_or_init(py)
                    .as_type_ptr()
            }
        }
        impl _pyo3::PyClass for PyTestStruct {
            type Frozen = _pyo3::pyclass::boolean_struct::False;
        }
        impl<'a, 'py> _pyo3::impl_::extract_argument::PyFunctionArgument<'a, 'py>
        for &'a PyTestStruct {
            type Holder = ::std::option::Option<_pyo3::PyRef<'py, PyTestStruct>>;
            #[inline]
            fn extract(
                obj: &'py _pyo3::PyAny,
                holder: &'a mut Self::Holder,
            ) -> _pyo3::PyResult<Self> {
                _pyo3::impl_::extract_argument::extract_pyclass_ref(obj, holder)
            }
        }
        impl<'a, 'py> _pyo3::impl_::extract_argument::PyFunctionArgument<'a, 'py>
        for &'a mut PyTestStruct {
            type Holder = ::std::option::Option<_pyo3::PyRefMut<'py, PyTestStruct>>;
            #[inline]
            fn extract(
                obj: &'py _pyo3::PyAny,
                holder: &'a mut Self::Holder,
            ) -> _pyo3::PyResult<Self> {
                _pyo3::impl_::extract_argument::extract_pyclass_ref_mut(obj, holder)
            }
        }
        impl _pyo3::IntoPy<_pyo3::PyObject> for PyTestStruct {
            fn into_py(self, py: _pyo3::Python) -> _pyo3::PyObject {
                _pyo3::IntoPy::into_py(_pyo3::Py::new(py, self).unwrap(), py)
            }
        }
        impl _pyo3::impl_::pyclass::PyClassImpl for PyTestStruct {
            const IS_BASETYPE: bool = false;
            const IS_SUBCLASS: bool = false;
            const IS_MAPPING: bool = false;
            const IS_SEQUENCE: bool = false;
            type BaseType = _pyo3::PyAny;
            type ThreadChecker = _pyo3::impl_::pyclass::SendablePyClass<PyTestStruct>;
            type Inventory = Pyo3MethodsInventoryForPyTestStruct;
            type PyClassMutability = <<_pyo3::PyAny as _pyo3::impl_::pyclass::PyClassBaseType>::PyClassMutability as _pyo3::impl_::pycell::PyClassMutability>::MutableChild;
            type Dict = _pyo3::impl_::pyclass::PyClassDummySlot;
            type WeakRef = _pyo3::impl_::pyclass::PyClassDummySlot;
            type BaseNativeType = _pyo3::PyAny;
            fn items_iter() -> _pyo3::impl_::pyclass::PyClassItemsIter {
                use _pyo3::impl_::pyclass::*;
                let collector = PyClassImplCollector::<Self>::new();
                static INTRINSIC_ITEMS: PyClassItems = PyClassItems {
                    methods: &[],
                    slots: &[],
                };
                PyClassItemsIter::new(
                    &INTRINSIC_ITEMS,
                    ::std::boxed::Box::new(
                        ::std::iter::Iterator::map(
                            _pyo3::inventory::iter::<
                                <Self as _pyo3::impl_::pyclass::PyClassImpl>::Inventory,
                            >(),
                            _pyo3::impl_::pyclass::PyClassInventory::items,
                        ),
                    ),
                )
            }
            fn doc(py: _pyo3::Python<'_>) -> _pyo3::PyResult<&'static ::std::ffi::CStr> {
                use _pyo3::impl_::pyclass::*;
                static DOC: _pyo3::once_cell::GILOnceCell<
                    ::std::borrow::Cow<'static, ::std::ffi::CStr>,
                > = _pyo3::once_cell::GILOnceCell::new();
                DOC.get_or_try_init(
                        py,
                        || {
                            let collector = PyClassImplCollector::<Self>::new();
                            build_pyclass_doc(
                                <PyTestStruct as _pyo3::PyTypeInfo>::NAME,
                                "\0",
                                ::std::option::Option::None
                                    .or_else(|| collector.new_text_signature()),
                            )
                        },
                    )
                    .map(::std::ops::Deref::deref)
            }
            fn lazy_type_object() -> &'static _pyo3::impl_::pyclass::LazyTypeObject<
                Self,
            > {
                use _pyo3::impl_::pyclass::LazyTypeObject;
                static TYPE_OBJECT: LazyTypeObject<PyTestStruct> = LazyTypeObject::new();
                &TYPE_OBJECT
            }
        }
        #[doc(hidden)]
        #[allow(non_snake_case)]
        impl PyTestStruct {}
        #[doc(hidden)]
        pub struct Pyo3MethodsInventoryForPyTestStruct {
            items: _pyo3::impl_::pyclass::PyClassItems,
        }
        impl Pyo3MethodsInventoryForPyTestStruct {
            pub const fn new(items: _pyo3::impl_::pyclass::PyClassItems) -> Self {
                Self { items }
            }
        }
        impl _pyo3::impl_::pyclass::PyClassInventory
        for Pyo3MethodsInventoryForPyTestStruct {
            fn items(&self) -> &_pyo3::impl_::pyclass::PyClassItems {
                &self.items
            }
        }
        impl ::inventory::Collect for Pyo3MethodsInventoryForPyTestStruct {
            #[inline]
            fn registry() -> &'static ::inventory::Registry {
                static REGISTRY: ::inventory::Registry = ::inventory::Registry::new();
                &REGISTRY
            }
        }
    };
    impl ::rigetti_pyo3::PyTryFrom<PyTestStruct> for TestStruct {
        fn py_try_from(
            py: ::rigetti_pyo3::pyo3::Python,
            item: &PyTestStruct,
        ) -> ::rigetti_pyo3::pyo3::PyResult<Self> {
            Ok(item.0.clone())
        }
    }
    impl ::rigetti_pyo3::PyTryFrom<::rigetti_pyo3::pyo3::PyAny> for PyTestStruct {
        fn py_try_from(
            py: ::rigetti_pyo3::pyo3::Python,
            item: &::rigetti_pyo3::pyo3::PyAny,
        ) -> ::rigetti_pyo3::pyo3::PyResult<Self> {
            item.extract()
        }
    }
    impl ::rigetti_pyo3::PyTryFrom<PyTestStruct> for PyTestStruct {
        fn py_try_from(
            py: ::rigetti_pyo3::pyo3::Python,
            item: &PyTestStruct,
        ) -> ::rigetti_pyo3::pyo3::PyResult<Self> {
            Ok(item.clone())
        }
    }
    #[allow(clippy::use_self)]
    impl ::rigetti_pyo3::ToPython<PyTestStruct> for TestStruct {
        fn to_python(
            &self,
            py: ::rigetti_pyo3::pyo3::Python<'_>,
        ) -> ::rigetti_pyo3::pyo3::PyResult<PyTestStruct> {
            { Ok(PyTestStruct::from(self.clone())) }
        }
    }
    #[allow(clippy::use_self)]
    impl<'a> ::rigetti_pyo3::ToPython<PyTestStruct> for &'a TestStruct {
        fn to_python(
            &self,
            py: ::rigetti_pyo3::pyo3::Python<'_>,
        ) -> ::rigetti_pyo3::pyo3::PyResult<PyTestStruct> {
            {
                <TestStruct as ::rigetti_pyo3::ToPython<
                    PyTestStruct,
                >>::to_python(*self, py)
            }
        }
    }
    #[allow(clippy::use_self)]
    impl ::rigetti_pyo3::ToPython<::rigetti_pyo3::pyo3::Py<::rigetti_pyo3::pyo3::PyAny>>
    for PyTestStruct {
        fn to_python(
            &self,
            py: ::rigetti_pyo3::pyo3::Python<'_>,
        ) -> ::rigetti_pyo3::pyo3::PyResult<
            ::rigetti_pyo3::pyo3::Py<::rigetti_pyo3::pyo3::PyAny>,
        > {
            { Ok(<Self as ::rigetti_pyo3::pyo3::ToPyObject>::to_object(self, py)) }
        }
    }
    #[allow(clippy::use_self)]
    impl<
        'a,
    > ::rigetti_pyo3::ToPython<::rigetti_pyo3::pyo3::Py<::rigetti_pyo3::pyo3::PyAny>>
    for &'a PyTestStruct {
        fn to_python(
            &self,
            py: ::rigetti_pyo3::pyo3::Python<'_>,
        ) -> ::rigetti_pyo3::pyo3::PyResult<
            ::rigetti_pyo3::pyo3::Py<::rigetti_pyo3::pyo3::PyAny>,
        > {
            {
                <PyTestStruct as ::rigetti_pyo3::ToPython<
                    ::rigetti_pyo3::pyo3::Py<::rigetti_pyo3::pyo3::PyAny>,
                >>::to_python(*self, py)
            }
        }
    }
    impl From<PyTestStruct> for TestStruct {
        fn from(wrapper: PyTestStruct) -> Self {
            wrapper.0
        }
    }
    impl From<TestStruct> for PyTestStruct {
        fn from(inner: TestStruct) -> Self {
            Self(inner)
        }
    }
    impl From<&TestStruct> for PyTestStruct {
        fn from(inner: &TestStruct) -> Self {
            Self(inner.clone())
        }
    }
    impl AsRef<TestStruct> for PyTestStruct {
        fn as_ref(&self) -> &TestStruct {
            &self.0
        }
    }
    impl ::rigetti_pyo3::PyWrapper for PyTestStruct {
        type Inner = TestStruct;
    }
    impl ::rigetti_pyo3::pyo3::conversion::ToPyObject for PyTestStruct {
        fn to_object(
            &self,
            py: ::rigetti_pyo3::pyo3::Python,
        ) -> ::rigetti_pyo3::pyo3::PyObject {
            #[allow(clippy::use_self)]
            const NAME: &'static str = "PyTestStruct";
            let cell = ::rigetti_pyo3::pyo3::PyCell::new(py, self.clone())
                .unwrap_or_else(|err| {
                    {
                        ::core::panicking::panic_fmt(
                            format_args!(
                                "failed to create {0} on Python heap: {1}", NAME, err,
                            ),
                        );
                    }
                });
            ::rigetti_pyo3::pyo3::conversion::ToPyObject::to_object(&cell, py)
        }
    }
    impl AsMut<<PyTestStruct as ::rigetti_pyo3::PyWrapper>::Inner> for PyTestStruct {
        fn as_mut(&mut self) -> &mut <PyTestStruct as ::rigetti_pyo3::PyWrapper>::Inner {
            &mut self.0
        }
    }
    impl PyTestStruct {
        ///Get the test_enum_unaliased field from Python.  Annotated with `@property`.
        fn get_test_enum_unaliased(
            &self,
            py: ::rigetti_pyo3::pyo3::Python<'_>,
        ) -> ::rigetti_pyo3::pyo3::PyResult<PyTestEnumUnaliased> {
            use ::rigetti_pyo3::{PyWrapper, ToPython};
            let inner = &self.as_inner().test_enum_unaliased;
            {
                let inner: PyTestEnumUnaliased = ::rigetti_pyo3::ToPython::<
                    PyTestEnumUnaliased,
                >::to_python(&inner, py)?;
                Ok::<_, ::rigetti_pyo3::pyo3::PyErr>(inner)
            }
        }
        ///Set the test_enum_unaliased field from Python.  Annotated with `@test_enum_unaliased.setter`.
        fn set_test_enum_unaliased(
            &mut self,
            py: ::rigetti_pyo3::pyo3::Python<'_>,
            from: PyTestEnumUnaliased,
        ) -> ::rigetti_pyo3::pyo3::PyResult<()> {
            use ::rigetti_pyo3::{PyTryFrom, PyWrapperMut};
            let from = &from;
            let new_val: TestEnum = {
                <_ as ::rigetti_pyo3::PyTryFrom<
                    PyTestEnumUnaliased,
                >>::py_try_from(py, from)
            }?;
            self.as_inner_mut().test_enum_unaliased = new_val;
            Ok(())
        }
        ///Get the test_enum_aliased field from Python.  Annotated with `@property`.
        fn get_test_enum_aliased(
            &self,
            py: ::rigetti_pyo3::pyo3::Python<'_>,
        ) -> ::rigetti_pyo3::pyo3::PyResult<PyTestEnumAliased> {
            use ::rigetti_pyo3::{PyWrapper, ToPython};
            let inner = &self.as_inner().test_enum_aliased;
            {
                let inner: PyTestEnumAliased = ::rigetti_pyo3::ToPython::<
                    PyTestEnumAliased,
                >::to_python(&inner, py)?;
                Ok::<_, ::rigetti_pyo3::pyo3::PyErr>(inner)
            }
        }
        ///Set the test_enum_aliased field from Python.  Annotated with `@test_enum_aliased.setter`.
        fn set_test_enum_aliased(
            &mut self,
            py: ::rigetti_pyo3::pyo3::Python<'_>,
            from: PyTestEnumAliased,
        ) -> ::rigetti_pyo3::pyo3::PyResult<()> {
            use ::rigetti_pyo3::{PyTryFrom, PyWrapperMut};
            let from = &from;
            let new_val: TestEnum = {
                <_ as ::rigetti_pyo3::PyTryFrom<
                    PyTestEnumAliased,
                >>::py_try_from(py, from)
            }?;
            self.as_inner_mut().test_enum_aliased = new_val;
            Ok(())
        }
    }
    const _: () = {
        use ::pyo3 as _pyo3;
        #[allow(non_upper_case_globals)]
        const _: () = {
            static __INVENTORY: ::inventory::Node = ::inventory::Node {
                value: &{
                    type Inventory = <PyTestStruct as _pyo3::impl_::pyclass::PyClassImpl>::Inventory;
                    Inventory::new(_pyo3::impl_::pyclass::PyClassItems {
                        methods: &[
                            _pyo3::class::PyMethodDefType::Getter(
                                _pyo3::class::PyGetterDef::new(
                                    "test_enum_unaliased\0",
                                    _pyo3::impl_::pymethods::PyGetter(
                                        PyTestStruct::__pymethod_get_get_test_enum_unaliased__,
                                    ),
                                    "Get the test_enum_unaliased field from Python.  Annotated with `@property`.\u{0}",
                                ),
                            ),
                            _pyo3::class::PyMethodDefType::Setter(
                                _pyo3::class::PySetterDef::new(
                                    "test_enum_unaliased\0",
                                    _pyo3::impl_::pymethods::PySetter(
                                        PyTestStruct::__pymethod_set_set_test_enum_unaliased__,
                                    ),
                                    "Set the test_enum_unaliased field from Python.  Annotated with `@test_enum_unaliased.setter`.\u{0}",
                                ),
                            ),
                            _pyo3::class::PyMethodDefType::Getter(
                                _pyo3::class::PyGetterDef::new(
                                    "test_enum_aliased\0",
                                    _pyo3::impl_::pymethods::PyGetter(
                                        PyTestStruct::__pymethod_get_get_test_enum_aliased__,
                                    ),
                                    "Get the test_enum_aliased field from Python.  Annotated with `@property`.\u{0}",
                                ),
                            ),
                            _pyo3::class::PyMethodDefType::Setter(
                                _pyo3::class::PySetterDef::new(
                                    "test_enum_aliased\0",
                                    _pyo3::impl_::pymethods::PySetter(
                                        PyTestStruct::__pymethod_set_set_test_enum_aliased__,
                                    ),
                                    "Set the test_enum_aliased field from Python.  Annotated with `@test_enum_aliased.setter`.\u{0}",
                                ),
                            ),
                        ],
                        slots: &[],
                    })
                },
                next: ::inventory::core::cell::UnsafeCell::new(
                    ::inventory::core::option::Option::None,
                ),
            };
            #[link_section = ".text.startup"]
            unsafe extern "C" fn __ctor() {
                unsafe {
                    ::inventory::ErasedNode::submit(__INVENTORY.value, &__INVENTORY)
                }
            }
            #[used]
            #[link_section = ".init_array"]
            static __CTOR: unsafe extern "C" fn() = __ctor;
        };
        #[doc(hidden)]
        #[allow(non_snake_case)]
        impl PyTestStruct {
            unsafe fn __pymethod_get_get_test_enum_unaliased__(
                py: _pyo3::Python<'_>,
                _slf: *mut _pyo3::ffi::PyObject,
            ) -> _pyo3::PyResult<*mut _pyo3::ffi::PyObject> {
                _pyo3::callback::convert(
                    py,
                    PyTestStruct::get_test_enum_unaliased(
                        _pyo3::impl_::extract_argument::extract_pyclass_ref::<
                            PyTestStruct,
                        >(
                            py.from_borrowed_ptr::<_pyo3::PyAny>(_slf),
                            &mut {
                                _pyo3::impl_::extract_argument::FunctionArgumentHolder::INIT
                            },
                        )?,
                        py,
                    ),
                )
            }
            unsafe fn __pymethod_set_set_test_enum_unaliased__(
                py: _pyo3::Python<'_>,
                _slf: *mut _pyo3::ffi::PyObject,
                _value: *mut _pyo3::ffi::PyObject,
            ) -> _pyo3::PyResult<::std::os::raw::c_int> {
                let _value = py
                    .from_borrowed_ptr_or_opt(_value)
                    .ok_or_else(|| {
                        _pyo3::exceptions::PyAttributeError::new_err(
                            "can't delete attribute",
                        )
                    })?;
                let _val = _pyo3::FromPyObject::extract(_value)?;
                _pyo3::callback::convert(
                    py,
                    PyTestStruct::set_test_enum_unaliased(
                        _pyo3::impl_::extract_argument::extract_pyclass_ref_mut::<
                            PyTestStruct,
                        >(
                            py.from_borrowed_ptr::<_pyo3::PyAny>(_slf),
                            &mut {
                                _pyo3::impl_::extract_argument::FunctionArgumentHolder::INIT
                            },
                        )?,
                        py,
                        _val,
                    ),
                )
            }
            unsafe fn __pymethod_get_get_test_enum_aliased__(
                py: _pyo3::Python<'_>,
                _slf: *mut _pyo3::ffi::PyObject,
            ) -> _pyo3::PyResult<*mut _pyo3::ffi::PyObject> {
                _pyo3::callback::convert(
                    py,
                    PyTestStruct::get_test_enum_aliased(
                        _pyo3::impl_::extract_argument::extract_pyclass_ref::<
                            PyTestStruct,
                        >(
                            py.from_borrowed_ptr::<_pyo3::PyAny>(_slf),
                            &mut {
                                _pyo3::impl_::extract_argument::FunctionArgumentHolder::INIT
                            },
                        )?,
                        py,
                    ),
                )
            }
            unsafe fn __pymethod_set_set_test_enum_aliased__(
                py: _pyo3::Python<'_>,
                _slf: *mut _pyo3::ffi::PyObject,
                _value: *mut _pyo3::ffi::PyObject,
            ) -> _pyo3::PyResult<::std::os::raw::c_int> {
                let _value = py
                    .from_borrowed_ptr_or_opt(_value)
                    .ok_or_else(|| {
                        _pyo3::exceptions::PyAttributeError::new_err(
                            "can't delete attribute",
                        )
                    })?;
                let _val = _pyo3::FromPyObject::extract(_value)?;
                _pyo3::callback::convert(
                    py,
                    PyTestStruct::set_test_enum_aliased(
                        _pyo3::impl_::extract_argument::extract_pyclass_ref_mut::<
                            PyTestStruct,
                        >(
                            py.from_borrowed_ptr::<_pyo3::PyAny>(_slf),
                            &mut {
                                _pyo3::impl_::extract_argument::FunctionArgumentHolder::INIT
                            },
                        )?,
                        py,
                        _val,
                    ),
                )
            }
        }
    };
    impl PyTestStruct {
        fn __new__() -> Self {
            Self(TestStruct {
                test_enum_unaliased: TestEnum::One,
                test_enum_aliased: TestEnum::One,
            })
        }
    }
    const _: () = {
        use ::pyo3 as _pyo3;
        #[allow(non_upper_case_globals)]
        const _: () = {
            static __INVENTORY: ::inventory::Node = ::inventory::Node {
                value: &{
                    type Inventory = <PyTestStruct as _pyo3::impl_::pyclass::PyClassImpl>::Inventory;
                    Inventory::new(_pyo3::impl_::pyclass::PyClassItems {
                        methods: &[],
                        slots: &[
                            _pyo3::ffi::PyType_Slot {
                                slot: _pyo3::ffi::Py_tp_new,
                                pfunc: {
                                    unsafe extern "C" fn trampoline(
                                        subtype: *mut _pyo3::ffi::PyTypeObject,
                                        args: *mut _pyo3::ffi::PyObject,
                                        kwargs: *mut _pyo3::ffi::PyObject,
                                    ) -> *mut _pyo3::ffi::PyObject {
                                        use _pyo3::impl_::pyclass::*;
                                        impl PyClassNewTextSignature<PyTestStruct>
                                        for PyClassImplCollector<PyTestStruct> {
                                            #[inline]
                                            fn new_text_signature(
                                                self,
                                            ) -> ::std::option::Option<&'static str> {
                                                ::std::option::Option::Some("()")
                                            }
                                        }
                                        _pyo3::impl_::trampoline::newfunc(
                                            subtype,
                                            args,
                                            kwargs,
                                            PyTestStruct::__pymethod___new____,
                                        )
                                    }
                                    trampoline
                                } as _pyo3::ffi::newfunc as _,
                            },
                        ],
                    })
                },
                next: ::inventory::core::cell::UnsafeCell::new(
                    ::inventory::core::option::Option::None,
                ),
            };
            #[link_section = ".text.startup"]
            unsafe extern "C" fn __ctor() {
                unsafe {
                    ::inventory::ErasedNode::submit(__INVENTORY.value, &__INVENTORY)
                }
            }
            #[used]
            #[link_section = ".init_array"]
            static __CTOR: unsafe extern "C" fn() = __ctor;
        };
        #[doc(hidden)]
        #[allow(non_snake_case)]
        impl PyTestStruct {
            unsafe fn __pymethod___new____(
                py: _pyo3::Python<'_>,
                _slf: *mut _pyo3::ffi::PyTypeObject,
                _args: *mut _pyo3::ffi::PyObject,
                _kwargs: *mut _pyo3::ffi::PyObject,
            ) -> _pyo3::PyResult<*mut _pyo3::ffi::PyObject> {
                use _pyo3::callback::IntoPyCallbackOutput;
                let function = PyTestStruct::__new__;
                const DESCRIPTION: _pyo3::impl_::extract_argument::FunctionDescription = _pyo3::impl_::extract_argument::FunctionDescription {
                    cls_name: ::std::option::Option::Some(
                        <PyTestStruct as _pyo3::type_object::PyTypeInfo>::NAME,
                    ),
                    func_name: "__new__",
                    positional_parameter_names: &[],
                    positional_only_parameters: 0usize,
                    required_positional_parameters: 0usize,
                    keyword_only_parameters: &[],
                };
                let mut output = [::std::option::Option::None; 0usize];
                let (_args, _kwargs) = DESCRIPTION
                    .extract_arguments_tuple_dict::<
                        _pyo3::impl_::extract_argument::NoVarargs,
                        _pyo3::impl_::extract_argument::NoVarkeywords,
                    >(py, _args, _kwargs, &mut output)?;
                let result = PyTestStruct::__new__();
                let initializer: _pyo3::PyClassInitializer<PyTestStruct> = result
                    .convert(py)?;
                let cell = initializer.create_cell_from_subtype(py, _slf)?;
                ::std::result::Result::Ok(cell as *mut _pyo3::ffi::PyObject)
            }
        }
    };
}
fn wrapper_tests(py: Python<'_>, m: &PyModule) -> PyResult<()> {
    python::init_submodule("wrapper_tests", py, m)
}
#[doc(hidden)]
mod wrapper_tests {
    pub(crate) struct MakeDef;
    pub static DEF: ::pyo3::impl_::pymodule::ModuleDef = MakeDef::make_def();
    pub const NAME: &'static str = "wrapper_tests\u{0}";
    /// This autogenerated function is called by the python interpreter when importing
    /// the module.
    #[export_name = "PyInit_wrapper_tests"]
    pub unsafe extern "C" fn init() -> *mut ::pyo3::ffi::PyObject {
        ::pyo3::impl_::trampoline::module_init(|py| DEF.make_module(py))
    }
}
const _: () = {
    use ::pyo3::impl_::pymodule as impl_;
    impl wrapper_tests::MakeDef {
        const fn make_def() -> impl_::ModuleDef {
            const INITIALIZER: impl_::ModuleInitializer = impl_::ModuleInitializer(
                wrapper_tests,
            );
            unsafe { impl_::ModuleDef::new(wrapper_tests::NAME, "\0", INITIALIZER) }
        }
    }
};
pub fn append_to_inittab() {
    unsafe {
        if ::pyo3::ffi::Py_IsInitialized() != 0 {
            {
                ::core::panicking::panic_fmt(
                    format_args!(
                        "called `append_to_inittab` but a Python interpreter is already running.",
                    ),
                );
            };
        }
        ::pyo3::ffi::PyImport_AppendInittab(
            wrapper_tests::NAME.as_ptr() as *const ::std::os::raw::c_char,
            ::std::option::Option::Some(wrapper_tests::init),
        );
    };
}
