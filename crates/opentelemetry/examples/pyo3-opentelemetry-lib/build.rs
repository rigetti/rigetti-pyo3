use pyo3_tracing_subscriber_build::write_stub_files;

fn main() {
    pyo3_build_config::add_extension_module_link_args();
    let target_dir = std::path::Path::new("./pyo3_opentelemetry_lib/_tracing_subscriber");
    std::fs::remove_dir_all(target_dir).unwrap();
    write_stub_files("pyo3_opentelemetry_lib", "_tracing_subscriber", target_dir).unwrap();
}
