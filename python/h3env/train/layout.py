"""Observation encoding v22's layout as the entity network reads it (E06 T5a, design §11): the
offsets of its parts and the stack columns it uses, the columns the behaviour analysis reads
(`h3env.train.inspect`, E06 T6a) and the turn order's (v22). `h3-sim`'s test
`the_python_layout_is_the_encodings` pins every value here against `observation.rs`; a new
observation encoding changes this file in the same change.
"""

#: The encoding this layout describes (`_ffi.versions()["observation_encoding"]`).
OBSERVATION_ENCODING = 22

GLOBALS = 38
HERO_PUBLIC_FEATURES = 14
OWN_HERO_FEATURES = 578
HEX_PLANES = 16
MAX_FIELD_HEXES = 255
STACK_FEATURES = 99
STACK_CAPACITY = 42
ORDER_FEATURES = 3

#: The viewer's public hero part; the opponent's follows it.
HEROES = GLOBALS
OWN_HERO = HEROES + 2 * HERO_PUBLIC_FEATURES
FIELD = OWN_HERO + OWN_HERO_FEATURES
STACKS = FIELD + HEX_PLANES * MAX_FIELD_HEXES
#: The turn order (v22), after the stack slots: v21's layout is the prefix before it, which a
#: network without `[network] order` reads.
ORDER = STACKS + STACK_FEATURES * STACK_CAPACITY
OBSERVATION_LEN = ORDER + ORDER_FEATURES * STACK_CAPACITY

#: A stack slot's columns the network reads apart: the slot holds a stack, its creature's handle,
#: its position's column and row (a two-hex stack's rear hex; a turret's off-field hex), it is
#: the active stack.
PRESENT = 0
CREATURE = 3
X = 12
Y = 13
ACTIVE = 18

#: The behaviour analysis's columns: the globals' round (0 in the tactics phase); a public hero
#: block's (the viewer's at `HEROES`, the opponent's after it) mana, maximum mana and casts this
#: round; a stack slot's alive and own flags, count, top creature's and full hit points, acted,
#: waited, defending, retaliations left (255 unlimited), shooter, war machine, creatures at the
#: start, its turn has started this round, a town's turret.
ROUND = 0
HERO_MANA = 6
HERO_MAX_MANA = 7
HERO_CASTS = 10
ALIVE = 1
OWN = 2
COUNT = 4
TOP_HP = 5
MAX_HP = 6
ACTED = 14
WAITED = 15
DEFENDING = 17
RETALIATIONS_LEFT = 22
SHOOTER = 26
WAR_MACHINE = 27
BASE_COUNT = 36
STARTED = 37
TURRET = 47

#: A stack slot's turn order columns (`h3_battle::turn_order`): its place in the rest of the
#: round's turns (1: the active stack; 0: none), among its side's turns in the rest of the round
#: (1: its side's next), in the next round's turns (0: none).
ORDER_ROUND = 0
ORDER_SIDE = 1
ORDER_NEXT = 2

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
