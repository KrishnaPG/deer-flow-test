/// A single action slot in the bottom deck.
pub struct ActionSlot {
    pub label: &'static str,
    pub hotkey: &'static str,
}

/// Returns the default set of 15 action slots.
pub fn default_action_slots() -> [ActionSlot; 15] {
    [
        ActionSlot {
            label: "Move",
            hotkey: "Q",
        },
        ActionSlot {
            label: "Hold",
            hotkey: "W",
        },
        ActionSlot {
            label: "Patrol",
            hotkey: "E",
        },
        ActionSlot {
            label: "Focus",
            hotkey: "R",
        },
        ActionSlot {
            label: "Scan",
            hotkey: "A",
        },
        ActionSlot {
            label: "Dock",
            hotkey: "S",
        },
        ActionSlot {
            label: "Escort",
            hotkey: "D",
        },
        ActionSlot {
            label: "Split",
            hotkey: "F",
        },
        ActionSlot {
            label: "Merge",
            hotkey: "Z",
        },
        ActionSlot {
            label: "Queue",
            hotkey: "X",
        },
        ActionSlot {
            label: "Intel",
            hotkey: "C",
        },
        ActionSlot {
            label: "Ping",
            hotkey: "V",
        },
        ActionSlot {
            label: "Repair",
            hotkey: "1",
        },
        ActionSlot {
            label: "Upgrade",
            hotkey: "2",
        },
        ActionSlot {
            label: "Cancel",
            hotkey: "Esc",
        },
    ]
}
