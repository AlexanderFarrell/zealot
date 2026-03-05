# Database Overview

## Source of Truth

Database SQL lives in `database/` and is composed into `zealotd/init.psql` by `./zdev build database` (or `./zdev build all`).

Composition order is controlled by `database/order.txt`.

## Core Tables and Types

Account:

- `account`: users, hashed passwords, settings JSON

Items:

- `item`: primary item content
- `item_item_link`: item-to-item relationships
- `item_relationship` enum: link semantics

Attributes:

- `attribute`: single-value typed attributes per item/key
- `attribute_list_value`: list-style attribute values
- `attribute_kind`: attribute metadata/config (system or account-owned)

Types:

- `item_type`: user/system type definitions
- `item_item_type_link`: item-to-type assignment
- `item_type_attribute_kind_link`: allowed/linked attribute kinds per type

Planning and activity:

- `repeat_entry` with `repeat_entry_status` enum
- `tracker_entry`
- `comment`

System seed:

- `constants/constants.psql` inserts default system attribute kinds
- seeds include `Plan` and `Repeat` item types with initial links

## Multi-Tenant Pattern

Most domain data is scoped to `account_id` and filtered server-side using session context.

## Initialization Strategy

On server startup:

1. Backend checks for existing `item` table.
2. If missing, runs embedded init SQL in a transaction.
3. Otherwise skips initialization.

This makes a fresh database bootstrappable from application startup without a separate migration runner.

## Legacy Note

`database/init.sql` contains older schema content and does not represent the active PostgreSQL initialization path used by `zealotd`.
