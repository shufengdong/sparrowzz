import contextlib
import copy
import importlib.util
import io
import json
import shutil
import tempfile
import unittest
from pathlib import Path
from unittest import mock


CATALOG_DIR = Path(__file__).resolve().parent
VALIDATOR_PATH = CATALOG_DIR / "validate_catalog.py"


def load_validator():
    spec = importlib.util.spec_from_file_location("dss_catalog_validator", VALIDATOR_PATH)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class CatalogValidatorTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.validator = load_validator()

    def setUp(self):
        self.temp_dir = tempfile.TemporaryDirectory()
        self.catalog_dir = Path(self.temp_dir.name)
        for name in ("functions.json", "dataframes.json", "study_facades.json", "case_projections.json"):
            shutil.copy2(CATALOG_DIR / name, self.catalog_dir / name)

    def tearDown(self):
        self.temp_dir.cleanup()

    def read(self, name):
        return json.loads((self.catalog_dir / name).read_text(encoding="utf-8"))

    def write(self, name, value):
        (self.catalog_dir / name).write_text(
            json.dumps(value, ensure_ascii=False, indent=2) + "\n",
            encoding="utf-8",
        )

    def facade(self, catalog, function):
        return next(item for item in catalog["facades"] if item["function"] == function)

    def dsa_contract(self, catalog, function):
        return next(
            item for item in catalog["dsa_functions"]
            if function in item["names"]
        )

    def constraint(self, facade, kind):
        return next(item for item in facade["resultConstraints"] if item["type"] == kind)

    def assert_invalid(self, pattern):
        with mock.patch.multiple(
            self.validator,
            FUNCTIONS=self.catalog_dir / "functions.json",
            DATAFRAMES=self.catalog_dir / "dataframes.json",
            STUDY_FACADES=self.catalog_dir / "study_facades.json",
            CASE_PROJECTIONS=self.catalog_dir / "case_projections.json",
        ):
            with contextlib.redirect_stdout(io.StringIO()):
                with self.assertRaisesRegex(AssertionError, pattern):
                    self.validator.main()

    def test_study_sequence_fixed_v1_profile_and_result_schemas(self):
        catalog = self.read("case_projections.json")
        profiles = [p for p in catalog["profiles"] if p["id"] == "dss_study_sequence_v1"]
        self.assertEqual(len(profiles), 1, "missing sequence v1 profile")
        profile = profiles[0]
        facade = self.facade(self.read("study_facades.json"), "ds_study_sequence")
        self.assertEqual(profile["inputs"], facade["inputs"])
        self.assertEqual(len(profile["inputs"]), 48)
        self.assertEqual([i["position"] for i in profile["inputs"]], list(range(1,49)))
        self.assertEqual([i["id"] for i in profile["inputs"][:2]], ["sequence_plan", "sequence_transition"])
        self.assertEqual([i["id"] for i in profile["inputs"][-3:]], ["sequence_request", "para", "sequence_diagnostic"])
        self.assertEqual(profile["results"], facade["results"])
        dataframes = self.read("dataframes.json")
        load_columns = {c["name"]: c for c in dataframes["sequence_inputs"]["load"]["contract"]["columns"]}
        self.assertEqual(load_columns["p_nom"]["unit"], "W")
        epoch_columns = {c["name"]: c for c in dataframes["sequence_inputs"]["execution_epoch"]["contract"]["columns"]}
        self.assertEqual(epoch_columns["base_frequency_hz"]["unit"], "Hz")
        for item in profile["inputs"] + profile["results"]:
            self.validator.schema_at(dataframes, item["schemaRef"])
        self.assertIn("study_status", [r["name"] for r in profile["results"]])
        self.assertEqual(self.dsa_contract(self.read("functions.json"), "ds_study_sequence")["arity"],48)

    def test_study_sequence_catalog_registers_source_and_projector(self):
        functions = self.read("functions.json")
        self.assertTrue(any("ds_dss_study_sequence" in item["names"] for item in functions["dsa_functions"]))
        self.assertIn("project_dss_study_sequence", json.dumps(functions))

    def test_study_sequence_reactor_preserves_to_terminal_incidence(self):
        dataframes = self.read("dataframes.json")
        reactor = dataframes["sequence_inputs"]["reactor"]
        self.assertEqual(reactor["columns"][2:4], ["phases", "terminal_map_to"])
        columns = reactor["contract"]["columns"]
        self.assertEqual(columns[3], {
            "name": "terminal_map_to", "dtype": "list<str>",
            "required": True, "nullable": True, "unit": "not_applicable",
        })
        self.validator.validate_study_sequence_catalog(
            self.read("functions.json"), dataframes,
            self.read("study_facades.json"), self.read("case_projections.json"),
        )
        reactor["columns"].remove("terminal_map_to")
        columns.pop(3)
        with self.assertRaisesRegex(AssertionError, r"physical schema differs at sequence_inputs\.reactor"):
            self.validator.validate_study_sequence_catalog(
                self.read("functions.json"), dataframes,
                self.read("study_facades.json"), self.read("case_projections.json"),
            )

    def test_reactor_to_terminals_are_optional_in_inherited_v2_contracts(self):
        dataframes = self.read("dataframes.json")
        for reference in ("frames.reactor", "execution_inputs.reactor", "harmonic_snapshot_inputs.reactor"):
            columns = self.validator.resolve_contract_columns(dataframes, reference)
            endpoints = [column for column in columns if column["name"] == "terminal_map_to"]
            self.assertEqual(endpoints, [{
                "name": "terminal_map_to", "dtype": "list<str>",
                "required": False, "nullable": True, "unit": "not_applicable",
            }], reference)
        for family in ("execution_inputs", "harmonic_snapshot_inputs"):
            self.assertEqual(dataframes[family]["reactor"]["contract"]["columnPolicy"], {
                "mode": "closed", "unknownColumns": "forbidden",
            })

    def test_study_sequence_catalog_rejects_fixed_slot_order_drift(self):
        catalog = self.read("case_projections.json")
        profile = next(p for p in catalog["profiles"] if p["id"] == "dss_study_sequence_v1")
        profile["inputs"][0]["schemaRef"] = "sequence_inputs.sequence_transition"
        with self.assertRaisesRegex(AssertionError, "sequence.*schema"):
            self.validator.validate_study_sequence_catalog(self.read("functions.json"), self.read("dataframes.json"), self.read("study_facades.json"), catalog)

    def test_study_sequence_catalog_rejects_unmapped_result(self):
        facades = self.read("study_facades.json")
        facade = self.facade(facades, "ds_study_sequence")
        facade["results"][0]["schemaRef"] = "result_frames.does_not_exist"
        with self.assertRaisesRegex(AssertionError, "schemaRef"):
            self.validator.validate_study_sequence_catalog(self.read("functions.json"), self.read("dataframes.json"), facades, self.read("case_projections.json"))

    def test_study_sequence_catalog_rejects_missing_result_family(self):
        facades = self.read("study_facades.json")
        self.facade(facades, "ds_study_sequence")["results"].pop(0)
        with self.assertRaisesRegex(AssertionError, "sequence.*result"):
            self.validator.validate_study_sequence_catalog(self.read("functions.json"), self.read("dataframes.json"), facades, self.read("case_projections.json"))

    def test_study_sequence_selection_uses_request_without_changing_para_schema(self):
        facade = self.facade(self.read("study_facades.json"), "ds_study_sequence")
        self.assertEqual(facade["resultConstraints"][0]["type"], "ordered_stage_selection")
        self.assertEqual(facade["resultConstraints"][0]["selector"], "sequence_request.result")
        columns = self.validator.resolve_contract_columns(self.read("dataframes.json"), "sequence_inputs.para")
        self.assertEqual([(c["name"], c["dtype"]) for c in columns], [("line_model", "str")])

    def test_harmonic_monitor_save_uses_full_base_schema(self):
        catalog = self.read("dataframes.json")
        self.assertEqual(
            catalog["result_frames"]["harmonic_monitor_save"]["base_schema"],
            "result_frames.harmonic_monitor",
        )

    def test_harmonic_execution_facade_uses_dedicated_stable_result_schemas(self):
        facades = self.read("study_facades.json")
        facade = self.facade(facades, "ds_harmonic_execution_plan")
        references = [result["schemaRef"] for result in facade["results"]]
        self.assertEqual(
            references,
            [f"result_frames.{name}" for name in self.validator.HARMONIC_EXECUTION_RESULT_COLUMNS],
        )
        catalog = self.read("dataframes.json")
        for name, base_columns in self.validator.HARMONIC_EXECUTION_RESULT_COLUMNS.items():
            schema = catalog["result_frames"][name]
            expected = base_columns + self.validator.HARMONIC_EXECUTION_IDENTITY_COLUMNS
            self.assertEqual(schema["columns"], expected)
            self.assertEqual(
                [column["name"] for column in schema["contract"]["columns"]],
                expected,
            )

    def test_rejects_harmonic_execution_result_schema_drift(self):
        catalog = self.read("dataframes.json")
        schema = catalog["result_frames"]["harmonic_execution_bus_voltage"]
        schema["contract"]["columns"][-1]["unit"] = "not_applicable"
        self.write("dataframes.json", catalog)
        self.assert_invalid(
            r"result_frames\.harmonic_execution_bus_voltage\.time_s has an invalid execution identity contract"
        )

    def test_rejects_non_nullable_harmonic_execution_identity(self):
        catalog = self.read("dataframes.json")
        schema = catalog["result_frames"]["harmonic_execution_branch_flow"]
        harmonic = next(column for column in schema["contract"]["columns"] if column["name"] == "harmonic")
        harmonic["nullable"] = False
        self.write("dataframes.json", catalog)
        self.assert_invalid(r"result_frames\.harmonic_execution_branch_flow\.harmonic must be nullable u32")

    def test_dataframe_builder_columns_accepts_direct_df_result(self):
        source = '''
pub(crate) fn monitor_output_frame() -> Result<DataFrame, PolarsError> {
    df![
        "monitor" => ["m1"],
        "time" => [0.0],
    ]
}
'''
        self.assertEqual(
            self.validator.dataframe_builder_columns(source, "monitor_output_frame"),
            ["monitor", "time"],
        )

    def test_facade_input_aliases_do_not_collide_with_result_names(self):
        catalog = self.read("study_facades.json")
        for facade in catalog["facades"]:
            aliases = {item["alias"] for item in facade["inputs"]}
            result_names = {item["name"] for item in facade["results"]}
            self.assertFalse(
                aliases & result_names,
                f"{facade['function']} input aliases collide with result names",
            )
        fault_study = self.facade(catalog, "ds_fault_study")
        fault_input = next(item for item in fault_study["inputs"] if item["id"] == "fault")
        self.assertEqual(fault_input["alias"], "fault_input")

    def test_rejects_recommended_result_name_colliding_with_input_alias(self):
        catalog = self.read("study_facades.json")
        facade = self.facade(catalog, "ds_radial_pf")
        recommended = next(item["name"] for item in facade["results"] if item["recommended"])
        facade["inputs"][0]["alias"] = recommended
        self.write("study_facades.json", catalog)
        self.assert_invalid(r"ds_radial_pf input alias bus_voltage collides with a result name")

    def test_rejects_non_recommended_result_name_colliding_with_input_alias(self):
        catalog = self.read("study_facades.json")
        facade = self.facade(catalog, "ds_motor_dynamics")
        self.assertFalse(next(item for item in facade["results"] if item["name"] == "bus_voltage")["recommended"])
        facade["inputs"][0]["alias"] = "bus_voltage"
        self.write("study_facades.json", catalog)
        self.assert_invalid(r"ds_motor_dynamics input alias bus_voltage collides with a result name")

    def test_five_path_lifecycles_have_fail_closed_dataframe_projections(self):
        catalog = self.read("case_projections.json")
        lifecycle_sources = {
            "ds_radial_dss_plan", "ds_newton_raphson_dss_plan", "ds_dss_fault_study",
            "ds_dss_harmonic_plan", "ds_dss_dynamic_plan",
        }
        path_profiles = [item for item in catalog["profiles"] if item["sourceFunction"] in lifecycle_sources]
        self.assertEqual(len(path_profiles), 5)
        self.assertEqual(
            {item["sourceFunction"] for item in path_profiles},
            {
                "ds_radial_dss_plan",
                "ds_newton_raphson_dss_plan",
                "ds_dss_fault_study",
                "ds_dss_harmonic_plan",
                "ds_dss_dynamic_plan",
            },
        )
        self.assertTrue(all(item["compatibility"]["failClosed"] for item in path_profiles))

    def test_all_published_bundles_have_pathless_dataframe_projections(self):
        count = self.validator.validate_rssbundles(
            self.read("case_projections.json"),
            self.read("study_facades.json"),
            self.read("dataframes.json"),
        )
        self.assertEqual(count, 180)

    def test_network_dynamics_catalog_publishes_v13_monitor_additively(self):
        functions = self.read("functions.json")
        facades = self.read("study_facades.json")
        projections = self.read("case_projections.json")

        contract = self.dsa_contract(functions, "ds_network_dynamics")
        facade = self.facade(facades, "ds_network_dynamics")
        projection = next(
            item for item in projections["profiles"]
            if item["id"] == "dss_dynamic_plan_v1"
        )

        self.assertEqual(contract["arity"], 30)
        self.assertEqual(contract["arguments"][-2:], ["para", "monitor"])
        self.assertEqual(len(contract["results"]), 16)
        self.assertEqual(contract["results"][-1], "monitor")
        self.assertIn("V12", contract["behavior"])
        self.assertIn("V13", contract["behavior"])

        self.assertIn("V13", facade["title"])
        self.assertEqual(len(facade["inputs"]), 30)
        self.assertEqual(
            facade["inputs"][-1],
            {
                "id": "monitor",
                "alias": "monitor_input",
                "inputKind": "model",
                "schemaRef": "frames.monitor",
                "required": True,
                "position": 30,
                "description": "Dynamic V13 的 PVSystem/Storage mode 0、1、3 Monitor 定义。",
            },
        )
        self.assertEqual(len(facade["results"]), 16)
        self.assertEqual(facade["results"][-1]["name"], "monitor")
        self.assertEqual(
            facade["results"][-1]["schemaRef"],
            "result_frames.qsts_monitor",
        )
        self.assertTrue(any("V12" in item and "29" in item for item in facade["differences"]))
        self.assertEqual(projection["inputs"], facade["inputs"])
        self.assertEqual(projection["results"], facade["results"])

    def test_enabled_case_projections_select_managed_projection_results(self):
        catalog = self.read("case_projections.json")
        selectors = {
            profile["id"]: profile["resultSelector"]["valueSource"]
            for profile in catalog["profiles"]
            if profile["compatibility"]["status"] == "enabled"
        }
        self.assertTrue(selectors)
        self.assertEqual(
            set(selectors.values()),
            {"profile.resultMap(manifest.projection.results)", "manifest.study_sequence.results"},
        )

    def test_native_dynamic_catalog_publishes_v13_monitor(self):
        functions = self.read("functions.json")
        dataframes = self.read("dataframes.json")
        contract = self.dsa_contract(functions, "ds_dss_dynamic_plan")

        self.assertIn("V13", contract["behavior"])
        self.assertEqual(len(contract["results"]), 16)
        self.assertEqual(contract["results"][-1], "monitor")
        monitor = dataframes["result_frames"]["qsts_monitor"]
        self.assertIn("ds_network_dynamics", monitor["context"])
        self.assertIn("ds_dss_dynamic_plan", monitor["context"])
        self.assertEqual(monitor["schema_source"], "psa/src/dsa/monitor_output.rs")
        self.assertEqual(monitor["contract"]["source"], ["psa/src/dsa/monitor_output.rs"])

    def test_manifest_dynamic_result_contract_accepts_v13_and_legacy_v12(self):
        profile = next(
            item for item in self.read("case_projections.json")["profiles"]
            if item["id"] == "dss_dynamic_plan_v1"
        )
        native_v13 = list(profile["bundleResultMap"])
        managed_v12 = [
            {"source": name, "target": profile["bundleResultMap"][name], "binding": f"result_{name}"}
            for name in native_v13[:-1]
        ]

        self.validator.validate_manifest_result_contract(
            "v13", profile, native_v13, managed_v12
        )
        self.validator.validate_manifest_result_contract(
            "v12", profile, native_v13[:-1], managed_v12
        )

    def test_manifest_dynamic_result_contract_rejects_malformed_sets(self):
        profile = next(
            item for item in self.read("case_projections.json")["profiles"]
            if item["id"] == "dss_dynamic_plan_v1"
        )
        native_v13 = list(profile["bundleResultMap"])
        managed_v12 = [
            {"source": name, "target": profile["bundleResultMap"][name], "binding": f"result_{name}"}
            for name in native_v13[:-1]
        ]
        malformed = {
            "unknown": (native_v13[:-1] + ["unknown"], managed_v12),
            "missing": (native_v13[:-2], managed_v12[:-1]),
            "duplicate": (native_v13 + ["monitor"], managed_v12),
            "reordered": (native_v13, [managed_v12[1], managed_v12[0], *managed_v12[2:]]),
        }
        for name, (source_results, projected_results) in malformed.items():
            with self.subTest(name=name), self.assertRaises(AssertionError):
                self.validator.validate_manifest_result_contract(
                    name, profile, source_results, projected_results
                )

    def test_rejects_case_projection_contract_drift(self):
        catalog = self.read("case_projections.json")
        catalog["profiles"][0]["inputs"][0]["inputKind"] = "data"
        self.write("case_projections.json", catalog)
        self.assert_invalid("input contract differs")

    def test_enabled_case_projections_publish_complete_ordered_outputs(self):
        catalog = self.read("case_projections.json")
        for profile in catalog["profiles"]:
            if profile["compatibility"]["status"] != "enabled":
                continue
            outputs = profile.get("projectorOutputs")
            self.assertIsInstance(outputs, list, profile["id"])
            self.assertTrue(outputs, profile["id"])
            self.assertEqual(
                [item["position"] for item in outputs],
                list(range(1, len(outputs) + 1)),
                profile["id"],
            )
            facade_outputs = [item for item in outputs if item["disposition"] == "facade_input"]
            self.assertEqual(
                [item["targetInput"] for item in facade_outputs],
                [item["id"] for item in profile["inputs"]],
                profile["id"],
            )

    def test_rejects_case_projection_without_projector_outputs(self):
        catalog = self.read("case_projections.json")
        del catalog["profiles"][0]["projectorOutputs"]
        self.write("case_projections.json", catalog)
        self.assert_invalid("projectorOutputs must be a non-empty list")

    def test_rejects_case_projection_output_position_gap(self):
        catalog = self.read("case_projections.json")
        catalog["profiles"][0]["projectorOutputs"][1]["position"] = 3
        self.write("case_projections.json", catalog)
        self.assert_invalid("projectorOutputs positions must be contiguous")

    def test_rejects_case_projection_output_without_unique_facade_mapping(self):
        catalog = self.read("case_projections.json")
        catalog["profiles"][0]["projectorOutputs"][0]["targetInput"] = "line"
        self.write("case_projections.json", catalog)
        self.assert_invalid("differs from target input")

    def test_rejects_case_projection_diagnostic_as_facade_input(self):
        catalog = self.read("case_projections.json")
        profile = next(item for item in catalog["profiles"] if item["id"] == "dss_fault_study_v1")
        profile["projectorOutputs"][-1]["disposition"] = "facade_input"
        profile["projectorOutputs"][-1]["targetInput"] = "para"
        self.write("case_projections.json", catalog)
        self.assert_invalid("differs from target input")

    def test_execution_and_harmonic_plan_schemas_are_machine_complete(self):
        dataframes = self.read("dataframes.json")
        for section in ("execution_inputs", "execution_outputs", "harmonic_execution_inputs"):
            self.assertIn(section, dataframes)
            self.assertTrue(dataframes[section])
            for name, schema in dataframes[section].items():
                self.assertIsInstance(schema.get("contract"), dict, f"{section}.{name}")
                self.assertIn("columns", schema["contract"], f"{section}.{name}")

    def test_execution_plan_para_is_distinct_from_imported_network_para(self):
        dataframes = self.read("dataframes.json")
        self.assertIn("para", dataframes["execution_inputs"])

        execution_para = dataframes["execution_inputs"]["para"]
        execution_columns = {
            column["name"]: column for column in execution_para["contract"]["columns"]
        }
        self.assertEqual(
            set(execution_columns),
            {"line_model", "max_iter", "max_control_iter", "tol", "result"},
        )
        self.assertTrue(execution_columns["line_model"]["required"])
        self.assertFalse(execution_columns["result"]["nullable"])
        self.assertEqual(execution_para["contract"]["rowCount"], "one")
        self.assertNotIn("base_frequency_hz", execution_columns)
        self.assertIn(
            "base_frequency_hz",
            dataframes["frames"]["para"]["required_for_imported_network"],
        )

    def test_execution_plan_facades_and_profiles_use_execution_para(self):
        facades = self.read("study_facades.json")
        projections = self.read("case_projections.json")

        for function in (
            "ds_radial_execution_plan",
            "ds_newton_raphson_execution_plan",
        ):
            facade = self.facade(facades, function)
            para = next(item for item in facade["inputs"] if item["id"] == "para")
            self.assertEqual(para["schemaRef"], "execution_inputs.para")

        for profile_id in (
            "dss_execution_radial_v1",
            "dss_execution_newton_v1",
        ):
            profile = next(
                item for item in projections["profiles"] if item["id"] == profile_id
            )
            para_input = next(
                item for item in profile["inputs"] if item["id"] == "para"
            )
            para_output = next(
                item for item in profile["projectorOutputs"] if item["id"] == "para"
            )
            self.assertEqual(para_input["schemaRef"], "execution_inputs.para")
            self.assertEqual(para_output["schemaRef"], "execution_inputs.para")

    def test_rejects_invalid_recursive_base_schema_reference(self):
        catalog = self.read("dataframes.json")
        catalog["result_frames"]["harmonic_monitor_save"]["base_schema"] = "result_frames.missing"
        self.write("dataframes.json", catalog)
        self.assert_invalid("unknown base_schema")

    def test_rejects_circular_base_schema_reference(self):
        catalog = self.read("dataframes.json")
        catalog["result_frames"]["harmonic_monitor"]["base_schema"] = "result_frames.harmonic_monitor_save"
        catalog["result_frames"]["harmonic_monitor_save"]["base_schema"] = "result_frames.harmonic_monitor"
        self.write("dataframes.json", catalog)
        self.assert_invalid("circular base_schema")

    def test_rejects_base_schema_that_targets_an_array(self):
        catalog = self.read("dataframes.json")
        catalog["result_frames"]["harmonic_monitor_save"]["base_schema"] = "result_frames.harmonic_monitor.columns"
        self.write("dataframes.json", catalog)
        self.assert_invalid("base_schema.*schema object")

    def test_rejects_base_schema_that_targets_a_nested_field_map(self):
        catalog = self.read("dataframes.json")
        catalog["result_frames"]["harmonic_monitor_save"]["base_schema"] = "frames.bus.required"
        self.write("dataframes.json", catalog)
        self.assert_invalid("base_schema.*<catalogSection>.<schemaName>")

    def test_rejects_constraint_with_missing_required_field(self):
        catalog = self.read("study_facades.json")
        constraint = self.constraint(self.facade(catalog, "ds_radial_pf"), "ordered_unique_selection")
        del constraint["selector"]
        self.write("study_facades.json", catalog)
        self.assert_invalid("ordered_unique_selection.*selector")

    def test_rejects_constraint_field_with_wrong_type(self):
        catalog = self.read("study_facades.json")
        constraint = self.constraint(self.facade(catalog, "ds_radial_pf"), "ordered_unique_selection")
        constraint["minimum"] = "1"
        self.write("study_facades.json", catalog)
        self.assert_invalid("ordered_unique_selection.*minimum")

    def test_rejects_unknown_constraint_result(self):
        catalog = self.read("study_facades.json")
        constraint = self.constraint(self.facade(catalog, "ds_radial_pf"), "default_results")
        constraint["results"] = ["missing"]
        self.write("study_facades.json", catalog)
        self.assert_invalid("unknown results")

    def test_rejects_unknown_conditional_input(self):
        catalog = self.read("study_facades.json")
        constraint = self.constraint(self.facade(catalog, "ds_radial_qsts"), "conditional_unavailable")
        constraint["when"]["input"] = "missing"
        self.write("study_facades.json", catalog)
        self.assert_invalid("unknown input")

    def test_rejects_duplicate_default_constraint(self):
        catalog = self.read("study_facades.json")
        facade = self.facade(catalog, "ds_radial_pf")
        facade["resultConstraints"].append(copy.deepcopy(self.constraint(facade, "default_results")))
        self.write("study_facades.json", catalog)
        self.assert_invalid("exactly one default_results or fixed_results")

    def test_rejects_fixed_result_binding_count_mismatch(self):
        catalog = self.read("study_facades.json")
        constraint = self.constraint(self.facade(catalog, "ds_wls_se"), "fixed_results")
        constraint["bindingCount"] = 2
        self.write("study_facades.json", catalog)
        self.assert_invalid("fixed_results.*bindingCount")

    def test_rejects_non_boolean_conditional_non_empty(self):
        catalog = self.read("study_facades.json")
        constraint = self.constraint(self.facade(catalog, "ds_radial_qsts"), "conditional_unavailable")
        constraint["when"]["nonEmpty"] = "true"
        self.write("study_facades.json", catalog)
        self.assert_invalid("conditional_unavailable.*nonEmpty")

    def test_rejects_selector_on_non_struct_input(self):
        catalog = self.read("study_facades.json")
        constraint = self.constraint(self.facade(catalog, "ds_radial_qsts"), "ordered_unique_selection")
        constraint["selector"] = "fault.kind"
        self.write("study_facades.json", catalog)
        self.assert_invalid("selector.*Struct input")

    def test_rejects_selector_for_unknown_struct_field(self):
        catalog = self.read("study_facades.json")
        constraint = self.constraint(self.facade(catalog, "ds_radial_pf"), "ordered_unique_selection")
        constraint["selector"] = "para.missing"
        self.write("study_facades.json", catalog)
        self.assert_invalid("selector.*unknown schema field")

    def test_rejects_selector_with_nested_path(self):
        catalog = self.read("study_facades.json")
        constraint = self.constraint(self.facade(catalog, "ds_radial_pf"), "ordered_unique_selection")
        constraint["selector"] = "para.nonexistent.result"
        self.write("study_facades.json", catalog)
        self.assert_invalid("selector.*<structAlias>.<field>")

    def test_rejects_selectable_results_without_selection_constraint(self):
        catalog = self.read("study_facades.json")
        facade = self.facade(catalog, "ds_radial_pf")
        facade["resultConstraints"] = [
            item for item in facade["resultConstraints"] if item["type"] != "ordered_unique_selection"
        ]
        self.write("study_facades.json", catalog)
        self.assert_invalid("selectable result mode")

    def test_rejects_selection_constraint_with_fixed_results(self):
        catalog = self.read("study_facades.json")
        facade = self.facade(catalog, "ds_wls_se")
        facade["resultConstraints"].insert(
            0,
            {
                "type": "ordered_unique_selection",
                "selector": "para.result",
                "minimum": 1,
                "bindingCount": "selected_result_count",
            },
        )
        self.write("study_facades.json", catalog)
        self.assert_invalid("fixed result mode")

    def test_six_historical_model_schemas_have_complete_contracts(self):
        catalog = self.read("dataframes.json")
        for name in ("fuse", "recloser", "relay", "reactor", "transformer_impedance", "transformer_winding"):
            contract = catalog["frames"][name].get("contract")
            self.assertIsInstance(contract, dict, name)
            self.assertEqual(contract.get("kind"), "model", name)
            self.assertTrue(contract.get("columns"), name)
            self.assertIn("columnPolicy", contract, name)
            self.assertIn("rowCount", contract, name)

    def test_eight_struct_schemas_are_parameter_card_contracts(self):
        catalog = self.read("dataframes.json")
        references = (
            "frames.para",
            "frames.solution",
            "execution_inputs.para",
            "analysis_inputs.dynamic_spec",
            "analysis_inputs.fault_location_para",
            "analysis_inputs.harmonic_para",
            "analysis_inputs.monte_carlo_spec",
            "analysis_inputs.protection_para",
        )
        for reference in references:
            section, name = reference.split(".")
            contract = catalog[section][name]["contract"]
            self.assertEqual(contract["kind"], "struct", reference)
            self.assertIn(contract["rowCount"], ("zero", "one", "zero_or_one"), reference)
            for column in contract["columns"]:
                self.assertEqual(
                    set(column["card"]),
                    {"default", "unit", "range", "enum", "visibleWhen", "enabledWhen"},
                    f"{reference}.{column['name']}",
                )

    def test_non_struct_schemas_do_not_publish_parameter_cards(self):
        catalog = self.read("dataframes.json")
        for section, schemas in catalog.items():
            if not isinstance(schemas, dict):
                continue
            for name, schema in schemas.items():
                if not isinstance(schema, dict) or not isinstance(schema.get("contract"), dict):
                    continue
                contract = schema["contract"]
                if contract.get("kind") == "struct":
                    continue
                for column in contract.get("columns", []):
                    self.assertNotIn("card", column, f"{section}.{name}.{column.get('name')}")

    def test_rejects_parameter_card_on_non_struct_schema(self):
        catalog = self.read("dataframes.json")
        card = copy.deepcopy(catalog["frames"]["para"]["contract"]["columns"][0]["card"])
        catalog["frames"]["bus"]["contract"]["columns"][0]["card"] = card
        self.write("dataframes.json", catalog)
        self.assert_invalid(r"non-Struct schema cannot declare card.*frames\.bus")

    def test_rejects_struct_column_without_complete_parameter_card(self):
        catalog = self.read("dataframes.json")
        del catalog["frames"]["para"]["contract"]["columns"][0]["card"]["range"]
        self.write("dataframes.json", catalog)
        self.assert_invalid(r"Struct card metadata incomplete.*frames\.para")

    def test_contiguous_zero_based_rule_is_published_for_unsigned_fields(self):
        catalog = self.read("dataframes.json")
        for reference, field in (
            ("execution_inputs.execution_epoch", "epoch_index"),
            ("harmonic_snapshot_inputs.snapshot", "snapshot_index"),
        ):
            section, name = reference.split(".")
            rules = catalog[section][name]["contract"]["rules"]
            self.assertIn({"type": "contiguous_zero_based", "field": field}, rules)

    def test_rejects_unknown_machine_rule(self):
        catalog = self.read("dataframes.json")
        catalog["frames"]["bus"]["contract"]["rules"].append({"type": "prose_rule"})
        self.write("dataframes.json", catalog)
        self.assert_invalid(r"unsupported rule type prose_rule.*frames\.bus")

    def test_rejects_malformed_contiguous_zero_based_rule(self):
        catalog = self.read("dataframes.json")
        rule = catalog["harmonic_snapshot_inputs"]["snapshot"]["contract"]["rules"][0]
        rule["start"] = 0
        self.write("dataframes.json", catalog)
        self.assert_invalid(r"contiguous_zero_based.*unknown fields.*start")

    def test_rejects_contiguous_zero_based_rule_on_non_unsigned_field(self):
        catalog = self.read("dataframes.json")
        rule = catalog["harmonic_snapshot_inputs"]["snapshot"]["contract"]["rules"][0]
        rule["field"] = "time_s"
        self.write("dataframes.json", catalog)
        self.assert_invalid(r"contiguous_zero_based.*unsigned integer.*time_s")

    def test_all_facade_result_schemas_resolve_to_complete_dtype_contracts(self):
        dataframes = self.read("dataframes.json")
        facades = self.read("study_facades.json")
        for facade in facades["facades"]:
            for result in facade["results"]:
                columns = self.validator.resolve_contract_columns(dataframes, result["schemaRef"])
                self.assertTrue(columns, result["schemaRef"])
                for column in columns:
                    self.assertIsInstance(column.get("dtype"), str, f"{result['schemaRef']}.{column.get('name')}")
                    self.assertIn("nullable", column, f"{result['schemaRef']}.{column.get('name')}")
                    self.assertIn("unit", column, f"{result['schemaRef']}.{column.get('name')}")

    def test_branch_flow_contract_matches_runtime_schema_through_execution_inheritance(self):
        dataframes = self.read("dataframes.json")
        base = [
            ("branch", "u32", True, False, "not_applicable"),
            ("element", "str", True, True, "not_applicable"),
            ("bus_from", "str", True, False, "not_applicable"),
            ("bus_to", "str", True, False, "not_applicable"),
            ("phase_from", "str", True, False, "not_applicable"),
            ("phase_to", "str", True, False, "not_applicable"),
            ("v_from_re", "f64", True, False, "V"),
            ("v_from_im", "f64", True, False, "V"),
            ("v_to_re", "f64", True, False, "V"),
            ("v_to_im", "f64", True, False, "V"),
            ("i_from_re", "f64", True, False, "A"),
            ("i_from_im", "f64", True, False, "A"),
            ("i_from_mag", "f64", True, False, "A"),
            ("i_to_re", "f64", True, False, "A"),
            ("i_to_im", "f64", True, False, "A"),
            ("i_to_mag", "f64", True, False, "A"),
            ("p_from", "f64", True, False, "W"),
            ("q_from", "f64", True, False, "var"),
            ("p_to", "f64", True, False, "W"),
            ("q_to", "f64", True, False, "var"),
            ("p_loss", "f64", True, False, "W"),
            ("q_loss", "f64", True, False, "var"),
            ("winding_1_phase", "u32", True, True, "not_applicable"),
            ("winding_1_v_re", "f64", True, True, "not_applicable"),
            ("winding_1_v_im", "f64", True, True, "not_applicable"),
            ("winding_1_i_re", "f64", True, True, "not_applicable"),
            ("winding_1_i_im", "f64", True, True, "not_applicable"),
            ("winding_1_tap", "f64", True, True, "not_applicable"),
            ("winding_2_phase", "u32", True, True, "not_applicable"),
            ("winding_2_v_re", "f64", True, True, "not_applicable"),
            ("winding_2_v_im", "f64", True, True, "not_applicable"),
            ("winding_2_i_re", "f64", True, True, "not_applicable"),
            ("winding_2_i_im", "f64", True, True, "not_applicable"),
            ("winding_2_tap", "f64", True, True, "not_applicable"),
        ]
        qsts = [
            ("time", "f64", True, False, "s"),
            ("load_multiplier", "f64", True, False, "pu"),
            ("generation_multiplier", "f64", True, False, "pu"),
        ]
        execution = [
            ("epoch_index", "u64", True, False, "not_applicable"),
            ("solve_event_sequence", "u64", True, False, "not_applicable"),
        ]

        def metadata(reference):
            return [
                (
                    column["name"],
                    column["dtype"],
                    column["required"],
                    column["nullable"],
                    column["unit"],
                )
                for column in self.validator.resolve_contract_columns(dataframes, reference)
            ]

        self.assertEqual(metadata("result_frames.branch_flow"), base)
        self.assertEqual(
            dataframes["qsts_outputs"]["branch_flow"]["base_schema"],
            "result_frames.branch_flow",
        )
        self.assertEqual(metadata("qsts_outputs.branch_flow"), base + qsts)
        self.assertEqual(
            dataframes["execution_outputs"]["branch_flow"]["base_schema"],
            "qsts_outputs.branch_flow",
        )
        self.assertEqual(
            metadata("execution_outputs.branch_flow"),
            base + qsts + execution,
        )

    def test_rejects_undeclared_dtype_with_affected_facade_and_field(self):
        catalog = self.read("dataframes.json")
        catalog["frames"]["bus"]["contract"]["columns"][0]["dtype"] = "decimal128"
        self.write("dataframes.json", catalog)
        self.assert_invalid(r"undeclared dtype decimal128.*frames\.bus\.id.*ds_radial_pf")

    def test_dtype_grammar_declares_runtime_u8_execution_field(self):
        catalog = self.read("dataframes.json")
        self.assertEqual(
            catalog["dtype_grammar"]["primitives"]["u8"],
            {
                "arrow": "UInt8",
                "parquet": "INT32/UINT_8",
                "logical": "Unsigned 8-bit integer",
            },
        )
        target_terminal = next(
            column
            for column in catalog["result_frames"]["execution_trace"]["contract"]["columns"]
            if column["name"] == "target_terminal"
        )
        self.assertEqual(target_terminal["dtype"], "u8")

    def test_rejects_malformed_dtype_expression(self):
        catalog = self.read("dataframes.json")
        catalog["frames"]["bus"]["contract"]["columns"][0]["dtype"] = "list<f64"
        self.write("dataframes.json", catalog)
        self.assert_invalid(r"invalid dtype syntax list<f64.*frames\.bus\.id")

    def test_rejects_base_schema_conflicting_column_override(self):
        catalog = self.read("dataframes.json")
        inherited = catalog["result_frames"]["bus_voltage"]["contract"]["columns"][0].copy()
        catalog["qsts_outputs"]["bus_voltage"]["contract"]["columns"].append(inherited)
        self.write("dataframes.json", catalog)
        self.assert_invalid(r"base_schema column conflict.*qsts_outputs\.bus_voltage.*bus")

    def test_rejects_prose_only_referenced_schema(self):
        catalog = self.read("dataframes.json")
        del catalog["frames"]["relay"]["contract"]
        catalog["frames"]["relay"]["note"] = "See the Rust parser."
        self.write("dataframes.json", catalog)
        self.assert_invalid(r"prose-only or incomplete schema.*frames\.relay.*ds_radial_qsts")

    def test_real_catalog_has_zero_completeness_diagnostics(self):
        dataframes = self.read("dataframes.json")
        facades = self.read("study_facades.json")
        stats = self.validator.validate_catalog_contracts(dataframes, facades)
        self.assertEqual(stats["facades"], len(facades["facades"]))
        self.assertEqual(stats["facades"], 19)
        self.assertEqual(stats["models"], 105)
        self.assertEqual(stats["structs"], 10)
        self.assertEqual(stats["results"], 128)
        self.assertEqual(stats["diagnostics"], 0)

    def test_rejects_discriminated_schema_without_variants(self):
        catalog = self.read("dataframes.json")
        del catalog["frames"]["relay"]["contract"]["variants"]
        self.write("dataframes.json", catalog)
        self.assert_invalid(r"discriminated schema requires discriminator and non-empty variants.*frames\.relay")

    def test_rejects_variant_with_unknown_field(self):
        catalog = self.read("dataframes.json")
        catalog["analysis_inputs"]["dynamic_event"]["contract"]["variants"][0]["required"].append("missing")
        self.write("dataframes.json", catalog)
        self.assert_invalid(r"variant fault_on references unknown fields.*missing.*dynamic_event")

    def test_display_text_is_not_required_by_schema_execution_contract(self):
        catalog = self.read("dataframes.json")
        catalog["frames"]["bus"]["note"] = "display-only text may change"
        self.write("dataframes.json", catalog)
        with mock.patch.multiple(
            self.validator,
            FUNCTIONS=self.catalog_dir / "functions.json",
            DATAFRAMES=self.catalog_dir / "dataframes.json",
            STUDY_FACADES=self.catalog_dir / "study_facades.json",
            CASE_PROJECTIONS=self.catalog_dir / "case_projections.json",
        ):
            with contextlib.redirect_stdout(io.StringIO()):
                self.assertEqual(self.validator.main(), 0)


if __name__ == "__main__":
    unittest.main()
