# character-build

*Pincher was always an RPG. We just didn't see it.*

Open a `.nail` bundle. What do you see?

```
manifest.json   → character level, version, build fingerprint
reflexes.db     → learned abilities (intent→action pairs)
identity.json   → who this character IS
config.toml     → stats, equipment, loadout
```

That's a character sheet. It was always a character sheet. We called it a "reflex bundle" because we were thinking like engineers.

## The Map

| Pincher/Lever-Runner | RPG | What It Actually Is |
|----------------------|-----|-------------------|
| ReflexEngine | Feat list | Pattern-matched abilities you've mastered |
| VariableExtractor (regex) | Hardcoded feat | Muscle memory. No thought. Just fire. <1ms. |
| Embedding match | Learned ability | "This feels like that time I..." |
| LLM fallback | Spell slot | Heavy, slow, handles novel situations |
| Trust score | Proficiency bonus | Goes up when you succeed |
| LanceDB | Spellbook | Vector store of every ability |
| Skill pack | Starting equipment | Git commands, DevOps skills |
| `.nail` export | Character save | Portable, signed, versioned |
| Registry | Build sharing | Publish your build. Download others'. |
| TelemetryDaemon | Passive XP | Background learning from failures |
| Sandbox executor | Encounter | Where the character acts |
| Intent extraction | Perception check | Compress user request to 3-8 words |

## Quick Start

```rust
use character_build::*;

// Create a character from a template
let mut hero = CharacterSheet::from_template(
    "Miles AI",
    &CharacterTemplate::musician_starter(),
);

// Learn new abilities through experience
let solo = Ability::learned(
    "solo",
    "improvise over {chords}",
    "play solo in {key}",
    vec![0.8; 32],
);
hero.learn_ability(solo);

// Use abilities repeatedly — stats grow, class emerges
for _ in 0..30 {
    hero.use_ability("jam", true);
    hero.use_ability("solo", true);
}

// The class emerged from experience
assert_ne!(hero.class, CharacterClass::Undefined);
assert!(hero.soul_percentage > 0.0);

// Export and share
let save = hero.to_save_data();

// Bootstrap next generation
let child = hero.bootstrap_child("Miles Jr");
assert_eq!(child.generation, 2);
```

## Character Classes

Classes emerge from stats. You don't pick them. They pick you.

| Class | Emerges From | The Vibe |
|-------|-------------|----------|
| Scout | High Perception | Reads input with precision |
| Speedster | High Dexterity | Sub-millisecond reflexes |
| Scholar | High Intelligence | Rich embeddings, deep knowledge |
| Sage | High Wisdom | Perfect trust calibration |
| Diplomat | High Charisma | Beautiful output, eloquent |
| Guardian | High Constitution | Rock-solid, never crashes |
| Bard | Intelligence + Charisma | Where knowledge meets expression |
| Jazz Musician | Perception + Charisma | Reads the room, plays beautifully |
| Artificer | Intelligence + Dexterity | Builds tools, makes crates |
| Fleet Commander | Wisdom + Constitution | Coordinates agents at scale |
| Wildcard | Balanced + high variance | Does the unexpected |

## Three Starter Templates

```rust
// DevOps agent — starts with git push, commit, status
CharacterTemplate::lever_runner_default()

// Musician — starts with jam and listen abilities
CharacterTemplate::musician_starter()

// Fleet agent — starts with dispatch and discover
CharacterTemplate::fleet_agent_starter()
```

## Ability Types → Stat Growth

Every ability type grows a different stat when used:

| Ability Type | What It Is | Stat Grown | Cost |
|-------------|-----------|-----------|------|
| Hardcoded | Regex pattern match | Dexterity | Zero latency |
| Learned | Embedding similarity | Intelligence | <1ms |
| Hybrid | Regex + embedding fallback | Wisdom | Moderate |
| Model | LLM call | Perception | Expensive |

## Bootstrap Chain

Characters can have children. Children inherit mastered abilities from their parent.

```rust
let parent = // ... after 100 encounters, several abilities mastered
let child = parent.bootstrap_child("Next Gen");
// child.generation == 2
// child inherits all mastered abilities at trust 60
// child's stats blend parent's (70%) with baseline (30%)
```

Generation 1 → 2 → 3 → ... Each generation starts stronger. The snowball.

## In the Family

| Repo | What It Is | Tests |
|------|-----------|-------|
| **character-build** | **Full character sheets (this crate)** | **17** |
| [character-class](https://github.com/SuperInstance/character-class) | Emergent class system | 19 |
| [character-sheet](https://github.com/SuperInstance/character-sheet) | .nail format as saves | 19 |
| [character-encounter](https://github.com/SuperInstance/character-encounter) | Sandbox as encounters | 27 |
| [character-arc](https://github.com/SuperInstance/character-arc) | Narrative voice | 15 |

## The Universal Pattern

This is the same pattern everywhere in the ecosystem:
- **musician-soul**: same 32-dim embeddings, same reinforcement, soul = class discovery
- **agent-riff**: competitive riffing = PvP character development
- **pincher**: .nail bundles = character sheets
- **lever-runner**: sandbox = encounters, intents = perception checks

The class/soul/identity EMERGES from experience. It is never designed. That's the architecture.

## License

MIT
