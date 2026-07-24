use std::borrow::Cow;
use std::collections::HashMap;

use pyo3::prelude::*;
use qcs_dependencies_client::opentelemetry::InstrumentationScope;
use rigetti_pyo3::create_init_submodule;

#[pyclass(name = "InstrumentationLibrary")]
#[derive(Debug, Clone)]
pub(crate) struct PyInstrumentationLibrary {
    name: String,
    version: Option<String>,
    schema_url: Option<String>,
    attributes: HashMap<String, String>,
}

#[pymethods]
impl PyInstrumentationLibrary {
    #[new]
    #[pyo3(signature = (name, /, version=None, schema_url=None, attributes=None))]
    fn new(
        name: String,
        version: Option<String>,
        schema_url: Option<String>,
        attributes: Option<HashMap<String, String>>,
    ) -> Self {
        let attributes = attributes.unwrap_or_default();
        Self {
            name,
            version,
            schema_url,
            attributes,
        }
    }
}

impl From<PyInstrumentationLibrary> for InstrumentationScope {
    fn from(py_instrumentation_library: PyInstrumentationLibrary) -> Self {
        let mut builder = Self::builder(Cow::from(py_instrumentation_library.name));
        if let Some(version) = py_instrumentation_library.version {
            builder = builder.with_version(Cow::from(version));
        }
        if let Some(schema_url) = py_instrumentation_library.schema_url {
            builder = builder.with_schema_url(Cow::from(schema_url));
        }
        let mut attributes = Vec::new();
        for (key, value) in py_instrumentation_library.attributes {
            let kv = qcs_dependencies_client::opentelemetry::KeyValue::new(
                qcs_dependencies_client::opentelemetry::Key::new(key),
                qcs_dependencies_client::opentelemetry::Value::from(value),
            );
            attributes.push(kv);
        }
        builder = builder.with_attributes(attributes);

        builder.build()
    }
}

create_init_submodule! {
    classes: [ PyInstrumentationLibrary ],
}
