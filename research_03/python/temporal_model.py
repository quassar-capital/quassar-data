"""Exact temporal difference-constraint representation for Research 03.

All time values are integer ticks in a caller-defined common unit.
Vertex zero is the fixed temporal origin. Event vertices are numbered
from one through the number of events.
"""

from __future__ import annotations

from dataclasses import dataclass
from typing import TypeAlias

Vertex: TypeAlias = int


def _integer(value: object, name: str) -> int:
    if type(value) is not int:
        raise TypeError(f"{name} must be an exact integer")
    return value


@dataclass(frozen=True, slots=True)
class EventInterval:
    """A finite closed interval containing an event's occurrence time."""

    name: str
    lower: int
    upper: int

    def __post_init__(self) -> None:
        if not isinstance(self.name, str) or not self.name.strip():
            raise ValueError("event name must be a nonempty string")
        _integer(self.lower, "lower")
        _integer(self.upper, "upper")
        if self.lower > self.upper:
            raise ValueError("event lower bound exceeds upper bound")


@dataclass(frozen=True, slots=True)
class DifferenceConstraint:
    """The inequality t_target - t_source <= upper_bound.

    Source and target use one-based event indices. The reference vertex
    is reserved for the interval constraints constructed by the model.
    """

    source: int
    target: int
    upper_bound: int

    def __post_init__(self) -> None:
        _integer(self.source, "source")
        _integer(self.target, "target")
        _integer(self.upper_bound, "upper_bound")


@dataclass(frozen=True, slots=True)
class ConstraintEdge:
    """A graph edge encoding t_target <= t_source + weight.

    provenance identifies the original event interval or declared
    difference constraint from which the edge was constructed.
    """

    source: Vertex
    target: Vertex
    weight: int
    provenance: str


@dataclass(frozen=True, slots=True)
class TemporalProblem:
    """An immutable finite temporal-constraint problem.

    All intervals and relative-time constraints are supplied in the
    same exact integer time unit. The graph is reconstructed from
    immutable input data and does not retain solver state.
    """

    events: tuple[EventInterval, ...]
    constraints: tuple[DifferenceConstraint, ...] = ()

    def __post_init__(self) -> None:
        if not isinstance(self.events, tuple) or not self.events:
            raise ValueError("events must be a nonempty tuple")
        if not isinstance(self.constraints, tuple):
            raise TypeError("constraints must be a tuple")

        names: set[str] = set()
        for event in self.events:
            if not isinstance(event, EventInterval):
                raise TypeError("every event must be an EventInterval")
            if event.name in names:
                raise ValueError(f"duplicate event name: {event.name}")
            names.add(event.name)

        count = len(self.events)
        for constraint in self.constraints:
            if not isinstance(constraint, DifferenceConstraint):
                raise TypeError("every constraint must be a DifferenceConstraint")
            if not 1 <= constraint.source <= count:
                raise ValueError("constraint source is outside the event set")
            if not 1 <= constraint.target <= count:
                raise ValueError("constraint target is outside the event set")

    @property
    def vertex_count(self) -> int:
        """Include the fixed temporal origin as vertex zero."""
        return len(self.events) + 1

    def edges(self) -> tuple[ConstraintEdge, ...]:
        """Encode the original inequalities without altering their meaning.

        For event i with interval [lower, upper]:

            t_i - t_0 <= upper
            t_0 - t_i <= -lower

        For a declared constraint:

            t_target - t_source <= upper_bound
        """
        result: list[ConstraintEdge] = []

        for index, event in enumerate(self.events, start=1):
            result.append(
                ConstraintEdge(
                    source=0,
                    target=index,
                    weight=event.upper,
                    provenance=f"interval:{index}:upper",
                )
            )
            result.append(
                ConstraintEdge(
                    source=index,
                    target=0,
                    weight=-event.lower,
                    provenance=f"interval:{index}:lower",
                )
            )

        for index, constraint in enumerate(self.constraints):
            result.append(
                ConstraintEdge(
                    source=constraint.source,
                    target=constraint.target,
                    weight=constraint.upper_bound,
                    provenance=f"constraint:{index}",
                )
            )

        return tuple(result)


def precedence_constraint(
    earlier: int,
    later: int,
    minimum_separation: int = 0,
) -> DifferenceConstraint:
    """Require t_later - t_earlier >= minimum_separation.

    A zero separation expresses non-strict precedence. A positive
    separation expresses a minimum time gap, not an open inequality.
    """
    _integer(minimum_separation, "minimum_separation")
    if minimum_separation < 0:
        raise ValueError("minimum_separation must be nonnegative")
    return DifferenceConstraint(
        source=later,
        target=earlier,
        upper_bound=-minimum_separation,
    )
