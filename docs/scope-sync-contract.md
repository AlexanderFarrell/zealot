# Scope-Based Sync Contract

**Ticket:** Z141  
**Status:** approved design; implementation is split into bounded follow-on tickets.  
**Prerequisite:** the server/principal/scope foundation in
[Core Model Foundation Design](core-model-foundation-design.md).

**Related policy:** [Server Administration, Enrolment, and Scope Sharing
Contract](server-administration-contract.md) (Z179) defines which identity and
administration data stays server-local, and the membership descriptors and
authorization events that may cross this replication boundary.

Zealot replicates a *scope*, never an entire server by default. A server can
therefore keep a private personal scope local while participating in a shared
project scope from another server. This document specifies the v1 replication
contract; it does not introduce a second mutation model or make desktop local
mode depend on sync.

## 1. Terms and participation

| Term | Meaning |
|---|---|
| **scope identity** | The stable `scope_id` UUID introduced by Z137. It survives replication and is the replication unit. |
| **origin server** | The `server_id` that first created the scope. It is provenance, not permanent exclusive write authority. |
| **participant** | A server that has an activated replication binding for the scope. A participant may hold an authoritative local replica. |
| **local-only** | A scope with no replication binding. Its only authority is its local server. |
| **remote-maintained** | A locally visible replica configured read-only. Mutations are rejected locally; its upstream participant is the write authority. |
| **shared** | A scope with more than one active writer participant. It uses the event/command contract and deterministic conflict rules below. |
| **synced** | A scope whose active binding has completed bootstrap and has durable inbound/outbound checkpoints. It does not imply that every participant is currently online. |

A scope begins local-only. An owner explicitly creates a binding, approves a
peer, bootstraps a replica, and activates it. Discovery alone never makes a
server a participant. A desktop local server is a local-only authoritative
store until its owner opts into this flow.

## 2. Trust, discovery, and activation

Each server has a durable `server_id` and a signing/encryption key set with key
identifiers. A peer record stores the peer `server_id`, pinned public keys,
endpoint(s), transport policy, trust status, and key-rotation history. A server
may learn a candidate endpoint through a manually entered invitation, an
administrator-managed directory, or a previously trusted peer advertisement;
none is trusted until an owner verifies the peer fingerprint and accepts the
scope invitation.

Before a peer exchanges any scope data, it must compare the durable
`server.schema_migration_version` values and reject the session unless they are
exactly equal. The value is the applied SQL migration head, not a mutable
schema-definition revision; this keeps incompatible storage shapes from being
mistaken for compatible replicas.

An invitation names the exact `scope_id`, requested participation mode
(`read_only` or `writer`), minimum protocol/schema versions, expiry, and a
single-use nonce. The origin/authorizing participant signs it. Activation is a
transactional sequence:

1. authenticate and pin the peer; verify invitation signature, expiry, and
   nonce;
2. negotiate a compatible protocol and scope-manifest schema;
3. transfer and validate a signed bootstrap snapshot plus its manifest;
4. persist the replica, a durable inbox, and the bootstrap checkpoint while the
   binding remains `bootstrap_pending`;
5. catch up through the manifest high-water event; then atomically mark the
   binding `active` and permit the negotiated role.

No partial snapshot is queryable as an active scope. A failed bootstrap is
discarded or resumed only through an explicit, validated recovery record.

## 3. Manifest, compatibility, and replica metadata

Every export, bootstrap, and replication session starts with a signed,
versioned scope manifest. It contains only the metadata needed to validate and
reproduce the scope, not server-local credentials or sessions:

```text
scope_id, origin_server_id, manifest_version, protocol_range,
scope_schema_revision, scope_status, participant_set_revision,
membership_revision, conflict_policy_id, media_policy,
snapshot_id, snapshot_hash, event_high_water, issued_at, signer_key_id
```

Each local `scope_replica` additionally records `scope_id`, local `server_id`,
binding mode/status, selected peer, last_pulled_checkpoint,
last_acked_outbound_checkpoint, last_successful_sync_at, last_error code, and
the currently accepted manifest hash. It contains no access token in the
replicated database payload. Checkpoints are opaque, monotonic positions in a
scope event stream, paired with event IDs; implementations must not infer them
from wall-clock time.

The receiver rejects an unknown required manifest field, unsupported major
protocol, unsupported required schema revision, unsigned manifest, mismatched
scope/origin identity, or a manifest that weakens the local trust policy. It may
accept additive optional fields. An incompatible replica is `paused_incompatible`,
not silently downgraded or partially applied.

## 4. What replicates

The v1 replication set is the complete scope closure:

* items and immutable item identity/provenance;
* content, item attributes, type assignments, parent and ordinary item links;
* comments, planner/repeat data, media metadata and blob references;
* scope-owned schema/lifecycle revisions and their immutable audit events;
* active scope members, roles, external-identity mappings, service identities,
  grants/revocations, and authorization/audit events needed to evaluate them;
* immutable domain-command events and the resulting durable ledger entries;
* media blob bytes only under the media transfer policy in section 7.

The normal replication boundary is **all data in one scope**. Type filtering,
subtree sync, and permission-derived partial replicas are deferred. They are
not harmless optimizations: each would create incomplete graph, schema,
authorization, and deletion semantics. A later design may add a derived
read-only publication projection with its own identity and no ability to merge
back into the source scope.

Server-local data never replicates blindly: accounts and password hashes,
sessions/cookies, API keys and secrets, local device registration, peer access
tokens, local endpoints, operator configuration, local notification settings,
and local audit/log retention. Backups preserve these according to the backup
contract but scope replication does not.

All replicated references must resolve within the same scope. The Z137
cross-scope graph prohibition remains in force; a receiver rejects rather than
rewrites an out-of-scope parent, link, attribute reference, comment target, or
media attachment.

## 5. Identity, membership, authorization, and revocation

Authentication remains server-local. Each replicated member has an immutable
`member_subject_id` owned by the scope contract, an optional human/service
descriptor, role, status, grant/revocation event IDs, and no password or API
key. The mapping table on each participant is local:

```text
external_scope_identity(scope_id, member_subject_id, local_principal_id nullable,
                        mapping_kind, verified_at, status)
```

An incoming member can map to a local human principal, a local service
principal, or remain an unmapped external identity. Unmapped identities are
visible only as safe display/audit descriptors and cannot authenticate or gain
local API access. Mapping never merges accounts automatically on matching name
or email; it requires a verified invitation/administrator action. A participant
may use a local service principal to perform replication, but the domain event
retains both the originating member subject and the receiving service actor.

Roles are the Z137 v1 roles: owner, editor, and viewer. Replicated membership
and role changes are authorization events, ordered before any dependent content
event. A revoked member is denied immediately once the revocation is durably
received. On reconnect, a participant first pulls and applies membership/
revocation events before accepting or sending ordinary writes for that member.
For a shared scope, a peer must reject an event whose author was revoked at the
event's causal frontier; it must not revive an older grant due to arrival order.
Removing the final active owner and granting a role beyond the local policy are
invalid commands everywhere.

## 6. One command path, event ledger, and transport

Local UI, HTTP, CLI, MCP, rules, imports, and sync all call the same scoped
domain commands. A successful command commits atomically:

1. validated domain state change;
2. immutable `scope_event` ledger row with event UUID, scope ID, author subject,
   actor principal, causal parents/version vector, command kind, normalized
   payload hash, and correlation ID;
3. transactional outbox row keyed by `(scope_id, event_id)`.

The outbox dispatcher may retry indefinitely. It marks an event delivered only
after the receiving participant durably records it. A receiver first writes an
inbox row keyed by `(source_server_id, event_id)` in the same transaction as
ledger/state application and its checkpoint advance. Duplicate delivery returns
the prior acknowledgement without reapplying. Inbox records, outbox records,
and checkpoints have retention only after every required active participant has
acknowledged a compacted range or a fresh snapshot supersedes it.

The protocol is authenticated pull/push over TLS with signed envelopes.
Sessions exchange manifest hash, supported versions, and checkpoints; either
side may request missing events or a snapshot recovery. Ordering is causal, not
network arrival order. A receiver buffers an event with missing causal parents,
requests the gap, and never applies it as an independent direct database write.
Reconnect/replay is therefore safe, and sync never bypasses scoped validation,
authorization, lifecycle rules, or audit creation.

## 7. Media and blobs

Events carry immutable blob descriptors (`blob_id`, byte length, MIME type,
SHA-256/strong content hash, and optional encrypted-object metadata), never an
unbounded inline payload. Missing blobs are transferred in authenticated,
resumable chunks addressed by the content hash. The receiver validates length
and hash before atomically making a blob available; a failed transfer leaves no
available media row.

The initial media policy is `eager_required`: a checkpoint cannot be announced
as fully applied until all media referenced by its event range is verified.
Future `metadata_first` or on-demand policies require explicit UI/API semantics
for unavailable attachments and may not make a corrupt/missing blob appear
successful. Deletion is a replicated tombstone; physical removal waits for the
same acknowledgement/retention rule as events and must retain enough metadata
to prevent a delayed blob from resurrecting it.

## 8. Conflict policy

V1 is deliberately constrained and deterministic:

* immutable additions with distinct stable IDs merge;
* causally ordered edits apply in event order;
* concurrent edits to the same scalar field, content body, membership/role,
  lifecycle subject state, or deletion versus edit produce a durable
  `sync_conflict` record and do not silently choose a winner;
* concurrent set-like additions merge by stable element ID; concurrent
  add/remove of the same element conflicts unless one causally follows the
  other;
* a deletion tombstone wins only when causally later; concurrent deletion is a
  conflict and retains both recoverable versions;
* membership revocation takes precedence over concurrent ordinary writes by the
  revoked author at the same causal frontier.

A conflict pauses only the affected object/command lane, not the entire scope
or unrelated events. The server exposes a safe conflict explanation and an
authorized resolution command. Resolution records the selected result and all
conflicted event IDs as causal parents, then flows through the same ledger and
outbox. Automatic last-writer-wins, text CRDTs, custom merge scripts, and
cross-scope conflict resolution are explicitly deferred. A restore/import is a
new validated snapshot/event operation, not a way to overwrite a live replica.

## 9. Security and operational requirements

Fully remote clients authenticate to one server and receive no peer credentials,
replication endpoints, or scope data beyond their authorized server/scope API
view. Only server-to-server participants run the replication protocol. Peer
authorization is scoped, mutually authenticated, key-pinned, least-privilege,
and revocable; TLS protects transport, signed envelopes provide provenance, and
payload hashes provide integrity. Secrets remain in local secret storage and
logs/audit explanations use identifiers and reason codes rather than item
content or tokens.

Operators can inspect binding state, peer fingerprint, manifest/version,
checkpoints, lag, conflicts, media integrity state, last error, and revocation
receipt. They can pause/resume a binding, rotate a peer key with overlap, revoke
a peer, and request validated snapshot recovery. They cannot force-apply an
unsigned event, skip an incompatible schema, suppress a conflict, or erase a
revocation history.

## 10. Implementation gates

The work is sequenced after Z138/Z139 establish scope-qualified storage and
authorization. The follow-on tickets own peer/membership trust, manifest and
bootstrap, event transport/durability, protocol/conflicts, media, and the
two-server proof respectively. Each must exercise both SQLite and PostgreSQL;
the final proof covers bootstrap, disconnect/reconnect, replayed delivery,
concurrent conflict, membership revocation, media corruption/retry, backup
restore, and recovery from a retained checkpoint.

No client-side mutation queue, direct replica SQL writes, whole-server dump
sync, user-account replication, or implicit peer discovery is permitted by this
contract.
