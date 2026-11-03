
"""Independent certificate verification for temporal feasibility.

The verifier evaluates original problem constraints and reconstructs
negative-cycle edges from their source records. It does not call the
feasibility solver or trust solver-internal distances.
"""

from __future__ import annotations

from feasibility import FeasibilityResult
from temporal_model import TemporalProblem


def verify_assignment(
    problem: TemporalProblem,
    assignment: tuple[int, ...],
) -> bool:
    """Verify an exact event-time assignment against original inputs."""
    if not isinstance(problem, TemporalProblem):
        raise TypeError("problem must be a TemporalProblem")

    if not isinstance(assignment, tuple):
        return False
    if len(assignment) != len(problem.events):
        return False
    if any(type(value) is not int for value in assignment):
        return False

    for event, time in zip(problem.events, assignment, strict=True):
        if not event.lower <= time <= event.upper:
            return False

    for constraint in problem.constraints:
        source_time = assignment[constraint.source - 1]
        target_time = assignment[constraint.target - 1]

        if target_time - source_time > constraint.upper_bound:
            return False

    return True


def verify_negative_cycle(
    problem: TemporalProblem,
    cycle: tuple[int, ...],
) -> bool:
    """Verify a directed negative cycle using original input constraints.

    The indices refer to the canonical edge ordering produced by
    TemporalProblem.edges(). Every selected edge is independently
    reconstructed from the corresponding original interval or
    difference constraint.

    A valid certificate must form a closed directed walk with strictly
    negative total weight. Repeated vertices are permitted because
    negativity of a closed walk is already sufficient for infeasibility.
    """
    if not isinstance(problem, TemporalProblem):
        raise TypeError("problem must be a TemporalProblem")

    if not isinstance(cycle, tuple) or not cycle:
        return False
    if any(type(index) is not int for index in cycle):
        return False

    event_count = len(problem.events)
    interval_edge_count = 2 * event_count
    edge_count = interval_edge_count + len(problem.constraints)

    if any(index < 0 or index >= edge_count for index in cycle):
        return False

    reconstructed: list[tuple[int, int, int]] = []

    for index in cycle:
        if index < interval_edge_count:
            event_index = index // 2
            event = problem.events[event_index]
            vertex = event_index + 1

            if index % 2 == 0:
                reconstructed.append((0, vertex, event.upper))
            else:
                reconstructed.append((vertex, 0, -event.lower))
        else:
            constraint = problem.constraints[index - interval_edge_count]
            reconstructed.append(
                (
                    constraint.source,
                    constraint.target,
                    constraint.upper_bound,
                )
            )

    for position, (_, target, _) in enumerate(reconstructed):
        following_source = reconstructed[
            (position + 1) % len(reconstructed)
        ][0]
        if target != following_source:
            return False

    total_weight = sum(weight for _, _, weight in reconstructed)
    return total_weight < 0


def verify_feasibility_result(
    problem: TemporalProblem,
    result: FeasibilityResult,
) -> bool:
    """Independently verify either solver outcome.

    A successful verification certifies the supplied result for the
    specified finite temporal problem. It does not certify the
    correctness of the original real-world timestamp evidence.
    """
    if not isinstance(problem, TemporalProblem):
        raise TypeError("problem must be a TemporalProblem")
    if not isinstance(result, FeasibilityResult):
        raise TypeError("result must be a FeasibilityResult")

    if result.feasible:
        if result.assignment is None or result.negative_cycle is not None:
            return False
        return verify_assignment(problem, result.assignment)

    if result.assignment is not None or result.negative_cycle is None:
        return False

    return verify_negative_cycle(problem, result.negative_cycle)
