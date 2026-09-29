# Core Model Foundation Design

**Tickets:** Z137, Z156, Z167  
**Status:** approved design; implementation is deliberately sliced below.  
**Applies to:** the Rust server, SQLite, PostgreSQL, HTTP API, CLI, MCP, Lua,
export/restore, and future sync.

This is a single design because tenancy, schema, and lifecycle are not safely
separable. A schema definition belongs somewhere; a lifecycle is schema; and a
transition must be authorized in the subject's scope. The design is additive to
the current account-scoped model and preserves current item IDs, links,
attributes, types, comments, media, planner queries, and API shapes during the
migration.

## Decisions at a glance

| Concern | Decision |
|---|---|
| Server identity | Every database has one durable `server` row with an opaque UUID. It is created once and is never derived from a hostname or account. |
| Actor identity | An `account` remains the authentication record. A `server_principal` maps an account or service identity to a stable, server-local principal. |
| Scope | Every item belongs to exactly one active scope. A scope has members, roles, an owner principal, and a stable UUID. |
| Existing data | Migration creates one personal scope per existing account, owner membership, and assigns that account's items to it. No item is copied or renumbered. |
| Cross-scope graph | v1 parent, type assignment, item-valued attribute, repeat, comment, and ordinary item links must remain inside a scope. Cross-scope sharing is deferred to an explicit capability, not an accidental reference. |
| Schema items | Item Type, Attribute Kind, Relation Type, Status Definition, and Lifecycle Definition are first-class items backed by normalized, indexed projections. Bootstrap tables remain protected kernel data. |
| Schema authority | A valid active immutable schema revision and its projection are committed in one transaction. The revision is authoritative; projections are query/validation representations and cannot be edited independently. |
| Lifecycle | An item has no lifecycle unless its effective type binding resolves to exactly one active lifecycle. Governed changes go through a transition command that appends history and materializes the current state. |
| Legacy `Status` | Existing values are preserved. For governed items the transition transaction mirrors the active status label to the current `Status` attribute; ungoverned items keep the present flexible attribute behavior. |

## 1. Server, principals, and scopes (Z137)

### 1.1 Terms and ownership

`server` is an installation identity, not a remote host or tenant. Its UUID is
included in backups, exports, sync envelopes, and audit records. It is generated
at bootstrap and is retained on restore; a deliberate clone operation must mint
a new ID.

`account` continues to hold login credentials and profile information. A
`server_principal` is the authorization identity:

* `kind = human` has a non-null, unique `account_id`.
* `kind = service` has no account and has a named, revocable service identity.
* both kinds have a UUID, display name, status, and timestamps.

This separates future service/API identities from human accounts without
changing the current login or API-key protocols. API keys authenticate an
account today and resolve to that account's human principal. A future
service-key resolves directly to a service principal.

`scope` is the notebook-like ownership boundary. It has `scope_id` (UUID),
`server_id`, `title`, optional `description`, `status` (`active` or `archived`),
`owner_principal_id`, `created_at`, and `updated_at`. `scope_member` has
`scope_id`, `principal_id`, role (`owner`, `editor`, `viewer`), status
(`active`, `revoked`), and timestamps. An active scope always has at least one
active owner; database constraints plus service validation prevent removal of
the last owner.

Roles are deliberately small in v1. Owners manage membership and schema;
editors create and modify allowed content; viewers read. Feature-specific
permissions (for example, an approval transition) are a later policy layer and
do not overload membership roles.

### 1.2 Required storage shape

The physical column types differ by engine (SQLite `TEXT` UUID/timestamp,
PostgreSQL `uuid`/`timestamptz`), but the semantics are identical.

```text
server(server_id PK, display_name, created_at)
server_principal(principal_id PK, server_id FK, kind, account_id nullable FK,
                 display_name, status, created_at, retired_at)
scope(scope_id PK, server_id FK, title, description, status,
      owner_principal_id FK, created_at, updated_at)
scope_member(scope_id FK, principal_id FK, role, status, created_at, updated_at,
             PK(scope_id, principal_id))
item.scope_id NOT NULL FK scope(scope_id)
```

Unique constraints are `(server_id, account_id)` for non-null human principal
accounts and `(server_id, normalized_title)` for active scopes only if the
product elects to prohibit duplicate titles. Scope UUIDs, not titles, are API
and sync identifiers. Index `item(scope_id, item_id)` and scope-qualified
indexes for all planner, attribute-filter, search, heading, and link queries.

Every item access begins by resolving the authenticated account to a principal,
checking active membership, then adding `scope_id` to the repository query. It
is invalid for an application service to accept an item ID and query it without
the resolved scope authorization context.

### 1.3 Legacy migration

The migration is restart-safe and transactionally idempotent:

1. Create the one server row if absent, then create a human principal for every
   account if absent.
2. Create `Personal — <username>` for each account that owns items, or create
   it for every account to make new empty accounts predictable.
3. Add an active owner membership for that account's principal.
4. Add nullable `item.scope_id`; backfill each item from its existing
   `account_id` owner's personal scope; validate no nulls; then make it
   non-null and index it.
5. Validate that every existing link, item-valued attribute, repeat, comment,
   media attachment, and planner query resolves inside the assigned scope.
   Existing same-account data satisfies this by construction. Abort rather than
   silently repair any corrupted cross-account reference.
6. Switch repositories to scope-qualified access and retain `account_id` as the
   item creator/legacy ownership field until a later, separately approved
   removal migration.

The current root items (Nexus, Build, Work, Topics, Spiritual, and Life) are
ordinary items. They migrate into their owning account's personal scope and
remain roots because their parent links and `Root` attributes are untouched.
They are not server-global system items.

Item types and attribute kinds remain server-global during this first scope
migration for compatibility. Their ownership/visibility moves in the schema
item migration below; ordinary value validation must continue to resolve the
same system and account-visible definitions throughout the transition.

### 1.4 Scope fixtures and verification

Both SQLite and PostgreSQL integration fixtures must prove:

* bootstrap produces one server row, one human principal per account, and one
  owner member per personal scope;
* every pre-existing item gets the correct scope and retains its ID, content,
  attributes, types, links, comments, media, repeat entries, and planner result;
* a member cannot fetch, search, filter, link to, or mutate an item outside its
  scope;
* parent/child navigation and item-valued attributes reject cross-scope writes;
* a service principal can be a member without an account; and
* rerunning the migration changes neither cardinalities nor IDs.

## 2. Protected bootstrap kernel and schema items (Z156)

### 2.1 Kernel boundary

Zealot cannot load a schema that is itself required to explain how to load its
schema. The protected bootstrap kernel therefore remains normalized database
data, migrations, and Rust validators—not editable ordinary items. It includes:

* server, principal, scope, and membership tables;
* items, item identity/ownership, attributes, links, accounts, sessions,
  migrations, backup metadata, and audit/transition append-only tables;
* the fixed bootstrap definition kinds and their parser/validator;
* a minimal system vocabulary: `Item Type`, `Attribute Kind`, `Relation Type`,
  `Status Definition`, `Lifecycle Definition`, `Schema Revision`, and the
  `Status` compatibility attribute; and
* repair tooling able to list, deactivate, reproject, or restore definitions
  even when an editable schema revision is invalid.

Kernel records use stable machine keys and UUIDs, are seeded by migrations, and
cannot be deleted or renamed through normal item mutation. They may receive a
display-label update only through an explicit, versioned kernel migration.

### 2.2 First-class definition model

The first migration promotes exactly these definitions:

| Definition item | Projection purpose |
|---|---|
| Item Type | type lookup, type assignment, required/allowed field validation |
| Attribute Kind | value parsing, validation, filtering, indexing |
| Relation Type | link validation, cardinality, direction, cross-scope policy |
| Status Definition | stable state identity, label, category, terminal/reversible metadata |
| Lifecycle Definition | allowed state graph and type lifecycle binding |

Each definition is an ordinary item in its owning scope, assigned one protected
definition type. Its stable machine identity is a UUID in a `schema_definition`
row, never its item ID, title, or mutable attribute key. A definition has
immutable `schema_revision` rows containing the normalized payload and revision
number. `schema_projection` tables are derived from the one active revision and
hold indexed query/validation fields. An activation transaction validates the
draft, creates its projection, atomically switches the definition's active
revision pointer, and records an audit event. No endpoint may update a
projection directly.

The item is the human-facing identity, documentation, discussions, links, and
scope location. The active revision is the semantic source. This avoids making
free-form item content executable schema while still making definitions
discoverable, linkable, exportable, and permissioned as first-class items.

### 2.3 Field assignments and revision states

An Item Type revision carries structured field assignments, not an unstructured
list of keys:

```json
{
  "attribute_kind_id": "uuid",
  "required": true,
  "cardinality": "one",
  "default": null,
  "validation_override": null,
  "indexed": true,
  "presentation": {"label": "Priority", "order": 20, "group": "Planning"}
}
```

`cardinality` is `one` or `many`; it must agree with the referenced attribute
kind's scalar/list shape. Defaults are validated before activation and applied
only by an explicit create/template operation—never retroactively to existing
items. `validation_override` may narrow a kind's validation, never widen it.
`indexed` requests a supported projection index and is rejected if the engine
cannot create its equivalent index.

Revisions move only as follows:

```text
draft -> active -> retired
draft -> invalid
invalid -> draft (new revision, never in-place repair)
active -> retired
```

There is one active revision per definition. A draft is editable through new
revision records. An invalid revision is retained with machine-readable
validation failures for explanation and repair, but has no projection and is
never consulted at runtime. Retiring a definition is blocked while it remains
referenced by active definitions or live item data unless a reviewed migration
plan supplies a replacement or an explicit data disposition.

### 2.4 Scope, deletion, and compatibility

Definition items normally belong to one scope and are visible to its active
members. Kernel definitions are server-global read-only. A future shared schema
library must use an explicit published/imported revision reference; it must not
make a scope's mutable definition silently visible elsewhere.

Rename changes display labels only. Stable key/UUID changes, base-type changes,
field removal, relation direction changes, or lifecycle graph changes require a
migration preview. The preview counts affected definitions, items, values,
links, views, rules, API clients, and exports; it names required conversions and
offers dry-run, apply, and rollback-before-activation steps. Destructive delete
is never a `force` flag on a first-class definition. The repair command can
reproject the active revision, restore the prior active revision, or mark an
unusable definition retired while preserving all historical revisions.

Existing custom and system `item_type` / `attribute_kind` rows migrate by
creating a corresponding definition item and revision, preserving numeric IDs
as `legacy_id` in the projection. The old table becomes the projection in the
same transaction. Existing type-to-attribute links become structured field
assignments (`required = false` unless current required semantics prove
otherwise). System rows become protected kernel-backed definition items; custom
rows become owner-scope definitions. No current API identifier or payload is
removed in this migration.

### 2.5 Interface and backup contract

HTTP, CLI, MCP, and Lua receive the same capabilities: inspect active and draft
definitions, validate a draft, request a preview, activate a valid revision,
and request a repair explanation. Existing type/attribute endpoints remain
compatibility facades over active projections and report deprecation metadata
only after equivalent revision endpoints ship. Exports include definition items,
revisions, active pointers, projections sufficient for portable restore, and
their scope UUIDs. Backup/restore preserves server UUID by default and verifies
that every active projection hashes to its active revision.

## 3. Status definitions and lifecycle transitions (Z167)

### 3.1 Lifecycle semantics

A Status Definition is a schema definition with stable ID, label, optional
category, and `terminal` / `reversible` flags. A Lifecycle Definition is a
versioned directed graph of status IDs and transition edges. An edge contains a
stable transition ID, from/to state, optional action label, required authority,
and optional guard reference. Lifecycle and status revisions use the revision
rules above; an active lifecycle may only reference active status definitions in
the same scope or protected kernel.

An active Item Type revision may bind one lifecycle. An item with no binding is
intentionally flexible. An item assigned several types is governed only if all
effective type bindings resolve to the same lifecycle; conflicting bindings make
the type assignment invalid with an explanation, rather than selecting one by
accident. A future per-item override must be explicit and audited.

Initial state is the lifecycle's configured default. Terminal means no outgoing
edge in that lifecycle version unless an explicit reversible edge exists; it is
not a global claim that a label such as `Complete` is always terminal.

### 3.2 Durable transition record

The transition command writes three records in one transaction:

```text
lifecycle_subject_state(subject_item_id PK, lifecycle_revision_id, status_id,
                        state_version, updated_at, last_transition_id)
status_transition(transition_id PK, subject_item_id, scope_id,
                  lifecycle_revision_id, prior_status_id nullable, next_status_id,
                  actor_principal_id, authority, occurred_at, correlation_id,
                  reason, payload_ref, expected_state_version)
```

`status_transition` is append-only. `payload_ref` points to a separately
validated immutable payload/audit blob and never stores arbitrary secret-bearing
request bodies. `correlation_id` ties a user gesture, CLI request, MCP request,
rule execution, or sync operation together. The authenticated principal and
the authority actually used are both recorded; a language model may propose a
transition, but never supplies authority.

The command requires `expected_state_version` (or an explicitly requested
read-then-transition helper that supplies it). It checks scope membership,
lifecycle version, current state, edge validity, guard/policy, and optimistic
concurrency before appending. A mismatch returns `conflict` with current state
and explanation; a prohibited edge returns `denied` with safe reason codes; a
dry run runs all checks and returns the resulting state/version without writes.
No status transition authorizes unrelated item-field mutation.

For a governed item, successful transition materializes the current status and
updates the legacy `Status` attribute to the active definition's label in the
same transaction. Direct legacy `Status` writes to a governed item are rejected
with an explanation pointing to the transition command. Ungoverned items retain
ordinary attribute writes, so existing flexible notes and records need no
lifecycle migration.

### 3.3 Clients, rules, sync, and views

All surfaces call one application service, `transition_item_status`, or its dry
run/inspect companions. The HTTP API, CLI, MCP, and UI must expose: effective
lifecycle, current status/version, available transitions with authority and
guard explanations, dry-run result, and transition history. Lua receives an
explicit transition function that runs under the rule's named service principal
and cannot bypass validation. Saved views and planner continue to filter on the
materialized `Status` compatibility attribute while new views may filter by
stable status ID. Sync transmits the immutable transition event plus causal
metadata; a receiver revalidates policy and detects state-version conflicts,
never blindly replays a state mutation.

### 3.4 Migration and fixtures

The first lifecycle rollout does not change any existing `Status` value. It
creates protected Status Definitions for the current seeded labels and a
compatibility lifecycle only where an administrator explicitly binds it to a
type. Existing items remain ungoverned until a type lifecycle binding is
activated. Binding preview reports every affected item, unknown/dropdown status
value, and required default/normalization decision. Unknown values block
activation unless mapped or intentionally represented by a new status
definition.

SQLite and PostgreSQL fixtures must cover: a valid transition and complete audit
record; terminal/reversible edge handling; denied membership/authority/guard;
optimistic-concurrency conflict; dry run with no writes; legacy compatibility
write; rejection of direct governed `Status` mutation; flexible ungoverned
items; type-binding conflict; migration of each seeded status; and export/restore
of definitions, active revisions, subject state, and history.

## 4. Implementation order and acceptance gates

1. **Foundation migration (Z137).** Add server/principal/scope/membership and
   scope-qualified repositories; backfill and run the scope fixtures before any
   schema ownership feature is enabled.
2. **Schema registry (Z156).** Add bootstrap kernel versioning, definition
   items/revisions/projections, field assignments, preview/repair endpoints,
   and compatibility facades. Migrate Item Type and Attribute Kind first, then
   Relation Type; enable Status/Lifecycle definition kinds without binding
   lifecycle behavior yet.
3. **Lifecycle engine (Z167).** Add status/lifecycle revisions, effective
   binding resolution, transition state/history, dry-run/explanation surface,
   and compatibility mirroring. Bind no existing type until migration fixtures
   and client support pass.
4. **Cross-cutting release gate.** Run the same fixtures on SQLite and
   PostgreSQL; verify scope isolation at every endpoint; verify old CLI/MCP/API
   payloads; take/restore a backup; and run a real migrated personal database
   through planner, links, comments, media, rules, export, and sync-envelope
   smoke tests.

No later slice may assume a global schema mutation, direct status write for a
governed item, or unscoped item lookup. Those are architectural invariants, not
conventions.
