//! A test-only encoder of the raw model, the inverse of [`crate::parse`] for every model the
//! version can hold: the round-trip and truncation properties build their inputs with it.

use crate::model::{
    Army, Artifacts, Bits, Body, Buildings, CustomHero, Loss, Message, Mission, Player, Pos, Quest,
    RawMap, Reward, Rewards, Stack, Template, Text, TimedEvent, Version, Victory, VictoryGoal,
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
    o.len(m.templates.len());
    for t in &m.templates {
        template(&mut o, t);
    }
    o.len(m.objects.len());
    for ob in &m.objects {
        o.pos(ob.pos);
        o.u32(ob.template);
        o.0.extend_from_slice(&[0; 5]);
        body(&mut o, v, &ob.body);
    }
    o.len(m.events.len());
    for e in &m.events {
        timed_event(&mut o, v, e);
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

fn template(o: &mut Out, t: &Template) {
    o.text(&t.def);
    o.0.extend_from_slice(&t.passable);
    o.0.extend_from_slice(&t.visitable);
    o.u16(t.terrains);
    o.u16(t.editor_terrains);
    o.u32(t.class);
    o.u32(t.subclass);
    o.u8(t.group);
    o.u8(t.priority);
    o.0.extend_from_slice(&[0; 16]);
}

fn army(o: &mut Out, v: Version, a: &Army) {
    for slot in a {
        match slot {
            Some(s) => stack(o, v, *s),
            None => {
                o.id(v, if v == Version::RoE { 0xFF } else { 0xFFFF });
                o.u16(0);
            }
        }
    }
}

fn stack(o: &mut Out, v: Version, s: Stack) {
    o.id(v, s.creature);
    o.u16(s.count);
}

fn message(o: &mut Out, v: Version, m: Option<&Message>) {
    o.bool(m.is_some());
    if let Some(m) = m {
        o.text(&m.text);
        o.bool(m.guards.is_some());
        if let Some(g) = &m.guards {
            army(o, v, g);
        }
        o.0.extend_from_slice(&[0; 4]);
    }
}

fn count(o: &mut Out, n: usize) {
    o.u8(u8::try_from(n).unwrap());
}

fn rewards(o: &mut Out, v: Version, r: &Rewards) {
    o.u32(r.experience);
    o.i32(r.mana);
    o.0.push(r.morale.to_le_bytes()[0]);
    o.0.push(r.luck.to_le_bytes()[0]);
    for &x in &r.resources {
        o.i32(x);
    }
    o.0.extend_from_slice(&r.primary);
    count(o, r.skills.len());
    for &(skill, level) in &r.skills {
        o.0.extend_from_slice(&[skill, level]);
    }
    count(o, r.artifacts.len());
    for &a in &r.artifacts {
        o.id(v, a);
    }
    count(o, r.spells.len());
    o.0.extend_from_slice(&r.spells);
    count(o, r.creatures.len());
    for &c in &r.creatures {
        stack(o, v, c);
    }
    o.0.extend_from_slice(&[0; 8]);
}

fn quest(o: &mut Out, v: Version, q: Option<&Quest>) {
    let Some(q) = q else {
        o.u8(0);
        return;
    };
    match &q.mission {
        Mission::Level(n) => {
            o.u8(1);
            o.u32(*n);
        }
        Mission::PrimarySkills(p) => {
            o.u8(2);
            o.0.extend_from_slice(p);
        }
        Mission::KillHero(n) => {
            o.u8(3);
            o.u32(*n);
        }
        Mission::KillCreature(n) => {
            o.u8(4);
            o.u32(*n);
        }
        Mission::Artifacts(a) => {
            o.u8(5);
            count(o, a.len());
            for &a in a {
                o.id(v, a);
            }
        }
        Mission::Army(a) => {
            o.u8(6);
            count(o, a.len());
            for &s in a {
                stack(o, v, s);
            }
        }
        Mission::Resources(r) => {
            o.u8(7);
            for &x in r {
                o.u32(x);
            }
        }
        Mission::Hero(h) => o.0.extend_from_slice(&[8, *h]),
        Mission::Player(p) => o.0.extend_from_slice(&[9, *p]),
    }
    o.i32(q.deadline);
    o.text(&q.first_visit);
    o.text(&q.next_visit);
    o.text(&q.completed);
}

fn reward(o: &mut Out, v: Version, r: Reward) {
    match r {
        Reward::Nothing => o.u8(0),
        Reward::Experience(n) => {
            o.u8(1);
            o.u32(n);
        }
        Reward::Mana(n) => {
            o.u8(2);
            o.u32(n);
        }
        Reward::Morale(n) => o.0.extend_from_slice(&[3, n.to_le_bytes()[0]]),
        Reward::Luck(n) => o.0.extend_from_slice(&[4, n.to_le_bytes()[0]]),
        Reward::Resource { resource, amount } => {
            o.0.extend_from_slice(&[5, resource]);
            o.u32(amount);
        }
        Reward::PrimarySkill { skill, amount } => o.0.extend_from_slice(&[6, skill, amount]),
        Reward::SecondarySkill { skill, level } => o.0.extend_from_slice(&[7, skill, level]),
        Reward::Artifact(a) => {
            o.u8(8);
            o.id(v, a);
        }
        Reward::Spell(s) => o.0.extend_from_slice(&[9, s]),
        Reward::Creatures(s) => {
            o.u8(10);
            stack(o, v, s);
        }
    }
}

pub fn timed_event(o: &mut Out, v: Version, e: &TimedEvent) {
    o.text(&e.name);
    o.text(&e.message);
    for &r in &e.resources {
        o.i32(r);
    }
    o.u8(e.players);
    if v == Version::Sod {
        o.bool(e.human);
    }
    o.bool(e.computer);
    o.u16(e.first_day);
    o.u16(e.interval);
    o.0.extend_from_slice(&[0; 16]);
}

fn opt_text(o: &mut Out, t: Option<&Text>) {
    o.bool(t.is_some());
    if let Some(t) = t {
        o.text(t);
    }
}

fn hero(o: &mut Out, v: Version, h: &crate::model::HeroBody) {
    if let Some(id) = h.identifier {
        o.u32(id);
    }
    o.u8(h.owner);
    o.u8(h.hero_type);
    opt_text(o, h.name.as_ref());
    if v == Version::Sod {
        o.bool(h.experience.is_some());
    }
    if let Some(x) = h.experience {
        o.u32(x);
    }
    o.bool(h.portrait.is_some());
    if let Some(p) = h.portrait {
        o.u8(p);
    }
    o.bool(h.secondary_skills.is_some());
    if let Some(s) = &h.secondary_skills {
        o.len(s.len());
        for &(skill, level) in s {
            o.0.extend_from_slice(&[skill, level]);
        }
    }
    o.bool(h.army.is_some());
    if let Some(a) = &h.army {
        army(o, v, a);
    }
    o.u8(h.formation);
    artifacts(o, v, h.artifacts.as_ref());
    o.u8(h.patrol);
    if v != Version::RoE {
        opt_text(o, h.biography.as_ref());
        o.u8(h.sex);
    }
    if let Some(s) = h.spell {
        o.u8(s);
    }
    if v == Version::Sod {
        o.bool(h.spells.is_some());
        if let Some(s) = &h.spells {
            o.bits(s);
        }
        o.bool(h.primary_skills.is_some());
        if let Some(p) = h.primary_skills {
            o.0.extend_from_slice(&p);
        }
    }
    o.0.extend_from_slice(&[0; 16]);
}

fn town(o: &mut Out, v: Version, t: &crate::model::TownBody) {
    if let Some(id) = t.identifier {
        o.u32(id);
    }
    o.u8(t.owner);
    opt_text(o, t.name.as_ref());
    o.bool(t.garrison.is_some());
    if let Some(g) = &t.garrison {
        army(o, v, g);
    }
    o.u8(t.formation);
    match &t.buildings {
        Buildings::Custom { built, forbidden } => {
            o.bool(true);
            o.0.extend_from_slice(built);
            o.0.extend_from_slice(forbidden);
        }
        Buildings::Standard { fort } => {
            o.bool(false);
            o.bool(*fort);
        }
    }
    if let Some(r) = &t.required_spells {
        o.bits(r);
    }
    o.bits(&t.banned_spells);
    o.len(t.events.len());
    for e in &t.events {
        timed_event(o, v, &e.event);
        o.0.extend_from_slice(&e.buildings);
        for &n in &e.creatures {
            o.u16(n);
        }
        o.0.extend_from_slice(&[0; 4]);
    }
    if v == Version::Sod {
        o.u8(t.alignment);
    }
    o.0.extend_from_slice(&[0; 3]);
}

fn body(o: &mut Out, v: Version, b: &Body) {
    match b {
        Body::None => {}
        Body::Event(e) => {
            message(o, v, e.message.as_ref());
            rewards(o, v, &e.rewards);
            o.u8(e.players);
            o.bool(e.computer);
            o.bool(e.remove_after_visit);
            o.0.extend_from_slice(&[0; 4]);
        }
        Body::Hero(h) => hero(o, v, h),
        Body::Monster(m) => {
            if let Some(id) = m.identifier {
                o.u32(id);
            }
            o.u16(m.count);
            o.u8(m.disposition);
            o.bool(m.treasure.is_some());
            if let Some(t) = &m.treasure {
                o.text(&t.text);
                for &r in &t.resources {
                    o.i32(r);
                }
                o.artifact(v, t.artifact);
            }
            o.bool(m.never_flees);
            o.bool(m.no_growth);
            o.0.extend_from_slice(&[0; 2]);
        }
        Body::Sign(t) => {
            o.text(t);
            o.0.extend_from_slice(&[0; 4]);
        }
        Body::SeerHut(task) => {
            match (task, v) {
                (Some(t), Version::RoE) => {
                    let Mission::Artifacts(a) = &t.quest.mission else {
                        panic!("an RoE seer hut asks for an artifact")
                    };
                    o.artifact(v, Some(a[0]));
                    reward(o, v, t.reward);
                }
                (Some(t), _) => {
                    quest(o, v, Some(&t.quest));
                    reward(o, v, t.reward);
                }
                (None, Version::RoE) => {
                    o.artifact(v, None);
                    o.u8(0);
                }
                (None, _) => {
                    quest(o, v, None);
                    o.u8(0);
                }
            }
            o.0.extend_from_slice(&[0; 2]);
        }
        Body::WitchHut(b) | Body::AbandonedMine(b) => o.bits(b),
        Body::Scholar { kind, code } => {
            o.0.extend_from_slice(&[kind.to_le_bytes()[0], *code]);
            o.0.extend_from_slice(&[0; 6]);
        }
        Body::Garrison {
            owner,
            army: a,
            removable,
        } => {
            o.u32(*owner);
            army(o, v, a);
            if v != Version::RoE {
                o.bool(*removable);
            }
            o.0.extend_from_slice(&[0; 8]);
        }
        Body::Artifact(m) => message(o, v, m.as_ref()),
        Body::Scroll { message: m, spell } => {
            message(o, v, m.as_ref());
            o.u32(*spell);
        }
        Body::Resource { message: m, amount } => {
            message(o, v, m.as_ref());
            o.u32(*amount);
            o.0.extend_from_slice(&[0; 4]);
        }
        Body::Town(t) => town(o, v, t),
        Body::Mine { owner } | Body::Owned { owner } => o.u32(*owner),
        Body::RandomDwelling(d) => {
            o.u32(d.owner);
            if let Some((town, set)) = &d.faction {
                o.u32(*town);
                if let Some(set) = set {
                    o.bits(set);
                }
            }
            if let Some((low, high)) = d.levels {
                o.0.extend_from_slice(&[low, high]);
            }
        }
        Body::Shrine { spell } => o.u32(*spell),
        Body::Pandora {
            message: m,
            rewards: r,
        } => {
            message(o, v, m.as_ref());
            rewards(o, v, r);
        }
        Body::Grail { radius } => o.i32(*radius),
        Body::QuestGuard(q) => quest(o, v, q.as_ref()),
        Body::HeroPlaceholder {
            owner,
            hero_type,
            power_rank,
        } => {
            o.0.extend_from_slice(&[*owner, *hero_type]);
            if let Some(r) = power_rank {
                o.u8(*r);
            }
        }
    }
}
