
"""Reproducible uncertainty-sensitivity experiments for Research 03.

This experiment studies how bounded clock uncertainty changes the
identifiability of temporal precedence in synthetic event systems.

It uses exact integer arithmetic, verifies each feasible assignment,
and checks monotonicity of necessary strict precedence relations.

Output is deterministic CSV written to standard output.
No external datasets, network access, or third-party packages are used.
"""

from __future__ import annotations

import argparse
import csv
import sys
from dataclasses import dataclass
from typing import TextIO

from certificate_verifier import verify_feasibility_result
from feasibility import solve_feasibility
from precedence import TemporalRelation, reconstruct_precedence
from temporal_model import (
    DifferenceConstraint,
    EventInterval,
    TemporalProblem,
)


@dataclass(frozen=True, slots=True)
class SyntheticScenario:
    """Fixed synthetic observations and optional timing constraints."""

    name: str
    centers: tuple[int, ...]
    uncertainty: tuple[int, ...]
    constraints: tuple[DifferenceConstraint, ...] = ()

    def __post_init__(self) -> None:
        if not self.name:
            raise ValueError("scenario name must not be empty")
        if not self.centers:
            raise ValueError("scenario must contain events")
        if len(self.centers) != len(self.uncertainty):
            raise ValueError("centers and uncertainty lengths must match")

        for value in (*self.centers, *self.uncertainty):
            if type(value) is not int:
                raise TypeError("scenario time values must be integers")

        if any(radius < 0 for radius in self.uncertainty):
            raise ValueError("uncertainty bounds must be nonnegative")

        # Validate the supplied constraint indices against the event set.
        self.problem_at(0)

    def problem_at(self, scale: int) -> TemporalProblem:
        """Construct closed intervals at an integer uncertainty scale."""
        if type(scale) is not int or scale < 0:
            raise ValueError("scale must be a nonnegative integer")

        events = tuple(
            EventInterval(
                name=f"event_{index}",
                lower=center - scale * radius,
                upper=center + scale * radius,
            )
            for index, (center, radius) in enumerate(
                zip(self.centers, self.uncertainty, strict=True),
                start=1,
            )
        )

        return TemporalProblem(
            events=events,
            constraints=self.constraints,
        )


@dataclass(frozen=True, slots=True)
class ExperimentRow:
    """One reproducible observation of temporal identifiability."""

    scenario: str
    scale: int
    feasible: bool
    event_count: int
    pair_count: int
    necessary_before: int
    necessary_after: int
    necessary_equal: int
    ambiguous: int
    unsupported_timestamp_order: int

    def as_record(self) -> dict[str, str | int]:
        """Return a stable CSV record with explicit field names."""
        return {
            "scenario": self.scenario,
            "scale": self.scale,
            "feasible": int(self.feasible),
            "event_count": self.event_count,
            "pair_count": self.pair_count,
            "necessary_before": self.necessary_before,
            "necessary_after": self.necessary_after,
            "necessary_equal": self.necessary_equal,
            "ambiguous": self.ambiguous,
            "unsupported_timestamp_order": self.unsupported_timestamp_order,
        }


def reference_scenarios() -> tuple[SyntheticScenario, ...]:
    """Define fixed synthetic cases with different timing structures."""
    return (
        SyntheticScenario(
            name="independent_clocks",
            centers=(0, 5, 10, 16),
            uncertainty=(1, 2, 1, 2),
        ),
        SyntheticScenario(
            name="overlapping_observations",
            centers=(0, 2, 4, 6),
            uncertainty=(2, 2, 2, 2),
        ),
        SyntheticScenario(
            name="constrained_precedence",
            centers=(0, 4, 8, 12),
            uncertainty=(1, 2, 2, 1),
            constraints=(
                DifferenceConstraint(2, 1, -1),
                DifferenceConstraint(3, 2, -1),
            ),
        ),
        SyntheticScenario(
            name="coincident_timestamps",
            centers=(0, 0, 0, 0),
            uncertainty=(1, 2, 1, 2),
        ),
    )


def _necessary_relations(
    temporal_problem: TemporalProblem,
) -> tuple[
    set[tuple[int, int, TemporalRelation]],
    dict[TemporalRelation, int],
    int,
]:
    """Classify all pairs and count unsupported naive timestamp orders."""
    result = reconstruct_precedence(temporal_problem)

    counts = {relation: 0 for relation in TemporalRelation}
    necessary: set[tuple[int, int, TemporalRelation]] = set()
    unsupported = 0

    for pair in result.pairs:
        counts[pair.relation] += 1

        if pair.relation in (
            TemporalRelation.BEFORE,
            TemporalRelation.AFTER,
        ):
            necessary.add(
                (
                    pair.earlier_index,
                    pair.later_index,
                    pair.relation,
                )
            )

        first_center = (
            temporal_problem.events[pair.earlier_index - 1].lower
            + temporal_problem.events[pair.earlier_index - 1].upper
        )
        second_center = (
            temporal_problem.events[pair.later_index - 1].lower
            + temporal_problem.events[pair.later_index - 1].upper
        )

        # Comparing doubled centers avoids floating-point arithmetic.
        # A naive timestamp sort asserts strict precedence whenever
        # the reported centers differ.
        if first_center < second_center:
            if pair.relation is not TemporalRelation.BEFORE:
                unsupported += 1
        elif first_center > second_center:
            if pair.relation is not TemporalRelation.AFTER:
                unsupported += 1

    return necessary, counts, unsupported


def run_scenario(
    scenario: SyntheticScenario,
    max_scale: int,
) -> tuple[ExperimentRow, ...]:
    """Evaluate an uncertainty expansion and check its monotonicity."""
    if type(max_scale) is not int or max_scale < 0:
        raise ValueError("max_scale must be a nonnegative integer")

    rows: list[ExperimentRow] = []
    previous_necessary: set[
        tuple[int, int, TemporalRelation]
    ] | None = None

    for scale in range(max_scale + 1):
        temporal_problem = scenario.problem_at(scale)
        feasibility = solve_feasibility(temporal_problem)

        if not verify_feasibility_result(
            temporal_problem,
            feasibility,
        ):
            raise RuntimeError(
                f"invalid feasibility certificate: "
                f"{scenario.name}, scale={scale}"
            )

        event_count = len(temporal_problem.events)
        pair_count = event_count * (event_count - 1) // 2

        if not feasibility.feasible:
            rows.append(
                ExperimentRow(
                    scenario=scenario.name,
                    scale=scale,
                    feasible=False,
                    event_count=event_count,
                    pair_count=pair_count,
                    necessary_before=0,
                    necessary_after=0,
                    necessary_equal=0,
                    ambiguous=0,
                    unsupported_timestamp_order=0,
                )
            )
            continue

        necessary, counts, unsupported = _necessary_relations(
            temporal_problem
        )

        if previous_necessary is not None:
            if not necessary.issubset(previous_necessary):
                raise AssertionError(
                    "uncertainty expansion created a new "
                    "necessary strict precedence relation"
                )

        previous_necessary = necessary

        if sum(counts.values()) != pair_count:
            raise AssertionError("pairwise classifications are incomplete")

        rows.append(
            ExperimentRow(
                scenario=scenario.name,
                scale=scale,
                feasible=True,
                event_count=event_count,
                pair_count=pair_count,
                necessary_before=counts[TemporalRelation.BEFORE],
                necessary_after=counts[TemporalRelation.AFTER],
                necessary_equal=counts[TemporalRelation.EQUAL],
                ambiguous=counts[TemporalRelation.AMBIGUOUS],
                unsupported_timestamp_order=unsupported,
            )
        )

    return tuple(rows)


def run_experiments(
    max_scale: int = 8,
) -> tuple[ExperimentRow, ...]:
    """Run every reference scenario in a deterministic order."""
    return tuple(
        row
        for scenario in reference_scenarios()
        for row in run_scenario(scenario, max_scale)
    )


def write_csv(
    rows: tuple[ExperimentRow, ...],
    destination: TextIO,
) -> None:
    """Write deterministic, machine-readable experiment results."""
    fieldnames = (
        "scenario",
        "scale",
        "feasible",
        "event_count",
        "pair_count",
        "necessary_before",
        "necessary_after",
        "necessary_equal",
        "ambiguous",
        "unsupported_timestamp_order",
    )

    writer = csv.DictWriter(
        destination,
        fieldnames=fieldnames,
        lineterminator="\n",
    )
    writer.writeheader()

    for row in rows:
        writer.writerow(row.as_record())


def main(argv: list[str] | None = None) -> int:
    """Execute the experiment using a reproducible CLI."""
    parser = argparse.ArgumentParser(
        description=(
            "Research 03: certified temporal uncertainty "
            "and identifiability experiments"
        )
    )
    parser.add_argument(
        "--max-scale",
        type=int,
        default=8,
        help="maximum nonnegative integer uncertainty multiplier",
    )

    args = parser.parse_args(argv)

    if args.max_scale < 0:
        parser.error("--max-scale must be nonnegative")

    rows = run_experiments(max_scale=args.max_scale)
    write_csv(rows, sys.stdout)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
