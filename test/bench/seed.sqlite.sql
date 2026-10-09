-- Z221 benchmark seed (SQLite). Placeholders @N@ (item count, >= 24) and
-- @ACCOUNT@ (account_id) are substituted by run.sh. Deterministic: no randomness.
-- Run against a database the server has already migrated, containing the
-- benchmark account, attribute kinds and item types, and no items.
-- Shape: see test/bench/README.md.
PRAGMA synchronous = OFF;
PRAGMA foreign_keys = OFF;
PRAGMA cache_size = -1000000;
BEGIN;

CREATE TEMP TABLE bty AS
WITH v(k, name) AS (VALUES (0,'Ticket'),(1,'Topic'),(2,'Book'),(3,'Word'),(4,'Recipe'),(5,'Project'),
                           (6,'Epic'),(7,'Document'),(8,'Meeting'),(9,'Repo'),(10,'Game'))
SELECT k, t.type_id
FROM v JOIN item_type t ON t.name = v.name AND t.account_id = @ACCOUNT@;

CREATE TEMP TABLE bn AS
WITH RECURSIVE n(i) AS (SELECT 1 UNION ALL SELECT i + 1 FROM n WHERE i < @N@),
s1 AS (
  SELECT i,
         i % 12 AS tidx,
         CASE WHEN i > 1 AND i % 100 < 54 THEN (i - 2) / 10 + 1 ELSE 0 END AS p1,
         CASE WHEN i % 9 = 0
              THEN CASE WHEN (i * 7919) % @N@ + 1 = i THEN i % @N@ + 1 ELSE (i * 7919) % @N@ + 1 END
              ELSE 0 END AS lnk
  FROM n
)
SELECT i, tidx, p1, lnk,
       'Item ' || substr('0000000' || i, -7) || ' ' ||
         CASE i % 8 WHEN 0 THEN 'Alpha' WHEN 1 THEN 'Beta' WHEN 2 THEN 'Gamma' WHEN 3 THEN 'Delta'
                    WHEN 4 THEN 'Epsilon' WHEN 5 THEN 'Zeta' WHEN 6 THEN 'Eta' ELSE 'Theta' END AS t,
       CASE WHEN p1 > 0 AND i > 2 AND i % 20 = 0 AND (i * 31) % (i - 1) + 1 <> p1
            THEN (i * 31) % (i - 1) + 1 ELSE 0 END AS p2,
       CASE WHEN (i / 7) % 100 < 18 THEN 2 + i % 2 ELSE 0 END AS nt,
       '' AS lt
FROM s1;
CREATE UNIQUE INDEX bn_i ON bn(i);
UPDATE bn SET lt = (SELECT b2.t FROM bn b2 WHERE b2.i = bn.lnk) WHERE lnk > 0;

INSERT INTO item (item_id, title, content, account_id)
SELECT i, t,
       CASE i % 4
         WHEN 0 THEN ''
         WHEN 1 THEN substr(replace(hex(zeroblob(17)), '00', 'lorem ipsum '), 1, 204)
         WHEN 2 THEN substr(replace(hex(zeroblob(84)), '00', 'lorem ipsum '), 1, 1008)
         ELSE replace(hex(zeroblob(167)), '00', 'lorem ipsum ')
       END || CASE WHEN lnk > 0 THEN char(10) || 'See [[' || lt || ']].' ELSE '' END,
       @ACCOUNT@
FROM bn;

INSERT INTO item_item_type_link (item_id, type_id)
SELECT bn.i, bty.type_id FROM bn JOIN bty ON bty.k = bn.tidx;
INSERT OR IGNORE INTO item_item_type_link (item_id, type_id)
SELECT bn.i, bty.type_id FROM bn JOIN bty ON bty.k = 1 WHERE bn.i % 10 = 0 AND bn.tidx <> 11;

INSERT INTO attribute (item_id, key, value_text)
SELECT i, 'Status', CASE i % 5 WHEN 0 THEN 'To Do' WHEN 1 THEN 'Working' WHEN 2 THEN 'Complete'
                               WHEN 3 THEN 'Hold' ELSE 'Specify' END
FROM bn WHERE tidx < 11;
INSERT INTO attribute (item_id, key, value_int)
SELECT i, 'Priority', 1 + i % 10 FROM bn WHERE tidx < 11;
INSERT INTO attribute (item_id, key, value_text)
SELECT i, 'Description', 'Benchmark item ' || i FROM bn WHERE tidx < 11 AND i % 2 = 0;
INSERT INTO attribute (item_id, key, value_text)
SELECT i, 'TicketID', 'BN' || i FROM bn WHERE tidx = 0;
INSERT INTO attribute (item_id, key, value_int)
SELECT i, 'Rating', 1 + i % 5 FROM bn WHERE tidx = 2;

INSERT INTO attribute_list_value (item_id, key, ordinal, value_item_id)
SELECT i, 'Parent', 0, p1 FROM bn WHERE p1 > 0;
INSERT INTO attribute_list_value (item_id, key, ordinal, value_item_id)
SELECT i, 'Parent', 1, p2 FROM bn WHERE p2 > 0;
CREATE TEMP TABLE bk(k INTEGER);
INSERT INTO bk VALUES (0), (1), (2);
INSERT INTO attribute_list_value (item_id, key, ordinal, value_item_id)
SELECT bn.i, 'Topic', bk.k, 12 * ((bn.i * 13 + bk.k * 7) % (@N@ / 12)) + 1
FROM bn JOIN bk ON bk.k < bn.nt;

INSERT OR IGNORE INTO item_item_link (first_item_id, second_item_id, relationship)
SELECT i, p1, 'parent' FROM bn WHERE p1 > 0;
INSERT OR IGNORE INTO item_item_link (first_item_id, second_item_id, relationship)
SELECT i, p2, 'parent' FROM bn WHERE p2 > 0;
INSERT OR IGNORE INTO item_item_link (first_item_id, second_item_id, relationship)
SELECT bn.i, 12 * ((bn.i * 13 + bk.k * 7) % (@N@ / 12)) + 1, 'topic'
FROM bn JOIN bk ON bk.k < bn.nt;
INSERT OR IGNORE INTO item_item_link (first_item_id, second_item_id, relationship)
SELECT i, lnk, 'wikilink' FROM bn WHERE lnk > 0;
-- The references and completeness marker a migrated, backfilled database has (Z223).
INSERT INTO item_wikilink_ref (item_id, target_title)
SELECT i, lt FROM bn WHERE lnk > 0;
INSERT INTO wikilink_ref_state (account_id) VALUES (@ACCOUNT@);

DROP TABLE bk;
DROP TABLE bn;
DROP TABLE bty;
COMMIT;
PRAGMA foreign_keys = ON;
ANALYZE;
