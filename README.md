# A manager for python virtual environments made using UV

PythonManager assists you in installing, updating and removing [Astral UV](https://docs.astral.sh/uv/).
You can use PythonManager to create and remove virtual environments, which are created using `UV`.
When activated, the virtual environment will be invoked in a child shell session in your current shell.
To deactivate the active environment, type `exit` in the terminal.

You can specify location of virual environments and the default python packages by updating the `settings.toml` file.

**Example usage:**

- Install Astral UV:

  ```bash
    python-manager install
  ```

- Update Astral UV if it is alredy installed:

  ```bash
    python-manager install --update
  ```

- Check if Astral UV is installed:

  ```bash
    python-manager check
  ```

- Create a new virtual environment with specific Python version 3.10 and packages maturin, numpy, pandas:

  ```bash
    python-manager create myenv -v 3.10 -p maturin numpy pandas
  ```

- Create a new virtual environment with specific Python version 3.10, default packages and maturin:

  ```bash
    python-manager create myenv -v 3.10 -d -p maturin
  ```

- Activate a virtual environment by name:

  ```bash
    python-manager activate myenv
  ```

- Activate using index number:

  ```bash
    python-manager activate
    #1. myenv
    #2. mysecondenv
    #Please select a virtual environment to activate:
    #1
  ```

- Delete a virtual environment by name:

  ```bash
    python-manager delete myenv
  ```

- Delete a virtual environment using index number:

  ```bash
    python-manager delete
    #1. myenv
    #2. mysecondenv
    #Please select a virtual environment to delete:
    #1
  ```

- List all available virtual environments:

  ```bash
    python-manager list
  ```
