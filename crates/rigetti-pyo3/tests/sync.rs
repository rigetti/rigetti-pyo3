//! Regression tests for synchronous calls into the async runtime.

#![cfg(feature = "async-tokio")]

use pyo3::{ffi::c_str, prelude::*, types::PyDict, wrap_pyfunction};
use rigetti_pyo3::sync::invoke_async_from_py_sync;
use std::{
    process::Command,
    thread,
    time::{Duration, Instant},
};

#[pyfunction]
fn sync_value(py: Python<'_>, value: usize) -> PyResult<usize> {
    invoke_async_from_py_sync(py, async move { Ok(value) })
}

#[test]
fn concurrent_cold_start() {
    if std::env::var_os("RIGETTI_PYO3_SYNC_COLD_START").is_none() {
        // A fresh process keeps the loop uninitialized and bounds a possible GIL deadlock.
        let mut child = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "concurrent_cold_start", "--nocapture"])
            .env("RIGETTI_PYO3_SYNC_COLD_START", "1")
            .spawn()
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(30);
        loop {
            if let Some(status) = child.try_wait().unwrap() {
                assert!(status.success(), "cold-start subprocess failed: {status}");
                return;
            }
            if Instant::now() >= deadline {
                child.kill().unwrap();
                child.wait().unwrap();
                panic!("concurrent cold-start calls deadlocked");
            }
            thread::sleep(Duration::from_millis(10));
        }
    }

    Python::initialize();
    Python::attach(|py| {
        let globals = PyDict::new(py);
        globals
            .set_item("sync_value", wrap_pyfunction!(sync_value, py).unwrap())
            .unwrap();
        py.run(
            c_str!(
                r"
import sys
import threading
import time
from concurrent.futures import ThreadPoolExecutor

assert 'asyncio' not in sys.modules

class SlowAsyncioImport:
    def find_spec(self, fullname, path, target=None):
        if fullname == 'asyncio':
            # Release the GIL during loop initialization so other callers contend on the lock.
            time.sleep(0.1)
        return None

finder = SlowAsyncioImport()
sys.meta_path.insert(0, finder)
try:
    barrier = threading.Barrier(6)

    def call(value):
        barrier.wait()
        return sync_value(value)

    with ThreadPoolExecutor(max_workers=6) as pool:
        assert list(pool.map(call, range(6))) == list(range(6))

    # Subsequent calls reuse the initialized loop.
    assert sync_value(42) == 42
    assert sum(t.name == 'rigetti-pyo3-worker-loop' for t in threading.enumerate()) == 1
finally:
    sys.meta_path.remove(finder)
"
            ),
            Some(&globals),
            None,
        )
        .unwrap();
    });
}
