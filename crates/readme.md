# Crates

Rust library crates for apps.

## Navigation

 - [zealot-api](./zealot-api) - How Zealot delivers to clients (HTTP, gRPC, etc.)
 - [zealot-app](./zealot-app) - How a Zealot app is structured. Includes repos (database contracts), services (business logic) and ports (capabilities).
 - [zealot-domain](./zealot-domain) - How Zealot's data is structured.
 - [zealot-infra](./zealot-infra) - Implementation of ports (email, auth, desktop notifications, etc.) and repos.

 Concepts:

 - Domain - The structure of the data and encapsulation.
 - App - A single running executable's config of ports, repos, and services
 - Port - A capability the Zealot app has (such as email, auth, desktop notifications, etc.)
 - Repo - A structured data store of information (access to a database)
 - Service - Business logic layer, uses ports and repos as needed.
 - 


