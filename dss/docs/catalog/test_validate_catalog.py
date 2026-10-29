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
        for name in ("functions.json", "dataframes.json", "study_facades.json"):
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

    def constraint(self, facade, kind):
        return next(item for item in facade["resultConstraints"] if item["type"] == kind)

    def assert_invalid(self, pattern):
        with mock.patch.multiple(
            self.validator,
            FUNCTIONS=self.catalog_dir / "functions.json",
            DATAFRAMES=self.catalog_dir / "dataframes.json",
            STUDY_FACADES=self.catalog_dir / "study_facades.json",
        ):
            with contextlib.redirect_stdout(io.StringIO()):
                with self.assertRaisesRegex(AssertionError, pattern):
                    self.validator.main()

    def test_harmonic_monitor_save_uses_full_base_schema(self):
        catalog = self.read("dataframes.json")
        self.assertEqual(
            catalog["result_frames"]["harmonic_monitor_save"]["base_schema"],
            "result_frames.harmonic_monitor",
        )

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

    def test_seven_struct_schemas_are_parameter_card_contracts(self):
        catalog = self.read("dataframes.json")
        references = (
            "frames.para",
            "frames.solution",
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

    def test_rejects_undeclared_dtype_with_affected_facade_and_field(self):
        catalog = self.read("dataframes.json")
        catalog["frames"]["bus"]["contract"]["columns"][0]["dtype"] = "decimal128"
        self.write("dataframes.json", catalog)
        self.assert_invalid(r"undeclared dtype decimal128.*frames\.bus\.id.*ds_radial_pf")

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
        self.assertEqual(stats["facades"], 15)
        self.assertEqual(stats["models"], 28)
        self.assertEqual(stats["structs"], 7)
        self.assertEqual(stats["results"], 85)
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
        ):
            with contextlib.redirect_stdout(io.StringIO()):
                self.assertEqual(self.validator.main(), 0)


if __name__ == "__main__":
    unittest.main()
