use pyo3::exceptions::{PyOSError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyBool;

use crate::options::{Options, SortOrder, SortableDirective, ThousandsSeparator, TimelessPosition};

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn parse_thousands(s: &str) -> PyResult<ThousandsSeparator> {
    match s.to_ascii_lowercase().as_str() {
        "add" => Ok(ThousandsSeparator::Add),
        "remove" => Ok(ThousandsSeparator::Remove),
        "keep" => Ok(ThousandsSeparator::Keep),
        other => Err(PyValueError::new_err(format!(
            "invalid thousands_separator: {other:?}"
        ))),
    }
}

fn parse_sort(obj: &Bound<'_, PyAny>) -> PyResult<SortOrder> {
    if obj.is_instance_of::<PyBool>() {
        let v: bool = obj.extract()?;
        return Ok(if v { SortOrder::Asc } else { SortOrder::Off });
    }
    let s: String = obj.extract()?;
    s.parse().map_err(|msg: String| PyValueError::new_err(msg))
}

fn parse_timeless(s: &str) -> PyResult<TimelessPosition> {
    s.parse().map_err(|msg: String| PyValueError::new_err(msg))
}

fn parse_sort_exclude(items: Vec<String>) -> PyResult<Vec<SortableDirective>> {
    items
        .iter()
        .map(|s| s.parse().map_err(|msg: String| PyValueError::new_err(msg)))
        .collect()
}

#[allow(clippy::too_many_arguments)]
fn build_options(
    indent: Option<usize>,
    currency_column: Option<usize>,
    cost_column: Option<usize>,
    thousands_separator: Option<String>,
    spaces_in_braces: Option<bool>,
    fixed_cjk_width: Option<bool>,
    sort: Option<SortOrder>,
    sort_timeless: Option<TimelessPosition>,
    sort_exclude: Option<Vec<SortableDirective>>,
) -> PyResult<Options> {
    let defaults = Options::default();
    let ts = match thousands_separator {
        Some(s) => parse_thousands(&s)?,
        None => defaults.thousands_separator,
    };
    Ok(Options {
        indent: indent.unwrap_or(defaults.indent),
        currency_column: currency_column.unwrap_or(defaults.currency_column),
        cost_column: cost_column.unwrap_or(defaults.cost_column),
        thousands_separator: ts,
        spaces_in_braces: spaces_in_braces.unwrap_or(defaults.spaces_in_braces),
        fixed_cjk_width: fixed_cjk_width.unwrap_or(defaults.fixed_cjk_width),
        sort: sort.unwrap_or(defaults.sort),
        sort_timeless: sort_timeless.unwrap_or(defaults.sort_timeless),
        sort_exclude: sort_exclude.unwrap_or(defaults.sort_exclude),
    })
}

/// Reusable formatting configuration object.
#[pyclass(name = "Options")]
#[derive(Debug, Clone)]
struct PyOptions {
    inner: Options,
}

#[pymethods]
impl PyOptions {
    #[new]
    #[pyo3(signature = (
        indent = None,
        currency_column = None,
        cost_column = None,
        thousands_separator = None,
        spaces_in_braces = None,
        fixed_cjk_width = None,
        sort = None,
        sort_timeless = None,
        sort_exclude = None,
    ))]
    #[allow(clippy::too_many_arguments)]
    fn new(
        indent: Option<usize>,
        currency_column: Option<usize>,
        cost_column: Option<usize>,
        thousands_separator: Option<String>,
        spaces_in_braces: Option<bool>,
        fixed_cjk_width: Option<bool>,
        sort: Option<&Bound<'_, PyAny>>,
        sort_timeless: Option<String>,
        sort_exclude: Option<Vec<String>>,
    ) -> PyResult<Self> {
        let sort = sort.map(parse_sort).transpose()?;
        let sort_timeless = sort_timeless.map(|s| parse_timeless(&s)).transpose()?;
        let sort_exclude = sort_exclude.map(parse_sort_exclude).transpose()?;
        let inner = build_options(
            indent,
            currency_column,
            cost_column,
            thousands_separator,
            spaces_in_braces,
            fixed_cjk_width,
            sort,
            sort_timeless,
            sort_exclude,
        )?;
        Ok(Self { inner })
    }

    fn __repr__(&self) -> String {
        let o = &self.inner;
        let ts = match o.thousands_separator {
            ThousandsSeparator::Add => "add",
            ThousandsSeparator::Remove => "remove",
            ThousandsSeparator::Keep => "keep",
        };
        let sort = match o.sort {
            SortOrder::Off => "'off'",
            SortOrder::Asc => "'asc'",
            SortOrder::Desc => "'desc'",
        };
        let sort_timeless = match o.sort_timeless {
            TimelessPosition::Begin => "'begin'",
            TimelessPosition::End => "'end'",
            TimelessPosition::Keep => "'keep'",
        };
        let sort_exclude: Vec<String> = o.sort_exclude.iter().map(|d| format!("'{d}'")).collect();
        let sort_exclude_str = format!("[{}]", sort_exclude.join(", "));
        format!(
            "Options(indent={}, currency_column={}, cost_column={}, \
             thousands_separator='{}', spaces_in_braces={}, fixed_cjk_width={}, sort={}, \
             sort_timeless={}, sort_exclude={})",
            o.indent,
            o.currency_column,
            o.cost_column,
            ts,
            if o.spaces_in_braces { "True" } else { "False" },
            if o.fixed_cjk_width { "True" } else { "False" },
            sort,
            sort_timeless,
            sort_exclude_str,
        )
    }
}

/// Interpret the `config` parameter:
/// - None -> FileConfig::default() (no config loading)
/// - True -> auto-discover from file's parent directory
/// - False -> FileConfig::default() (no config loading)
/// - str  -> load specific config file path
fn resolve_config_param(
    config: Option<&Bound<'_, PyAny>>,
    file_path: &str,
) -> PyResult<crate::config::FileConfig> {
    let Some(val) = config else {
        return Ok(crate::config::FileConfig::default());
    };

    // Check PyBool before string extraction (bool is a subclass of str in Python)
    if val.is_instance_of::<PyBool>() {
        let enabled: bool = val.extract()?;
        if !enabled {
            return Ok(crate::config::FileConfig::default());
        }
        let p = std::path::Path::new(file_path);
        let dir = p.parent().unwrap_or(p);
        crate::config::find_project_config_strict(dir).map_err(PyValueError::new_err)
    } else {
        let config_path: String = val.extract()?;
        let content = std::fs::read_to_string(&config_path)
            .map_err(|e| PyOSError::new_err(format!("{config_path}: {e}")))?;
        crate::config::FileConfig::parse(&content)
            .map_err(|e| PyValueError::new_err(format!("{config_path}: {e}")))
    }
}

/// Resolve an `Options` value from an optional `PyOptions` object plus kwargs.
/// If both `options` and individual kwargs are provided, kwargs override.
#[allow(clippy::too_many_arguments)]
fn resolve_options(
    options: Option<&PyOptions>,
    base_config: crate::config::FileConfig,
    indent: Option<usize>,
    currency_column: Option<usize>,
    cost_column: Option<usize>,
    thousands_separator: Option<String>,
    spaces_in_braces: Option<bool>,
    fixed_cjk_width: Option<bool>,
    sort: Option<SortOrder>,
    sort_timeless: Option<TimelessPosition>,
    sort_exclude: Option<Vec<SortableDirective>>,
) -> PyResult<Options> {
    let base = match options {
        Some(o) => o.inner.clone(),
        None => base_config.into_options(),
    };

    let ts = match thousands_separator {
        Some(s) => parse_thousands(&s)?,
        None => base.thousands_separator,
    };

    Ok(Options {
        indent: indent.unwrap_or(base.indent),
        currency_column: currency_column.unwrap_or(base.currency_column),
        cost_column: cost_column.unwrap_or(base.cost_column),
        thousands_separator: ts,
        spaces_in_braces: spaces_in_braces.unwrap_or(base.spaces_in_braces),
        fixed_cjk_width: fixed_cjk_width.unwrap_or(base.fixed_cjk_width),
        sort: sort.unwrap_or(base.sort),
        sort_timeless: sort_timeless.unwrap_or(base.sort_timeless),
        sort_exclude: sort_exclude.unwrap_or(base.sort_exclude),
    })
}

/// Parse a TOML config string and return an Options object.
/// Raises ValueError on invalid TOML.
#[pyfunction]
fn parse_config(content: &str) -> PyResult<PyOptions> {
    let config = crate::config::FileConfig::parse(content)
        .map_err(|e| PyValueError::new_err(format!("invalid config: {e}")))?;
    Ok(PyOptions {
        inner: config.into_options(),
    })
}

/// Load project config by searching upward from the given directory.
/// Returns an Options object with defaults filled in.
/// Raises ValueError if config file exists but contains invalid TOML.
/// Raises OSError if the directory doesn't exist.
#[pyfunction]
fn load_project_config(dir: &str) -> PyResult<PyOptions> {
    let path = std::path::Path::new(dir);
    if !path.is_dir() {
        return Err(PyOSError::new_err(format!("{dir}: not a directory")));
    }
    let config = crate::config::find_project_config_strict(path).map_err(PyValueError::new_err)?;
    Ok(PyOptions {
        inner: config.into_options(),
    })
}

/// Format a beancount string and return the formatted output.
#[pyfunction]
#[pyo3(signature = (
    input,
    options = None,
    indent = None,
    currency_column = None,
    cost_column = None,
    thousands_separator = None,
    spaces_in_braces = None,
    fixed_cjk_width = None,
    sort = None,
    sort_timeless = None,
    sort_exclude = None,
))]
#[allow(clippy::too_many_arguments)]
fn format(
    input: &str,
    options: Option<&PyOptions>,
    indent: Option<usize>,
    currency_column: Option<usize>,
    cost_column: Option<usize>,
    thousands_separator: Option<String>,
    spaces_in_braces: Option<bool>,
    fixed_cjk_width: Option<bool>,
    sort: Option<&Bound<'_, PyAny>>,
    sort_timeless: Option<String>,
    sort_exclude: Option<Vec<String>>,
) -> PyResult<String> {
    let sort = sort.map(parse_sort).transpose()?;
    let sort_timeless = sort_timeless.map(|s| parse_timeless(&s)).transpose()?;
    let sort_exclude = sort_exclude.map(parse_sort_exclude).transpose()?;
    let opts = resolve_options(
        options,
        crate::config::FileConfig::default(),
        indent,
        currency_column,
        cost_column,
        thousands_separator,
        spaces_in_braces,
        fixed_cjk_width,
        sort,
        sort_timeless,
        sort_exclude,
    )?;
    Ok(crate::format(input, &opts))
}

/// Format a single beancount file by path and return the formatted content.
#[pyfunction]
#[pyo3(signature = (
    path,
    config = None,
    options = None,
    indent = None,
    currency_column = None,
    cost_column = None,
    thousands_separator = None,
    spaces_in_braces = None,
    fixed_cjk_width = None,
    sort = None,
    sort_timeless = None,
    sort_exclude = None,
))]
#[allow(clippy::too_many_arguments)]
fn format_file(
    path: &str,
    config: Option<&Bound<'_, PyAny>>,
    options: Option<&PyOptions>,
    indent: Option<usize>,
    currency_column: Option<usize>,
    cost_column: Option<usize>,
    thousands_separator: Option<String>,
    spaces_in_braces: Option<bool>,
    fixed_cjk_width: Option<bool>,
    sort: Option<&Bound<'_, PyAny>>,
    sort_timeless: Option<String>,
    sort_exclude: Option<Vec<String>>,
) -> PyResult<String> {
    let sort = sort.map(parse_sort).transpose()?;
    let sort_timeless = sort_timeless.map(|s| parse_timeless(&s)).transpose()?;
    let sort_exclude = sort_exclude.map(parse_sort_exclude).transpose()?;
    let base_config = resolve_config_param(config, path)?;
    let opts = resolve_options(
        options,
        base_config,
        indent,
        currency_column,
        cost_column,
        thousands_separator,
        spaces_in_braces,
        fixed_cjk_width,
        sort,
        sort_timeless,
        sort_exclude,
    )?;
    let content =
        std::fs::read_to_string(path).map_err(|e| PyOSError::new_err(format!("{path}: {e}")))?;
    Ok(crate::format(&content, &opts))
}

/// Recursively format a beancount file and all its includes.
/// Returns a list of (absolute_path, formatted_content) tuples.
#[pyfunction]
#[pyo3(signature = (
    path,
    config = None,
    options = None,
    indent = None,
    currency_column = None,
    cost_column = None,
    thousands_separator = None,
    spaces_in_braces = None,
    fixed_cjk_width = None,
    sort = None,
    sort_timeless = None,
    sort_exclude = None,
))]
#[allow(clippy::too_many_arguments)]
fn format_recursive(
    path: &str,
    config: Option<&Bound<'_, PyAny>>,
    options: Option<&PyOptions>,
    indent: Option<usize>,
    currency_column: Option<usize>,
    cost_column: Option<usize>,
    thousands_separator: Option<String>,
    spaces_in_braces: Option<bool>,
    fixed_cjk_width: Option<bool>,
    sort: Option<&Bound<'_, PyAny>>,
    sort_timeless: Option<String>,
    sort_exclude: Option<Vec<String>>,
) -> PyResult<Vec<(String, String)>> {
    use std::path::Path;

    let sort = sort.map(parse_sort).transpose()?;
    let sort_timeless = sort_timeless.map(|s| parse_timeless(&s)).transpose()?;
    let sort_exclude = sort_exclude.map(parse_sort_exclude).transpose()?;

    let p = Path::new(path);
    if !p.exists() {
        return Err(PyOSError::new_err(format!(
            "{path}: No such file or directory"
        )));
    }

    let base_config = resolve_config_param(config, path)?;
    let opts = resolve_options(
        options,
        base_config,
        indent,
        currency_column,
        cost_column,
        thousands_separator,
        spaces_in_braces,
        fixed_cjk_width,
        sort,
        sort_timeless,
        sort_exclude,
    )?;

    let results = crate::recursive::format_recursive(p, &opts);
    Ok(results
        .into_iter()
        .map(|f| (f.path.to_string_lossy().into_owned(), f.content))
        .collect())
}

/// A fast beancount file formatter with CJK support.
#[pymodule]
fn beanfmt(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("__version__", VERSION)?;
    m.add_class::<PyOptions>()?;
    m.add_function(wrap_pyfunction!(format, m)?)?;
    m.add_function(wrap_pyfunction!(format_file, m)?)?;
    m.add_function(wrap_pyfunction!(format_recursive, m)?)?;
    m.add_function(wrap_pyfunction!(parse_config, m)?)?;
    m.add_function(wrap_pyfunction!(load_project_config, m)?)?;
    Ok(())
}
