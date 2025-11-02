# Pylot

<!--toc:start-->

- [Pylot](#pylot)
  - [A manager for python virtual environments made using UV](#a-manager-for-python-virtual-environments-made-using-uv)
  - [Create completions for your shell](#create-completions-for-your-shell)
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
    - [Shortcuts/Aliases](#shortcutsaliases)
    <!--toc:end-->

## A manager for python virtual environments made using UV

[![Rust](https://github.com/FrodeUlr/pylot/actions/workflows/rust.yml/badge.svg?branch=master)](https://github.com/FrodeUlr/pylot/actions/workflows/rust.yml) [![codecov](https://codecov.io/github/FrodeUlr/pylot/graph/badge.svg?token=BNWQAU7KR2)](https://codecov.io/github/FrodeUlr/pylot)

`Pylot` assists you in installing, updating and removing [Astral UV](https://docs.astral.sh/uv/).  
You can use `Pylot` to create and remove virtual environments, which are created using `UV`.  
When activated, the virtual environment will be invoked in a child shell session in your current shell.  
To deactivate the active environment, type `exit` in the terminal.

You can specify location of virtual environments and the default python packages by updating the `settings.toml` file.

## Create completions for your shell

`Pylot` can generate shell completions for various shells, currently supporting `bash`, `zsh`, `fish`, `powershell` and `elvish`.  
An example of how to generate and install completions for different shells is shown below:

```bash
# bash
# Add the generated file to your bash completions directory
pylot complete bash > /etc/bash_completion.d/pylot.bash
# zsh
# Add the generated file to $FPATH or source it in your .zshrc
pylot complete zsh > ~/.zsh/completions/pylot.zsh
# fish
# Add the generated file to your fish completions directory or source it in your config.fish
pylot complete fish > ~/.config/fish/completions/pylot_completion.fish
# powershell
# Add this to your powershell profile
pylot complete powershell | Out-String | Invoke-Expression
# elvish
# Add the generated file to your elvish completions directory or source it in your rc.elvish
pylot complete elvish > ~/.elvish/completions/pylot.elvish

```

## **Example usage:**

### Install Astral UV

Run the following command:

```bash
  pylot uv install
```

### Update Astral UV if it is already installed

Run the following command:

```bash
  pylot uv update
```

### Check if Astral UV is installed

Run the following command:

```bash
  pylot uv check
```

### Create a new virtual environment with specific Python version 3.10 and packages maturin, numpy, pandas

Run the following command:

```bash
  pylot venv create myenv -v 3.10 -p maturin numpy pandas
```

### Create a new virtual environment with specific Python version 3.10, default packages and maturin

Run the following command:

```bash
  pylot venv create myenv -v 3.10 -d -p maturin
```

### Activate a virtual environment by name

Run the following command:

```bash
  pylot venv activate myenv
```

### Activate a Virtual Environment by Index

Run the following command:

```bash
  pylot venv activate
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
  pylot venv delete myenv
```

### Delete a virtual environment using index number

Run the following command:

```bash
  pylot venv delete
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
  pylot venv list
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
  pylot uv uninstall
```

### Shortcuts/Aliases

```bash
# Install Astral UV
pylot u i
# Uninstall Astral UV
pylot u u
# Update Astral UV
pylot u up
# Create virtual environment
pylot v c myenv -v 3.10 -p maturin numpy pandas
# Delete virtual environment
pylot v d myenv
# List virtual environments
pylot v l
# And so on...
```
