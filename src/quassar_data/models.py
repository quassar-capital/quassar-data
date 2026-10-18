"""Validated, point-in-time financial observations."""

from __future__ import annotations

from dataclasses import dataclass
from datetime import datetime, timezone
from decimal import Decimal
import re

_IDENTIFIER = re.compile(r"^[A-Za-z0-9][A-Za-z0-9._:/-]{0,127}$", re.ASCII)
_UNIT = re.compile(r"^[A-Za-z][A-Za-z0-9_/-]{0,31}$", re.ASCII)


def _identifier(value: str, field: str) -> str:
    if not isinstance(value, str) or _IDENTIFIER.fullmatch(value) is None:
        raise ValueError(
            f"{field} must be a nonempty ASCII identifier (maximum 128 characters)"
        )
    return value


def _timestamp(value: datetime, field: str) -> datetime:
    if (
        not isinstance(value, datetime)
        or value.tzinfo is None
        or value.utcoffset() is None
    ):
        raise ValueError(f"{field} must be a timezone-aware datetime")
    return value.astimezone(timezone.utc)


@dataclass(frozen=True, slots=True, kw_only=True)
class FinancialObservation:
    """One numeric observation with explicit publication and availability times.

    Times represent instants, not trading-session labels. The value is a finite
    Decimal supplied by the caller, not a binary floating-point approximation.
    Availability is specific to the research data source and its access policy.
    """

    instrument_id: str
    field: str
    value: Decimal
    unit: str
    observation_time: datetime
    publication_time: datetime
    availability_time: datetime
    source_id: str

    def __post_init__(self) -> None:
        for field in ("instrument_id", "field", "source_id"):
            _identifier(getattr(self, field), field)

        if not isinstance(self.unit, str) or _UNIT.fullmatch(self.unit) is None:
            raise ValueError(
                "unit must be a nonempty ASCII unit code (maximum 32 characters)"
            )

        if not isinstance(self.value, Decimal) or not self.value.is_finite():
            raise ValueError("value must be a finite Decimal")

        for field in (
            "observation_time",
            "publication_time",
            "availability_time",
        ):
            object.__setattr__(
                self,
                field,
                _timestamp(getattr(self, field), field),
            )

        if self.observation_time > self.publication_time:
            raise ValueError(
                "observation_time must not follow publication_time"
            )

        if self.publication_time > self.availability_time:
            raise ValueError(
                "publication_time must not follow availability_time"
            )

    def available_at(self, decision_time: datetime) -> bool:
        """Check temporal eligibility only; not provenance or revision correctness."""
        return self.availability_time <= _timestamp(
            decision_time, "decision_time"
        )
