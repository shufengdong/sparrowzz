#!/usr/bin/env python3
"""Validate the public RSDss documentation catalogs against the dispatch source."""

import json
import re
import sys
from pathlib import Path
from typing import Any, Dict, Set


ROOT = Path(__file__).resolve().parents[4]
DOCS = ROOT / "sparrowzz/dss/docs"
FUNCTIONS = DOCS / "catalog/functions.json"
DATAFRAMES = DOCS / "catalog/dataframes.json"
STUDY_FACADES = DOCS / "catalog/study_facades.json"
TENSOREVAL = next(ROOT.glob("*/src/bin/tensoreval.rs"))
RUNTIME_SRC = TENSOREVAL.parents[1]
TRANSLATE = RUNTIME_SRC / "ml/expr_df/translate.rs"
FRAME_OPS = RUNTIME_SRC / "ml/expr_df/frame_ops.rs"
NATIVE_DSS = RUNTIME_SRC / "ml/expr_df/frame_ops/dsa/native_dss.rs"

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

NATIVE_DSS_RESULT_CONSTANTS = {
    "HARMONIC_RESULTS": {"ds_dss_harmonic_plan"},
    "AUTO_ADD_RESULTS": {"ds_dss_auto_add"},
    "RELCALC_RESULTS": {"ds_dss_relcalc"},
}

INPUT_KIND_BY_SCHEMA = {
    "frames.para": "struct",
    "frames.solution": "struct",
    "frames.load_shape": "data",
    "frames.shape_assignment": "model",
    "frames.storage_controller": "model",
    "frames.switch_control": "model",
    "frames.tcc_curve": "data",
    "frames.fuse": "model",
    "frames.recloser": "model",
    "frames.relay": "model",
    "frames.monitor": "model",
    "frames.energy_meter": "model",
    "frames.fault": "data",
    "analysis_inputs.state_estimation_measurement": "data",
    "analysis_inputs.monte_carlo_spec": "struct",
    "analysis_inputs.monte_carlo_multipliers": "data",
    "analysis_inputs.protection_fault_case": "data",
    "analysis_inputs.protection_current_sample": "data",
    "analysis_inputs.protection_pair": "data",
    "analysis_inputs.protection_observed_action": "data",
    "analysis_inputs.protection_para": "struct",
    "analysis_inputs.fault_location_measurement": "data",
    "analysis_inputs.fault_location_hypothesis_switch": "data",
    "analysis_inputs.fault_location_hypothesis_line": "data",
    "analysis_inputs.fault_location_para": "struct",
    "analysis_inputs.dynamic_timeline": "data",
    "analysis_inputs.dynamic_event": "data",
    "analysis_inputs.dynamic_spec": "struct",
    "analysis_inputs.harmonic_para": "struct",
    "analysis_inputs.restore_fault": "data",
    "analysis_inputs.dynamic_fault": "data",
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

QSTS_METER_RESULT_BUILDERS = {
    "qsts_monitor": ("psa/src/dsa/pf/metering.rs", "monitor_frame"),
    "qsts_monitor_save": ("psa/src/dsa/pf/metering.rs", "monitor_save_frame"),
    "qsts_energy_meter": ("psa/src/dsa/pf/metering.rs", "meter_frame"),
    "qsts_meter_loss_breakdown": ("psa/src/dsa/pf/metering.rs", "meter_loss_frame"),
    "qsts_reliability": ("psa/src/dsa/pf/metering.rs", "reliability_frame"),
    "qsts_meter_zone": ("psa/src/dsa/pf/metering.rs", "meter_zone_frame"),
    "meter_phase_voltage": ("psa/src/dsa/pf/metering.rs", "meter_phase_voltage_frame"),
    "qsts_meter_save": ("psa/src/dsa/pf/metering.rs", "meter_frame"),
    "qsts_load_duration": ("psa/src/dsa/pf/time_series.rs", "load_duration_frame"),
}

PSA_RESULT_BUILDERS = {
    "load_state": ("psa/src/dsa/output/power_flow.rs", "load_state_result_df"),
    "generator_state": ("psa/src/dsa/output/device_state.rs", "generator_state_result_df"),
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
    "meter_phase_voltage": ("psa/src/dsa/pf/metering.rs", "meter_phase_voltage_frame"),
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


def expanded_arguments(contract, network_order):
    result = []
    for argument in contract.get("arguments", []):
        if argument == "network[20]":
            result.extend(network_order)
        else:
            result.append(argument)
    return result


def dataframe_study_names(dsa_by_name):
    result = set()
    for name, contract in dsa_by_name.items():
        if not name.startswith("ds_"):
            continue
        arguments = contract.get("arguments", [])
        accepts_location = any("path" in argument or "url" in argument for argument in arguments)
        if accepts_location or name.startswith("ds_dss_") or "_dss_plan" in name:
            continue
        result.add(name)
    return result


def schema_at(dataframes, reference):
    value = dataframes  # type: Any
    for component in reference.split("."):
        if not isinstance(value, dict) or component not in value:
            fail(f"unknown schemaRef {reference}")
        value = value[component]
    if not isinstance(value, (dict, list)):
        fail(f"schemaRef {reference} does not identify a schema object")
    return value


def validate_study_facades(functions, dataframes, study_facades, dsa_by_name):
    if study_facades.get("schema_version") != 1:
        fail("study_facades.json must use schema_version 1")
    semantics = study_facades.get("input_kind_semantics")
    if not isinstance(semantics, dict) or set(semantics) != {"model", "data", "struct"}:
        fail("study_facades input_kind_semantics must define model, data and struct")
    facades = study_facades.get("facades")
    if not isinstance(facades, list):
        fail("study_facades.facades must be a list")

    facade_ids = [facade.get("id") for facade in facades]
    facade_functions = [facade.get("function") for facade in facades]
    if any(not isinstance(value, str) or not value for value in facade_ids):
        fail("every study facade must have a non-empty string id")
    if len(facade_ids) != len(set(facade_ids)):
        fail("study facade ids must be unique")
    if len(facade_functions) != len(set(facade_functions)):
        fail("each DataFrame study function must have its own facade")

    expected_functions = dataframe_study_names(dsa_by_name)
    actual_functions = set(facade_functions)
    if actual_functions != expected_functions:
        fail(
            "study facade function mismatch: "
            f"missing={sorted(expected_functions - actual_functions)}, "
            f"extra={sorted(actual_functions - expected_functions)}"
        )

    network_order = functions["network_argument_order"]
    for facade in facades:
        function = facade["function"]
        if function.startswith("import_") or "_dss_plan" in function or function.startswith("ds_dss_"):
            fail(f"path, URL or import facade is forbidden: {function}")
        contract = dsa_by_name.get(function)
        if contract is None:
            fail(f"unknown study facade function {function}")
        if any("path" in argument or "url" in argument for argument in contract.get("arguments", [])):
            fail(f"location-based function entered study facade catalog: {function}")
        for field in ("family", "title", "description"):
            if not isinstance(facade.get(field), str) or not facade[field].strip():
                fail(f"{function} must define a non-empty {field}")
        if not isinstance(facade.get("differences"), list) or not facade["differences"]:
            fail(f"{function} must explain its differences")

        expected_arguments = expanded_arguments(contract, network_order)
        inputs = facade.get("inputs")
        if not isinstance(inputs, list):
            fail(f"{function}.inputs must be a list")
        canonical_arity = contract.get("arity", contract.get("complete_arity"))
        if canonical_arity != len(inputs):
            fail(f"{function} arity differs: catalog={canonical_arity}, facade={len(inputs)}")
        positions = [item.get("position") for item in inputs]
        if positions != list(range(1, len(inputs) + 1)):
            fail(f"{function} input positions must be contiguous from 1")
        aliases = [item.get("alias") for item in inputs]
        if aliases != expected_arguments:
            fail(f"{function} input aliases differ: expected={expected_arguments}, facade={aliases}")
        if "network[20]" in contract.get("arguments", []):
            if aliases[:20] != network_order or "network[20]" in aliases:
                fail(f"{function} must expand network[20] in canonical order")
        input_ids = [item.get("id") for item in inputs]
        if len(input_ids) != len(set(input_ids)) or any(not isinstance(value, str) or not value for value in input_ids):
            fail(f"{function} input ids must be non-empty and unique")
        for item in inputs:
            missing = {"id", "alias", "inputKind", "schemaRef", "required", "position", "description"} - set(item)
            if missing:
                fail(f"{function} input {item.get('id')} lacks fields {sorted(missing)}")
            if item["inputKind"] not in {"model", "data", "struct"}:
                fail(f"{function}.{item['id']} has invalid inputKind {item['inputKind']}")
            expected_kind = INPUT_KIND_BY_SCHEMA.get(item["schemaRef"])
            if item["schemaRef"].startswith("frames.") and item["schemaRef"].split(".", 1)[1] in network_order:
                expected_kind = "model"
            if expected_kind is None:
                fail(f"{function}.{item['id']} has no audited inputKind for {item['schemaRef']}")
            if item["inputKind"] != expected_kind:
                fail(
                    f"{function}.{item['id']} inputKind differs for {item['schemaRef']}: "
                    f"expected={expected_kind}, facade={item['inputKind']}"
                )
            if item["required"] is not True:
                fail(f"{function}.{item['id']} is positional and must set required=true")
            if not isinstance(item["description"], str) or not item["description"].strip():
                fail(f"{function}.{item['id']} must have a description")
            schema_at(dataframes, item["schemaRef"])

        expected_results = contract.get("results", contract.get("returns"))
        results = facade.get("results")
        if not isinstance(results, list):
            fail(f"{function}.results must be a list")
        names = [item.get("name") for item in results]
        if names != expected_results:
            fail(f"{function} results differ: expected={expected_results}, facade={names}")
        result_ids = [item.get("id") for item in results]
        if len(result_ids) != len(set(result_ids)) or any(not isinstance(value, str) or not value for value in result_ids):
            fail(f"{function} result ids must be non-empty and unique")
        recommended = set()
        for item in results:
            missing = {"id", "name", "schemaRef", "description", "recommended"} - set(item)
            if missing:
                fail(f"{function} result {item.get('id')} lacks fields {sorted(missing)}")
            if item["recommended"] is not True and item["recommended"] is not False:
                fail(f"{function}.{item['id']} recommended must be boolean")
            if item["recommended"]:
                recommended.add(item["name"])
            if not isinstance(item["description"], str) or not item["description"].strip():
                fail(f"{function}.{item['id']} must have a description")
            schema_at(dataframes, item["schemaRef"])
        if not recommended:
            fail(f"{function} must recommend at least one result")

        constraints = facade.get("resultConstraints")
        if not isinstance(constraints, list) or not constraints:
            fail(f"{function} must define resultConstraints")
        constrained_default = None
        for constraint in constraints:
            kind = constraint.get("type")
            if kind not in {"ordered_unique_selection", "default_results", "fixed_results", "conditional_unavailable"}:
                fail(f"{function} has unknown result constraint {kind}")
            referenced = constraint.get("results", [])
            if referenced and not set(referenced).issubset(names):
                fail(f"{function} result constraint references unknown results {referenced}")
            if kind in {"default_results", "fixed_results"}:
                if not referenced:
                    fail(f"{function} {kind} must name at least one result")
                constrained_default = set(referenced)
            if kind == "conditional_unavailable":
                if constraint.get("result") not in names:
                    fail(f"{function} conditional constraint references an unknown result")
                condition_input = constraint.get("when", {}).get("input")
                if condition_input not in input_ids:
                    fail(f"{function} conditional constraint references unknown input {condition_input}")
        if constrained_default is None:
            fail(f"{function} must declare default_results or fixed_results")
        if not constrained_default.issubset(recommended):
            fail(f"{function} recommended results do not include its default/fixed result set")

    by_function = {facade["function"]: facade for facade in facades}
    for group in [
        ("ds_radial_pf", "ds_newton_raphson_pf"),
        ("ds_radial_qsts", "ds_newton_raphson_qsts"),
        ("ds_wls_se", "ds_mnmr_se", "ds_mce_se"),
        ("ds_network_dynamics", "ds_motor_dynamics"),
    ]:
        if len({by_function[name]["id"] for name in group}) != len(group):
            fail(f"related algorithms must remain separate facades: {group}")


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
    study_facades = load_json(STUDY_FACADES)
    if functions.get("schema_version") != 2 or dataframes.get("schema_version") != 2:
        fail("both catalogs must use schema_version 2")

    translate = TRANSLATE.read_text(encoding="utf-8")
    frame_ops = FRAME_OPS.read_text(encoding="utf-8")
    native_dss = NATIVE_DSS.read_text(encoding="utf-8")
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

    validate_study_facades(functions, dataframes, study_facades, dsa_by_name)

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

    for constant, function_names in NATIVE_DSS_RESULT_CONSTANTS.items():
        rust_results = result_constant(native_dss, constant)
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

    appended = list(dataframes["qsts_result_frames"]["common_appended_columns"])
    for result_name in result_constant(frame_ops, "POWER_FLOW_RESULT_NAMES"):
        expected = dataframes["result_frames"][result_name]["columns"] + appended
        actual = dataframes["qsts_outputs"][result_name]["columns"]
        if actual != expected:
            fail(f"qsts_outputs.{result_name} does not compose the static result with QSTS metadata")
    for result_name in QSTS_RESULT_BUILDERS:
        if result_name == "execution_trace":
            continue
        expected = dataframes["result_frames"][result_name]["columns_before_time_metadata"] + appended
        actual = dataframes["qsts_outputs"][result_name]["columns"]
        if actual != expected:
            fail(f"qsts_outputs.{result_name} does not compose the state result with QSTS metadata")
    for result_name, contract in dataframes["qsts_outputs"].items():
        schema_at(dataframes, contract["base_schema"])

    for result_name, (source_path, function_name) in QSTS_METER_RESULT_BUILDERS.items():
        source = (ROOT / source_path).read_text(encoding="utf-8")
        rust_columns = dataframe_builder_columns(source, function_name)
        catalog_columns = dataframes["result_frames"][result_name]["columns"]
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
        if source and "://" not in source and not (ROOT / source).exists():
            fail(f"missing schema_source: {source}")

    validate_local_links()
    print(
        f"validated {len(actual_dsa)} DSA functions, "
        f"{len(study_facades['facades'])} DataFrame study facades, "
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
