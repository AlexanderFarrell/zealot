# Zealot Architecture

Goals:

- 🤸 Flexibility
- 📦 Portability
- 🛡️ Robustness
- 🧼 Cleanliness
- 📓 Documented

Let's discuss each one

# Requirements for Architecture

## Flexibility

- Zealot must be able to interface with multiple methods:
    - Web UI
    - Desktop App
    - Mobile App
    - MCP
    - CLI
    - API
- Zealot must allow arbitrary content (via items) which is constrained by rules (item type).
- Zealot must allow attributes (scalar or list) of specific types
- Zealot must be configurable
    - Transport
        - HTTP
        - WS
        - gRPC
    - Schema Storage
        - PostgreSQL
        - SQLite
        - MySQL
        - MariaDB
        - MongoDB
    - Blob Storage
        - Filesystem
        - S3 (or equivalent)
    - Authentication
        - Plain
        - OAuth2

## Portability

- Zealot must be able to be deployed as a local app
- Zealot must be able to be deployed as a server
- Zealot must run on multiple OSs (Windows, MacOS, Linux, FreeBSD, iOS, Android)

## Robustness

- Zealot must be performant enough to handle large numbers of items
- Zealot must make back up easy and automated
- Zealot must allow history and accountability of actions
- Zealot must be easily recoverable in case of total system failure (this is important data being stored here)
- Zealot must be secure by default

## Cleanliness

- Versioning
    - Proof of concepts are allowed for bleeding-edge features (provided they are on master)
    - Regular tagged versions created
    - Regular polish tasks
- Code should not be duplicated across output build artifacts (cli, server, mcp, etc.)
    - Use libraries for common logic
- Core functionality should be battle tested
- Testing throughout

# Go Architecture

For Go, we have the following binaries:

- Zealot-Server - Server Application
- Zealot-Daemon - Desktop Daemon Application
- Zealot-CLI - The CLI interface for interacting with Zealot
- Zealot-MCP - Interface for AI to interact with Zealot

We also have the following libraries/modules:

- Core - Stores ports (how we interface) and abstract business logic
- Engine - Implementations of Services such as rules engine
- Store - Individual storage implementations (PostgreSQL, SQLite, etc.)
- Transport - How do we interact with Zealot externally?
- Test - Integration and E2E tests