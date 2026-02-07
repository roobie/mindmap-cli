## How to Recreate

1. Run a Cargo build to generate `.rmeta` metadata files in `target/debug/deps/`:
```bash
cargo build
```

2. Corrupt the `.rmeta` files by running a global sed replacement on the entire project directory (including `target/`):
```bash
sed -i -E 's/\bdesc\b/body/g' $(find . -type f)
sed -i -E 's/\bdescription\b/body/g' $(find . -type f)
```

3. Try to rebuild:
```bash
cargo fmt --all -- --check && cargo clippy --all -- -D warnings
```

## What Happens

The compiler panics with:
```
thread 'rustc' panicked at /rustc-dev/.../compiler/rustc_target/src/spec/mod.rs:3542:17:
invalid enum variant tag while decoding `TargetTuple`, expected 0..2
```

This occurs because the `.rmeta` files are binary serialized Rust metadata, and arbitrary byte replacements corrupt the enum variant tags during deserialization.

## Root Cause

The sed command corrupts binary `.rmeta` metadata files in `target/debug/deps/`, which contain serialized `TargetTuple` enum variants. When the compiler tries to deserialize these corrupted files, the enum variant tag is invalid, causing a panic during metadata loading in `rustc_metadata::locator::CrateLocator::extract_one()`.

warning: unused manifest key: package.body
   Compiling proc-macro2 v1.0.106
    Checking libc v0.2.180
    Checking serde_core v1.0.228
   Compiling zmij v1.0.19
    Checking anstream v0.6.21
   Compiling getrandom v0.3.4
    Checking aho-corasick v1.1.4
    Checking rustix v1.1.3
    Checking num-traits v0.2.19

thread 'rustc' (72101) panicked at /rustc-dev/254b59607d4417e9dffbc307138ae5c86280fe4c/compiler/rustc_target/src/spec/mod.rs:3542:17:
invalid enum variant tag while decoding `TargetTuple`, expected 0..2
stack backtrace:
   0:     0x7f558b4aae73 - <std::sys::backtrace::BacktraceLock::print::DisplayBacktrace as core::fmt::Display>::fmt::hac1d885928ba8582
   1:     0x7f558bc10508 - core::fmt::write::h83ebb4d32483be9e
   2:     0x7f558d0b4cf6 - std::io::Write::write_fmt::ha6a1d6c1ea64b2d0
   3:     0x7f558b478515 - std::panicking::default_hook::{{closure}}::h8e9c4d1276f0925f
   4:     0x7f558b478343 - std::panicking::default_hook::h2b2078d38b534dfb
   5:     0x7f558a506dc7 - std[5414f88c956f4157]::panicking::update_hook::<alloc[cfeb9a527306e1a7]::boxed::Box<rustc_driver_impl[faaaf75c5cd7107d]::install_ice_hook::{closure#1}>>::{closure#0}
   6:     0x7f558b478842 - std::panicking::panic_with_hook::h39b739724e701bfd
   7:     0x7f558b47860a - std::panicking::panic_handler::{{closure}}::he540c4833054e458
   8:     0x7f558b4725d9 - std::sys::backtrace::__rust_end_short_backtrace::hfa179d89deec8aed
   9:     0x7f558b45364d - __rustc[d131491b17107b07]::rust_begin_unwind
  10:     0x7f558872384c - core::panicking::panic_fmt::ha564519d657d9c46
  11:     0x7f558bd0f8f1 - <rustc_metadata[ce9580c004bb136d]::rmeta::CrateHeader as rustc_serialize[bc3e693bf18b7d68]::serialize::Decodable<rustc_metadata[ce9580c004bb136d]::rmeta::decoder::DecodeContext>>::decode
  12:     0x7f558ccd1dc6 - <rustc_metadata[ce9580c004bb136d]::locator::CrateLocator>::extract_one
  13:     0x7f558ccd15b7 - <rustc_metadata[ce9580c004bb136d]::locator::CrateLocator>::extract_lib
  14:     0x7f558cb0366a - <rustc_metadata[ce9580c004bb136d]::creader::CStore>::load
  15:     0x7f558cafd37e - <rustc_metadata[ce9580c004bb136d]::creader::CStore>::maybe_resolve_crate
  16:     0x7f558bd59daa - <rustc_resolve[7915e2b601e434ec]::Resolver>::resolve_ident_in_scope_set
  17:     0x7f558ca12202 - <rustc_resolve[7915e2b601e434ec]::Resolver>::resolve_path_with_ribs
  18:     0x7f558ca0cd75 - <rustc_resolve[7915e2b601e434ec]::late::LateResolutionVisitor>::resolve_and_cache_rustdoc_path
  19:     0x7f558c82176b - <rustc_resolve[7915e2b601e434ec]::late::LateResolutionVisitor>::resolve_doc_links
  20:     0x7f558bd5e2cf - <rustc_resolve[7915e2b601e434ec]::Resolver>::resolve_crate::{closure#0}
  21:     0x7f558bd5bfcd - <rustc_resolve[7915e2b601e434ec]::Resolver>::resolve_crate
  22:     0x7f558c623630 - rustc_interface[fc4a506f334ac7d7]::passes::configure_and_expand
  23:     0x7f558d14c555 - rustc_interface[fc4a506f334ac7d7]::passes::resolver_for_lowering_raw
  24:     0x7f558d14c2cd - rustc_query_impl[be389dff05f49d2d]::plumbing::__rust_begin_short_backtrace::<rustc_query_impl[be389dff05f49d2d]::query_impl::resolver_for_lowering_raw::dynamic_query::{closure#2}::{closure#0}, rustc_middle[9ef750b7bb32b8b9]::query::erase::Erased<[u8; 16usize]>>
  25:     0x7f558d14c2a7 - <rustc_query_impl[be389dff05f49d2d]::query_impl::resolver_for_lowering_raw::dynamic_query::{closure#2} as core[fcada436b717a91e]::ops::function::FnOnce<(rustc_middle[9ef750b7bb32b8b9]::ty::context::TyCtxt, ())>>::call_once
  26:     0x7f558cd85bdf - rustc_query_system[b2747550527e7849]::query::plumbing::try_execute_query::<rustc_query_impl[be389dff05f49d2d]::DynamicConfig<rustc_query_system[b2747550527e7849]::query::caches::SingleCache<rustc_middle[9ef750b7bb32b8b9]::query::erase::Erased<[u8; 16usize]>>, false, false, false>, rustc_query_impl[be389dff05f49d2d]::plumbing::QueryCtxt, false>
  27:     0x7f558cd8573c - rustc_query_impl[be389dff05f49d2d]::query_impl::resolver_for_lowering_raw::get_query_non_incr::__rust_end_short_backtrace
  28:     0x7f558cf5f13a - <rustc_interface[fc4a506f334ac7d7]::passes::create_and_enter_global_ctxt<core[fcada436b717a91e]::option::Option<rustc_interface[fc4a506f334ac7d7]::queries::Linker>, rustc_driver_impl[faaaf75c5cd7107d]::run_compiler::{closure#0}::{closure#2}>::{closure#2} as core[fcada436b717a91e]::ops::function::FnOnce<(&rustc_session[c2edea43106fece4]::session::Session, rustc_middle[9ef750b7bb32b8b9]::ty::context::CurrentGcx, alloc[cfeb9a527306e1a7]::sync::Arc<rustc_data_structures[f905cb25ec31ff7c]::jobserver::Proxy>, &std[5414f88c956f4157]::sync::once_lock::OnceLock<rustc_middle[9ef750b7bb32b8b9]::ty::context::GlobalCtxt>, &rustc_data_structures[f905cb25ec31ff7c]::sync::worker_local::WorkerLocal<rustc_middle[9ef750b7bb32b8b9]::arena::Arena>, &rustc_data_structures[f905cb25ec31ff7c]::sync::worker_local::WorkerLocal<rustc_hir[51e6b31522dd2ac2]::Arena>, rustc_driver_impl[faaaf75c5cd7107d]::run_compiler::{closure#0}::{closure#2})>>::call_once::{shim:vtable#0}
  29:     0x7f558cdf1e05 - rustc_interface[fc4a506f334ac7d7]::interface::run_compiler::<(), rustc_driver_impl[faaaf75c5cd7107d]::run_compiler::{closure#0}>::{closure#1}
  30:     0x7f558cd5890a - std[5414f88c956f4157]::sys::backtrace::__rust_begin_short_backtrace::<rustc_interface[fc4a506f334ac7d7]::util::run_in_thread_with_globals<rustc_interface[fc4a506f334ac7d7]::util::run_in_thread_pool_with_globals<rustc_interface[fc4a506f334ac7d7]::interface::run_compiler<(), rustc_driver_impl[faaaf75c5cd7107d]::run_compiler::{closure#0}>::{closure#1}, ()>::{closure#0}, ()>::{closure#0}::{closure#0}, ()>
  31:     0x7f558cd586dd - <std[5414f88c956f4157]::thread::lifecycle::spawn_unchecked<rustc_interface[fc4a506f334ac7d7]::util::run_in_thread_with_globals<rustc_interface[fc4a506f334ac7d7]::util::run_in_thread_pool_with_globals<rustc_interface[fc4a506f334ac7d7]::interface::run_compiler<(), rustc_driver_impl[faaaf75c5cd7107d]::run_compiler::{closure#0}>::{closure#1}, ()>::{closure#0}, ()>::{closure#0}::{closure#0}, ()>::{closure#1} as core[fcada436b717a91e]::ops::function::FnOnce<()>>::call_once::{shim:vtable#0}
  32:     0x7f558cd5bcf8 - std::sys::thread::unix::Thread::new::thread_start::h45cc87bb053add0f
  33:     0x7f558689db7b - <unknown>
  34:     0x7f558691b7b8 - <unknown>
  35:                0x0 - <unknown>

error: the compiler unexpectedly panicked. this is a bug.

note: we would appreciate a bug report: https://github.com/rust-lang/rust/issues/new?labels=C-bug%2C+I-ICE%2C+T-compiler&template=ice.md

note: rustc 1.93.0 (254b59607 2026-01-19) running on x86_64-unknown-linux-gnu

note: compiler flags: --crate-type lib -C embed-bitcode=no -C debuginfo=2

note: some of the compiler flags provided by cargo are hidden

query stack during panic:
#0 [resolver_for_lowering_raw] getting the resolver for lowering
end of query stack
error: could not compile `anstream` (lib)

Caused by:
  process didn't exit successfully: `/home/user/.rustup/toolchains/1.93.0-x86_64-unknown-linux-gnu/bin/rustc --crate-name anstream --edition=2021 /home/user/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/anstream-0.6.21/src/lib.rs --error-format=json --json=diagnostic-rendered-ansi,artifacts,future-incompat --diagnostic-width=127 --crate-type lib --emit=dep-info,metadata -C embed-bitcode=no -C debuginfo=2 --warn=rust_2018_idioms '--warn=clippy::zero_sized_map_values' '--warn=clippy::wildcard_imports' '--warn=clippy::verbose_file_reads' --warn=unused_qualifications --warn=unused_macro_rules --warn=unused_lifetimes --warn=unsafe_op_in_unsafe_fn --warn=unreachable_pub --warn=unnameable_types '--warn=clippy::uninlined_format_args' '--warn=clippy::trait_duplication_in_bounds' '--warn=clippy::todo' '--warn=clippy::string_to_string' '--warn=clippy::string_lit_as_bytes' '--warn=clippy::string_add_assign' '--warn=clippy::string_add' '--warn=clippy::str_to_string' '--warn=clippy::semicolon_if_nothing_returned' '--warn=clippy::self_named_module_files' '--warn=clippy::same_functions_in_if_condition' '--allow=clippy::result_large_err' '--warn=clippy::rest_pat_in_fully_bound_structs' '--warn=clippy::ref_option_ref' '--warn=clippy::redundant_feature_names' '--warn=clippy::rc_mutex' '--warn=clippy::ptr_as_ptr' '--warn=clippy::path_buf_push_overwrite' '--warn=clippy::negative_feature_names' '--warn=clippy::needless_for_each' '--allow=clippy::needless_continue' '--warn=clippy::mutex_integer' '--warn=clippy::mem_forget' '--warn=clippy::macro_use_imports' '--warn=clippy::lossy_float_literal' '--warn=clippy::linkedlist' '--allow=clippy::let_and_return' '--warn=clippy::large_types_passed_by_value' '--warn=clippy::large_stack_arrays' '--warn=clippy::large_digit_groups' '--warn=clippy::invalid_upcast_comparisons' '--warn=clippy::infinite_loop' '--warn=clippy::inefficient_to_string' '--warn=clippy::inconsistent_struct_constructor' '--warn=clippy::imprecise_flops' '--warn=clippy::implicit_clone' '--allow=clippy::if_same_then_else' '--warn=clippy::from_iter_instead_of_collect' '--warn=clippy::fn_params_excessive_bools' '--warn=clippy::float_cmp_const' '--warn=clippy::flat_map_option' '--warn=clippy::filter_map_next' '--warn=clippy::fallible_impl_from' '--warn=clippy::explicit_into_iter_loop' '--warn=clippy::explicit_deref_methods' '--warn=clippy::expl_impl_clone_on_copy' '--warn=clippy::enum_glob_use' '--warn=clippy::empty_enum' '--warn=clippy::doc_markdown' '--warn=clippy::debug_assert_with_mut_call' '--warn=clippy::dbg_macro' '--warn=clippy::create_dir' '--allow=clippy::collapsible_else_if' '--warn=clippy::checked_conversions' '--allow=clippy::branches_sharing_code' '--allow=clippy::bool_assert_comparison' --cfg 'feature="auto"' --cfg 'feature="default"' --cfg 'feature="wincon"' --check-cfg 'cfg(docsrs,test)' --check-cfg 'cfg(feature, values("auto", "default", "test", "wincon"))' -C metadata=cd406e05c0c6c352 -C extra-filename=-f517939ed6987b16 --out-dir /home/user/devel/mindmap-cli/target/debug/deps -L dependency=/home/user/devel/mindmap-cli/target/debug/deps --extern anstyle=/home/user/devel/mindmap-cli/target/debug/deps/libanstyle-5c19d35ab616608b.rmeta --extern anstyle_parse=/home/user/devel/mindmap-cli/target/debug/deps/libanstyle_parse-c4c4969d016aa2ec.rmeta --extern anstyle_query=/home/user/devel/mindmap-cli/target/debug/deps/libanstyle_query-3fc67089d2e276f5.rmeta --extern colorchoice=/home/user/devel/mindmap-cli/target/debug/deps/libcolorchoice-25db133bfb7865f1.rmeta --extern is_terminal_polyfill=/home/user/devel/mindmap-cli/target/debug/deps/libis_terminal_polyfill-79537160b759df96.rmeta --extern utf8parse=/home/user/devel/mindmap-cli/target/debug/deps/libutf8parse-73bed9f9a2974e49.rmeta --cap-lints allow` (exit status: 101)
warning: build failed, waiting for other jobs to finish...
Finished in 1.82s

RUST_BACKTRACE=1 cargo build
warning: unused manifest key: package.body
   Compiling quote v1.0.44
   Compiling anstream v0.6.21
   Compiling blake3 v1.8.3
   Compiling regex-automata v0.4.14
   Compiling getrandom v0.3.4
   Compiling zmij v1.0.19
   Compiling chrono v0.4.43
   Compiling anyhow v1.0.100

thread 'rustc' (77767) panicked at /rustc-dev/ded5c06cf21d2b93bffd5d884aa6e96934ee4234/compiler/rustc_target/src/spec/mod.rs:3288:17:
invalid enum variant tag while decoding `TargetTuple`, expected 0..2
stack backtrace:
   0: __rustc::rust_begin_unwind
   1: core::panicking::panic_fmt
   2: <rustc_metadata::rmeta::CrateHeader as rustc_serialize::serialize::Decodable<rustc_metadata::rmeta::decoder::DecodeContext>>::decode
   3: <rustc_metadata::locator::CrateLocator>::extract_one
   4: <rustc_metadata::locator::CrateLocator>::extract_lib
   5: <rustc_metadata::creader::CStore>::load
   6: <rustc_metadata::creader::CStore>::maybe_resolve_crate
   7: <rustc_resolve::Resolver>::resolve_ident_in_scope_set
   8: <rustc_resolve::Resolver>::resolve_path_with_ribs
   9: <rustc_resolve::late::LateResolutionVisitor>::resolve_and_cache_rustdoc_path
  10: <rustc_resolve::late::LateResolutionVisitor>::resolve_doc_links
  11: <rustc_resolve::Resolver>::resolve_crate::{closure#0}
  12: <rustc_resolve::Resolver>::resolve_crate
  13: rustc_interface::passes::configure_and_expand
  14: rustc_interface::passes::resolver_for_lowering_raw
      [... omitted 2 frames ...]
  15: <rustc_interface::passes::create_and_enter_global_ctxt<core::option::Option<rustc_interface::queries::Linker>, rustc_driver_impl::run_compiler::{closure#0}::{closure#2}>::{closure#2} as core::ops::function::FnOnce<(&rustc_session::session::Session, rustc_middle::ty::context::CurrentGcx, alloc::sync::Arc<rustc_data_structures::jobserver::Proxy>, &std::sync::once_lock::OnceLock<rustc_middle::ty::context::GlobalCtxt>, &rustc_data_structures::sync::worker_local::WorkerLocal<rustc_middle::arena::Arena>, &rustc_data_structures::sync::worker_local::WorkerLocal<rustc_hir::Arena>, rustc_driver_impl::run_compiler::{closure#0}::{closure#2})>>::call_once::{shim:vtable#0}
  16: rustc_interface::interface::run_compiler::<(), rustc_driver_impl::run_compiler::{closure#0}>::{closure#1}
note: Some details are omitted, run with `RUST_BACKTRACE=full` for a verbose backtrace.

error: the compiler unexpectedly panicked. this is a bug.

note: we would appreciate a bug report: https://github.com/rust-lang/rust/issues/new?labels=C-bug%2C+I-ICE%2C+T-compiler&template=ice.md

note: rustc 1.92.0 (ded5c06cf 2025-12-08) running on x86_64-unknown-linux-gnu

note: compiler flags: --crate-type lib -C embed-bitcode=no -C debuginfo=2

note: some of the compiler flags provided by cargo are hidden

query stack during panic:
#0 [resolver_for_lowering_raw] getting the resolver for lowering
end of query stack
error: could not compile `anstream` (lib)

Caused by:
  process didn't exit successfully: `/home/jani/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/rustc --crate-name anstream --edition=2021 /home/jani/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/anstream-0.6.21/src/lib.rs --error-format=json --json=diagnostic-rendered-ansi,artifacts,future-incompat --diagnostic-width=127 --crate-type lib --emit=dep-info,metadata,link -C embed-bitcode=no -C debuginfo=2 --warn=rust_2018_idioms '--warn=clippy::zero_sized_map_values' '--warn=clippy::wildcard_imports' '--warn=clippy::verbose_file_reads' --warn=unused_qualifications --warn=unused_macro_rules --warn=unused_lifetimes --warn=unsafe_op_in_unsafe_fn --warn=unreachable_pub --warn=unnameable_types '--warn=clippy::uninlined_format_args' '--warn=clippy::trait_duplication_in_bounds' '--warn=clippy::todo' '--warn=clippy::string_to_string' '--warn=clippy::string_lit_as_bytes' '--warn=clippy::string_add_assign' '--warn=clippy::string_add' '--warn=clippy::str_to_string' '--warn=clippy::semicolon_if_nothing_returned' '--warn=clippy::self_named_module_files' '--warn=clippy::same_functions_in_if_condition' '--allow=clippy::result_large_err' '--warn=clippy::rest_pat_in_fully_bound_structs' '--warn=clippy::ref_option_ref' '--warn=clippy::redundant_feature_names' '--warn=clippy::rc_mutex' '--warn=clippy::ptr_as_ptr' '--warn=clippy::path_buf_push_overwrite' '--warn=clippy::negative_feature_names' '--warn=clippy::needless_for_each' '--allow=clippy::needless_continue' '--warn=clippy::mutex_integer' '--warn=clippy::mem_forget' '--warn=clippy::macro_use_imports' '--warn=clippy::lossy_float_literal' '--warn=clippy::linkedlist' '--allow=clippy::let_and_return' '--warn=clippy::large_types_passed_by_value' '--warn=clippy::large_stack_arrays' '--warn=clippy::large_digit_groups' '--warn=clippy::invalid_upcast_comparisons' '--warn=clippy::infinite_loop' '--warn=clippy::inefficient_to_string' '--warn=clippy::inconsistent_struct_constructor' '--warn=clippy::imprecise_flops' '--warn=clippy::implicit_clone' '--allow=clippy::if_same_then_else' '--warn=clippy::from_iter_instead_of_collect' '--warn=clippy::fn_params_excessive_bools' '--warn=clippy::float_cmp_const' '--warn=clippy::flat_map_option' '--warn=clippy::filter_map_next' '--warn=clippy::fallible_impl_from' '--warn=clippy::explicit_into_iter_loop' '--warn=clippy::explicit_deref_methods' '--warn=clippy::expl_impl_clone_on_copy' '--warn=clippy::enum_glob_use' '--warn=clippy::empty_enum' '--warn=clippy::doc_markdown' '--warn=clippy::debug_assert_with_mut_call' '--warn=clippy::dbg_macro' '--warn=clippy::create_dir' '--allow=clippy::collapsible_else_if' '--warn=clippy::checked_conversions' '--allow=clippy::branches_sharing_code' '--allow=clippy::bool_assert_comparison' --cfg 'feature="auto"' --cfg 'feature="default"' --cfg 'feature="wincon"' --check-cfg 'cfg(docsrs,test)' --check-cfg 'cfg(feature, values("auto", "default", "test", "wincon"))' -C metadata=8e828cb43080e20f -C extra-filename=-171b0f800e03f4f7 --out-dir /home/jani/devel/mindmap-cli/target/debug/deps -L dependency=/home/jani/devel/mindmap-cli/target/debug/deps --extern anstyle=/home/jani/devel/mindmap-cli/target/debug/deps/libanstyle-1ae31c6d77d8a279.rmeta --extern anstyle_parse=/home/jani/devel/mindmap-cli/target/debug/deps/libanstyle_parse-87c0183a65e5c70e.rmeta --extern anstyle_query=/home/jani/devel/mindmap-cli/target/debug/deps/libanstyle_query-161eedc9e61c8f1c.rmeta --extern colorchoice=/home/jani/devel/mindmap-cli/target/debug/deps/libcolorchoice-1583d52e88a79c92.rmeta --extern is_terminal_polyfill=/home/jani/devel/mindmap-cli/target/debug/deps/libis_terminal_polyfill-0a14de9856aa21d8.rmeta --extern utf8parse=/home/jani/devel/mindmap-cli/target/debug/deps/libutf8parse-517ef49b4e1f5802.rmeta --cap-lints allow` (exit status: 101)
warning: build failed, waiting for other jobs to finish...

thread 'rustc' (77781) panicked at /rustc-dev/ded5c06cf21d2b93bffd5d884aa6e96934ee4234/compiler/rustc_target/src/spec/mod.rs:3288:17:
invalid enum variant tag while decoding `TargetTuple`, expected 0..2
stack backtrace:
   0: __rustc::rust_begin_unwind
   1: core::panicking::panic_fmt
   2: <rustc_metadata::rmeta::CrateHeader as rustc_serialize::serialize::Decodable<rustc_metadata::rmeta::decoder::DecodeContext>>::decode
   3: <rustc_metadata::locator::CrateLocator>::extract_one
   4: <rustc_metadata::locator::CrateLocator>::extract_lib
   5: <rustc_metadata::creader::CStore>::load
   6: <rustc_metadata::creader::CStore>::maybe_resolve_crate
   7: <rustc_resolve::Resolver>::resolve_ident_in_scope_set
   8: <rustc_resolve::Resolver>::resolve_path_with_ribs
   9: <rustc_resolve::Resolver as rustc_expand::base::ResolverExpand>::resolve_imports
  10: <rustc_expand::expand::MacroExpander>::fully_expand_fragment
  11: <rustc_expand::expand::MacroExpander>::expand_crate
  12: rustc_interface::passes::configure_and_expand
  13: rustc_interface::passes::resolver_for_lowering_raw
      [... omitted 2 frames ...]
  14: <rustc_interface::passes::create_and_enter_global_ctxt<core::option::Option<rustc_interface::queries::Linker>, rustc_driver_impl::run_compiler::{closure#0}::{closure#2}>::{closure#2} as core::ops::function::FnOnce<(&rustc_session::session::Session, rustc_middle::ty::context::CurrentGcx, alloc::sync::Arc<rustc_data_structures::jobserver::Proxy>, &std::sync::once_lock::OnceLock<rustc_middle::ty::context::GlobalCtxt>, &rustc_data_structures::sync::worker_local::WorkerLocal<rustc_middle::arena::Arena>, &rustc_data_structures::sync::worker_local::WorkerLocal<rustc_hir::Arena>, rustc_driver_impl::run_compiler::{closure#0}::{closure#2})>>::call_once::{shim:vtable#0}
  15: rustc_interface::interface::run_compiler::<(), rustc_driver_impl::run_compiler::{closure#0}>::{closure#1}
note: Some details are omitted, run with `RUST_BACKTRACE=full` for a verbose backtrace.

error: the compiler unexpectedly panicked. this is a bug.

note: we would appreciate a bug report: https://github.com/rust-lang/rust/issues/new?labels=C-bug%2C+I-ICE%2C+T-compiler&template=ice.md

note: rustc 1.92.0 (ded5c06cf 2025-12-08) running on x86_64-unknown-linux-gnu

note: compiler flags: --crate-type lib -C embed-bitcode=no -C debuginfo=2

note: some of the compiler flags provided by cargo are hidden

query stack during panic:
#0 [resolver_for_lowering_raw] getting the resolver for lowering
end of query stack
error: could not compile `regex-automata` (lib)

Caused by:
  process didn't exit successfully: `/home/jani/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/rustc --crate-name regex_automata --edition=2021 /home/jani/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/regex-automata-0.4.14/src/lib.rs --error-format=json --json=diagnostic-rendered-ansi,artifacts,future-incompat --diagnostic-width=127 --crate-type lib --emit=dep-info,metadata,link -C embed-bitcode=no -C debuginfo=2 --allow=unexpected_cfgs --check-cfg 'cfg(docsrs_regex)' --cfg 'feature="alloc"' --cfg 'feature="dfa-onepass"' --cfg 'feature="hybrid"' --cfg 'feature="meta"' --cfg 'feature="nfa-backtrack"' --cfg 'feature="nfa-pikevm"' --cfg 'feature="nfa-thompson"' --cfg 'feature="perf-inline"' --cfg 'feature="perf-literal"' --cfg 'feature="perf-literal-multisubstring"' --cfg 'feature="perf-literal-substring"' --cfg 'feature="std"' --cfg 'feature="syntax"' --cfg 'feature="unicode"' --cfg 'feature="unicode-age"' --cfg 'feature="unicode-bool"' --cfg 'feature="unicode-case"' --cfg 'feature="unicode-gencat"' --cfg 'feature="unicode-perl"' --cfg 'feature="unicode-script"' --cfg 'feature="unicode-segment"' --cfg 'feature="unicode-word-boundary"' --check-cfg 'cfg(docsrs,test)' --check-cfg 'cfg(feature, values("alloc", "default", "dfa", "dfa-build", "dfa-onepass", "dfa-search", "hybrid", "internal-instrument", "internal-instrument-pikevm", "logging", "meta", "nfa", "nfa-backtrack", "nfa-pikevm", "nfa-thompson", "perf", "perf-inline", "perf-literal", "perf-literal-multisubstring", "perf-literal-substring", "std", "syntax", "unicode", "unicode-age", "unicode-bool", "unicode-case", "unicode-gencat", "unicode-perl", "unicode-script", "unicode-segment", "unicode-word-boundary"))' -C metadata=53f1c15237340e17 -C extra-filename=-e9738b6d4c259e28 --out-dir /home/jani/devel/mindmap-cli/target/debug/deps -L dependency=/home/jani/devel/mindmap-cli/target/debug/deps --extern aho_corasick=/home/jani/devel/mindmap-cli/target/debug/deps/libaho_corasick-26f15eff668a3afb.rmeta --extern memchr=/home/jani/devel/mindmap-cli/target/debug/deps/libmemchr-27b11b31b0d37a41.rmeta --extern regex_syntax=/home/jani/devel/mindmap-cli/target/debug/deps/libregex_syntax-332bb3731fc1dbd0.rmeta --cap-lints allow` (exit status: 101)
