# ADR-0005: Configuration conventions

- **Status:** Accepted
- **Date:** 2026-10-16

## Context

The server must start from documented configuration, and configuration errors
must be actionable. The project also has a security requirement that sensitive
credentials never reach the logs, and remote access must be disabled unless it is
configured.

Without a convention, configuration leaks into code, defaults become implicit,
and the same option is spelled differently in different subsystems.

## Decision

- **Sources, in order of precedence:** environment variables override a
  configuration file, which overrides built-in defaults. The file path itself
  comes from the environment or a command-line argument.
- **Environment prefix:** all environment variables use `TIMBRE_`, for example
  `TIMBRE_CONFIG` and `TIMBRE_LOG_LEVEL`.
- **Validated at startup:** configuration is loaded and validated before the
  server serves traffic. An invalid value fails startup with a message naming the
  key, the offending value, and the expected form.
- **No configuration in code:** a value that may change between deployments is
  configuration. Defaults are declared with the loader, not scattered at call
  sites.
- **Secrets:** secrets are read from the environment or a file that is never
  committed, are wrapped so they do not print, and are never logged. `.env` and
  `.env.*` are ignored by Git.
- **Secure by default:** features that open the system to the network, remote
  access in particular, are off unless explicitly enabled and configured.
- **Typed access:** subsystems receive a typed configuration struct from the
  server; they do not read the environment directly.

## Consequences

A deployment's behavior is reproducible from its file plus its environment, and a
mistyped key fails loudly at startup instead of silently taking a default.

The cost is a small amount of ceremony: a new option must be added to the typed
configuration and documented, rather than read ad hoc where it is used.
