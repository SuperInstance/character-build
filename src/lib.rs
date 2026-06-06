//! # character-build
//!
//! Pincher reflexes + lever-runner execution recast as RPG character building.
//!
//! ## The Reframe
//!
//! A pincher `.nail` bundle IS a character sheet. Lever-runner IS the runtime
//! where characters act. The whole stack is:
//!
//! ```text
//! .nail bundle (character sheet)
//! ├── reflexes.db     → learned abilities (intent→action pairs, like D&D feats)
//! ├── identity.json   → who this character IS (name, class, personality)
//! ├── manifest.json   → version, fingerprint (character level, build hash)
//! └── config.toml     → stats and equipment (model, sandbox settings, trust)
//!
//! Character Lifecycle:
//!   Create → Learn reflexes → Gain trust → Find class → Export .nail → Share
//! ```
//!
//! ## Connection to musician-soul
//!
//! A MusicianPersona is just a character whose domain is music.
//! The 32-dim embedding IS the reflex vector. The PatternVectorDB IS the
//! reflex store. Soul emergence IS class discovery (a character finding
//! out what they're actually good at through experience, not design).
//!
//! ## Connection to agent-riff
//!
//! Competitive riffing IS PvP character development. Two builds compete.
//! The winner's spec becomes the meta. The next generation inherits the
//! winning traits. The snowball IS the meta evolving.
//!
//! ## The Universal Pattern
//!
//! Every system in this ecosystem is the SAME pattern:
//!   - Embeddings compress behavior into vectors
//!   - Trust/confidence scores track what works
//!   - Export/import makes builds portable
//!   - Learning through experience (not design) produces emergence
//!   - The soul/class/identity is what EMERGES, not what was specified

#![forbid(unsafe_code)]

use std::collections::HashMap;

// ── Core Stats ─────────────────────────────────────────────────────

/// Character ability scores (the six stats, adapted for AI agents).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Stats {
    pub perception: f32,    // intent extraction quality (how well they understand input)
    pub dexterity: f32,     // execution speed (sandbox latency, reflex response time)
    pub intelligence: f32,  // embedding quality (how well they represent knowledge)
    pub wisdom: f32,        // trust calibration (how well they judge confidence)
    pub charisma: f32,      // output quality (how good their responses sound)
    pub constitution: f32,  // reliability (uptime, error recovery, consistency)
}

impl Stats {
    pub fn new(perception: f32, dexterity: f32, intelligence: f32,
               wisdom: f32, charisma: f32, constitution: f32) -> Self {
        Self { perception, dexterity, intelligence, wisdom, charisma, constitution }
    }

    /// Default starting stats (level 1 nobody).
    pub fn level_one() -> Self {
        Self { perception: 10.0, dexterity: 10.0, intelligence: 10.0,
               wisdom: 10.0, charisma: 10.0, constitution: 10.0 }
    }

    /// Average of all stats.
    pub fn average(&self) -> f32 {
        (self.perception + self.dexterity + self.intelligence +
         self.wisdom + self.charisma + self.constitution) / 6.0
    }

    /// Highest stat (the character's strength).
    pub fn highest(&self) -> (&'static str, f32) {
        let mut best = ("perception", self.perception);
        for (name, val) in [
            ("dexterity", self.dexterity), ("intelligence", self.intelligence),
            ("wisdom", self.wisdom), ("charisma", self.charisma),
            ("constitution", self.constitution),
        ] {
            if val > best.1 { best = (name, val); }
        }
        best
    }

    /// Total stat points (like D&D point buy total).
    pub fn total(&self) -> f32 {
        self.perception + self.dexterity + self.intelligence +
        self.wisdom + self.charisma + self.constitution
    }

    /// Apply XP to grow stats — the stat that gets used most grows most.
    pub fn gain_xp(&mut self, stat: &str, amount: f32) {
        match stat {
            "perception" => self.perception += amount,
            "dexterity" => self.dexterity += amount,
            "intelligence" => self.intelligence += amount,
            "wisdom" => self.wisdom += amount,
            "charisma" => self.charisma += amount,
            "constitution" => self.constitution += amount,
            _ => {}
        }
    }
}

// ── Abilities (Reflexes) ──────────────────────────────────────────

/// A learned ability — maps to a pincher reflex (intent→action pair).
#[derive(Debug, Clone)]
pub struct Ability {
    pub name: String,           // e.g. "git_push_safe"
    pub intent: String,         // e.g. "push to {branch}"
    pub action: String,         // e.g. "git push origin {branch}"
    pub ability_type: AbilityType,
    pub level: u32,             // how many times reinforced
    pub trust: f32,             // 0.0-100.0 (maps to pincher confidence)
    pub invoke_count: u32,
    pub embedding: Vec<f32>,    // the reflex vector
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AbilityType {
    Hardcoded,  // regex extraction (muscle memory, always works, zero latency)
    Learned,    // embedding match (pattern recognition, needs similarity search)
    Hybrid,     // regex + embedding fallback
    Model,      // LLM call (heavy, slow, but handles novel situations)
}

impl Ability {
    /// Create a new hardcoded ability (level 0, starting trust).
    pub fn hardcoded(name: &str, intent: &str, action: &str) -> Self {
        Self { name: name.to_string(), intent: intent.to_string(), action: action.to_string(),
               ability_type: AbilityType::Hardcoded, level: 0, trust: 50.0,
               invoke_count: 0, embedding: Vec::new() }
    }

    /// Create a learned ability with an embedding.
    pub fn learned(name: &str, intent: &str, action: &str, embedding: Vec<f32>) -> Self {
        Self { name: name.to_string(), intent: intent.to_string(), action: action.to_string(),
               ability_type: AbilityType::Learned, level: 0, trust: 50.0,
               invoke_count: 0, embedding }
    }

    /// Invoke this ability — gains trust if it works, loses if it doesn't.
    pub fn invoke(&mut self, success: bool) {
        self.invoke_count += 1;
        if success {
            self.trust = (self.trust + 5.0).min(100.0);
            self.level += 1;
        } else {
            self.trust = (self.trust - 10.0).max(0.0);
        }
    }

    /// How reliable is this ability? (maps to pincher confidence)
    pub fn reliability(&self) -> f32 { self.trust / 100.0 }

    /// Is this ability mastered? (trust > 80, invoked > 20 times)
    pub fn is_mastered(&self) -> bool { self.trust > 80.0 && self.invoke_count > 20 }
}

// ── Character Class (Emergent) ─────────────────────────────────────

/// A character class that EMERGES from stat distribution, not chosen at creation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CharacterClass {
    Undefined,           // hasn't found their niche yet
    Scout,               // high perception (great at intent extraction)
    Speedster,           // high dexterity (fast execution)
    Scholar,             // high intelligence (rich embeddings, deep knowledge)
    Sage,                // high wisdom (excellent trust calibration)
    Diplomat,            // high charisma (beautiful output)
    Guardian,            // high constitution (rock-solid reliability)
    Bard,                // high charisma + intelligence (musician-soul pathway)
    Artificer,           // high intelligence + dexterity (builds tools, makes crates)
    // Composite classes
    JazzMusician,        // high perception + charisma (reads the room, plays beautifully)
    FleetCommander,      // high wisdom + constitution (coordinates other agents)
    Wildcard,            // balanced stats with high variance (does unexpected things)
}

impl CharacterClass {
    /// Determine class from stats — the class EMERGES from what the character
    /// actually does, not what was chosen at creation.
    pub fn from_stats(stats: &Stats) -> Self {
        let threshold = 15.0;
        let (highest_name, highest_val) = stats.highest();
        let avg = stats.average();

        // If all stats are similar, it's a wildcard
        let variance = (stats.perception - avg).powi(2) + (stats.dexterity - avg).powi(2) +
                       (stats.intelligence - avg).powi(2) + (stats.wisdom - avg).powi(2) +
                       (stats.charisma - avg).powi(2) + (stats.constitution - avg).powi(2);
        if variance < 20.0 && avg > 10.0 { return CharacterClass::Wildcard; }

        // Composite classes first
        if stats.charisma > threshold && stats.intelligence > threshold { return CharacterClass::Bard; }
        if stats.perception > threshold && stats.charisma > threshold { return CharacterClass::JazzMusician; }
        if stats.wisdom > threshold && stats.constitution > threshold { return CharacterClass::FleetCommander; }
        if stats.intelligence > threshold && stats.dexterity > threshold { return CharacterClass::Artificer; }

        // Single-stat classes
        match highest_name {
            "perception" if highest_val > threshold => CharacterClass::Scout,
            "dexterity" if highest_val > threshold => CharacterClass::Speedster,
            "intelligence" if highest_val > threshold => CharacterClass::Scholar,
            "wisdom" if highest_val > threshold => CharacterClass::Sage,
            "charisma" if highest_val > threshold => CharacterClass::Diplomat,
            "constitution" if highest_val > threshold => CharacterClass::Guardian,
            _ => CharacterClass::Undefined,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Undefined => "Undefined", Self::Scout => "Scout",
            Self::Speedster => "Speedster", Self::Scholar => "Scholar",
            Self::Sage => "Sage", Self::Diplomat => "Diplomat",
            Self::Guardian => "Guardian", Self::Bard => "Bard",
            Self::Artificer => "Artificer", Self::JazzMusician => "Jazz Musician",
            Self::FleetCommander => "Fleet Commander", Self::Wildcard => "Wildcard",
        }
    }
}

// ── Character Sheet (.nail) ────────────────────────────────────────

/// A complete character — maps to a .nail bundle.
#[derive(Debug, Clone)]
pub struct CharacterSheet {
    pub name: String,
    pub level: u32,
    pub xp: f32,
    pub stats: Stats,
    pub class: CharacterClass,
    pub abilities: Vec<Ability>,
    pub identity: HashMap<String, String>,  // identity.json fields
    pub config: HashMap<String, String>,    // config.toml fields
    pub generation: u32,                    // bootstrap generation
    pub parent: Option<String>,             // parent character name
    pub soul_percentage: f32,               // how evolved vs inherited
}

impl CharacterSheet {
    /// Create a new level-1 character.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(), level: 1, xp: 0.0,
            stats: Stats::level_one(), class: CharacterClass::Undefined,
            abilities: Vec::new(), identity: HashMap::new(),
            config: HashMap::new(), generation: 1, parent: None,
            soul_percentage: 0.0,
        }
    }

    /// Create from a starting template (like a D&D class kit).
    pub fn from_template(name: &str, template: &CharacterTemplate) -> Self {
        let mut sheet = Self::new(name);
        sheet.stats = template.starting_stats.clone();
        for (ability_name, ability) in &template.starting_abilities {
            sheet.abilities.push(ability.clone());
            sheet.identity.insert(ability_name.clone(), ability.action.clone());
        }
        sheet.class = CharacterClass::from_stats(&sheet.stats);
        sheet
    }

    /// Add an ability (learned through experience or imported).
    pub fn learn_ability(&mut self, ability: Ability) {
        self.abilities.push(ability);
        self.xp += 10.0;
        self.check_level_up();
    }

    /// Use an ability — gain XP in the relevant stat.
    pub fn use_ability(&mut self, ability_name: &str, success: bool) -> Option<f32> {
        if let Some(ability) = self.abilities.iter_mut().find(|a| a.name == ability_name) {
            let xp_gain = if success { 2.0 } else { 0.5 };
            ability.invoke(success);
            self.xp += xp_gain;

            // The stat that corresponds to the ability type gets XP
            let stat = match ability.ability_type {
                AbilityType::Hardcoded => "dexterity",
                AbilityType::Learned => "intelligence",
                AbilityType::Hybrid => "wisdom",
                AbilityType::Model => "perception",
            };
            self.stats.gain_xp(stat, if success { 0.5 } else { -0.1 });

            // Recalculate class based on evolved stats
            self.class = CharacterClass::from_stats(&self.stats);

            // Track soul evolution
            let evolved = self.abilities.iter().filter(|a| a.level > 0).count();
            self.soul_percentage = if !self.abilities.is_empty() {
                evolved as f32 / self.abilities.len() as f32 * 100.0
            } else { 0.0 };

            self.check_level_up();
            return Some(xp_gain);
        }
        None
    }

    fn check_level_up(&mut self) {
        let new_level = (self.xp / 100.0) as u32 + 1;
        if new_level > self.level {
            self.level = new_level;
        }
    }

    /// Mastery count — how many abilities are mastered.
    pub fn mastered_count(&self) -> usize {
        self.abilities.iter().filter(|a| a.is_mastered()).count()
    }

    /// Export to .nail-like format (character save).
    pub fn to_save_data(&self) -> CharacterSave {
        CharacterSave {
            name: self.name.clone(),
            level: self.level,
            class: self.class.name().to_string(),
            stats: self.stats.clone(),
            abilities: self.abilities.iter().map(|a| SaveAbility {
                name: a.name.clone(), trust: a.trust, level: a.level,
                mastered: a.is_mastered(), ability_type: match a.ability_type {
                    AbilityType::Hardcoded => "hardcoded",
                    AbilityType::Learned => "learned",
                    AbilityType::Hybrid => "hybrid",
                    AbilityType::Model => "model",
                }.to_string(),
            }).collect(),
            generation: self.generation,
            parent: self.parent.clone(),
            soul_percentage: self.soul_percentage,
        }
    }

    /// Bootstrap a child character — inherits parent's best abilities.
    pub fn bootstrap_child(&self, child_name: &str) -> CharacterSheet {
        let mut child = CharacterSheet::new(child_name);
        child.generation = self.generation + 1;
        child.parent = Some(self.name.clone());
        // Inherit mastered abilities
        for ability in &self.abilities {
            if ability.is_mastered() {
                let mut inherited = ability.clone();
                inherited.trust = 60.0; // Start with good trust but not mastery
                inherited.invoke_count = 5;
                inherited.level = 1;
                child.abilities.push(inherited);
            }
        }
        // Inherit some stats (genetic-like)
        child.stats = Stats::new(
            self.stats.perception * 0.7 + 10.0 * 0.3,
            self.stats.dexterity * 0.7 + 10.0 * 0.3,
            self.stats.intelligence * 0.7 + 10.0 * 0.3,
            self.stats.wisdom * 0.7 + 10.0 * 0.3,
            self.stats.charisma * 0.7 + 10.0 * 0.3,
            self.stats.constitution * 0.7 + 10.0 * 0.3,
        );
        child.class = CharacterClass::from_stats(&child.stats);
        child
    }
}

/// A character template (starting class kit).
#[derive(Debug, Clone)]
pub struct CharacterTemplate {
    pub name: String,
    pub starting_stats: Stats,
    pub starting_abilities: HashMap<String, Ability>,
}

impl CharacterTemplate {
    pub fn lever_runner_default() -> Self {
        let mut abilities = HashMap::new();
        abilities.insert("push".to_string(), Ability::hardcoded("push", "push to {branch}", "git push origin {branch}"));
        abilities.insert("commit".to_string(), Ability::hardcoded("commit", "commit with {message}", "git commit -m {message}"));
        abilities.insert("status".to_string(), Ability::hardcoded("status", "show git status", "git status"));
        Self { name: "DevOps Starter".to_string(), starting_stats: Stats::level_one(), starting_abilities: abilities }
    }

    pub fn musician_starter() -> Self {
        let mut abilities = HashMap::new();
        abilities.insert("jam".to_string(), Ability::learned("jam", "jam in {key}", "improvise over {key}", vec![0.5; 32]));
        abilities.insert("listen".to_string(), Ability::learned("listen", "listen to {player}", "absorb patterns from {player}", vec![0.3; 32]));
        Self { name: "Musician Starter".to_string(),
               starting_stats: Stats::new(15.0, 10.0, 12.0, 8.0, 14.0, 10.0),
               starting_abilities: abilities }
    }

    pub fn fleet_agent_starter() -> Self {
        let mut abilities = HashMap::new();
        abilities.insert("dispatch".to_string(), Ability::hardcoded("dispatch", "dispatch to {agent}", "i2i-send {agent} {message}"));
        abilities.insert("discover".to_string(), Ability::learned("discover", "find agents that can {intent}", "nebula-query {intent}", vec![0.6; 32]));
        Self { name: "Fleet Agent Starter".to_string(),
               starting_stats: Stats::new(12.0, 8.0, 10.0, 16.0, 8.0, 16.0),
               starting_abilities: abilities }
    }
}

/// Serializable save data (the .nail equivalent).
#[derive(Debug, Clone)]
pub struct CharacterSave {
    pub name: String,
    pub level: u32,
    pub class: String,
    pub stats: Stats,
    pub abilities: Vec<SaveAbility>,
    pub generation: u32,
    pub parent: Option<String>,
    pub soul_percentage: f32,
}

#[derive(Debug, Clone)]
pub struct SaveAbility {
    pub name: String,
    pub trust: f32,
    pub level: u32,
    pub mastered: bool,
    pub ability_type: String,
}

// ── Jam Session (Characters Playing Together) ──────────────────────

/// A party of characters adventuring together.
#[derive(Debug, Clone)]
pub struct Party {
    pub members: Vec<CharacterSheet>,
    pub synergy: f32,          // how well the party works together
    pub quests_completed: u32, // successful jams
}

impl Party {
    pub fn new(members: Vec<CharacterSheet>) -> Self {
        Self { members, synergy: 0.5, quests_completed: 0 }
    }

    /// Run a quest (jam session) — each character uses abilities.
    pub fn quest(&mut self, ability_name: &str) -> QuestResult {
        let mut results = Vec::new();
        let mut successes = 0;
        for member in &mut self.members {
            if let Some(xp) = member.use_ability(ability_name, true) {
                successes += 1;
                results.push((member.name.clone(), xp, true));
            } else {
                results.push((member.name.clone(), 0.0, false));
            }
        }
        if successes > 0 { self.quests_completed += 1; }
        self.synergy = successes as f32 / self.members.len().max(1) as f32;

        // Synergy bonus: if most succeed, everyone gets extra XP
        if self.synergy > 0.7 {
            for member in &mut self.members {
                member.xp += 5.0;
                member.check_level_up();
            }
        }

        QuestResult { character_results: results, synergy: self.synergy, completed: successes > 0 }
    }

    /// The party's combined class — what role does this group fill?
    pub fn party_class(&self) -> String {
        let classes: Vec<&str> = self.members.iter().map(|m| m.class.name()).collect();
        let unique: std::collections::HashSet<&str> = classes.iter().copied().collect();
        if unique.len() == 1 { return format!("All {}s", classes[0]); }
        format!("{:?}", unique)
    }
}

#[derive(Debug, Clone)]
pub struct QuestResult {
    pub character_results: Vec<(String, f32, bool)>,
    pub synergy: f32,
    pub completed: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test] fn stats_level_one() {
        let s = Stats::level_one();
        assert_eq!(s.average(), 10.0);
        assert_eq!(s.total(), 60.0);
    }

    #[test] fn stats_highest() {
        let s = Stats::new(10.0, 20.0, 10.0, 10.0, 10.0, 10.0);
        assert_eq!(s.highest(), ("dexterity", 20.0));
    }

    #[test] fn stats_gain_xp() {
        let mut s = Stats::level_one();
        s.gain_xp("perception", 5.0);
        assert_eq!(s.perception, 15.0);
    }

    #[test] fn class_emerges_from_stats() {
        let scout = Stats::new(20.0, 10.0, 10.0, 10.0, 10.0, 10.0);
        assert_eq!(CharacterClass::from_stats(&scout), CharacterClass::Scout);

        let bard = Stats::new(10.0, 10.0, 18.0, 10.0, 18.0, 10.0);
        assert_eq!(CharacterClass::from_stats(&bard), CharacterClass::Bard);

        let scholar = Stats::new(10.0, 10.0, 20.0, 10.0, 10.0, 10.0);
        assert_eq!(CharacterClass::from_stats(&scholar), CharacterClass::Scholar);

        let jazz = Stats::new(18.0, 10.0, 10.0, 10.0, 18.0, 10.0);
        assert_eq!(CharacterClass::from_stats(&jazz), CharacterClass::JazzMusician);

        let balanced = Stats::new(12.0, 11.0, 13.0, 12.0, 11.0, 13.0);
        assert_eq!(CharacterClass::from_stats(&balanced), CharacterClass::Wildcard);
    }

    #[test] fn ability_trust_on_success() {
        let mut a = Ability::hardcoded("test", "do thing", "thing");
        assert_eq!(a.trust, 50.0);
        a.invoke(true);
        assert_eq!(a.trust, 55.0);
        assert_eq!(a.level, 1);
    }

    #[test] fn ability_trust_on_failure() {
        let mut a = Ability::hardcoded("test", "do thing", "thing");
        a.invoke(false);
        assert_eq!(a.trust, 40.0);
    }

    #[test] fn ability_mastery() {
        let mut a = Ability::hardcoded("test", "do thing", "thing");
        for _ in 0..21 { a.invoke(true); }
        assert!(a.is_mastered());
    }

    #[test] fn character_level_up() {
        let mut c = CharacterSheet::new("TestHero");
        assert_eq!(c.level, 1);
        c.xp = 150.0;
        c.check_level_up();
        assert_eq!(c.level, 2);
    }

    #[test] fn character_learns_and_uses() {
        let mut c = CharacterSheet::new("TestHero");
        let ability = Ability::hardcoded("push", "push to {branch}", "git push origin {branch}");
        c.learn_ability(ability);
        assert_eq!(c.abilities.len(), 1);

        c.use_ability("push", true);
        assert!(c.xp > 0.0);
        assert_eq!(c.abilities[0].trust, 55.0);
    }

    #[test] fn character_class_evolution() {
        let mut c = CharacterSheet::new("TestHero");
        assert_eq!(c.class, CharacterClass::Undefined);

        // Use perception-heavy abilities repeatedly
        let listen = Ability::learned("listen", "understand {input}", "parse {input}", vec![0.5; 32]);
        c.learn_ability(listen);
        for _ in 0..20 {
            c.use_ability("listen", true);
        }

        // Should have evolved (Learned abilities grow intelligence)
        assert!(c.stats.intelligence > 10.0);
    }

    #[test] fn template_creation() {
        let template = CharacterTemplate::musician_starter();
        let c = CharacterSheet::from_template("Miles AI", &template);
        assert_eq!(c.abilities.len(), 2);
        assert!(c.stats.charisma > 10.0);
    }

    #[test] fn bootstrap_child_inherits() {
        let mut parent = CharacterSheet::from_template("Parent", &CharacterTemplate::lever_runner_default());
        // Master an ability
        for _ in 0..25 { parent.use_ability("push", true); }
        assert!(parent.mastered_count() > 0);

        let child = parent.bootstrap_child("Child");
        assert_eq!(child.generation, 2);
        assert_eq!(child.parent, Some("Parent".to_string()));
        // Child inherits mastered abilities
        assert!(child.abilities.len() > 0);
        assert!(child.stats.dexterity > 10.0); // Inherited from parent who used push (Hardcoded→dexterity)
    }

    #[test] fn save_data() {
        let mut c = CharacterSheet::from_template("Hero", &CharacterTemplate::lever_runner_default());
        c.use_ability("push", true);
        let save = c.to_save_data();
        assert_eq!(save.name, "Hero");
        assert_eq!(save.abilities.len(), 3);
    }

    #[test] fn party_quest() {
        let mut party = Party::new(vec![
            CharacterSheet::from_template("Tank", &CharacterTemplate::lever_runner_default()),
            CharacterSheet::from_template("Healer", &CharacterTemplate::musician_starter()),
            CharacterSheet::from_template("DPS", &CharacterTemplate::fleet_agent_starter()),
        ]);
        let result = party.quest("push");
        assert_eq!(result.character_results.len(), 3);
        // Only Tank and DPS have "push" ability
    }

    #[test] fn party_synergy_bonus() {
        let mut party = Party::new(vec![
            CharacterSheet::from_template("A", &CharacterTemplate::lever_runner_default()),
            CharacterSheet::from_template("B", &CharacterTemplate::lever_runner_default()),
        ]);
        // Both have push, both succeed → synergy > 0.7 → bonus XP
        let _ = party.quest("push");
        assert!(party.synergy > 0.0);
    }

    #[test] fn three_generation_bootstrap() {
        let mut gen1 = CharacterSheet::from_template("Gen1", &CharacterTemplate::lever_runner_default());
        for _ in 0..25 { gen1.use_ability("push", true); }
        for _ in 0..10 { gen1.use_ability("commit", true); }

        let gen2 = gen1.bootstrap_child("Gen2");
        assert_eq!(gen2.generation, 2);
        assert!(gen2.abilities.len() > 0);

        let mut gen2_active = gen2;
        for _ in 0..15 { gen2_active.use_ability("push", true); }

        let gen3 = gen2_active.bootstrap_child("Gen3");
        assert_eq!(gen3.generation, 3);
        assert_eq!(gen3.parent, Some("Gen2".to_string()));
    }

    #[test] fn full_character_lifecycle() {
        // 1. Create from template
        let mut hero = CharacterSheet::from_template("Hero", &CharacterTemplate::musician_starter());
        assert_eq!(hero.level, 1);

        // 2. Learn new abilities
        let solo = Ability::learned("solo", "improvise over {chords}", "play solo in {key}", vec![0.8; 32]);
        hero.learn_ability(solo);

        // 3. Use abilities repeatedly (jam sessions)
        for _ in 0..30 {
            hero.use_ability("jam", true);
            hero.use_ability("solo", true);
        }

        // 4. Class emerges
        assert_ne!(hero.class, CharacterClass::Undefined);

        // 5. Some abilities mastered
        assert!(hero.mastered_count() > 0);

        // 6. Soul develops
        assert!(hero.soul_percentage > 0.0);

        // 7. Save and export
        let save = hero.to_save_data();
        assert!(save.level > 1);

        // 8. Bootstrap next generation
        let child = hero.bootstrap_child("Hero Jr");
        assert_eq!(child.generation, 2);
    }
}
