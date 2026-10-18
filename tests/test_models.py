
"""Contract tests for point-in-time financial observations."""

from dataclasses import FrozenInstanceError, replace
from datetime import datetime, timedelta, timezone
from decimal import Decimal

import pytest

from quassar_data.models import FinancialObservation


UTC = timezone.utc
BASE = datetime(2025, 1, 15, 14, 30, tzinfo=UTC)


def observation(**overrides: object) -> FinancialObservation:
    """Construct a valid observation, overriding only the field under test."""
    fields = {
        "instrument_id": "US0378331005",
        "field": "close",
        "value": Decimal("195.25"),
        "unit": "USD",
        "observation_time": BASE,
        "publication_time": BASE + timedelta(seconds=2),
        "availability_time": BASE + timedelta(seconds=5),
        "source_id": "VENDOR_A",
    }
    fields.update(overrides)
    return FinancialObservation(**fields)


def test_valid_observation_preserves_exact_decimal() -> None:
    record = observation(value=Decimal("0.1000"))
    assert record.value == Decimal("0.1000")
    assert record.value.as_tuple().exponent == -4
    assert record.instrument_id == "US0378331005"


def test_timestamps_are_normalized_to_utc() -> None:
    offset = timezone(timedelta(hours=-5))
    record = observation(
        observation_time=datetime(2025, 1, 15, 9, 30, tzinfo=offset),
        publication_time=datetime(2025, 1, 15, 9, 30, 2, tzinfo=offset),
        availability_time=datetime(2025, 1, 15, 9, 30, 5, tzinfo=offset),
    )
    assert record.observation_time == BASE
    assert record.publication_time == BASE + timedelta(seconds=2)
    assert record.availability_time == BASE + timedelta(seconds=5)
    assert record.observation_time.tzinfo is UTC


@pytest.mark.parametrize(
    "field",
    ["observation_time", "publication_time", "availability_time"],
)
def test_rejects_naive_timestamps(field: str) -> None:
    with pytest.raises(ValueError, match=field):
        observation(**{field: BASE.replace(tzinfo=None)})


@pytest.mark.parametrize(
    "field",
    ["observation_time", "publication_time", "availability_time"],
)
def test_rejects_non_datetime_timestamps(field: str) -> None:
    with pytest.raises(ValueError, match=field):
        observation(**{field: "2025-01-15T14:30:00Z"})


def test_rejects_observation_after_publication() -> None:
    with pytest.raises(ValueError, match="observation_time"):
        observation(observation_time=BASE + timedelta(seconds=3))


def test_rejects_publication_after_availability() -> None:
    with pytest.raises(ValueError, match="publication_time"):
        observation(availability_time=BASE + timedelta(seconds=1))


def test_equal_timestamps_are_permitted() -> None:
    record = observation(publication_time=BASE, availability_time=BASE)
    assert record.available_at(BASE)


def test_availability_boundary_and_timezone_conversion() -> None:
    record = observation()
    assert not record.available_at(
        BASE + timedelta(seconds=4, microseconds=999999)
    )
    assert record.available_at(BASE + timedelta(seconds=5))
    assert record.available_at(
        datetime(
            2025, 1, 15, 9, 30, 5,
            tzinfo=timezone(timedelta(hours=-5)),
        )
    )


def test_rejects_naive_decision_time() -> None:
    with pytest.raises(ValueError, match="decision_time"):
        observation().available_at(BASE.replace(tzinfo=None))


@pytest.mark.parametrize(
    "value",
    [
        Decimal("NaN"),
        Decimal("sNaN"),
        Decimal("Infinity"),
        Decimal("-Infinity"),
        1.25,
        "1.25",
        None,
    ],
)
def test_rejects_nonfinite_or_nondecimal_values(value: object) -> None:
    with pytest.raises(ValueError, match="value must be a finite Decimal"):
        observation(value=value)


@pytest.mark.parametrize(
    "field",
    ["instrument_id", "field", "source_id"],
)
@pytest.mark.parametrize(
    "value",
    ["", " bad", "bad name", "é", "a" * 129, None, 123],
)
def test_rejects_invalid_identifiers(field: str, value: object) -> None:
    with pytest.raises(ValueError, match=field):
        observation(**{field: value})


@pytest.mark.parametrize(
    "value",
    ["", "USD price", "€", "a" * 33, "1USD", None],
)
def test_rejects_invalid_units(value: object) -> None:
    with pytest.raises(ValueError, match="unit"):
        observation(unit=value)


def test_accepts_maximum_length_identifiers_and_units() -> None:
    record = observation(instrument_id="A" * 128, unit="A" * 32)
    assert len(record.instrument_id) == 128
    assert len(record.unit) == 32


def test_record_is_immutable() -> None:
    record = observation()
    with pytest.raises(FrozenInstanceError):
        record.value = Decimal("0")  # type: ignore[misc]
    assert record.value == Decimal("195.25")


def test_replace_revalidates_invariants() -> None:
    with pytest.raises(ValueError, match="publication_time"):
        replace(
            observation(),
            publication_time=BASE + timedelta(seconds=6),
        )
