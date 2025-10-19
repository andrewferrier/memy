# Architectural Principles

- Don't use [auto_vacuum](https://sqlite.org/pragma.html#pragma_auto_vacuum) - can actually make performance worse, and the database is never likely to be *that* big.
