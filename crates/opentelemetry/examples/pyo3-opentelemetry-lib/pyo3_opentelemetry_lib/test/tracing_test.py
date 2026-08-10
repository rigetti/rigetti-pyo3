from __future__ import annotations

import json
import os
from collections import Counter
from time import time
from typing import TYPE_CHECKING, Any, Callable, Dict, List, MutableMapping

import pytest
from opentelemetry import propagate
from opentelemetry.context import attach, detach
from opentelemetry.proto.trace.v1.trace_pb2 import ResourceSpans
from opentelemetry.trace import Tracer
from opentelemetry.trace.propagation import get_current_span

import pyo3_opentelemetry_lib
from pyo3_opentelemetry_lib._tracing_subscriber import (
    BatchConfig,
    CurrentThreadTracingConfig,
    GlobalTracingConfig,
    SimpleConfig,
    Tracing,
    subscriber,
)
from pyo3_opentelemetry_lib._tracing_subscriber.layers import otel_otlp as otlp
from pyo3_opentelemetry_lib._tracing_subscriber.layers import otel_otlp_file as file

if TYPE_CHECKING:
    from pyo3_opentelemetry_lib._tracing_subscriber import TracingConfig


_TEST_ARTIFACTS_DIR = os.path.join(os.path.dirname(__file__), "__artifacts__")


def global_tracing(param: Any):
    """
    Do not run these tests unless the `global_tracing_configuration` option is set (see conftest.py).
    It is necessary to run each test with global tracing configuration separately because
    `GlobalTracingConfig` can only be initialized once per process.

    Alternative solutions such as `pytest-forked <https://github.com/pytest-dev/pytest-forked>`_
    did not work with the `otel_service_data` fixture.
    """
    return pytest.param(param, marks=pytest.mark.global_tracing_configuration)


_TEST_FILE_EXPORT = [
    lambda filename: CurrentThreadTracingConfig(
        export_process=SimpleConfig(
            subscriber=subscriber.Config(layer=file.Config(file_path=os.path.join(_TEST_ARTIFACTS_DIR, filename)))
        )
    ),
    global_tracing(
        lambda filename: GlobalTracingConfig(
            export_process=SimpleConfig(
                subscriber=subscriber.Config(layer=file.Config(file_path=os.path.join(_TEST_ARTIFACTS_DIR, filename)))
            )
        )
    ),
]


@pytest.mark.parametrize(
    "config_builder", _TEST_FILE_EXPORT, ids=[str(i).zfill(2) for i in range(len(_TEST_FILE_EXPORT))]
)
async def test_file_export(config_builder: Callable[[str], TracingConfig], tracer: Tracer, file_export_filter: None):
    """
    Test that OTLP spans are accurately exported to a file.
    """
    await _test_file_export(config_builder, tracer)


_TEST_FILE_EXPORT_MULTI_THREADS = [
    lambda filename: CurrentThreadTracingConfig(
        export_process=SimpleConfig(
            subscriber=subscriber.Config(layer=file.Config(file_path=os.path.join(_TEST_ARTIFACTS_DIR, filename)))
        )
    ),
]


@pytest.mark.parametrize(
    "config_builder",
    _TEST_FILE_EXPORT_MULTI_THREADS,
    ids=[str(i).zfill(2) for i in range(len(_TEST_FILE_EXPORT_MULTI_THREADS))],
)
async def test_file_export_multi_threads(
    config_builder: Callable[[str], TracingConfig], tracer: Tracer, file_export_filter: None
):
    """
    Test that `CurrentThreadTracingConfig` can be initialized and used multiple times within the
    same process.
    """
    for _ in range(3):
        await _test_file_export(config_builder, tracer)


async def _test_file_export(config_builder: Callable[[str], TracingConfig], tracer: Tracer):
    """
    Implements a single test for file export.
    """
    filename = f"test_file_export-{time()}.txt"
    config = config_builder(filename)
    with Tracing(config=config):
        with tracer.start_as_current_span("test_file_export_tracing"):
            current_span = get_current_span()
            span_context = current_span.get_span_context()
            assert span_context.is_valid
            trace_id = span_context.trace_id
            assert trace_id != 0
            # This function is implemented and instrumented in `examples/pyo3-opentelemetry-lib/src/lib.rs`.
            result = pyo3_opentelemetry_lib.example_function()

    _assert_propagated_trace_id_eq(result, trace_id)

    # Read the OTLP spans written to file.
    file_path = os.path.join(_TEST_ARTIFACTS_DIR, filename)
    with open(file_path, "r") as f:
        resource_spans: List[Dict[str, Any]] = []
        for line in f.readlines():
            datum = json.loads(line)
            resource_spans += datum["resourceSpans"]

    counter: Counter[str] = Counter()
    for resource_span in resource_spans:
        for scoped_span in resource_span["scopeSpans"]:
            for span in scoped_span["spans"]:
                span_trace_id = int(span["traceId"], 16)
                assert span_trace_id is None or span_trace_id == trace_id, filename
                counter[span["name"]] += 1
    # Assert that only the spans we expect are present. This makes use of the Rust `EnvFilter`,
    # which we configure in the `file_export_filter` fixture (ie the `RUST_LOG` environment variable).
    assert len(counter) == 1
    assert counter["example_function_impl"] == 1


_TEST_FILE_EXPORT_ASYNC = [
    global_tracing(
        lambda filename: GlobalTracingConfig(
            export_process=SimpleConfig(
                subscriber=subscriber.Config(layer=file.Config(file_path=os.path.join(_TEST_ARTIFACTS_DIR, filename)))
            )
        )
    ),
]


@pytest.mark.parametrize(
    "config_builder", _TEST_FILE_EXPORT_ASYNC, ids=[str(i).zfill(2) for i in range(len(_TEST_FILE_EXPORT_ASYNC))]
)
async def test_file_export_asynchronous(
    config_builder: Callable[[str], TracingConfig], tracer: Tracer, file_export_filter: None
):
    """
    Test that the `GlobalTracingConfig` supports async spans.
    """
    filename = f"test_file_export_async-{time()}.txt"
    config = config_builder(filename)
    with Tracing(config=config):
        with tracer.start_as_current_span("test_file_export_tracing"):
            current_span = get_current_span()
            span_context = current_span.get_span_context()
            assert span_context.is_valid
            trace_id = span_context.trace_id
            assert trace_id != 0
            # This function is implemented and instrumented in `examples/pyo3-opentelemetry-lib/src/lib.rs`.
            result = await pyo3_opentelemetry_lib.example_function_async()

    _assert_propagated_trace_id_eq(result, trace_id)

    file_path = os.path.join(_TEST_ARTIFACTS_DIR, filename)
    with open(file_path, "r") as f:
        resource_spans: List[Dict[str, Any]] = []
        for line in f.readlines():
            datum = json.loads(line)
            resource_spans += datum["resourceSpans"]

    counter: Counter[str] = Counter()
    for resource_span in resource_spans:
        for scoped_span in resource_span["scopeSpans"]:
            for span in scoped_span["spans"]:
                counter[span["name"]] += 1
                span_trace_id = int(span["traceId"], 16)
                assert span_trace_id is None or span_trace_id == trace_id, filename
                if span["name"] == "example_function_impl_async":
                    duration_ns = int(span["endTimeUnixNano"]) - int(span["startTimeUnixNano"])
                    expected_duration_ms = 100
                    assert duration_ns > (expected_duration_ms * 10**6)
                    assert duration_ns < (1.5 * expected_duration_ms * 10**6)
    # Assert that only the spans we expect are present. This makes use of the Rust `EnvFilter`,
    # which we configure in the `file_export_filter` fixture (ie the `RUST_LOG` environment variable).
    assert len(counter) == 2
    assert counter["example_function_impl"] == 1
    assert counter["example_function_impl_async"] == 1


_TEST_OTLP_EXPORT = [
    CurrentThreadTracingConfig(export_process=SimpleConfig(subscriber=subscriber.Config(layer=otlp.Config()))),
    CurrentThreadTracingConfig(export_process=BatchConfig(subscriber=subscriber.Config(layer=otlp.Config()))),
    global_tracing(GlobalTracingConfig(export_process=SimpleConfig(subscriber=subscriber.Config(layer=otlp.Config())))),
    global_tracing(GlobalTracingConfig(export_process=BatchConfig(subscriber=subscriber.Config(layer=otlp.Config())))),
]


@pytest.mark.parametrize(
    "config",
    _TEST_OTLP_EXPORT,
    ids=[str(i).zfill(2) for i in range(len(_TEST_OTLP_EXPORT))],
)
async def test_otlp_export(
    config: TracingConfig,
    tracer: Tracer,
    otlp_test_namespace: str,
    otlp_service_data: MutableMapping[str, List[ResourceSpans]],
):
    """
    Test that the `otlp.Config` can be used to export spans to an OTLP collector. Here, we use a mock
    gRPC service (see `otlp_service_data` fixture) to collect spans and make assertions on them.
    """
    with Tracing(config=config):
        with tracer.start_as_current_span("test_file_export_tracing"):
            current_span = get_current_span()
            span_context = current_span.get_span_context()
            assert span_context.is_valid
            trace_id = span_context.trace_id
            assert trace_id != 0
            # This function is implemented and instrumented in `examples/pyo3-opentelemetry-lib/src/lib.rs`.
            result = pyo3_opentelemetry_lib.example_function()

    _assert_propagated_trace_id_eq(result, trace_id)

    counter: Counter[str] = Counter()
    data = otlp_service_data.get(otlp_test_namespace, None)
    assert data is not None
    for resource_span in data:
        for scope_span in resource_span.scope_spans:
            for span in scope_span.spans:
                assert int.from_bytes(span.trace_id, "big") == trace_id, trace_id
                counter[span.name] += 1
    # Assert that only the spans we expect are present. This makes use of the Rust `EnvFilter`,
    # which we configure in the `otel_service_data` fixture (ie the `RUST_LOG` environment variable).
    assert len(counter) == 1
    assert counter["example_function_impl"] == 1


_TEST_OTLP_EXPORT_MULTI_THREADS = [
    CurrentThreadTracingConfig(export_process=SimpleConfig(subscriber=subscriber.Config(layer=otlp.Config()))),
    CurrentThreadTracingConfig(export_process=BatchConfig(subscriber=subscriber.Config(layer=otlp.Config()))),
]


@pytest.mark.parametrize(
    "config",
    _TEST_OTLP_EXPORT_MULTI_THREADS,
    ids=[str(i).zfill(2) for i in range(len(_TEST_OTLP_EXPORT_MULTI_THREADS))],
)
async def test_otlp_export_multi_threads(
    config: TracingConfig,
    tracer: Tracer,
    otlp_test_namespace: str,
    otlp_service_data: MutableMapping[str, List[ResourceSpans]],
):
    """
    Test that `CurrentThreadTracingConfig` can be used to export spans to an OTLP collector multiple times
    within the same process.
    """
    for _ in range(3):
        with Tracing(config=config):
            with tracer.start_as_current_span("test_file_export_tracing"):
                current_span = get_current_span()
                span_context = current_span.get_span_context()
                assert span_context.is_valid
                trace_id = span_context.trace_id
                assert trace_id != 0
                # This function is implemented and instrumented in `examples/pyo3-opentelemetry-lib/src/lib.rs`.
                result = pyo3_opentelemetry_lib.example_function()

        _assert_propagated_trace_id_eq(result, trace_id)

        data = otlp_service_data.get(otlp_test_namespace, None)
        assert data is not None
        counter: Counter[str] = Counter()
        for resource_span in data:
            for scope_span in resource_span.scope_spans:
                for span in scope_span.spans:
                    if int.from_bytes(span.trace_id, "big") == trace_id:
                        counter[span.name] += 1
        # Assert that only the spans we expect are present. This makes use of the Rust `EnvFilter`,
        # which we configure in the `otel_service_data` fixture (ie the `RUST_LOG` environment variable).
        assert len(counter) == 1
        assert counter["example_function_impl"] == 1


TEST_OTLP_EXPORT_ASYNC = [
    global_tracing(GlobalTracingConfig(export_process=SimpleConfig(subscriber=subscriber.Config(layer=otlp.Config())))),
    global_tracing(GlobalTracingConfig(export_process=BatchConfig(subscriber=subscriber.Config(layer=otlp.Config())))),
]


@pytest.mark.parametrize(
    "config",
    TEST_OTLP_EXPORT_ASYNC,
    ids=[str(i).zfill(2) for i in range(len(TEST_OTLP_EXPORT_ASYNC))],
)
async def test_otlp_export_asynchronous(
    config: TracingConfig,
    tracer: Tracer,
    otlp_test_namespace: str,
    otlp_service_data: MutableMapping[str, List[ResourceSpans]],
):
    """
    Test that the `GlobalTracingConfig` supports async spans when using the OTLP layer.
    """
    with Tracing(config=config):
        with tracer.start_as_current_span("test_file_export_tracing"):
            current_span = get_current_span()
            span_context = current_span.get_span_context()
            assert span_context.is_valid
            trace_id = span_context.trace_id
            assert trace_id != 0
            # This function is implemented and instrumented in `examples/pyo3-opentelemetry-lib/src/lib.rs`.
            result = await pyo3_opentelemetry_lib.example_function_async()

    _assert_propagated_trace_id_eq(result, trace_id)

    data = otlp_service_data.get(otlp_test_namespace, None)
    assert data is not None
    counter: Counter[str] = Counter()
    for resource_span in data:
        for scope_span in resource_span.scope_spans:
            for span in scope_span.spans:
                counter[span.name] += 1
                assert int.from_bytes(span.trace_id, "big") == trace_id, trace_id
                if span.name == "example_function_impl_async":
                    duration_ns = span.end_time_unix_nano - span.start_time_unix_nano
                    expected_duration_ms = 100
                    assert duration_ns > (expected_duration_ms * 10**6)
                    assert duration_ns < (1.5 * expected_duration_ms * 10**6)
    # Assert that only the spans we expect are present. This makes use of the Rust `EnvFilter`,
    # which we configure in the `otel_service_data` fixture (ie the `RUST_LOG` environment variable).
    assert len(counter) == 2
    assert counter["example_function_impl"] == 1
    assert counter["example_function_impl_async"] == 1


def _assert_propagated_trace_id_eq(carrier: Dict[str, str], trace_id: int):
    """
    The rust code is configured to return a hash map of the current span context. Here we
    parse that map and assert that the trace id is the same as the one we initialized on the
    Python side.
    """
    new_context = propagate.extract(carrier=carrier)
    token = attach(new_context)
    assert get_current_span().get_span_context().trace_id == trace_id
    detach(token)
