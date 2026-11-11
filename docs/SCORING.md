# Explainable scoring model

The engine is intentionally decomposable. It never generates a recommendation first and invents a reason later.

## Candidate score

Overall = 38% Role Fit + 32% Structural Fit + 18% Tactical/Match Fit + 12% Complexity Fit.

These weights are a first practical model, not official Sports Interactive coefficients.

## Role Fit

Role Fit is a weighted average of the attributes defined for the role profile. Missing attributes are penalised. Player availability is handled separately in the future native reader.

## Structural Fit

Checks:
- width
- depth
- creator availability
- screening / rest-defence
- pressing coverage
- formation-specific bonuses/penalties

## Tactical / Match Fit

Checks the relationship between the squad DNA and setup:
- high press needs pressing capacity
- high defensive line needs recovery pace
- direct play benefits transition speed
- positive mentality benefits creation and finishing

## Complexity Fit

A first-order penalty for instruction volume. FM27 publicly exposes Tactical Effectiveness / Complexity concepts, but SI does not publish the hidden coefficient table, so this is an explainable heuristic.

## Confidence

Confidence currently measures data completeness and squad size. Once a native FM27 reader exists, it should also incorporate read quality, player availability freshness and opponent-report freshness.
