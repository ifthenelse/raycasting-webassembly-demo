# Testing

## Strategy
TODO: what is covered by which kind of test.

## Suites
| Suite | Location | Speed | Run with |
| --- | --- | --- | --- |
| unit | `tests/unit/` | fast | `scripts/verify` |
| integration | TODO | TODO | TODO |
| e2e | TODO | slow, opt-in | TODO |

## Fixtures and environments
Fixtures are kept apart from production data. Tests that touch live or state-changing systems are opt-in and isolated.

## Definition of a passing release
`scripts/verify` green in CI, plus: TODO.
