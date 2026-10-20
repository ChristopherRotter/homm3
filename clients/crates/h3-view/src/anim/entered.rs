//! The stacks the mouse enters, played on the timeline (E17 M3 T6c-2; `docs/epics/E17-design.md`
//! §9.10, §7.13): `battle_mouse_stack` 0x477C20 starts a stack's group 1 at frame 0 when its DEF
//! has one and it does not show it, else, not standing, makes it stand; the idle steps (0x4960D0)
//! then play the group as a fidget, one frame a step, and it stands at the step after its last.

use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use super::{Key, Pose, Timeline};

impl Timeline {
    /// Plays the stacks the mouse entered (`entered`, each `(t, stack)`, in time order): `group`
    /// the DEF group a fidget shows, `frames` a stack's frames in it (0: none); entering redraws
    /// the stack at once (0x477DF8: only inside its new frame's rectangle, design §9.10). A
    /// stack's fidget ends at its last frame (it stands at the next idle step), at the
    /// start of an action that stands the fidgeting stacks (`battle_stand_fidgets` 0x479DE0:
    /// [`super::Action::stands`]), at a key of its own (an action draws it) or when the mouse
    /// enters it again; an idle step at the same time comes first. No stalls: the viewer draws
    /// none of its own in a check (§7.2; *provisional* in a window, where the original stalls one
    /// step in about 12).
    pub fn enter(&mut self, entered: &[(u32, u8)], group: u32, frames: &dyn Fn(u8) -> u32) {
        if entered.is_empty() {
            return;
        }
        let starts: Vec<(u32, bool)> = self.actions.iter().map(|a| (a.start, a.stands)).collect();
        let mut added: BTreeMap<u8, Vec<Key>> = BTreeMap::new();
        for (n, &(t, stack)) in entered.iter().enumerate() {
            let Some(member) = self.members.iter().find(|m| m.stack == stack) else {
                continue;
            };
            let standing = member.start.group;
            // The pose then (in memory): the mouse's last key after the timeline's own last.
            let own = self
                .keys
                .iter()
                .filter(|k| k.stack == stack && k.drawn <= t)
                .last()
                .map(|k| (k.drawn, k.pose));
            let mine = added
                .get(&stack)
                .and_then(|list| list.iter().rfind(|k| k.drawn <= t))
                .map(|k| (k.drawn, k.pose));
            let now = match (own, mine) {
                (Some((a, own)), Some((b, mine))) => {
                    if b >= a {
                        mine
                    } else {
                        own
                    }
                }
                (Some((_, own)), None) => own,
                (None, Some((_, mine))) => mine,
                (None, None) => member.start,
            };
            let key = |drawn: u32, shown: u32, pose: Pose| Key {
                stack,
                drawn,
                shown,
                pose,
            };
            let base = Pose {
                offset: [0, 0],
                ..now
            };
            let stand = Pose {
                group: standing,
                frame: 0,
                ..base
            };
            let list = added.entry(stack).or_default();
            // A fidget the mouse started earlier ends here.
            list.retain(|k| k.drawn <= t);
            let count = frames(stack);
            if count == 0 || now.group == group {
                if now.group != standing || now.frame != 0 {
                    list.push(key(t, t, stand));
                }
                continue;
            }
            list.push(key(
                t,
                t,
                Pose {
                    group,
                    frame: 0,
                    ..base
                },
            ));
            // Its end: its own next key, the next standing action, the mouse entering it again.
            let own_next = self
                .keys
                .iter()
                .filter(|k| k.stack == stack && k.drawn > t)
                .map(|k| k.drawn)
                .min();
            let calm = starts
                .iter()
                .find(|&&(s, stands)| stands && s >= t)
                .map(|&(s, _)| s);
            let again = entered[n + 1..]
                .iter()
                .find(|(_, s)| *s == stack)
                .map(|(at, _)| *at);
            let until = [own_next, calm, again].into_iter().flatten().min();
            let mut frame = 0;
            let mut ended = false;
            for step in self.steps_after(t, count) {
                if until.is_some_and(|u| step > u) {
                    break;
                }
                frame += 1;
                if frame < count {
                    list.push(key(
                        step,
                        step,
                        Pose {
                            group,
                            frame,
                            ..base
                        },
                    ));
                } else {
                    list.push(key(step, step, stand));
                    ended = true;
                    break;
                }
            }
            // A standing action's start stands it (redrawn at once), unless its own key or the
            // mouse comes first.
            if !ended && let Some(calm) = calm.filter(|&c| until == Some(c)) {
                list.push(key(calm, calm, stand));
            }
        }
        self.merge(&added);
    }

    /// The idle steps after `t`, at most `count` of them (the battle loop's: between the commands,
    /// then after the last, on its 100 ms grid).
    fn steps_after(&self, t: u32, count: u32) -> Vec<u32> {
        let want = usize::try_from(count).unwrap_or(usize::MAX);
        let mut out: Vec<u32> = self
            .steps
            .iter()
            .copied()
            .filter(|&s| s > t)
            .take(want)
            .collect();
        if out.len() < want {
            let left = u32::try_from(want - out.len()).unwrap_or(u32::MAX);
            let from = t.max(self.idle.from);
            let until =
                from.saturating_add(super::IDLE_STEP.saturating_mul(left.saturating_add(1)));
            let mut idle = self.idle;
            idle.run(until, |s| {
                if s > t && out.len() < want {
                    out.push(s);
                }
            });
        }
        out
    }

    /// The mouse's keys merged into the timeline's: each stack's in time order, a key of the mouse
    /// before the timeline's own drawn at the same time (the mouse acts as the battle waits,
    /// before the action it gives starts).
    fn merge(&mut self, added: &BTreeMap<u8, Vec<Key>>) {
        let mut pending: BTreeMap<u8, core::iter::Peekable<core::slice::Iter<'_, Key>>> = added
            .iter()
            .map(|(s, l)| (*s, l.iter().peekable()))
            .collect();
        let mut keys =
            Vec::with_capacity(self.keys.len() + added.values().map(Vec::len).sum::<usize>());
        for key in core::mem::take(&mut self.keys) {
            if let Some(list) = pending.get_mut(&key.stack) {
                while let Some(&&mine) = list.peek() {
                    if mine.drawn > key.drawn {
                        break;
                    }
                    keys.push(mine);
                    list.next();
                }
            }
            keys.push(key);
        }
        for list in pending.into_values() {
            keys.extend(list.copied());
        }
        self.keys = keys;
    }
}
