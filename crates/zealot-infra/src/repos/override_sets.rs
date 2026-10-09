//! SQL fragments shared by the SQLite and PostgreSQL item type repositories for resolving which
//! type-level defaults apply to a member. Ids are database ids, never user input, so they are
//! written into the SQL text.

/// Most override types one key can have before the bitmask grouping stops being usable.
pub const MAX_UNIVERSE: usize = 30;

fn id_list(ids: &[i64]) -> String {
    ids.iter()
        .map(|id| id.to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

/// Integer where bit `n` is set when the member `l.item_id` has `universe[n]` among its types.
pub fn mask_expr(universe: &[i64]) -> String {
    if universe.is_empty() {
        return "0".to_string();
    }
    let cases: String = universe
        .iter()
        .enumerate()
        .map(|(bit, id)| format!("WHEN {id} THEN {}", 1i64 << bit))
        .collect::<Vec<_>>()
        .join(" ");
    format!(
        "COALESCE((SELECT SUM(CASE x.type_id {cases} END) FROM item_item_type_link x \
         WHERE x.item_id = l.item_id AND x.type_id IN ({})), 0)",
        id_list(universe)
    )
}

/// Predicate: among `universe`, the member `l.item_id` has exactly the types in `set`.
pub fn exact_set(universe: &[i64], set: &[i64]) -> String {
    if universe.is_empty() {
        return "1 = 1".to_string();
    }
    let in_universe = format!(
        "(SELECT COUNT(*) FROM item_item_type_link x WHERE x.item_id = l.item_id \
         AND x.type_id IN ({})) = {}",
        id_list(universe),
        set.len()
    );
    if set.is_empty() {
        return in_universe;
    }
    format!(
        "{in_universe} AND (SELECT COUNT(*) FROM item_item_type_link x WHERE x.item_id = l.item_id \
         AND x.type_id IN ({})) = {}",
        id_list(set),
        set.len()
    )
}

/// The ids of `universe` selected by `mask`.
pub fn set_from_mask(universe: &[i64], mask: i64) -> Vec<i64> {
    universe
        .iter()
        .enumerate()
        .filter(|(bit, _)| mask & (1i64 << bit) != 0)
        .map(|(_, id)| *id)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mask_round_trips_to_the_set() {
        let universe = [7, 9, 12];
        assert_eq!(set_from_mask(&universe, 0), Vec::<i64>::new());
        assert_eq!(set_from_mask(&universe, 0b101), vec![7, 12]);
    }

    #[test]
    fn empty_universe_matches_everyone() {
        assert_eq!(mask_expr(&[]), "0");
        assert_eq!(exact_set(&[], &[]), "1 = 1");
    }
}
