#!/usr/bin/env python3
"""Validate the public RSDss documentation catalogs against the dispatch source."""

import json
import re
import sys
from pathlib import Path
from typing import Dict, Set


ROOT = Path(__file__).resolve().parents[4]
DOCS = ROOT / "sparrowzz/dss/docs"
FUNCTIONS = DOCS / "catalog/functions.json"
DATAFRAMES = DOCS / "catalog/dataframes.json"
TRANSLATE = ROOT / "mems/src/ml/expr_df/translate.rs"
FRAME_OPS = ROOT / "mems/src/ml/expr_df/frame_ops.rs"

RESULT_CONSTANTS = {
    "MOTOR_DYNAMIC_RESULT_NAMES": {"ds_motor_dynamics"},
    "PROTECTION_COORDINATION_RESULT_NAMES": {"ds_protection_coordination"},
    "DSS_PLAN_RESULT_NAMES": {
        "ds_radial_dss_plan",
        "ds_newton_raphson_dss_plan",
        "ds_radial_dss_plan_url",
        "ds_newton_raphson_dss_plan_url",
    },
    "QSTS_RESULT_NAMES": {"ds_radial_qsts", "ds_newton_raphson_qsts"},
    "MONTE_CARLO_RESULT_NAMES": {"ds_monte_carlo"},
    "RESTORE_RESULT_NAMES": {"ds_restore"},
    "HARMONIC_RESULT_NAMES": {"ds_harmonic_study"},
    "FAULT_RESULT_NAMES": {"ds_fault_study"},
    "POWER_FLOW_RESULT_NAMES": {"ds_radial_pf", "ds_newton_raphson_pf"},
}

QSTS_RESULT_BUILDERS = {
    "storage_state": ("qsts_storage_state_frame", []),
    "storage_controller_state": ("qsts_storage_controller_state_frame", []),
    "switch_control_state": ("qsts_switch_control_state_frame", ["phases", "phase_states"]),
    "fuse_state": ("qsts_fuse_state_frame", ["phases", "phase_states"]),
    "recloser_state": ("qsts_recloser_state_frame", ["phases", "phase_states"]),
    "relay_state": ("qsts_relay_state_frame", ["phases", "phase_states"]),
    "fault_state": ("qsts_fault_state_frame", ["phases"]),
    "execution_trace": ("qsts_execution_trace_frame", []),
}

PSA_RESULT_BUILDERS = {
    "load_state": ("psa/src/dsa/output/power_flow.rs", "load_state_result_df"),
    "branch_flow": ("psa/src/dsa/output/power_flow.rs", "branch_flow_result_df"),
    "source_flow": ("psa/src/dsa/output/power_flow.rs", "source_flow_result_df"),
    "transformer_winding_flow": ("psa/src/dsa/output/power_flow.rs", "transformer_winding_flow_df"),
    "conductor_flow": ("psa/src/dsa/output/line_conductor.rs", "line_conductor_flow_frame"),
    "regulator_state": ("psa/src/dsa/output/device_state.rs", "regulator_state_result_df"),
    "capacitor_state": ("psa/src/dsa/output/device_state.rs", "capacitor_state_result_df"),
    "reactor_state": ("psa/src/dsa/output/device_state.rs", "reactor_state_result_df"),
    "ibr_state": ("psa/src/dsa/output/device_state.rs", "ibr_state_result_df"),
    "gfm_state": ("psa/src/dsa/output/device_state.rs", "gfm_state_result_df"),
    "violations": ("psa/src/dsa/output/power_flow.rs", "violation_result_df"),
    "harmonic_bus_voltage": ("psa/src/dsa/harmonics/api.rs", "bus_voltage_frame"),
    "harmonic_branch_flow": ("psa/src/dsa/harmonics/api.rs", "branch_flow_frame"),
    "harmonic_bus_sequence": ("psa/src/dsa/harmonics/api.rs", "bus_sequence_frame"),
    "harmonic_branch_sequence": ("psa/src/dsa/harmonics/api.rs", "branch_sequence_frame"),
    "harmonic_monitor": ("psa/src/dsa/harmonics/monitor.rs", "monitor_frame"),
}


def fail(message):
    raise AssertionError(message)


def load_json(path):
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        fail(f"cannot read {path.relative_to(ROOT)}: {error}")


def catalog_dsa_entries(functions):
    by_name = {}  # type: Dict[str, dict]
    duplicates = set()  # type: Set[str]
    for entry in functions["dsa_functions"]:
        for name in entry["names"]:
            if name in by_name:
                duplicates.add(name)
            by_name[name] = entry
    return by_name, duplicates


def result_constant(source, name):
    match = re.search(
        rf"const\s+{re.escape(name)}\s*:\s*&\[&str\]\s*=\s*&\[(.*?)\];",
        source,
        re.DOTALL,
    )
    if match is None:
        fail(f"missing Rust result constant {name}")
    return re.findall(r'"([^"]+)"', match.group(1))


def dataframe_builder_columns(source, function_name):
    start = source.find("fn " + function_name + "(")
    if start < 0:
        fail("missing DataFrame builder " + function_name)
    end = source.find("\n#[cfg", start)
    body = source[start : end if end >= 0 else len(source)]
    frame_start = body.find("df![")
    if frame_start < 0:
        fail("missing df! body in " + function_name)
    ends = [
        position
        for position in (
            body.find("\n    .map_err", frame_start),
            body.find("\n    .ok()", frame_start),
        )
        if position >= 0
    ]
    if not ends:
        fail("cannot delimit df! body in " + function_name)
    frame_end = min(ends)
    return re.findall(r'"([^"]+)"\s*=>', body[frame_start:frame_end])


def registered_dsa_names(source):
    start = source.find("pub(super) fn is_dsa_function(")
    if start < 0:
        fail("missing is_dsa_function registry")
    end = source.find('#[cfg(not(feature = "with_dsa"))]', start)
    if end < 0:
        fail("cannot delimit is_dsa_function registry")
    names = set(re.findall(r'"([^"]+)"', source[start:end]))
    return {name for name in names if name.startswith("ds_") or name.startswith("import_")}


def validate_local_links():
    link_pattern = re.compile(r"\[[^\]]+\]\(([^)]+)\)")
    for markdown in DOCS.glob("*.md"):
        for target in link_pattern.findall(markdown.read_text(encoding="utf-8")):
            path_text = target.split("#", 1)[0]
            if not path_text or "://" in path_text or path_text.startswith("mailto:"):
                continue
            target_path = (markdown.parent / path_text).resolve()
            if not target_path.exists():
                fail(f"broken link in {markdown.relative_to(ROOT)}: {target}")


def main():
    functions = load_json(FUNCTIONS)
    dataframes = load_json(DATAFRAMES)
    if functions.get("schema_version") != 2 or dataframes.get("schema_version") != 2:
        fail("both catalogs must use schema_version 2")

    translate = TRANSLATE.read_text(encoding="utf-8")
    frame_ops = FRAME_OPS.read_text(encoding="utf-8")
    registered_dsa = registered_dsa_names(frame_ops)

    dsa_by_name, duplicates = catalog_dsa_entries(functions)
    if duplicates:
        fail(f"duplicate DSA catalog names: {sorted(duplicates)}")
    actual_dsa = set(dsa_by_name)
    if actual_dsa != registered_dsa:
        fail(
            "DSA catalog mismatch: "
            f"missing={sorted(registered_dsa - actual_dsa)}, "
            f"extra={sorted(actual_dsa - registered_dsa)}"
        )

    for name in sorted(registered_dsa):
        quoted = f'"{name}"'
        if quoted not in translate:
            fail(f"{name} is cataloged but missing from translate.rs arity dispatch")
        if quoted not in frame_ops:
            fail(f"{name} is cataloged but missing from frame_ops.rs dispatch")

    for constant, function_names in RESULT_CONSTANTS.items():
        rust_results = result_constant(frame_ops, constant)
        for function_name in function_names:
            catalog_results = dsa_by_name[function_name].get("results")
            if catalog_results != rust_results:
                fail(
                    f"{function_name} result list differs from {constant}: "
                    f"catalog={catalog_results}, rust={rust_results}"
                )

    for result_name, builder in QSTS_RESULT_BUILDERS.items():
        function_name, appended = builder
        rust_columns = dataframe_builder_columns(frame_ops, function_name) + appended
        key = "columns" if result_name == "execution_trace" else "columns_before_time_metadata"
        catalog_columns = dataframes["result_frames"][result_name][key]
        if catalog_columns != rust_columns:
            fail(
                f"{result_name} columns differ from {function_name}: "
                f"catalog={catalog_columns}, rust={rust_columns}"
            )

    for result_name, builder in PSA_RESULT_BUILDERS.items():
        source_path, function_name = builder
        source = (ROOT / source_path).read_text(encoding="utf-8")
        rust_columns = dataframe_builder_columns(source, function_name)
        catalog_columns = dataframes["result_frames"][result_name]["columns"]
        if catalog_columns != rust_columns:
            fail(
                f"{result_name} columns differ from {function_name}: "
                f"catalog={catalog_columns}, rust={rust_columns}"
            )

    network_order = [row["name"] for row in dataframes["network_order"]]
    if network_order != functions["network_argument_order"]:
        fail("network argument order differs between catalogs")
    if [row["position"] for row in dataframes["network_order"]] != list(range(1, 21)):
        fail("network_order positions must be 1..20")
    if [row["position"] for row in dataframes["qsts_extension_order"]] != list(range(21, 34)):
        fail("qsts_extension_order positions must be 21..33")

    for name, contract in dataframes["analysis_inputs"].items():
        for context in contract.get("context", []):
            if context not in registered_dsa:
                fail(f"analysis input {name} references unknown function {context}")

    for group in dataframes["result_frames"].values():
        source = group.get("schema_source")
        if source and not (ROOT / source).exists():
            fail(f"missing schema_source: {source}")

    validate_local_links()
    print(
        f"validated {len(actual_dsa)} DSA functions, "
        f"{len(dataframes['analysis_inputs'])} analysis inputs and "
        f"{len(dataframes['result_frames'])} result contracts"
    )
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except AssertionError as error:
        print(f"catalog validation failed: {error}", file=sys.stderr)
        sys.exit(1)
