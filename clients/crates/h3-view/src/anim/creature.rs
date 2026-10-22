//! The creatures' own animations (E17 M3 T7c-3; `docs/epics/E17-design.md` §7.16): Fire
//! Shield's return on a striker (`fire_shield_damage_striker` 0x4409C0) and the explosion a
//! Magog's or a Lich's projectile ends in (`stack_shot` 0x43FA15, 0x43FE07).

use alloc::vec::Vec;

use super::{Explosion, MagicLook, Player, Riding, STANDING, STRUCK, Sound, SpotKey};
use crate::battle_log::Logged;

/// Fire Shield (the executable's spell number).
pub const FIRE_SHIELD: u16 = 29;
/// Fire Shield's return over the shields that struck back (0x440B6D: Fire Shield's own
/// animation).
pub const SHIELD_RETURN: u8 = 11;

impl Player<'_> {
    /// Fire Shield strikes back at `striker` with `damage`, killing `kills` (0x4409C0, after the
    /// strike's line and the striker's effects, before the retaliation; E17 M3 T7c-3,
    /// `creatures`): each stack of `struck` with a shield (Fire Shield cast on it, or its
    /// creature's own) marked; Fire Shield's sound (category 2) on the striker;
    /// `strike_animation` with [`SHIELD_RETURN`] over the shields and the striker's reaction
    /// (player 2); then the damage's line named by Fire Shield (`battle_log_damage` with a
    /// number of 1, 0x440B8C); the sound waited for (not built).
    pub(super) fn shield_return(&mut self, striker: u8, (damage, kills): (u64, u32), struck: &[u8]) {
        let spell = self.spell_handle(FIRE_SHIELD);
        let shields: Vec<_> = struck
            .iter()
            .filter_map(|&s| self.live(s).map(|l| (s, l)))
            .filter(|(_, l)| l.shielded || spell.is_some_and(|h| l.spells.contains(&h)))
            .collect();
        if let Some(h) = spell {
            self.cue(striker, Sound::SpellFile(h), false);
        }
        let riding = self.stage.magic.get(&SHIELD_RETURN).cloned().map(|look| {
            let over = shields
                .iter()
                .map(|(s, l)| (*s, self.magic_at(l, &look), !l.right))
                .collect();
            Riding {
                number: SHIELD_RETURN,
                look,
                over,
            }
        });
        let reactions = self.reactions(&[(striker, kills)]);
        self.iterate(
            None,
            (0, 0, 0, 2),
            (&reactions, false),
            [STANDING, STRUCK],
            riding.as_ref(),
        );
        if let Some(spell) = spell {
            let line = Logged::SpellDamage {
                spell,
                hit: (striker, damage, kills),
            };
            self.log.push((self.clock.now, line));
        }
    }

    /// The explosion a projectile ends in over `target` (0x43FA15 to 0x43FB50, 0x43FE07 to
    /// 0x43FF33; E17 M3 T7c-3, `creatures`): the spell's sound (category 3), then each frame of
    /// its animation a redraw waiting the explosion's time on the deadline, not shown, the frame
    /// drawn plain over everything at the creature's centre (as place 1, never mirrored; the
    /// Liches' too, though the table marks theirs translucent: CLT-Q14, built as the original's;
    /// its DEF loaded apart, so memory's magic animation does not name it), then shown; a redraw
    /// at once erases it. The reactions follow (`strike_animation`).
    pub(super) fn explosion(&mut self, e: Explosion, target: u8) {
        let (Some(live), Some(look)) = (self.live(target), self.stage.magic.get(&e.animation))
        else {
            return;
        };
        let look = MagicLook {
            place: 1,
            ..look.clone()
        };
        self.cue(target, Sound::Spell(e.spell), false);
        let at = self.magic_at(&live, &look);
        let time = self.speed.scale(e.time);
        let key = |shown: u32, frame: Option<u32>| SpotKey {
            animation: e.animation,
            shown,
            frame,
            def: look.def.clone(),
            translucent: false,
            at,
        };
        for f in 0..look.frames {
            let drawn = self.clock.now;
            let shown = self.clock.show(time);
            self.flush(drawn, shown);
            self.spots.push(key(shown, Some(f)));
        }
        self.spots.push(key(self.clock.now, None));
        self.redraw();
    }
}
