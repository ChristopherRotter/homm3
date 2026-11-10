//! The whole game's mapping (E16 M5 part 1, SRV-Q25; ADR-0018 points 5 and 7 as amended): the
//! engine's observation of a game on the wire ([`crate::wire::game`]). Every field of the
//! engine's types is named in a pattern, without `..`, and every enum is matched without a
//! wildcard: a new field or variant fails this crate's build until it is mapped. What leaves is
//! projected by construction: [`observe`] takes the viewer and calls the engine's `observe`.

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

use h3_adventure::{
    AdventureObservation, BoatView, Choice as EngineChoice, ChoiceKind as EngineChoiceKind,
    GuardsView, HeroView, HireTarget as EngineHireTarget, Known as EngineKnown, KnownObject,
    MonsterView, ObjectTypeId, ObjectView, Offer as EngineOffer, OfferKind as EngineOfferKind,
    OwnHeroView, StaticTile, TileMask, TileView, UniversityVisit as EngineVisit,
};
use h3_content::{Formation as EngineFormation, ResourceId};
use h3_contracts::{ArmyStack as EngineStack, FortLevel};
use h3_game::{
    DecisionView, GameRuleset, GameState, GarrisonStackView, GarrisonView, HeroInfo as EngineInfo,
    HireCreature as EngineCreature, HireView, LevelUpView, MonthKind, PlayerView,
    SlotState as EngineSlot, TownBuildingsView, TownOwnerView, TownView, Viewer,
};
use h3_heroes::{
    ArtifactInstance, BattleBonuses, HeroDetails as EngineDetails, OfferLevels as EngineLevels,
    PrimarySkill as EnginePrimary,
};
use h3_kernel::handle::Handle;

use crate::wire::WireU64;
use crate::wire::game::{
    Adventure, Amount, ArmyStack, Artifact, Boat, Bonuses, Choice, ChoiceKind, Decision, Formation,
    GameObservation, Garrison, GarrisonStack, Guards, HeroDetails, HeroInfo, Hire, HireCreature,
    HireTarget, Known, KnownObject as WireKnownObject, LevelUp, MapHero, Monster, Month, Object,
    ObjectRef, Offer, OfferKind, OfferLevels, OwnMapHero, Player, Pos, PrimarySkill, SkillLevel,
    SlotState, Tile, Town, TownBuildings, TownOwnerFacts, UniversityVisit,
};

/// What `viewer` may know of the game, on the wire (`h3_game::observe`).
pub fn observe(state: &GameState, rules: &GameRuleset, viewer: Viewer) -> GameObservation {
    observation(&h3_game::observe(state, rules, viewer))
}

/// A game observation on the wire.
pub(crate) fn observation(o: &h3_game::GameObservation) -> GameObservation {
    let h3_game::GameObservation {
        viewer,
        day,
        month,
        players,
        turn_group,
        treasury,
        events_today,
        decision,
        heroes,
        adventure,
        towns,
        battle,
    } = o;
    GameObservation {
        viewer: viewer.raw(),
        day: *day,
        month: match month {
            MonthKind::Normal => Month::Normal {},
            MonthKind::Creature(creature) => Month::Creature {
                creature: creature.raw(),
            },
            MonthKind::Plague => Month::Plague {},
        },
        players: players
            .iter()
            .map(|&PlayerView { colour, acting }| Player { colour, acting })
            .collect(),
        turn_group: turn_group.iter().map(|p| p.raw()).collect(),
        treasury: treasury.clone(),
        events_today: events_today.clone(),
        decision: decision.as_ref().map(decision_view),
        heroes: heroes.iter().map(hero_info).collect(),
        adventure: adventure_view(adventure),
        towns: towns.iter().map(town).collect(),
        battle: battle.as_ref().map(crate::map::observation),
    }
}

/// An open decision on the wire.
fn decision_view(d: &DecisionView) -> Decision {
    match d {
        DecisionView::Offer(o) => Decision::Offer { offer: offer(o) },
        DecisionView::LevelUp(LevelUpView {
            hero,
            level,
            primary,
            offers,
        }) => Decision::LevelUp {
            level_up: LevelUp {
                hero: hero.raw(),
                level: *level,
                primary: primary_skill(*primary),
                offers: offers.map(Handle::raw),
            },
        },
        DecisionView::Meeting { heroes } => Decision::Meeting {
            heroes: heroes.map(h3_contracts::HeroId::raw),
        },
        DecisionView::ArmyDialog { hero, group, mine } => Decision::ArmyDialog {
            hero: hero.raw(),
            group: army(group),
            mine: mine.map(object_ref),
        },
        DecisionView::Choice(c) => Decision::Choice { choice: choice(c) },
        DecisionView::University { visit, slots } => {
            let EngineVisit {
                hero,
                object_type,
                anchor,
                skills,
                price,
            } = visit;
            Decision::University {
                visit: UniversityVisit {
                    hero: hero.raw(),
                    object_type: object_type.raw(),
                    anchor: pos(*anchor),
                    skills: skills.iter().map(|&s| s.raw()).collect(),
                    price: amounts(price),
                },
                slots: slots
                    .iter()
                    .map(|slot| match slot {
                        EngineSlot::Known => SlotState::Known,
                        EngineSlot::NoRoom => SlotState::NoRoom,
                        EngineSlot::Barred => SlotState::Barred,
                        EngineSlot::Open => SlotState::Open,
                    })
                    .collect(),
            }
        }
        DecisionView::Hire(h) => Decision::Hire { hire: hire(h) },
    }
}

/// An offer on the wire.
fn offer(o: &EngineOffer) -> Offer {
    let EngineOffer {
        hero,
        object_type,
        anchor,
        kind,
    } = o;
    Offer {
        hero: hero.raw(),
        object_type: object_type.raw(),
        anchor: pos(*anchor),
        kind: match kind {
            EngineOfferKind::Reveal { price } => OfferKind::Reveal {
                price: amounts(price),
            },
            EngineOfferKind::Pass => OfferKind::Pass {},
            EngineOfferKind::Join => OfferKind::Join {},
            EngineOfferKind::JoinForGold { price } => OfferKind::JoinForGold {
                price: amounts(price),
            },
            EngineOfferKind::Pursue => OfferKind::Pursue {},
            EngineOfferKind::Fight => OfferKind::Fight {},
            EngineOfferKind::Gift { price } => OfferKind::Gift {
                price: amounts(price),
            },
            EngineOfferKind::Search => OfferKind::Search {},
            EngineOfferKind::OpenBox => OfferKind::OpenBox {},
            EngineOfferKind::SearchPyramid => OfferKind::SearchPyramid {},
            EngineOfferKind::Boat { price } => OfferKind::Boat {
                price: amounts(price),
            },
            EngineOfferKind::Hire { creatures } => OfferKind::Hire {
                creatures: creatures.iter().map(|&c| c.raw()).collect(),
            },
            EngineOfferKind::Guarded {
                creature,
                count_word,
            } => OfferKind::Guarded {
                creature: creature.raw(),
                count_word: *count_word,
            },
        },
    }
}

/// A primary skill on the wire.
const fn primary_skill(skill: EnginePrimary) -> PrimarySkill {
    match skill {
        EnginePrimary::Attack => PrimarySkill::Attack,
        EnginePrimary::Defense => PrimarySkill::Defense,
        EnginePrimary::Power => PrimarySkill::Power,
        EnginePrimary::Knowledge => PrimarySkill::Knowledge,
    }
}

/// A visit's choice on the wire.
fn choice(c: &EngineChoice) -> Choice {
    let EngineChoice {
        hero,
        object_type,
        anchor,
        kind,
    } = c;
    Choice {
        hero: hero.raw(),
        object_type: object_type.raw(),
        anchor: pos(*anchor),
        kind: match kind {
            EngineChoiceKind::Chest { gold, experience } => ChoiceKind::Chest {
                gold: amount(*gold),
                experience: *experience,
            },
            EngineChoiceKind::Primary {
                skills,
                amount: gain,
                price,
                leave,
            } => ChoiceKind::Primary {
                skills: skills.map(primary_skill),
                amount: *gain,
                price: amounts(price),
                leave: *leave,
            },
        },
    }
}

/// A hire dialog on the wire.
fn hire(h: &HireView) -> Hire {
    let HireView {
        target,
        object_type,
        anchor,
        creatures,
    } = h;
    Hire {
        target: match target {
            EngineHireTarget::Hero(hero) => HireTarget::Hero { hero: hero.raw() },
        },
        object_type: object_type.raw(),
        anchor: pos(*anchor),
        creatures: creatures
            .iter()
            .map(
                |EngineCreature {
                     creature,
                     count,
                     cost,
                 }| HireCreature {
                    creature: creature.raw(),
                    count: *count,
                    cost: amounts(cost),
                },
            )
            .collect(),
    }
}

/// What a viewer knows of a hero on the wire.
fn hero_info(h: &EngineInfo) -> HeroInfo {
    let EngineInfo {
        id,
        hero_type,
        details,
    } = h;
    HeroInfo {
        id: id.raw(),
        hero_type: hero_type.raw(),
        details: details.as_deref().map(hero_details),
    }
}

/// A hero's details on the wire.
fn hero_details(d: &EngineDetails) -> HeroDetails {
    let EngineDetails {
        hero_type,
        level,
        experience,
        offered,
        primary,
        skills,
        worn,
        backpack,
        spells,
        mana,
        bonuses,
    } = d;
    let EngineLevels { wisdom, school } = *offered;
    let BattleBonuses {
        morale,
        luck,
        marks,
    } = *bonuses;
    HeroDetails {
        hero_type: hero_type.raw(),
        level: *level,
        experience: *experience,
        offered: OfferLevels { wisdom, school },
        primary: *primary,
        skills: skills
            .iter()
            .map(|&(skill, level)| SkillLevel {
                skill: skill.raw(),
                level,
            })
            .collect(),
        worn: worn.iter().map(|a| a.map(artifact)).collect(),
        backpack: backpack.iter().copied().map(artifact).collect(),
        spells: spells.iter().map(|&s| s.raw()).collect(),
        mana: *mana,
        bonuses: Bonuses {
            morale,
            luck,
            marks,
        },
    }
}

/// An artifact on the wire.
const fn artifact(a: ArtifactInstance) -> Artifact {
    let ArtifactInstance { artifact, spell } = a;
    Artifact {
        artifact: artifact.raw(),
        spell: match spell {
            Some(s) => Some(s.raw()),
            None => None,
        },
    }
}

/// A town as its viewer sees it on the wire.
fn town(t: &TownView) -> Town {
    let TownView {
        town,
        anchor,
        owner,
        faction,
        name,
        fort,
        capitol,
        built_today,
        garrison,
        owner_facts,
        buildings,
    } = t;
    let GarrisonView {
        level,
        stacks,
        army: garrison_army,
    } = garrison;
    Town {
        town: town.raw(),
        anchor: pos(*anchor),
        owner: owner.map(h3_contracts::PlayerId::raw),
        faction: faction.raw(),
        name: name.as_deref().map(String::from),
        fort: match fort {
            FortLevel::None => 0,
            FortLevel::Fort => 1,
            FortLevel::Citadel => 2,
            FortLevel::Castle => 3,
        },
        capitol: *capitol,
        built_today: *built_today,
        garrison: Garrison {
            level: *level,
            stacks: stacks
                .iter()
                .map(
                    |&GarrisonStackView {
                         creature,
                         count_word,
                         count,
                     }| GarrisonStack {
                        creature: creature.raw(),
                        count_word,
                        count,
                    },
                )
                .collect(),
            army: garrison_army.as_deref().map(army),
        },
        owner_facts: owner_facts
            .as_ref()
            .map(|TownOwnerView { hall, income }| TownOwnerFacts {
                hall: *hall,
                income: amounts(income),
            }),
        buildings: buildings.map(|TownBuildingsView { built, forbidden }| TownBuildings {
            built: WireU64(built.bits()),
            forbidden: WireU64(forbidden.bits()),
        }),
    }
}

/// Amounts of resources on the wire.
fn amounts(price: &[(ResourceId, u32)]) -> Vec<Amount> {
    price.iter().copied().map(amount).collect()
}

/// An adventure observation on the wire.
pub(crate) fn adventure_view(o: &AdventureObservation) -> Adventure {
    let AdventureObservation {
        viewer,
        width,
        height,
        levels,
        stride,
        explored,
        fringe,
        tiles,
        heroes,
        objects,
        bought,
        keys,
        known,
        boats,
    } = o;
    let (palette, tiles) = packed(tiles);
    Adventure {
        viewer: viewer.raw(),
        width: *width,
        height: *height,
        levels: *levels,
        // At most 1 024: a width is a `u16`.
        stride: u16::try_from(*stride).unwrap_or(u16::MAX),
        explored: words(explored),
        fringe: words(fringe),
        palette,
        tiles,
        heroes: heroes.iter().map(map_hero).collect(),
        objects: objects.iter().map(object).collect(),
        bought: bought.iter().map(|&t| t.raw()).collect(),
        keys: *keys,
        known: known.iter().map(known_object).collect(),
        boats: boats.iter().map(boat).collect(),
    }
}

/// The tiles as their distinct views, in the order of their first use, and each tile's index
/// among them.
fn packed(tiles: &[TileView]) -> (Vec<Tile>, Vec<u32>) {
    let mut index = BTreeMap::new();
    let mut palette = Vec::new();
    let packed = tiles
        .iter()
        .map(|&view| {
            let view = tile(view);
            *index.entry(view).or_insert_with(|| {
                // Below the tile count, which a map's `u16` sides and `u8` levels bound.
                let next = u32::try_from(palette.len()).unwrap_or(u32::MAX);
                palette.push(view);
                next
            })
        })
        .collect();
    (palette, packed)
}

/// A plane's words on the wire.
fn words(plane: &[u64]) -> Vec<WireU64> {
    plane.iter().map(|&w| WireU64(w)).collect()
}

/// A tile's view on the wire.
fn tile(view: TileView) -> Tile {
    match view {
        TileView::Hidden => Tile::Hidden {},
        TileView::Ground {
            terrain,
            road,
            river,
        } => Tile::Ground {
            terrain: terrain.raw(),
            road: road.map(Handle::raw),
            river: river.map(Handle::raw),
        },
        TileView::Seen(StaticTile {
            terrain,
            road,
            river,
            blocked,
            visitable,
            no_native,
            winds,
            hole,
        }) => Tile::Seen {
            terrain: terrain.raw(),
            road: road.map(Handle::raw),
            river: river.map(Handle::raw),
            blocked,
            visitable,
            no_native,
            winds,
            hole,
        },
    }
}

/// A tile's position on the wire.
pub(crate) const fn pos(p: h3_adventure::Pos) -> Pos {
    let h3_adventure::Pos { x, y, level } = p;
    Pos { x, y, level }
}

/// An object by type and anchor on the wire.
pub(crate) const fn object_ref(
    (object_type, anchor): (ObjectTypeId, h3_adventure::Pos),
) -> ObjectRef {
    ObjectRef {
        object_type: object_type.raw(),
        anchor: pos(anchor),
    }
}

/// An amount of a resource on the wire.
pub(crate) const fn amount((resource, amount): (ResourceId, u32)) -> Amount {
    Amount {
        resource: resource.raw(),
        amount,
    }
}

/// An army's stacks on the wire.
pub(crate) fn army(stacks: &[EngineStack]) -> Vec<ArmyStack> {
    stacks
        .iter()
        .map(
            |&EngineStack {
                 slot,
                 creature,
                 count,
             }| ArmyStack {
                slot,
                creature: creature.raw(),
                count,
            },
        )
        .collect()
}

/// A tile mask on the wire: its bits.
const fn mask(m: TileMask) -> WireU64 {
    WireU64(m.bits())
}

/// A hero on the map on the wire.
fn map_hero(h: &HeroView) -> MapHero {
    let HeroView {
        id,
        owner,
        pos: at,
        boat,
        own,
    } = h;
    MapHero {
        id: id.raw(),
        owner: owner.raw(),
        pos: pos(*at),
        boat: *boat,
        own: own.as_ref().map(own_map_hero),
    }
}

/// An own hero's details on the map on the wire.
fn own_map_hero(h: &OwnHeroView) -> OwnMapHero {
    let OwnHeroView {
        army: stacks,
        movement,
        max_movement,
        formation,
        tactics,
        marks,
    } = h;
    OwnMapHero {
        army: army(stacks),
        movement: *movement,
        max_movement: *max_movement,
        formation: match formation {
            EngineFormation::Loose => Formation::Loose,
            EngineFormation::Tight => Formation::Tight,
        },
        tactics: *tactics,
        marks: marks.iter().copied().map(object_ref).collect(),
    }
}

/// An object on the wire.
fn object(o: &ObjectView) -> Object {
    let ObjectView {
        object_type,
        anchor,
        blocked,
        visitable,
        drawn,
        monster,
        owner,
        production,
        guards,
        visited,
        boat_tile,
        town,
    } = o;
    Object {
        object_type: object_type.raw(),
        anchor: pos(*anchor),
        blocked: mask(*blocked),
        visitable: mask(*visitable),
        drawn: mask(*drawn),
        monster: monster.map(
            |MonsterView {
                 creature,
                 count_word,
             }| Monster {
                creature: creature.raw(),
                count_word,
            },
        ),
        owner: owner.map(h3_contracts::PlayerId::raw),
        production: production.map(amount),
        guards: guards.as_ref().map(
            |GuardsView {
                 creature,
                 count_word,
                 army: stacks,
             }| Guards {
                creature: creature.map(Handle::raw),
                count_word: *count_word,
                army: stacks.as_deref().map(army),
            },
        ),
        visited: *visited,
        boat_tile: boat_tile.map(pos),
        town: town.map(h3_contracts::TownId::raw),
    }
}

/// What a player knows of an object on the wire.
fn known_object(k: &KnownObject) -> WireKnownObject {
    let KnownObject {
        object_type,
        anchor,
        known,
    } = k;
    WireKnownObject {
        object_type: object_type.raw(),
        anchor: pos(*anchor),
        known: match known {
            EngineKnown::Spell(spell) => Known::Spell {
                spell: spell.map(Handle::raw),
            },
            EngineKnown::Skill(skill) => Known::Skill {
                skill: skill.map(Handle::raw),
            },
            EngineKnown::Price(price) => Known::Price {
                price: amounts(price),
            },
            EngineKnown::Skills(skills) => Known::Skills {
                skills: skills.iter().map(|&s| s.raw()).collect(),
            },
            EngineKnown::Emptied(emptied) => Known::Emptied { emptied: *emptied },
        },
    }
}

/// An empty boat on the wire.
const fn boat(b: &BoatView) -> Boat {
    let BoatView { tile, kind, facing } = *b;
    Boat {
        tile: pos(tile),
        kind,
        facing,
    }
}
