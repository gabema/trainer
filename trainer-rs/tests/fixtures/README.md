# Golden fixtures

Every file here was captured from the **C# implementation**, so the Rust port is
asserted against what the shipping app actually produced rather than against an
assumed algorithm. Several of them exist because an assumption turned out to be
wrong; those are noted.

The temporary generator that produced them, `Trainer.Tests/Fixtures/GoldenFixtureGenerator.cs`,
was removed once the port no longer needed it (task 1.5). It ran only when
`TRAINER_GENERATE_FIXTURES` was set, so an ordinary `dotnet test` never rewrote a
fixture. To recover it:

```bash
git show 676144a:Trainer.Tests/Fixtures/GoldenFixtureGenerator.cs
```

`InMemoryJsRuntime.cs` deliberately survives it: the cross-implementation tests in
`Trainer.Tests/Services/RustInteropTests.cs` still use it, and both live until
`rust-ui` deletes the C# project.

**These files are inputs, never outputs.** CI asserts that no test run modifies
one — golden data a test can rewrite would make the suite self-confirming, which
is precisely how `export.json` once passed a byte-identity check while testing
nothing.

## Week bucketing

| file | pins |
|---|---|
| `week-keys.csv` | 11,323 `(date, weekKey)` pairs, 2010–2040. Proves the port reproduces .NET's `FirstFourDayWeek` rule paired with the **calendar** year, which is not ISO 8601 — `chrono::IsoWeek` would re-bucket every activity near a year boundary |
| `week-boundaries.csv` | `GetWeekStartDate` / `GetWeekEndDate` for all 1,636 observed week keys |
| `week-key-anomalies.csv` | The 27 years where the first week key does not round-trip, because its Monday lies in the previous year's bucket. Occurs whenever 1 January is not a Monday |
| `week-unmatched-keys.csv` | The fallback for week keys no date in the year produces; no other fixture reached that branch |

## Serialization

| file | pins |
|---|---|
| `timestamps-export-*` / `timestamps-storage-*` | Real `System.Text.Json` output of `List<Activity>` under **both** serializer configurations, across six timezones. Exports omit unset optional fields; storage writes them as `null` |
| `timestamps-roundtrip-*` | What each emitted string parses back to through the converter's own `Read` path |
| `timestamps-crosszone-*` | Reading Los Angeles data under another timezone. Shows the C# reader **discards the parsed offset**, so re-saving in a different zone keeps the wall clock but moves the instant |
| `json-escaping.json` | All 128 ASCII code points plus a non-ASCII spread. `JavaScriptEncoder.Default` escapes `" & ' + < > \`` and everything at U+007F and above — so every positive UTC offset contains an escaped `+` |
| `double-formatting.json` | How doubles are written. A whole-valued coordinate is `10`, not `10.0` |

## Data and services

| file | pins |
|---|---|
| `export.json` | A real 527-activity export, de-identified by `deidentify.py`. Keeps every structural property: field-presence combinations, the three `notes` states, both hour-only offsets, signed location ids, the `2026.01` boundary bucket |
| `idb-snapshot.json` | A real raw IndexedDB dump, de-identified. Proves values are structured-cloned **Arrays**, that `activityNextId` is a bare **Number**, and that storage writes every optional field explicitly |
| `csharp-export.json` | An export whose bytes come start-to-finish from the C# export path, with no de-identifier in between |
| `rust-export.json` | Produced by the **Rust** serializer, imported by the C# side, so the two implementations meet on a real file |
| `active-activities.json` | The third wire format. `ActiveActivityService` serializes with default options and no `DateTimeConverter`: full `±hh:mm` offsets, an offset-less form RFC 3339 rejects, and trimmed fractional seconds |
| `legacy-migration.json` | The one-time localStorage-to-IndexedDB migration, driven through the real code path |

## De-identification

`deidentify.py` produces `export.json` and `idb-snapshot.json` from a real profile.
It preserves structure while replacing names, units, note text and coordinates,
and it emits **.NET-escaped** JSON so the fixture matches what the app writes.

Three times during this port a synthetic replacement silently weakened a fixture
by differing from the real value in character class or numeric form: plain-ASCII
notes hid all escaping, then missing escape-triggering characters, then Python's
float `repr` writing `10.0` where .NET writes `10`. Anyone regenerating these
should assume that failure mode rather than trust the output.

## `indexeddb-storage.js`

The JavaScript IndexedDB shim from the C# implementation, verbatim. Not test
data but a test *subject*: `trainer-web/src/shim_interop_tests.rs` loads it into
the browser and drives it against the Rust storage layer, so the two are checked
for agreement by running both rather than by comparing one to a description.

It is kept here because it is the program that wrote every existing user's
database. `Trainer/` was deleted once the Rust build shipped; this file outlives
it for the same reason `csharp-export.json` does.
