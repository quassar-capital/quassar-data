
"""Scientific regression tests for Research 03 temporal certification.

The tests check feasibility, independent certificates, tight temporal
bounds, strict precedence, equality, ambiguity, and uncertainty
monotonicity. Small integer systems are compared with an independent
exhaustive enumeration oracle.

No external data or network access is required.
"""

from __future__ import annotations

import itertools
import random
import sys
from pathlib import Path

import pytest

RESEARCH_PATH = Path(__file__).resolve().parents[1] / "research_03" / "python"
sys.path.insert(0, str(RESEARCH_PATH))

from certificate_verifier import (  # noqa: E402
    verify_assignment,
    verify_feasibility_result,
    verify_negative_cycle,
)
from feasibility import solve_feasibility  # noqa: E402
from precedence import (  # noqa: E402
    TemporalRelation,
    reconstruct_precedence,
)
from temporal_model import (  # noqa: E402
    DifferenceConstraint,
    EventInterval,
    TemporalProblem,
    precedence_constraint,
)


def problem(
    intervals: tuple[tuple[int, int], ...],
    constraints: tuple[DifferenceConstraint, ...] = (),
) -> TemporalProblem:
    """Build a finite temporal problem with deterministic event names."""
    events = tuple(
        EventInterval(f"event_{index}", lower, upper)
        for index, (lower, upper) in enumerate(intervals, start=1)
    )
    return TemporalProblem(events, constraints)


def exhaustive_assignments(
    temporal_problem: TemporalProblem,
) -> tuple[tuple[int, ...], ...]:
    """Enumerate all feasible integer assignments for small systems.

    This oracle does not call the graph solver or its certificate
    verifier. Integer difference constraints with integer bounds
    admit integral extrema whenever feasible.
    """
    domains = (
        range(event.lower, event.upper + 1)
        for event in temporal_problem.events
    )

    valid = []
    for assignment in itertools.product(*domains):
        if any(
            assignment[constraint.target - 1]
            - assignment[constraint.source - 1]
            > constraint.upper_bound
            for constraint in temporal_problem.constraints
        ):
            continue
        valid.append(assignment)

    return tuple(valid)


def verify_bound_path(
    temporal_problem: TemporalProblem,
    path: tuple[int, ...],
    source: int,
    target: int,
    expected_weight: int,
) -> None:
    """Check path continuity and weight against original graph edges."""
    edges = temporal_problem.edges()
    current = source
    total = 0

    assert path
    for index in path:
        assert type(index) is int
        assert 0 <= index < len(edges)
        edge = edges[index]
        assert edge.source == current
        current = edge.target
        total += edge.weight

    assert current == target
    assert total == expected_weight


def test_exact_model_and_edge_provenance() -> None:
    temporal_problem = problem(
        ((2, 5), (7, 10)),
        (precedence_constraint(1, 2, 1),),
    )

    assert temporal_problem.vertex_count == 3

    edges = temporal_problem.edges()
    assert len(edges) == 5

    assert (edges[0].source, edges[0].target, edges[0].weight) == (
        0,
        1,
        5,
    )
    assert (edges[1].source, edges[1].target, edges[1].weight) == (
        1,
        0,
        -2,
    )
    assert (edges[4].source, edges[4].target, edges[4].weight) == (
        2,
        1,
        -1,
    )

    assert edges[4].provenance == "constraint:0"


@pytest.mark.parametrize(
    ("intervals", "constraints"),
    [
        (((0, 0),), ()),
        (((-5, 5), (3, 8)), ()),
        (
            ((0, 4), (1, 7)),
            (precedence_constraint(1, 2, 2),),
        ),
        (
            ((-3, 3), (-3, 3)),
            (
                precedence_constraint(1, 2),
                precedence_constraint(2, 1),
            ),
        ),
    ],
)
def test_feasible_assignments_are_independently_verified(
    intervals: tuple[tuple[int, int], ...],
    constraints: tuple[DifferenceConstraint, ...],
) -> None:
    temporal_problem = problem(intervals, constraints)
    result = solve_feasibility(temporal_problem)

    assert result.feasible
    assert result.assignment is not None
    assert result.negative_cycle is None
    assert verify_assignment(temporal_problem, result.assignment)
    assert verify_feasibility_result(temporal_problem, result)


@pytest.mark.parametrize(
    ("intervals", "constraints"),
    [
        (
            ((0, 0), (0, 0)),
            (precedence_constraint(1, 2, 1),),
        ),
        (
            ((0, 10), (0, 10)),
            (
                precedence_constraint(1, 2, 2),
                precedence_constraint(2, 1, 1),
            ),
        ),
        (
            ((0, 1), (5, 6)),
            (precedence_constraint(2, 1),),
        ),
        (
            ((0, 0),),
            (DifferenceConstraint(1, 1, -1),),
        ),
    ],
)
def test_infeasibility_has_independent_negative_cycle_certificate(
    intervals: tuple[tuple[int, int], ...],
    constraints: tuple[DifferenceConstraint, ...],
) -> None:
    temporal_problem = problem(intervals, constraints)
    result = solve_feasibility(temporal_problem)

    assert not result.feasible
    assert result.assignment is None
    assert result.negative_cycle is not None
    assert verify_negative_cycle(temporal_problem, result.negative_cycle)
    assert verify_feasibility_result(temporal_problem, result)

    with pytest.raises(ValueError, match="inconsistent"):
        reconstruct_precedence(temporal_problem)


def test_independent_verifier_rejects_invalid_assignments() -> None:
    temporal_problem = problem(
        ((0, 2), (1, 4)),
        (precedence_constraint(1, 2, 1),),
    )

    assert verify_assignment(temporal_problem, (0, 1))
    assert not verify_assignment(temporal_problem, (2, 1))
    assert not verify_assignment(temporal_problem, (-1, 1))
    assert not verify_assignment(temporal_problem, (0,))
    assert not verify_assignment(temporal_problem, (0, True))
    assert not verify_assignment(temporal_problem, [0, 1])


def test_independent_verifier_rejects_invalid_cycles() -> None:
    temporal_problem = problem(
        ((0, 0), (0, 0)),
        (precedence_constraint(1, 2, 1),),
    )

    assert not verify_negative_cycle(temporal_problem, ())
    assert not verify_negative_cycle(temporal_problem, (999,))
    assert not verify_negative_cycle(temporal_problem, (-1,))
    assert not verify_negative_cycle(temporal_problem, (True,))
    assert not verify_negative_cycle(temporal_problem, (0,))
    assert not verify_negative_cycle(temporal_problem, (0, 1))


@pytest.mark.parametrize(
    ("intervals", "expected"),
    [
        (((0, 1), (3, 4)), TemporalRelation.BEFORE),
        (((3, 4), (0, 1)), TemporalRelation.AFTER),
        (((2, 2), (2, 2)), TemporalRelation.EQUAL),
        (((0, 3), (1, 4)), TemporalRelation.AMBIGUOUS),
        (((0, 1), (1, 2)), TemporalRelation.AMBIGUOUS),
    ],
)
def test_pairwise_temporal_classification(
    intervals: tuple[tuple[int, int], ...],
    expected: TemporalRelation,
) -> None:
    temporal_problem = problem(intervals)
    result = reconstruct_precedence(temporal_problem)
    certificate = result.get_pair(1, 2)

    assert certificate.relation is expected

    assignments = exhaustive_assignments(temporal_problem)
    differences = [second - first for first, second in assignments]

    assert certificate.lower == min(differences)
    assert certificate.upper == max(differences)


def test_constraints_can_identify_order_with_overlapping_intervals() -> None:
    temporal_problem = problem(
        ((0, 10), (0, 10)),
        (precedence_constraint(1, 2, 3),),
    )

    certificate = reconstruct_precedence(temporal_problem).get_pair(1, 2)

    assert certificate.relation is TemporalRelation.BEFORE
    assert certificate.lower == 3
    assert certificate.upper == 10


def test_non_strict_precedence_is_not_strict_precedence() -> None:
    temporal_problem = problem(
        ((0, 2), (0, 2)),
        (precedence_constraint(1, 2),),
    )

    certificate = reconstruct_precedence(temporal_problem).get_pair(1, 2)

    assert certificate.lower == 0
    assert certificate.relation is TemporalRelation.AMBIGUOUS


def test_shortest_path_certificates_reconstruct_original_edges() -> None:
    temporal_problem = problem(
        ((0, 10), (0, 10), (0, 10)),
        (
            precedence_constraint(1, 2, 2),
            precedence_constraint(2, 3, 3),
        ),
    )

    result = reconstruct_precedence(temporal_problem)

    for certificate in result.pairs:
        verify_bound_path(
            temporal_problem,
            certificate.forward_path,
            certificate.earlier_index,
            certificate.later_index,
            certificate.upper,
        )
        verify_bound_path(
            temporal_problem,
            certificate.reverse_path,
            certificate.later_index,
            certificate.earlier_index,
            -certificate.lower,
        )

    assert result.get_pair(1, 3).lower == 5
    assert result.get_pair(1, 3).relation is TemporalRelation.BEFORE


def test_invalid_model_inputs_are_rejected() -> None:
    with pytest.raises(ValueError):
        EventInterval("bad", 5, 4)

    with pytest.raises(ValueError):
        EventInterval("", 0, 1)

    with pytest.raises(TypeError):
        EventInterval("bad", True, 1)

    with pytest.raises(ValueError):
        TemporalProblem(
            (
                EventInterval("duplicate", 0, 1),
                EventInterval("duplicate", 1, 2),
            )
        )

    with pytest.raises(ValueError):
        problem(
            ((0, 1),),
            (DifferenceConstraint(1, 2, 0),),
        )

    with pytest.raises(ValueError):
        precedence_constraint(1, 2, -1)


def test_uncertainty_expansion_cannot_create_necessary_order() -> None:
    centers = (0, 5, 10)
    uncertainty = (1, 1, 1)
    previous_relations: set[tuple[int, int, TemporalRelation]] | None = None

    for scale in range(7):
        intervals = tuple(
            (
                center - scale * radius,
                center + scale * radius,
            )
            for center, radius in zip(centers, uncertainty)
        )
        result = reconstruct_precedence(problem(intervals))

        necessary = {
            (
                certificate.earlier_index,
                certificate.later_index,
                certificate.relation,
            )
            for certificate in result.pairs
            if certificate.relation
            in (TemporalRelation.BEFORE, TemporalRelation.AFTER)
        }

        if previous_relations is not None:
            assert necessary <= previous_relations

        previous_relations = necessary


def test_random_small_systems_against_exhaustive_oracle() -> None:
    """Compare exact graph results against independently enumerated times."""
    rng = random.Random(20261103)

    for _ in range(80):
        intervals = tuple(
            (
                lower,
                lower + rng.randrange(3),
            )
            for lower in (rng.randrange(-2, 3) for _ in range(3))
        )

        constraints = tuple(
            DifferenceConstraint(
                source=rng.randrange(1, 4),
                target=rng.randrange(1, 4),
                upper_bound=rng.randrange(-3, 4),
            )
            for _ in range(rng.randrange(5))
        )

        temporal_problem = problem(intervals, constraints)
        assignments = exhaustive_assignments(temporal_problem)
        feasibility = solve_feasibility(temporal_problem)

        assert feasibility.feasible == bool(assignments)
        assert verify_feasibility_result(temporal_problem, feasibility)

        if not assignments:
            continue

        result = reconstruct_precedence(temporal_problem)

        for certificate in result.pairs:
            i = certificate.earlier_index - 1
            j = certificate.later_index - 1

            differences = [
                assignment[j] - assignment[i]
                for assignment in assignments
            ]

            assert certificate.lower == min(differences)
            assert certificate.upper == max(differences)

            if all(value > 0 for value in differences):
                expected = TemporalRelation.BEFORE
            elif all(value < 0 for value in differences):
                expected = TemporalRelation.AFTER
            elif all(value == 0 for value in differences):
                expected = TemporalRelation.EQUAL
            else:
                expected = TemporalRelation.AMBIGUOUS

            assert certificate.relation is expected

            verify_bound_path(
                temporal_problem,
                certificate.forward_path,
                certificate.earlier_index,
                certificate.later_index,
                certificate.upper,
            )
            verify_bound_path(
                temporal_problem,
                certificate.reverse_path,
                certificate.later_index,
                certificate.earlier_index,
                -certificate.lower,
            )
