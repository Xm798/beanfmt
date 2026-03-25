"""Tests for beanfmt Python config file support."""

import os
import tempfile

import pytest

import beanfmt


# ---------------------------------------------------------------------------
# parse_config
# ---------------------------------------------------------------------------


class TestParseConfig:
    def test_valid_toml(self):
        opts = beanfmt.parse_config('indent = 2\ncurrency_column = 80\n')
        assert repr(opts).startswith("Options(indent=2, currency_column=80,")

    def test_invalid_toml(self):
        with pytest.raises(ValueError, match="invalid config"):
            beanfmt.parse_config("indent = ???")

    def test_empty_returns_defaults(self):
        opts = beanfmt.parse_config("")
        default = beanfmt.Options()
        assert repr(opts) == repr(default)

    def test_sort_bool(self):
        opts = beanfmt.parse_config("sort = true\n")
        assert "'asc'" in repr(opts)

    def test_partial_config(self):
        opts = beanfmt.parse_config("indent = 8\n")
        assert "indent=8" in repr(opts)
        # Other fields should be defaults
        assert "currency_column=70" in repr(opts)


# ---------------------------------------------------------------------------
# load_project_config
# ---------------------------------------------------------------------------


class TestLoadProjectConfig:
    def test_find_config_in_dir(self):
        with tempfile.TemporaryDirectory() as d:
            cfg = os.path.join(d, ".beanfmt.toml")
            with open(cfg, "w") as f:
                f.write("indent = 2\n")
            opts = beanfmt.load_project_config(d)
            assert "indent=2" in repr(opts)

    def test_traverse_upward(self):
        with tempfile.TemporaryDirectory() as d:
            sub = os.path.join(d, "a", "b")
            os.makedirs(sub)
            cfg = os.path.join(d, ".beanfmt.toml")
            with open(cfg, "w") as f:
                f.write("indent = 2\n")
            opts = beanfmt.load_project_config(sub)
            assert "indent=2" in repr(opts)

    def test_no_config_returns_defaults(self):
        with tempfile.TemporaryDirectory() as d:
            opts = beanfmt.load_project_config(d)
            default = beanfmt.Options()
            assert repr(opts) == repr(default)

    def test_nonexistent_dir(self):
        with pytest.raises(OSError, match="not a directory"):
            beanfmt.load_project_config("/nonexistent/path/xyz")

    def test_invalid_toml_raises(self):
        with tempfile.TemporaryDirectory() as d:
            cfg = os.path.join(d, ".beanfmt.toml")
            with open(cfg, "w") as f:
                f.write("indent = ???\n")
            with pytest.raises(ValueError):
                beanfmt.load_project_config(d)

    def test_prefers_dotfile(self):
        with tempfile.TemporaryDirectory() as d:
            with open(os.path.join(d, "beanfmt.toml"), "w") as f:
                f.write("indent = 8\n")
            with open(os.path.join(d, ".beanfmt.toml"), "w") as f:
                f.write("indent = 2\n")
            opts = beanfmt.load_project_config(d)
            assert "indent=2" in repr(opts)


# ---------------------------------------------------------------------------
# format_file with config param
# ---------------------------------------------------------------------------


SAMPLE_BEAN = """\
2024-01-01 open Assets:Bank    USD
2024-01-01 * "Test"
    Assets:Bank    1000.00 USD
    Income:Salary
"""


class TestFormatFileConfig:
    def _write_bean(self, d, content=SAMPLE_BEAN):
        path = os.path.join(d, "test.bean")
        with open(path, "w") as f:
            f.write(content)
        return path

    def test_config_none_uses_defaults(self):
        with tempfile.TemporaryDirectory() as d:
            path = self._write_bean(d)
            result = beanfmt.format_file(path, config=None)
            assert "Assets:Bank" in result

    def test_config_false_uses_defaults(self):
        with tempfile.TemporaryDirectory() as d:
            path = self._write_bean(d)
            result = beanfmt.format_file(path, config=False)
            assert "Assets:Bank" in result

    def test_config_true_discovers(self):
        with tempfile.TemporaryDirectory() as d:
            with open(os.path.join(d, ".beanfmt.toml"), "w") as f:
                f.write("indent = 8\n")
            path = self._write_bean(d)
            result = beanfmt.format_file(path, config=True)
            # With indent=8, postings should be indented by 8 spaces
            for line in result.splitlines():
                if line.strip().startswith("Assets:Bank") and not line.startswith("2"):
                    assert line.startswith("        ")  # 8 spaces

    def test_config_true_no_config_file(self):
        """config=True with no config file should use defaults (not error)."""
        with tempfile.TemporaryDirectory() as d:
            path = self._write_bean(d)
            result = beanfmt.format_file(path, config=True)
            assert "Assets:Bank" in result

    def test_config_path_string(self):
        with tempfile.TemporaryDirectory() as d:
            cfg = os.path.join(d, "my-config.toml")
            with open(cfg, "w") as f:
                f.write("indent = 8\n")
            path = self._write_bean(d)
            result = beanfmt.format_file(path, config=cfg)
            for line in result.splitlines():
                if line.strip().startswith("Assets:Bank") and not line.startswith("2"):
                    assert line.startswith("        ")  # 8 spaces

    def test_config_path_invalid_toml(self):
        with tempfile.TemporaryDirectory() as d:
            cfg = os.path.join(d, "bad.toml")
            with open(cfg, "w") as f:
                f.write("indent = ???\n")
            path = self._write_bean(d)
            with pytest.raises(ValueError):
                beanfmt.format_file(path, config=cfg)

    def test_config_path_nonexistent(self):
        with tempfile.TemporaryDirectory() as d:
            path = self._write_bean(d)
            with pytest.raises(OSError):
                beanfmt.format_file(path, config="/nonexistent/config.toml")

    def test_kwargs_override_config(self):
        """Individual kwargs should override config file values."""
        with tempfile.TemporaryDirectory() as d:
            with open(os.path.join(d, ".beanfmt.toml"), "w") as f:
                f.write("indent = 8\n")
            path = self._write_bean(d)
            # config says indent=8, but kwarg says indent=2
            result = beanfmt.format_file(path, config=True, indent=2)
            for line in result.splitlines():
                if line.strip().startswith("Assets:Bank") and not line.startswith("2"):
                    assert line.startswith("  ")  # 2 spaces
                    assert not line.startswith("        ")  # not 8 spaces

    def test_config_true_with_invalid_file_raises(self):
        with tempfile.TemporaryDirectory() as d:
            with open(os.path.join(d, ".beanfmt.toml"), "w") as f:
                f.write("indent = ???\n")
            path = self._write_bean(d)
            with pytest.raises(ValueError):
                beanfmt.format_file(path, config=True)


# ---------------------------------------------------------------------------
# format_recursive with config param
# ---------------------------------------------------------------------------


class TestFormatRecursiveConfig:
    def test_config_true(self):
        with tempfile.TemporaryDirectory() as d:
            with open(os.path.join(d, ".beanfmt.toml"), "w") as f:
                f.write("indent = 8\n")
            path = os.path.join(d, "main.bean")
            with open(path, "w") as f:
                f.write(SAMPLE_BEAN)
            results = beanfmt.format_recursive(path, config=True)
            assert len(results) >= 1
            _, content = results[0]
            for line in content.splitlines():
                if line.strip().startswith("Assets:Bank") and not line.startswith("2"):
                    assert line.startswith("        ")  # 8 spaces

    def test_config_none(self):
        with tempfile.TemporaryDirectory() as d:
            path = os.path.join(d, "main.bean")
            with open(path, "w") as f:
                f.write(SAMPLE_BEAN)
            results = beanfmt.format_recursive(path, config=None)
            assert len(results) >= 1
