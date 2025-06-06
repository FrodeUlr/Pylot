# A manager for python virtual environments made using UV

PythonManager assists you in install, update and remove [Astral UV](https://docs.astral.sh/uv/).
You can use PythonManager to create and remove virtual environments, which are created using `UV`.
When activated, the virtual environment will be invoked in a child shell session in your current shell.
To deactivate the active environment, type `exit` in the terminal.

**Example usage:**

- Create and activate a new virtual environment with specific Python version 3.10 and packages maturin, numpy, pandas:

  ```bash
    python-manager create myenv -v 3.10 -p maturin numpy pandas
    python-manager activate myenv
  ```
