##############################################################################
# Copyright 2023 Rigetti Computing
#
#    Licensed under the Apache License, Version 2.0 (the "License");
#    you may not use this file except in compliance with the License.
#    You may obtain a copy of the License at
#
#        http://www.apache.org/licenses/LICENSE-2.0
#
#    Unless required by applicable law or agreed to in writing, software
#    distributed under the License is distributed on an "AS IS" BASIS,
#    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
#    See the License for the specific language governing permissions and
#    limitations under the License.
##############################################################################

import pytest
from opentelemetry import propagate
from opentelemetry.context import attach, detach
from opentelemetry.trace import Tracer
from opentelemetry.trace.propagation import get_current_span

import pyo3_opentelemetry_lib


def test_function_context_propagation(tracer: Tracer) -> None:
    with tracer.start_as_current_span("test_function_context_propagation"):
        current_span = get_current_span()
        trace_id = current_span.get_span_context().trace_id
        result = pyo3_opentelemetry_lib.example_function()

    assert get_current_span().get_span_context().trace_id != trace_id

    new_context = propagate.extract(carrier=result)
    token = attach(new_context)
    assert get_current_span().get_span_context().trace_id == trace_id
    detach(token)


@pytest.mark.asyncio
async def test_async_function_context_propagation(tracer: Tracer) -> None:
    with tracer.start_as_current_span("test_async_function_context_propagation"):
        current_span = get_current_span()
        trace_id = current_span.get_span_context().trace_id
        result = await pyo3_opentelemetry_lib.example_function_async()

    assert get_current_span().get_span_context().trace_id != trace_id

    new_context = propagate.extract(carrier=result)
    token = attach(new_context)
    assert get_current_span().get_span_context().trace_id == trace_id
    detach(token)


def test_example_struct_method_propagation(tracer: Tracer) -> None:
    with tracer.start_as_current_span("test_example_struct_method_propagation"):
        current_span = get_current_span()
        trace_id = current_span.get_span_context().trace_id
        example_struct = pyo3_opentelemetry_lib.ExampleStruct()
        result = example_struct.example_method()

    assert get_current_span().get_span_context().trace_id != trace_id

    new_context = propagate.extract(carrier=result)
    token = attach(new_context)
    assert get_current_span().get_span_context().trace_id == trace_id
    detach(token)


@pytest.mark.asyncio
async def test_async_example_struct_method_propagation(tracer: Tracer) -> None:
    with tracer.start_as_current_span("test_async_example_struct_method_propagation"):
        current_span = get_current_span()
        trace_id = current_span.get_span_context().trace_id
        example_struct = pyo3_opentelemetry_lib.ExampleStruct()
        result = await example_struct.example_method_async()

    assert get_current_span().get_span_context().trace_id != trace_id

    new_context = propagate.extract(carrier=result)
    token = attach(new_context)
    assert get_current_span().get_span_context().trace_id == trace_id
    detach(token)
