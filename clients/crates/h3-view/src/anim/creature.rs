//! The creatures' own animations (E17 M3 T7c-3; `docs/epics/E17-design.md` §7.16): Fire
//! Shield's return on a striker (`fire_shield_damage_striker` 0x4409C0) and the explosion a
//! Magog's or a Lich's projectile ends in (`stack_shot` 0x43FA15, 0x43FE07); a Dread Knight's
//! Death Blow and an Ogre Mage's Bloodlust (E17 M6 T5d2a, design §12.23).

use alloc::vec::Vec;

use h3_render::Bugs;

use h3_protocol::wire::Event;
use h3_render::recolour::Effect;

use super::{
    BLOODLUST_STEPS, Explosion, Live, MagicLook, Player, Riding, STANDING, STRUCK, Sound, SpotKey,
    TICK,
};
use crate::battle_log::Logged;

/// Fire Shield (the executable's spell number).
pub const FIRE_SHIELD: u16 = 29;
/// Fire Shield's return over the shields that struck back (0x440B6D: Fire Shield's own
/// animation).
pub const SHIELD_RETURN: u8 = 11;
/// A Dread Knight's Death Blow over the stack it strikes (`damage_stage1_with_luck` 0x443B3C:
/// `sp03_`).
pub const DEATH_BLOW: u8 = 73;

impl Player<'_> {
    /// Fire Shield strikes back at `striker` with `damage`, killing `kills` (0x4409C0, after the
    /// strike's line and the striker's effects, before the retaliation; E17 M3 T7c-3,
    /// `creatures`): each stack of `struck` with a shield (Fire Shield cast on it, or its
    /// creature's own) marked; Fire Shield's sound (category 2) on the striker;
    /// `strike_animation` with [`SHIELD_RETURN`] over the shields and the striker's reaction
    /// (player 2); then the damage's line named by Fire Shield (`battle_log_damage` with a
    /// number of 1, 0x440B8C); the sound waited for (0x440BAE, E17 M5 T5b2).
    pub(super) fn shield_return(
        &mut self,
        striker: u8,
        (damage, kills): (u64, u32),
        struck: &[u8],
    ) {
        let spell = self.spell_handle(FIRE_SHIELD);
        let shields: Vec<_> = struck
            .iter()
            .filter_map(|&s| self.live(s).map(|l| (s, l)))
            .filter(|(_, l)| l.shielded || spell.is_some_and(|h| l.spells.contains(&h)))
            .collect();
        let cue = spell.map(|h| self.cue(striker, Sound::SpellFile(h), false));
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
            (&reactions, false, true),
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
        if let Some(cue) = cue {
            self.wait_for(cue);
        }
    }

    /// The explosion a projectile ends in over `target` (0x43FA15 to 0x43FB50, 0x43FE07 to
    /// 0x43FF33; E17 M3 T7c-3, `creatures`): the spell's sound (category 3), then each frame of
    /// its animation a redraw waiting the explosion's time on the deadline, not shown, the frame
    /// drawn plain over everything at the creature's centre (as place 1, never mirrored; the
    /// Liches' too, though the table marks theirs translucent: CLT-Q14, translucent as the table
    /// says with the bugs fixed, E17 M3 T9d; its DEF loaded apart, so memory's magic animation
    /// does not name it), then shown; a redraw at once erases it. The reactions follow
    /// (`strike_animation`).
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
        let translucent = look.translucent && self.stage.bugs == Bugs::Fixed;
        let key = |shown: u32, frame: Option<u32>| SpotKey {
            animation: e.animation,
            shown,
            frame,
            def: look.def.clone(),
            translucent,
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

    /// `striker`'s Death Blow doubles its strike's damage on `target` (`damage_stage1_with_luck`
    /// 0x4439A7 to 0x443B46, the Dread Knight's 20 % roll; E17 M6 T5d2a, design §12.23): the
    /// log's line (GENRLTXT 366 or 367 by the striker's number), `Deathblo.wav` (category 2),
    /// animation [`DEATH_BLOW`] over the stack struck without its hit at 100 ms a frame, then the
    /// sound waited for. It comes as the strike's damage is reckoned: after good luck's animation,
    /// before the striker's own (`melee_strike` 0x441A0E, after 0x4418B3), a retaliation's too.
    pub(super) fn death_blow(&mut self, striker: u8, target: u8) {
        let Some(live) = self.live(striker) else {
            return;
        };
        let line = Logged::DeathBlow {
            stack: striker,
            count: live.count,
        };
        self.log.push((self.clock.now, line));
        let cue = self.cue(striker, Sound::DeathBlow, false);
        if let Some(struck) = self.live(target) {
            let time = self.speed.scale(TICK);
            self.over_one(DEATH_BLOW, (target, &struck), time, false);
        }
        self.wait_for(cue);
    }

    /// A creature `stack` casts Bloodlust (`spell`, its handle) on `target` (`struck`, a stack
    /// the timeline follows) as its action: an Ogre Mage's (`creature_cast_action` 0x4486A4:
    /// Advanced, one stack; `cast_spell` 0x5A11F3, whose case tests no caster; E17 M6 T5d2a,
    /// design §12.23), a Master Genie's drawn one the same (*provisional*, `master_genie_cast`
    /// 0x447EE0 casts it at Advanced through `cast_spell`; no recorded case): its frames toward the target's hex ([`Player::creature_casting`]), the
    /// cast's start (the outline and the mouse's stack cleared, a redraw at once), the spell's
    /// sound, its effect, the log's line (GENRLTXT 566 or 567 by the caster's number, then 568
    /// with the target's name), the glow over the target (as a hero's, [`Player::glow`]); at the
    /// cast's tail the caster standing outlined (0x5A216B), then its shot sound waited for and
    /// ended, and it turns back if it turned.
    pub(super) fn creature_bloodlust(
        &mut self,
        (stack, spell): (u8, u16),
        (target, struck): (u8, &Live),
        effects: &[Event],
    ) {
        let (turned, _) = self.creature_casting(stack, struck.hex);
        self.outlined.push((self.clock.now, None));
        self.log.push((self.clock.now, Logged::Unmark));
        self.redraw();
        self.cue(stack, Sound::Spell(spell), false);
        self.effects(effects);
        let count = self.live(stack).map_or(0, |l| l.count);
        let line = Logged::CreatureCast {
            stack,
            count,
            spell,
            target,
            target_count: struck.count,
        };
        self.log.push((self.clock.now, line));
        self.glow(&[target], &BLOODLUST_STEPS, Effect::Bloodlust);
        self.stand_caster(stack);
        self.creature_cast_over(stack, turned);
    }
}
