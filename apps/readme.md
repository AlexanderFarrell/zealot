# Apps

Output binaries for Zealot, such as the server, desktop app, mobile app, and related interfaces.

## Navigation

- [cli](./cli) - The command line interface for Zealot.
- [desktop](./desktop) - The desktop application.
- [mcp](./mcp) - The MCP server for AI agents and LLM-driven workflows.
- [mobile](./mobile) - Mobile apps for Android and iOS.
- [server](./server) - The server and local daemon.
- [tui](./tui) - The terminal user interface for power users.
- [web](./web) - The portable web interface.

Below, we elaborate on why each app matters.

## CLI

The CLI makes Zealot scriptable and composable. It fits naturally into terminal-driven workflows and enables fast, automatable interaction with the system.

Use cases:

- Scripting and automation
- Integration with other tools via shell commands
- Fast execution of focused actions

## Desktop

The desktop app is the primary environment for deep work and high-leverage interaction. It is designed for serious daily use on a full workstation.

Use cases:

- Serious planning sessions
- Mass import and export
- Notifications and background integration
- Rich dashboards and analytics

## MCP

The MCP server allows Zealot to be used through LLMs and AI agents. It creates a structured bridge between natural language systems and Zealot's underlying capabilities.

Use cases:

- Strategic planning with LLMs
- Agent-assisted task execution
- Asking questions about your plans, projects, and data
- Natural language interaction with Zealot

## Mobile

Mobile apps keep Zealot accessible wherever you are. They are ideal for capture, review, and lightweight interaction away from your main machine.

Use cases:

- Quick capture on the go
- Access Zealot anywhere
- Read or update documentation from your phone
- Focused, lightweight workflows

## Server

The server is the core runtime for Zealot. It can run locally as a desktop daemon or remotely as a central server for distributed use.

Use cases:

- Run locally on your machine with optional cloud backup
- Run remotely and sync with desktop and mobile clients
- Support shared or distributed deployments

## TUI

The TUI is a command center for Zealot in the terminal. It supports fast interaction with low overhead while preserving a highly keyboard-driven workflow.

Use cases:

- Powerful terminal-based interaction
- Low cognitive overhead
- Fast navigation and execution

## Web

The web app provides browser-based access to Zealot without requiring a full local installation.

Use cases:

- Quick access from anywhere
- Portable use without installation
- Lightweight sharing and collaboration
- Easy access from locked-down or temporary machines