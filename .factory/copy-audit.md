# Product copy audit

Audited 2026-09-06 against `frontend/src/main.ts`. Counts treat symbols and hyphenated terms as one word. No sentence exceeds 22 words, and no banned marketing word is present.

## First screen

| Copy | Words |
| --- | ---: |
| Record every telemetry export. | 4 |
| For observability teams, this proxy limits downloads and records who requested each one. | 13 |
| Try it with sample data. | 6 |
| Loads allowed, denied, and upstream-error sample receipts. | 6 |
| Use your installation. | 3 |
| Signed JSON and Markdown. | 4 |
| Result bodies never stored. | 4 |
| Optional archive costs $49 once. | 5 |

The first screen states the job, audience, first action, privacy boundary, output, and price in one view.

## Landing sections

| Copy | Words |
| --- | ---: |
| How it works. | 3 |
| Each export has one declared policy check and one signed record. | 11 |
| Send a bounded export request. | 5 |
| Use an approved endpoint and declare the time range, row cap, fields, and purpose. | 14 |
| Check the declared policy. | 5 |
| The proxy checks the bounds before it forwards your existing upstream authorization. | 11 |
| Keep a signed receipt. | 5 |
| The receipt records who requested the export, its bounds, and its outcome. | 12 |
| Send an approved export request. | 5 |
| What it does not do. | 5 |
| It keeps export metadata, a query digest, and a signature. | 10 |
| It does not keep telemetry. | 5 |
| It does not store result bodies. | 6 |
| It does not replace upstream permissions. | 6 |
| It does not provide telemetry dashboards. | 6 |

## Demo entry

| Copy | Words |
| --- | ---: |
| Review sample export receipts. | 4 |
| Three realistic receipts are ready below. | 6 |
| They never reach your installation. | 5 |
| View sample receipts. | 3 |
| Sample receipt desk. | 3 |
| Filter the three sample outcomes, then open a receipt to inspect its signed fields. | 14 |

## Product and state copy

| Copy | Words |
| --- | ---: |
| Signed export records. | 3 |
| Bounded export → signed receipt. | 4 |
| Machine-signed records from this installation. | 5 |
| The newest receipt appears first. | 5 |
| Sample policy active. | 3 |
| Ready to issue receipts. | 4 |
| Upstream needs configuration. | 3 |
| You’re offline. | 2 |
| Showing the most recently loaded receipt list. | 7 |
| No receipts match. | 3 |
| Send a bounded request through the export route, or clear the filters to see all receipts. | 15 |
| Administrator access required. | 3 |
| Enter the token from the server’s admin-access.key file. | 8 |
| It stays in this browser tab. | 6 |
| The ledger could not be reached. | 6 |
| Check the server connection, then use Refresh. | 7 |
| Existing exports remain in SQLite. | 5 |
| The proxy forwards your existing Authorization and Cookie headers only to the configured upstream. | 13 |
| Your trusted auth proxy supplies requester identity. | 7 |
| The upstream body and status return with receipt ID and signature headers. | 11 |
| The core proxy and individual signed receipts stay free. | 9 |
| Fleet archive packages the loaded audit set for an offline review or handoff. | 12 |
| Sociobot/Dodo is merchant of record. | 5 |
| Refunds are handled there. | 4 |
| Demo — sample data, nothing is saved. | 7 |
| Sample receipt — no server record was created. | 8 |
| Records export metadata without storing telemetry. | 6 |

## Terminology

| Concept | Term used |
| --- | --- |
| An attempted telemetry download | export |
| The signed audit record | receipt |
| The receipt collection | receipt desk |
| The policy-enforcing service | proxy |
| The protected operator credential | administrator token |
| The paid bulk JSON file | Fleet archive |
| The external observability API | upstream |
