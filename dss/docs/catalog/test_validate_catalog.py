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


if __name__ == "__main__":
    unittest.main()
