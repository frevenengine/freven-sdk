use freven_mod_api::ActionKindId;

/// Default action registration keys used by the standard break/place mechanics.
pub mod action_keys {
    pub const BREAK: &str = "freven.vanilla:break";
    pub const PLACE: &str = "freven.vanilla:place";
}

/// Legacy deterministic ids used by existing tests/fixtures.
pub const ACTION_KIND_BLOCK_BREAK: ActionKindId = ActionKindId(1);
pub const ACTION_KIND_BLOCK_PLACE: ActionKindId = ActionKindId(2);
