# `wasmer-runtime-core` [DEPRECATED] [![Build Status](https://github.com/wasmerio/wasmer/workflows/build/badge.svg?style=flat-square)](https://github.com/wasmerio/wasmer/actions?query=workflow%3Abuild) [![Join Wasmer Slack](https://img.shields.io/static/v1?label=Slack&message=join%20chat&color=brighgreen&style=flat-square)](https://slack.wasmer.io) [![MIT License](https://img.shields.io/github/license/wasmerio/wasmer.svg?style=flat-square)](https://github.com/wasmerio/wasmer/blob/master/LICENSE) [![crates.io](https://img.shields.io/crates/v/wasmer-runtime-core.svg)](https://crates.io/crates/wasmer-runtime-core)

## Deprecation notice: please read

Thanks to users feedback, collected experience and various use cases,
Wasmer has decided to entirely improve its API to offer the best user
experience and the best features to as many users as possible.

The new version of Wasmer (`1.0.0`) includes many improvements
in terms of performance or the memory consumption, in addition to a ton
of new features and much better flexibility!
You can check revamped new API in the [`wasmer`] crate.

In order to help our existing users to enjoy the performance boost and
memory improvements without updating their program that much, we have
created a new version of the `wasmer-runtime-core` crate, which is now
*an adaptation* of the new API but with the old API syntax, as much as
possible. Indeed, it was not always possible to provide the exact same
API, but changes are subtle.

We have carefully documented most of the differences in [the
`runtime-core/CHANGES.md` document][changes].

It is important to understand the public of this port. We do not
recommend to advanced users of Wasmer to use this port. Advanced API,
like `ModuleInfo` or the `vm` module (incl. `vm::Ctx`) have not been
fully ported because it was very internals to Wasmer. For advanced
users, we highly recommend to migrate to the new version of Wasmer,
which is awesome by the way (completely neutral opinion). The public
for this port is beginners or regular users that do not necesarily
have time to update their code immediately but that want to enjoy a
performance boost and memory improvements.

[`wasmer`]: https://crates.io/crates/wasmer/
[changes]: ./CHANGES.md

## Introduction

The `wasmer-runtime-core` was the entry point to the Wasmer Runtime,
by providing common types to compile and to instantiate a WebAssembly
module.

Most Wasmer users should prefer the API which is re-exported by the
[`wasmer-runtime`] library by default. This crate provides additional
APIs which may be useful to users that wish to customize the Wasmer
runtime.

[`wasmer-runtime`]: https://github.com/wasmerio/wasmer/tree/master/lib/deprecated/runtime
