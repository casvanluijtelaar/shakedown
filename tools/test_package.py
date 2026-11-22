"""Tests of tools/package.py (DST-01, NFR-LG-04). Run: python -m unittest discover -s tools -p "test_*.py"

The licence test runs cargo metadata on this workspace, so it needs cargo and the crates fetched.
"""

import hashlib
import os
import sys
import tempfile
import unittest
import zipfile

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import package  # noqa: E402


class DST01Tag(unittest.TestCase):
    def test_dst01_tag_of_the_workspace_version_passes(self):
        self.assertIsNone(package.check_tag("v0.2.0-rc.1", "0.2.0-rc.1"))

    def test_dst01_tag_of_another_version_is_refused_naming_both(self):
        reason = package.check_tag("v0.2.1", "0.2.0")
        self.assertIn("v0.2.1", reason)
        self.assertIn("0.2.0", reason)

    def test_dst01_tag_without_v_is_refused(self):
        self.assertIsNotNone(package.check_tag("0.2.0", "0.2.0"))

    def test_dst01_prerelease_part(self):
        self.assertTrue(package.is_prerelease("0.2.0-rc.1"))
        self.assertFalse(package.is_prerelease("0.2.0"))

    def test_workspace_version_is_read_from_cargo_toml(self):
        self.assertRegex(package.workspace_version(), r"^\d+\.\d+\.\d+")


class DST01Zip(unittest.TestCase):
    def test_dst01_sha256_line_is_hash_two_spaces_and_name(self):
        with tempfile.TemporaryDirectory() as d:
            path = os.path.join(d, package.ZIP_NAME)
            package.write_zip(path, [("shakedown.exe", b"exe"), ("@shakedown/addons/x.pbo", b"pbo")])
            line = package.sha256_line(path)
            with open(path, "rb") as f:
                want = hashlib.sha256(f.read()).hexdigest()
            self.assertEqual(line, f"{want}  {package.ZIP_NAME}\n")
            with zipfile.ZipFile(path) as z:
                self.assertEqual(z.namelist(), ["shakedown.exe", "@shakedown/addons/x.pbo"])


class NFRLG04Licences(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.metadata = package.cargo_metadata()

    def test_nfr_lg_04_shipped_packages_hold_the_binaries_dependencies_and_no_workspace_crate(self):
        names = {p["name"] for p in package.shipped_packages(self.metadata)}
        self.assertIn("arma-rs", names)  # the extension's
        self.assertIn("rmcp", names)  # shakedown.exe's
        self.assertIn("serde", names)
        self.assertFalse(names & {"shakedown", "shakedown-ext", "shakedown-protocol", "skdfx-ext"})
        self.assertNotIn("jsonschema", names)  # a dev-dependency, compiled into no shipped binary

    def test_nfr_lg_04_listing_names_every_package_with_its_licence_text(self):
        text = package.third_party_licenses(self.metadata)
        for p in package.shipped_packages(self.metadata):
            self.assertIn(f"\n{p['name']} {p['version']}\nLicence: ", text)
        self.assertIn("Apache License", text)
        self.assertIn("Permission is hereby granted", text)
        self.assertIn("https://github.com/BrettMayson/arma-rs", text)

    def test_nfr_lg_04_package_with_no_licence_file_or_note_stops_the_listing(self):
        with tempfile.TemporaryDirectory() as d:
            manifest = os.path.join(d, "Cargo.toml")
            open(manifest, "w").close()
            metadata = {
                "packages": [
                    {"id": "root", "name": "shakedown", "version": "0", "license": None, "manifest_path": manifest},
                    {"id": "dep", "name": "bare", "version": "1.0.0", "license": "MIT", "manifest_path": manifest},
                ],
                "resolve": {"nodes": [
                    {"id": "root", "deps": [{"pkg": "dep", "dep_kinds": [{"kind": None}]}]},
                    {"id": "dep", "deps": []},
                ]},
                "workspace_members": ["root"],
            }
            with self.assertRaises(SystemExit) as stop:
                package.third_party_licenses(metadata)
            self.assertIn("bare 1.0.0", str(stop.exception))


if __name__ == "__main__":
    unittest.main()
