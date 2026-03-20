# Project Architecture Blueprint

Generated: 2026-03-20

## Overview

Pylot is a Rust workspace with three crates organized around a simple adapter-to-core split:

- `pylot`: command-line frontend and public application API.
- `pylot-tui`: terminal UI frontend.
- `shared`: reusable core and infrastructure services used by both frontends.

The implemented style is a layered modular monolith rather than separate services:

- Frontend adapters accept user input.
- Application orchestration delegates into shared modules.
- Shared modules perform configuration loading, process execution, UV management, and virtual environment lifecycle work.
- External dependencies are the local filesystem, shell processes, and platform package managers (`uv`, `winget`, `sh`, `pwsh`/`powershell`).

## Primary Architecture

```mermaid
flowchart LR
    User[User]

    subgraph Frontends[Frontend Crates]
        CLIBin[pylot/src/main.rs\nCLI entrypoint]
        CLIApi[pylot/src/lib.rs\napp orchestration API]
        TUILib[tui/src/lib.rs\nTUI runtime loop]
        TUIState[tui/src/app.rs\nstate and actions]
        TUIRender[tui/src/ui.rs\nrendering]
    end

    subgraph Shared[shared crate]
        SharedFacade[shared/src/lib.rs\nmodule re-exports]
        Settings[cfg/settings.rs\nsettings singleton]
        Logger[cfg/logger.rs\nlog initialization]
        VenvManager[virtualenv/venvmanager.rs\nlist and select venvs]
        UvVenv[virtualenv/uvvenv.rs\ncreate/delete/activate]
        Traits[virtualenv/venvtraits.rs\nbehavior contracts]
        UvCtrl[uv/uvctrl.rs\ninstall/update/uninstall/check UV]
        Processes[core/processes.rs\nspawn processes and shells]
        Utils[utility/utils.rs\nconfirm, requirements, which]
        Constants[utility/constants.rs\nplatform constants]
        Errors[error.rs\ntyped error model]
    end

    subgraph External[External System Boundary]
        SettingsFile[settings.toml]
        VenvDir[virtual env directories]
        UV[uv binary]
        Shell[OS shell\npwsh/powershell/sh]
        PackageManager[winget or shell installer]
    end

    User --> CLIBin
    User --> TUILib

    CLIBin --> CLIApi
    CLIBin --> Settings
    CLIBin --> Logger

    TUILib --> TUIState
    TUILib --> TUIRender
    TUILib --> VenvManager
    TUILib --> UvVenv
    TUILib --> UvCtrl

    CLIApi --> VenvManager
    CLIApi --> UvVenv
    CLIApi --> UvCtrl
    CLIApi --> Utils
    CLIApi --> Errors

    SharedFacade --> Settings
    SharedFacade --> Logger
    SharedFacade --> VenvManager
    SharedFacade --> UvVenv
    SharedFacade --> Traits
    SharedFacade --> UvCtrl
    SharedFacade --> Processes
    SharedFacade --> Utils
    SharedFacade --> Constants
    SharedFacade --> Errors

    VenvManager --> Settings
    VenvManager --> UvVenv
    VenvManager --> VenvDir

    UvVenv --> Traits
    UvVenv --> Settings
    UvVenv --> Processes
    UvVenv --> UvCtrl
    UvVenv --> Utils
    UvVenv --> UV
    UvVenv --> Shell
    UvVenv --> VenvDir

    UvCtrl --> Processes
    UvCtrl --> Utils
    UvCtrl --> PackageManager
    UvCtrl --> UV

    Settings --> SettingsFile
    Processes --> Shell
```

## Runtime Interaction Flow

```mermaid
flowchart TD
    Start[User runs command or opens TUI]
    LoadSettings[Load settings and initialize logging]
    ChoosePath{CLI or TUI?}
    ParseCLI[Parse clap command]
    ShowTUI[Start event loop and render state]
    ChooseAction{Requested action}
    UVAction[UV install/update/uninstall/check]
    VenvList[List or select venv]
    VenvCreate[Validate name and package inputs]
    VenvActivate[Build shell activation command]
    VenvDelete[Confirm and remove directory]
    Spawn[Spawn child process or shell]
    ReadOutput[Stream stdout and stderr into logs or UI status]
    UpdateState[Refresh venv list and UV state]
    End[Operation complete]

    Start --> LoadSettings
    LoadSettings --> ChoosePath
    ChoosePath -->|CLI| ParseCLI
    ChoosePath -->|TUI| ShowTUI
    ParseCLI --> ChooseAction
    ShowTUI --> ChooseAction

    ChooseAction -->|UV command| UVAction
    ChooseAction -->|List/select| VenvList
    ChooseAction -->|Create| VenvCreate
    ChooseAction -->|Activate| VenvActivate
    ChooseAction -->|Delete| VenvDelete

    UVAction --> Spawn
    VenvCreate --> Spawn
    VenvActivate --> Spawn
    VenvDelete --> UpdateState
    VenvList --> End

    Spawn --> ReadOutput
    ReadOutput --> UpdateState
    UpdateState --> End
```

## Architectural Boundaries

- `pylot/src/main.rs` is the executable adapter. It parses commands, initializes settings/logging, and dispatches into the library API or TUI.
- `pylot/src/lib.rs` is the application orchestration layer for CLI operations. It validates inputs, enforces flow, and delegates execution to shared components.
- `shared/src/virtualenv/uvvenv.rs` contains the concrete virtual environment lifecycle behavior.
- `shared/src/virtualenv/venvmanager.rs` centralizes discovery, selection, and table rendering for environments.
- `shared/src/uv/uvctrl.rs` encapsulates UV installation, update, uninstall, and availability checks.
- `shared/src/core/processes.rs` is the process boundary for spawning subprocesses and activating child shells.
- `shared/src/cfg/settings.rs` provides a process-wide settings singleton and creates the configured venv directory if needed.
- `tui/src/lib.rs`, `tui/src/app.rs`, and `tui/src/ui.rs` implement a thin interactive adapter over the same shared operations.

## Key Design Patterns

- Shared core via crate reuse: both frontends call into the same `shared` crate rather than duplicating environment or UV logic.
- Thin adapters: CLI and TUI own presentation and input handling, while environment and tool management stay in shared modules.
- Trait-based behavior: `Create`, `Delete`, and `Activate` define the lifecycle operations implemented by `UvVenv`.
- Process boundary isolation: all command spawning and shell activation flow through `shared/src/core/processes.rs`.
- Global singletons where convenient: `Settings` and `VENVMANAGER` use `LazyLock`, which simplifies access but also makes dependency injection less explicit.

## Extension Guidance

- Add new end-user commands in `pylot/src/cli/cmds.rs` and dispatch them from `pylot/src/main.rs`.
- Keep orchestration in `pylot/src/lib.rs` small; move reusable behavior into `shared` when both frontends could need it.
- Add new venv lifecycle behavior behind `shared/src/virtualenv/venvtraits.rs` and implement it in `shared/src/virtualenv/uvvenv.rs` or another concrete type.
- Keep OS-specific process and package-manager logic inside `shared/src/core/processes.rs`, `shared/src/uv/uvctrl.rs`, and `shared/src/utility/constants.rs`.
- Prefer updating TUI state and UI rendering in `tui/src/app.rs` and `tui/src/ui.rs` without pulling terminal concerns into `shared`.

## Risks And Constraints Visible In The Current Design

- The workspace is intentionally local-process oriented; all core operations depend on shell commands and filesystem state.
- `shared` mixes domain behavior and infrastructure details, which is pragmatic here but means the core is not isolated from OS concerns.
- Global state (`Settings`, `VENVMANAGER`) reduces setup friction but can make advanced testing and inversion of control harder.
- The TUI uses background tasks for long-running operations, but ultimately depends on the same shell and UV command behavior as the CLI.