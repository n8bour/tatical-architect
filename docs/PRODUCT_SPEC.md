# FM Tactical Architect - Product Spec

## Core promise

Given a real FM27 squad, produce the strongest tactical model the squad can realistically execute, with transparent reasons and alternatives.

## Required output

1. Squad DNA
2. Three or more tactical candidates
3. Best XI for each candidate
4. In Possession roles
5. Out of Possession roles
6. Team Instructions
7. Player Instructions
8. Opposition plan when opponent data exists
9. Tactical Effectiveness / Complexity interpretation
10. Training and development priorities
11. Confidence + data quality
12. Human-readable justification for every score

## Future native-reader contract

The memory reader should publish a `SquadSnapshot` with:
- game build
- save identifier / hash
- club
- players
- attributes
- positions + familiarity
- fitness / sharpness
- injuries
- personality + traits
- tactical familiarity
- player availability
- staff context when relevant

The tactical engine must not directly consume raw memory offsets. It must consume this normalised snapshot.
