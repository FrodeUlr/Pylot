# PyPilot

<!--toc:start-->

- [A manager for python virtual environments made using UV](#a-manager-for-python-virtual-environments-made-using-uv)
- [**Example usage:**](#example-usage)
  - [Install Astral UV](#install-astral-uv)
  - [Update Astral UV if it is already installed](#update-astral-uv-if-it-is-already-installed)
  - [Check if Astral UV is installed](#check-if-astral-uv-is-installed)
  - [Create a new virtual environment with specific Python version 3.10 and packages maturin, numpy, pandas](#create-a-new-virtual-environment-with-specific-python-version-310-and-packages-maturin-numpy-pandas)
  - [Create a new virtual environment with specific Python version 3.10, default packages and maturin](#create-a-new-virtual-environment-with-specific-python-version-310-default-packages-and-maturin)
  - [Activate a virtual environment by name](#activate-a-virtual-environment-by-name)
  - [Activate a Virtual Environment by Index](#activate-a-virtual-environment-by-index)
  - [Delete a virtual environment by name](#delete-a-virtual-environment-by-name)
  - [Delete a virtual environment using index number](#delete-a-virtual-environment-using-index-number)
  - [List all available virtual environments](#list-all-available-virtual-environments)
  - [Uninstall Astral UV](#uninstall-astral-uv)
  <!--toc:end-->

## A manager for python virtual environments made using UV

[![Rust](https://github.com/FrodeUlr/PyPilot/actions/workflows/rust.yml/badge.svg?branch=master)](https://github.com/FrodeUlr/PyPilot/actions/workflows/rust.yml) [![codecov](https://codecov.io/github/FrodeUlr/PyPilot/graph/badge.svg?token=BNWQAU7KR2)](https://codecov.io/github/FrodeUlr/PyPilot)

`PyPilot` assists you in installing, updating and removing [Astral UV](https://docs.astral.sh/uv/).  
You can use `PyPilot` to create and remove virtual environments, which are created using `UV`.  
When activated, the virtual environment will be invoked in a child shell session in your current shell.  
To deactivate the active environment, type `exit` in the terminal.

You can specify location of virtual environments and the default python packages by updating the `settings.toml` file.

## **Example usage:**

### Install Astral UV

Run the following command:

```bash
  PyPilot install-uv
```

### Update Astral UV if it is already installed

Run the following command:

```bash
  PyPilot install-uv --update
```

### Check if Astral UV is installed

Run the following command:

```bash
  PyPilot check
```

### Create a new virtual environment with specific Python version 3.10 and packages maturin, numpy, pandas

Run the following command:

```bash
  PyPilot create myenv -v 3.10 -p maturin numpy pandas
```

### Create a new virtual environment with specific Python version 3.10, default packages and maturin

Run the following command:

```bash
  PyPilot create myenv -v 3.10 -d -p maturin
```

### Activate a virtual environment by name

Run the following command:

```bash
  PyPilot activate myenv
```

### Activate a Virtual Environment by Index

Run the following command:

```bash
  PyPilot activate
```

You will see a list of available virtual environments:

```text
  ╭───────┬──────────────┬─────────╮
  │ Index ┆ Name         ┆ Version │
  ╞═══════╪══════════════╪═════════╡
  │ 1     ┆ MyVenv       ┆ 3.11.13 │
  ├╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌┤
  │ 2     ┆ AnotherVenv  ┆ 3.11.13 │
  ╰───────┴──────────────┴─────────╯
  Please select a virtual environment to activate:
```

Type the index number (e.g., `1`) and press Enter.

### Delete a virtual environment by name

Run the following command:

```bash
  PyPilot delete myenv
```

### Delete a virtual environment using index number

Run the following command:

```bash
  PyPilot delete
```

You will see a list of available virtual environments:

```text
  ╭───────┬──────────────┬─────────╮
  │ Index ┆ Name         ┆ Version │
  ╞═══════╪══════════════╪═════════╡
  │ 1     ┆ MyVenv       ┆ 3.11.13 │
  ├╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌┤
  │ 2     ┆ AnotherVenv  ┆ 3.11.13 │
  ╰───────┴──────────────┴─────────╯
  Please select a virtual environment to delete:
```

Type the index number (e.g., `1`) and press Enter.

### List all available virtual environments

Run the following command:

```bash
  PyPilot list
```

You will see a list of available virtual environments:

```text
  ╭───────┬──────────────┬─────────╮
  │ Index ┆ Name         ┆ Version │
  ╞═══════╪══════════════╪═════════╡
  │ 1     ┆ MyVenv       ┆ 3.11.13 │
  ├╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌┤
  │ 2     ┆ AnotherVenv  ┆ 3.11.13 │
  ╰───────┴──────────────┴─────────╯
```

### Uninstall Astral UV

Run the following command:

```bash
  PyPilot uninstall-uv
```
