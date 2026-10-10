//! A test-only encoder of the raw model, the inverse of [`crate::parse`] for every model the
//! version can hold: the round-trip and truncation properties build their inputs with it.

use crate::model::{
    Artifacts, Bits, CustomHero, Loss, Player, Pos, RawMap, Text, Version, Victory, VictoryGoal,
};

#[derive(Default)]
pub struct Out(pub Vec<u8>);

impl Out {
    pub fn u8(&mut self, v: u8) {
        self.0.push(v);
    }
    pub fn bool(&mut self, v: bool) {
        self.u8(u8::from(v));
    }
    pub fn u16(&mut self, v: u16) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    pub fn u32(&mut self, v: u32) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    pub fn i32(&mut self, v: i32) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    pub fn len(&mut self, n: usize) {
        self.u32(u32::try_from(n).unwrap());
    }
    pub fn text(&mut self, t: &Text) {
        self.len(t.0.len());
        self.0.extend_from_slice(&t.0);
    }
    pub fn bits(&mut self, b: &Bits) {
        self.0.extend_from_slice(&b.0);
    }
    pub fn pos(&mut self, p: Pos) {
        self.0.extend_from_slice(&[p.x, p.y, p.z]);
    }
    pub fn id(&mut self, v: Version, id: u16) {
        if v == Version::RoE {
            self.u8(u8::try_from(id).unwrap());
        } else {
            self.u16(id);
        }
    }
    pub fn artifact(&mut self, v: Version, a: Option<u16>) {
        let none = if v == Version::RoE { 0xFF } else { 0xFFFF };
        self.id(v, a.unwrap_or(none));
    }
}

pub fn encode(m: &RawMap) -> Vec<u8> {
    let mut o = Out::default();
    let h = &m.header;
    let v = h.version;
    o.u32(v.code());
    o.bool(h.any_players);
    o.u32(h.size);
    o.bool(h.two_levels);
    o.text(&h.name);
    o.text(&h.description);
    o.u8(h.difficulty);
    if v != Version::RoE {
        o.u8(h.level_limit);
    }
    for p in &m.players {
        player(&mut o, v, p);
    }
    victory(&mut o, v, m.victory);
    match m.loss {
        Loss::Standard => o.u8(0xFF),
        Loss::LoseTown { pos } => {
            o.u8(0);
            o.pos(pos);
        }
        Loss::LoseHero { pos } => {
            o.u8(1);
            o.pos(pos);
        }
        Loss::TimeExpires { days } => {
            o.u8(2);
            o.u16(days);
        }
    }
    match m.teams {
        None => o.u8(0),
        Some(t) => {
            o.u8(t.count);
            o.0.extend_from_slice(&t.of_player);
        }
    }
    o.bits(&m.allowed_heroes);
    if v != Version::RoE {
        o.len(m.placeholders.len());
        o.0.extend_from_slice(&m.placeholders);
    }
    if v == Version::Sod {
        o.u8(u8::try_from(m.disposed_heroes.len()).unwrap());
        for d in &m.disposed_heroes {
            o.u8(d.hero_type);
            o.u8(d.portrait);
            o.text(&d.name);
            o.u8(d.players);
        }
    }
    o.0.extend_from_slice(&[0; 31]);
    o.bits(&m.banned_artifacts);
    o.bits(&m.banned_spells);
    o.bits(&m.banned_skills);
    o.len(m.rumors.len());
    for r in &m.rumors {
        o.text(&r.name);
        o.text(&r.text);
    }
    if v == Version::Sod {
        let mut custom = m.custom_heroes.iter().peekable();
        for t in 0..156u8 {
            match custom.next_if(|h| h.hero_type == t) {
                Some(h) => {
                    o.bool(true);
                    custom_hero(&mut o, v, h);
                }
                None => o.bool(false),
            }
        }
    }
    for t in &m.terrain.tiles {
        o.0.extend_from_slice(&[
            t.terrain,
            t.view,
            t.river,
            t.river_dir,
            t.road,
            t.road_dir,
            t.flags,
        ]);
    }
    o.0
}

fn player(o: &mut Out, v: Version, p: &Player) {
    o.bool(p.human);
    o.bool(p.computer);
    let Some(d) = &p.details else {
        o.0.extend_from_slice(
            &[0xAA; 13][..match v {
                Version::RoE => 6,
                Version::Ab => 12,
                Version::Sod => 13,
            }],
        );
        return;
    };
    o.0.push(d.ai_tactic.to_le_bytes()[0]);
    if v == Version::Sod {
        o.u8(d.town_choice);
    }
    if v == Version::RoE {
        o.u8(u8::try_from(d.allowed_towns).unwrap());
    } else {
        o.u16(d.allowed_towns);
    }
    o.bool(d.random_town);
    o.bool(d.main_town.is_some());
    if let Some(t) = d.main_town {
        if v != Version::RoE {
            o.bool(t.generate_hero);
            o.u8(t.town_type);
        }
        o.pos(t.pos);
    }
    o.bool(d.random_hero);
    match &d.main_hero {
        None => o.u8(0xFF),
        Some(h) => {
            o.u8(h.hero_type);
            o.u8(h.portrait);
            o.text(&h.name);
        }
    }
    if v != Version::RoE {
        o.u8(d.unknown);
        o.len(d.heroes.len());
        for h in &d.heroes {
            o.u8(h.hero_type);
            o.text(&h.name);
        }
    }
}

fn victory(o: &mut Out, v: Version, victory: Victory) {
    let Victory::Special {
        allow_standard,
        applies_to_ai,
        goal,
    } = victory
    else {
        o.u8(0xFF);
        return;
    };
    let code = match goal {
        VictoryGoal::AcquireArtifact { .. } => 0,
        VictoryGoal::AccumulateCreatures { .. } => 1,
        VictoryGoal::AccumulateResources { .. } => 2,
        VictoryGoal::UpgradeTown { .. } => 3,
        VictoryGoal::BuildGrail { .. } => 4,
        VictoryGoal::DefeatHero { .. } => 5,
        VictoryGoal::CaptureTown { .. } => 6,
        VictoryGoal::DefeatMonster { .. } => 7,
        VictoryGoal::FlagDwellings => 8,
        VictoryGoal::FlagMines => 9,
        VictoryGoal::TransportArtifact { .. } => 10,
    };
    o.u8(code);
    o.bool(allow_standard);
    o.bool(applies_to_ai);
    match goal {
        VictoryGoal::AcquireArtifact { artifact } => o.id(v, artifact),
        VictoryGoal::AccumulateCreatures { creature, count } => {
            o.id(v, creature);
            o.i32(count);
        }
        VictoryGoal::AccumulateResources { resource, amount } => {
            o.u8(resource);
            o.i32(amount);
        }
        VictoryGoal::UpgradeTown { pos, hall, castle } => {
            o.pos(pos);
            o.u8(hall);
            o.u8(castle);
        }
        VictoryGoal::BuildGrail { pos }
        | VictoryGoal::DefeatHero { pos }
        | VictoryGoal::CaptureTown { pos }
        | VictoryGoal::DefeatMonster { pos } => o.pos(pos),
        VictoryGoal::FlagDwellings | VictoryGoal::FlagMines => {}
        VictoryGoal::TransportArtifact { artifact, pos } => {
            o.u8(artifact);
            o.pos(pos);
        }
    }
}

fn custom_hero(o: &mut Out, v: Version, h: &CustomHero) {
    o.bool(h.experience.is_some());
    if let Some(x) = h.experience {
        o.u32(x);
    }
    o.bool(h.secondary_skills.is_some());
    if let Some(s) = &h.secondary_skills {
        o.len(s.len());
        for &(skill, level) in s {
            o.u8(skill);
            o.u8(level);
        }
    }
    artifacts(o, v, h.artifacts.as_ref());
    o.bool(h.biography.is_some());
    if let Some(b) = &h.biography {
        o.text(b);
    }
    o.u8(h.sex);
    o.bool(h.spells.is_some());
    if let Some(s) = &h.spells {
        o.bits(s);
    }
    o.bool(h.primary_skills.is_some());
    if let Some(p) = h.primary_skills {
        o.0.extend_from_slice(&p);
    }
}

pub fn artifacts(o: &mut Out, v: Version, a: Option<&Artifacts>) {
    o.bool(a.is_some());
    if let Some(a) = a {
        for &w in &a.worn {
            o.artifact(v, w);
        }
        o.u16(u16::try_from(a.backpack.len()).unwrap());
        for &b in &a.backpack {
            o.artifact(v, b);
        }
    }
}
