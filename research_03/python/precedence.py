
"""Exact pairwise temporal precedence reconstruction.

Shortest-path closure yields the tightest implied upper bounds on
event-time differences. All arithmetic is exact integer arithmetic.

The reference vertex is included in closure but excluded from the
public event-pair classification.
"""

from __future__ import annotations

from dataclasses import dataclass
from enum import StrEnum

from feasibility import solve_feasibility
from temporal_model import ConstraintEdge, TemporalProblem


class TemporalRelation(StrEnum):
    """Classification of the temporal relationship between two events."""

    BEFORE = "necessary_before"
    AFTER = "necessary_after"
    EQUAL = "necessary_equal"
    AMBIGUOUS = "ambiguous"


@dataclass(frozen=True, slots=True)
class PairwiseCertificate:
    """Tight bounds and reconstructible paths for an ordered event pair.

    For events i and j:

        lower <= t_j - t_i <= upper

    Both bounds are exact for the feasible difference-constraint system.
    Forward and reverse paths refer to canonical input edge indices.
    """

    earlier_index: int
    later_index: int
    lower: int
    upper: int
    relation: TemporalRelation
    forward_path: tuple[int, ...]
    reverse_path: tuple[int, ...]

    def __post_init__(self) -> None:
        if self.earlier_index == self.later_index:
            raise ValueError("pair must contain distinct events")
        if self.lower > self.upper:
            raise ValueError("inconsistent pairwise bounds")


@dataclass(frozen=True, slots=True)
class PrecedenceResult:
    """Complete upper-bound closure and pairwise event classifications."""

    distances: tuple[tuple[int, ...], ...]
    pairs: tuple[PairwiseCertificate, ...]

    def get_pair(self, first: int, second: int) -> PairwiseCertificate:
        """Return the certificate for a pair in canonical index order."""
        if type(first) is not int or type(second) is not int:
            raise TypeError("event indices must be integers")
        if first == second:
            raise ValueError("event indices must differ")

        count = len(self.distances) - 1
        if not (1 <= first <= count and 1 <= second <= count):
            raise IndexError("event index outside the problem")

        i, j = sorted((first, second))
        for certificate in self.pairs:
            if certificate.earlier_index == i and certificate.later_index == j:
                return certificate

        raise RuntimeError("pairwise certificate is missing")


def _path(
    source: int,
    target: int,
    next_edge: list[list[int | None]],
    edges: tuple[ConstraintEdge, ...],
    vertex_count: int,
) -> tuple[int, ...]:
    """Reconstruct a shortest directed path using original edge indices."""
    if source == target:
        return ()

    result: list[int] = []
    current = source

    for _ in range(vertex_count):
        index = next_edge[current][target]
        if index is None:
            raise RuntimeError("missing shortest-path successor")

        edge = edges[index]
        if edge.source != current:
            raise RuntimeError("invalid shortest-path successor")

        result.append(index)
        current = edge.target

        if current == target:
            return tuple(result)

    raise RuntimeError("shortest-path reconstruction exceeded vertex count")


def _relation(lower: int, upper: int) -> TemporalRelation:
    """Classify the exact difference t_j - t_i."""
    if lower > 0:
        return TemporalRelation.BEFORE
    if upper < 0:
        return TemporalRelation.AFTER
    if lower == 0 and upper == 0:
        return TemporalRelation.EQUAL
    return TemporalRelation.AMBIGUOUS


def reconstruct_precedence(problem: TemporalProblem) -> PrecedenceResult:
    """Compute exact temporal closure and pairwise classifications.

    Raises ValueError when the original temporal system is infeasible.

    Floyd-Warshall complexity:
        Time: O(V^3 + E + V^2)
        Space: O(V^2 + E)

    Here V includes the fixed temporal reference vertex.

    Because every event has a finite input interval, all event-pair
    difference bounds are finite for a feasible problem.
    """
    if not isinstance(problem, TemporalProblem):
        raise TypeError("problem must be a TemporalProblem")

    feasibility = solve_feasibility(problem)
    if not feasibility.feasible:
        raise ValueError("temporal constraints are inconsistent")

    edges = problem.edges()
    count = problem.vertex_count

    distance: list[list[int | None]] = [
        [None] * count for _ in range(count)
    ]
    next_edge: list[list[int | None]] = [
        [None] * count for _ in range(count)
    ]

    for vertex in range(count):
        distance[vertex][vertex] = 0

    for index, edge in enumerate(edges):
        previous = distance[edge.source][edge.target]
        if previous is None or edge.weight < previous:
            distance[edge.source][edge.target] = edge.weight
            next_edge[edge.source][edge.target] = index

    for intermediate in range(count):
        for source in range(count):
            first = distance[source][intermediate]
            if first is None:
                continue

            for target in range(count):
                second = distance[intermediate][target]
                if second is None:
                    continue

                candidate = first + second
                previous = distance[source][target]

                if previous is None or candidate < previous:
                    distance[source][target] = candidate
                    next_edge[source][target] = next_edge[
                        source
                    ][intermediate]

    if any(distance[vertex][vertex] != 0 for vertex in range(count)):
        raise RuntimeError("negative cycle escaped feasibility analysis")

    # Every event is connected to the temporal origin by its finite
    # lower and upper bounds. Consequently, every pair is reachable.
    if any(value is None for row in distance for value in row):
        raise RuntimeError("finite event intervals require full reachability")

    exact_distance = tuple(
        tuple(value for value in row if value is not None)
        for row in distance
    )

    pairs: list[PairwiseCertificate] = []

    for first in range(1, count):
        for second in range(first + 1, count):
            upper = exact_distance[first][second]
            lower = -exact_distance[second][first]

            forward = _path(
                first,
                second,
                next_edge,
                edges,
                count,
            )
            reverse = _path(
                second,
                first,
                next_edge,
                edges,
                count,
            )

            if sum(edges[index].weight for index in forward) != upper:
                raise RuntimeError("forward path does not certify its bound")
            if sum(edges[index].weight for index in reverse) != -lower:
                raise RuntimeError("reverse path does not certify its bound")

            pairs.append(
                PairwiseCertificate(
                    earlier_index=first,
                    later_index=second,
                    lower=lower,
                    upper=upper,
                    relation=_relation(lower, upper),
                    forward_path=forward,
                    reverse_path=reverse,
                )
            )

    return PrecedenceResult(
        distances=exact_distance,
        pairs=tuple(pairs),
    )
