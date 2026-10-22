"""Observation encoding v21's layout as the entity network reads it (E06 T5a, design §11): the
offsets of its parts and the stack columns it uses. `h3-sim`'s test
`the_python_layout_is_the_encodings` pins every value here against `observation.rs`; a new
observation encoding changes this file in the same change.
"""

#: The encoding this layout describes (`_ffi.versions()["observation_encoding"]`).
OBSERVATION_ENCODING = 21

GLOBALS = 38
HERO_PUBLIC_FEATURES = 14
OWN_HERO_FEATURES = 578
HEX_PLANES = 16
MAX_FIELD_HEXES = 255
STACK_FEATURES = 99
STACK_CAPACITY = 42

#: The viewer's public hero part; the opponent's follows it.
HEROES = GLOBALS
OWN_HERO = HEROES + 2 * HERO_PUBLIC_FEATURES
FIELD = OWN_HERO + OWN_HERO_FEATURES
STACKS = FIELD + HEX_PLANES * MAX_FIELD_HEXES
OBSERVATION_LEN = STACKS + STACK_FEATURES * STACK_CAPACITY

#: A stack slot's columns the network reads apart: the slot holds a stack, its creature's handle,
#: its position's column and row (a two-hex stack's rear hex; a turret's off-field hex), it is
#: the active stack.
PRESENT = 0
CREATURE = 3
X = 12
Y = 13
ACTIVE = 18

#: Action encoding v13's entries the heads build (`h3_sim::action`): Wait, Defend, the moves,
#: 24 attack positions per stack, 2 shots per stack, a heal per stack, End tactics, Retreat,
#: Surrender, the two casts, the wall shots.
ACTION_ENCODING = 13
MOVE_BASE = 2
ATTACK_SLOTS = 24
ATTACK_BASE = MOVE_BASE + MAX_FIELD_HEXES
SHOOT_BASE = ATTACK_BASE + ATTACK_SLOTS * STACK_CAPACITY
HEAL_BASE = SHOOT_BASE + 2 * STACK_CAPACITY
END_TACTICS = HEAL_BASE + STACK_CAPACITY
WALL_SHOT_BASE = END_TACTICS + 5
MAX_WALL_TARGETS = 8
ACTION_COUNT = WALL_SHOT_BASE + MAX_WALL_TARGETS
#: The target heads: none, every hex, every stack slot.
TARGET_STACK_BASE = 1 + MAX_FIELD_HEXES
TARGET_COUNT = TARGET_STACK_BASE + STACK_CAPACITY
SPELL_COUNT = 128
