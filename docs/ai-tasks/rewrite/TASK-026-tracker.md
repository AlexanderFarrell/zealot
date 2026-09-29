# TASK-026: Legacy Tracker — Retired

The abandoned Tracker client and its hard-coded 1–10 level model must not be rebuilt.
Typed numeric series are implemented through Statistic items and Statistic Entries.

Long-lived databases may still contain Tracker assignments. Leave those assignments and their
comments untouched. Numeric history cannot be reconstructed because the earlier migration did not
preserve numeric levels. A future manual migration may create Statistic items and copy only metadata
that can be verified; it must not invent historical values.
