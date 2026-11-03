
"""Exact feasibility analysis and certificates for temporal constraints.

A feasible system returns an explicit integer assignment of event times.
An infeasible system returns edge indices forming a negative directed
cycle. Certificate verification is intentionally delegated to a separate
component, rather than trusting the solver's own output.
"""

from __future__ import annotations

from dataclasses import dataclass

from temporal_model import ConstraintEdge, TemporalProblem


@dataclass(frozen=True, slots=True)
class FeasibilityResult:
    """Exactly one certificate is populated according to feasibility."""

    feasible: bool
    assignment: tuple[int, ...] | None
    negative_cycle: tuple[int, ...] | None

    def __post_init__(self) -> None:
        if type(self.feasible) is not bool:
            raise TypeError("feasible must be boolean")
        if self.feasible:
            if self.assignment is None or self.negative_cycle is not None:
                raise ValueError("feasible result requires only an assignment")
        elif self.negative_cycle is None or self.assignment is not None:
            raise ValueError("infeasible result requires only a negative cycle")


def _extract_negative_cycle(
    predecessor: list[int | None],
    edges: tuple[ConstraintEdge, ...],
    updated_vertex: int,
    vertex_count: int,
) -> tuple[int, ...]:
    """Recover a directed negative cycle from predecessor edge indices."""
    vertex = updated_vertex

    # Enter the cycle, even when the last update occurred downstream.
    for _ in range(vertex_count):
        edge_index = predecessor[vertex]
        if edge_index is None:
            raise RuntimeError("missing predecessor during cycle recovery")
        vertex = edges[edge_index].source

    start = vertex
    reverse_cycle: list[int] = []

    while True:
        edge_index = predecessor[vertex]
        if edge_index is None:
            raise RuntimeError("incomplete predecessor cycle")

        reverse_cycle.append(edge_index)
        vertex = edges[edge_index].source

        if vertex == start:
            break
        if len(reverse_cycle) > vertex_count:
            raise RuntimeError("predecessor cycle exceeds vertex count")

    cycle = tuple(reversed(reverse_cycle))

    # Internal consistency checks; independent verification follows later.
    if sum(edges[index].weight for index in cycle) >= 0:
        raise RuntimeError("recovered cycle is not negative")

    for current, following in zip(
        cycle, cycle[1:] + cycle[:1], strict=True
    ):
        if edges[current].target != edges[following].source:
            raise RuntimeError("recovered cycle is not directed")

    return cycle


def solve_feasibility(problem: TemporalProblem) -> FeasibilityResult:
    """Decide feasibility using exact Bellman-Ford relaxation.

    Initializing every distance to zero is equivalent to introducing
    an auxiliary super-source with zero-weight edges to all vertices.
    This guarantees that every negative cycle is reachable.

    For a feasible system, distances are shifted so that the temporal
    reference vertex has value zero. The resulting event-time assignment
    satisfies the original interval and difference constraints.

    Time complexity: O(VE).
    Auxiliary space complexity: O(V), excluding the input edge tuple.
    """
    if not isinstance(problem, TemporalProblem):
        raise TypeError("problem must be a TemporalProblem")

    edges = problem.edges()
    count = problem.vertex_count

    distance = [0] * count
    predecessor: list[int | None] = [None] * count
    last_updated: int | None = None

    for _ in range(count):
        last_updated = None

        for index, edge in enumerate(edges):
            candidate = distance[edge.source] + edge.weight

            if candidate < distance[edge.target]:
                distance[edge.target] = candidate
                predecessor[edge.target] = index
                last_updated = edge.target

        if last_updated is None:
            break

    if last_updated is not None:
        cycle = _extract_negative_cycle(
            predecessor=predecessor,
            edges=edges,
            updated_vertex=last_updated,
            vertex_count=count,
        )
        return FeasibilityResult(
            feasible=False,
            assignment=None,
            negative_cycle=cycle,
        )

    origin = distance[0]
    assignment = tuple(value - origin for value in distance[1:])

    return FeasibilityResult(
        feasible=True,
        assignment=assignment,
        negative_cycle=None,
    )
