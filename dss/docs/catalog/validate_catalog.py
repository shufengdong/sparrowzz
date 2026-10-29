#!/usr/bin/env python3
"""Validate the public RSDss documentation catalogs against the dispatch source."""

import json
import hashlib
import re
import sys
from pathlib import Path
from typing import Any, Dict, List, Optional, Set, Tuple


ROOT = Path(__file__).resolve().parents[4]
DOCS = ROOT / "sparrowzz/dss/docs"
FUNCTIONS = DOCS / "catalog/functions.json"
DATAFRAMES = DOCS / "catalog/dataframes.json"
STUDY_FACADES = DOCS / "catalog/study_facades.json"
CASE_PROJECTIONS = DOCS / "catalog/case_projections.json"
RSSBUNDLES = ROOT / "sparrowzz/dss/rssbundle"
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
        "ds_radial_execution_plan",
        "ds_newton_raphson_execution_plan",
    },
    "QSTS_RESULT_NAMES": {"ds_radial_qsts", "ds_newton_raphson_qsts"},
    "MONTE_CARLO_RESULT_NAMES": {"ds_monte_carlo"},
    "RESTORE_RESULT_NAMES": {"ds_restore"},
    "HARMONIC_RESULT_NAMES": {"ds_harmonic_study", "ds_harmonic_execution_plan"},
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


def schema_fields(schema):
    fields = set()
    contract = schema.get("contract") if isinstance(schema, dict) else None
    if isinstance(contract, dict) and isinstance(contract.get("columns"), list):
        fields.update(
            item.get("name")
            for item in contract["columns"]
            if isinstance(item, dict) and isinstance(item.get("name"), str)
        )
    for key in (
        "required",
        "optional",
        "defaults",
        "required_additional",
        "optional_additional",
    ):
        value = schema.get(key) if isinstance(schema, dict) else None
        if isinstance(value, dict):
            fields.update(value)
    if isinstance(schema, dict):
        for key in ("columns", "columns_before_time_metadata", "prefix_columns"):
            value = schema.get(key)
            if isinstance(value, list):
                fields.update(item for item in value if isinstance(item, str))
    return fields


def validate_base_schemas(dataframes):
    references = {}  # type: Dict[str, str]

    def collect(value, path):
        if isinstance(value, dict):
            if "base_schema" in value:
                reference = value["base_schema"]
                if not isinstance(reference, str) or not reference or "." not in reference:
                    fail(f"{path}.base_schema must use a complete catalog path")
                try:
                    target = schema_at(dataframes, reference)
                except AssertionError:
                    fail(f"unknown base_schema {reference} at {path}")
                if not isinstance(target, dict):
                    fail(f"{path}.base_schema must reference a schema object, got {reference}")
                components = reference.split(".")
                if len(components) != 2 or any(not component for component in components):
                    fail(
                        f"{path}.base_schema must use "
                        f"<catalogSection>.<schemaName>, got {reference}"
                    )
                references[path] = reference
            for key, item in value.items():
                collect(item, f"{path}.{key}" if path else key)
        elif isinstance(value, list):
            for index, item in enumerate(value):
                collect(item, f"{path}[{index}]")

    collect(dataframes, "")

    visited = set()  # type: Set[str]
    active = []

    def visit(path):
        if path in active:
            cycle = active[active.index(path) :] + [path]
            fail("circular base_schema reference: " + " -> ".join(cycle))
        if path in visited:
            return
        active.append(path)
        target = references.get(path)
        if target in references:
            visit(target)
        active.pop()
        visited.add(path)

    for path in references:
        visit(path)



_DTYPE_NAME = re.compile(r"^[A-Za-z_][A-Za-z0-9_]*$")
_DTYPE_GENERIC = re.compile(r"^([A-Za-z_][A-Za-z0-9_]*)<(.+)>$")


def validate_dtype_grammar(dataframes):
    grammar = dataframes.get("dtype_grammar")
    if not isinstance(grammar, dict) or grammar.get("version") != 1:
        fail("dataframes.dtype_grammar must use version 1")
    for key in ("primitives", "constructors", "semantic_types", "upload_conversions"):
        if not isinstance(grammar.get(key), dict):
            fail(f"dtype_grammar.{key} must be an object")
    names = set(grammar["primitives"]) | set(grammar["semantic_types"])
    if len(names) != len(grammar["primitives"]) + len(grammar["semantic_types"]):
        fail("dtype grammar primitive and semantic names must be disjoint")
    for name, definition in grammar["primitives"].items():
        if not _DTYPE_NAME.fullmatch(name) or not isinstance(definition, dict):
            fail(f"invalid primitive dtype definition {name}")
        for field in ("arrow", "parquet", "logical"):
            if not isinstance(definition.get(field), str) or not definition[field]:
                fail(f"primitive dtype {name} lacks {field}")
    for name, definition in grammar["constructors"].items():
        if not _DTYPE_NAME.fullmatch(name) or not isinstance(definition, dict):
            fail(f"invalid dtype constructor definition {name}")
        if definition.get("arity") != 1:
            fail(f"dtype constructor {name} must declare arity 1")
        for field in ("arrow", "parquet"):
            if not isinstance(definition.get(field), str) or "{element}" not in definition[field]:
                fail(f"dtype constructor {name} lacks a machine {field} template")
        if not isinstance(definition.get("nullableElements"), bool):
            fail(f"dtype constructor {name} must declare nullableElements")
    for name, definition in grammar["semantic_types"].items():
        if not _DTYPE_NAME.fullmatch(name) or not isinstance(definition, dict):
            fail(f"invalid semantic dtype definition {name}")
        if not isinstance(definition.get("accepts"), list) or not definition["accepts"]:
            fail(f"semantic dtype {name} must declare accepted physical dtypes")
        for field in ("canonicalArrow", "canonicalParquet"):
            if not isinstance(definition.get(field), str) or not definition[field]:
                fail(f"semantic dtype {name} lacks {field}")
    return grammar


def parse_dtype(dtype, grammar, location):
    if not isinstance(dtype, str) or not dtype:
        fail(f"{location} dtype must be a non-empty string")
    names = set(grammar["primitives"]) | set(grammar["semantic_types"])
    if dtype in names:
        return
    match = _DTYPE_GENERIC.fullmatch(dtype)
    if match is None:
        if any(token in dtype for token in "<>"):
            fail(f"invalid dtype syntax {dtype} at {location}")
        fail(f"undeclared dtype {dtype} at {location}")
    constructor, inner = match.groups()
    if constructor not in grammar["constructors"]:
        fail(f"undeclared dtype constructor {constructor} at {location}")
    # The grammar is unary. Reject commas and malformed nesting rather than
    # accepting an undocumented multi-argument expression.
    depth = 0
    for char in inner:
        if char == "<":
            depth += 1
        elif char == ">":
            depth -= 1
            if depth < 0:
                fail(f"invalid dtype syntax {dtype} at {location}")
        elif char == "," and depth == 0:
            fail(f"invalid dtype syntax {dtype} at {location}")
    if depth != 0:
        fail(f"invalid dtype syntax {dtype} at {location}")
    parse_dtype(inner, grammar, location)


def _top_level_schema(dataframes, reference):
    components = reference.split(".")
    if len(components) != 2 or any(not component for component in components):
        fail(f"schema reference must use <catalogSection>.<schemaName>, got {reference}")
    schema = schema_at(dataframes, reference)
    if not isinstance(schema, dict):
        fail(f"schemaRef {reference} does not identify a schema object")
    return schema


def resolve_contract_columns(dataframes, reference, active=None):
    active = [] if active is None else active
    if reference in active:
        cycle = active[active.index(reference):] + [reference]
        fail("circular base_schema reference: " + " -> ".join(cycle))
    schema = _top_level_schema(dataframes, reference)
    contract = schema.get("contract")
    if not isinstance(contract, dict):
        fail(f"prose-only or incomplete schema {reference}: missing contract")
    local = contract.get("columns")
    if not isinstance(local, list):
        fail(f"prose-only or incomplete schema {reference}: contract.columns must be a list")
    local_names = []
    for index, column in enumerate(local):
        if not isinstance(column, dict) or not isinstance(column.get("name"), str) or not column["name"]:
            fail(f"{reference}.contract.columns[{index}] must define a name")
        local_names.append(column["name"])
    if len(local_names) != len(set(local_names)):
        fail(f"{reference} contract has duplicate local columns")
    base = schema.get("base_schema")
    if not base:
        return list(local)
    base_columns = resolve_contract_columns(dataframes, base, active + [reference])
    base_names = {column["name"] for column in base_columns}
    conflicts = [name for name in local_names if name in base_names]
    if conflicts:
        fail(f"base_schema column conflict in {reference}: {', '.join(conflicts)}")
    merge = contract.get("baseMerge")
    if not isinstance(merge, dict) or merge.get("override") != "forbidden":
        fail(f"{reference}.contract.baseMerge must forbid overrides")
    order = merge.get("order")
    if order == "base_then_local":
        return base_columns + list(local)
    if order == "local_then_base":
        return list(local) + base_columns
    fail(f"{reference}.contract.baseMerge.order is invalid")


def _facade_references(study_facades):
    direct = {}
    for facade in study_facades["facades"]:
        function = facade["function"]
        for role, entries in (("input", facade["inputs"]), ("result", facade["results"])):
            for entry in entries:
                direct.setdefault(entry["schemaRef"], set()).add((function, role))
    return direct


def _affected_text(affected, reference):
    rows = affected.get(reference, set())
    return ", ".join(sorted(function for function, _ in rows)) or "none"


def _contract_error(affected, reference, detail):
    fail(f"{detail}; schema={reference}; affected facades=[{_affected_text(affected, reference)}]")


def _rule_field_references(rule):
    values = []
    for key in ("field", "left", "rightField", "lengthField"):
        value = rule.get(key)
        if isinstance(value, str):
            values.append(value)
    for key in ("fields", "required", "optional", "enabled"):
        value = rule.get(key)
        if isinstance(value, list):
            values.extend(item for item in value if isinstance(item, str))
    when = rule.get("when")
    if (
        isinstance(when, dict)
        and "input" not in when
        and isinstance(when.get("field"), str)
    ):
        values.append(when["field"])
    return values


def _validate_card(reference, column, all_fields, affected):
    name = column["name"]
    card = column.get("card")
    expected = {"default", "unit", "range", "enum", "visibleWhen", "enabledWhen"}
    if not isinstance(card, dict) or set(card) != expected:
        _contract_error(affected, reference, f"Struct card metadata incomplete at {reference}.{name}")
    default = card["default"]
    if not isinstance(default, dict) or default.get("state") not in {"value", "required", "omitted"}:
        _contract_error(affected, reference, f"invalid default state at {reference}.{name}")
    if default["state"] == "value" and "value" not in default:
        _contract_error(affected, reference, f"default value missing at {reference}.{name}")
    if default["state"] != "value" and "value" in default:
        _contract_error(affected, reference, f"non-value default carries a value at {reference}.{name}")
    for key in ("unit", "range", "enum"):
        value = card[key]
        if not isinstance(value, dict) or value.get("state") not in {"value", "bounded", "values", "not_applicable"}:
            _contract_error(affected, reference, f"invalid {key} metadata at {reference}.{name}")
    bounded = card["range"]
    if bounded["state"] == "bounded":
        for key in ("minimumInclusive", "maximumInclusive"):
            if not isinstance(bounded.get(key), bool):
                _contract_error(affected, reference, f"range {key} missing at {reference}.{name}")
        if "minimum" not in bounded or "maximum" not in bounded:
            _contract_error(affected, reference, f"range bounds missing at {reference}.{name}")
    enum = card["enum"]
    if enum["state"] == "values" and (not isinstance(enum.get("values"), list) or not enum["values"]):
        _contract_error(affected, reference, f"enum values missing at {reference}.{name}")
    for key in ("visibleWhen", "enabledWhen"):
        value = card[key]
        if not isinstance(value, dict) or value.get("state") not in {"always", "condition"}:
            _contract_error(affected, reference, f"invalid {key} at {reference}.{name}")
        if value["state"] == "condition":
            when = value.get("when")
            if not isinstance(when, dict) or when.get("field") not in all_fields:
                _contract_error(affected, reference, f"{key} references unknown field at {reference}.{name}")


def _validate_contract(dataframes, reference, expected_kind, grammar, affected):
    schema = _top_level_schema(dataframes, reference)
    contract = schema.get("contract")
    if not isinstance(contract, dict):
        _contract_error(affected, reference, f"prose-only or incomplete schema {reference}")
    required_contract_fields = {
        "kind", "rowCount", "columnPolicy", "columns", "stableKeys", "sortKeys",
        "columnAliases", "allowedConversions", "rules", "source", "shape"
    }
    missing = required_contract_fields - set(contract)
    if missing:
        _contract_error(affected, reference, f"prose-only or incomplete schema {reference}: missing {sorted(missing)}")
    if contract.get("kind") != expected_kind:
        _contract_error(affected, reference, f"contract kind differs: expected {expected_kind}, got {contract.get('kind')}")
    valid_rows = {"zero", "one", "zero_or_one", "zero_or_more", "one_or_more"}
    if contract.get("rowCount") not in valid_rows:
        _contract_error(affected, reference, f"invalid rowCount {contract.get('rowCount')}")
    policy = contract.get("columnPolicy")
    if not isinstance(policy, dict) or policy.get("mode") not in {"closed", "open", "dynamic"}:
        _contract_error(affected, reference, "columnPolicy must declare closed, open or dynamic mode")
    if policy["mode"] in {"closed", "open"} and policy.get("unknownColumns") not in {"forbidden", "ignored"}:
        _contract_error(affected, reference, "columnPolicy must declare unknownColumns behavior")
    if policy["mode"] == "dynamic" and not isinstance(policy.get("rules"), list):
        _contract_error(affected, reference, "dynamic columnPolicy requires machine rules")
    for key in ("stableKeys", "sortKeys", "allowedConversions", "rules", "source"):
        if not isinstance(contract.get(key), list):
            _contract_error(affected, reference, f"contract.{key} must be a list")
    if not contract["source"] or any(not isinstance(value, str) or not value for value in contract["source"]):
        _contract_error(affected, reference, "contract.source must cite parser, builder or test evidence")
    for source in contract["source"]:
        source_path = source.split("::", 1)[0]
        if "://" not in source_path and not (ROOT / source_path).exists():
            _contract_error(affected, reference, f"contract.source does not exist: {source}")
    if not isinstance(contract.get("columnAliases"), dict):
        _contract_error(affected, reference, "contract.columnAliases must be an object")
    try:
        columns = resolve_contract_columns(dataframes, reference)
    except AssertionError as error:
        _contract_error(affected, reference, str(error))
    if not columns:
        _contract_error(affected, reference, "prose-only or incomplete schema has no resolved columns")
    names = [column["name"] for column in columns]
    for index, column in enumerate(columns):
        name = column["name"]
        for key in ("dtype", "required", "nullable", "unit"):
            if key not in column:
                _contract_error(affected, reference, f"column metadata missing {key} at {reference}.{name}")
        if not isinstance(column["required"], bool) or not isinstance(column["nullable"], bool):
            _contract_error(affected, reference, f"required/nullable must be boolean at {reference}.{name}")
        if not isinstance(column["unit"], str) or not column["unit"]:
            _contract_error(affected, reference, f"unit must be explicit at {reference}.{name}")
        try:
            parse_dtype(column["dtype"], grammar, f"{reference}.{name}")
        except AssertionError as error:
            _contract_error(affected, reference, str(error))
    for key in ("stableKeys", "sortKeys"):
        unknown = set(contract[key]) - set(names)
        if unknown:
            _contract_error(affected, reference, f"contract.{key} references unknown fields {sorted(unknown)}")
    for alias, canonical in contract["columnAliases"].items():
        if not isinstance(alias, str) or canonical not in names:
            _contract_error(affected, reference, f"column alias {alias} has unknown target {canonical}")
    unknown_conversions = set(contract["allowedConversions"]) - set(grammar["upload_conversions"])
    if unknown_conversions:
        _contract_error(affected, reference, f"unknown upload conversions {sorted(unknown_conversions)}")
    virtual_fields = set()
    discriminator = contract.get("discriminator")
    if isinstance(discriminator, dict) and discriminator.get("synthetic") is True:
        virtual_fields.add(discriminator.get("field"))
    for rule_index, rule in enumerate(contract["rules"]):
        if not isinstance(rule, dict) or not isinstance(rule.get("type"), str):
            _contract_error(affected, reference, f"rule {rule_index} must have a machine type")
        unknown = set(_rule_field_references(rule)) - set(names) - virtual_fields
        if unknown:
            _contract_error(affected, reference, f"rule {rule_index} references unknown fields {sorted(unknown)}")
    if expected_kind == "struct":
        if contract["rowCount"] not in {"zero", "one", "zero_or_one"}:
            _contract_error(affected, reference, "Struct rowCount must be zero, one or zero_or_one")
        for column in columns:
            _validate_card(reference, column, set(names), affected)
    shape = contract.get("shape")
    if shape not in {"uniform", "discriminated"}:
        _contract_error(affected, reference, "contract.shape must be uniform or discriminated")
    discriminator = contract.get("discriminator")
    variants = contract.get("variants")
    if shape == "uniform":
        if discriminator is not None or variants is not None:
            _contract_error(affected, reference, "uniform schema must not declare discriminator or variants")
    else:
        if not isinstance(discriminator, dict) or not isinstance(variants, list) or not variants:
            _contract_error(affected, reference, "discriminated schema requires discriminator and non-empty variants")
        field = discriminator.get("field")
        source = discriminator.get("source")
        if source not in {"column", "presence", "external_struct"}:
            _contract_error(affected, reference, "discriminator.source must be column, presence or external_struct")
        if source == "column" and field not in names:
            _contract_error(affected, reference, f"discriminator references unknown field {field}")
        if source == "presence" and discriminator.get("synthetic") is not True:
            _contract_error(affected, reference, "presence discriminator must be explicitly synthetic")
        if not isinstance(field, str) or not field:
            _contract_error(affected, reference, "discriminator must define a field")
        ids = []
        for variant in variants:
            if not isinstance(variant, dict) or not isinstance(variant.get("id"), str) or not variant["id"]:
                _contract_error(affected, reference, "variant must define a stable id")
            ids.append(variant["id"])
            when = variant.get("when")
            if not isinstance(when, dict) or not any(key in when for key in ("equals", "notEquals", "in", "present")):
                _contract_error(affected, reference, f"variant {variant['id']} lacks a machine when clause")
            present = when.get("present", [])
            if present and (not isinstance(present, list) or any(item not in names for item in present)):
                _contract_error(affected, reference, f"variant {variant['id']} presence discriminator references unknown fields")
            unknown = set(_rule_field_references(variant)) - set(names)
            if unknown:
                _contract_error(affected, reference, f"variant {variant['id']} references unknown fields {sorted(unknown)}")
            for side_schema in variant.get("sideSchemas", []):
                try:
                    _top_level_schema(dataframes, side_schema)
                except AssertionError as error:
                    _contract_error(affected, reference, f"variant {variant['id']} has invalid side schema: {error}")
        if len(ids) != len(set(ids)):
            _contract_error(affected, reference, "variant ids must be unique")
    return columns


def validate_catalog_contracts(dataframes, study_facades):
    grammar = validate_dtype_grammar(dataframes)
    direct = _facade_references(study_facades)
    affected = {reference: set(rows) for reference, rows in direct.items()}
    # Propagate facade context through every base_schema edge so a low-level
    # diagnostic names each public facade whose effective contract is broken.
    changed = True
    while changed:
        changed = False
        for reference, rows in list(affected.items()):
            schema = _top_level_schema(dataframes, reference)
            base = schema.get("base_schema")
            if base:
                before = len(affected.setdefault(base, set()))
                affected[base].update(rows)
                changed |= len(affected[base]) != before
    model_refs = {entry["schemaRef"] for facade in study_facades["facades"] for entry in facade["inputs"] if entry["inputKind"] == "model"}
    data_refs = {entry["schemaRef"] for facade in study_facades["facades"] for entry in facade["inputs"] if entry["inputKind"] == "data"}
    struct_refs = {entry["schemaRef"] for facade in study_facades["facades"] for entry in facade["inputs"] if entry["inputKind"] == "struct"}
    result_refs = {entry["schemaRef"] for facade in study_facades["facades"] for entry in facade["results"]}
    expected = {}
    for kind, refs in (("model", model_refs), ("data", data_refs), ("struct", struct_refs), ("result", result_refs)):
        for reference in refs:
            if reference in expected and expected[reference] != kind:
                _contract_error(affected, reference, f"schema used as both {expected[reference]} and {kind}")
            expected[reference] = kind
    # Include base schemas using the consuming schema kind; incompatible use is rejected.
    queue = list(expected)
    while queue:
        reference = queue.pop()
        schema = _top_level_schema(dataframes, reference)
        base = schema.get("base_schema")
        if not base:
            continue
        kind = expected[reference]
        if base in expected and expected[base] != kind:
            _contract_error(affected, base, f"base schema kind conflict {expected[base]} versus {kind}")
        if base not in expected:
            expected[base] = kind
            queue.append(base)
    def inheritance_depth(reference, active=None):
        active = set() if active is None else active
        if reference in active:
            return 0
        base = _top_level_schema(dataframes, reference).get("base_schema")
        return 0 if not base else 1 + inheritance_depth(base, active | {reference})

    # Validate roots before derived contracts so an invalid shared base is
    # diagnosed at its authoritative schema and names every affected facade.
    for reference in sorted(expected, key=lambda value: (inheritance_depth(value), value)):
        _validate_contract(dataframes, reference, expected[reference], grammar, affected)
    return {
        "facades": len(study_facades["facades"]),
        "models": len(model_refs),
        "data": len(data_refs),
        "structs": len(struct_refs),
        "results": len(result_refs),
        "resolvedResults": len({reference for reference, kind in expected.items() if kind == "result"}),
        "diagnostics": 0,
    }

def validate_result_constraints(function, facade, dataframes, result_names, recommended):
    constraints = facade.get("resultConstraints")
    if not isinstance(constraints, list) or not constraints:
        fail(f"{function} must define resultConstraints")

    inputs = facade["inputs"]
    inputs_by_id = {item["id"]: item for item in inputs}
    inputs_by_alias = {item["alias"]: item for item in inputs}
    defaults = []
    selection_count = 0

    def result_list(constraint, kind):
        results = constraint.get("results")
        if not isinstance(results, list) or not results:
            fail(f"{function} {kind}.results must be a non-empty list")
        if any(not isinstance(name, str) or not name for name in results):
            fail(f"{function} {kind}.results must contain non-empty strings")
        if len(results) != len(set(results)):
            fail(f"{function} {kind}.results must be unique")
        unknown = set(results) - set(result_names)
        if unknown:
            fail(f"{function} {kind} references unknown results {sorted(unknown)}")
        return results

    for index, constraint in enumerate(constraints):
        if not isinstance(constraint, dict):
            fail(f"{function} resultConstraints[{index}] must be an object")
        kind = constraint.get("type")
        if not isinstance(kind, str):
            fail(f"{function} resultConstraints[{index}].type must be a string")

        if kind == "ordered_unique_selection":
            selection_count += 1
            selector = constraint.get("selector")
            if not isinstance(selector, str) or not selector.strip() or "." not in selector:
                fail(f"{function} ordered_unique_selection.selector must be a dotted string")
            minimum = constraint.get("minimum")
            if isinstance(minimum, bool) or not isinstance(minimum, int) or minimum < 1:
                fail(f"{function} ordered_unique_selection.minimum must be an integer >= 1")
            if constraint.get("bindingCount") != "selected_result_count":
                fail(
                    f"{function} ordered_unique_selection.bindingCount "
                    "must equal selected_result_count"
                )
            components = selector.split(".")
            if len(components) != 2 or any(not component for component in components):
                fail(
                    f"{function} ordered_unique_selection selector must use "
                    "<structAlias>.<field>"
                )
            selector_input = inputs_by_alias.get(components[0])
            if selector_input is None or selector_input["inputKind"] != "struct":
                fail(
                    f"{function} ordered_unique_selection selector must start with a Struct input alias"
                )
            selector_schema = schema_at(dataframes, selector_input["schemaRef"])
            if components[-1] not in schema_fields(selector_schema):
                fail(
                    f"{function} ordered_unique_selection selector has unknown schema field "
                    f"{components[-1]}"
                )
        elif kind == "default_results":
            defaults.append(kind)
            selected = result_list(constraint, kind)
            if not set(selected).issubset(recommended):
                fail(f"{function} recommended results do not include its default result set")
        elif kind == "fixed_results":
            defaults.append(kind)
            selected = result_list(constraint, kind)
            binding_count = constraint.get("bindingCount")
            if isinstance(binding_count, bool) or not isinstance(binding_count, int):
                fail(f"{function} fixed_results.bindingCount must be an integer")
            if binding_count != len(selected):
                fail(
                    f"{function} fixed_results.bindingCount must equal its result count "
                    f"{len(selected)}"
                )
            if selected != result_names:
                fail(f"{function} fixed_results must equal the facade result list")
            if not set(selected).issubset(recommended):
                fail(f"{function} recommended results do not include its fixed result set")
        elif kind == "conditional_unavailable":
            result = constraint.get("result")
            if not isinstance(result, str) or not result:
                fail(f"{function} conditional_unavailable.result must be a non-empty string")
            if result not in result_names:
                fail(f"{function} conditional_unavailable references unknown result {result}")
            when = constraint.get("when")
            if not isinstance(when, dict):
                fail(f"{function} conditional_unavailable.when must be an object")
            input_id = when.get("input")
            if not isinstance(input_id, str) or not input_id:
                fail(f"{function} conditional_unavailable.when.input must be a non-empty string")
            if input_id not in inputs_by_id:
                fail(f"{function} conditional_unavailable references unknown input {input_id}")
            if not isinstance(when.get("nonEmpty"), bool):
                fail(f"{function} conditional_unavailable.when.nonEmpty must be boolean")
        else:
            fail(f"{function} has unknown result constraint {kind}")

    if len(defaults) != 1:
        fail(f"{function} must define exactly one default_results or fixed_results constraint")
    if defaults[0] == "default_results" and selection_count != 1:
        fail(
            f"{function} selectable result mode requires exactly one "
            "ordered_unique_selection and one default_results constraint"
        )
    if defaults[0] == "fixed_results" and selection_count != 0:
        fail(
            f"{function} fixed result mode requires exactly one fixed_results constraint "
            "and no ordered_unique_selection"
        )


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
            if item["schemaRef"].startswith("execution_inputs."):
                schema_name = item["schemaRef"].split(".", 1)[1]
                expected_kind = (
                    "model"
                    if schema_name in set(network_order) | {
                        "shape_assignment", "storage_controller", "switch_control",
                        "fuse", "recloser", "relay", "monitor", "energy_meter",
                    }
                    else "data"
                )
            if item["schemaRef"] == "harmonic_execution_inputs.monitor":
                expected_kind = "model"
            if item["schemaRef"] in {
                "harmonic_execution_inputs.frequency_axis",
                "harmonic_execution_inputs.frequency_spectrum",
            }:
                expected_kind = "data"
            if item["schemaRef"] == "harmonic_execution_inputs.plan":
                expected_kind = "struct"
            if item["schemaRef"].startswith("harmonic_snapshot_inputs."):
                expected_kind = (
                    "data"
                    if item["schemaRef"] == "harmonic_snapshot_inputs.snapshot"
                    else "model"
                )
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

        validate_result_constraints(function, facade, dataframes, names, recommended)

    by_function = {facade["function"]: facade for facade in facades}
    for group in [
        ("ds_radial_pf", "ds_newton_raphson_pf"),
        ("ds_radial_qsts", "ds_newton_raphson_qsts"),
        ("ds_wls_se", "ds_mnmr_se", "ds_mce_se"),
        ("ds_network_dynamics", "ds_motor_dynamics"),
    ]:
        if len({by_function[name]["id"] for name in group}) != len(group):
            fail(f"related algorithms must remain separate facades: {group}")


def validate_case_projections(case_projections, study_facades, dsa_by_name, frame_ops, translate):
    if case_projections.get("schema_version") != 1:
        fail("case_projections.json must use schema_version 1")
    profiles = case_projections.get("profiles")
    if not isinstance(profiles, list) or not profiles:
        fail("case_projections.profiles must be a non-empty list")
    facade_by_function = {facade["function"]: facade for facade in study_facades["facades"]}
    expected_sources = {
        "ds_radial_pf",
        "ds_newton_raphson_pf",
        "ds_radial_dss_plan",
        "ds_newton_raphson_dss_plan",
        "ds_dss_fault_study",
        "ds_dss_harmonic_plan",
        "ds_dss_dynamic_plan",
        "ds_dss_auto_add",
        "ds_dss_relcalc",
    }
    ids = [profile.get("id") for profile in profiles]
    sources = [profile.get("sourceFunction") for profile in profiles]
    if any(not isinstance(value, str) or not value for value in ids) or len(ids) != len(set(ids)):
        fail("case projection profile ids must be non-empty and unique")
    if len(sources) != len(set(sources)):
        fail("each path study function must have exactly one case projection profile")
    if set(sources) != expected_sources:
        fail(
            "case projection source mismatch: "
            f"missing={sorted(expected_sources - set(sources))}, "
            f"extra={sorted(set(sources) - expected_sources)}"
        )
    for profile in profiles:
        source = profile["sourceFunction"]
        target = profile.get("targetFunction")
        projector = profile.get("projector")
        if source not in dsa_by_name:
            fail(f"case projection {profile['id']} references unknown source function {source}")
        compatibility = profile.get("compatibility")
        if not isinstance(compatibility, dict) or compatibility.get("failClosed") is not True:
            fail(f"case projection {profile['id']} must be fail-closed")
        if compatibility.get("status") not in {"enabled", "disabled"}:
            fail(f"case projection {profile['id']} has invalid compatibility status")
        if not isinstance(compatibility.get("stableDiagnostic"), str) or not compatibility["stableDiagnostic"]:
            fail(f"case projection {profile['id']} lacks a stable diagnostic")
        if profile.get("managedEntry") != "managed.rss":
            fail(f"case projection {profile['id']} must publish managed.rss")
        if compatibility["status"] == "disabled":
            if target or projector != "unsupported" or profile.get("inputs") or profile.get("results"):
                fail(f"disabled case projection {profile['id']} cannot advertise an executable target")
            continue
        if target not in facade_by_function:
            fail(f"case projection {profile['id']} references unknown DataFrame facade {target}")
        if not isinstance(projector, str) or f'"{projector}"' not in frame_ops or f'"{projector}"' not in translate:
            fail(f"case projection {profile['id']} references unknown projector {projector}")
        facade = facade_by_function[target]
        if profile.get("inputs") != facade["inputs"]:
            fail(f"case projection {profile['id']} input contract differs from {target}")
        if profile.get("results") != facade["results"]:
            fail(f"case projection {profile['id']} result contract differs from {target}")
        source_results = profile.get("sourceResults")
        result_map = profile.get("resultMap")
        if not isinstance(source_results, list) or not source_results or len(source_results) != len(set(source_results)):
            fail(f"case projection {profile['id']} sourceResults must be non-empty and unique")
        if not isinstance(result_map, dict) or set(result_map) != set(source_results):
            fail(f"case projection {profile['id']} resultMap must cover every source result")
        target_results = {result["name"] for result in facade["results"]}
        unknown = set(result_map.values()) - target_results
        if unknown:
            fail(f"case projection {profile['id']} maps unknown target results {sorted(unknown)}")
        bundle_result_map = profile.get("bundleResultMap")
        if not isinstance(bundle_result_map, dict) or not bundle_result_map:
            fail(f"case projection {profile['id']} must declare bundleResultMap")
        unknown = set(bundle_result_map.values()) - target_results
        if unknown:
            fail(f"case projection {profile['id']} bundleResultMap references unknown target results {sorted(unknown)}")
        selector = profile.get("resultSelector")
        if selector != {"input": "para", "field": "result", "valueSource": "profile.resultMap(manifest.study.results)"}:
            fail(f"case projection {profile['id']} must declare the para.result manifest mapping")


def validate_rssbundles(case_projections, study_facades, dataframes):
    profiles = {profile["id"]: profile for profile in case_projections["profiles"]}
    facades = {facade["function"]: facade for facade in study_facades["facades"]}
    manifests = sorted(RSSBUNDLES.glob("**/manifest.json"))
    if not manifests:
        fail("rssbundle library contains no manifests")
    for path in manifests:
        manifest = load_json(path)
        label = str(path.relative_to(RSSBUNDLES))
        if manifest.get("schema_version") != 2:
            fail(f"{label} must use bundle schema_version 2")
        projection = manifest.get("projection")
        if not isinstance(projection, dict):
            fail(f"{label} lacks a projection contract")
        profile = profiles.get(projection.get("profile"))
        if profile is None:
            fail(f"{label} references unknown projection profile {projection.get('profile')}")
        for manifest_key, profile_key in (
            ("source_function", "sourceFunction"),
            ("projector", "projector"),
            ("target_function", "targetFunction"),
            ("managed_entry", "managedEntry"),
        ):
            if projection.get(manifest_key) != profile.get(profile_key):
                fail(f"{label} projection {manifest_key} differs from {profile['id']}")
        if projection.get("status") != "enabled" or projection.get("stable_diagnostic") != profile["compatibility"]["stableDiagnostic"]:
            fail(f"{label} projection must publish the enabled fail-closed profile status")
        facade = facades[profile["targetFunction"]]
        for item in facade["inputs"] + facade["results"]:
            schema_at(dataframes, item["schemaRef"])
        source_results = manifest.get("study", {}).get("results")
        projected_results = projection.get("results")
        if not isinstance(source_results, list) or not isinstance(projected_results, list) or len(source_results) != len(projected_results):
            fail(f"{label} projection results differ from study.results")
        expected_map = profile["bundleResultMap"]
        for source, result in zip(source_results, projected_results):
            if result.get("source") != source or result.get("target") != expected_map.get(source):
                fail(f"{label} cannot map published result {source} through {profile['id']}")
            if result.get("binding") != f"result_{source}":
                fail(f"{label} uses a non-canonical managed result binding for {source}")
        managed = path.parent / projection["managed_entry"]
        if not managed.is_file():
            fail(f"{label} is missing {projection['managed_entry']}")
        script = managed.read_text(encoding="utf-8")
        if any(token in script.lower() for token in (".dss", "import_dss", "source_root", "http://", "https://")):
            fail(f"{label} managed entry contains a forbidden path or import token")
        expected_arguments = ", ".join(item["alias"] for item in facade["inputs"])
        expected_bindings = ", ".join(result["binding"] for result in projected_results)
        expected_left = expected_bindings if len(projected_results) == 1 else f"[{expected_bindings}]"
        if f"{expected_left} = {profile['targetFunction']}({expected_arguments});" not in script:
            fail(f"{label} managed entry differs from its DataFrame facade contract")
        declared = [item for item in manifest.get("files", []) if item.get("path") == projection["managed_entry"]]
        if len(declared) != 1 or declared[0].get("role") != "managed-rss":
            fail(f"{label} must declare managed.rss with role managed-rss")
        actual_hash = hashlib.sha256(managed.read_bytes()).hexdigest()
        if declared[0].get("sha256") != actual_hash:
            fail(f"{label} managed entry hash is stale")
    return len(manifests)


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
    for markdown in DOCS.rglob("*.md"):
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
    case_projections = load_json(CASE_PROJECTIONS)
    if functions.get("schema_version") != 2:
        fail("functions.json must use schema_version 2")
    if dataframes.get("schema_version") != 3:
        fail("dataframes.json must use schema_version 3")

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

    validate_base_schemas(dataframes)
    validate_study_facades(functions, dataframes, study_facades, dsa_by_name)
    validate_case_projections(case_projections, study_facades, dsa_by_name, frame_ops, translate)
    bundle_count = validate_rssbundles(case_projections, study_facades, dataframes)
    contract_stats = validate_catalog_contracts(dataframes, study_facades)

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
        f"{contract_stats['facades']} DataFrame study facades, "
        f"{contract_stats['models']} model, {contract_stats['data']} data, "
        f"{contract_stats['structs']} struct and {contract_stats['results']} direct "
        f"result schemas ({contract_stats['resolvedResults']} including inherited bases), "
        f"{bundle_count} projected bundles, zero completeness diagnostics"
    )
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except AssertionError as error:
        print(f"catalog validation failed: {error}", file=sys.stderr)
        sys.exit(1)
