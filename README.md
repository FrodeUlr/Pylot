# A manager for python virtual environments made using UV

PyPilot assists you in installing, updating and removing [Astral UV](https://docs.astral.sh/uv/).  
You can use PyPilot to create and remove virtual environments, which are created using `UV`.  
When activated, the virtual environment will be invoked in a child shell session in your current shell.  
To deactivate the active environment, type `exit` in the terminal.

You can specify location of virtual environments and the default python packages by updating the `settings.toml` file.

**Example usage:**

- Install Astral UV:

  ```bash
    pypilot install-uv
  ```

- Update Astral UV if it is already installed:

  ```bash
    pypilot install-uv --update
  ```

- Check if Astral UV is installed:

  ```bash
    pypilot check
  ```

- Create a new virtual environment with specific Python version 3.10 and packages maturin, numpy, pandas:

  ```bash
    pypilot create myenv -v 3.10 -p maturin numpy pandas
  ```

- Create a new virtual environment with specific Python version 3.10, default packages and maturin:

  ```bash
    pypilot create myenv -v 3.10 -d -p maturin
  ```

- Activate a virtual environment by name:

  ```bash
    pypilot activate myenv
  ```

- Activate using index number:

  ```bash
    pypilot activate
    #1. myenv
    #2. mysecondenv
    #Please select a virtual environment to activate:
    #1
  ```

- Delete a virtual environment by name:

  ```bash
    pypilot delete myenv
  ```

- Delete a virtual environment using index number:

  ```bash
    pypilot delete
    #1. myenv
    #2. mysecondenv
    #Please select a virtual environment to delete:
    #1
  ```

- List all available virtual environments:

  ```bash
    pypilot list
  ```

- Uninstall Astral UV:

  ```bash
    pypilot uninstall-uv
  ```
