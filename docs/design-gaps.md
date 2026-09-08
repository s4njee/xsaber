# xsaber design gaps

This is a dated ledger of deliberate deviations from
`design_handoff_xsaber_ftp_client/`. The handoff is reference material, not a
product specification. Every later story that departs from it appends a row
with the date, rationale, and acceptance impact.

| ID | Date | Mock treatment | xsaber treatment | Rationale / acceptance impact |
|---|---|---|---|---|
| DG-001 | 2026-09-08 | The mock labels the connection as an FTP/SFTP client. | The badge reads `XSYNC · ED25519`, or `SFTP · ED25519` on fallback. | xsync v3 is the primary protocol; FTP/FTPS are out of scope for v1. The badge must match actual capabilities. |
| DG-002 | 2026-09-08 | The site tree lists FTP, FTPS and S3 alongside SFTP. | Those entries remain visible as disabled protocol options with an explanatory tooltip. | They are non-goals for v1, but retaining the affordance makes the unsupported scope discoverable. |
| DG-003 | 2026-09-08 | Help text says “Preserve timestamps (MFMT)”. | Help text says “Preserve timestamps”; no MFMT claim is made. | MFMT is an FTP verb and xsaber is not an FTP client. Timestamp preservation follows the selected backend's native API. |
| DG-004 | 2026-09-08 | “Verify checksums” implies SHA-256. | Verification compares BLAKE3 for xsync transfers; SFTP uses the available transfer verification policy. | BLAKE3 is the xsync integrity primitive. The UI must name the algorithm rather than imply SHA-256. |
| DG-005 | 2026-09-08 | The Apply button's treatment is left as an open question in the mock. | Apply remains a solid button. | This is the least ambiguous usable state until the mock's open question is resolved; revisit if the design changes. |
| DG-006 | 2026-09-08 | The mock presents a Sync tab as a finished navigation destination. | Sync is a placeholder in v1. | Sync is a non-goal; the tab may explain that the feature is planned but must not imply a working sync engine. |
