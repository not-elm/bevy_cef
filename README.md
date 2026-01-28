# bevy_cef

A powerful Bevy plugin for embedding web content using the Chromium Embedded Framework (CEF).
Render websites, local HTML files, and web applications directly onto 3D meshes or 2D sprites with full interactivity
and bidirectional communication between JavaScript and Bevy.

[![Crates.io](https://img.shields.io/crates/v/bevy_cef)](https://crates.io/crates/bevy_cef)
[![Documentation](https://docs.rs/bevy_cef/badge.svg)](https://docs.rs/bevy_cef)
[![License](https://img.shields.io/badge/license-Apache%202.0%20OR%20MIT-blue.svg)](https://github.com/not-elm/bevy_cef#license)

https://github.com/user-attachments/assets/54f476d0-8eda-4030-a3f6-dc4f2f54209f

## ✨ Features

- **🌐 Full Web Browser Integration** - Embed complete web pages with CSS, JavaScript, and modern web APIs
- **🎮 3D Mesh & 2D Sprite Rendering** - Render web content on any 3D surface or 2D sprite
- **⚡ Interactive Input** - Full mouse, keyboard, and touch input support with automatic event forwarding
- **🔄 Bidirectional Communication** - Seamless data exchange between JavaScript and Bevy systems
- **📁 Local Asset Serving** - Serve local HTML/CSS/JS files with hot reload support
- **🛠️ Developer Tools** - Full Chrome DevTools integration for debugging
- **🎯 Navigation Controls** - Browser history, zoom, audio controls, and more
- **🔒 Multi-Process Architecture** - Secure CEF multi-process design for stability

## 📦 Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
bevy = "0.16"
bevy_cef = "0.1.0"
```

### Platform Requirements

On macOS, using CEF typically requires creating an app bundle.
For development, this library provides a `debug` feature flag.
Once enabled, you can run the app without needing the bundle.

> [!NOTE]
> Use this feature only during development; for releases, bundle the renderer process and the CEF framework inside the
> app.

### Installation debug tools(macOS)

When using `debug`, you need to prepare a separate CEF framework and debug render process.
Please follow the steps below to set it up.

```shell
> cargo install export-cef-dir
> export-cef-dir --force $HOME/.local/share
> cargo install bevy_cef_debug_render_process 
> mv $HOME/.cargo/bin/bevy_cef_debug_render_process "$HOME/.local/share/cef/Chromium Embedded Framework.framework/Libraries/bevy_cef_debug_render_process"
```

## Examples

See [`examples/`](./examples).

On macOS, you need to enable `debug` feature enabled:

```shell
cargo run --example simple --features debug
```

## 🌍 Platform Support

| Platform | Status     | Notes                             |
|----------|------------|-----------------------------------|
| macOS    | ✅ Full     | Primary development platform      |
| Windows  | ⚠️ Planned | CEF support ready, testing needed |
| Linux    | ⚠️ Planned | CEF support ready, testing needed |

## 🤝 Contributing

We welcome contributions! Here's how you can help:

1. **🐛 Bug Reports** - Open an issue with detailed reproduction steps
2. **💡 Feature Requests** - Suggest new features or improvements
3. **🔧 Pull Requests** - Submit bug fixes or new features
4. **📚 Documentation** - Improve docs, examples, or tutorials
5. **🧪 Testing** - Help test on different platforms

### Development Setup

1. Clone the repository
2. Install Rust and Cargo
3. Install the debugging tool with reference to [Installation debug tools](#installation-debug-toolsmacos).
4. Run `cargo build --features debug` to build the project

## Version Compatibility

| Bevy   | bevy_cef | CEF  | Status        |
|--------|----------|------|---------------|
| 0.17 ~ | 0.2.0    | TODO | ⚠️ Unreleased |
| 0.16   | 0.1.0    | 139  | ✅ Current     |

## License

Licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE2) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT License ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as
defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

## Contact

- [Discord](https://discord.com/channels/691052431525675048/1404180578969981018)

I tried `cargo run --example simple --features debug`, but it crashed the app.
Please investigate the cause of the error using the following logs

## Console Logs

```
notelm@notelmnoMacBook-Pro bevy_cef % cargo run --example simple --features debug
warning: /Users/notelm/workspace/bevys/bevy_cef/Cargo.toml: version requirement `144.2.0+144.0.11` for dependency `cef` includes semver metadata which will be ignored, removing the metadata is recommended to avoid confusion
warning: /Users/notelm/workspace/bevys/bevy_cef/crates/bevy_cef_debug_render_process/Cargo.toml: version requirement `144.2.0+144.0.11` for dependency `cef` includes semver metadata which will be ignored, removing the metadata is recommended to avoid confusion
warning: /Users/notelm/workspace/bevys/bevy_cef/crates/bevy_cef_debug_render_process/Cargo.toml: version requirement `144.2.0+144.0.11` for dependency `cef-dll-sys` includes semver metadata which will be ignored, removing the metadata is recommended to avoid confusion
warning: /Users/notelm/workspace/bevys/bevy_cef/crates/bevy_cef_core/Cargo.toml: version requirement `144.2.0+144.0.11` for dependency `cef` includes semver metadata which will be ignored, removing the metadata is recommended to avoid confusion
warning: /Users/notelm/workspace/bevys/bevy_cef/crates/bevy_cef_core/Cargo.toml: version requirement `144.2.0+144.0.11` for dependency `cef-dll-sys` includes semver metadata which will be ignored, removing the metadata is recommended to avoid confusion
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.67s
     Running `target/debug/examples/simple`
2026-01-28T05:15:22.435846Z  INFO bevy_diagnostic::system_information_diagnostics_plugin::internal: SystemInfo { os: "macOS 26.2", kernel: "25.2.0", cpu: "Apple M4 Pro", core_count: "12", memory: "48.0 GiB" }
2026-01-28T05:15:22.536320Z  INFO bevy_render::renderer: AdapterInfo { name: "Apple M4 Pro", vendor: 0, device: 0, device_type: IntegratedGpu, driver: "", driver_info: "", backend: Metal }
[0128/141525.568834:WARNING:cef/libcef/common/resource_util.cc:83] Please customize CefSettings.root_cache_path for your application. Use of the default value may lead to unintended process singleton behavior.
2026-01-28T05:15:25.785527Z  INFO bevy_render::batching::gpu_preprocessing: GPU preprocessing is fully supported on this device.
2026-01-28T05:15:25.810193Z  INFO bevy_winit::system: Creating new window simple (0v0)
[583:21734986:0128/141525.919597:ERROR:content/browser/gpu/gpu_process_host.cc:998] GPU process launch failed: error_code=1003
[583:21734986:0128/141525.921351:ERROR:content/browser/network_service_instance_impl.cc:610] Network service crashed or was terminated, restarting service.
[583:21734986:0128/141525.923313:ERROR:content/browser/gpu/gpu_process_host.cc:998] GPU process launch failed: error_code=1003
[583:21734986:0128/141525.923711:ERROR:content/browser/network_service_instance_impl.cc:610] Network service crashed or was terminated, restarting service.
[583:21734986:0128/141525.924656:ERROR:content/browser/gpu/gpu_process_host.cc:998] GPU process launch failed: error_code=1003
[583:21734986:0128/141525.924676:FATAL:content/browser/gpu/gpu_data_manager_impl_private.cc:415] GPU process isn't usable. Goodbye.
zsh: trace trap  cargo run --example simple --features debug

```

## Crash report

```
Sleep/Wake UUID:       050ECDBD-446D-4AE9-894A-6BEFD88138E3

Time Awake Since Boot: 1100000 seconds
Time Since Wake:       133850 seconds

System Integrity Protection: enabled

Triggered by Thread: 0  CrBrowserMain, Dispatch Queue: com.apple.main-thread

Exception Type:    EXC_BREAKPOINT (SIGTRAP)
Exception Codes:   0x0000000000000001, 0x000000012f4bde70

Termination Reason:  Namespace SIGNAL, Code 5, Trace/BPT trap: 5
Terminating Process: exc handler [583]


Thread 0 Crashed:: CrBrowserMain Dispatch queue: com.apple.main-thread
0   Chromium Embedded Framework   	       0x12f4bde70 ChromeWebAppShortcutCopierMain + 4696284
1   Chromium Embedded Framework   	       0x12f4bd8a0 ChromeWebAppShortcutCopierMain + 4694796
2   Chromium Embedded Framework   	       0x12f4bdf28 ChromeWebAppShortcutCopierMain + 4696468
3   Chromium Embedded Framework   	       0x12f4bdf40 ChromeWebAppShortcutCopierMain + 4696492
4   Chromium Embedded Framework   	       0x12d6f58c0 _v8_internal_Node_Print(void*) + 9625936
5   Chromium Embedded Framework   	       0x12d6f3084 _v8_internal_Node_Print(void*) + 9615636
6   Chromium Embedded Framework   	       0x12d6f58d4 _v8_internal_Node_Print(void*) + 9625956
7   Chromium Embedded Framework   	       0x12d6f1ef4 _v8_internal_Node_Print(void*) + 9611140
8   Chromium Embedded Framework   	       0x12d6fb50c _v8_internal_Node_Print(void*) + 9649564
9   Chromium Embedded Framework   	       0x12d6fcab0 _v8_internal_Node_Print(void*) + 9655104
10  Chromium Embedded Framework   	       0x12d523578 _v8_internal_Node_Print(void*) + 7716360
11  Chromium Embedded Framework   	       0x12d5933c4 _v8_internal_Node_Print(void*) + 8174676
12  Chromium Embedded Framework   	       0x12d594654 _v8_internal_Node_Print(void*) + 8179428
13  Chromium Embedded Framework   	       0x12d593858 _v8_internal_Node_Print(void*) + 8175848
14  Chromium Embedded Framework   	       0x12f50e500 ChromeWebAppShortcutCopierMain + 5025644
15  Chromium Embedded Framework   	       0x12f52e46c ChromeWebAppShortcutCopierMain + 5156568
16  Chromium Embedded Framework   	       0x12b09d178 temporal_rs_ZonedDateTime_offset_nanoseconds + 88712
17  Chromium Embedded Framework   	       0x12f52ecf0 ChromeWebAppShortcutCopierMain + 5158748
18  Chromium Embedded Framework   	       0x12f4f0b6c ChromeWebAppShortcutCopierMain + 4904408
19  Chromium Embedded Framework   	       0x12f4f13f0 ChromeWebAppShortcutCopierMain + 4906588
20  Chromium Embedded Framework   	       0x12b0af5f0 temporal_rs_OwnedRelativeTo_empty + 26900
21  simple                        	       0x1006a0c64 cef::bindings::aarch64_apple_darwin::do_message_loop_work::hac659492df1aabc5 + 12
22  simple                        	       0x10037949c bevy_cef::common::message_loop::cef_do_message_loop_work::h8f1ace1ee38ed07e + 12
23  simple                        	       0x1002fc788 core::ops::function::FnMut::call_mut::ha6b9f8d91b37da1c + 44
24  simple                        	       0x1003616dc core::ops::function::impls::_$LT$impl$u20$core..ops..function..FnMut$LT$A$GT$$u20$for$u20$$RF$mut$u20$F$GT$::call_mut::h5ac841d5c61e5b62 + 52
25  simple                        	       0x10038e26c _$LT$Func$u20$as$u20$bevy_ecs..system..function_system..SystemParamFunction$LT$fn$LP$F0$RP$$u20$.$GT$$u20$Out$GT$$GT$::run::call_inner::h828502a3b5a066a8 + 52
26  simple                        	       0x100361474 _$LT$Func$u20$as$u20$bevy_ecs..system..function_system..SystemParamFunction$LT$fn$LP$F0$RP$$u20$.$GT$$u20$Out$GT$$GT$::run::h895cf1dda808cdbf + 48
27  simple                        	       0x100398da4 _$LT$bevy_ecs..system..function_system..FunctionSystem$LT$Marker$C$In$C$Out$C$F$GT$$u20$as$u20$bevy_ecs..system..system..System$GT$::run_unsafe::h15f96cd8046e7dee + 468
28  simple                        	       0x1003bf03c bevy_ecs::system::system::System::run_without_applying_deferred::h84d375a4c2542f53 + 264
29  simple                        	       0x1057c6464 bevy_ecs::schedule::executor::__rust_begin_short_backtrace::run_without_applying_deferred::h28516ba104c77bb7 + 56
30  simple                        	       0x105854e88 _$LT$bevy_ecs..schedule..executor..single_threaded..SingleThreadedExecutor$u20$as$u20$bevy_ecs..schedule..executor..SystemExecutor$GT$::run::_$u7b$$u7b$closure$u7d$$u7d$::h19c5017b631ecb2c + 76
31  simple                        	       0x1057db95c core::ops::function::FnOnce::call_once::h7afd215909ce7c2b + 16
32  simple                        	       0x1057a7cd4 _$LT$core..panic..unwind_safe..AssertUnwindSafe$LT$F$GT$$u20$as$u20$core..ops..function..FnOnce$LT$$LP$$RP$$GT$$GT$::call_once::h1134e32544e3cb20 + 40
33  simple                        	       0x10570fa24 std::panicking::catch_unwind::do_call::h79a711b2ead31739 + 64
34  simple                        	       0x1056f6f80 __rust_try + 32
35  simple                        	       0x1056de37c std::panic::catch_unwind::h2c39fa031c1c494d + 76
36  simple                        	       0x105854c6c _$LT$bevy_ecs..schedule..executor..single_threaded..SingleThreadedExecutor$u20$as$u20$bevy_ecs..schedule..executor..SystemExecutor$GT$::run::he9b683e3c95704a4 + 600
37  simple                        	       0x10582d8f8 bevy_ecs::schedule::schedule::Schedule::run::he391fd6c38a13bbd + 140
38  simple                        	       0x1056608a8 bevy_ecs::world::World::run_schedule::_$u7b$$u7b$closure$u7d$$u7d$::h6260a9bb55a1efcc + 44
39  simple                        	       0x105662164 bevy_ecs::world::World::try_schedule_scope::h7d6c756fd343a72d + 256
40  simple                        	       0x105661034 bevy_ecs::world::World::schedule_scope::h2e14a4b380adb1a0 + 40
41  simple                        	       0x105660870 bevy_ecs::world::World::run_schedule::hc6d01712a92308ba + 40
42  simple                        	       0x10567d274 bevy_app::sub_app::SubApp::run_default_schedule::h9fec9e241b474cd2 + 120
43  simple                        	       0x10567df68 bevy_app::sub_app::SubApps::update::hc6b8d1d4553f5ed8 + 24
44  simple                        	       0x105675ef8 bevy_app::app::App::update::h6299a20037e0d8d3 + 44
45  simple                        	       0x10071d0e0 bevy_winit::state::WinitAppRunnerState::run_app_update::hdc490dcddf5d94a2 + 96
46  simple                        	       0x10071c8c0 bevy_winit::state::WinitAppRunnerState::redraw_requested::h1f30757d28fdc622 + 828
47  simple                        	       0x10071c474 _$LT$bevy_winit..state..WinitAppRunnerState$u20$as$u20$winit..application..ApplicationHandler$LT$bevy_winit..WinitUserEvent$GT$$GT$::about_to_wait::h7b147c6669cc89a6 + 180
48  simple                        	       0x10070a758 winit::event_loop::EventLoop$LT$T$GT$::run_app::_$u7b$$u7b$closure$u7d$$u7d$::h63f2d49927795742 + 468
49  simple                        	       0x1006def68 winit::platform_impl::macos::event_loop::map_user_event::_$u7b$$u7b$closure$u7d$$u7d$::hcf90e9861ca9b5ec + 164
50  simple                        	       0x100875ab4 _$LT$alloc..boxed..Box$LT$F$C$A$GT$$u20$as$u20$core..ops..function..FnMut$LT$Args$GT$$GT$::call_mut::h71d1270348335e61 + 84
51  simple                        	       0x10087b694 winit::platform_impl::macos::event_handler::EventHandler::handle_event::hcc459ccd21ffcd9b + 328
52  simple                        	       0x10083fc0c winit::platform_impl::macos::app_state::ApplicationDelegate::handle_event::hf4fc7de9e80a0675 + 208
53  simple                        	       0x100840438 winit::platform_impl::macos::app_state::ApplicationDelegate::cleared::h778fe018e2ab2321 + 788
54  simple                        	       0x10087c0d0 winit::platform_impl::macos::observer::control_flow_end_handler::_$u7b$$u7b$closure$u7d$$u7d$::h7478452a073f97ea + 280
55  simple                        	       0x10087bdf4 winit::platform_impl::macos::observer::control_flow_handler::_$u7b$$u7b$closure$u7d$$u7d$::h47e2c1ca3eeea93d + 44
56  simple                        	       0x100859ce4 std::panicking::catch_unwind::do_call::hcb350c5d9da49418 + 60
57  simple                        	       0x10087d6c0 __rust_try + 32
58  simple                        	       0x1008773b0 std::panic::catch_unwind::h7ed61c510a10f6e8 + 72
59  simple                        	       0x1008429bc winit::platform_impl::macos::event_loop::stop_app_on_panic::hd4ed8b53274f4d16 + 52
60  simple                        	       0x10087bbd0 winit::platform_impl::macos::observer::control_flow_handler::h9965d6221841a7ab + 320
61  simple                        	       0x10087bfa4 winit::platform_impl::macos::observer::control_flow_end_handler::hf6ef815383a0a437 + 48
62  CoreFoundation                	       0x19f6f3e48 __CFRUNLOOP_IS_CALLING_OUT_TO_AN_OBSERVER_CALLBACK_FUNCTION__ + 36
63  CoreFoundation                	       0x19f6f3d44 __CFRunLoopDoObservers + 648
64  CoreFoundation                	       0x19f6f33f0 __CFRunLoopRun + 924
65  CoreFoundation                	       0x19f7ade34 _CFRunLoopRunSpecificWithOptions + 532
66  HIToolbox                     	       0x1ac1e3790 RunCurrentEventLoopInMode + 316
67  HIToolbox                     	       0x1ac1e6ab8 ReceiveNextEventCommon + 488
68  HIToolbox                     	       0x1ac370b64 _BlockUntilNextEventMatchingListInMode + 48
69  AppKit                        	       0x1a400cb5c _DPSBlockUntilNextEventMatchingListInMode + 236
70  AppKit                        	       0x1a3b06e48 _DPSNextEvent + 588
71  AppKit                        	       0x1a45d1d0c -[NSApplication(NSEventRouting) _nextEventMatchingEventMask:untilDate:inMode:dequeue:] + 688
72  AppKit                        	       0x1a45d1a18 -[NSApplication(NSEventRouting) nextEventMatchingMask:untilDate:inMode:dequeue:] + 72
73  AppKit                        	       0x1a3aff780 -[NSApplication run] + 368
74  simple                        	       0x1008bdcf0 _$LT$$LP$$RP$$u20$as$u20$objc2..encode..EncodeArguments$GT$::__invoke::hca008099b0c64f15 + 52
75  simple                        	       0x1008b8c00 objc2::runtime::message_receiver::msg_send_primitive::send::h90591bc6187c70ad + 60
76  simple                        	       0x1008b2de4 objc2::runtime::message_receiver::MessageReceiver::send_message::h5cf96313d21e6808 + 176
77  simple                        	       0x1008aea10 objc2::__macro_helpers::msg_send::MsgSend::send_message::h3e2dcd8aaadb190c + 172
78  simple                        	       0x1008af5f0 objc2_app_kit::generated::__NSApplication::NSApplication::run::h911ae9e1b203aadb + 68
79  simple                        	       0x1006df33c winit::platform_impl::macos::event_loop::EventLoop$LT$T$GT$::run_on_demand::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::h8299637e5ea5977f + 160
80  simple                        	       0x1007bb9ec objc2::rc::autorelease::autoreleasepool::h3c114f531de2fdfd + 180
81  simple                        	       0x1006df290 winit::platform_impl::macos::event_loop::EventLoop$LT$T$GT$::run_on_demand::_$u7b$$u7b$closure$u7d$$u7d$::hc91b11dfb8018ea3 + 44
82  simple                        	       0x10070b32c winit::platform_impl::macos::event_handler::EventHandler::set::h28c81e434735e5b0 + 608
83  simple                        	       0x10073c5b4 winit::platform_impl::macos::app_state::ApplicationDelegate::set_event_handler::h847e95ec9931fdca + 152
84  simple                        	       0x1006df228 winit::platform_impl::macos::event_loop::EventLoop$LT$T$GT$::run_on_demand::h757e142636f4ded7 + 256
85  simple                        	       0x1006df7b8 winit::platform_impl::macos::event_loop::EventLoop$LT$T$GT$::run::h0a901617f86b114d + 28
86  simple                        	       0x10070a578 winit::event_loop::EventLoop$LT$T$GT$::run_app::h72e27d4a06c04dd3 + 72
87  simple                        	       0x10071dbc0 bevy_winit::state::winit_runner::h1b63cafa665a103a + 1084
88  simple                        	       0x1007a156c _$LT$bevy_winit..WinitPlugin$u20$as$u20$bevy_app..plugin..Plugin$GT$::build::_$u7b$$u7b$closure$u7d$$u7d$::hb6f259281a0b51a0 + 60
89  simple                        	       0x1006d838c core::ops::function::FnOnce::call_once$u7b$$u7b$vtable.shim$u7d$$u7d$::h7e6d9ec680d4d784 + 64
90  simple                        	       0x10565b72c _$LT$alloc..boxed..Box$LT$F$C$A$GT$$u20$as$u20$core..ops..function..FnOnce$LT$Args$GT$$GT$::call_once::ha0affe2dda881c20 + 104
91  simple                        	       0x105676068 bevy_app::app::App::run::hbdbb2ea2cde7cf9a + 320
92  simple                        	       0x1002abd8c simple::main::h3033982a4590c6c4 + 108 (simple.rs:13)
93  simple                        	       0x1002af264 core::ops::function::FnOnce::call_once::ha96ae5164d9c3d50 + 20 (function.rs:253)
94  simple                        	       0x1002a2388 std::sys::backtrace::__rust_begin_short_backtrace::h3b02af1e24de8ca8 + 24 (backtrace.rs:158)
95  simple                        	       0x1002aefcc std::rt::lang_start::_$u7b$$u7b$closure$u7d$$u7d$::h296362a2232fa8a8 + 28 (rt.rs:206)
96  simple                        	       0x105bf4c68 std::rt::lang_start_internal::hdb28e94b6865fa11 + 940
97  simple                        	       0x1002aefa4 std::rt::lang_start::hdfa26720ab6d54b6 + 84 (rt.rs:205)
98  simple                        	       0x1002ac134 main + 36
99  dyld                          	       0x19f28dd54 start + 7184

Thread 1:: IO Task Pool (0)
0   libsystem_kernel.dylib        	       0x19f6164f8 __psynch_cvwait + 8
1   libsystem_pthread.dylib       	       0x19f6560dc _pthread_cond_wait + 984
2   simple                        	       0x1058b19b0 std::sys::sync::condvar::pthread::Condvar::wait::h71a06353eafcdf14 + 184
3   simple                        	       0x1058b215c std::sync::poison::condvar::Condvar::wait::hf2c637a3647cef57 + 56
4   simple                        	       0x1058b5130 parking::Inner::park::h7e586f5f04c2b074 + 716
5   simple                        	       0x1058b4d3c parking::Parker::park::h30fbb16b0ec84050 + 40
6   simple                        	       0x10586de58 async_io::driver::block_on::_$u7b$$u7b$closure$u7d$$u7d$::hed9df0a608ddffaf + 932
7   simple                        	       0x105866940 std::thread::local::LocalKey$LT$T$GT$::try_with::h451620b445a61598 + 232
8   simple                        	       0x1058665a0 std::thread::local::LocalKey$LT$T$GT$::with::hed9c9defac547347 + 24
9   simple                        	       0x10586da14 async_io::driver::block_on::hbd24457baf9f6709 + 144
10  simple                        	       0x105863ae0 bevy_tasks::task_pool::TaskPool::new_internal::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::h7723d0b40a74f859 + 260
11  simple                        	       0x105871a48 std::panicking::catch_unwind::do_call::hd79b881c82ee687f + 68
12  simple                        	       0x105873f54 __rust_try + 32
13  simple                        	       0x1058719a4 std::panic::catch_unwind::h3f3542d315a98067 + 80
14  simple                        	       0x1058638ec bevy_tasks::task_pool::TaskPool::new_internal::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::ha1de2a24a679355d + 276
15  simple                        	       0x105866db4 std::thread::local::LocalKey$LT$T$GT$::try_with::h9b0cfac459ed4ea9 + 220
16  simple                        	       0x105866388 std::thread::local::LocalKey$LT$T$GT$::with::h22802b7d536f0b76 + 24
17  simple                        	       0x105863758 bevy_tasks::task_pool::TaskPool::new_internal::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::heaec4c69ed0fa4ba + 80
18  simple                        	       0x1058716c8 std::sys::backtrace::__rust_begin_short_backtrace::h2ff75244d19fde0d + 16
19  simple                        	       0x105860bf0 std::thread::Builder::spawn_unchecked_::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::h0fe23b32d7a6bec9 + 124
20  simple                        	       0x10586f110 _$LT$core..panic..unwind_safe..AssertUnwindSafe$LT$F$GT$$u20$as$u20$core..ops..function..FnOnce$LT$$LP$$RP$$GT$$GT$::call_once::h89dbe1df15805ed9 + 44
21  simple                        	       0x105871aa4 std::panicking::catch_unwind::do_call::hfbeac956c127a39e + 68
22  simple                        	       0x105863f24 __rust_try + 32
23  simple                        	       0x105860888 std::thread::Builder::spawn_unchecked_::_$u7b$$u7b$closure$u7d$$u7d$::hec7e514187e68944 + 768
24  simple                        	       0x105869ddc core::ops::function::FnOnce::call_once$u7b$$u7b$vtable.shim$u7d$$u7d$::h957af17fe6bc8197 + 24
25  simple                        	       0x105c01754 std::sys::pal::unix::thread::Thread::new::thread_start::h87df50f049a92661 + 52
26  libsystem_pthread.dylib       	       0x19f655c08 _pthread_start + 136
27  libsystem_pthread.dylib       	       0x19f650ba8 thread_start + 8

Thread 2:: IO Task Pool (1)
0   libsystem_kernel.dylib        	       0x19f6164f8 __psynch_cvwait + 8
1   libsystem_pthread.dylib       	       0x19f6560dc _pthread_cond_wait + 984
2   simple                        	       0x1058b19b0 std::sys::sync::condvar::pthread::Condvar::wait::h71a06353eafcdf14 + 184
3   simple                        	       0x1058b215c std::sync::poison::condvar::Condvar::wait::hf2c637a3647cef57 + 56
4   simple                        	       0x1058b5130 parking::Inner::park::h7e586f5f04c2b074 + 716
5   simple                        	       0x1058b4d3c parking::Parker::park::h30fbb16b0ec84050 + 40
6   simple                        	       0x10586de58 async_io::driver::block_on::_$u7b$$u7b$closure$u7d$$u7d$::hed9df0a608ddffaf + 932
7   simple                        	       0x105866940 std::thread::local::LocalKey$LT$T$GT$::try_with::h451620b445a61598 + 232
8   simple                        	       0x1058665a0 std::thread::local::LocalKey$LT$T$GT$::with::hed9c9defac547347 + 24
9   simple                        	       0x10586da14 async_io::driver::block_on::hbd24457baf9f6709 + 144
10  simple                        	       0x105863ae0 bevy_tasks::task_pool::TaskPool::new_internal::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::h7723d0b40a74f859 + 260
11  simple                        	       0x105871a48 std::panicking::catch_unwind::do_call::hd79b881c82ee687f + 68
12  simple                        	       0x105873f54 __rust_try + 32
13  simple                        	       0x1058719a4 std::panic::catch_unwind::h3f3542d315a98067 + 80
14  simple                        	       0x1058638ec bevy_tasks::task_pool::TaskPool::new_internal::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::ha1de2a24a679355d + 276
15  simple                        	       0x105866db4 std::thread::local::LocalKey$LT$T$GT$::try_with::h9b0cfac459ed4ea9 + 220
16  simple                        	       0x105866388 std::thread::local::LocalKey$LT$T$GT$::with::h22802b7d536f0b76 + 24
17  simple                        	       0x105863758 bevy_tasks::task_pool::TaskPool::new_internal::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::heaec4c69ed0fa4ba + 80
18  simple                        	       0x1058716c8 std::sys::backtrace::__rust_begin_short_backtrace::h2ff75244d19fde0d + 16
19  simple                        	       0x105860bf0 std::thread::Builder::spawn_unchecked_::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::h0fe23b32d7a6bec9 + 124
20  simple                        	       0x10586f110 _$LT$core..panic..unwind_safe..AssertUnwindSafe$LT$F$GT$$u20$as$u20$core..ops..function..FnOnce$LT$$LP$$RP$$GT$$GT$::call_once::h89dbe1df15805ed9 + 44
21  simple                        	       0x105871aa4 std::panicking::catch_unwind::do_call::hfbeac956c127a39e + 68
22  simple                        	       0x105863f24 __rust_try + 32
23  simple                        	       0x105860888 std::thread::Builder::spawn_unchecked_::_$u7b$$u7b$closure$u7d$$u7d$::hec7e514187e68944 + 768
24  simple                        	       0x105869ddc core::ops::function::FnOnce::call_once$u7b$$u7b$vtable.shim$u7d$$u7d$::h957af17fe6bc8197 + 24
25  simple                        	       0x105c01754 std::sys::pal::unix::thread::Thread::new::thread_start::h87df50f049a92661 + 52
26  libsystem_pthread.dylib       	       0x19f655c08 _pthread_start + 136
27  libsystem_pthread.dylib       	       0x19f650ba8 thread_start + 8

Thread 3:: IO Task Pool (2)
0   libsystem_kernel.dylib        	       0x19f6164f8 __psynch_cvwait + 8
1   libsystem_pthread.dylib       	       0x19f6560dc _pthread_cond_wait + 984
2   simple                        	       0x1058b19b0 std::sys::sync::condvar::pthread::Condvar::wait::h71a06353eafcdf14 + 184
3   simple                        	       0x1058b215c std::sync::poison::condvar::Condvar::wait::hf2c637a3647cef57 + 56
4   simple                        	       0x1058b5130 parking::Inner::park::h7e586f5f04c2b074 + 716
5   simple                        	       0x1058b4d3c parking::Parker::park::h30fbb16b0ec84050 + 40
6   simple                        	       0x10586de58 async_io::driver::block_on::_$u7b$$u7b$closure$u7d$$u7d$::hed9df0a608ddffaf + 932
7   simple                        	       0x105866940 std::thread::local::LocalKey$LT$T$GT$::try_with::h451620b445a61598 + 232
8   simple                        	       0x1058665a0 std::thread::local::LocalKey$LT$T$GT$::with::hed9c9defac547347 + 24
9   simple                        	       0x10586da14 async_io::driver::block_on::hbd24457baf9f6709 + 144
10  simple                        	       0x105863ae0 bevy_tasks::task_pool::TaskPool::new_internal::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::h7723d0b40a74f859 + 260
11  simple                        	       0x105871a48 std::panicking::catch_unwind::do_call::hd79b881c82ee687f + 68
12  simple                        	       0x105873f54 __rust_try + 32
13  simple                        	       0x1058719a4 std::panic::catch_unwind::h3f3542d315a98067 + 80
14  simple                        	       0x1058638ec bevy_tasks::task_pool::TaskPool::new_internal::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::ha1de2a24a679355d + 276
15  simple                        	       0x105866db4 std::thread::local::LocalKey$LT$T$GT$::try_with::h9b0cfac459ed4ea9 + 220
16  simple                        	       0x105866388 std::thread::local::LocalKey$LT$T$GT$::with::h22802b7d536f0b76 + 24
17  simple                        	       0x105863758 bevy_tasks::task_pool::TaskPool::new_internal::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::heaec4c69ed0fa4ba + 80
18  simple                        	       0x1058716c8 std::sys::backtrace::__rust_begin_short_backtrace::h2ff75244d19fde0d + 16
19  simple                        	       0x105860bf0 std::thread::Builder::spawn_unchecked_::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::h0fe23b32d7a6bec9 + 124
20  simple                        	       0x10586f110 _$LT$core..panic..unwind_safe..AssertUnwindSafe$LT$F$GT$$u20$as$u20$core..ops..function..FnOnce$LT$$LP$$RP$$GT$$GT$::call_once::h89dbe1df15805ed9 + 44
21  simple                        	       0x105871aa4 std::panicking::catch_unwind::do_call::hfbeac956c127a39e + 68
22  simple                        	       0x105863f24 __rust_try + 32
23  simple                        	       0x105860888 std::thread::Builder::spawn_unchecked_::_$u7b$$u7b$closure$u7d$$u7d$::hec7e514187e68944 + 768
24  simple                        	       0x105869ddc core::ops::function::FnOnce::call_once$u7b$$u7b$vtable.shim$u7d$$u7d$::h957af17fe6bc8197 + 24
25  simple                        	       0x105c01754 std::sys::pal::unix::thread::Thread::new::thread_start::h87df50f049a92661 + 52
26  libsystem_pthread.dylib       	       0x19f655c08 _pthread_start + 136
27  libsystem_pthread.dylib       	       0x19f650ba8 thread_start + 8

Thread 4:: Async Compute Task Pool (0)
0   libsystem_kernel.dylib        	       0x19f6164f8 __psynch_cvwait + 8
1   libsystem_pthread.dylib       	       0x19f6560dc _pthread_cond_wait + 984
2   simple                        	       0x1058b19b0 std::sys::sync::condvar::pthread::Condvar::wait::h71a06353eafcdf14 + 184
3   simple                        	       0x1058b215c std::sync::poison::condvar::Condvar::wait::hf2c637a3647cef57 + 56
4   simple                        	       0x1058b5130 parking::Inner::park::h7e586f5f04c2b074 + 716
5   simple                        	       0x1058b4d3c parking::Parker::park::h30fbb16b0ec84050 + 40
6   simple                        	       0x10586de58 async_io::driver::block_on::_$u7b$$u7b$closure$u7d$$u7d$::hed9df0a608ddffaf + 932
7   simple                        	       0x105866940 std::thread::local::LocalKey$LT$T$GT$::try_with::h451620b445a61598 + 232
8   simple                        	       0x1058665a0 std::thread::local::LocalKey$LT$T$GT$::with::hed9c9defac547347 + 24
9   simple                        	       0x10586da14 async_io::driver::block_on::hbd24457baf9f6709 + 144
10  simple                        	       0x105863ae0 bevy_tasks::task_pool::TaskPool::new_internal::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::h7723d0b40a74f859 + 260
11  simple                        	       0x105871a48 std::panicking::catch_unwind::do_call::hd79b881c82ee687f + 68
12  simple                        	       0x105873f54 __rust_try + 32
13  simple                        	       0x1058719a4 std::panic::catch_unwind::h3f3542d315a98067 + 80
14  simple                        	       0x1058638ec bevy_tasks::task_pool::TaskPool::new_internal::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::ha1de2a24a679355d + 276
15  simple                        	       0x105866db4 std::thread::local::LocalKey$LT$T$GT$::try_with::h9b0cfac459ed4ea9 + 220
16  simple                        	       0x105866388 std::thread::local::LocalKey$LT$T$GT$::with::h22802b7d536f0b76 + 24
17  simple                        	       0x105863758 bevy_tasks::task_pool::TaskPool::new_internal::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::heaec4c69ed0fa4ba + 80
18  simple                        	       0x1058716c8 std::sys::backtrace::__rust_begin_short_backtrace::h2ff75244d19fde0d + 16
19  simple                        	       0x105860bf0 std::thread::Builder::spawn_unchecked_::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::h0fe23b32d7a6bec9 + 124
20  simple                        	       0x10586f110 _$LT$core..panic..unwind_safe..AssertUnwindSafe$LT$F$GT$$u20$as$u20$core..ops..function..FnOnce$LT$$LP$$RP$$GT$$GT$::call_once::h89dbe1df15805ed9 + 44
21  simple                        	       0x105871aa4 std::panicking::catch_unwind::do_call::hfbeac956c127a39e + 68
22  simple                        	       0x105863f24 __rust_try + 32
23  simple                        	       0x105860888 std::thread::Builder::spawn_unchecked_::_$u7b$$u7b$closure$u7d$$u7d$::hec7e514187e68944 + 768
24  simple                        	       0x105869ddc core::ops::function::FnOnce::call_once$u7b$$u7b$vtable.shim$u7d$$u7d$::h957af17fe6bc8197 + 24
25  simple                        	       0x105c01754 std::sys::pal::unix::thread::Thread::new::thread_start::h87df50f049a92661 + 52
26  libsystem_pthread.dylib       	       0x19f655c08 _pthread_start + 136
27  libsystem_pthread.dylib       	       0x19f650ba8 thread_start + 8

Thread 5:: Async Compute Task Pool (1)
0   libsystem_kernel.dylib        	       0x19f6164f8 __psynch_cvwait + 8
1   libsystem_pthread.dylib       	       0x19f6560dc _pthread_cond_wait + 984
2   simple                        	       0x1058b19b0 std::sys::sync::condvar::pthread::Condvar::wait::h71a06353eafcdf14 + 184
3   simple                        	       0x1058b215c std::sync::poison::condvar::Condvar::wait::hf2c637a3647cef57 + 56
4   simple                        	       0x1058b5130 parking::Inner::park::h7e586f5f04c2b074 + 716
5   simple                        	       0x1058b4d3c parking::Parker::park::h30fbb16b0ec84050 + 40
6   simple                        	       0x10586de58 async_io::driver::block_on::_$u7b$$u7b$closure$u7d$$u7d$::hed9df0a608ddffaf + 932
7   simple                        	       0x105866940 std::thread::local::LocalKey$LT$T$GT$::try_with::h451620b445a61598 + 232
8   simple                        	       0x1058665a0 std::thread::local::LocalKey$LT$T$GT$::with::hed9c9defac547347 + 24
9   simple                        	       0x10586da14 async_io::driver::block_on::hbd24457baf9f6709 + 144
10  simple                        	       0x105863ae0 bevy_tasks::task_pool::TaskPool::new_internal::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::h7723d0b40a74f859 + 260
11  simple                        	       0x105871a48 std::panicking::catch_unwind::do_call::hd79b881c82ee687f + 68
12  simple                        	       0x105873f54 __rust_try + 32
13  simple                        	       0x1058719a4 std::panic::catch_unwind::h3f3542d315a98067 + 80
14  simple                        	       0x1058638ec bevy_tasks::task_pool::TaskPool::new_internal::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::ha1de2a24a679355d + 276
15  simple                        	       0x105866db4 std::thread::local::LocalKey$LT$T$GT$::try_with::h9b0cfac459ed4ea9 + 220
16  simple                        	       0x105866388 std::thread::local::LocalKey$LT$T$GT$::with::h22802b7d536f0b76 + 24
17  simple                        	       0x105863758 bevy_tasks::task_pool::TaskPool::new_internal::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::heaec4c69ed0fa4ba + 80
18  simple                        	       0x1058716c8 std::sys::backtrace::__rust_begin_short_backtrace::h2ff75244d19fde0d + 16
19  simple                        	       0x105860bf0 std::thread::Builder::spawn_unchecked_::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::h0fe23b32d7a6bec9 + 124
20  simple                        	       0x10586f110 _$LT$core..panic..unwind_safe..AssertUnwindSafe$LT$F$GT$$u20$as$u20$core..ops..function..FnOnce$LT$$LP$$RP$$GT$$GT$::call_once::h89dbe1df15805ed9 + 44
21  simple                        	       0x105871aa4 std::panicking::catch_unwind::do_call::hfbeac956c127a39e + 68
22  simple                        	       0x105863f24 __rust_try + 32
23  simple                        	       0x105860888 std::thread::Builder::spawn_unchecked_::_$u7b$$u7b$closure$u7d$$u7d$::hec7e514187e68944 + 768
24  simple                        	       0x105869ddc core::ops::function::FnOnce::call_once$u7b$$u7b$vtable.shim$u7d$$u7d$::h957af17fe6bc8197 + 24
25  simple                        	       0x105c01754 std::sys::pal::unix::thread::Thread::new::thread_start::h87df50f049a92661 + 52
26  libsystem_pthread.dylib       	       0x19f655c08 _pthread_start + 136
27  libsystem_pthread.dylib       	       0x19f650ba8 thread_start + 8

Thread 6:: Async Compute Task Pool (2)
0   libsystem_kernel.dylib        	       0x19f6164f8 __psynch_cvwait + 8
1   libsystem_pthread.dylib       	       0x19f6560dc _pthread_cond_wait + 984
2   simple                        	       0x1058b19b0 std::sys::sync::condvar::pthread::Condvar::wait::h71a06353eafcdf14 + 184
3   simple                        	       0x1058b215c std::sync::poison::condvar::Condvar::wait::hf2c637a3647cef57 + 56
4   simple                        	       0x1058b5130 parking::Inner::park::h7e586f5f04c2b074 + 716
5   simple                        	       0x1058b4d3c parking::Parker::park::h30fbb16b0ec84050 + 40
6   simple                        	       0x10586de58 async_io::driver::block_on::_$u7b$$u7b$closure$u7d$$u7d$::hed9df0a608ddffaf + 932
7   simple                        	       0x105866940 std::thread::local::LocalKey$LT$T$GT$::try_with::h451620b445a61598 + 232
8   simple                        	       0x1058665a0 std::thread::local::LocalKey$LT$T$GT$::with::hed9c9defac547347 + 24
9   simple                        	       0x10586da14 async_io::driver::block_on::hbd24457baf9f6709 + 144
10  simple                        	       0x105863ae0 bevy_tasks::task_pool::TaskPool::new_internal::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::h7723d0b40a74f859 + 260
11  simple                        	       0x105871a48 std::panicking::catch_unwind::do_call::hd79b881c82ee687f + 68
12  simple                        	       0x105873f54 __rust_try + 32
13  simple                        	       0x1058719a4 std::panic::catch_unwind::h3f3542d315a98067 + 80
14  simple                        	       0x1058638ec bevy_tasks::task_pool::TaskPool::new_internal::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::ha1de2a24a679355d + 276
15  simple                        	       0x105866db4 std::thread::local::LocalKey$LT$T$GT$::try_with::h9b0cfac459ed4ea9 + 220
16  simple                        	       0x105866388 std::thread::local::LocalKey$LT$T$GT$::with::h22802b7d536f0b76 + 24
17  simple                        	       0x105863758 bevy_tasks::task_pool::TaskPool::new_internal::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::heaec4c69ed0fa4ba + 80
18  simple                        	       0x1058716c8 std::sys::backtrace::__rust_begin_short_backtrace::h2ff75244d19fde0d + 16
19  simple                        	       0x105860bf0 std::thread::Builder::spawn_unchecked_::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::h0fe23b32d7a6bec9 + 124
20  simple                        	       0x10586f110 _$LT$core..panic..unwind_safe..AssertUnwindSafe$LT$F$GT$$u20$as$u20$core..ops..function..FnOnce$LT$$LP$$RP$$GT$$GT$::call_once::h89dbe1df15805ed9 + 44
21  simple                        	       0x105871aa4 std::panicking::catch_unwind::do_call::hfbeac956c127a39e + 68
22  simple                        	       0x105863f24 __rust_try + 32
23  simple                        	       0x105860888 std::thread::Builder::spawn_unchecked_::_$u7b$$u7b$closure$u7d$$u7d$::hec7e514187e68944 + 768
24  simple                        	       0x105869ddc core::ops::function::FnOnce::call_once$u7b$$u7b$vtable.shim$u7d$$u7d$::h957af17fe6bc8197 + 24
25  simple                        	       0x105c01754 std::sys::pal::unix::thread::Thread::new::thread_start::h87df50f049a92661 + 52
26  libsystem_pthread.dylib       	       0x19f655c08 _pthread_start + 136
27  libsystem_pthread.dylib       	       0x19f650ba8 thread_start + 8

Thread 7:: Compute Task Pool (0)
0   libsystem_kernel.dylib        	       0x19f6164f8 __psynch_cvwait + 8
1   libsystem_pthread.dylib       	       0x19f6560dc _pthread_cond_wait + 984
2   simple                        	       0x1058b19b0 std::sys::sync::condvar::pthread::Condvar::wait::h71a06353eafcdf14 + 184
3   simple                        	       0x1058b215c std::sync::poison::condvar::Condvar::wait::hf2c637a3647cef57 + 56
4   simple                        	       0x1058b5130 parking::Inner::park::h7e586f5f04c2b074 + 716
5   simple                        	       0x1058b4d3c parking::Parker::park::h30fbb16b0ec84050 + 40
6   simple                        	       0x10586de58 async_io::driver::block_on::_$u7b$$u7b$closure$u7d$$u7d$::hed9df0a608ddffaf + 932
7   simple                        	       0x105866940 std::thread::local::LocalKey$LT$T$GT$::try_with::h451620b445a61598 + 232
8   simple                        	       0x1058665a0 std::thread::local::LocalKey$LT$T$GT$::with::hed9c9defac547347 + 24
9   simple                        	       0x10586da14 async_io::driver::block_on::hbd24457baf9f6709 + 144
10  simple                        	       0x105863ae0 bevy_tasks::task_pool::TaskPool::new_internal::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::h7723d0b40a74f859 + 260
11  simple                        	       0x105871a48 std::panicking::catch_unwind::do_call::hd79b881c82ee687f + 68
12  simple                        	       0x105873f54 __rust_try + 32
13  simple                        	       0x1058719a4 std::panic::catch_unwind::h3f3542d315a98067 + 80
14  simple                        	       0x1058638ec bevy_tasks::task_pool::TaskPool::new_internal::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::ha1de2a24a679355d + 276
15  simple                        	       0x105866db4 std::thread::local::LocalKey$LT$T$GT$::try_with::h9b0cfac459ed4ea9 + 220
16  simple                        	       0x105866388 std::thread::local::LocalKey$LT$T$GT$::with::h22802b7d536f0b76 + 24
17  simple                        	       0x105863758 bevy_tasks::task_pool::TaskPool::new_internal::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::heaec4c69ed0fa4ba + 80
18  simple                        	       0x1058716c8 std::sys::backtrace::__rust_begin_short_backtrace::h2ff75244d19fde0d + 16
19  simple                        	       0x105860bf0 std::thread::Builder::spawn_unchecked_::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::h0fe23b32d7a6bec9 + 124
20  simple                        	       0x10586f110 _$LT$core..panic..unwind_safe..AssertUnwindSafe$LT$F$GT$$u20$as$u20$core..ops..function..FnOnce$LT$$LP$$RP$$GT$$GT$::call_once::h89dbe1df15805ed9 + 44
21  simple                        	       0x105871aa4 std::panicking::catch_unwind::do_call::hfbeac956c127a39e + 68
22  simple                        	       0x105863f24 __rust_try + 32
23  simple                        	       0x105860888 std::thread::Builder::spawn_unchecked_::_$u7b$$u7b$closure$u7d$$u7d$::hec7e514187e68944 + 768
24  simple                        	       0x105869ddc core::ops::function::FnOnce::call_once$u7b$$u7b$vtable.shim$u7d$$u7d$::h957af17fe6bc8197 + 24
25  simple                        	       0x105c01754 std::sys::pal::unix::thread::Thread::new::thread_start::h87df50f049a92661 + 52
26  libsystem_pthread.dylib       	       0x19f655c08 _pthread_start + 136
27  libsystem_pthread.dylib       	       0x19f650ba8 thread_start + 8

Thread 8:: async-io
0   libsystem_kernel.dylib        	       0x19f6159c8 __psynch_mutexwait + 8
1   libsystem_pthread.dylib       	       0x19f652e3c _pthread_mutex_firstfit_lock_wait + 84
2   libsystem_pthread.dylib       	       0x19f650868 _pthread_mutex_firstfit_lock_slow + 220
3   simple                        	       0x105c01418 std::sys::pal::unix::sync::mutex::Mutex::lock::h8491ee2064f70632 + 12
4   simple                        	       0x10589c2ec std::sync::poison::mutex::Mutex$LT$T$GT$::lock::hf90b0750b0d12247 + 36
5   simple                        	       0x105898738 async_io::reactor::Reactor::lock::h09a06a03efd66a1d + 36
6   simple                        	       0x105896cd8 async_io::driver::main_loop::h8864dc6a2da80f6d + 276
7   simple                        	       0x105896ba4 async_io::driver::unparker::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::h1043717272351e1a + 24
8   simple                        	       0x10589732c std::sys::backtrace::__rust_begin_short_backtrace::h70a7b233cefb4cbe + 24
9   simple                        	       0x10589ea28 std::thread::Builder::spawn_unchecked_::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::hcac0b12a7d75ad7d + 100
10  simple                        	       0x105894f48 _$LT$core..panic..unwind_safe..AssertUnwindSafe$LT$F$GT$$u20$as$u20$core..ops..function..FnOnce$LT$$LP$$RP$$GT$$GT$::call_once::h6a2c5088c2cf95da + 44
11  simple                        	       0x10589a178 std::panicking::catch_unwind::do_call::h88097bd9eab7bb79 + 68
12  simple                        	       0x10589ffa4 __rust_try + 32
13  simple                        	       0x10589e6d8 std::thread::Builder::spawn_unchecked_::_$u7b$$u7b$closure$u7d$$u7d$::hf8710d5df7914a17 + 692
14  simple                        	       0x105892628 core::ops::function::FnOnce::call_once$u7b$$u7b$vtable.shim$u7d$$u7d$::h07956b5b53d93086 + 24
15  simple                        	       0x105c01754 std::sys::pal::unix::thread::Thread::new::thread_start::h87df50f049a92661 + 52
16  libsystem_pthread.dylib       	       0x19f655c08 _pthread_start + 136
17  libsystem_pthread.dylib       	       0x19f650ba8 thread_start + 8

Thread 9:: Compute Task Pool (1)
0   libsystem_kernel.dylib        	       0x19f6164f8 __psynch_cvwait + 8
1   libsystem_pthread.dylib       	       0x19f6560dc _pthread_cond_wait + 984
2   simple                        	       0x1058b19b0 std::sys::sync::condvar::pthread::Condvar::wait::h71a06353eafcdf14 + 184
3   simple                        	       0x1058b215c std::sync::poison::condvar::Condvar::wait::hf2c637a3647cef57 + 56
4   simple                        	       0x1058b5130 parking::Inner::park::h7e586f5f04c2b074 + 716
5   simple                        	       0x1058b4d3c parking::Parker::park::h30fbb16b0ec84050 + 40
6   simple                        	       0x10586de58 async_io::driver::block_on::_$u7b$$u7b$closure$u7d$$u7d$::hed9df0a608ddffaf + 932
7   simple                        	       0x105866940 std::thread::local::LocalKey$LT$T$GT$::try_with::h451620b445a61598 + 232
8   simple                        	       0x1058665a0 std::thread::local::LocalKey$LT$T$GT$::with::hed9c9defac547347 + 24
9   simple                        	       0x10586da14 async_io::driver::block_on::hbd24457baf9f6709 + 144
10  simple                        	       0x105863ae0 bevy_tasks::task_pool::TaskPool::new_internal::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::h7723d0b40a74f859 + 260
11  simple                        	       0x105871a48 std::panicking::catch_unwind::do_call::hd79b881c82ee687f + 68
12  simple                        	       0x105873f54 __rust_try + 32
13  simple                        	       0x1058719a4 std::panic::catch_unwind::h3f3542d315a98067 + 80
14  simple                        	       0x1058638ec bevy_tasks::task_pool::TaskPool::new_internal::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::ha1de2a24a679355d + 276
15  simple                        	       0x105866db4 std::thread::local::LocalKey$LT$T$GT$::try_with::h9b0cfac459ed4ea9 + 220
16  simple                        	       0x105866388 std::thread::local::LocalKey$LT$T$GT$::with::h22802b7d536f0b76 + 24
17  simple                        	       0x105863758 bevy_tasks::task_pool::TaskPool::new_internal::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::heaec4c69ed0fa4ba + 80
18  simple                        	       0x1058716c8 std::sys::backtrace::__rust_begin_short_backtrace::h2ff75244d19fde0d + 16
19  simple                        	       0x105860bf0 std::thread::Builder::spawn_unchecked_::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::h0fe23b32d7a6bec9 + 124
20  simple                        	       0x10586f110 _$LT$core..panic..unwind_safe..AssertUnwindSafe$LT$F$GT$$u20$as$u20$core..ops..function..FnOnce$LT$$LP$$RP$$GT$$GT$::call_once::h89dbe1df15805ed9 + 44
21  simple                        	       0x105871aa4 std::panicking::catch_unwind::do_call::hfbeac956c127a39e + 68
22  simple                        	       0x105863f24 __rust_try + 32
23  simple                        	       0x105860888 std::thread::Builder::spawn_unchecked_::_$u7b$$u7b$closure$u7d$$u7d$::hec7e514187e68944 + 768
24  simple                        	       0x105869ddc core::ops::function::FnOnce::call_once$u7b$$u7b$vtable.shim$u7d$$u7d$::h957af17fe6bc8197 + 24
25  simple                        	       0x105c01754 std::sys::pal::unix::thread::Thread::new::thread_start::h87df50f049a92661 + 52
26  libsystem_pthread.dylib       	       0x19f655c08 _pthread_start + 136
27  libsystem_pthread.dylib       	       0x19f650ba8 thread_start + 8

Thread 10:: Compute Task Pool (2)
0   libsystem_kernel.dylib        	       0x19f618f30 kevent + 8
1   simple                        	       0x1058ae4c0 rustix::backend::event::syscalls::kevent::h1a287094417ceef1 + 324
2   simple                        	       0x1058a8c10 rustix::event::kqueue::kevent_timespec::h1525ff94233bcb72 + 156
3   simple                        	       0x1058a5bcc polling::kqueue::Poller::wait_deadline::ha3d7fbe46cccc0bc + 212
4   simple                        	       0x1058a4c90 polling::Poller::wait_impl::h96f408f051c897f8 + 132
5   simple                        	       0x1058a4bb4 polling::Poller::wait::h1619029d9ce0d281 + 76
6   simple                        	       0x1058990e4 async_io::reactor::ReactorLock::react::h2d5c19e2503546c2 + 548
7   simple                        	       0x10586df7c async_io::driver::block_on::_$u7b$$u7b$closure$u7d$$u7d$::hed9df0a608ddffaf + 1224
8   simple                        	       0x105866940 std::thread::local::LocalKey$LT$T$GT$::try_with::h451620b445a61598 + 232
9   simple                        	       0x1058665a0 std::thread::local::LocalKey$LT$T$GT$::with::hed9c9defac547347 + 24
10  simple                        	       0x10586da14 async_io::driver::block_on::hbd24457baf9f6709 + 144
11  simple                        	       0x105863ae0 bevy_tasks::task_pool::TaskPool::new_internal::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::h7723d0b40a74f859 + 260
12  simple                        	       0x105871a48 std::panicking::catch_unwind::do_call::hd79b881c82ee687f + 68
13  simple                        	       0x105873f54 __rust_try + 32
14  simple                        	       0x1058719a4 std::panic::catch_unwind::h3f3542d315a98067 + 80
15  simple                        	       0x1058638ec bevy_tasks::task_pool::TaskPool::new_internal::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::ha1de2a24a679355d + 276
16  simple                        	       0x105866db4 std::thread::local::LocalKey$LT$T$GT$::try_with::h9b0cfac459ed4ea9 + 220
17  simple                        	       0x105866388 std::thread::local::LocalKey$LT$T$GT$::with::h22802b7d536f0b76 + 24
18  simple                        	       0x105863758 bevy_tasks::task_pool::TaskPool::new_internal::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::heaec4c69ed0fa4ba + 80
19  simple                        	       0x1058716c8 std::sys::backtrace::__rust_begin_short_backtrace::h2ff75244d19fde0d + 16
20  simple                        	       0x105860bf0 std::thread::Builder::spawn_unchecked_::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::h0fe23b32d7a6bec9 + 124
21  simple                        	       0x10586f110 _$LT$core..panic..unwind_safe..AssertUnwindSafe$LT$F$GT$$u20$as$u20$core..ops..function..FnOnce$LT$$LP$$RP$$GT$$GT$::call_once::h89dbe1df15805ed9 + 44
22  simple                        	       0x105871aa4 std::panicking::catch_unwind::do_call::hfbeac956c127a39e + 68
23  simple                        	       0x105863f24 __rust_try + 32
24  simple                        	       0x105860888 std::thread::Builder::spawn_unchecked_::_$u7b$$u7b$closure$u7d$$u7d$::hec7e514187e68944 + 768
25  simple                        	       0x105869ddc core::ops::function::FnOnce::call_once$u7b$$u7b$vtable.shim$u7d$$u7d$::h957af17fe6bc8197 + 24
26  simple                        	       0x105c01754 std::sys::pal::unix::thread::Thread::new::thread_start::h87df50f049a92661 + 52
27  libsystem_pthread.dylib       	       0x19f655c08 _pthread_start + 136
28  libsystem_pthread.dylib       	       0x19f650ba8 thread_start + 8

Thread 11:: Compute Task Pool (3)
0   libsystem_kernel.dylib        	       0x19f6164f8 __psynch_cvwait + 8
1   libsystem_pthread.dylib       	       0x19f6560dc _pthread_cond_wait + 984
2   simple                        	       0x1058b19b0 std::sys::sync::condvar::pthread::Condvar::wait::h71a06353eafcdf14 + 184
3   simple                        	       0x1058b215c std::sync::poison::condvar::Condvar::wait::hf2c637a3647cef57 + 56
4   simple                        	       0x1058b5130 parking::Inner::park::h7e586f5f04c2b074 + 716
5   simple                        	       0x1058b4d3c parking::Parker::park::h30fbb16b0ec84050 + 40
6   simple                        	       0x10586de58 async_io::driver::block_on::_$u7b$$u7b$closure$u7d$$u7d$::hed9df0a608ddffaf + 932
7   simple                        	       0x105866940 std::thread::local::LocalKey$LT$T$GT$::try_with::h451620b445a61598 + 232
8   simple                        	       0x1058665a0 std::thread::local::LocalKey$LT$T$GT$::with::hed9c9defac547347 + 24
9   simple                        	       0x10586da14 async_io::driver::block_on::hbd24457baf9f6709 + 144
10  simple                        	       0x105863ae0 bevy_tasks::task_pool::TaskPool::new_internal::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::h7723d0b40a74f859 + 260
11  simple                        	       0x105871a48 std::panicking::catch_unwind::do_call::hd79b881c82ee687f + 68
12  simple                        	       0x105873f54 __rust_try + 32
13  simple                        	       0x1058719a4 std::panic::catch_unwind::h3f3542d315a98067 + 80
14  simple                        	       0x1058638ec bevy_tasks::task_pool::TaskPool::new_internal::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::ha1de2a24a679355d + 276
15  simple                        	       0x105866db4 std::thread::local::LocalKey$LT$T$GT$::try_with::h9b0cfac459ed4ea9 + 220
16  simple                        	       0x105866388 std::thread::local::LocalKey$LT$T$GT$::with::h22802b7d536f0b76 + 24
17  simple                        	       0x105863758 bevy_tasks::task_pool::TaskPool::new_internal::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::heaec4c69ed0fa4ba + 80
18  simple                        	       0x1058716c8 std::sys::backtrace::__rust_begin_short_backtrace::h2ff75244d19fde0d + 16
19  simple                        	       0x105860bf0 std::thread::Builder::spawn_unchecked_::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::h0fe23b32d7a6bec9 + 124
20  simple                        	       0x10586f110 _$LT$core..panic..unwind_safe..AssertUnwindSafe$LT$F$GT$$u20$as$u20$core..ops..function..FnOnce$LT$$LP$$RP$$GT$$GT$::call_once::h89dbe1df15805ed9 + 44
21  simple                        	       0x105871aa4 std::panicking::catch_unwind::do_call::hfbeac956c127a39e + 68
22  simple                        	       0x105863f24 __rust_try + 32
23  simple                        	       0x105860888 std::thread::Builder::spawn_unchecked_::_$u7b$$u7b$closure$u7d$$u7d$::hec7e514187e68944 + 768
24  simple                        	       0x105869ddc core::ops::function::FnOnce::call_once$u7b$$u7b$vtable.shim$u7d$$u7d$::h957af17fe6bc8197 + 24
25  simple                        	       0x105c01754 std::sys::pal::unix::thread::Thread::new::thread_start::h87df50f049a92661 + 52
26  libsystem_pthread.dylib       	       0x19f655c08 _pthread_start + 136
27  libsystem_pthread.dylib       	       0x19f650ba8 thread_start + 8

Thread 12:: Compute Task Pool (4)
0   libsystem_kernel.dylib        	       0x19f6164f8 __psynch_cvwait + 8
1   libsystem_pthread.dylib       	       0x19f6560dc _pthread_cond_wait + 984
2   simple                        	       0x1058b19b0 std::sys::sync::condvar::pthread::Condvar::wait::h71a06353eafcdf14 + 184
3   simple                        	       0x1058b215c std::sync::poison::condvar::Condvar::wait::hf2c637a3647cef57 + 56
4   simple                        	       0x1058b5130 parking::Inner::park::h7e586f5f04c2b074 + 716
5   simple                        	       0x1058b4d3c parking::Parker::park::h30fbb16b0ec84050 + 40
6   simple                        	       0x10586de58 async_io::driver::block_on::_$u7b$$u7b$closure$u7d$$u7d$::hed9df0a608ddffaf + 932
7   simple                        	       0x105866940 std::thread::local::LocalKey$LT$T$GT$::try_with::h451620b445a61598 + 232
8   simple                        	       0x1058665a0 std::thread::local::LocalKey$LT$T$GT$::with::hed9c9defac547347 + 24
9   simple                        	       0x10586da14 async_io::driver::block_on::hbd24457baf9f6709 + 144
10  simple                        	       0x105863ae0 bevy_tasks::task_pool::TaskPool::new_internal::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::h7723d0b40a74f859 + 260
11  simple                        	       0x105871a48 std::panicking::catch_unwind::do_call::hd79b881c82ee687f + 68
12  simple                        	       0x105873f54 __rust_try + 32
13  simple                        	       0x1058719a4 std::panic::catch_unwind::h3f3542d315a98067 + 80
14  simple                        	       0x1058638ec bevy_tasks::task_pool::TaskPool::new_internal::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::ha1de2a24a679355d + 276
15  simple                        	       0x105866db4 std::thread::local::LocalKey$LT$T$GT$::try_with::h9b0cfac459ed4ea9 + 220
16  simple                        	       0x105866388 std::thread::local::LocalKey$LT$T$GT$::with::h22802b7d536f0b76 + 24
17  simple                        	       0x105863758 bevy_tasks::task_pool::TaskPool::new_internal::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::heaec4c69ed0fa4ba + 80
18  simple                        	       0x1058716c8 std::sys::backtrace::__rust_begin_short_backtrace::h2ff75244d19fde0d + 16
19  simple                        	       0x105860bf0 std::thread::Builder::spawn_unchecked_::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::h0fe23b32d7a6bec9 + 124
20  simple                        	       0x10586f110 _$LT$core..panic..unwind_safe..AssertUnwindSafe$LT$F$GT$$u20$as$u20$core..ops..function..FnOnce$LT$$LP$$RP$$GT$$GT$::call_once::h89dbe1df15805ed9 + 44
21  simple                        	       0x105871aa4 std::panicking::catch_unwind::do_call::hfbeac956c127a39e + 68
22  simple                        	       0x105863f24 __rust_try + 32
23  simple                        	       0x105860888 std::thread::Builder::spawn_unchecked_::_$u7b$$u7b$closure$u7d$$u7d$::hec7e514187e68944 + 768
24  simple                        	       0x105869ddc core::ops::function::FnOnce::call_once$u7b$$u7b$vtable.shim$u7d$$u7d$::h957af17fe6bc8197 + 24
25  simple                        	       0x105c01754 std::sys::pal::unix::thread::Thread::new::thread_start::h87df50f049a92661 + 52
26  libsystem_pthread.dylib       	       0x19f655c08 _pthread_start + 136
27  libsystem_pthread.dylib       	       0x19f650ba8 thread_start + 8

Thread 13:: Compute Task Pool (5)
0   libsystem_kernel.dylib        	       0x19f6164f8 __psynch_cvwait + 8
1   libsystem_pthread.dylib       	       0x19f6560dc _pthread_cond_wait + 984
2   simple                        	       0x1058b19b0 std::sys::sync::condvar::pthread::Condvar::wait::h71a06353eafcdf14 + 184
3   simple                        	       0x1058b215c std::sync::poison::condvar::Condvar::wait::hf2c637a3647cef57 + 56
4   simple                        	       0x1058b5130 parking::Inner::park::h7e586f5f04c2b074 + 716
5   simple                        	       0x1058b4d3c parking::Parker::park::h30fbb16b0ec84050 + 40
6   simple                        	       0x10586de58 async_io::driver::block_on::_$u7b$$u7b$closure$u7d$$u7d$::hed9df0a608ddffaf + 932
7   simple                        	       0x105866940 std::thread::local::LocalKey$LT$T$GT$::try_with::h451620b445a61598 + 232
8   simple                        	       0x1058665a0 std::thread::local::LocalKey$LT$T$GT$::with::hed9c9defac547347 + 24
9   simple                        	       0x10586da14 async_io::driver::block_on::hbd24457baf9f6709 + 144
10  simple                        	       0x105863ae0 bevy_tasks::task_pool::TaskPool::new_internal::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::h7723d0b40a74f859 + 260
11  simple                        	       0x105871a48 std::panicking::catch_unwind::do_call::hd79b881c82ee687f + 68
12  simple                        	       0x105873f54 __rust_try + 32
13  simple                        	       0x1058719a4 std::panic::catch_unwind::h3f3542d315a98067 + 80
14  simple                        	       0x1058638ec bevy_tasks::task_pool::TaskPool::new_internal::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::ha1de2a24a679355d + 276
15  simple                        	       0x105866db4 std::thread::local::LocalKey$LT$T$GT$::try_with::h9b0cfac459ed4ea9 + 220
16  simple                        	       0x105866388 std::thread::local::LocalKey$LT$T$GT$::with::h22802b7d536f0b76 + 24
17  simple                        	       0x105863758 bevy_tasks::task_pool::TaskPool::new_internal::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::heaec4c69ed0fa4ba + 80
18  simple                        	       0x1058716c8 std::sys::backtrace::__rust_begin_short_backtrace::h2ff75244d19fde0d + 16
19  simple                        	       0x105860bf0 std::thread::Builder::spawn_unchecked_::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::h0fe23b32d7a6bec9 + 124
20  simple                        	       0x10586f110 _$LT$core..panic..unwind_safe..AssertUnwindSafe$LT$F$GT$$u20$as$u20$core..ops..function..FnOnce$LT$$LP$$RP$$GT$$GT$::call_once::h89dbe1df15805ed9 + 44
21  simple                        	       0x105871aa4 std::panicking::catch_unwind::do_call::hfbeac956c127a39e + 68
22  simple                        	       0x105863f24 __rust_try + 32
23  simple                        	       0x105860888 std::thread::Builder::spawn_unchecked_::_$u7b$$u7b$closure$u7d$$u7d$::hec7e514187e68944 + 768
24  simple                        	       0x105869ddc core::ops::function::FnOnce::call_once$u7b$$u7b$vtable.shim$u7d$$u7d$::h957af17fe6bc8197 + 24
25  simple                        	       0x105c01754 std::sys::pal::unix::thread::Thread::new::thread_start::h87df50f049a92661 + 52
26  libsystem_pthread.dylib       	       0x19f655c08 _pthread_start + 136
27  libsystem_pthread.dylib       	       0x19f650ba8 thread_start + 8

Thread 14:: ctrl-c
0   libsystem_kernel.dylib        	       0x19f612bb0 semaphore_wait_trap + 8
1   libdispatch.dylib             	       0x19f49a990 _dispatch_sema4_wait + 28
2   libdispatch.dylib             	       0x19f49af40 _dispatch_semaphore_wait_slow + 132
3   simple                        	       0x10567f4fc ctrlc::platform::unix::implementation::sem_wait_forever::h57686728d3c0c92a + 48
4   simple                        	       0x10565f950 ctrlc::platform::unix::block_ctrl_c::hb803e6b46ec3dfa5 + 20
5   simple                        	       0x105666c2c ctrlc::set_handler_inner::_$u7b$$u7b$closure$u7d$$u7d$::h39ca9869d5417c95 + 24
6   simple                        	       0x1056736d4 std::sys::backtrace::__rust_begin_short_backtrace::h8a8cc888cd60e6ef + 16
7   simple                        	       0x1056594a4 std::thread::Builder::spawn_unchecked_::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::hd6e9bbddf5667bde + 88
8   simple                        	       0x10566471c _$LT$core..panic..unwind_safe..AssertUnwindSafe$LT$F$GT$$u20$as$u20$core..ops..function..FnOnce$LT$$LP$$RP$$GT$$GT$::call_once::h5123a29a249bed10 + 40
9   simple                        	       0x10565ec10 std::panicking::catch_unwind::do_call::h99551042f19c334b + 64
10  simple                        	       0x10565d50c __rust_try + 32
11  simple                        	       0x105659160 std::thread::Builder::spawn_unchecked_::_$u7b$$u7b$closure$u7d$$u7d$::h6e114e23ce529a23 + 644
12  simple                        	       0x10565312c core::ops::function::FnOnce::call_once$u7b$$u7b$vtable.shim$u7d$$u7d$::hb767d9bb8531d226 + 24
13  simple                        	       0x105c01754 std::sys::pal::unix::thread::Thread::new::thread_start::h87df50f049a92661 + 52
14  libsystem_pthread.dylib       	       0x19f655c08 _pthread_start + 136
15  libsystem_pthread.dylib       	       0x19f650ba8 thread_start + 8

Thread 15:: notify-rs debouncer loop
0   libsystem_kernel.dylib        	       0x19f6162f4 __semwait_signal + 8
1   libsystem_c.dylib             	       0x19f4eed6c nanosleep + 220
2   simple                        	       0x105bf57c0 std::thread::sleep::h17057c3b27540418 + 84
3   simple                        	       0x104d57950 notify_debouncer_full::new_debouncer_opt::_$u7b$$u7b$closure$u7d$$u7d$::hb1ac894d1b229b11 + 128
4   simple                        	       0x104da1bb8 std::sys::backtrace::__rust_begin_short_backtrace::hf85bb894ed39128e + 16
5   simple                        	       0x104d79d70 std::thread::Builder::spawn_unchecked_::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::ha5b1c662df64ae7e + 124
6   simple                        	       0x104d9efc4 _$LT$core..panic..unwind_safe..AssertUnwindSafe$LT$F$GT$$u20$as$u20$core..ops..function..FnOnce$LT$$LP$$RP$$GT$$GT$::call_once::h0d23523e05f3bc65 + 44
7   simple                        	       0x104e964d8 std::panicking::catch_unwind::do_call::hd259eb5268343aae + 68
8   simple                        	       0x104d9e250 __rust_try + 32
9   simple                        	       0x104d79a08 std::thread::Builder::spawn_unchecked_::_$u7b$$u7b$closure$u7d$$u7d$::h971eb8b4be5c1343 + 768
10  simple                        	       0x104d1c000 core::ops::function::FnOnce::call_once$u7b$$u7b$vtable.shim$u7d$$u7d$::hc21e4375ec08643b + 24
11  simple                        	       0x105c01754 std::sys::pal::unix::thread::Thread::new::thread_start::h87df50f049a92661 + 52
12  libsystem_pthread.dylib       	       0x19f655c08 _pthread_start + 136
13  libsystem_pthread.dylib       	       0x19f650ba8 thread_start + 8

Thread 16::  Dispatch queue: com.apple.root.user-interactive-qos
0   libsystem_kernel.dylib        	       0x19f612c34 mach_msg2_trap + 8
1   libsystem_kernel.dylib        	       0x19f625028 mach_msg2_internal + 76
2   libsystem_kernel.dylib        	       0x19f61b98c mach_msg_overwrite + 484
3   libsystem_kernel.dylib        	       0x19f612fb4 mach_msg + 24
4   CoreFoundation                	       0x19f6f4ba0 __CFRunLoopServiceMachPort + 160
5   CoreFoundation                	       0x19f6f34f8 __CFRunLoopRun + 1188
6   CoreFoundation                	       0x19f7ade34 _CFRunLoopRunSpecificWithOptions + 532
7   Foundation                    	       0x1a1942964 -[NSRunLoop(NSRunLoop) runMode:beforeDate:] + 212
8   AppKit                        	       0x1a4057bfc -[NSAnimation _runBlocking] + 412
9   libdispatch.dylib             	       0x19f498b5c _dispatch_call_block_and_release + 32
10  libdispatch.dylib             	       0x19f4b2ad4 _dispatch_client_callout + 16
11  libdispatch.dylib             	       0x19f4cf9dc <deduplicated_symbol> + 32
12  libdispatch.dylib             	       0x19f4ab13c _dispatch_root_queue_drain + 736
13  libdispatch.dylib             	       0x19f4ab784 _dispatch_worker_thread2 + 180
14  libsystem_pthread.dylib       	       0x19f651e10 _pthread_wqthread + 232
15  libsystem_pthread.dylib       	       0x19f650b9c start_wqthread + 8

Thread 17:: notify-rs fsevents loop
0   libsystem_kernel.dylib        	       0x19f612c34 mach_msg2_trap + 8
1   libsystem_kernel.dylib        	       0x19f625028 mach_msg2_internal + 76
2   libsystem_kernel.dylib        	       0x19f61b98c mach_msg_overwrite + 484
3   libsystem_kernel.dylib        	       0x19f612fb4 mach_msg + 24
4   CoreFoundation                	       0x19f6f4ba0 __CFRunLoopServiceMachPort + 160
5   CoreFoundation                	       0x19f6f34f8 __CFRunLoopRun + 1188
6   CoreFoundation                	       0x19f7ade34 _CFRunLoopRunSpecificWithOptions + 532
7   CoreFoundation                	       0x19f746a40 CFRunLoopRun + 64
8   simple                        	       0x104ffca7c notify::fsevent::FsEventWatcher::run::_$u7b$$u7b$closure$u7d$$u7d$::h115ce633794dd37e + 212
9   simple                        	       0x104ffa110 std::sys::backtrace::__rust_begin_short_backtrace::h0e8136c158efde34 + 16
10  simple                        	       0x104ff1e1c std::thread::Builder::spawn_unchecked_::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::had9e234c58a6b976 + 116
11  simple                        	       0x104ffe7b8 _$LT$core..panic..unwind_safe..AssertUnwindSafe$LT$F$GT$$u20$as$u20$core..ops..function..FnOnce$LT$$LP$$RP$$GT$$GT$::call_once::hd4eb61dc2f7423a1 + 44
12  simple                        	       0x1050021a0 std::panicking::catch_unwind::do_call::hb3c1c61f12e1b465 + 68
13  simple                        	       0x104ff2f98 __rust_try + 32
14  simple                        	       0x104ff1c60 std::thread::Builder::spawn_unchecked_::_$u7b$$u7b$closure$u7d$$u7d$::hd5a9da3262eb8561 + 728
15  simple                        	       0x105002314 core::ops::function::FnOnce::call_once$u7b$$u7b$vtable.shim$u7d$$u7d$::hf49d8a5ef3bd1f1a + 24
16  simple                        	       0x105c01754 std::sys::pal::unix::thread::Thread::new::thread_start::h87df50f049a92661 + 52
17  libsystem_pthread.dylib       	       0x19f655c08 _pthread_start + 136
18  libsystem_pthread.dylib       	       0x19f650ba8 thread_start + 8

Thread 18:

Thread 19:

Thread 20:

Thread 21:: com.apple.NSEventThread
0   libsystem_kernel.dylib        	       0x19f612c34 mach_msg2_trap + 8
1   libsystem_kernel.dylib        	       0x19f625028 mach_msg2_internal + 76
2   libsystem_kernel.dylib        	       0x19f61b98c mach_msg_overwrite + 484
3   libsystem_kernel.dylib        	       0x19f612fb4 mach_msg + 24
4   CoreFoundation                	       0x19f6f4ba0 __CFRunLoopServiceMachPort + 160
5   CoreFoundation                	       0x19f6f34f8 __CFRunLoopRun + 1188
6   CoreFoundation                	       0x19f7ade34 _CFRunLoopRunSpecificWithOptions + 532
7   AppKit                        	       0x1a3b96a34 _NSEventThread + 184
8   libsystem_pthread.dylib       	       0x19f655c08 _pthread_start + 136
9   libsystem_pthread.dylib       	       0x19f650ba8 thread_start + 8

Thread 22:: caulk.messenger.shared:17
0   libsystem_kernel.dylib        	       0x19f612bb0 semaphore_wait_trap + 8
1   caulk                         	       0x1abc6be08 caulk::semaphore::timed_wait(double) + 224
2   caulk                         	       0x1abc6bcb0 caulk::concurrent::details::worker_thread::run() + 32
3   caulk                         	       0x1abc6b950 void* caulk::thread_proxy<std::__1::tuple<caulk::thread::attributes, void (caulk::concurrent::details::worker_thread::*)(), std::__1::tuple<caulk::concurrent::details::worker_thread*>>>(void*) + 96
4   libsystem_pthread.dylib       	       0x19f655c08 _pthread_start + 136
5   libsystem_pthread.dylib       	       0x19f650ba8 thread_start + 8

Thread 23:: caulk.messenger.shared:high
0   libsystem_kernel.dylib        	       0x19f612bb0 semaphore_wait_trap + 8
1   caulk                         	       0x1abc6be08 caulk::semaphore::timed_wait(double) + 224
2   caulk                         	       0x1abc6bcb0 caulk::concurrent::details::worker_thread::run() + 32
3   caulk                         	       0x1abc6b950 void* caulk::thread_proxy<std::__1::tuple<caulk::thread::attributes, void (caulk::concurrent::details::worker_thread::*)(), std::__1::tuple<caulk::concurrent::details::worker_thread*>>>(void*) + 96
4   libsystem_pthread.dylib       	       0x19f655c08 _pthread_start + 136
5   libsystem_pthread.dylib       	       0x19f650ba8 thread_start + 8

Thread 24:: caulk::deferred_logger
0   libsystem_kernel.dylib        	       0x19f612bb0 semaphore_wait_trap + 8
1   caulk                         	       0x1abc6be08 caulk::semaphore::timed_wait(double) + 224
2   caulk                         	       0x1abc6bcb0 caulk::concurrent::details::worker_thread::run() + 32
3   caulk                         	       0x1abc6b950 void* caulk::thread_proxy<std::__1::tuple<caulk::thread::attributes, void (caulk::concurrent::details::worker_thread::*)(), std::__1::tuple<caulk::concurrent::details::worker_thread*>>>(void*) + 96
4   libsystem_pthread.dylib       	       0x19f655c08 _pthread_start + 136
5   libsystem_pthread.dylib       	       0x19f650ba8 thread_start + 8

Thread 25:: AudioSession - RootQueue
0   libsystem_kernel.dylib        	       0x19f612bc8 semaphore_timedwait_trap + 8
1   libdispatch.dylib             	       0x19f4cdc8c _dispatch_sema4_timedwait + 64
2   libdispatch.dylib             	       0x19f49af08 _dispatch_semaphore_wait_slow + 76
3   libdispatch.dylib             	       0x19f4aadc0 _dispatch_worker_thread + 324
4   libsystem_pthread.dylib       	       0x19f655c08 _pthread_start + 136
5   libsystem_pthread.dylib       	       0x19f650ba8 thread_start + 8

Thread 26:: com.apple.audio.IOThread.client
0   libsystem_kernel.dylib        	       0x19f612bbc semaphore_wait_signal_trap + 8
1   caulk                         	       0x1abc88fac caulk::mach::semaphore::wait_signal_or_error(caulk::mach::semaphore&) + 36
2   CoreAudio                     	       0x1a296b6f0 HALC_ProxyIOContext::IOWorkLoop() + 5052
3   CoreAudio                     	       0x1a2969c8c invocation function for block in HALC_ProxyIOContext::HALC_ProxyIOContext(unsigned int, unsigned int) + 172
4   CoreAudio                     	       0x1a2b37710 HALC_IOThread::Entry(void*) + 88
5   libsystem_pthread.dylib       	       0x19f655c08 _pthread_start + 136
6   libsystem_pthread.dylib       	       0x19f650ba8 thread_start + 8

Thread 27:: gilrs
0   libsystem_kernel.dylib        	       0x19f612c34 mach_msg2_trap + 8
1   libsystem_kernel.dylib        	       0x19f625028 mach_msg2_internal + 76
2   libsystem_kernel.dylib        	       0x19f61b98c mach_msg_overwrite + 484
3   libsystem_kernel.dylib        	       0x19f612fb4 mach_msg + 24
4   CoreFoundation                	       0x19f6f4ba0 __CFRunLoopServiceMachPort + 160
5   CoreFoundation                	       0x19f6f34f8 __CFRunLoopRun + 1188
6   CoreFoundation                	       0x19f7ade34 _CFRunLoopRunSpecificWithOptions + 532
7   CoreFoundation                	       0x19f746a40 CFRunLoopRun + 64
8   simple                        	       0x1041cfaa8 core_foundation::runloop::CFRunLoop::run_current::hff31c1552c5c8f29 + 12
9   simple                        	       0x102e74018 gilrs_core::platform::platform::gamepad::Gilrs::spawn_thread::_$u7b$$u7b$closure$u7d$$u7d$::h817fe5c103cb72e3 + 988
10  simple                        	       0x102e5d25c std::sys::backtrace::__rust_begin_short_backtrace::h60c71449010a58d7 + 16
11  simple                        	       0x102e82e48 std::thread::Builder::spawn_unchecked_::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::h1d393ecf4a8e3432 + 116
12  simple                        	       0x102e62f1c _$LT$core..panic..unwind_safe..AssertUnwindSafe$LT$F$GT$$u20$as$u20$core..ops..function..FnOnce$LT$$LP$$RP$$GT$$GT$::call_once::hed43e0acd19cfce2 + 44
13  simple                        	       0x102e82f7c std::panicking::catch_unwind::do_call::h7982dc619b31e30c + 68
14  simple                        	       0x102e84fa4 __rust_try + 32
15  simple                        	       0x102e8299c std::thread::Builder::spawn_unchecked_::_$u7b$$u7b$closure$u7d$$u7d$::hea9e1044530aa965 + 728
16  simple                        	       0x102e6a9bc core::ops::function::FnOnce::call_once$u7b$$u7b$vtable.shim$u7d$$u7d$::h62f8830b16be48a7 + 24
17  simple                        	       0x105c01754 std::sys::pal::unix::thread::Thread::new::thread_start::h87df50f049a92661 + 52
18  libsystem_pthread.dylib       	       0x19f655c08 _pthread_start + 136
19  libsystem_pthread.dylib       	       0x19f650ba8 thread_start + 8

Thread 28:: gilrs
0   libsystem_kernel.dylib        	       0x19f6162f4 __semwait_signal + 8
1   libsystem_c.dylib             	       0x19f4eed6c nanosleep + 220
2   simple                        	       0x105bf57c0 std::thread::sleep::h17057c3b27540418 + 84
3   simple                        	       0x102e539fc gilrs::ff::server::run::h1f3ab47ccb1a3edd + 6888
4   simple                        	       0x102e53dfc gilrs::ff::server::init::_$u7b$$u7b$closure$u7d$$u7d$::hafc8ef086dd20fa5 + 32
5   simple                        	       0x102e3cc8c std::sys::backtrace::__rust_begin_short_backtrace::hb5b70ca1bdbfefbd + 16
6   simple                        	       0x102e375d8 std::thread::Builder::spawn_unchecked_::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::h3434a3cc753a8765 + 116
7   simple                        	       0x102e2eb64 _$LT$core..panic..unwind_safe..AssertUnwindSafe$LT$F$GT$$u20$as$u20$core..ops..function..FnOnce$LT$$LP$$RP$$GT$$GT$::call_once::h65cde94902cf8373 + 44
8   simple                        	       0x102e43014 std::panicking::catch_unwind::do_call::h72fadfd76ed15e46 + 68
9   simple                        	       0x102e39264 __rust_try + 32
10  simple                        	       0x102e3741c std::thread::Builder::spawn_unchecked_::_$u7b$$u7b$closure$u7d$$u7d$::h997c49fc8d6a6212 + 728
11  simple                        	       0x102e48514 core::ops::function::FnOnce::call_once$u7b$$u7b$vtable.shim$u7d$$u7d$::h9bc8e158fa7b696b + 24
12  simple                        	       0x105c01754 std::sys::pal::unix::thread::Thread::new::thread_start::h87df50f049a92661 + 52
13  libsystem_pthread.dylib       	       0x19f655c08 _pthread_start + 136
14  libsystem_pthread.dylib       	       0x19f650ba8 thread_start + 8

Thread 29:: StackSamplingProfiler
0   libsystem_kernel.dylib        	       0x19f612c34 mach_msg2_trap + 8
1   libsystem_kernel.dylib        	       0x19f625028 mach_msg2_internal + 76
2   libsystem_kernel.dylib        	       0x19f61b98c mach_msg_overwrite + 484
3   libsystem_kernel.dylib        	       0x19f612fb4 mach_msg + 24
4   Chromium Embedded Framework   	       0x12f5807f8 ChromeWebAppShortcutCopierMain + 5493348
5   Chromium Embedded Framework   	       0x12f50b608 ChromeWebAppShortcutCopierMain + 5013620
6   Chromium Embedded Framework   	       0x12f4c49c0 ChromeWebAppShortcutCopierMain + 4723756
7   Chromium Embedded Framework   	       0x12f52ecf0 ChromeWebAppShortcutCopierMain + 5158748
8   Chromium Embedded Framework   	       0x12f4f0b6c ChromeWebAppShortcutCopierMain + 4904408
9   Chromium Embedded Framework   	       0x12f55053c ChromeWebAppShortcutCopierMain + 5296040
10  Chromium Embedded Framework   	       0x12f5506c8 ChromeWebAppShortcutCopierMain + 5296436
11  Chromium Embedded Framework   	       0x12f5646b0 ChromeWebAppShortcutCopierMain + 5378332
12  libsystem_pthread.dylib       	       0x19f655c08 _pthread_start + 136
13  libsystem_pthread.dylib       	       0x19f650ba8 thread_start + 8

Thread 30:: HangWatcher
0   libsystem_kernel.dylib        	       0x19f612c34 mach_msg2_trap + 8
1   libsystem_kernel.dylib        	       0x19f625028 mach_msg2_internal + 76
2   libsystem_kernel.dylib        	       0x19f61b98c mach_msg_overwrite + 484
3   libsystem_kernel.dylib        	       0x19f612fb4 mach_msg + 24
4   Chromium Embedded Framework   	       0x12f5807f8 ChromeWebAppShortcutCopierMain + 5493348
5   Chromium Embedded Framework   	       0x12f50b608 ChromeWebAppShortcutCopierMain + 5013620
6   Chromium Embedded Framework   	       0x12f54a304 ChromeWebAppShortcutCopierMain + 5270896
7   Chromium Embedded Framework   	       0x12f54a3f0 ChromeWebAppShortcutCopierMain + 5271132
8   Chromium Embedded Framework   	       0x12f5646b0 ChromeWebAppShortcutCopierMain + 5378332
9   libsystem_pthread.dylib       	       0x19f655c08 _pthread_start + 136
10  libsystem_pthread.dylib       	       0x19f650ba8 thread_start + 8

Thread 31:: PerfettoTrace
0   libsystem_kernel.dylib        	       0x19f61eb84 kevent64 + 8
1   Chromium Embedded Framework   	       0x12f58c5c8 ChromeWebAppShortcutCopierMain + 5541940
2   Chromium Embedded Framework   	       0x12f52ecf0 ChromeWebAppShortcutCopierMain + 5158748
3   Chromium Embedded Framework   	       0x12f4f0b6c ChromeWebAppShortcutCopierMain + 4904408
4   Chromium Embedded Framework   	       0x12f55053c ChromeWebAppShortcutCopierMain + 5296040
5   Chromium Embedded Framework   	       0x12f5506c8 ChromeWebAppShortcutCopierMain + 5296436
6   Chromium Embedded Framework   	       0x12f5646b0 ChromeWebAppShortcutCopierMain + 5378332
7   libsystem_pthread.dylib       	       0x19f655c08 _pthread_start + 136
8   libsystem_pthread.dylib       	       0x19f650ba8 thread_start + 8

Thread 32:: ThreadPoolServiceThread
0   libsystem_kernel.dylib        	       0x19f61eb84 kevent64 + 8
1   Chromium Embedded Framework   	       0x12f58c5c8 ChromeWebAppShortcutCopierMain + 5541940
2   Chromium Embedded Framework   	       0x12f52ecf0 ChromeWebAppShortcutCopierMain + 5158748
3   Chromium Embedded Framework   	       0x12f4f0b6c ChromeWebAppShortcutCopierMain + 4904408
4   Chromium Embedded Framework   	       0x12f55053c ChromeWebAppShortcutCopierMain + 5296040
5   Chromium Embedded Framework   	       0x12f538664 ChromeWebAppShortcutCopierMain + 5198032
6   Chromium Embedded Framework   	       0x12f5506c8 ChromeWebAppShortcutCopierMain + 5296436
7   Chromium Embedded Framework   	       0x12f5646b0 ChromeWebAppShortcutCopierMain + 5378332
8   libsystem_pthread.dylib       	       0x19f655c08 _pthread_start + 136
9   libsystem_pthread.dylib       	       0x19f650ba8 thread_start + 8

Thread 33:: ThreadPoolForegroundWorker
0   libsystem_kernel.dylib        	       0x19f612c34 mach_msg2_trap + 8
1   libsystem_kernel.dylib        	       0x19f625028 mach_msg2_internal + 76
2   libsystem_kernel.dylib        	       0x19f61b98c mach_msg_overwrite + 484
3   libsystem_kernel.dylib        	       0x19f612fb4 mach_msg + 24
4   Chromium Embedded Framework   	       0x12f5807f8 ChromeWebAppShortcutCopierMain + 5493348
5   Chromium Embedded Framework   	       0x12f50b608 ChromeWebAppShortcutCopierMain + 5013620
6   Chromium Embedded Framework   	       0x12f548718 ChromeWebAppShortcutCopierMain + 5263748
7   Chromium Embedded Framework   	       0x12f548228 ChromeWebAppShortcutCopierMain + 5262484
8   Chromium Embedded Framework   	       0x12f548100 ChromeWebAppShortcutCopierMain + 5262188
9   Chromium Embedded Framework   	       0x12f5646b0 ChromeWebAppShortcutCopierMain + 5378332
10  libsystem_pthread.dylib       	       0x19f655c08 _pthread_start + 136
11  libsystem_pthread.dylib       	       0x19f650ba8 thread_start + 8

Thread 34:: ThreadPoolBackgroundWorker
0   libsystem_kernel.dylib        	       0x19f612c34 mach_msg2_trap + 8
1   libsystem_kernel.dylib        	       0x19f625028 mach_msg2_internal + 76
2   libsystem_kernel.dylib        	       0x19f61b98c mach_msg_overwrite + 484
3   libsystem_kernel.dylib        	       0x19f612fb4 mach_msg + 24
4   Chromium Embedded Framework   	       0x12f5807f8 ChromeWebAppShortcutCopierMain + 5493348
5   Chromium Embedded Framework   	       0x12f50b608 ChromeWebAppShortcutCopierMain + 5013620
6   Chromium Embedded Framework   	       0x12f548718 ChromeWebAppShortcutCopierMain + 5263748
7   Chromium Embedded Framework   	       0x12f5481a4 ChromeWebAppShortcutCopierMain + 5262352
8   Chromium Embedded Framework   	       0x12f548134 ChromeWebAppShortcutCopierMain + 5262240
9   Chromium Embedded Framework   	       0x12f5646b0 ChromeWebAppShortcutCopierMain + 5378332
10  libsystem_pthread.dylib       	       0x19f655c08 _pthread_start + 136
11  libsystem_pthread.dylib       	       0x19f650ba8 thread_start + 8

Thread 35:: ThreadPoolForegroundWorker
0   libsystem_kernel.dylib        	       0x19f612c34 mach_msg2_trap + 8
1   libsystem_kernel.dylib        	       0x19f625028 mach_msg2_internal + 76
2   libsystem_kernel.dylib        	       0x19f61b98c mach_msg_overwrite + 484
3   libsystem_kernel.dylib        	       0x19f612fb4 mach_msg + 24
4   Chromium Embedded Framework   	       0x12f5807f8 ChromeWebAppShortcutCopierMain + 5493348
5   Chromium Embedded Framework   	       0x12f50b608 ChromeWebAppShortcutCopierMain + 5013620
6   Chromium Embedded Framework   	       0x12f548718 ChromeWebAppShortcutCopierMain + 5263748
7   Chromium Embedded Framework   	       0x12f548228 ChromeWebAppShortcutCopierMain + 5262484
8   Chromium Embedded Framework   	       0x12f548100 ChromeWebAppShortcutCopierMain + 5262188
9   Chromium Embedded Framework   	       0x12f5646b0 ChromeWebAppShortcutCopierMain + 5378332
10  libsystem_pthread.dylib       	       0x19f655c08 _pthread_start + 136
11  libsystem_pthread.dylib       	       0x19f650ba8 thread_start + 8

Thread 36:: Chrome_IOThread
0   libsystem_kernel.dylib        	       0x19f61eb84 kevent64 + 8
1   Chromium Embedded Framework   	       0x12f58c5c8 ChromeWebAppShortcutCopierMain + 5541940
2   Chromium Embedded Framework   	       0x12f52ecf0 ChromeWebAppShortcutCopierMain + 5158748
3   Chromium Embedded Framework   	       0x12f4f0b6c ChromeWebAppShortcutCopierMain + 4904408
4   Chromium Embedded Framework   	       0x12f55053c ChromeWebAppShortcutCopierMain + 5296040
5   Chromium Embedded Framework   	       0x12d53c56c _v8_internal_Node_Print(void*) + 7818748
6   Chromium Embedded Framework   	       0x12f5506c8 ChromeWebAppShortcutCopierMain + 5296436
7   Chromium Embedded Framework   	       0x12f5646b0 ChromeWebAppShortcutCopierMain + 5378332
8   libsystem_pthread.dylib       	       0x19f655c08 _pthread_start + 136
9   libsystem_pthread.dylib       	       0x19f650ba8 thread_start + 8

Thread 37:: MemoryInfra
0   libsystem_kernel.dylib        	       0x19f612c34 mach_msg2_trap + 8
1   libsystem_kernel.dylib        	       0x19f625028 mach_msg2_internal + 76
2   libsystem_kernel.dylib        	       0x19f61b98c mach_msg_overwrite + 484
3   libsystem_kernel.dylib        	       0x19f612fb4 mach_msg + 24
4   Chromium Embedded Framework   	       0x12f5807f8 ChromeWebAppShortcutCopierMain + 5493348
5   Chromium Embedded Framework   	       0x12f50b608 ChromeWebAppShortcutCopierMain + 5013620
6   Chromium Embedded Framework   	       0x12f4c49c0 ChromeWebAppShortcutCopierMain + 4723756
7   Chromium Embedded Framework   	       0x12f52ecf0 ChromeWebAppShortcutCopierMain + 5158748
8   Chromium Embedded Framework   	       0x12f4f0b6c ChromeWebAppShortcutCopierMain + 4904408
9   Chromium Embedded Framework   	       0x12f55053c ChromeWebAppShortcutCopierMain + 5296040
10  Chromium Embedded Framework   	       0x12f5506c8 ChromeWebAppShortcutCopierMain + 5296436
11  Chromium Embedded Framework   	       0x12f5646b0 ChromeWebAppShortcutCopierMain + 5378332
12  libsystem_pthread.dylib       	       0x19f655c08 _pthread_start + 136
13  libsystem_pthread.dylib       	       0x19f650ba8 thread_start + 8

Thread 38:: NetworkConfigWatcher
0   libsystem_kernel.dylib        	       0x19f612c34 mach_msg2_trap + 8
1   libsystem_kernel.dylib        	       0x19f625028 mach_msg2_internal + 76
2   libsystem_kernel.dylib        	       0x19f61b98c mach_msg_overwrite + 484
3   libsystem_kernel.dylib        	       0x19f612fb4 mach_msg + 24
4   CoreFoundation                	       0x19f6f4ba0 __CFRunLoopServiceMachPort + 160
5   CoreFoundation                	       0x19f6f34f8 __CFRunLoopRun + 1188
6   CoreFoundation                	       0x19f7ade34 _CFRunLoopRunSpecificWithOptions + 532
7   Foundation                    	       0x1a1942964 -[NSRunLoop(NSRunLoop) runMode:beforeDate:] + 212
8   Chromium Embedded Framework   	       0x12f57c2b0 ChromeWebAppShortcutCopierMain + 5475612
9   Chromium Embedded Framework   	       0x12f57a174 ChromeWebAppShortcutCopierMain + 5467104
10  Chromium Embedded Framework   	       0x12f52ecf0 ChromeWebAppShortcutCopierMain + 5158748
11  Chromium Embedded Framework   	       0x12f4f0b6c ChromeWebAppShortcutCopierMain + 4904408
12  Chromium Embedded Framework   	       0x12f55053c ChromeWebAppShortcutCopierMain + 5296040
13  Chromium Embedded Framework   	       0x12f5506c8 ChromeWebAppShortcutCopierMain + 5296436
14  Chromium Embedded Framework   	       0x12f5646b0 ChromeWebAppShortcutCopierMain + 5378332
15  libsystem_pthread.dylib       	       0x19f655c08 _pthread_start + 136
16  libsystem_pthread.dylib       	       0x19f650ba8 thread_start + 8

Thread 39:: CrShutdownDetector
0   libsystem_kernel.dylib        	       0x19f613908 read + 8
1   Chromium Embedded Framework   	       0x1312481a4 rust_png$cxxbridge1$ResultOfWriter$operator$sizeof + 13197472
2   Chromium Embedded Framework   	       0x12f5646b0 ChromeWebAppShortcutCopierMain + 5378332
3   libsystem_pthread.dylib       	       0x19f655c08 _pthread_start + 136
4   libsystem_pthread.dylib       	       0x19f650ba8 thread_start + 8

Thread 40:: NetworkConfigWatcher
0   libsystem_kernel.dylib        	       0x19f612c34 mach_msg2_trap + 8
1   libsystem_kernel.dylib        	       0x19f625028 mach_msg2_internal + 76
2   libsystem_kernel.dylib        	       0x19f61b98c mach_msg_overwrite + 484
3   libsystem_kernel.dylib        	       0x19f612fb4 mach_msg + 24
4   CoreFoundation                	       0x19f6f4ba0 __CFRunLoopServiceMachPort + 160
5   CoreFoundation                	       0x19f6f34f8 __CFRunLoopRun + 1188
6   CoreFoundation                	       0x19f7ade34 _CFRunLoopRunSpecificWithOptions + 532
7   Foundation                    	       0x1a1942964 -[NSRunLoop(NSRunLoop) runMode:beforeDate:] + 212
8   Chromium Embedded Framework   	       0x12f57c2b0 ChromeWebAppShortcutCopierMain + 5475612
9   Chromium Embedded Framework   	       0x12f57a174 ChromeWebAppShortcutCopierMain + 5467104
10  Chromium Embedded Framework   	       0x12f52ecf0 ChromeWebAppShortcutCopierMain + 5158748
11  Chromium Embedded Framework   	       0x12f4f0b6c ChromeWebAppShortcutCopierMain + 4904408
12  Chromium Embedded Framework   	       0x12f55053c ChromeWebAppShortcutCopierMain + 5296040
13  Chromium Embedded Framework   	       0x12f5506c8 ChromeWebAppShortcutCopierMain + 5296436
14  Chromium Embedded Framework   	       0x12f5646b0 ChromeWebAppShortcutCopierMain + 5378332
15  libsystem_pthread.dylib       	       0x19f655c08 _pthread_start + 136
16  libsystem_pthread.dylib       	       0x19f650ba8 thread_start + 8

Thread 41:: ThreadPoolForegroundWorker
0   libsystem_kernel.dylib        	       0x19f613684 __open + 8
1   libsystem_kernel.dylib        	       0x19f61e8cc open + 64
2   Chromium Embedded Framework   	       0x12f5633b0 ChromeWebAppShortcutCopierMain + 5373468
3   Chromium Embedded Framework   	       0x12f4b44b8 ChromeWebAppShortcutCopierMain + 4656932
4   Chromium Embedded Framework   	       0x12f4b4574 ChromeWebAppShortcutCopierMain + 4657120
5   Chromium Embedded Framework   	       0x1307bb890 rust_png$cxxbridge1$ResultOfWriter$operator$sizeof + 2135948
6   Chromium Embedded Framework   	       0x13079b71c rust_png$cxxbridge1$ResultOfWriter$operator$sizeof + 2004504
7   Chromium Embedded Framework   	       0x1307ae4b4 rust_png$cxxbridge1$ResultOfWriter$operator$sizeof + 2081712
8   Chromium Embedded Framework   	       0x1307a0bd0 rust_png$cxxbridge1$ResultOfWriter$operator$sizeof + 2026188
9   Chromium Embedded Framework   	       0x1307a47f8 rust_png$cxxbridge1$ResultOfWriter$operator$sizeof + 2041588
10  Chromium Embedded Framework   	       0x13079ca84 rust_png$cxxbridge1$ResultOfWriter$operator$sizeof + 2009472
11  Chromium Embedded Framework   	       0x13079cc54 rust_png$cxxbridge1$ResultOfWriter$operator$sizeof + 2009936
12  Chromium Embedded Framework   	       0x13078ad38 rust_png$cxxbridge1$ResultOfWriter$operator$sizeof + 1936436
13  Chromium Embedded Framework   	       0x13078c3e8 rust_png$cxxbridge1$ResultOfWriter$operator$sizeof + 1942244
14  Chromium Embedded Framework   	       0x12b3fd9cc fontations_ffi$cxxbridge1$bitmap_metrics + 175016
15  Chromium Embedded Framework   	       0x12f53467c ChromeWebAppShortcutCopierMain + 5181672
16  Chromium Embedded Framework   	       0x12f50e500 ChromeWebAppShortcutCopierMain + 5025644
17  Chromium Embedded Framework   	       0x12f537a20 ChromeWebAppShortcutCopierMain + 5194892
18  Chromium Embedded Framework   	       0x12f53713c ChromeWebAppShortcutCopierMain + 5192616
19  Chromium Embedded Framework   	       0x12f548534 ChromeWebAppShortcutCopierMain + 5263264
20  Chromium Embedded Framework   	       0x12f548228 ChromeWebAppShortcutCopierMain + 5262484
21  Chromium Embedded Framework   	       0x12f548100 ChromeWebAppShortcutCopierMain + 5262188
22  Chromium Embedded Framework   	       0x12f5646b0 ChromeWebAppShortcutCopierMain + 5378332
23  libsystem_pthread.dylib       	       0x19f655c08 _pthread_start + 136
24  libsystem_pthread.dylib       	       0x19f650ba8 thread_start + 8

Thread 42:: ThreadPoolForegroundWorker
0   libsystem_kernel.dylib        	       0x19f612c34 mach_msg2_trap + 8
1   libsystem_kernel.dylib        	       0x19f625028 mach_msg2_internal + 76
2   libsystem_kernel.dylib        	       0x19f61b98c mach_msg_overwrite + 484
3   libsystem_kernel.dylib        	       0x19f612fb4 mach_msg + 24
4   Chromium Embedded Framework   	       0x12f5807f8 ChromeWebAppShortcutCopierMain + 5493348
5   Chromium Embedded Framework   	       0x12f50b608 ChromeWebAppShortcutCopierMain + 5013620
6   Chromium Embedded Framework   	       0x12f548718 ChromeWebAppShortcutCopierMain + 5263748
7   Chromium Embedded Framework   	       0x12f548228 ChromeWebAppShortcutCopierMain + 5262484
8   Chromium Embedded Framework   	       0x12f548100 ChromeWebAppShortcutCopierMain + 5262188
9   Chromium Embedded Framework   	       0x12f5646b0 ChromeWebAppShortcutCopierMain + 5378332
10  libsystem_pthread.dylib       	       0x19f655c08 _pthread_start + 136
11  libsystem_pthread.dylib       	       0x19f650ba8 thread_start + 8

Thread 43:: ThreadPoolForegroundWorker
0   libsystem_kernel.dylib        	       0x19f612c34 mach_msg2_trap + 8
1   libsystem_kernel.dylib        	       0x19f625028 mach_msg2_internal + 76
2   libsystem_kernel.dylib        	       0x19f61b98c mach_msg_overwrite + 484
3   libsystem_kernel.dylib        	       0x19f612fb4 mach_msg + 24
4   Chromium Embedded Framework   	       0x12f5807f8 ChromeWebAppShortcutCopierMain + 5493348
5   Chromium Embedded Framework   	       0x12f50b608 ChromeWebAppShortcutCopierMain + 5013620
6   Chromium Embedded Framework   	       0x12f548718 ChromeWebAppShortcutCopierMain + 5263748
7   Chromium Embedded Framework   	       0x12f548228 ChromeWebAppShortcutCopierMain + 5262484
8   Chromium Embedded Framework   	       0x12f548100 ChromeWebAppShortcutCopierMain + 5262188
9   Chromium Embedded Framework   	       0x12f5646b0 ChromeWebAppShortcutCopierMain + 5378332
10  libsystem_pthread.dylib       	       0x19f655c08 _pthread_start + 136
11  libsystem_pthread.dylib       	       0x19f650ba8 thread_start + 8

Thread 44:: ThreadPoolForegroundWorker
0   Chromium Embedded Framework   	       0x12ea62444 _v8_internal_Node_Print(void*) + 29994196
1   Chromium Embedded Framework   	       0x12ea62068 _v8_internal_Node_Print(void*) + 29993208
2   Chromium Embedded Framework   	       0x12ea61f08 _v8_internal_Node_Print(void*) + 29992856
3   Chromium Embedded Framework   	       0x12ea61e20 _v8_internal_Node_Print(void*) + 29992624
4   Chromium Embedded Framework   	       0x12ea618b4 _v8_internal_Node_Print(void*) + 29991236
5   Chromium Embedded Framework   	       0x12ea6328c _v8_internal_Node_Print(void*) + 29997852
6   Chromium Embedded Framework   	       0x12f53467c ChromeWebAppShortcutCopierMain + 5181672
7   Chromium Embedded Framework   	       0x12f50e500 ChromeWebAppShortcutCopierMain + 5025644
8   Chromium Embedded Framework   	       0x12f537b00 ChromeWebAppShortcutCopierMain + 5195116
9   Chromium Embedded Framework   	       0x12f536f44 ChromeWebAppShortcutCopierMain + 5192112
10  Chromium Embedded Framework   	       0x12f548534 ChromeWebAppShortcutCopierMain + 5263264
11  Chromium Embedded Framework   	       0x12f548228 ChromeWebAppShortcutCopierMain + 5262484
12  Chromium Embedded Framework   	       0x12f548100 ChromeWebAppShortcutCopierMain + 5262188
13  Chromium Embedded Framework   	       0x12f5646b0 ChromeWebAppShortcutCopierMain + 5378332
14  libsystem_pthread.dylib       	       0x19f655c08 _pthread_start + 136
15  libsystem_pthread.dylib       	       0x19f650ba8 thread_start + 8

Thread 45:: NetworkNotificationThreadMac
0   libsystem_kernel.dylib        	       0x19f612c34 mach_msg2_trap + 8
1   libsystem_kernel.dylib        	       0x19f625028 mach_msg2_internal + 76
2   libsystem_kernel.dylib        	       0x19f61b98c mach_msg_overwrite + 484
3   libsystem_kernel.dylib        	       0x19f612fb4 mach_msg + 24
4   CoreFoundation                	       0x19f6f4ba0 __CFRunLoopServiceMachPort + 160
5   CoreFoundation                	       0x19f6f34f8 __CFRunLoopRun + 1188
6   CoreFoundation                	       0x19f7ade34 _CFRunLoopRunSpecificWithOptions + 532
7   Foundation                    	       0x1a1942964 -[NSRunLoop(NSRunLoop) runMode:beforeDate:] + 212
8   Chromium Embedded Framework   	       0x12f57c2b0 ChromeWebAppShortcutCopierMain + 5475612
9   Chromium Embedded Framework   	       0x12f57a174 ChromeWebAppShortcutCopierMain + 5467104
10  Chromium Embedded Framework   	       0x12f52ecf0 ChromeWebAppShortcutCopierMain + 5158748
11  Chromium Embedded Framework   	       0x12f4f0b6c ChromeWebAppShortcutCopierMain + 4904408
12  Chromium Embedded Framework   	       0x12f55053c ChromeWebAppShortcutCopierMain + 5296040
13  Chromium Embedded Framework   	       0x12f5506c8 ChromeWebAppShortcutCopierMain + 5296436
14  Chromium Embedded Framework   	       0x12f5646b0 ChromeWebAppShortcutCopierMain + 5378332
15  libsystem_pthread.dylib       	       0x19f655c08 _pthread_start + 136
16  libsystem_pthread.dylib       	       0x19f650ba8 thread_start + 8

Thread 46:: CompositorTileWorker1
0   libsystem_kernel.dylib        	       0x19f6164f8 __psynch_cvwait + 8
1   libsystem_pthread.dylib       	       0x19f6560dc _pthread_cond_wait + 984
2   Chromium Embedded Framework   	       0x12f563e84 ChromeWebAppShortcutCopierMain + 5376240
3   Chromium Embedded Framework   	       0x130843748 rust_png$cxxbridge1$ResultOfWriter$operator$sizeof + 2692676
4   Chromium Embedded Framework   	       0x12f5646b0 ChromeWebAppShortcutCopierMain + 5378332
5   libsystem_pthread.dylib       	       0x19f655c08 _pthread_start + 136
6   libsystem_pthread.dylib       	       0x19f650ba8 thread_start + 8

Thread 47:: ThreadPoolSingleThreadForegroundBlocking0
0   libsystem_kernel.dylib        	       0x19f612c34 mach_msg2_trap + 8
1   libsystem_kernel.dylib        	       0x19f625028 mach_msg2_internal + 76
2   libsystem_kernel.dylib        	       0x19f61b98c mach_msg_overwrite + 484
3   libsystem_kernel.dylib        	       0x19f612fb4 mach_msg + 24
4   Chromium Embedded Framework   	       0x12f5807f8 ChromeWebAppShortcutCopierMain + 5493348
5   Chromium Embedded Framework   	       0x12f50b608 ChromeWebAppShortcutCopierMain + 5013620
6   Chromium Embedded Framework   	       0x12f548718 ChromeWebAppShortcutCopierMain + 5263748
7   Chromium Embedded Framework   	       0x12f548280 ChromeWebAppShortcutCopierMain + 5262572
8   Chromium Embedded Framework   	       0x12f54815c ChromeWebAppShortcutCopierMain + 5262280
9   Chromium Embedded Framework   	       0x12f5646b0 ChromeWebAppShortcutCopierMain + 5378332
10  libsystem_pthread.dylib       	       0x19f655c08 _pthread_start + 136
11  libsystem_pthread.dylib       	       0x19f650ba8 thread_start + 8

Thread 48:: ThreadPoolSingleThreadSharedBackgroundBlocking1
0   libsystem_kernel.dylib        	       0x19f612c34 mach_msg2_trap + 8
1   libsystem_kernel.dylib        	       0x19f625028 mach_msg2_internal + 76
2   libsystem_kernel.dylib        	       0x19f61b98c mach_msg_overwrite + 484
3   libsystem_kernel.dylib        	       0x19f612fb4 mach_msg + 24
4   Chromium Embedded Framework   	       0x12f5807f8 ChromeWebAppShortcutCopierMain + 5493348
5   Chromium Embedded Framework   	       0x12f50b608 ChromeWebAppShortcutCopierMain + 5013620
6   Chromium Embedded Framework   	       0x12f548718 ChromeWebAppShortcutCopierMain + 5263748
7   Chromium Embedded Framework   	       0x12f5481d0 ChromeWebAppShortcutCopierMain + 5262396
8   Chromium Embedded Framework   	       0x12f548170 ChromeWebAppShortcutCopierMain + 5262300
9   Chromium Embedded Framework   	       0x12f5646b0 ChromeWebAppShortcutCopierMain + 5378332
10  libsystem_pthread.dylib       	       0x19f655c08 _pthread_start + 136
11  libsystem_pthread.dylib       	       0x19f650ba8 thread_start + 8

Thread 49:: NetworkConfigWatcher
0   libsystem_kernel.dylib        	       0x19f612c34 mach_msg2_trap + 8
1   libsystem_kernel.dylib        	       0x19f625028 mach_msg2_internal + 76
2   libsystem_kernel.dylib        	       0x19f61b98c mach_msg_overwrite + 484
3   libsystem_kernel.dylib        	       0x19f612fb4 mach_msg + 24
4   CoreFoundation                	       0x19f6f4ba0 __CFRunLoopServiceMachPort + 160
5   CoreFoundation                	       0x19f6f34f8 __CFRunLoopRun + 1188
6   CoreFoundation                	       0x19f7ade34 _CFRunLoopRunSpecificWithOptions + 532
7   Foundation                    	       0x1a1942964 -[NSRunLoop(NSRunLoop) runMode:beforeDate:] + 212
8   Chromium Embedded Framework   	       0x12f57c2b0 ChromeWebAppShortcutCopierMain + 5475612
9   Chromium Embedded Framework   	       0x12f57a174 ChromeWebAppShortcutCopierMain + 5467104
10  Chromium Embedded Framework   	       0x12f52ecf0 ChromeWebAppShortcutCopierMain + 5158748
11  Chromium Embedded Framework   	       0x12f4f0b6c ChromeWebAppShortcutCopierMain + 4904408
12  Chromium Embedded Framework   	       0x12f55053c ChromeWebAppShortcutCopierMain + 5296040
13  Chromium Embedded Framework   	       0x12f5506c8 ChromeWebAppShortcutCopierMain + 5296436
14  Chromium Embedded Framework   	       0x12f5646b0 ChromeWebAppShortcutCopierMain + 5378332
15  libsystem_pthread.dylib       	       0x19f655c08 _pthread_start + 136
16  libsystem_pthread.dylib       	       0x19f650ba8 thread_start + 8

Thread 50:: ThreadPoolSingleThreadSharedForeground2
0   libsystem_kernel.dylib        	       0x19f612c34 mach_msg2_trap + 8
1   libsystem_kernel.dylib        	       0x19f625028 mach_msg2_internal + 76
2   libsystem_kernel.dylib        	       0x19f61b98c mach_msg_overwrite + 484
3   libsystem_kernel.dylib        	       0x19f612fb4 mach_msg + 24
4   Chromium Embedded Framework   	       0x12f5807f8 ChromeWebAppShortcutCopierMain + 5493348
5   Chromium Embedded Framework   	       0x12f50b608 ChromeWebAppShortcutCopierMain + 5013620
6   Chromium Embedded Framework   	       0x12f548718 ChromeWebAppShortcutCopierMain + 5263748
7   Chromium Embedded Framework   	       0x12f548254 ChromeWebAppShortcutCopierMain + 5262528
8   Chromium Embedded Framework   	       0x12f548148 ChromeWebAppShortcutCopierMain + 5262260
9   Chromium Embedded Framework   	       0x12f5646b0 ChromeWebAppShortcutCopierMain + 5378332
10  libsystem_pthread.dylib       	       0x19f655c08 _pthread_start + 136
11  libsystem_pthread.dylib       	       0x19f650ba8 thread_start + 8

Thread 51:: ThreadPoolForegroundWorker
0   libsystem_kernel.dylib        	       0x19f612c34 mach_msg2_trap + 8
1   libsystem_kernel.dylib        	       0x19f625028 mach_msg2_internal + 76
2   libsystem_kernel.dylib        	       0x19f61b98c mach_msg_overwrite + 484
3   libsystem_kernel.dylib        	       0x19f612fb4 mach_msg + 24
4   Chromium Embedded Framework   	       0x12f5807f8 ChromeWebAppShortcutCopierMain + 5493348
5   Chromium Embedded Framework   	       0x12f50b608 ChromeWebAppShortcutCopierMain + 5013620
6   Chromium Embedded Framework   	       0x12f548718 ChromeWebAppShortcutCopierMain + 5263748
7   Chromium Embedded Framework   	       0x12f548228 ChromeWebAppShortcutCopierMain + 5262484
8   Chromium Embedded Framework   	       0x12f548100 ChromeWebAppShortcutCopierMain + 5262188
9   Chromium Embedded Framework   	       0x12f5646b0 ChromeWebAppShortcutCopierMain + 5378332
10  libsystem_pthread.dylib       	       0x19f655c08 _pthread_start + 136
11  libsystem_pthread.dylib       	       0x19f650ba8 thread_start + 8

Thread 52:: NetworkConfigWatcher
0   libsystem_kernel.dylib        	       0x19f612c34 mach_msg2_trap + 8
1   libsystem_kernel.dylib        	       0x19f625028 mach_msg2_internal + 76
2   libsystem_kernel.dylib        	       0x19f61b98c mach_msg_overwrite + 484
3   libsystem_kernel.dylib        	       0x19f612fb4 mach_msg + 24
4   CoreFoundation                	       0x19f6f4ba0 __CFRunLoopServiceMachPort + 160
5   CoreFoundation                	       0x19f6f34f8 __CFRunLoopRun + 1188
6   CoreFoundation                	       0x19f7ade34 _CFRunLoopRunSpecificWithOptions + 532
7   Foundation                    	       0x1a1942964 -[NSRunLoop(NSRunLoop) runMode:beforeDate:] + 212
8   Chromium Embedded Framework   	       0x12f57c2b0 ChromeWebAppShortcutCopierMain + 5475612
9   Chromium Embedded Framework   	       0x12f57a174 ChromeWebAppShortcutCopierMain + 5467104
10  Chromium Embedded Framework   	       0x12f52ecf0 ChromeWebAppShortcutCopierMain + 5158748
11  Chromium Embedded Framework   	       0x12f4f0b6c ChromeWebAppShortcutCopierMain + 4904408
12  Chromium Embedded Framework   	       0x12f55053c ChromeWebAppShortcutCopierMain + 5296040
13  Chromium Embedded Framework   	       0x12f5506c8 ChromeWebAppShortcutCopierMain + 5296436
14  Chromium Embedded Framework   	       0x12f5646b0 ChromeWebAppShortcutCopierMain + 5378332
15  libsystem_pthread.dylib       	       0x19f655c08 _pthread_start + 136
16  libsystem_pthread.dylib       	       0x19f650ba8 thread_start + 8

Thread 53:: ThreadPoolForegroundWorker
0   libsystem_kernel.dylib        	       0x19f612c34 mach_msg2_trap + 8
1   libsystem_kernel.dylib        	       0x19f625028 mach_msg2_internal + 76
2   libsystem_kernel.dylib        	       0x19f61b98c mach_msg_overwrite + 484
3   libsystem_kernel.dylib        	       0x19f612fb4 mach_msg + 24
4   Chromium Embedded Framework   	       0x12f5807f8 ChromeWebAppShortcutCopierMain + 5493348
5   Chromium Embedded Framework   	       0x12f50b608 ChromeWebAppShortcutCopierMain + 5013620
6   Chromium Embedded Framework   	       0x12f548718 ChromeWebAppShortcutCopierMain + 5263748
7   Chromium Embedded Framework   	       0x12f548228 ChromeWebAppShortcutCopierMain + 5262484
8   Chromium Embedded Framework   	       0x12f548100 ChromeWebAppShortcutCopierMain + 5262188
9   Chromium Embedded Framework   	       0x12f5646b0 ChromeWebAppShortcutCopierMain + 5378332
10  libsystem_pthread.dylib       	       0x19f655c08 _pthread_start + 136
11  libsystem_pthread.dylib       	       0x19f650ba8 thread_start + 8

Thread 54:: ThreadPoolForegroundWorker
0   libsystem_kernel.dylib        	       0x19f612c34 mach_msg2_trap + 8
1   libsystem_kernel.dylib        	       0x19f625028 mach_msg2_internal + 76
2   libsystem_kernel.dylib        	       0x19f61b98c mach_msg_overwrite + 484
3   libsystem_kernel.dylib        	       0x19f612fb4 mach_msg + 24
4   Chromium Embedded Framework   	       0x12f5807f8 ChromeWebAppShortcutCopierMain + 5493348
5   Chromium Embedded Framework   	       0x12f50b608 ChromeWebAppShortcutCopierMain + 5013620
6   Chromium Embedded Framework   	       0x12f548718 ChromeWebAppShortcutCopierMain + 5263748
7   Chromium Embedded Framework   	       0x12f548228 ChromeWebAppShortcutCopierMain + 5262484
8   Chromium Embedded Framework   	       0x12f548100 ChromeWebAppShortcutCopierMain + 5262188
9   Chromium Embedded Framework   	       0x12f5646b0 ChromeWebAppShortcutCopierMain + 5378332
10  libsystem_pthread.dylib       	       0x19f655c08 _pthread_start + 136
11  libsystem_pthread.dylib       	       0x19f650ba8 thread_start + 8

Thread 55:: ThreadPoolForegroundWorker
0   libsystem_kernel.dylib        	       0x19f612c34 mach_msg2_trap + 8
1   libsystem_kernel.dylib        	       0x19f625028 mach_msg2_internal + 76
2   libsystem_kernel.dylib        	       0x19f61b98c mach_msg_overwrite + 484
3   libsystem_kernel.dylib        	       0x19f612fb4 mach_msg + 24
4   Chromium Embedded Framework   	       0x12f5807f8 ChromeWebAppShortcutCopierMain + 5493348
5   Chromium Embedded Framework   	       0x12f50b608 ChromeWebAppShortcutCopierMain + 5013620
6   Chromium Embedded Framework   	       0x12f548718 ChromeWebAppShortcutCopierMain + 5263748
7   Chromium Embedded Framework   	       0x12f548228 ChromeWebAppShortcutCopierMain + 5262484
8   Chromium Embedded Framework   	       0x12f548100 ChromeWebAppShortcutCopierMain + 5262188
9   Chromium Embedded Framework   	       0x12f5646b0 ChromeWebAppShortcutCopierMain + 5378332
10  libsystem_pthread.dylib       	       0x19f655c08 _pthread_start + 136
11  libsystem_pthread.dylib       	       0x19f650ba8 thread_start + 8

Thread 56:: ThreadPoolForegroundWorker
0   libsystem_kernel.dylib        	       0x19f612c34 mach_msg2_trap + 8
1   libsystem_kernel.dylib        	       0x19f625028 mach_msg2_internal + 76
2   libsystem_kernel.dylib        	       0x19f61b98c mach_msg_overwrite + 484
3   libsystem_kernel.dylib        	       0x19f612fb4 mach_msg + 24
4   Chromium Embedded Framework   	       0x12f5807f8 ChromeWebAppShortcutCopierMain + 5493348
5   Chromium Embedded Framework   	       0x12f50b608 ChromeWebAppShortcutCopierMain + 5013620
6   Chromium Embedded Framework   	       0x12f548718 ChromeWebAppShortcutCopierMain + 5263748
7   Chromium Embedded Framework   	       0x12f548228 ChromeWebAppShortcutCopierMain + 5262484
8   Chromium Embedded Framework   	       0x12f548100 ChromeWebAppShortcutCopierMain + 5262188
9   Chromium Embedded Framework   	       0x12f5646b0 ChromeWebAppShortcutCopierMain + 5378332
10  libsystem_pthread.dylib       	       0x19f655c08 _pthread_start + 136
11  libsystem_pthread.dylib       	       0x19f650ba8 thread_start + 8

Thread 57:: ThreadPoolForegroundWorker
0   libsystem_kernel.dylib        	       0x19f612c34 mach_msg2_trap + 8
1   libsystem_kernel.dylib        	       0x19f625028 mach_msg2_internal + 76
2   libsystem_kernel.dylib        	       0x19f61b98c mach_msg_overwrite + 484
3   libsystem_kernel.dylib        	       0x19f612fb4 mach_msg + 24
4   Chromium Embedded Framework   	       0x12f5807f8 ChromeWebAppShortcutCopierMain + 5493348
5   Chromium Embedded Framework   	       0x12f50b608 ChromeWebAppShortcutCopierMain + 5013620
6   Chromium Embedded Framework   	       0x12f548718 ChromeWebAppShortcutCopierMain + 5263748
7   Chromium Embedded Framework   	       0x12f548228 ChromeWebAppShortcutCopierMain + 5262484
8   Chromium Embedded Framework   	       0x12f548100 ChromeWebAppShortcutCopierMain + 5262188
9   Chromium Embedded Framework   	       0x12f5646b0 ChromeWebAppShortcutCopierMain + 5378332
10  libsystem_pthread.dylib       	       0x19f655c08 _pthread_start + 136
11  libsystem_pthread.dylib       	       0x19f650ba8 thread_start + 8

Thread 58:: ThreadPoolForegroundWorker
0   libsystem_kernel.dylib        	       0x19f612c34 mach_msg2_trap + 8
1   libsystem_kernel.dylib        	       0x19f625028 mach_msg2_internal + 76
2   libsystem_kernel.dylib        	       0x19f61b98c mach_msg_overwrite + 484
3   libsystem_kernel.dylib        	       0x19f612fb4 mach_msg + 24
4   Chromium Embedded Framework   	       0x12f5807f8 ChromeWebAppShortcutCopierMain + 5493348
5   Chromium Embedded Framework   	       0x12f50b608 ChromeWebAppShortcutCopierMain + 5013620
6   Chromium Embedded Framework   	       0x12f548718 ChromeWebAppShortcutCopierMain + 5263748
7   Chromium Embedded Framework   	       0x12f548228 ChromeWebAppShortcutCopierMain + 5262484
8   Chromium Embedded Framework   	       0x12f548100 ChromeWebAppShortcutCopierMain + 5262188
9   Chromium Embedded Framework   	       0x12f5646b0 ChromeWebAppShortcutCopierMain + 5378332
10  libsystem_pthread.dylib       	       0x19f655c08 _pthread_start + 136
11  libsystem_pthread.dylib       	       0x19f650ba8 thread_start + 8

Thread 59:: ThreadPoolForegroundWorker
0   libsystem_kernel.dylib        	       0x19f612c34 mach_msg2_trap + 8
1   libsystem_kernel.dylib        	       0x19f625028 mach_msg2_internal + 76
2   libsystem_kernel.dylib        	       0x19f61b98c mach_msg_overwrite + 484
3   libsystem_kernel.dylib        	       0x19f612fb4 mach_msg + 24
4   Chromium Embedded Framework   	       0x12f5807f8 ChromeWebAppShortcutCopierMain + 5493348
5   Chromium Embedded Framework   	       0x12f50b608 ChromeWebAppShortcutCopierMain + 5013620
6   Chromium Embedded Framework   	       0x12f548718 ChromeWebAppShortcutCopierMain + 5263748
7   Chromium Embedded Framework   	       0x12f548228 ChromeWebAppShortcutCopierMain + 5262484
8   Chromium Embedded Framework   	       0x12f548100 ChromeWebAppShortcutCopierMain + 5262188
9   Chromium Embedded Framework   	       0x12f5646b0 ChromeWebAppShortcutCopierMain + 5378332
10  libsystem_pthread.dylib       	       0x19f655c08 _pthread_start + 136
11  libsystem_pthread.dylib       	       0x19f650ba8 thread_start + 8

Thread 60:: ThreadPoolForegroundWorker
0   libsystem_kernel.dylib        	       0x19f612c34 mach_msg2_trap + 8
1   libsystem_kernel.dylib        	       0x19f625028 mach_msg2_internal + 76
2   libsystem_kernel.dylib        	       0x19f61b98c mach_msg_overwrite + 484
3   libsystem_kernel.dylib        	       0x19f612fb4 mach_msg + 24
4   Chromium Embedded Framework   	       0x12f5807f8 ChromeWebAppShortcutCopierMain + 5493348
5   Chromium Embedded Framework   	       0x12f50b608 ChromeWebAppShortcutCopierMain + 5013620
6   Chromium Embedded Framework   	       0x12f548718 ChromeWebAppShortcutCopierMain + 5263748
7   Chromium Embedded Framework   	       0x12f548228 ChromeWebAppShortcutCopierMain + 5262484
8   Chromium Embedded Framework   	       0x12f548100 ChromeWebAppShortcutCopierMain + 5262188
9   Chromium Embedded Framework   	       0x12f5646b0 ChromeWebAppShortcutCopierMain + 5378332
10  libsystem_pthread.dylib       	       0x19f655c08 _pthread_start + 136
11  libsystem_pthread.dylib       	       0x19f650ba8 thread_start + 8

Thread 61:: ThreadPoolForegroundWorker
0   libsystem_kernel.dylib        	       0x19f612c34 mach_msg2_trap + 8
1   libsystem_kernel.dylib        	       0x19f625028 mach_msg2_internal + 76
2   libsystem_kernel.dylib        	       0x19f61b98c mach_msg_overwrite + 484
3   libsystem_kernel.dylib        	       0x19f612fb4 mach_msg + 24
4   Chromium Embedded Framework   	       0x12f5807f8 ChromeWebAppShortcutCopierMain + 5493348
5   Chromium Embedded Framework   	       0x12f50b608 ChromeWebAppShortcutCopierMain + 5013620
6   Chromium Embedded Framework   	       0x12f548718 ChromeWebAppShortcutCopierMain + 5263748
7   Chromium Embedded Framework   	       0x12f548228 ChromeWebAppShortcutCopierMain + 5262484
8   Chromium Embedded Framework   	       0x12f548100 ChromeWebAppShortcutCopierMain + 5262188
9   Chromium Embedded Framework   	       0x12f5646b0 ChromeWebAppShortcutCopierMain + 5378332
10  libsystem_pthread.dylib       	       0x19f655c08 _pthread_start + 136
11  libsystem_pthread.dylib       	       0x19f650ba8 thread_start + 8

Thread 62:: ThreadPoolSingleThreadSharedForegroundBlocking3
0   libsystem_kernel.dylib        	       0x19f612c34 mach_msg2_trap + 8
1   libsystem_kernel.dylib        	       0x19f625028 mach_msg2_internal + 76
2   libsystem_kernel.dylib        	       0x19f61b98c mach_msg_overwrite + 484
3   libsystem_kernel.dylib        	       0x19f612fb4 mach_msg + 24
4   Chromium Embedded Framework   	       0x12f5807f8 ChromeWebAppShortcutCopierMain + 5493348
5   Chromium Embedded Framework   	       0x12f50b608 ChromeWebAppShortcutCopierMain + 5013620
6   Chromium Embedded Framework   	       0x12f548718 ChromeWebAppShortcutCopierMain + 5263748
7   Chromium Embedded Framework   	       0x12f548254 ChromeWebAppShortcutCopierMain + 5262528
8   Chromium Embedded Framework   	       0x12f548148 ChromeWebAppShortcutCopierMain + 5262260
9   Chromium Embedded Framework   	       0x12f5646b0 ChromeWebAppShortcutCopierMain + 5378332
10  libsystem_pthread.dylib       	       0x19f655c08 _pthread_start + 136
11  libsystem_pthread.dylib       	       0x19f650ba8 thread_start + 8

Thread 63:
0   libsystem_kernel.dylib        	       0x19f6164f8 __psynch_cvwait + 8
1   libsystem_pthread.dylib       	       0x19f6560dc _pthread_cond_wait + 984
2   simple                        	       0x1058b19b0 std::sys::sync::condvar::pthread::Condvar::wait::h71a06353eafcdf14 + 184
3   simple                        	       0x1058b215c std::sync::poison::condvar::Condvar::wait::hf2c637a3647cef57 + 56
4   simple                        	       0x1058b5130 parking::Inner::park::h7e586f5f04c2b074 + 716
5   simple                        	       0x1058b4d3c parking::Parker::park::h30fbb16b0ec84050 + 40
6   simple                        	       0x1038fce70 async_io::driver::block_on::_$u7b$$u7b$closure$u7d$$u7d$::h3a4fbe6d982a1d9e + 960
7   simple                        	       0x1035e2a04 std::thread::local::LocalKey$LT$T$GT$::try_with::hd7b4350d7b271be6 + 228
8   simple                        	       0x1035dfc68 std::thread::local::LocalKey$LT$T$GT$::with::h8d2bf8976c42a5e7 + 32
9   simple                        	       0x1038fac6c async_io::driver::block_on::hbf9223aafc64ab9d + 156
10  simple                        	       0x103615acc bevy_tasks::task_pool::TaskPool::scope_with_executor_inner::h36dbfcdcd9161d50 + 332
11  simple                        	       0x10361a900 bevy_tasks::task_pool::TaskPool::scope::_$u7b$$u7b$closure$u7d$$u7d$::h5013b8dea351ad83 + 156
12  simple                        	       0x1035e24f0 std::thread::local::LocalKey$LT$T$GT$::try_with::hae5713f6c3a744c9 + 196
13  simple                        	       0x1035e0060 std::thread::local::LocalKey$LT$T$GT$::with::he666816c6ab6f028 + 48
14  simple                        	       0x10361a858 bevy_tasks::task_pool::TaskPool::scope::h4784166cccba7333 + 52
15  simple                        	       0x103871a64 _$LT$bevy_render..pipelined_rendering..PipelinedRenderingPlugin$u20$as$u20$bevy_app..plugin..Plugin$GT$::cleanup::_$u7b$$u7b$closure$u7d$$u7d$::h769fc96171fe7e08 + 172
16  simple                        	       0x1035353f4 std::sys::backtrace::__rust_begin_short_backtrace::ha8cf03e9c582ebf9 + 16
17  simple                        	       0x1038d9cc8 std::thread::Builder::spawn_unchecked_::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::hacbba7c1c51f5df7 + 116
18  simple                        	       0x10391a650 _$LT$core..panic..unwind_safe..AssertUnwindSafe$LT$F$GT$$u20$as$u20$core..ops..function..FnOnce$LT$$LP$$RP$$GT$$GT$::call_once::heae5d561cd8f14d3 + 44
19  simple                        	       0x103921320 std::panicking::catch_unwind::do_call::h25b68353837fbe24 + 68
20  simple                        	       0x1039131e4 __rust_try + 32
21  simple                        	       0x1038d9968 std::thread::Builder::spawn_unchecked_::_$u7b$$u7b$closure$u7d$$u7d$::h6484971cd2c10037 + 728
22  simple                        	       0x1037b67d4 core::ops::function::FnOnce::call_once$u7b$$u7b$vtable.shim$u7d$$u7d$::hf1e2892413594782 + 24
23  simple                        	       0x105c01754 std::sys::pal::unix::thread::Thread::new::thread_start::h87df50f049a92661 + 52
24  libsystem_pthread.dylib       	       0x19f655c08 _pthread_start + 136
25  libsystem_pthread.dylib       	       0x19f650ba8 thread_start + 8


Thread 0 crashed with ARM Thread State (64-bit):
    x0: 0x0000000136f0bf00   x1: 0x0000012c098669b0   x2: 0x0000000000000004   x3: 0x000000019f65c9d0
    x4: 0x000000016fb53880   x5: 0x0000000000000020   x6: 0x000000000000000a   x7: 0xffffffff00016800
    x8: 0x0000000000000000   x9: 0x0000000000000000  x10: 0x0000000000000000  x11: 0x0000000000000002
   x12: 0x0000000000000000  x13: 0x0000000000000000  x14: 0x0000000000000000  x15: 0x0000000000000000
   x16: 0x000000019f6508cc  x17: 0x000000020c576c40  x18: 0x0000000000000000  x19: 0x0000000000000083
   x20: 0x000000016fb53e60  x21: 0x000000016fb53c68  x22: 0x0000000136f0b000  x23: 0x000000016fb53ca0
   x24: 0x0000000000000002  x25: 0x0000000000000061  x26: 0x0000000135d8a8db  x27: 0x000000000000019f
   x28: 0x0000012c09866930   fp: 0x000000016fb53c50   lr: 0x000000012f4bddac
    sp: 0x000000016fb53820   pc: 0x000000012f4bde70 cpsr: 0x60000000
   far: 0x0000000000000000  esr: 0xf2000000 (Breakpoint) brk 0

Binary Images:
       0x1002a0000 -        0x108787fff simple (*) <f3844b84-1750-321e-b2de-be823904d28b> */simple
       0x11bb18000 -        0x11c343fff com.apple.AGXMetalG16X (342.3) <f3e5018e-3b97-3b77-837f-18455b13e98f> /System/Library/Extensions/AGXMetalG16X.bundle/Contents/MacOS/AGXMetalG16X
       0x11ac00000 -        0x11ad43fff com.apple.audio.units.Components (1.14) <0312381d-61ae-3ab9-9cea-b1e46a0c4e54> /System/Library/Components/CoreAudio.component/Contents/MacOS/CoreAudio
       0x12ad0c000 -        0x136283fff org.cef.framework (144.0.11.0) <4c4c4428-5555-3144-a170-b7e14b64c7dd> /Users/USER/*/Chromium Embedded Framework.framework/Chromium Embedded Framework
       0x11baf4000 -        0x11bafffff libobjc-trampolines.dylib (*) <d4baeab8-b553-3779-a0ff-d8848e7a22df> /usr/lib/libobjc-trampolines.dylib
       0x19f695000 -        0x19fbddc3f com.apple.CoreFoundation (6.9) <649000a2-3eb4-3cf5-970a-d3cb37b5780c> /System/Library/Frameworks/CoreFoundation.framework/Versions/A/CoreFoundation
       0x1ac122000 -        0x1ac42527f com.apple.HIToolbox (2.1.1) <fb92ce0c-1ee5-3f03-992c-df53ed9b3cb4> /System/Library/Frameworks/Carbon.framework/Versions/A/Frameworks/HIToolbox.framework/Versions/A/HIToolbox
       0x1a3ae7000 -        0x1a521627f com.apple.AppKit (6.9) <4e909aec-68bc-3fc9-a87a-de928e1e36e1> /System/Library/Frameworks/AppKit.framework/Versions/C/AppKit
       0x19f285000 -        0x19f323fc3 dyld (*) <0975afba-c46b-364c-bd84-a75daa9e455a> /usr/lib/dyld
               0x0 - 0xffffffffffffffff ??? (*) <00000000-0000-0000-0000-000000000000> ???
       0x19f65c000 -        0x19f6643ef libsystem_platform.dylib (*) <4dbaf982-1576-3ffc-86be-03a9d2c96be5> /usr/lib/system/libsystem_platform.dylib
       0x19f64f000 -        0x19f65babb libsystem_pthread.dylib (*) <527c4ba0-91a5-378b-b3e2-d38269ca5a66> /usr/lib/system/libsystem_pthread.dylib
       0x19f612000 -        0x19f64e49f libsystem_kernel.dylib (*) <548c45c8-9733-3f0d-8ef4-c06df1df2ad0> /usr/lib/system/libsystem_kernel.dylib
       0x19f497000 -        0x19f4dde5f libdispatch.dylib (*) <a4b349e8-dd6f-3b71-84d9-34f3b4acd849> /usr/lib/system/libdispatch.dylib
       0x19f4e1000 -        0x19f563047 libsystem_c.dylib (*) <fb5569a9-cb26-36c2-aa05-e99243692b60> /usr/lib/system/libsystem_c.dylib
       0x1a0ee7000 -        0x1a1e8a4df com.apple.Foundation (6.9) <6a518869-0a98-34cb-8a15-cc28f898255e> /System/Library/Frameworks/Foundation.framework/Versions/C/Foundation
       0x1abc6a000 -        0x1abc92d7f com.apple.audio.caulk (1.0) <d4644b08-911d-30af-82e7-c404878abf47> /System/Library/PrivateFrameworks/caulk.framework/Versions/A/caulk
       0x1b0bbc000 -        0x1b0c7e81f com.apple.MediaExperience (1.0) <20e67caa-84cf-379a-98e4-b84267bf9982> /System/Library/PrivateFrameworks/MediaExperience.framework/Versions/A/MediaExperience
       0x1a2765000 -        0x1a2f1057f com.apple.audio.CoreAudio (5.0) <f37b241b-2a83-3f86-bd94-329a18ba4715> /System/Library/Frameworks/CoreAudio.framework/Versions/A/CoreAudio

External Modification Summary:
  Calls made by other processes targeting this process:
    task_for_pid: 0
    thread_create: 0
    thread_set_state: 0
  Calls made by this process:
    task_for_pid: 0
    thread_create: 0
    thread_set_state: 0
  Calls made by all processes on this machine:
    task_for_pid: 0
    thread_create: 0
    thread_set_state: 0

VM Region Summary:
ReadOnly portion of Libraries: Total=2.2G resident=0K(0%) swapped_out_or_unallocated=2.2G(100%)
Writable regions: Total=531.8M written=1202K(0%) resident=1202K(0%) swapped_out=0K(0%) unallocated=530.7M(100%)

                                VIRTUAL   REGION 
REGION TYPE                        SIZE    COUNT (non-coalesced) 
===========                     =======  ======= 
Accelerate framework               256K        2 
Activity Tracing                   256K        1 
AttributeGraph Data               1024K        1 
ColorSync                           16K        1 
CoreAnimation                      512K       32 
CoreGraphics                        80K        5 
CoreUI image data                  240K        4 
Foundation                          48K        2 
Kernel Alloc Once                   32K        1 
MALLOC                           141.4M       30 
MALLOC guard page                 3472K        4 
Memory Tag 253                    48.0G      873 
Memory Tag 253 (reserved)          544K       34         reserved VM address space (unallocated)
PROTECTED_MEMORY                    16K        1 
STACK GUARD                       1008K       63 
Stack                            324.4M       64 
Stack Guard                       56.0M        1 
VM_ALLOCATE                       3040K       81 
VM_ALLOCATE (reserved)            2576K       21         reserved VM address space (unallocated)
__AUTH                            5790K      630 
__AUTH_CONST                      88.1M     1011 
__CTF                               824        1 
__DATA                            37.6M      965 
__DATA_CONST                      48.4M     1021 
__DATA_DIRTY                      8231K      871 
__FONT_DATA                        2352        1 
__INFO_FILTER                         8        1 
__LINKEDIT                       750.0M        6 
__OBJC_RO                         78.4M        1 
__OBJC_RW                         2570K        1 
__TEXT                             1.5G     1044 
__TPRO_CONST                       128K        2 
dyld private memory                128K        1 
mapped file                      303.2M       42 
page table in kernel              1202K        1 
shared memory                      992K       18 
===========                     =======  ======= 
TOTAL                             51.3G     6838 
TOTAL, minus reserved VM space    51.3G     6838 


-----------
Full Report
-----------

{"app_name":"simple","timestamp":"2026-01-28 14:15:29.00 +0900","app_version":"","slice_uuid":"f3844b84-1750-321e-b2de-be823904d28b","build_version":"","platform":1,"share_with_app_devs":0,"is_first_party":1,"bug_type":"309","os_version":"macOS 26.2 (25C56)","roots_installed":0,"incident_id":"3713375A-C84B-4078-945B-B78CB35001E9","name":"simple"}
{
  "uptime" : 1100000,
  "procRole" : "Foreground",
  "version" : 2,
  "userID" : 501,
  "deployVersion" : 210,
  "modelCode" : "Mac16,8",
  "coalitionID" : 215308,
  "osVersion" : {
    "train" : "macOS 26.2",
    "build" : "25C56",
    "releaseType" : "User"
  },
  "captureTime" : "2026-01-28 14:15:25.9314 +0900",
  "codeSigningMonitor" : 2,
  "incident" : "3713375A-C84B-4078-945B-B78CB35001E9",
  "pid" : 583,
  "translated" : false,
  "cpuType" : "ARM-64",
  "procLaunch" : "2026-01-28 14:15:17.5482 +0900",
  "procStartAbsTime" : 28307072685721,
  "procExitAbsTime" : 28307273739816,
  "procName" : "simple",
  "procPath" : "\/Users\/USER\/*\/simple",
  "parentProc" : "zsh",
  "parentPid" : 81610,
  "coalitionName" : "com.jetbrains.rustrover",
  "crashReporterKey" : "E312544C-7AA8-24EE-DD9A-8259785FE235",
  "appleIntelligenceStatus" : {"state":"available"},
  "developerMode" : 1,
  "responsiblePid" : 39508,
  "responsibleProc" : "rustrover",
  "codeSigningID" : "simple-34ebe75c8cde8666",
  "codeSigningTeamID" : "",
  "codeSigningFlags" : 570556929,
  "codeSigningValidationCategory" : 10,
  "codeSigningTrustLevel" : 4294967295,
  "codeSigningAuxiliaryInfo" : 0,
  "instructionByteStream" : {"beforePC":"RDGLmuADAJEDAQmLBQETiwkAAJTf\/\/8XDVx0lWAzA9AAbDeR2Qfvlg==","atPC":"AAAg1AAAQNQgACDU\/wMB0fRPAqn9ewOp\/cMAkeMTAanlGwCpEwBA+Q=="},
  "bootSessionUUID" : "F1F2AB54-A323-45F3-8B43-CD53A245875C",
  "wakeTime" : 133850,
  "sleepWakeUUID" : "050ECDBD-446D-4AE9-894A-6BEFD88138E3",
  "sip" : "enabled",
  "exception" : {"codes":"0x0000000000000001, 0x000000012f4bde70","rawCodes":[1,5088468592],"type":"EXC_BREAKPOINT","signal":"SIGTRAP"},
  "termination" : {"flags":0,"code":5,"namespace":"SIGNAL","indicator":"Trace\/BPT trap: 5","byProc":"exc handler","byPid":583},
  "os_fault" : {"process":"simple"},
  "extMods" : {"caller":{"thread_create":0,"thread_set_state":0,"task_for_pid":0},"system":{"thread_create":0,"thread_set_state":0,"task_for_pid":0},"targeted":{"thread_create":0,"thread_set_state":0,"task_for_pid":0},"warnings":0},
  "faultingThread" : 0,
  "threads" : [{"threadState":{"x":[{"value":5216714496},{"value":1288649992624},{"value":4},{"value":6969215440},{"value":6169114752},{"value":32},{"value":10},{"value":18446744069414676480},{"value":0},{"value":0},{"value":0},{"value":2},{"value":0},{"value":0},{"value":0},{"value":0},{"value":6969166028,"symbolLocation":0,"symbol":"pthread_mutex_unlock"},{"value":8796990528,"symbolLocation":0,"symbol":"_main_thread"},{"value":0},{"value":131},{"value":6169116256},{"value":6169115752},{"value":5216710656},{"value":6169115808},{"value":2},{"value":97},{"value":5198358747},{"value":415},{"value":1288649992496}],"flavor":"ARM_THREAD_STATE64","lr":{"value":5088468396},"cpsr":{"value":1610612736},"fp":{"value":6169115728},"sp":{"value":6169114656},"esr":{"value":4060086272,"description":"(Breakpoint) brk 0"},"pc":{"value":5088468592,"matchesCrashFrame":1},"far":{"value":0}},"id":21734986,"triggered":true,"name":"CrBrowserMain","queue":"com.apple.main-thread","frames":[{"imageOffset":75177584,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":4696284,"imageIndex":3},{"imageOffset":75176096,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":4694796,"imageIndex":3},{"imageOffset":75177768,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":4696468,"imageIndex":3},{"imageOffset":75177792,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":4696492,"imageIndex":3},{"imageOffset":43948224,"symbol":"_v8_internal_Node_Print(void*)","symbolLocation":9625936,"imageIndex":3},{"imageOffset":43937924,"symbol":"_v8_internal_Node_Print(void*)","symbolLocation":9615636,"imageIndex":3},{"imageOffset":43948244,"symbol":"_v8_internal_Node_Print(void*)","symbolLocation":9625956,"imageIndex":3},{"imageOffset":43933428,"symbol":"_v8_internal_Node_Print(void*)","symbolLocation":9611140,"imageIndex":3},{"imageOffset":43971852,"symbol":"_v8_internal_Node_Print(void*)","symbolLocation":9649564,"imageIndex":3},{"imageOffset":43977392,"symbol":"_v8_internal_Node_Print(void*)","symbolLocation":9655104,"imageIndex":3},{"imageOffset":42038648,"symbol":"_v8_internal_Node_Print(void*)","symbolLocation":7716360,"imageIndex":3},{"imageOffset":42496964,"symbol":"_v8_internal_Node_Print(void*)","symbolLocation":8174676,"imageIndex":3},{"imageOffset":42501716,"symbol":"_v8_internal_Node_Print(void*)","symbolLocation":8179428,"imageIndex":3},{"imageOffset":42498136,"symbol":"_v8_internal_Node_Print(void*)","symbolLocation":8175848,"imageIndex":3},{"imageOffset":75506944,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5025644,"imageIndex":3},{"imageOffset":75637868,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5156568,"imageIndex":3},{"imageOffset":3740024,"symbol":"temporal_rs_ZonedDateTime_offset_nanoseconds","symbolLocation":88712,"imageIndex":3},{"imageOffset":75640048,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5158748,"imageIndex":3},{"imageOffset":75385708,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":4904408,"imageIndex":3},{"imageOffset":75387888,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":4906588,"imageIndex":3},{"imageOffset":3814896,"symbol":"temporal_rs_OwnedRelativeTo_empty","symbolLocation":26900,"imageIndex":3},{"imageOffset":4197476,"symbol":"cef::bindings::aarch64_apple_darwin::do_message_loop_work::hac659492df1aabc5","symbolLocation":12,"imageIndex":0},{"imageOffset":890012,"symbol":"bevy_cef::common::message_loop::cef_do_message_loop_work::h8f1ace1ee38ed07e","symbolLocation":12,"imageIndex":0},{"imageOffset":378760,"symbol":"core::ops::function::FnMut::call_mut::ha6b9f8d91b37da1c","symbolLocation":44,"imageIndex":0},{"imageOffset":792284,"symbol":"core::ops::function::impls::_$LT$impl$u20$core..ops..function..FnMut$LT$A$GT$$u20$for$u20$$RF$mut$u20$F$GT$::call_mut::h5ac841d5c61e5b62","symbolLocation":52,"imageIndex":0},{"imageOffset":975468,"symbol":"_$LT$Func$u20$as$u20$bevy_ecs..system..function_system..SystemParamFunction$LT$fn$LP$F0$RP$$u20$.$GT$$u20$Out$GT$$GT$::run::call_inner::h828502a3b5a066a8","symbolLocation":52,"imageIndex":0},{"imageOffset":791668,"symbol":"_$LT$Func$u20$as$u20$bevy_ecs..system..function_system..SystemParamFunction$LT$fn$LP$F0$RP$$u20$.$GT$$u20$Out$GT$$GT$::run::h895cf1dda808cdbf","symbolLocation":48,"imageIndex":0},{"imageOffset":1019300,"symbol":"_$LT$bevy_ecs..system..function_system..FunctionSystem$LT$Marker$C$In$C$Out$C$F$GT$$u20$as$u20$bevy_ecs..system..system..System$GT$::run_unsafe::h15f96cd8046e7dee","symbolLocation":468,"imageIndex":0},{"imageOffset":1175612,"symbol":"bevy_ecs::system::system::System::run_without_applying_deferred::h84d375a4c2542f53","symbolLocation":264,"imageIndex":0},{"imageOffset":89285732,"symbol":"bevy_ecs::schedule::executor::__rust_begin_short_backtrace::run_without_applying_deferred::h28516ba104c77bb7","symbolLocation":56,"imageIndex":0},{"imageOffset":89869960,"symbol":"_$LT$bevy_ecs..schedule..executor..single_threaded..SingleThreadedExecutor$u20$as$u20$bevy_ecs..schedule..executor..SystemExecutor$GT$::run::_$u7b$$u7b$closure$u7d$$u7d$::h19c5017b631ecb2c","symbolLocation":76,"imageIndex":0},{"imageOffset":89373020,"symbol":"core::ops::function::FnOnce::call_once::h7afd215909ce7c2b","symbolLocation":16,"imageIndex":0},{"imageOffset":89160916,"symbol":"_$LT$core..panic..unwind_safe..AssertUnwindSafe$LT$F$GT$$u20$as$u20$core..ops..function..FnOnce$LT$$LP$$RP$$GT$$GT$::call_once::h1134e32544e3cb20","symbolLocation":40,"imageIndex":0},{"imageOffset":88537636,"symbol":"std::panicking::catch_unwind::do_call::h79a711b2ead31739","symbolLocation":64,"imageIndex":0},{"imageOffset":88436608,"symbol":"__rust_try","symbolLocation":32,"imageIndex":0},{"imageOffset":88335228,"symbol":"std::panic::catch_unwind::h2c39fa031c1c494d","symbolLocation":76,"imageIndex":0},{"imageOffset":89869420,"symbol":"_$LT$bevy_ecs..schedule..executor..single_threaded..SingleThreadedExecutor$u20$as$u20$bevy_ecs..schedule..executor..SystemExecutor$GT$::run::he9b683e3c95704a4","symbolLocation":600,"imageIndex":0},{"imageOffset":89708792,"symbol":"bevy_ecs::schedule::schedule::Schedule::run::he391fd6c38a13bbd","symbolLocation":140,"imageIndex":0},{"imageOffset":87820456,"symbol":"bevy_ecs::world::World::run_schedule::_$u7b$$u7b$closure$u7d$$u7d$::h6260a9bb55a1efcc","symbolLocation":44,"imageIndex":0},{"imageOffset":87826788,"symbol":"bevy_ecs::world::World::try_schedule_scope::h7d6c756fd343a72d","symbolLocation":256,"imageIndex":0},{"imageOffset":87822388,"symbol":"bevy_ecs::world::World::schedule_scope::h2e14a4b380adb1a0","symbolLocation":40,"imageIndex":0},{"imageOffset":87820400,"symbol":"bevy_ecs::world::World::run_schedule::hc6d01712a92308ba","symbolLocation":40,"imageIndex":0},{"imageOffset":87937652,"symbol":"bevy_app::sub_app::SubApp::run_default_schedule::h9fec9e241b474cd2","symbolLocation":120,"imageIndex":0},{"imageOffset":87940968,"symbol":"bevy_app::sub_app::SubApps::update::hc6b8d1d4553f5ed8","symbolLocation":24,"imageIndex":0},{"imageOffset":87908088,"symbol":"bevy_app::app::App::update::h6299a20037e0d8d3","symbolLocation":44,"imageIndex":0},{"imageOffset":4706528,"symbol":"bevy_winit::state::WinitAppRunnerState::run_app_update::hdc490dcddf5d94a2","symbolLocation":96,"imageIndex":0},{"imageOffset":4704448,"symbol":"bevy_winit::state::WinitAppRunnerState::redraw_requested::h1f30757d28fdc622","symbolLocation":828,"imageIndex":0},{"imageOffset":4703348,"symbol":"_$LT$bevy_winit..state..WinitAppRunnerState$u20$as$u20$winit..application..ApplicationHandler$LT$bevy_winit..WinitUserEvent$GT$$GT$::about_to_wait::h7b147c6669cc89a6","symbolLocation":180,"imageIndex":0},{"imageOffset":4630360,"symbol":"winit::event_loop::EventLoop$LT$T$GT$::run_app::_$u7b$$u7b$closure$u7d$$u7d$::h63f2d49927795742","symbolLocation":468,"imageIndex":0},{"imageOffset":4452200,"symbol":"winit::platform_impl::macos::event_loop::map_user_event::_$u7b$$u7b$closure$u7d$$u7d$::hcf90e9861ca9b5ec","symbolLocation":164,"imageIndex":0},{"imageOffset":6118068,"symbol":"_$LT$alloc..boxed..Box$LT$F$C$A$GT$$u20$as$u20$core..ops..function..FnMut$LT$Args$GT$$GT$::call_mut::h71d1270348335e61","symbolLocation":84,"imageIndex":0},{"imageOffset":6141588,"symbol":"winit::platform_impl::macos::event_handler::EventHandler::handle_event::hcc459ccd21ffcd9b","symbolLocation":328,"imageIndex":0},{"imageOffset":5897228,"symbol":"winit::platform_impl::macos::app_state::ApplicationDelegate::handle_event::hf4fc7de9e80a0675","symbolLocation":208,"imageIndex":0},{"imageOffset":5899320,"symbol":"winit::platform_impl::macos::app_state::ApplicationDelegate::cleared::h778fe018e2ab2321","symbolLocation":788,"imageIndex":0},{"imageOffset":6144208,"symbol":"winit::platform_impl::macos::observer::control_flow_end_handler::_$u7b$$u7b$closure$u7d$$u7d$::h7478452a073f97ea","symbolLocation":280,"imageIndex":0},{"imageOffset":6143476,"symbol":"winit::platform_impl::macos::observer::control_flow_handler::_$u7b$$u7b$closure$u7d$$u7d$::h47e2c1ca3eeea93d","symbolLocation":44,"imageIndex":0},{"imageOffset":6003940,"symbol":"std::panicking::catch_unwind::do_call::hcb350c5d9da49418","symbolLocation":60,"imageIndex":0},{"imageOffset":6149824,"symbol":"__rust_try","symbolLocation":32,"imageIndex":0},{"imageOffset":6124464,"symbol":"std::panic::catch_unwind::h7ed61c510a10f6e8","symbolLocation":72,"imageIndex":0},{"imageOffset":5908924,"symbol":"winit::platform_impl::macos::event_loop::stop_app_on_panic::hd4ed8b53274f4d16","symbolLocation":52,"imageIndex":0},{"imageOffset":6142928,"symbol":"winit::platform_impl::macos::observer::control_flow_handler::h9965d6221841a7ab","symbolLocation":320,"imageIndex":0},{"imageOffset":6143908,"symbol":"winit::platform_impl::macos::observer::control_flow_end_handler::hf6ef815383a0a437","symbolLocation":48,"imageIndex":0},{"imageOffset":388680,"symbol":"__CFRUNLOOP_IS_CALLING_OUT_TO_AN_OBSERVER_CALLBACK_FUNCTION__","symbolLocation":36,"imageIndex":5},{"imageOffset":388420,"symbol":"__CFRunLoopDoObservers","symbolLocation":648,"imageIndex":5},{"imageOffset":386032,"symbol":"__CFRunLoopRun","symbolLocation":924,"imageIndex":5},{"imageOffset":1150516,"symbol":"_CFRunLoopRunSpecificWithOptions","symbolLocation":532,"imageIndex":5},{"imageOffset":792464,"symbol":"RunCurrentEventLoopInMode","symbolLocation":316,"imageIndex":6},{"imageOffset":805560,"symbol":"ReceiveNextEventCommon","symbolLocation":488,"imageIndex":6},{"imageOffset":2419556,"symbol":"_BlockUntilNextEventMatchingListInMode","symbolLocation":48,"imageIndex":6},{"imageOffset":5397340,"symbol":"_DPSBlockUntilNextEventMatchingListInMode","symbolLocation":236,"imageIndex":7},{"imageOffset":130632,"symbol":"_DPSNextEvent","symbolLocation":588,"imageIndex":7},{"imageOffset":11447564,"symbol":"-[NSApplication(NSEventRouting) _nextEventMatchingEventMask:untilDate:inMode:dequeue:]","symbolLocation":688,"imageIndex":7},{"imageOffset":11446808,"symbol":"-[NSApplication(NSEventRouting) nextEventMatchingMask:untilDate:inMode:dequeue:]","symbolLocation":72,"imageIndex":7},{"imageOffset":100224,"symbol":"-[NSApplication run]","symbolLocation":368,"imageIndex":7},{"imageOffset":6413552,"symbol":"_$LT$$LP$$RP$$u20$as$u20$objc2..encode..EncodeArguments$GT$::__invoke::hca008099b0c64f15","symbolLocation":52,"imageIndex":0},{"imageOffset":6392832,"symbol":"objc2::runtime::message_receiver::msg_send_primitive::send::h90591bc6187c70ad","symbolLocation":60,"imageIndex":0},{"imageOffset":6368740,"symbol":"objc2::runtime::message_receiver::MessageReceiver::send_message::h5cf96313d21e6808","symbolLocation":176,"imageIndex":0},{"imageOffset":6351376,"symbol":"objc2::__macro_helpers::msg_send::MsgSend::send_message::h3e2dcd8aaadb190c","symbolLocation":172,"imageIndex":0},{"imageOffset":6354416,"symbol":"objc2_app_kit::generated::__NSApplication::NSApplication::run::h911ae9e1b203aadb","symbolLocation":68,"imageIndex":0},{"imageOffset":4453180,"symbol":"winit::platform_impl::macos::event_loop::EventLoop$LT$T$GT$::run_on_demand::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::h8299637e5ea5977f","symbolLocation":160,"imageIndex":0},{"imageOffset":5356012,"symbol":"objc2::rc::autorelease::autoreleasepool::h3c114f531de2fdfd","symbolLocation":180,"imageIndex":0},{"imageOffset":4453008,"symbol":"winit::platform_impl::macos::event_loop::EventLoop$LT$T$GT$::run_on_demand::_$u7b$$u7b$closure$u7d$$u7d$::hc91b11dfb8018ea3","symbolLocation":44,"imageIndex":0},{"imageOffset":4633388,"symbol":"winit::platform_impl::macos::event_handler::EventHandler::set::h28c81e434735e5b0","symbolLocation":608,"imageIndex":0},{"imageOffset":4834740,"symbol":"winit::platform_impl::macos::app_state::ApplicationDelegate::set_event_handler::h847e95ec9931fdca","symbolLocation":152,"imageIndex":0},{"imageOffset":4452904,"symbol":"winit::platform_impl::macos::event_loop::EventLoop$LT$T$GT$::run_on_demand::h757e142636f4ded7","symbolLocation":256,"imageIndex":0},{"imageOffset":4454328,"symbol":"winit::platform_impl::macos::event_loop::EventLoop$LT$T$GT$::run::h0a901617f86b114d","symbolLocation":28,"imageIndex":0},{"imageOffset":4629880,"symbol":"winit::event_loop::EventLoop$LT$T$GT$::run_app::h72e27d4a06c04dd3","symbolLocation":72,"imageIndex":0},{"imageOffset":4709312,"symbol":"bevy_winit::state::winit_runner::h1b63cafa665a103a","symbolLocation":1084,"imageIndex":0},{"imageOffset":5248364,"symbol":"_$LT$bevy_winit..WinitPlugin$u20$as$u20$bevy_app..plugin..Plugin$GT$::build::_$u7b$$u7b$closure$u7d$$u7d$::hb6f259281a0b51a0","symbolLocation":60,"imageIndex":0},{"imageOffset":4424588,"symbol":"core::ops::function::FnOnce::call_once$u7b$$u7b$vtable.shim$u7d$$u7d$::h7e6d9ec680d4d784","symbolLocation":64,"imageIndex":0},{"imageOffset":87799596,"symbol":"_$LT$alloc..boxed..Box$LT$F$C$A$GT$$u20$as$u20$core..ops..function..FnOnce$LT$Args$GT$$GT$::call_once::ha0affe2dda881c20","symbolLocation":104,"imageIndex":0},{"imageOffset":87908456,"symbol":"bevy_app::app::App::run::hbdbb2ea2cde7cf9a","symbolLocation":320,"imageIndex":0},{"imageOffset":48524,"sourceLine":13,"sourceFile":"simple.rs","symbol":"simple::main::h3033982a4590c6c4","imageIndex":0,"symbolLocation":108},{"imageOffset":62052,"sourceLine":253,"sourceFile":"function.rs","symbol":"core::ops::function::FnOnce::call_once::ha96ae5164d9c3d50","imageIndex":0,"symbolLocation":20},{"imageOffset":9096,"sourceLine":158,"sourceFile":"backtrace.rs","symbol":"std::sys::backtrace::__rust_begin_short_backtrace::h3b02af1e24de8ca8","imageIndex":0,"symbolLocation":24},{"imageOffset":61388,"sourceLine":206,"sourceFile":"rt.rs","symbol":"std::rt::lang_start::_$u7b$$u7b$closure$u7d$$u7d$::h296362a2232fa8a8","imageIndex":0,"symbolLocation":28},{"imageOffset":93670504,"symbol":"std::rt::lang_start_internal::hdb28e94b6865fa11","symbolLocation":940,"imageIndex":0},{"imageOffset":61348,"sourceLine":205,"sourceFile":"rt.rs","symbol":"std::rt::lang_start::hdfa26720ab6d54b6","imageIndex":0,"symbolLocation":84},{"imageOffset":49460,"symbol":"main","symbolLocation":36,"imageIndex":0},{"imageOffset":36180,"symbol":"start","symbolLocation":7184,"imageIndex":8}]},{"id":21735204,"name":"IO Task Pool (0)","threadState":{"x":[{"value":260},{"value":0},{"value":3072},{"value":0},{"value":0},{"value":160},{"value":0},{"value":0},{"value":6171283176},{"value":0},{"value":0},{"value":2},{"value":2},{"value":0},{"value":0},{"value":0},{"value":305},{"value":8819788104},{"value":0},{"value":33395476544},{"value":4615042592},{"value":6171291872},{"value":0},{"value":0},{"value":3072},{"value":3073},{"value":3328},{"value":0},{"value":0}],"flavor":"ARM_THREAD_STATE64","lr":{"value":6969188572},"cpsr":{"value":1610612736},"fp":{"value":6171283296},"sp":{"value":6171283152},"esr":{"value":1442840704,"description":"(Syscall)"},"pc":{"value":6968927480},"far":{"value":0}},"frames":[{"imageOffset":17656,"symbol":"__psynch_cvwait","symbolLocation":8,"imageIndex":12},{"imageOffset":28892,"symbol":"_pthread_cond_wait","symbolLocation":984,"imageIndex":11},{"imageOffset":90249648,"symbol":"std::sys::sync::condvar::pthread::Condvar::wait::h71a06353eafcdf14","symbolLocation":184,"imageIndex":0},{"imageOffset":90251612,"symbol":"std::sync::poison::condvar::Condvar::wait::hf2c637a3647cef57","symbolLocation":56,"imageIndex":0},{"imageOffset":90263856,"symbol":"parking::Inner::park::h7e586f5f04c2b074","symbolLocation":716,"imageIndex":0},{"imageOffset":90262844,"symbol":"parking::Parker::park::h30fbb16b0ec84050","symbolLocation":40,"imageIndex":0},{"imageOffset":89972312,"symbol":"async_io::driver::block_on::_$u7b$$u7b$closure$u7d$$u7d$::hed9df0a608ddffaf","symbolLocation":932,"imageIndex":0},{"imageOffset":89942336,"symbol":"std::thread::local::LocalKey$LT$T$GT$::try_with::h451620b445a61598","symbolLocation":232,"imageIndex":0},{"imageOffset":89941408,"symbol":"std::thread::local::LocalKey$LT$T$GT$::with::hed9c9defac547347","symbolLocation":24,"imageIndex":0},{"imageOffset":89971220,"symbol":"async_io::driver::block_on::hbd24457baf9f6709","symbolLocation":144,"imageIndex":0},{"imageOffset":89930464,"symbol":"bevy_tasks::task_pool::TaskPool::new_internal::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::h7723d0b40a74f859","symbolLocation":260,"imageIndex":0},{"imageOffset":89987656,"symbol":"std::panicking::catch_unwind::do_call::hd79b881c82ee687f","symbolLocation":68,"imageIndex":0},{"imageOffset":89997140,"symbol":"__rust_try","symbolLocation":32,"imageIndex":0},{"imageOffset":89987492,"symbol":"std::panic::catch_unwind::h3f3542d315a98067","symbolLocation":80,"imageIndex":0},{"imageOffset":89929964,"symbol":"bevy_tasks::task_pool::TaskPool::new_internal::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::ha1de2a24a679355d","symbolLocation":276,"imageIndex":0},{"imageOffset":89943476,"symbol":"std::thread::local::LocalKey$LT$T$GT$::try_with::h9b0cfac459ed4ea9","symbolLocation":220,"imageIndex":0},{"imageOffset":89940872,"symbol":"std::thread::local::LocalKey$LT$T$GT$::with::h22802b7d536f0b76","symbolLocation":24,"imageIndex":0},{"imageOffset":89929560,"symbol":"bevy_tasks::task_pool::TaskPool::new_internal::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::heaec4c69ed0fa4ba","symbolLocation":80,"imageIndex":0},{"imageOffset":89986760,"symbol":"std::sys::backtrace::__rust_begin_short_backtrace::h2ff75244d19fde0d","symbolLocation":16,"imageIndex":0},{"imageOffset":89918448,"symbol":"std::thread::Builder::spawn_unchecked_::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::h0fe23b32d7a6bec9","symbolLocation":124,"imageIndex":0},{"imageOffset":89977104,"symbol":"_$LT$core..panic..unwind_safe..AssertUnwindSafe$LT$F$GT$$u20$as$u20$core..ops..function..FnOnce$LT$$LP$$RP$$GT$$GT$::call_once::h89dbe1df15805ed9","symbolLocation":44,"imageIndex":0},{"imageOffset":89987748,"symbol":"std::panicking::catch_unwind::do_call::hfbeac956c127a39e","symbolLocation":68,"imageIndex":0},{"imageOffset":89931556,"symbol":"__rust_try","symbolLocation":32,"imageIndex":0},{"imageOffset":89917576,"symbol":"std::thread::Builder::spawn_unchecked_::_$u7b$$u7b$closure$u7d$$u7d$::hec7e514187e68944","symbolLocation":768,"imageIndex":0},{"imageOffset":89955804,"symbol":"core::ops::function::FnOnce::call_once$u7b$$u7b$vtable.shim$u7d$$u7d$::h957af17fe6bc8197","symbolLocation":24,"imageIndex":0},{"imageOffset":93722452,"symbol":"std::sys::pal::unix::thread::Thread::new::thread_start::h87df50f049a92661","symbolLocation":52,"imageIndex":0},{"imageOffset":27656,"symbol":"_pthread_start","symbolLocation":136,"imageIndex":11},{"imageOffset":7080,"symbol":"thread_start","symbolLocation":8,"imageIndex":11}]},{"id":21735205,"name":"IO Task Pool (1)","threadState":{"x":[{"value":260},{"value":0},{"value":3584},{"value":0},{"value":0},{"value":160},{"value":0},{"value":0},{"value":6173429480},{"value":0},{"value":0},{"value":2},{"value":2},{"value":0},{"value":0},{"value":0},{"value":305},{"value":8819788104},{"value":0},{"value":33395477056},{"value":4615043728},{"value":6173438176},{"value":0},{"value":0},{"value":3584},{"value":3585},{"value":3840},{"value":0},{"value":0}],"flavor":"ARM_THREAD_STATE64","lr":{"value":6969188572},"cpsr":{"value":1610612736},"fp":{"value":6173429600},"sp":{"value":6173429456},"esr":{"value":1442840704,"description":"(Syscall)"},"pc":{"value":6968927480},"far":{"value":0}},"frames":[{"imageOffset":17656,"symbol":"__psynch_cvwait","symbolLocation":8,"imageIndex":12},{"imageOffset":28892,"symbol":"_pthread_cond_wait","symbolLocation":984,"imageIndex":11},{"imageOffset":90249648,"symbol":"std::sys::sync::condvar::pthread::Condvar::wait::h71a06353eafcdf14","symbolLocation":184,"imageIndex":0},{"imageOffset":90251612,"symbol":"std::sync::poison::condvar::Condvar::wait::hf2c637a3647cef57","symbolLocation":56,"imageIndex":0},{"imageOffset":90263856,"symbol":"parking::Inner::park::h7e586f5f04c2b074","symbolLocation":716,"imageIndex":0},{"imageOffset":90262844,"symbol":"parking::Parker::park::h30fbb16b0ec84050","symbolLocation":40,"imageIndex":0},{"imageOffset":89972312,"symbol":"async_io::driver::block_on::_$u7b$$u7b$closure$u7d$$u7d$::hed9df0a608ddffaf","symbolLocation":932,"imageIndex":0},{"imageOffset":89942336,"symbol":"std::thread::local::LocalKey$LT$T$GT$::try_with::h451620b445a61598","symbolLocation":232,"imageIndex":0},{"imageOffset":89941408,"symbol":"std::thread::local::LocalKey$LT$T$GT$::with::hed9c9defac547347","symbolLocation":24,"imageIndex":0},{"imageOffset":89971220,"symbol":"async_io::driver::block_on::hbd24457baf9f6709","symbolLocation":144,"imageIndex":0},{"imageOffset":89930464,"symbol":"bevy_tasks::task_pool::TaskPool::new_internal::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::h7723d0b40a74f859","symbolLocation":260,"imageIndex":0},{"imageOffset":89987656,"symbol":"std::panicking::catch_unwind::do_call::hd79b881c82ee687f","symbolLocation":68,"imageIndex":0},{"imageOffset":89997140,"symbol":"__rust_try","symbolLocation":32,"imageIndex":0},{"imageOffset":89987492,"symbol":"std::panic::catch_unwind::h3f3542d315a98067","symbolLocation":80,"imageIndex":0},{"imageOffset":89929964,"symbol":"bevy_tasks::task_pool::TaskPool::new_internal::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::ha1de2a24a679355d","symbolLocation":276,"imageIndex":0},{"imageOffset":89943476,"symbol":"std::thread::local::LocalKey$LT$T$GT$::try_with::h9b0cfac459ed4ea9","symbolLocation":220,"imageIndex":0},{"imageOffset":89940872,"symbol":"std::thread::local::LocalKey$LT$T$GT$::with::h22802b7d536f0b76","symbolLocation":24,"imageIndex":0},{"imageOffset":89929560,"symbol":"bevy_tasks::task_pool::TaskPool::new_internal::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::heaec4c69ed0fa4ba","symbolLocation":80,"imageIndex":0},{"imageOffset":89986760,"symbol":"std::sys::backtrace::__rust_begin_short_backtrace::h2ff75244d19fde0d","symbolLocation":16,"imageIndex":0},{"imageOffset":89918448,"symbol":"std::thread::Builder::spawn_unchecked_::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::h0fe23b32d7a6bec9","symbolLocation":124,"imageIndex":0},{"imageOffset":89977104,"symbol":"_$LT$core..panic..unwind_safe..AssertUnwindSafe$LT$F$GT$$u20$as$u20$core..ops..function..FnOnce$LT$$LP$$RP$$GT$$GT$::call_once::h89dbe1df15805ed9","symbolLocation":44,"imageIndex":0},{"imageOffset":89987748,"symbol":"std::panicking::catch_unwind::do_call::hfbeac956c127a39e","symbolLocation":68,"imageIndex":0},{"imageOffset":89931556,"symbol":"__rust_try","symbolLocation":32,"imageIndex":0},{"imageOffset":89917576,"symbol":"std::thread::Builder::spawn_unchecked_::_$u7b$$u7b$closure$u7d$$u7d$::hec7e514187e68944","symbolLocation":768,"imageIndex":0},{"imageOffset":89955804,"symbol":"core::ops::function::FnOnce::call_once$u7b$$u7b$vtable.shim$u7d$$u7d$::h957af17fe6bc8197","symbolLocation":24,"imageIndex":0},{"imageOffset":93722452,"symbol":"std::sys::pal::unix::thread::Thread::new::thread_start::h87df50f049a92661","symbolLocation":52,"imageIndex":0},{"imageOffset":27656,"symbol":"_pthread_start","symbolLocation":136,"imageIndex":11},{"imageOffset":7080,"symbol":"thread_start","symbolLocation":8,"imageIndex":11}]},{"id":21735206,"name":"IO Task Pool (2)","threadState":{"x":[{"value":260},{"value":0},{"value":2048},{"value":0},{"value":0},{"value":160},{"value":0},{"value":0},{"value":6175575784},{"value":0},{"value":0},{"value":2},{"value":2},{"value":0},{"value":0},{"value":0},{"value":305},{"value":8819788104},{"value":0},{"value":33395476096},{"value":4615042336},{"value":6175584480},{"value":0},{"value":0},{"value":2048},{"value":2049},{"value":2304},{"value":0},{"value":0}],"flavor":"ARM_THREAD_STATE64","lr":{"value":6969188572},"cpsr":{"value":1610612736},"fp":{"value":6175575904},"sp":{"value":6175575760},"esr":{"value":1442840704,"description":"(Syscall)"},"pc":{"value":6968927480},"far":{"value":0}},"frames":[{"imageOffset":17656,"symbol":"__psynch_cvwait","symbolLocation":8,"imageIndex":12},{"imageOffset":28892,"symbol":"_pthread_cond_wait","symbolLocation":984,"imageIndex":11},{"imageOffset":90249648,"symbol":"std::sys::sync::condvar::pthread::Condvar::wait::h71a06353eafcdf14","symbolLocation":184,"imageIndex":0},{"imageOffset":90251612,"symbol":"std::sync::poison::condvar::Condvar::wait::hf2c637a3647cef57","symbolLocation":56,"imageIndex":0},{"imageOffset":90263856,"symbol":"parking::Inner::park::h7e586f5f04c2b074","symbolLocation":716,"imageIndex":0},{"imageOffset":90262844,"symbol":"parking::Parker::park::h30fbb16b0ec84050","symbolLocation":40,"imageIndex":0},{"imageOffset":89972312,"symbol":"async_io::driver::block_on::_$u7b$$u7b$closure$u7d$$u7d$::hed9df0a608ddffaf","symbolLocation":932,"imageIndex":0},{"imageOffset":89942336,"symbol":"std::thread::local::LocalKey$LT$T$GT$::try_with::h451620b445a61598","symbolLocation":232,"imageIndex":0},{"imageOffset":89941408,"symbol":"std::thread::local::LocalKey$LT$T$GT$::with::hed9c9defac547347","symbolLocation":24,"imageIndex":0},{"imageOffset":89971220,"symbol":"async_io::driver::block_on::hbd24457baf9f6709","symbolLocation":144,"imageIndex":0},{"imageOffset":89930464,"symbol":"bevy_tasks::task_pool::TaskPool::new_internal::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::h7723d0b40a74f859","symbolLocation":260,"imageIndex":0},{"imageOffset":89987656,"symbol":"std::panicking::catch_unwind::do_call::hd79b881c82ee687f","symbolLocation":68,"imageIndex":0},{"imageOffset":89997140,"symbol":"__rust_try","symbolLocation":32,"imageIndex":0},{"imageOffset":89987492,"symbol":"std::panic::catch_unwind::h3f3542d315a98067","symbolLocation":80,"imageIndex":0},{"imageOffset":89929964,"symbol":"bevy_tasks::task_pool::TaskPool::new_internal::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::ha1de2a24a679355d","symbolLocation":276,"imageIndex":0},{"imageOffset":89943476,"symbol":"std::thread::local::LocalKey$LT$T$GT$::try_with::h9b0cfac459ed4ea9","symbolLocation":220,"imageIndex":0},{"imageOffset":89940872,"symbol":"std::thread::local::LocalKey$LT$T$GT$::with::h22802b7d536f0b76","symbolLocation":24,"imageIndex":0},{"imageOffset":89929560,"symbol":"bevy_tasks::task_pool::TaskPool::new_internal::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::heaec4c69ed0fa4ba","symbolLocation":80,"imageIndex":0},{"imageOffset":89986760,"symbol":"std::sys::backtrace::__rust_begin_short_backtrace::h2ff75244d19fde0d","symbolLocation":16,"imageIndex":0},{"imageOffset":89918448,"symbol":"std::thread::Builder::spawn_unchecked_::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::h0fe23b32d7a6bec9","symbolLocation":124,"imageIndex":0},{"imageOffset":89977104,"symbol":"_$LT$core..panic..unwind_safe..AssertUnwindSafe$LT$F$GT$$u20$as$u20$core..ops..function..FnOnce$LT$$LP$$RP$$GT$$GT$::call_once::h89dbe1df15805ed9","symbolLocation":44,"imageIndex":0},{"imageOffset":89987748,"symbol":"std::panicking::catch_unwind::do_call::hfbeac956c127a39e","symbolLocation":68,"imageIndex":0},{"imageOffset":89931556,"symbol":"__rust_try","symbolLocation":32,"imageIndex":0},{"imageOffset":89917576,"symbol":"std::thread::Builder::spawn_unchecked_::_$u7b$$u7b$closure$u7d$$u7d$::hec7e514187e68944","symbolLocation":768,"imageIndex":0},{"imageOffset":89955804,"symbol":"core::ops::function::FnOnce::call_once$u7b$$u7b$vtable.shim$u7d$$u7d$::h957af17fe6bc8197","symbolLocation":24,"imageIndex":0},{"imageOffset":93722452,"symbol":"std::sys::pal::unix::thread::Thread::new::thread_start::h87df50f049a92661","symbolLocation":52,"imageIndex":0},{"imageOffset":27656,"symbol":"_pthread_start","symbolLocation":136,"imageIndex":11},{"imageOffset":7080,"symbol":"thread_start","symbolLocation":8,"imageIndex":11}]},{"id":21735207,"name":"Async Compute Task Pool (0)","threadState":{"x":[{"value":260},{"value":0},{"value":0},{"value":0},{"value":0},{"value":160},{"value":0},{"value":0},{"value":6177722088},{"value":0},{"value":0},{"value":2},{"value":2},{"value":0},{"value":0},{"value":0},{"value":305},{"value":8819788104},{"value":0},{"value":33395476608},{"value":4615042640},{"value":6177730784},{"value":0},{"value":0},{"value":0},{"value":1},{"value":256},{"value":0},{"value":0}],"flavor":"ARM_THREAD_STATE64","lr":{"value":6969188572},"cpsr":{"value":1610612736},"fp":{"value":6177722208},"sp":{"value":6177722064},"esr":{"value":1442840704,"description":"(Syscall)"},"pc":{"value":6968927480},"far":{"value":0}},"frames":[{"imageOffset":17656,"symbol":"__psynch_cvwait","symbolLocation":8,"imageIndex":12},{"imageOffset":28892,"symbol":"_pthread_cond_wait","symbolLocation":984,"imageIndex":11},{"imageOffset":90249648,"symbol":"std::sys::sync::condvar::pthread::Condvar::wait::h71a06353eafcdf14","symbolLocation":184,"imageIndex":0},{"imageOffset":90251612,"symbol":"std::sync::poison::condvar::Condvar::wait::hf2c637a3647cef57","symbolLocation":56,"imageIndex":0},{"imageOffset":90263856,"symbol":"parking::Inner::park::h7e586f5f04c2b074","symbolLocation":716,"imageIndex":0},{"imageOffset":90262844,"symbol":"parking::Parker::park::h30fbb16b0ec84050","symbolLocation":40,"imageIndex":0},{"imageOffset":89972312,"symbol":"async_io::driver::block_on::_$u7b$$u7b$closure$u7d$$u7d$::hed9df0a608ddffaf","symbolLocation":932,"imageIndex":0},{"imageOffset":89942336,"symbol":"std::thread::local::LocalKey$LT$T$GT$::try_with::h451620b445a61598","symbolLocation":232,"imageIndex":0},{"imageOffset":89941408,"symbol":"std::thread::local::LocalKey$LT$T$GT$::with::hed9c9defac547347","symbolLocation":24,"imageIndex":0},{"imageOffset":89971220,"symbol":"async_io::driver::block_on::hbd24457baf9f6709","symbolLocation":144,"imageIndex":0},{"imageOffset":89930464,"symbol":"bevy_tasks::task_pool::TaskPool::new_internal::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::h7723d0b40a74f859","symbolLocation":260,"imageIndex":0},{"imageOffset":89987656,"symbol":"std::panicking::catch_unwind::do_call::hd79b881c82ee687f","symbolLocation":68,"imageIndex":0},{"imageOffset":89997140,"symbol":"__rust_try","symbolLocation":32,"imageIndex":0},{"imageOffset":89987492,"symbol":"std::panic::catch_unwind::h3f3542d315a98067","symbolLocation":80,"imageIndex":0},{"imageOffset":89929964,"symbol":"bevy_tasks::task_pool::TaskPool::new_internal::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::ha1de2a24a679355d","symbolLocation":276,"imageIndex":0},{"imageOffset":89943476,"symbol":"std::thread::local::LocalKey$LT$T$GT$::try_with::h9b0cfac459ed4ea9","symbolLocation":220,"imageIndex":0},{"imageOffset":89940872,"symbol":"std::thread::local::LocalKey$LT$T$GT$::with::h22802b7d536f0b76","symbolLocation":24,"imageIndex":0},{"imageOffset":89929560,"symbol":"bevy_tasks::task_pool::TaskPool::new_internal::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::heaec4c69ed0fa4ba","symbolLocation":80,"imageIndex":0},{"imageOffset":89986760,"symbol":"std::sys::backtrace::__rust_begin_short_backtrace::h2ff75244d19fde0d","symbolLocation":16,"imageIndex":0},{"imageOffset":89918448,"symbol":"std::thread::Builder::spawn_unchecked_::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::h0fe23b32d7a6bec9","symbolLocation":124,"imageIndex":0},{"imageOffset":89977104,"symbol":"_$LT$core..panic..unwind_safe..AssertUnwindSafe$LT$F$GT$$u20$as$u20$core..ops..function..FnOnce$LT$$LP$$RP$$GT$$GT$::call_once::h89dbe1df15805ed9","symbolLocation":44,"imageIndex":0},{"imageOffset":89987748,"symbol":"std::panicking::catch_unwind::do_call::hfbeac956c127a39e","symbolLocation":68,"imageIndex":0},{"imageOffset":89931556,"symbol":"__rust_try","symbolLocation":32,"imageIndex":0},{"imageOffset":89917576,"symbol":"std::thread::Builder::spawn_unchecked_::_$u7b$$u7b$closure$u7d$$u7d$::hec7e514187e68944","symbolLocation":768,"imageIndex":0},{"imageOffset":89955804,"symbol":"core::ops::function::FnOnce::call_once$u7b$$u7b$vtable.shim$u7d$$u7d$::h957af17fe6bc8197","symbolLocation":24,"imageIndex":0},{"imageOffset":93722452,"symbol":"std::sys::pal::unix::thread::Thread::new::thread_start::h87df50f049a92661","symbolLocation":52,"imageIndex":0},{"imageOffset":27656,"symbol":"_pthread_start","symbolLocation":136,"imageIndex":11},{"imageOffset":7080,"symbol":"thread_start","symbolLocation":8,"imageIndex":11}]},{"id":21735208,"name":"Async Compute Task Pool (1)","threadState":{"x":[{"value":260},{"value":0},{"value":0},{"value":0},{"value":0},{"value":160},{"value":0},{"value":0},{"value":6179868392},{"value":0},{"value":0},{"value":2},{"value":2},{"value":0},{"value":0},{"value":0},{"value":305},{"value":8819788104},{"value":0},{"value":33395476416},{"value":4615042544},{"value":6179877088},{"value":0},{"value":0},{"value":0},{"value":1},{"value":256},{"value":0},{"value":0}],"flavor":"ARM_THREAD_STATE64","lr":{"value":6969188572},"cpsr":{"value":1610612736},"fp":{"value":6179868512},"sp":{"value":6179868368},"esr":{"value":1442840704,"description":"(Syscall)"},"pc":{"value":6968927480},"far":{"value":0}},"frames":[{"imageOffset":17656,"symbol":"__psynch_cvwait","symbolLocation":8,"imageIndex":12},{"imageOffset":28892,"symbol":"_pthread_cond_wait","symbolLocation":984,"imageIndex":11},{"imageOffset":90249648,"symbol":"std::sys::sync::condvar::pthread::Condvar::wait::h71a06353eafcdf14","symbolLocation":184,"imageIndex":0},{"imageOffset":90251612,"symbol":"std::sync::poison::condvar::Condvar::wait::hf2c637a3647cef57","symbolLocation":56,"imageIndex":0},{"imageOffset":90263856,"symbol":"parking::Inner::park::h7e586f5f04c2b074","symbolLocation":716,"imageIndex":0},{"imageOffset":90262844,"symbol":"parking::Parker::park::h30fbb16b0ec84050","symbolLocation":40,"imageIndex":0},{"imageOffset":89972312,"symbol":"async_io::driver::block_on::_$u7b$$u7b$closure$u7d$$u7d$::hed9df0a608ddffaf","symbolLocation":932,"imageIndex":0},{"imageOffset":89942336,"symbol":"std::thread::local::LocalKey$LT$T$GT$::try_with::h451620b445a61598","symbolLocation":232,"imageIndex":0},{"imageOffset":89941408,"symbol":"std::thread::local::LocalKey$LT$T$GT$::with::hed9c9defac547347","symbolLocation":24,"imageIndex":0},{"imageOffset":89971220,"symbol":"async_io::driver::block_on::hbd24457baf9f6709","symbolLocation":144,"imageIndex":0},{"imageOffset":89930464,"symbol":"bevy_tasks::task_pool::TaskPool::new_internal::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::h7723d0b40a74f859","symbolLocation":260,"imageIndex":0},{"imageOffset":89987656,"symbol":"std::panicking::catch_unwind::do_call::hd79b881c82ee687f","symbolLocation":68,"imageIndex":0},{"imageOffset":89997140,"symbol":"__rust_try","symbolLocation":32,"imageIndex":0},{"imageOffset":89987492,"symbol":"std::panic::catch_unwind::h3f3542d315a98067","symbolLocation":80,"imageIndex":0},{"imageOffset":89929964,"symbol":"bevy_tasks::task_pool::TaskPool::new_internal::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::ha1de2a24a679355d","symbolLocation":276,"imageIndex":0},{"imageOffset":89943476,"symbol":"std::thread::local::LocalKey$LT$T$GT$::try_with::h9b0cfac459ed4ea9","symbolLocation":220,"imageIndex":0},{"imageOffset":89940872,"symbol":"std::thread::local::LocalKey$LT$T$GT$::with::h22802b7d536f0b76","symbolLocation":24,"imageIndex":0},{"imageOffset":89929560,"symbol":"bevy_tasks::task_pool::TaskPool::new_internal::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::heaec4c69ed0fa4ba","symbolLocation":80,"imageIndex":0},{"imageOffset":89986760,"symbol":"std::sys::backtrace::__rust_begin_short_backtrace::h2ff75244d19fde0d","symbolLocation":16,"imageIndex":0},{"imageOffset":89918448,"symbol":"std::thread::Builder::spawn_unchecked_::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::h0fe23b32d7a6bec9","symbolLocation":124,"imageIndex":0},{"imageOffset":89977104,"symbol":"_$LT$core..panic..unwind_safe..AssertUnwindSafe$LT$F$GT$$u20$as$u20$core..ops..function..FnOnce$LT$$LP$$RP$$GT$$GT$::call_once::h89dbe1df15805ed9","symbolLocation":44,"imageIndex":0},{"imageOffset":89987748,"symbol":"std::panicking::catch_unwind::do_call::hfbeac956c127a39e","symbolLocation":68,"imageIndex":0},{"imageOffset":89931556,"symbol":"__rust_try","symbolLocation":32,"imageIndex":0},{"imageOffset":89917576,"symbol":"std::thread::Builder::spawn_unchecked_::_$u7b$$u7b$closure$u7d$$u7d$::hec7e514187e68944","symbolLocation":768,"imageIndex":0},{"imageOffset":89955804,"symbol":"core::ops::function::FnOnce::call_once$u7b$$u7b$vtable.shim$u7d$$u7d$::h957af17fe6bc8197","symbolLocation":24,"imageIndex":0},{"imageOffset":93722452,"symbol":"std::sys::pal::unix::thread::Thread::new::thread_start::h87df50f049a92661","symbolLocation":52,"imageIndex":0},{"imageOffset":27656,"symbol":"_pthread_start","symbolLocation":136,"imageIndex":11},{"imageOffset":7080,"symbol":"thread_start","symbolLocation":8,"imageIndex":11}]},{"id":21735209,"name":"Async Compute Task Pool (2)","threadState":{"x":[{"value":260},{"value":0},{"value":0},{"value":0},{"value":0},{"value":160},{"value":0},{"value":0},{"value":6182014696},{"value":0},{"value":0},{"value":2},{"value":2},{"value":0},{"value":0},{"value":0},{"value":305},{"value":8819788104},{"value":0},{"value":33395476352},{"value":4615042432},{"value":6182023392},{"value":0},{"value":0},{"value":0},{"value":1},{"value":256},{"value":0},{"value":0}],"flavor":"ARM_THREAD_STATE64","lr":{"value":6969188572},"cpsr":{"value":1610612736},"fp":{"value":6182014816},"sp":{"value":6182014672},"esr":{"value":1442840704,"description":"(Syscall)"},"pc":{"value":6968927480},"far":{"value":0}},"frames":[{"imageOffset":17656,"symbol":"__psynch_cvwait","symbolLocation":8,"imageIndex":12},{"imageOffset":28892,"symbol":"_pthread_cond_wait","symbolLocation":984,"imageIndex":11},{"imageOffset":90249648,"symbol":"std::sys::sync::condvar::pthread::Condvar::wait::h71a06353eafcdf14","symbolLocation":184,"imageIndex":0},{"imageOffset":90251612,"symbol":"std::sync::poison::condvar::Condvar::wait::hf2c637a3647cef57","symbolLocation":56,"imageIndex":0},{"imageOffset":90263856,"symbol":"parking::Inner::park::h7e586f5f04c2b074","symbolLocation":716,"imageIndex":0},{"imageOffset":90262844,"symbol":"parking::Parker::park::h30fbb16b0ec84050","symbolLocation":40,"imageIndex":0},{"imageOffset":89972312,"symbol":"async_io::driver::block_on::_$u7b$$u7b$closure$u7d$$u7d$::hed9df0a608ddffaf","symbolLocation":932,"imageIndex":0},{"imageOffset":89942336,"symbol":"std::thread::local::LocalKey$LT$T$GT$::try_with::h451620b445a61598","symbolLocation":232,"imageIndex":0},{"imageOffset":89941408,"symbol":"std::thread::local::LocalKey$LT$T$GT$::with::hed9c9defac547347","symbolLocation":24,"imageIndex":0},{"imageOffset":89971220,"symbol":"async_io::driver::block_on::hbd24457baf9f6709","symbolLocation":144,"imageIndex":0},{"imageOffset":89930464,"symbol":"bevy_tasks::task_pool::TaskPool::new_internal::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::h7723d0b40a74f859","symbolLocation":260,"imageIndex":0},{"imageOffset":89987656,"symbol":"std::panicking::catch_unwind::do_call::hd79b881c82ee687f","symbolLocation":68,"imageIndex":0},{"imageOffset":89997140,"symbol":"__rust_try","symbolLocation":32,"imageIndex":0},{"imageOffset":89987492,"symbol":"std::panic::catch_unwind::h3f3542d315a98067","symbolLocation":80,"imageIndex":0},{"imageOffset":89929964,"symbol":"bevy_tasks::task_pool::TaskPool::new_internal::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::ha1de2a24a679355d","symbolLocation":276,"imageIndex":0},{"imageOffset":89943476,"symbol":"std::thread::local::LocalKey$LT$T$GT$::try_with::h9b0cfac459ed4ea9","symbolLocation":220,"imageIndex":0},{"imageOffset":89940872,"symbol":"std::thread::local::LocalKey$LT$T$GT$::with::h22802b7d536f0b76","symbolLocation":24,"imageIndex":0},{"imageOffset":89929560,"symbol":"bevy_tasks::task_pool::TaskPool::new_internal::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::heaec4c69ed0fa4ba","symbolLocation":80,"imageIndex":0},{"imageOffset":89986760,"symbol":"std::sys::backtrace::__rust_begin_short_backtrace::h2ff75244d19fde0d","symbolLocation":16,"imageIndex":0},{"imageOffset":89918448,"symbol":"std::thread::Builder::spawn_unchecked_::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::h0fe23b32d7a6bec9","symbolLocation":124,"imageIndex":0},{"imageOffset":89977104,"symbol":"_$LT$core..panic..unwind_safe..AssertUnwindSafe$LT$F$GT$$u20$as$u20$core..ops..function..FnOnce$LT$$LP$$RP$$GT$$GT$::call_once::h89dbe1df15805ed9","symbolLocation":44,"imageIndex":0},{"imageOffset":89987748,"symbol":"std::panicking::catch_unwind::do_call::hfbeac956c127a39e","symbolLocation":68,"imageIndex":0},{"imageOffset":89931556,"symbol":"__rust_try","symbolLocation":32,"imageIndex":0},{"imageOffset":89917576,"symbol":"std::thread::Builder::spawn_unchecked_::_$u7b$$u7b$closure$u7d$$u7d$::hec7e514187e68944","symbolLocation":768,"imageIndex":0},{"imageOffset":89955804,"symbol":"core::ops::function::FnOnce::call_once$u7b$$u7b$vtable.shim$u7d$$u7d$::h957af17fe6bc8197","symbolLocation":24,"imageIndex":0},{"imageOffset":93722452,"symbol":"std::sys::pal::unix::thread::Thread::new::thread_start::h87df50f049a92661","symbolLocation":52,"imageIndex":0},{"imageOffset":27656,"symbol":"_pthread_start","symbolLocation":136,"imageIndex":11},{"imageOffset":7080,"symbol":"thread_start","symbolLocation":8,"imageIndex":11}]},{"id":21735210,"name":"Compute Task Pool (0)","threadState":{"x":[{"value":260},{"value":0},{"value":0},{"value":0},{"value":0},{"value":160},{"value":0},{"value":0},{"value":6184161000},{"value":0},{"value":0},{"value":2},{"value":2},{"value":0},{"value":0},{"value":0},{"value":305},{"value":8819788104},{"value":0},{"value":33395475840},{"value":4615042288},{"value":6184169696},{"value":0},{"value":0},{"value":0},{"value":1},{"value":256},{"value":0},{"value":0}],"flavor":"ARM_THREAD_STATE64","lr":{"value":6969188572},"cpsr":{"value":1610612736},"fp":{"value":6184161120},"sp":{"value":6184160976},"esr":{"value":1442840704,"description":"(Syscall)"},"pc":{"value":6968927480},"far":{"value":0}},"frames":[{"imageOffset":17656,"symbol":"__psynch_cvwait","symbolLocation":8,"imageIndex":12},{"imageOffset":28892,"symbol":"_pthread_cond_wait","symbolLocation":984,"imageIndex":11},{"imageOffset":90249648,"symbol":"std::sys::sync::condvar::pthread::Condvar::wait::h71a06353eafcdf14","symbolLocation":184,"imageIndex":0},{"imageOffset":90251612,"symbol":"std::sync::poison::condvar::Condvar::wait::hf2c637a3647cef57","symbolLocation":56,"imageIndex":0},{"imageOffset":90263856,"symbol":"parking::Inner::park::h7e586f5f04c2b074","symbolLocation":716,"imageIndex":0},{"imageOffset":90262844,"symbol":"parking::Parker::park::h30fbb16b0ec84050","symbolLocation":40,"imageIndex":0},{"imageOffset":89972312,"symbol":"async_io::driver::block_on::_$u7b$$u7b$closure$u7d$$u7d$::hed9df0a608ddffaf","symbolLocation":932,"imageIndex":0},{"imageOffset":89942336,"symbol":"std::thread::local::LocalKey$LT$T$GT$::try_with::h451620b445a61598","symbolLocation":232,"imageIndex":0},{"imageOffset":89941408,"symbol":"std::thread::local::LocalKey$LT$T$GT$::with::hed9c9defac547347","symbolLocation":24,"imageIndex":0},{"imageOffset":89971220,"symbol":"async_io::driver::block_on::hbd24457baf9f6709","symbolLocation":144,"imageIndex":0},{"imageOffset":89930464,"symbol":"bevy_tasks::task_pool::TaskPool::new_internal::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::h7723d0b40a74f859","symbolLocation":260,"imageIndex":0},{"imageOffset":89987656,"symbol":"std::panicking::catch_unwind::do_call::hd79b881c82ee687f","symbolLocation":68,"imageIndex":0},{"imageOffset":89997140,"symbol":"__rust_try","symbolLocation":32,"imageIndex":0},{"imageOffset":89987492,"symbol":"std::panic::catch_unwind::h3f3542d315a98067","symbolLocation":80,"imageIndex":0},{"imageOffset":89929964,"symbol":"bevy_tasks::task_pool::TaskPool::new_internal::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::ha1de2a24a679355d","symbolLocation":276,"imageIndex":0},{"imageOffset":89943476,"symbol":"std::thread::local::LocalKey$LT$T$GT$::try_with::h9b0cfac459ed4ea9","symbolLocation":220,"imageIndex":0},{"imageOffset":89940872,"symbol":"std::thread::local::LocalKey$LT$T$GT$::with::h22802b7d536f0b76","symbolLocation":24,"imageIndex":0},{"imageOffset":89929560,"symbol":"bevy_tasks::task_pool::TaskPool::new_internal::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::heaec4c69ed0fa4ba","symbolLocation":80,"imageIndex":0},{"imageOffset":89986760,"symbol":"std::sys::backtrace::__rust_begin_short_backtrace::h2ff75244d19fde0d","symbolLocation":16,"imageIndex":0},{"imageOffset":89918448,"symbol":"std::thread::Builder::spawn_unchecked_::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::h0fe23b32d7a6bec9","symbolLocation":124,"imageIndex":0},{"imageOffset":89977104,"symbol":"_$LT$core..panic..unwind_safe..AssertUnwindSafe$LT$F$GT$$u20$as$u20$core..ops..function..FnOnce$LT$$LP$$RP$$GT$$GT$::call_once::h89dbe1df15805ed9","symbolLocation":44,"imageIndex":0},{"imageOffset":89987748,"symbol":"std::panicking::catch_unwind::do_call::hfbeac956c127a39e","symbolLocation":68,"imageIndex":0},{"imageOffset":89931556,"symbol":"__rust_try","symbolLocation":32,"imageIndex":0},{"imageOffset":89917576,"symbol":"std::thread::Builder::spawn_unchecked_::_$u7b$$u7b$closure$u7d$$u7d$::hec7e514187e68944","symbolLocation":768,"imageIndex":0},{"imageOffset":89955804,"symbol":"core::ops::function::FnOnce::call_once$u7b$$u7b$vtable.shim$u7d$$u7d$::h957af17fe6bc8197","symbolLocation":24,"imageIndex":0},{"imageOffset":93722452,"symbol":"std::sys::pal::unix::thread::Thread::new::thread_start::h87df50f049a92661","symbolLocation":52,"imageIndex":0},{"imageOffset":27656,"symbol":"_pthread_start","symbolLocation":136,"imageIndex":11},{"imageOffset":7080,"symbol":"thread_start","symbolLocation":8,"imageIndex":11}]},{"id":21735211,"name":"async-io","threadState":{"x":[{"value":4},{"value":0},{"value":0},{"value":21735213},{"value":8352},{"value":10999411247616},{"value":9895604652544},{"value":10000000},{"value":33395475943},{"value":33395475936},{"value":33395475928},{"value":21735211},{"value":2},{"value":258},{"value":18446744073708372150},{"value":2304},{"value":301},{"value":8819788120},{"value":0},{"value":33395475904},{"value":258},{"value":0},{"value":33395475928},{"value":21735211},{"value":33395475936},{"value":0},{"value":0},{"value":0},{"value":0}],"flavor":"ARM_THREAD_STATE64","lr":{"value":6969175612},"cpsr":{"value":1610612736},"fp":{"value":6186314000},"sp":{"value":6186313952},"esr":{"value":1442840704,"description":"(Syscall)"},"pc":{"value":6968924616},"far":{"value":0}},"frames":[{"imageOffset":14792,"symbol":"__psynch_mutexwait","symbolLocation":8,"imageIndex":12},{"imageOffset":15932,"symbol":"_pthread_mutex_firstfit_lock_wait","symbolLocation":84,"imageIndex":11},{"imageOffset":6248,"symbol":"_pthread_mutex_firstfit_lock_slow","symbolLocation":220,"imageIndex":11},{"imageOffset":93721624,"symbol":"std::sys::pal::unix::sync::mutex::Mutex::lock::h8491ee2064f70632","symbolLocation":12,"imageIndex":0},{"imageOffset":90161900,"symbol":"std::sync::poison::mutex::Mutex$LT$T$GT$::lock::hf90b0750b0d12247","symbolLocation":36,"imageIndex":0},{"imageOffset":90146616,"symbol":"async_io::reactor::Reactor::lock::h09a06a03efd66a1d","symbolLocation":36,"imageIndex":0},{"imageOffset":90139864,"symbol":"async_io::driver::main_loop::h8864dc6a2da80f6d","symbolLocation":276,"imageIndex":0},{"imageOffset":90139556,"symbol":"async_io::driver::unparker::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::h1043717272351e1a","symbolLocation":24,"imageIndex":0},{"imageOffset":90141484,"symbol":"std::sys::backtrace::__rust_begin_short_backtrace::h70a7b233cefb4cbe","symbolLocation":24,"imageIndex":0},{"imageOffset":90171944,"symbol":"std::thread::Builder::spawn_unchecked_::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::hcac0b12a7d75ad7d","symbolLocation":100,"imageIndex":0},{"imageOffset":90132296,"symbol":"_$LT$core..panic..unwind_safe..AssertUnwindSafe$LT$F$GT$$u20$as$u20$core..ops..function..FnOnce$LT$$LP$$RP$$GT$$GT$::call_once::h6a2c5088c2cf95da","symbolLocation":44,"imageIndex":0},{"imageOffset":90153336,"symbol":"std::panicking::catch_unwind::do_call::h88097bd9eab7bb79","symbolLocation":68,"imageIndex":0},{"imageOffset":90177444,"symbol":"__rust_try","symbolLocation":32,"imageIndex":0},{"imageOffset":90171096,"symbol":"std::thread::Builder::spawn_unchecked_::_$u7b$$u7b$closure$u7d$$u7d$::hf8710d5df7914a17","symbolLocation":692,"imageIndex":0},{"imageOffset":90121768,"symbol":"core::ops::function::FnOnce::call_once$u7b$$u7b$vtable.shim$u7d$$u7d$::h07956b5b53d93086","symbolLocation":24,"imageIndex":0},{"imageOffset":93722452,"symbol":"std::sys::pal::unix::thread::Thread::new::thread_start::h87df50f049a92661","symbolLocation":52,"imageIndex":0},{"imageOffset":27656,"symbol":"_pthread_start","symbolLocation":136,"imageIndex":11},{"imageOffset":7080,"symbol":"thread_start","symbolLocation":8,"imageIndex":11}]},{"id":21735212,"name":"Compute Task Pool (1)","threadState":{"x":[{"value":260},{"value":0},{"value":256},{"value":0},{"value":0},{"value":160},{"value":0},{"value":0},{"value":6188453608},{"value":0},{"value":0},{"value":2},{"value":2},{"value":0},{"value":0},{"value":0},{"value":305},{"value":8819788104},{"value":0},{"value":33395476992},{"value":4615043680},{"value":6188462304},{"value":0},{"value":0},{"value":256},{"value":257},{"value":512},{"value":0},{"value":0}],"flavor":"ARM_THREAD_STATE64","lr":{"value":6969188572},"cpsr":{"value":1610612736},"fp":{"value":6188453728},"sp":{"value":6188453584},"esr":{"value":1442840704,"description":"(Syscall)"},"pc":{"value":6968927480},"far":{"value":0}},"frames":[{"imageOffset":17656,"symbol":"__psynch_cvwait","symbolLocation":8,"imageIndex":12},{"imageOffset":28892,"symbol":"_pthread_cond_wait","symbolLocation":984,"imageIndex":11},{"imageOffset":90249648,"symbol":"std::sys::sync::condvar::pthread::Condvar::wait::h71a06353eafcdf14","symbolLocation":184,"imageIndex":0},{"imageOffset":90251612,"symbol":"std::sync::poison::condvar::Condvar::wait::hf2c637a3647cef57","symbolLocation":56,"imageIndex":0},{"imageOffset":90263856,"symbol":"parking::Inner::park::h7e586f5f04c2b074","symbolLocation":716,"imageIndex":0},{"imageOffset":90262844,"symbol":"parking::Parker::park::h30fbb16b0ec84050","symbolLocation":40,"imageIndex":0},{"imageOffset":89972312,"symbol":"async_io::driver::block_on::_$u7b$$u7b$closure$u7d$$u7d$::hed9df0a608ddffaf","symbolLocation":932,"imageIndex":0},{"imageOffset":89942336,"symbol":"std::thread::local::LocalKey$LT$T$GT$::try_with::h451620b445a61598","symbolLocation":232,"imageIndex":0},{"imageOffset":89941408,"symbol":"std::thread::local::LocalKey$LT$T$GT$::with::hed9c9defac547347","symbolLocation":24,"imageIndex":0},{"imageOffset":89971220,"symbol":"async_io::driver::block_on::hbd24457baf9f6709","symbolLocation":144,"imageIndex":0},{"imageOffset":89930464,"symbol":"bevy_tasks::task_pool::TaskPool::new_internal::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::h7723d0b40a74f859","symbolLocation":260,"imageIndex":0},{"imageOffset":89987656,"symbol":"std::panicking::catch_unwind::do_call::hd79b881c82ee687f","symbolLocation":68,"imageIndex":0},{"imageOffset":89997140,"symbol":"__rust_try","symbolLocation":32,"imageIndex":0},{"imageOffset":89987492,"symbol":"std::panic::catch_unwind::h3f3542d315a98067","symbolLocation":80,"imageIndex":0},{"imageOffset":89929964,"symbol":"bevy_tasks::task_pool::TaskPool::new_internal::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::ha1de2a24a679355d","symbolLocation":276,"imageIndex":0},{"imageOffset":89943476,"symbol":"std::thread::local::LocalKey$LT$T$GT$::try_with::h9b0cfac459ed4ea9","symbolLocation":220,"imageIndex":0},{"imageOffset":89940872,"symbol":"std::thread::local::LocalKey$LT$T$GT$::with::h22802b7d536f0b76","symbolLocation":24,"imageIndex":0},{"imageOffset":89929560,"symbol":"bevy_tasks::task_pool::TaskPool::new_internal::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::heaec4c69ed0fa4ba","symbolLocation":80,"imageIndex":0},{"imageOffset":89986760,"symbol":"std::sys::backtrace::__rust_begin_short_backtrace::h2ff75244d19fde0d","symbolLocation":16,"imageIndex":0},{"imageOffset":89918448,"symbol":"std::thread::Builder::spawn_unchecked_::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::h0fe23b32d7a6bec9","symbolLocation":124,"imageIndex":0},{"imageOffset":89977104,"symbol":"_$LT$core..panic..unwind_safe..AssertUnwindSafe$LT$F$GT$$u20$as$u20$core..ops..function..FnOnce$LT$$LP$$RP$$GT$$GT$::call_once::h89dbe1df15805ed9","symbolLocation":44,"imageIndex":0},{"imageOffset":89987748,"symbol":"std::panicking::catch_unwind::do_call::hfbeac956c127a39e","symbolLocation":68,"imageIndex":0},{"imageOffset":89931556,"symbol":"__rust_try","symbolLocation":32,"imageIndex":0},{"imageOffset":89917576,"symbol":"std::thread::Builder::spawn_unchecked_::_$u7b$$u7b$closure$u7d$$u7d$::hec7e514187e68944","symbolLocation":768,"imageIndex":0},{"imageOffset":89955804,"symbol":"core::ops::function::FnOnce::call_once$u7b$$u7b$vtable.shim$u7d$$u7d$::h957af17fe6bc8197","symbolLocation":24,"imageIndex":0},{"imageOffset":93722452,"symbol":"std::sys::pal::unix::thread::Thread::new::thread_start::h87df50f049a92661","symbolLocation":52,"imageIndex":0},{"imageOffset":27656,"symbol":"_pthread_start","symbolLocation":136,"imageIndex":11},{"imageOffset":7080,"symbol":"thread_start","symbolLocation":8,"imageIndex":11}]},{"id":21735213,"name":"Compute Task Pool (2)","threadState":{"x":[{"value":4},{"value":0},{"value":0},{"value":33404223488},{"value":1024},{"value":0},{"value":18446744072631617535},{"value":18446726482597246976},{"value":0},{"value":0},{"value":4442491392,"symbolLocation":512,"symbol":"async_io::reactor::Reactor::get::REACTOR::hf82b073bcca8b787"},{"value":0},{"value":0},{"value":0},{"value":4611869976},{"value":33395474432},{"value":363},{"value":18446744072367376383},{"value":0},{"value":33404450016},{"value":4607574016},{"value":33403475424},{"value":4442022672,"symbolLocation":393864,"symbol":"tracing_core::dispatcher::NONE::hfd31a223a2c8a04e"},{"value":0},{"value":0},{"value":0},{"value":0},{"value":0},{"value":0}],"flavor":"ARM_THREAD_STATE64","lr":{"value":4387955904},"cpsr":{"value":2684354560},"fp":{"value":6190599168},"sp":{"value":6190598976},"esr":{"value":1442840704,"description":"(Syscall)"},"pc":{"value":6968938288},"far":{"value":0}},"frames":[{"imageOffset":28464,"symbol":"kevent","symbolLocation":8,"imageIndex":12},{"imageOffset":90236096,"symbol":"rustix::backend::event::syscalls::kevent::h1a287094417ceef1","symbolLocation":324,"imageIndex":0},{"imageOffset":90213392,"symbol":"rustix::event::kqueue::kevent_timespec::h1525ff94233bcb72","symbolLocation":156,"imageIndex":0},{"imageOffset":90201036,"symbol":"polling::kqueue::Poller::wait_deadline::ha3d7fbe46cccc0bc","symbolLocation":212,"imageIndex":0},{"imageOffset":90197136,"symbol":"polling::Poller::wait_impl::h96f408f051c897f8","symbolLocation":132,"imageIndex":0},{"imageOffset":90196916,"symbol":"polling::Poller::wait::h1619029d9ce0d281","symbolLocation":76,"imageIndex":0},{"imageOffset":90149092,"symbol":"async_io::reactor::ReactorLock::react::h2d5c19e2503546c2","symbolLocation":548,"imageIndex":0},{"imageOffset":89972604,"symbol":"async_io::driver::block_on::_$u7b$$u7b$closure$u7d$$u7d$::hed9df0a608ddffaf","symbolLocation":1224,"imageIndex":0},{"imageOffset":89942336,"symbol":"std::thread::local::LocalKey$LT$T$GT$::try_with::h451620b445a61598","symbolLocation":232,"imageIndex":0},{"imageOffset":89941408,"symbol":"std::thread::local::LocalKey$LT$T$GT$::with::hed9c9defac547347","symbolLocation":24,"imageIndex":0},{"imageOffset":89971220,"symbol":"async_io::driver::block_on::hbd24457baf9f6709","symbolLocation":144,"imageIndex":0},{"imageOffset":89930464,"symbol":"bevy_tasks::task_pool::TaskPool::new_internal::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::h7723d0b40a74f859","symbolLocation":260,"imageIndex":0},{"imageOffset":89987656,"symbol":"std::panicking::catch_unwind::do_call::hd79b881c82ee687f","symbolLocation":68,"imageIndex":0},{"imageOffset":89997140,"symbol":"__rust_try","symbolLocation":32,"imageIndex":0},{"imageOffset":89987492,"symbol":"std::panic::catch_unwind::h3f3542d315a98067","symbolLocation":80,"imageIndex":0},{"imageOffset":89929964,"symbol":"bevy_tasks::task_pool::TaskPool::new_internal::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::ha1de2a24a679355d","symbolLocation":276,"imageIndex":0},{"imageOffset":89943476,"symbol":"std::thread::local::LocalKey$LT$T$GT$::try_with::h9b0cfac459ed4ea9","symbolLocation":220,"imageIndex":0},{"imageOffset":89940872,"symbol":"std::thread::local::LocalKey$LT$T$GT$::with::h22802b7d536f0b76","symbolLocation":24,"imageIndex":0},{"imageOffset":89929560,"symbol":"bevy_tasks::task_pool::TaskPool::new_internal::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::heaec4c69ed0fa4ba","symbolLocation":80,"imageIndex":0},{"imageOffset":89986760,"symbol":"std::sys::backtrace::__rust_begin_short_backtrace::h2ff75244d19fde0d","symbolLocation":16,"imageIndex":0},{"imageOffset":89918448,"symbol":"std::thread::Builder::spawn_unchecked_::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::h0fe23b32d7a6bec9","symbolLocation":124,"imageIndex":0},{"imageOffset":89977104,"symbol":"_$LT$core..panic..unwind_safe..AssertUnwindSafe$LT$F$GT$$u20$as$u20$core..ops..function..FnOnce$LT$$LP$$RP$$GT$$GT$::call_once::h89dbe1df15805ed9","symbolLocation":44,"imageIndex":0},{"imageOffset":89987748,"symbol":"std::panicking::catch_unwind::do_call::hfbeac956c127a39e","symbolLocation":68,"imageIndex":0},{"imageOffset":89931556,"symbol":"__rust_try","symbolLocation":32,"imageIndex":0},{"imageOffset":89917576,"symbol":"std::thread::Builder::spawn_unchecked_::_$u7b$$u7b$closure$u7d$$u7d$::hec7e514187e68944","symbolLocation":768,"imageIndex":0},{"imageOffset":89955804,"symbol":"core::ops::function::FnOnce::call_once$u7b$$u7b$vtable.shim$u7d$$u7d$::h957af17fe6bc8197","symbolLocation":24,"imageIndex":0},{"imageOffset":93722452,"symbol":"std::sys::pal::unix::thread::Thread::new::thread_start::h87df50f049a92661","symbolLocation":52,"imageIndex":0},{"imageOffset":27656,"symbol":"_pthread_start","symbolLocation":136,"imageIndex":11},{"imageOffset":7080,"symbol":"thread_start","symbolLocation":8,"imageIndex":11}]},{"id":21735214,"name":"Compute Task Pool (3)","threadState":{"x":[{"value":260},{"value":0},{"value":256},{"value":0},{"value":0},{"value":160},{"value":0},{"value":0},{"value":6192746216},{"value":0},{"value":0},{"value":2},{"value":2},{"value":0},{"value":0},{"value":0},{"value":305},{"value":8819788104},{"value":0},{"value":33395477120},{"value":33407729664},{"value":6192754912},{"value":0},{"value":0},{"value":256},{"value":257},{"value":512},{"value":0},{"value":0}],"flavor":"ARM_THREAD_STATE64","lr":{"value":6969188572},"cpsr":{"value":1610612736},"fp":{"value":6192746336},"sp":{"value":6192746192},"esr":{"value":1442840704,"description":"(Syscall)"},"pc":{"value":6968927480},"far":{"value":0}},"frames":[{"imageOffset":17656,"symbol":"__psynch_cvwait","symbolLocation":8,"imageIndex":12},{"imageOffset":28892,"symbol":"_pthread_cond_wait","symbolLocation":984,"imageIndex":11},{"imageOffset":90249648,"symbol":"std::sys::sync::condvar::pthread::Condvar::wait::h71a06353eafcdf14","symbolLocation":184,"imageIndex":0},{"imageOffset":90251612,"symbol":"std::sync::poison::condvar::Condvar::wait::hf2c637a3647cef57","symbolLocation":56,"imageIndex":0},{"imageOffset":90263856,"symbol":"parking::Inner::park::h7e586f5f04c2b074","symbolLocation":716,"imageIndex":0},{"imageOffset":90262844,"symbol":"parking::Parker::park::h30fbb16b0ec84050","symbolLocation":40,"imageIndex":0},{"imageOffset":89972312,"symbol":"async_io::driver::block_on::_$u7b$$u7b$closure$u7d$$u7d$::hed9df0a608ddffaf","symbolLocation":932,"imageIndex":0},{"imageOffset":89942336,"symbol":"std::thread::local::LocalKey$LT$T$GT$::try_with::h451620b445a61598","symbolLocation":232,"imageIndex":0},{"imageOffset":89941408,"symbol":"std::thread::local::LocalKey$LT$T$GT$::with::hed9c9defac547347","symbolLocation":24,"imageIndex":0},{"imageOffset":89971220,"symbol":"async_io::driver::block_on::hbd24457baf9f6709","symbolLocation":144,"imageIndex":0},{"imageOffset":89930464,"symbol":"bevy_tasks::task_pool::TaskPool::new_internal::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::h7723d0b40a74f859","symbolLocation":260,"imageIndex":0},{"imageOffset":89987656,"symbol":"std::panicking::catch_unwind::do_call::hd79b881c82ee687f","symbolLocation":68,"imageIndex":0},{"imageOffset":89997140,"symbol":"__rust_try","symbolLocation":32,"imageIndex":0},{"imageOffset":89987492,"symbol":"std::panic::catch_unwind::h3f3542d315a98067","symbolLocation":80,"imageIndex":0},{"imageOffset":89929964,"symbol":"bevy_tasks::task_pool::TaskPool::new_internal::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::ha1de2a24a679355d","symbolLocation":276,"imageIndex":0},{"imageOffset":89943476,"symbol":"std::thread::local::LocalKey$LT$T$GT$::try_with::h9b0cfac459ed4ea9","symbolLocation":220,"imageIndex":0},{"imageOffset":89940872,"symbol":"std::thread::local::LocalKey$LT$T$GT$::with::h22802b7d536f0b76","symbolLocation":24,"imageIndex":0},{"imageOffset":89929560,"symbol":"bevy_tasks::task_pool::TaskPool::new_internal::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::heaec4c69ed0fa4ba","symbolLocation":80,"imageIndex":0},{"imageOffset":89986760,"symbol":"std::sys::backtrace::__rust_begin_short_backtrace::h2ff75244d19fde0d","symbolLocation":16,"imageIndex":0},{"imageOffset":89918448,"symbol":"std::thread::Builder::spawn_unchecked_::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::h0fe23b32d7a6bec9","symbolLocation":124,"imageIndex":0},{"imageOffset":89977104,"symbol":"_$LT$core..panic..unwind_safe..AssertUnwindSafe$LT$F$GT$$u20$as$u20$core..ops..function..FnOnce$LT$$LP$$RP$$GT$$GT$::call_once::h89dbe1df15805ed9","symbolLocation":44,"imageIndex":0},{"imageOffset":89987748,"symbol":"std::panicking::catch_unwind::do_call::hfbeac956c127a39e","symbolLocation":68,"imageIndex":0},{"imageOffset":89931556,"symbol":"__rust_try","symbolLocation":32,"imageIndex":0},{"imageOffset":89917576,"symbol":"std::thread::Builder::spawn_unchecked_::_$u7b$$u7b$closure$u7d$$u7d$::hec7e514187e68944","symbolLocation":768,"imageIndex":0},{"imageOffset":89955804,"symbol":"core::ops::function::FnOnce::call_once$u7b$$u7b$vtable.shim$u7d$$u7d$::h957af17fe6bc8197","symbolLocation":24,"imageIndex":0},{"imageOffset":93722452,"symbol":"std::sys::pal::unix::thread::Thread::new::thread_start::h87df50f049a92661","symbolLocation":52,"imageIndex":0},{"imageOffset":27656,"symbol":"_pthread_start","symbolLocation":136,"imageIndex":11},{"imageOffset":7080,"symbol":"thread_start","symbolLocation":8,"imageIndex":11}]},{"id":21735215,"name":"Compute Task Pool (4)","threadState":{"x":[{"value":260},{"value":0},{"value":0},{"value":0},{"value":0},{"value":160},{"value":0},{"value":0},{"value":6194892520},{"value":0},{"value":0},{"value":2},{"value":2},{"value":0},{"value":0},{"value":0},{"value":305},{"value":8819788104},{"value":0},{"value":33395476224},{"value":4615042384},{"value":6194901216},{"value":0},{"value":0},{"value":0},{"value":1},{"value":256},{"value":0},{"value":0}],"flavor":"ARM_THREAD_STATE64","lr":{"value":6969188572},"cpsr":{"value":1610612736},"fp":{"value":6194892640},"sp":{"value":6194892496},"esr":{"value":1442840704,"description":"(Syscall)"},"pc":{"value":6968927480},"far":{"value":0}},"frames":[{"imageOffset":17656,"symbol":"__psynch_cvwait","symbolLocation":8,"imageIndex":12},{"imageOffset":28892,"symbol":"_pthread_cond_wait","symbolLocation":984,"imageIndex":11},{"imageOffset":90249648,"symbol":"std::sys::sync::condvar::pthread::Condvar::wait::h71a06353eafcdf14","symbolLocation":184,"imageIndex":0},{"imageOffset":90251612,"symbol":"std::sync::poison::condvar::Condvar::wait::hf2c637a3647cef57","symbolLocation":56,"imageIndex":0},{"imageOffset":90263856,"symbol":"parking::Inner::park::h7e586f5f04c2b074","symbolLocation":716,"imageIndex":0},{"imageOffset":90262844,"symbol":"parking::Parker::park::h30fbb16b0ec84050","symbolLocation":40,"imageIndex":0},{"imageOffset":89972312,"symbol":"async_io::driver::block_on::_$u7b$$u7b$closure$u7d$$u7d$::hed9df0a608ddffaf","symbolLocation":932,"imageIndex":0},{"imageOffset":89942336,"symbol":"std::thread::local::LocalKey$LT$T$GT$::try_with::h451620b445a61598","symbolLocation":232,"imageIndex":0},{"imageOffset":89941408,"symbol":"std::thread::local::LocalKey$LT$T$GT$::with::hed9c9defac547347","symbolLocation":24,"imageIndex":0},{"imageOffset":89971220,"symbol":"async_io::driver::block_on::hbd24457baf9f6709","symbolLocation":144,"imageIndex":0},{"imageOffset":89930464,"symbol":"bevy_tasks::task_pool::TaskPool::new_internal::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::h7723d0b40a74f859","symbolLocation":260,"imageIndex":0},{"imageOffset":89987656,"symbol":"std::panicking::catch_unwind::do_call::hd79b881c82ee687f","symbolLocation":68,"imageIndex":0},{"imageOffset":89997140,"symbol":"__rust_try","symbolLocation":32,"imageIndex":0},{"imageOffset":89987492,"symbol":"std::panic::catch_unwind::h3f3542d315a98067","symbolLocation":80,"imageIndex":0},{"imageOffset":89929964,"symbol":"bevy_tasks::task_pool::TaskPool::new_internal::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::ha1de2a24a679355d","symbolLocation":276,"imageIndex":0},{"imageOffset":89943476,"symbol":"std::thread::local::LocalKey$LT$T$GT$::try_with::h9b0cfac459ed4ea9","symbolLocation":220,"imageIndex":0},{"imageOffset":89940872,"symbol":"std::thread::local::LocalKey$LT$T$GT$::with::h22802b7d536f0b76","symbolLocation":24,"imageIndex":0},{"imageOffset":89929560,"symbol":"bevy_tasks::task_pool::TaskPool::new_internal::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::heaec4c69ed0fa4ba","symbolLocation":80,"imageIndex":0},{"imageOffset":89986760,"symbol":"std::sys::backtrace::__rust_begin_short_backtrace::h2ff75244d19fde0d","symbolLocation":16,"imageIndex":0},{"imageOffset":89918448,"symbol":"std::thread::Builder::spawn_unchecked_::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::h0fe23b32d7a6bec9","symbolLocation":124,"imageIndex":0},{"imageOffset":89977104,"symbol":"_$LT$core..panic..unwind_safe..AssertUnwindSafe$LT$F$GT$$u20$as$u20$core..ops..function..FnOnce$LT$$LP$$RP$$GT$$GT$::call_once::h89dbe1df15805ed9","symbolLocation":44,"imageIndex":0},{"imageOffset":89987748,"symbol":"std::panicking::catch_unwind::do_call::hfbeac956c127a39e","symbolLocation":68,"imageIndex":0},{"imageOffset":89931556,"symbol":"__rust_try","symbolLocation":32,"imageIndex":0},{"imageOffset":89917576,"symbol":"std::thread::Builder::spawn_unchecked_::_$u7b$$u7b$closure$u7d$$u7d$::hec7e514187e68944","symbolLocation":768,"imageIndex":0},{"imageOffset":89955804,"symbol":"core::ops::function::FnOnce::call_once$u7b$$u7b$vtable.shim$u7d$$u7d$::h957af17fe6bc8197","symbolLocation":24,"imageIndex":0},{"imageOffset":93722452,"symbol":"std::sys::pal::unix::thread::Thread::new::thread_start::h87df50f049a92661","symbolLocation":52,"imageIndex":0},{"imageOffset":27656,"symbol":"_pthread_start","symbolLocation":136,"imageIndex":11},{"imageOffset":7080,"symbol":"thread_start","symbolLocation":8,"imageIndex":11}]},{"id":21735216,"name":"Compute Task Pool (5)","threadState":{"x":[{"value":260},{"value":0},{"value":0},{"value":0},{"value":0},{"value":160},{"value":0},{"value":0},{"value":6197038824},{"value":0},{"value":0},{"value":2},{"value":2},{"value":0},{"value":0},{"value":0},{"value":305},{"value":8819788104},{"value":0},{"value":33395476864},{"value":4615042864},{"value":6197047520},{"value":0},{"value":0},{"value":0},{"value":1},{"value":256},{"value":0},{"value":0}],"flavor":"ARM_THREAD_STATE64","lr":{"value":6969188572},"cpsr":{"value":1610612736},"fp":{"value":6197038944},"sp":{"value":6197038800},"esr":{"value":1442840704,"description":"(Syscall)"},"pc":{"value":6968927480},"far":{"value":0}},"frames":[{"imageOffset":17656,"symbol":"__psynch_cvwait","symbolLocation":8,"imageIndex":12},{"imageOffset":28892,"symbol":"_pthread_cond_wait","symbolLocation":984,"imageIndex":11},{"imageOffset":90249648,"symbol":"std::sys::sync::condvar::pthread::Condvar::wait::h71a06353eafcdf14","symbolLocation":184,"imageIndex":0},{"imageOffset":90251612,"symbol":"std::sync::poison::condvar::Condvar::wait::hf2c637a3647cef57","symbolLocation":56,"imageIndex":0},{"imageOffset":90263856,"symbol":"parking::Inner::park::h7e586f5f04c2b074","symbolLocation":716,"imageIndex":0},{"imageOffset":90262844,"symbol":"parking::Parker::park::h30fbb16b0ec84050","symbolLocation":40,"imageIndex":0},{"imageOffset":89972312,"symbol":"async_io::driver::block_on::_$u7b$$u7b$closure$u7d$$u7d$::hed9df0a608ddffaf","symbolLocation":932,"imageIndex":0},{"imageOffset":89942336,"symbol":"std::thread::local::LocalKey$LT$T$GT$::try_with::h451620b445a61598","symbolLocation":232,"imageIndex":0},{"imageOffset":89941408,"symbol":"std::thread::local::LocalKey$LT$T$GT$::with::hed9c9defac547347","symbolLocation":24,"imageIndex":0},{"imageOffset":89971220,"symbol":"async_io::driver::block_on::hbd24457baf9f6709","symbolLocation":144,"imageIndex":0},{"imageOffset":89930464,"symbol":"bevy_tasks::task_pool::TaskPool::new_internal::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::h7723d0b40a74f859","symbolLocation":260,"imageIndex":0},{"imageOffset":89987656,"symbol":"std::panicking::catch_unwind::do_call::hd79b881c82ee687f","symbolLocation":68,"imageIndex":0},{"imageOffset":89997140,"symbol":"__rust_try","symbolLocation":32,"imageIndex":0},{"imageOffset":89987492,"symbol":"std::panic::catch_unwind::h3f3542d315a98067","symbolLocation":80,"imageIndex":0},{"imageOffset":89929964,"symbol":"bevy_tasks::task_pool::TaskPool::new_internal::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::ha1de2a24a679355d","symbolLocation":276,"imageIndex":0},{"imageOffset":89943476,"symbol":"std::thread::local::LocalKey$LT$T$GT$::try_with::h9b0cfac459ed4ea9","symbolLocation":220,"imageIndex":0},{"imageOffset":89940872,"symbol":"std::thread::local::LocalKey$LT$T$GT$::with::h22802b7d536f0b76","symbolLocation":24,"imageIndex":0},{"imageOffset":89929560,"symbol":"bevy_tasks::task_pool::TaskPool::new_internal::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::heaec4c69ed0fa4ba","symbolLocation":80,"imageIndex":0},{"imageOffset":89986760,"symbol":"std::sys::backtrace::__rust_begin_short_backtrace::h2ff75244d19fde0d","symbolLocation":16,"imageIndex":0},{"imageOffset":89918448,"symbol":"std::thread::Builder::spawn_unchecked_::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::h0fe23b32d7a6bec9","symbolLocation":124,"imageIndex":0},{"imageOffset":89977104,"symbol":"_$LT$core..panic..unwind_safe..AssertUnwindSafe$LT$F$GT$$u20$as$u20$core..ops..function..FnOnce$LT$$LP$$RP$$GT$$GT$::call_once::h89dbe1df15805ed9","symbolLocation":44,"imageIndex":0},{"imageOffset":89987748,"symbol":"std::panicking::catch_unwind::do_call::hfbeac956c127a39e","symbolLocation":68,"imageIndex":0},{"imageOffset":89931556,"symbol":"__rust_try","symbolLocation":32,"imageIndex":0},{"imageOffset":89917576,"symbol":"std::thread::Builder::spawn_unchecked_::_$u7b$$u7b$closure$u7d$$u7d$::hec7e514187e68944","symbolLocation":768,"imageIndex":0},{"imageOffset":89955804,"symbol":"core::ops::function::FnOnce::call_once$u7b$$u7b$vtable.shim$u7d$$u7d$::h957af17fe6bc8197","symbolLocation":24,"imageIndex":0},{"imageOffset":93722452,"symbol":"std::sys::pal::unix::thread::Thread::new::thread_start::h87df50f049a92661","symbolLocation":52,"imageIndex":0},{"imageOffset":27656,"symbol":"_pthread_start","symbolLocation":136,"imageIndex":11},{"imageOffset":7080,"symbol":"thread_start","symbolLocation":8,"imageIndex":11}]},{"id":21735217,"name":"ctrl-c","threadState":{"x":[{"value":14},{"value":8589934595},{"value":171798697235},{"value":40694815130115},{"value":14680198217728},{"value":40694815129600},{"value":48},{"value":0},{"value":0},{"value":1},{"value":1784269877405560833},{"value":1784269877403463681},{"value":4614893920},{"value":0},{"value":4611836008},{"value":33403453440},{"value":18446744073709551580},{"value":8819792464},{"value":0},{"value":4615062608},{"value":4615062544},{"value":18446744073709551615},{"value":4441903520,"symbolLocation":274712,"symbol":"tracing_core::dispatcher::NONE::hfd31a223a2c8a04e"},{"value":0},{"value":0},{"value":0},{"value":0},{"value":0},{"value":0}],"flavor":"ARM_THREAD_STATE64","lr":{"value":6967372176},"cpsr":{"value":1610612736},"fp":{"value":6199192368},"sp":{"value":6199192352},"esr":{"value":1442840704,"description":"(Syscall)"},"pc":{"value":6968912816},"far":{"value":0}},"frames":[{"imageOffset":2992,"symbol":"semaphore_wait_trap","symbolLocation":8,"imageIndex":12},{"imageOffset":14736,"symbol":"_dispatch_sema4_wait","symbolLocation":28,"imageIndex":13},{"imageOffset":16192,"symbol":"_dispatch_semaphore_wait_slow","symbolLocation":132,"imageIndex":13},{"imageOffset":87946492,"symbol":"ctrlc::platform::unix::implementation::sem_wait_forever::h57686728d3c0c92a","symbolLocation":48,"imageIndex":0},{"imageOffset":87816528,"symbol":"ctrlc::platform::unix::block_ctrl_c::hb803e6b46ec3dfa5","symbolLocation":20,"imageIndex":0},{"imageOffset":87845932,"symbol":"ctrlc::set_handler_inner::_$u7b$$u7b$closure$u7d$$u7d$::h39ca9869d5417c95","symbolLocation":24,"imageIndex":0},{"imageOffset":87897812,"symbol":"std::sys::backtrace::__rust_begin_short_backtrace::h8a8cc888cd60e6ef","symbolLocation":16,"imageIndex":0},{"imageOffset":87790756,"symbol":"std::thread::Builder::spawn_unchecked_::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::hd6e9bbddf5667bde","symbolLocation":88,"imageIndex":0},{"imageOffset":87836444,"symbol":"_$LT$core..panic..unwind_safe..AssertUnwindSafe$LT$F$GT$$u20$as$u20$core..ops..function..FnOnce$LT$$LP$$RP$$GT$$GT$::call_once::h5123a29a249bed10","symbolLocation":40,"imageIndex":0},{"imageOffset":87813136,"symbol":"std::panicking::catch_unwind::do_call::h99551042f19c334b","symbolLocation":64,"imageIndex":0},{"imageOffset":87807244,"symbol":"__rust_try","symbolLocation":32,"imageIndex":0},{"imageOffset":87789920,"symbol":"std::thread::Builder::spawn_unchecked_::_$u7b$$u7b$closure$u7d$$u7d$::h6e114e23ce529a23","symbolLocation":644,"imageIndex":0},{"imageOffset":87765292,"symbol":"core::ops::function::FnOnce::call_once$u7b$$u7b$vtable.shim$u7d$$u7d$::hb767d9bb8531d226","symbolLocation":24,"imageIndex":0},{"imageOffset":93722452,"symbol":"std::sys::pal::unix::thread::Thread::new::thread_start::h87df50f049a92661","symbolLocation":52,"imageIndex":0},{"imageOffset":27656,"symbol":"_pthread_start","symbolLocation":136,"imageIndex":11},{"imageOffset":7080,"symbol":"thread_start","symbolLocation":8,"imageIndex":11}]},{"id":21735218,"name":"notify-rs debouncer loop","threadState":{"x":[{"value":4},{"value":0},{"value":1},{"value":1},{"value":0},{"value":75000000},{"value":52},{"value":0},{"value":8797032232,"symbolLocation":0,"symbol":"clock_sem"},{"value":16387},{"value":17},{"value":2},{"value":0},{"value":0},{"value":16},{"value":90},{"value":334},{"value":8819788224},{"value":0},{"value":6201337000},{"value":6201337000},{"value":4615069872},{"value":4441175488,"symbolLocation":71744,"symbol":"image::io::free_functions::MAGIC_BYTES::hd344a57b3ccfd3cd"},{"value":0},{"value":0},{"value":0},{"value":0},{"value":0},{"value":0}],"flavor":"ARM_THREAD_STATE64","lr":{"value":6967717228},"cpsr":{"value":1610612736},"fp":{"value":6201336976},"sp":{"value":6201336928},"esr":{"value":1442840704,"description":"(Syscall)"},"pc":{"value":6968926964},"far":{"value":0}},"frames":[{"imageOffset":17140,"symbol":"__semwait_signal","symbolLocation":8,"imageIndex":12},{"imageOffset":56684,"symbol":"nanosleep","symbolLocation":220,"imageIndex":14},{"imageOffset":93673408,"symbol":"std::thread::sleep::h17057c3b27540418","symbolLocation":84,"imageIndex":0},{"imageOffset":78346576,"symbol":"notify_debouncer_full::new_debouncer_opt::_$u7b$$u7b$closure$u7d$$u7d$::hb1ac894d1b229b11","symbolLocation":128,"imageIndex":0},{"imageOffset":78650296,"symbol":"std::sys::backtrace::__rust_begin_short_backtrace::hf85bb894ed39128e","symbolLocation":16,"imageIndex":0},{"imageOffset":78486896,"symbol":"std::thread::Builder::spawn_unchecked_::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::ha5b1c662df64ae7e","symbolLocation":124,"imageIndex":0},{"imageOffset":78639044,"symbol":"_$LT$core..panic..unwind_safe..AssertUnwindSafe$LT$F$GT$$u20$as$u20$core..ops..function..FnOnce$LT$$LP$$RP$$GT$$GT$::call_once::h0d23523e05f3bc65","symbolLocation":44,"imageIndex":0},{"imageOffset":79652056,"symbol":"std::panicking::catch_unwind::do_call::hd259eb5268343aae","symbolLocation":68,"imageIndex":0},{"imageOffset":78635600,"symbol":"__rust_try","symbolLocation":32,"imageIndex":0},{"imageOffset":78486024,"symbol":"std::thread::Builder::spawn_unchecked_::_$u7b$$u7b$closure$u7d$$u7d$::h971eb8b4be5c1343","symbolLocation":768,"imageIndex":0},{"imageOffset":78102528,"symbol":"core::ops::function::FnOnce::call_once$u7b$$u7b$vtable.shim$u7d$$u7d$::hc21e4375ec08643b","symbolLocation":24,"imageIndex":0},{"imageOffset":93722452,"symbol":"std::sys::pal::unix::thread::Thread::new::thread_start::h87df50f049a92661","symbolLocation":52,"imageIndex":0},{"imageOffset":27656,"symbol":"_pthread_start","symbolLocation":136,"imageIndex":11},{"imageOffset":7080,"symbol":"thread_start","symbolLocation":8,"imageIndex":11}]},{"id":21735219,"threadState":{"x":[{"value":268451845},{"value":21592279046},{"value":8589934592},{"value":672914001100800},{"value":0},{"value":672914001100800},{"value":2},{"value":4294967295},{"value":0},{"value":17179869184},{"value":0},{"value":2},{"value":0},{"value":0},{"value":156675},{"value":0},{"value":18446744073709551569},{"value":8819789984},{"value":0},{"value":4294967295},{"value":2},{"value":672914001100800},{"value":0},{"value":672914001100800},{"value":6201908952},{"value":8589934592},{"value":21592279046},{"value":18446744073709550527},{"value":4412409862}],"flavor":"ARM_THREAD_STATE64","lr":{"value":6968987688},"cpsr":{"value":0},"fp":{"value":6201908800},"sp":{"value":6201908720},"esr":{"value":1442840704,"description":"(Syscall)"},"pc":{"value":6968912948},"far":{"value":0}},"queue":"com.apple.root.user-interactive-qos","frames":[{"imageOffset":3124,"symbol":"mach_msg2_trap","symbolLocation":8,"imageIndex":12},{"imageOffset":77864,"symbol":"mach_msg2_internal","symbolLocation":76,"imageIndex":12},{"imageOffset":39308,"symbol":"mach_msg_overwrite","symbolLocation":484,"imageIndex":12},{"imageOffset":4020,"symbol":"mach_msg","symbolLocation":24,"imageIndex":12},{"imageOffset":392096,"symbol":"__CFRunLoopServiceMachPort","symbolLocation":160,"imageIndex":5},{"imageOffset":386296,"symbol":"__CFRunLoopRun","symbolLocation":1188,"imageIndex":5},{"imageOffset":1150516,"symbol":"_CFRunLoopRunSpecificWithOptions","symbolLocation":532,"imageIndex":5},{"imageOffset":10860900,"symbol":"-[NSRunLoop(NSRunLoop) runMode:beforeDate:]","symbolLocation":212,"imageIndex":15},{"imageOffset":5704700,"symbol":"-[NSAnimation _runBlocking]","symbolLocation":412,"imageIndex":7},{"imageOffset":7004,"symbol":"_dispatch_call_block_and_release","symbolLocation":32,"imageIndex":13},{"imageOffset":113364,"symbol":"_dispatch_client_callout","symbolLocation":16,"imageIndex":13},{"imageOffset":231900,"symbol":"<deduplicated_symbol>","symbolLocation":32,"imageIndex":13},{"imageOffset":82236,"symbol":"_dispatch_root_queue_drain","symbolLocation":736,"imageIndex":13},{"imageOffset":83844,"symbol":"_dispatch_worker_thread2","symbolLocation":180,"imageIndex":13},{"imageOffset":11792,"symbol":"_pthread_wqthread","symbolLocation":232,"imageIndex":11},{"imageOffset":7068,"symbol":"start_wqthread","symbolLocation":8,"imageIndex":11}]},{"id":21735220,"name":"notify-rs fsevents loop","threadState":{"x":[{"value":268451845},{"value":21592279046},{"value":8589934592},{"value":70381629079552},{"value":0},{"value":70381629079552},{"value":2},{"value":4294967295},{"value":0},{"value":17179869184},{"value":0},{"value":2},{"value":0},{"value":0},{"value":16387},{"value":0},{"value":18446744073709551569},{"value":18157},{"value":0},{"value":4294967295},{"value":2},{"value":70381629079552},{"value":0},{"value":70381629079552},{"value":6204054248},{"value":8589934592},{"value":21592279046},{"value":18446744073709550527},{"value":4412409862}],"flavor":"ARM_THREAD_STATE64","lr":{"value":6968987688},"cpsr":{"value":0},"fp":{"value":6204054096},"sp":{"value":6204054016},"esr":{"value":1442840704,"description":"(Syscall)"},"pc":{"value":6968912948},"far":{"value":0}},"frames":[{"imageOffset":3124,"symbol":"mach_msg2_trap","symbolLocation":8,"imageIndex":12},{"imageOffset":77864,"symbol":"mach_msg2_internal","symbolLocation":76,"imageIndex":12},{"imageOffset":39308,"symbol":"mach_msg_overwrite","symbolLocation":484,"imageIndex":12},{"imageOffset":4020,"symbol":"mach_msg","symbolLocation":24,"imageIndex":12},{"imageOffset":392096,"symbol":"__CFRunLoopServiceMachPort","symbolLocation":160,"imageIndex":5},{"imageOffset":386296,"symbol":"__CFRunLoopRun","symbolLocation":1188,"imageIndex":5},{"imageOffset":1150516,"symbol":"_CFRunLoopRunSpecificWithOptions","symbolLocation":532,"imageIndex":5},{"imageOffset":727616,"symbol":"CFRunLoopRun","symbolLocation":64,"imageIndex":5},{"imageOffset":81119868,"symbol":"notify::fsevent::FsEventWatcher::run::_$u7b$$u7b$closure$u7d$$u7d$::h115ce633794dd37e","symbolLocation":212,"imageIndex":0},{"imageOffset":81109264,"symbol":"std::sys::backtrace::__rust_begin_short_backtrace::h0e8136c158efde34","symbolLocation":16,"imageIndex":0},{"imageOffset":81075740,"symbol":"std::thread::Builder::spawn_unchecked_::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::had9e234c58a6b976","symbolLocation":116,"imageIndex":0},{"imageOffset":81127352,"symbol":"_$LT$core..panic..unwind_safe..AssertUnwindSafe$LT$F$GT$$u20$as$u20$core..ops..function..FnOnce$LT$$LP$$RP$$GT$$GT$::call_once::hd4eb61dc2f7423a1","symbolLocation":44,"imageIndex":0},{"imageOffset":81142176,"symbol":"std::panicking::catch_unwind::do_call::hb3c1c61f12e1b465","symbolLocation":68,"imageIndex":0},{"imageOffset":81080216,"symbol":"__rust_try","symbolLocation":32,"imageIndex":0},{"imageOffset":81075296,"symbol":"std::thread::Builder::spawn_unchecked_::_$u7b$$u7b$closure$u7d$$u7d$::hd5a9da3262eb8561","symbolLocation":728,"imageIndex":0},{"imageOffset":81142548,"symbol":"core::ops::function::FnOnce::call_once$u7b$$u7b$vtable.shim$u7d$$u7d$::hf49d8a5ef3bd1f1a","symbolLocation":24,"imageIndex":0},{"imageOffset":93722452,"symbol":"std::sys::pal::unix::thread::Thread::new::thread_start::h87df50f049a92661","symbolLocation":52,"imageIndex":0},{"imageOffset":27656,"symbol":"_pthread_start","symbolLocation":136,"imageIndex":11},{"imageOffset":7080,"symbol":"thread_start","symbolLocation":8,"imageIndex":11}]},{"id":21735222,"frames":[],"threadState":{"x":[{"value":6204633088},{"value":18179},{"value":6204096512},{"value":0},{"value":409604},{"value":18446744073709551615},{"value":0},{"value":0},{"value":0},{"value":0},{"value":0},{"value":0},{"value":0},{"value":0},{"value":0},{"value":0},{"value":0},{"value":0},{"value":0},{"value":0},{"value":0},{"value":0},{"value":0},{"value":0},{"value":0},{"value":0},{"value":0},{"value":0},{"value":0}],"flavor":"ARM_THREAD_STATE64","lr":{"value":0},"cpsr":{"value":0},"fp":{"value":0},"sp":{"value":6204633088},"esr":{"value":1442840704,"description":"(Syscall)"},"pc":{"value":6969166740},"far":{"value":0}}},{"id":21735236,"frames":[],"threadState":{"x":[{"value":6205206528},{"value":26899},{"value":6204669952},{"value":6205205376},{"value":5193734},{"value":1},{"value":0},{"value":0},{"value":0},{"value":0},{"value":0},{"value":0},{"value":0},{"value":0},{"value":0},{"value":0},{"value":0},{"value":0},{"value":0},{"value":0},{"value":0},{"value":0},{"value":0},{"value":0},{"value":0},{"value":0},{"value":0},{"value":0},{"value":0}],"flavor":"ARM_THREAD_STATE64","lr":{"value":0},"cpsr":{"value":0},"fp":{"value":0},"sp":{"value":6205205216},"esr":{"value":1442840704,"description":"(Syscall)"},"pc":{"value":6969166740},"far":{"value":0}}},{"id":21735237,"frames":[],"threadState":{"x":[{"value":6205779968},{"value":158259},{"value":6205243392},{"value":0},{"value":409604},{"value":18446744073709551615},{"value":0},{"value":0},{"value":0},{"value":0},{"value":0},{"value":0},{"value":0},{"value":0},{"value":0},{"value":0},{"value":0},{"value":0},{"value":0},{"value":0},{"value":0},{"value":0},{"value":0},{"value":0},{"value":0},{"value":0},{"value":0},{"value":0},{"value":0}],"flavor":"ARM_THREAD_STATE64","lr":{"value":0},"cpsr":{"value":0},"fp":{"value":0},"sp":{"value":6205779968},"esr":{"value":1442840704,"description":"(Syscall)"},"pc":{"value":6969166740},"far":{"value":0}}},{"id":21735238,"name":"com.apple.NSEventThread","threadState":{"x":[{"value":268451845},{"value":21592279046},{"value":8589934592},{"value":145148419768320},{"value":0},{"value":145148419768320},{"value":2},{"value":4294967295},{"value":0},{"value":17179869184},{"value":0},{"value":2},{"value":0},{"value":0},{"value":33795},{"value":0},{"value":18446744073709551569},{"value":8819789984},{"value":0},{"value":4294967295},{"value":2},{"value":145148419768320},{"value":0},{"value":145148419768320},{"value":6206349448},{"value":8589934592},{"value":21592279046},{"value":18446744073709550527},{"value":4412409862}],"flavor":"ARM_THREAD_STATE64","lr":{"value":6968987688},"cpsr":{"value":0},"fp":{"value":6206349296},"sp":{"value":6206349216},"esr":{"value":1442840704,"description":"(Syscall)"},"pc":{"value":6968912948},"far":{"value":0}},"frames":[{"imageOffset":3124,"symbol":"mach_msg2_trap","symbolLocation":8,"imageIndex":12},{"imageOffset":77864,"symbol":"mach_msg2_internal","symbolLocation":76,"imageIndex":12},{"imageOffset":39308,"symbol":"mach_msg_overwrite","symbolLocation":484,"imageIndex":12},{"imageOffset":4020,"symbol":"mach_msg","symbolLocation":24,"imageIndex":12},{"imageOffset":392096,"symbol":"__CFRunLoopServiceMachPort","symbolLocation":160,"imageIndex":5},{"imageOffset":386296,"symbol":"__CFRunLoopRun","symbolLocation":1188,"imageIndex":5},{"imageOffset":1150516,"symbol":"_CFRunLoopRunSpecificWithOptions","symbolLocation":532,"imageIndex":5},{"imageOffset":719412,"symbol":"_NSEventThread","symbolLocation":184,"imageIndex":7},{"imageOffset":27656,"symbol":"_pthread_start","symbolLocation":136,"imageIndex":11},{"imageOffset":7080,"symbol":"thread_start","symbolLocation":8,"imageIndex":11}]},{"id":21735243,"name":"caulk.messenger.shared:17","threadState":{"x":[{"value":14},{"value":53653668282},{"value":0},{"value":6206926954},{"value":53653668256},{"value":25},{"value":0},{"value":0},{"value":0},{"value":4294967295},{"value":0},{"value":0},{"value":0},{"value":0},{"value":0},{"value":0},{"value":18446744073709551580},{"value":8819792464},{"value":0},{"value":33396990976},{"value":0},{"value":0},{"value":0},{"value":0},{"value":0},{"value":0},{"value":0},{"value":0},{"value":0}],"flavor":"ARM_THREAD_STATE64","lr":{"value":7176896008},"cpsr":{"value":2147483648},"fp":{"value":6206926720},"sp":{"value":6206926688},"esr":{"value":1442840704,"description":"(Syscall)"},"pc":{"value":6968912816},"far":{"value":0}},"frames":[{"imageOffset":2992,"symbol":"semaphore_wait_trap","symbolLocation":8,"imageIndex":12},{"imageOffset":7688,"symbol":"caulk::semaphore::timed_wait(double)","symbolLocation":224,"imageIndex":16},{"imageOffset":7344,"symbol":"caulk::concurrent::details::worker_thread::run()","symbolLocation":32,"imageIndex":16},{"imageOffset":6480,"symbol":"void* caulk::thread_proxy<std::__1::tuple<caulk::thread::attributes, void (caulk::concurrent::details::worker_thread::*)(), std::__1::tuple<caulk::concurrent::details::worker_thread*>>>(void*)","symbolLocation":96,"imageIndex":16},{"imageOffset":27656,"symbol":"_pthread_start","symbolLocation":136,"imageIndex":11},{"imageOffset":7080,"symbol":"thread_start","symbolLocation":8,"imageIndex":11}]},{"id":21735244,"name":"caulk.messenger.shared:high","threadState":{"x":[{"value":14},{"value":64515},{"value":64515},{"value":15},{"value":4294967295},{"value":0},{"value":0},{"value":0},{"value":0},{"value":4294967295},{"value":1},{"value":33410494424},{"value":0},{"value":0},{"value":0},{"value":0},{"value":18446744073709551580},{"value":8819792464},{"value":0},{"value":33396991200},{"value":0},{"value":0},{"value":0},{"value":0},{"value":0},{"value":0},{"value":0},{"value":0},{"value":0}],"flavor":"ARM_THREAD_STATE64","lr":{"value":7176896008},"cpsr":{"value":2147483648},"fp":{"value":6207500160},"sp":{"value":6207500128},"esr":{"value":1442840704,"description":"(Syscall)"},"pc":{"value":6968912816},"far":{"value":0}},"frames":[{"imageOffset":2992,"symbol":"semaphore_wait_trap","symbolLocation":8,"imageIndex":12},{"imageOffset":7688,"symbol":"caulk::semaphore::timed_wait(double)","symbolLocation":224,"imageIndex":16},{"imageOffset":7344,"symbol":"caulk::concurrent::details::worker_thread::run()","symbolLocation":32,"imageIndex":16},{"imageOffset":6480,"symbol":"void* caulk::thread_proxy<std::__1::tuple<caulk::thread::attributes, void (caulk::concurrent::details::worker_thread::*)(), std::__1::tuple<caulk::concurrent::details::worker_thread*>>>(void*)","symbolLocation":96,"imageIndex":16},{"imageOffset":27656,"symbol":"_pthread_start","symbolLocation":136,"imageIndex":11},{"imageOffset":7080,"symbol":"thread_start","symbolLocation":8,"imageIndex":11}]},{"id":21735280,"name":"caulk::deferred_logger","threadState":{"x":[{"value":14},{"value":1},{"value":0},{"value":1},{"value":0},{"value":1},{"value":0},{"value":0},{"value":0},{"value":4294967295},{"value":0},{"value":0},{"value":4740612120},{"value":6208073400},{"value":16383},{"value":0},{"value":18446744073709551580},{"value":8819792464},{"value":0},{"value":33405321016},{"value":0},{"value":0},{"value":0},{"value":0},{"value":0},{"value":0},{"value":0},{"value":0},{"value":0}],"flavor":"ARM_THREAD_STATE64","lr":{"value":7176896008},"cpsr":{"value":2147483648},"fp":{"value":6208073600},"sp":{"value":6208073568},"esr":{"value":1442840704,"description":"(Syscall)"},"pc":{"value":6968912816},"far":{"value":0}},"frames":[{"imageOffset":2992,"symbol":"semaphore_wait_trap","symbolLocation":8,"imageIndex":12},{"imageOffset":7688,"symbol":"caulk::semaphore::timed_wait(double)","symbolLocation":224,"imageIndex":16},{"imageOffset":7344,"symbol":"caulk::concurrent::details::worker_thread::run()","symbolLocation":32,"imageIndex":16},{"imageOffset":6480,"symbol":"void* caulk::thread_proxy<std::__1::tuple<caulk::thread::attributes, void (caulk::concurrent::details::worker_thread::*)(), std::__1::tuple<caulk::concurrent::details::worker_thread*>>>(void*)","symbolLocation":96,"imageIndex":16},{"imageOffset":27656,"symbol":"_pthread_start","symbolLocation":136,"imageIndex":11},{"imageOffset":7080,"symbol":"thread_start","symbolLocation":8,"imageIndex":11}]},{"id":21735282,"name":"AudioSession - RootQueue","threadState":{"x":[{"value":14},{"value":4294967115611373572},{"value":999999958},{"value":68719460488},{"value":33400970944},{"value":7260746765},{"value":0},{"value":0},{"value":999999958},{"value":3},{"value":13835058055282163714},{"value":80000000},{"value":1354761538589157},{"value":1337167205059055},{"value":217088},{"value":26},{"value":18446744073709551578},{"value":8819792448},{"value":0},{"value":28307317715866},{"value":33404999232},{"value":1000000000},{"value":33404999096},{"value":6208647392},{"value":0},{"value":0},{"value":18446744071411073023},{"value":0},{"value":0}],"flavor":"ARM_THREAD_STATE64","lr":{"value":6967581836},"cpsr":{"value":2147483648},"fp":{"value":6208646976},"sp":{"value":6208646944},"esr":{"value":1442840704,"description":"(Syscall)"},"pc":{"value":6968912840},"far":{"value":0}},"frames":[{"imageOffset":3016,"symbol":"semaphore_timedwait_trap","symbolLocation":8,"imageIndex":12},{"imageOffset":224396,"symbol":"_dispatch_sema4_timedwait","symbolLocation":64,"imageIndex":13},{"imageOffset":16136,"symbol":"_dispatch_semaphore_wait_slow","symbolLocation":76,"imageIndex":13},{"imageOffset":81344,"symbol":"_dispatch_worker_thread","symbolLocation":324,"imageIndex":13},{"imageOffset":27656,"symbol":"_pthread_start","symbolLocation":136,"imageIndex":11},{"imageOffset":7080,"symbol":"thread_start","symbolLocation":8,"imageIndex":11}]},{"id":21735284,"name":"com.apple.audio.IOThread.client","threadState":{"x":[{"value":14},{"value":126979},{"value":0},{"value":0},{"value":0},{"value":24},{"value":4740694016},{"value":18446726482597246976},{"value":1},{"value":14886384472705597465},{"value":1099511628032},{"value":1099511628034},{"value":48},{"value":4096},{"value":0},{"value":256},{"value":18446744073709551579},{"value":8819792472},{"value":0},{"value":33397891384},{"value":33397891376},{"value":33397891408},{"value":1},{"value":33404299488},{"value":271},{"value":0},{"value":7027989464},{"value":33397891376},{"value":33397890560}],"flavor":"ARM_THREAD_STATE64","lr":{"value":7177015212},"cpsr":{"value":1610612736},"fp":{"value":6209219840},"sp":{"value":6209219824},"esr":{"value":1442840704,"description":"(Syscall)"},"pc":{"value":6968912828},"far":{"value":0}},"frames":[{"imageOffset":3004,"symbol":"semaphore_wait_signal_trap","symbolLocation":8,"imageIndex":12},{"imageOffset":126892,"symbol":"caulk::mach::semaphore::wait_signal_or_error(caulk::mach::semaphore&)","symbolLocation":36,"imageIndex":16},{"imageOffset":2123504,"symbol":"HALC_ProxyIOContext::IOWorkLoop()","symbolLocation":5052,"imageIndex":18},{"imageOffset":2116748,"symbol":"invocation function for block in HALC_ProxyIOContext::HALC_ProxyIOContext(unsigned int, unsigned int)","symbolLocation":172,"imageIndex":18},{"imageOffset":4007696,"symbol":"HALC_IOThread::Entry(void*)","symbolLocation":88,"imageIndex":18},{"imageOffset":27656,"symbol":"_pthread_start","symbolLocation":136,"imageIndex":11},{"imageOffset":7080,"symbol":"thread_start","symbolLocation":8,"imageIndex":11}]},{"id":21735302,"name":"gilrs","threadState":{"x":[{"value":268451845},{"value":21592279046},{"value":8589934592},{"value":536574559256576},{"value":0},{"value":536574559256576},{"value":2},{"value":4294967295},{"value":0},{"value":17179869184},{"value":0},{"value":2},{"value":0},{"value":0},{"value":124931},{"value":33407975424},{"value":18446744073709551569},{"value":18446744072367376383},{"value":0},{"value":4294967295},{"value":2},{"value":536574559256576},{"value":0},{"value":536574559256576},{"value":6211361176},{"value":8589934592},{"value":21592279046},{"value":18446744073709550527},{"value":4412409862}],"flavor":"ARM_THREAD_STATE64","lr":{"value":6968987688},"cpsr":{"value":0},"fp":{"value":6211361024},"sp":{"value":6211360944},"esr":{"value":1442840704,"description":"(Syscall)"},"pc":{"value":6968912948},"far":{"value":0}},"frames":[{"imageOffset":3124,"symbol":"mach_msg2_trap","symbolLocation":8,"imageIndex":12},{"imageOffset":77864,"symbol":"mach_msg2_internal","symbolLocation":76,"imageIndex":12},{"imageOffset":39308,"symbol":"mach_msg_overwrite","symbolLocation":484,"imageIndex":12},{"imageOffset":4020,"symbol":"mach_msg","symbolLocation":24,"imageIndex":12},{"imageOffset":392096,"symbol":"__CFRunLoopServiceMachPort","symbolLocation":160,"imageIndex":5},{"imageOffset":386296,"symbol":"__CFRunLoopRun","symbolLocation":1188,"imageIndex":5},{"imageOffset":1150516,"symbol":"_CFRunLoopRunSpecificWithOptions","symbolLocation":532,"imageIndex":5},{"imageOffset":727616,"symbol":"CFRunLoopRun","symbolLocation":64,"imageIndex":5},{"imageOffset":66255528,"symbol":"core_foundation::runloop::CFRunLoop::run_current::hff31c1552c5c8f29","symbolLocation":12,"imageIndex":0},{"imageOffset":45957144,"symbol":"gilrs_core::platform::platform::gamepad::Gilrs::spawn_thread::_$u7b$$u7b$closure$u7d$$u7d$::h817fe5c103cb72e3","symbolLocation":988,"imageIndex":0},{"imageOffset":45863516,"symbol":"std::sys::backtrace::__rust_begin_short_backtrace::h60c71449010a58d7","symbolLocation":16,"imageIndex":0},{"imageOffset":46018120,"symbol":"std::thread::Builder::spawn_unchecked_::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::h1d393ecf4a8e3432","symbolLocation":116,"imageIndex":0},{"imageOffset":45887260,"symbol":"_$LT$core..panic..unwind_safe..AssertUnwindSafe$LT$F$GT$$u20$as$u20$core..ops..function..FnOnce$LT$$LP$$RP$$GT$$GT$::call_once::hed43e0acd19cfce2","symbolLocation":44,"imageIndex":0},{"imageOffset":46018428,"symbol":"std::panicking::catch_unwind::do_call::h7982dc619b31e30c","symbolLocation":68,"imageIndex":0},{"imageOffset":46026660,"symbol":"__rust_try","symbolLocation":32,"imageIndex":0},{"imageOffset":46016924,"symbol":"std::thread::Builder::spawn_unchecked_::_$u7b$$u7b$closure$u7d$$u7d$::hea9e1044530aa965","symbolLocation":728,"imageIndex":0},{"imageOffset":45918652,"symbol":"core::ops::function::FnOnce::call_once$u7b$$u7b$vtable.shim$u7d$$u7d$::h62f8830b16be48a7","symbolLocation":24,"imageIndex":0},{"imageOffset":93722452,"symbol":"std::sys::pal::unix::thread::Thread::new::thread_start::h87df50f049a92661","symbolLocation":52,"imageIndex":0},{"imageOffset":27656,"symbol":"_pthread_start","symbolLocation":136,"imageIndex":11},{"imageOffset":7080,"symbol":"thread_start","symbolLocation":8,"imageIndex":11}]},{"id":21735303,"name":"gilrs","threadState":{"x":[{"value":4},{"value":0},{"value":1},{"value":1},{"value":0},{"value":49997125},{"value":52},{"value":0},{"value":8797032232,"symbolLocation":0,"symbol":"clock_sem"},{"value":16387},{"value":17},{"value":0},{"value":728924666},{"value":1000000000},{"value":18446744073708372147},{"value":33401503744},{"value":334},{"value":8819788224},{"value":0},{"value":6213507272},{"value":6213507272},{"value":33395179376},{"value":4439359408,"symbolLocation":6696,"symbol":"_$LT$bevy_gilrs..GilrsPlugin$u20$as$u20$bevy_app..plugin..Plugin$GT$::build::__CALLSITE::META::hc3ccffff1867f4d7"},{"value":0},{"value":0},{"value":0},{"value":0},{"value":0},{"value":0}],"flavor":"ARM_THREAD_STATE64","lr":{"value":6967717228},"cpsr":{"value":1610612736},"fp":{"value":6213507248},"sp":{"value":6213507200},"esr":{"value":1442840704,"description":"(Syscall)"},"pc":{"value":6968926964},"far":{"value":0}},"frames":[{"imageOffset":17140,"symbol":"__semwait_signal","symbolLocation":8,"imageIndex":12},{"imageOffset":56684,"symbol":"nanosleep","symbolLocation":220,"imageIndex":14},{"imageOffset":93673408,"symbol":"std::thread::sleep::h17057c3b27540418","symbolLocation":84,"imageIndex":0},{"imageOffset":45824508,"symbol":"gilrs::ff::server::run::h1f3ab47ccb1a3edd","symbolLocation":6888,"imageIndex":0},{"imageOffset":45825532,"symbol":"gilrs::ff::server::init::_$u7b$$u7b$closure$u7d$$u7d$::hafc8ef086dd20fa5","symbolLocation":32,"imageIndex":0},{"imageOffset":45730956,"symbol":"std::sys::backtrace::__rust_begin_short_backtrace::hb5b70ca1bdbfefbd","symbolLocation":16,"imageIndex":0},{"imageOffset":45708760,"symbol":"std::thread::Builder::spawn_unchecked_::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::h3434a3cc753a8765","symbolLocation":116,"imageIndex":0},{"imageOffset":45673316,"symbol":"_$LT$core..panic..unwind_safe..AssertUnwindSafe$LT$F$GT$$u20$as$u20$core..ops..function..FnOnce$LT$$LP$$RP$$GT$$GT$::call_once::h65cde94902cf8373","symbolLocation":44,"imageIndex":0},{"imageOffset":45756436,"symbol":"std::panicking::catch_unwind::do_call::h72fadfd76ed15e46","symbolLocation":68,"imageIndex":0},{"imageOffset":45716068,"symbol":"__rust_try","symbolLocation":32,"imageIndex":0},{"imageOffset":45708316,"symbol":"std::thread::Builder::spawn_unchecked_::_$u7b$$u7b$closure$u7d$$u7d$::h997c49fc8d6a6212","symbolLocation":728,"imageIndex":0},{"imageOffset":45778196,"symbol":"core::ops::function::FnOnce::call_once$u7b$$u7b$vtable.shim$u7d$$u7d$::h9bc8e158fa7b696b","symbolLocation":24,"imageIndex":0},{"imageOffset":93722452,"symbol":"std::sys::pal::unix::thread::Thread::new::thread_start::h87df50f049a92661","symbolLocation":52,"imageIndex":0},{"imageOffset":27656,"symbol":"_pthread_start","symbolLocation":136,"imageIndex":11},{"imageOffset":7080,"symbol":"thread_start","symbolLocation":8,"imageIndex":11}]},{"id":21735373,"name":"StackSamplingProfiler","threadState":{"x":[{"value":268451845},{"value":17179869442},{"value":0},{"value":526730494214144},{"value":0},{"value":526730494214144},{"value":32},{"value":74},{"value":0},{"value":17179869184},{"value":32},{"value":0},{"value":0},{"value":0},{"value":122639},{"value":32800},{"value":18446744073709551569},{"value":6222508032},{"value":0},{"value":74},{"value":32},{"value":526730494214144},{"value":0},{"value":526730494214144},{"value":6222507120},{"value":0},{"value":17179870466},{"value":18446744073709550527},{"value":1282}],"flavor":"ARM_THREAD_STATE64","lr":{"value":6968987688},"cpsr":{"value":0},"fp":{"value":6222506480},"sp":{"value":6222506400},"esr":{"value":1442840704,"description":"(Syscall)"},"pc":{"value":6968912948},"far":{"value":0}},"frames":[{"imageOffset":3124,"symbol":"mach_msg2_trap","symbolLocation":8,"imageIndex":12},{"imageOffset":77864,"symbol":"mach_msg2_internal","symbolLocation":76,"imageIndex":12},{"imageOffset":39308,"symbol":"mach_msg_overwrite","symbolLocation":484,"imageIndex":12},{"imageOffset":4020,"symbol":"mach_msg","symbolLocation":24,"imageIndex":12},{"imageOffset":75974648,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5493348,"imageIndex":3},{"imageOffset":75494920,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5013620,"imageIndex":3},{"imageOffset":75205056,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":4723756,"imageIndex":3},{"imageOffset":75640048,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5158748,"imageIndex":3},{"imageOffset":75385708,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":4904408,"imageIndex":3},{"imageOffset":75777340,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5296040,"imageIndex":3},{"imageOffset":75777736,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5296436,"imageIndex":3},{"imageOffset":75859632,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5378332,"imageIndex":3},{"imageOffset":27656,"symbol":"_pthread_start","symbolLocation":136,"imageIndex":11},{"imageOffset":7080,"symbol":"thread_start","symbolLocation":8,"imageIndex":11}]},{"id":21735376,"name":"HangWatcher","threadState":{"x":[{"value":268451845},{"value":17179869442},{"value":0},{"value":411402032381952},{"value":0},{"value":411402032381952},{"value":32},{"value":10000},{"value":0},{"value":17179869184},{"value":32},{"value":0},{"value":0},{"value":0},{"value":95787},{"value":8797120232,"symbolLocation":0,"symbol":"__CFConstantStringClassReference"},{"value":18446744073709551569},{"value":8819796768},{"value":0},{"value":10000},{"value":32},{"value":411402032381952},{"value":0},{"value":411402032381952},{"value":6230928880},{"value":0},{"value":17179870466},{"value":18446744073709550527},{"value":1282}],"flavor":"ARM_THREAD_STATE64","lr":{"value":6968987688},"cpsr":{"value":0},"fp":{"value":6230928240},"sp":{"value":6230928160},"esr":{"value":1442840704,"description":"(Syscall)"},"pc":{"value":6968912948},"far":{"value":0}},"frames":[{"imageOffset":3124,"symbol":"mach_msg2_trap","symbolLocation":8,"imageIndex":12},{"imageOffset":77864,"symbol":"mach_msg2_internal","symbolLocation":76,"imageIndex":12},{"imageOffset":39308,"symbol":"mach_msg_overwrite","symbolLocation":484,"imageIndex":12},{"imageOffset":4020,"symbol":"mach_msg","symbolLocation":24,"imageIndex":12},{"imageOffset":75974648,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5493348,"imageIndex":3},{"imageOffset":75494920,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5013620,"imageIndex":3},{"imageOffset":75752196,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5270896,"imageIndex":3},{"imageOffset":75752432,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5271132,"imageIndex":3},{"imageOffset":75859632,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5378332,"imageIndex":3},{"imageOffset":27656,"symbol":"_pthread_start","symbolLocation":136,"imageIndex":11},{"imageOffset":7080,"symbol":"thread_start","symbolLocation":8,"imageIndex":11}]},{"id":21735377,"name":"PerfettoTrace","threadState":{"x":[{"value":4},{"value":0},{"value":0},{"value":1271316498432},{"value":1},{"value":0},{"value":0},{"value":0},{"value":0},{"value":1},{"value":6239350560},{"value":1},{"value":1271310393408},{"value":1271310393416},{"value":0},{"value":168},{"value":369},{"value":6239350784},{"value":0},{"value":1271315172640},{"value":1271311437952},{"value":117251},{"value":0},{"value":1271311438216},{"value":1},{"value":12297829382473034411},{"value":1271316498432},{"value":65528},{"value":1}],"flavor":"ARM_THREAD_STATE64","lr":{"value":5089314248},"cpsr":{"value":2684354560},"fp":{"value":6239350272},"sp":{"value":6239350080},"esr":{"value":1442840704,"description":"(Syscall)"},"pc":{"value":6968961924},"far":{"value":0}},"frames":[{"imageOffset":52100,"symbol":"kevent64","symbolLocation":8,"imageIndex":12},{"imageOffset":76023240,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5541940,"imageIndex":3},{"imageOffset":75640048,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5158748,"imageIndex":3},{"imageOffset":75385708,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":4904408,"imageIndex":3},{"imageOffset":75777340,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5296040,"imageIndex":3},{"imageOffset":75777736,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5296436,"imageIndex":3},{"imageOffset":75859632,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5378332,"imageIndex":3},{"imageOffset":27656,"symbol":"_pthread_start","symbolLocation":136,"imageIndex":11},{"imageOffset":7080,"symbol":"thread_start","symbolLocation":8,"imageIndex":11}]},{"id":21735378,"name":"ThreadPoolServiceThread","threadState":{"x":[{"value":4},{"value":0},{"value":0},{"value":1288634969680},{"value":4},{"value":0},{"value":0},{"value":0},{"value":0},{"value":1},{"value":490},{"value":1179469521375},{"value":1271310393696},{"value":1271310393704},{"value":0},{"value":168},{"value":369},{"value":6247772160},{"value":0},{"value":1271315173408},{"value":1271311438336},{"value":8000},{"value":0},{"value":1271311438600},{"value":1},{"value":12297829382473034411},{"value":1288634969680},{"value":65528},{"value":1}],"flavor":"ARM_THREAD_STATE64","lr":{"value":5089314248},"cpsr":{"value":2684354560},"fp":{"value":6247771616},"sp":{"value":6247771424},"esr":{"value":1442840704,"description":"(Syscall)"},"pc":{"value":6968961924},"far":{"value":0}},"frames":[{"imageOffset":52100,"symbol":"kevent64","symbolLocation":8,"imageIndex":12},{"imageOffset":76023240,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5541940,"imageIndex":3},{"imageOffset":75640048,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5158748,"imageIndex":3},{"imageOffset":75385708,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":4904408,"imageIndex":3},{"imageOffset":75777340,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5296040,"imageIndex":3},{"imageOffset":75679332,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5198032,"imageIndex":3},{"imageOffset":75777736,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5296436,"imageIndex":3},{"imageOffset":75859632,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5378332,"imageIndex":3},{"imageOffset":27656,"symbol":"_pthread_start","symbolLocation":136,"imageIndex":11},{"imageOffset":7080,"symbol":"thread_start","symbolLocation":8,"imageIndex":11}]},{"id":21735379,"name":"ThreadPoolForegroundWorker","threadState":{"x":[{"value":268451845},{"value":17179869442},{"value":0},{"value":414528768573440},{"value":0},{"value":414528768573440},{"value":32},{"value":60262},{"value":0},{"value":17179869184},{"value":32},{"value":0},{"value":0},{"value":0},{"value":96515},{"value":5195174016},{"value":18446744073709551569},{"value":1271314850560},{"value":0},{"value":60262},{"value":32},{"value":414528768573440},{"value":0},{"value":414528768573440},{"value":6256192800},{"value":0},{"value":17179870466},{"value":18446744073709550527},{"value":1282}],"flavor":"ARM_THREAD_STATE64","lr":{"value":6968987688},"cpsr":{"value":0},"fp":{"value":6256192160},"sp":{"value":6256192080},"esr":{"value":1442840704,"description":"(Syscall)"},"pc":{"value":6968912948},"far":{"value":0}},"frames":[{"imageOffset":3124,"symbol":"mach_msg2_trap","symbolLocation":8,"imageIndex":12},{"imageOffset":77864,"symbol":"mach_msg2_internal","symbolLocation":76,"imageIndex":12},{"imageOffset":39308,"symbol":"mach_msg_overwrite","symbolLocation":484,"imageIndex":12},{"imageOffset":4020,"symbol":"mach_msg","symbolLocation":24,"imageIndex":12},{"imageOffset":75974648,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5493348,"imageIndex":3},{"imageOffset":75494920,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5013620,"imageIndex":3},{"imageOffset":75745048,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5263748,"imageIndex":3},{"imageOffset":75743784,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5262484,"imageIndex":3},{"imageOffset":75743488,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5262188,"imageIndex":3},{"imageOffset":75859632,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5378332,"imageIndex":3},{"imageOffset":27656,"symbol":"_pthread_start","symbolLocation":136,"imageIndex":11},{"imageOffset":7080,"symbol":"thread_start","symbolLocation":8,"imageIndex":11}]},{"id":21735380,"name":"ThreadPoolBackgroundWorker","threadState":{"x":[{"value":268451845},{"value":17179869442},{"value":0},{"value":501390187167744},{"value":0},{"value":501390187167744},{"value":32},{"value":60596},{"value":0},{"value":17179869184},{"value":32},{"value":0},{"value":0},{"value":0},{"value":116739},{"value":4294967295},{"value":18446744073709551569},{"value":1271314848768},{"value":0},{"value":60596},{"value":32},{"value":501390187167744},{"value":0},{"value":501390187167744},{"value":6264614176},{"value":0},{"value":17179870466},{"value":18446744073709550527},{"value":1282}],"flavor":"ARM_THREAD_STATE64","lr":{"value":6968987688},"cpsr":{"value":0},"fp":{"value":6264613536},"sp":{"value":6264613456},"esr":{"value":1442840704,"description":"(Syscall)"},"pc":{"value":6968912948},"far":{"value":0}},"frames":[{"imageOffset":3124,"symbol":"mach_msg2_trap","symbolLocation":8,"imageIndex":12},{"imageOffset":77864,"symbol":"mach_msg2_internal","symbolLocation":76,"imageIndex":12},{"imageOffset":39308,"symbol":"mach_msg_overwrite","symbolLocation":484,"imageIndex":12},{"imageOffset":4020,"symbol":"mach_msg","symbolLocation":24,"imageIndex":12},{"imageOffset":75974648,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5493348,"imageIndex":3},{"imageOffset":75494920,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5013620,"imageIndex":3},{"imageOffset":75745048,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5263748,"imageIndex":3},{"imageOffset":75743652,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5262352,"imageIndex":3},{"imageOffset":75743540,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5262240,"imageIndex":3},{"imageOffset":75859632,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5378332,"imageIndex":3},{"imageOffset":27656,"symbol":"_pthread_start","symbolLocation":136,"imageIndex":11},{"imageOffset":7080,"symbol":"thread_start","symbolLocation":8,"imageIndex":11}]},{"id":21735381,"name":"ThreadPoolForegroundWorker","threadState":{"x":[{"value":268451845},{"value":17179869442},{"value":0},{"value":498091652284416},{"value":0},{"value":498091652284416},{"value":32},{"value":60262},{"value":0},{"value":17179869184},{"value":32},{"value":0},{"value":0},{"value":0},{"value":115971},{"value":47472},{"value":18446744073709551569},{"value":1271314852352},{"value":0},{"value":60262},{"value":32},{"value":498091652284416},{"value":0},{"value":498091652284416},{"value":6273035552},{"value":0},{"value":17179870466},{"value":18446744073709550527},{"value":1282}],"flavor":"ARM_THREAD_STATE64","lr":{"value":6968987688},"cpsr":{"value":0},"fp":{"value":6273034912},"sp":{"value":6273034832},"esr":{"value":1442840704,"description":"(Syscall)"},"pc":{"value":6968912948},"far":{"value":0}},"frames":[{"imageOffset":3124,"symbol":"mach_msg2_trap","symbolLocation":8,"imageIndex":12},{"imageOffset":77864,"symbol":"mach_msg2_internal","symbolLocation":76,"imageIndex":12},{"imageOffset":39308,"symbol":"mach_msg_overwrite","symbolLocation":484,"imageIndex":12},{"imageOffset":4020,"symbol":"mach_msg","symbolLocation":24,"imageIndex":12},{"imageOffset":75974648,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5493348,"imageIndex":3},{"imageOffset":75494920,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5013620,"imageIndex":3},{"imageOffset":75745048,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5263748,"imageIndex":3},{"imageOffset":75743784,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5262484,"imageIndex":3},{"imageOffset":75743488,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5262188,"imageIndex":3},{"imageOffset":75859632,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5378332,"imageIndex":3},{"imageOffset":27656,"symbol":"_pthread_start","symbolLocation":136,"imageIndex":11},{"imageOffset":7080,"symbol":"thread_start","symbolLocation":8,"imageIndex":11}]},{"id":21735382,"name":"Chrome_IOThread","threadState":{"x":[{"value":4},{"value":0},{"value":0},{"value":1288634984656},{"value":3},{"value":0},{"value":0},{"value":0},{"value":0},{"value":1},{"value":6281457440},{"value":72057594037927935},{"value":16383},{"value":0},{"value":2},{"value":11},{"value":369},{"value":1271314854144},{"value":0},{"value":1271315171104},{"value":1271311440640},{"value":114691},{"value":0},{"value":1271311440904},{"value":1},{"value":12297829382473034411},{"value":1288634984656},{"value":65528},{"value":1}],"flavor":"ARM_THREAD_STATE64","lr":{"value":5089314248},"cpsr":{"value":2684354560},"fp":{"value":6281457104},"sp":{"value":6281456912},"esr":{"value":1442840704,"description":"(Syscall)"},"pc":{"value":6968961924},"far":{"value":0}},"frames":[{"imageOffset":52100,"symbol":"kevent64","symbolLocation":8,"imageIndex":12},{"imageOffset":76023240,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5541940,"imageIndex":3},{"imageOffset":75640048,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5158748,"imageIndex":3},{"imageOffset":75385708,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":4904408,"imageIndex":3},{"imageOffset":75777340,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5296040,"imageIndex":3},{"imageOffset":42141036,"symbol":"_v8_internal_Node_Print(void*)","symbolLocation":7818748,"imageIndex":3},{"imageOffset":75777736,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5296436,"imageIndex":3},{"imageOffset":75859632,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5378332,"imageIndex":3},{"imageOffset":27656,"symbol":"_pthread_start","symbolLocation":136,"imageIndex":11},{"imageOffset":7080,"symbol":"thread_start","symbolLocation":8,"imageIndex":11}]},{"id":21735383,"name":"MemoryInfra","threadState":{"x":[{"value":268451845},{"value":17179869442},{"value":0},{"value":488196047634432},{"value":0},{"value":488196047634432},{"value":32},{"value":15000},{"value":0},{"value":17179869184},{"value":32},{"value":0},{"value":0},{"value":0},{"value":113667},{"value":480},{"value":18446744073709551569},{"value":6289879040},{"value":0},{"value":15000},{"value":32},{"value":488196047634432},{"value":0},{"value":488196047634432},{"value":6289878128},{"value":0},{"value":17179870466},{"value":18446744073709550527},{"value":1282}],"flavor":"ARM_THREAD_STATE64","lr":{"value":6968987688},"cpsr":{"value":0},"fp":{"value":6289877488},"sp":{"value":6289877408},"esr":{"value":1442840704,"description":"(Syscall)"},"pc":{"value":6968912948},"far":{"value":0}},"frames":[{"imageOffset":3124,"symbol":"mach_msg2_trap","symbolLocation":8,"imageIndex":12},{"imageOffset":77864,"symbol":"mach_msg2_internal","symbolLocation":76,"imageIndex":12},{"imageOffset":39308,"symbol":"mach_msg_overwrite","symbolLocation":484,"imageIndex":12},{"imageOffset":4020,"symbol":"mach_msg","symbolLocation":24,"imageIndex":12},{"imageOffset":75974648,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5493348,"imageIndex":3},{"imageOffset":75494920,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5013620,"imageIndex":3},{"imageOffset":75205056,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":4723756,"imageIndex":3},{"imageOffset":75640048,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5158748,"imageIndex":3},{"imageOffset":75385708,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":4904408,"imageIndex":3},{"imageOffset":75777340,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5296040,"imageIndex":3},{"imageOffset":75777736,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5296436,"imageIndex":3},{"imageOffset":75859632,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5378332,"imageIndex":3},{"imageOffset":27656,"symbol":"_pthread_start","symbolLocation":136,"imageIndex":11},{"imageOffset":7080,"symbol":"thread_start","symbolLocation":8,"imageIndex":11}]},{"id":21735387,"name":"NetworkConfigWatcher","threadState":{"x":[{"value":268451845},{"value":21592279046},{"value":8589934592},{"value":433220466245632},{"value":208211448},{"value":433220466245632},{"value":2},{"value":4294967295},{"value":0},{"value":17179869184},{"value":0},{"value":2},{"value":0},{"value":0},{"value":100867},{"value":0},{"value":18446744073709551569},{"value":8819789984},{"value":0},{"value":4294967295},{"value":2},{"value":433220466245632},{"value":208211448},{"value":433220466245632},{"value":6298295816},{"value":8589934592},{"value":21592279046},{"value":18446744073709550527},{"value":4412409862}],"flavor":"ARM_THREAD_STATE64","lr":{"value":6968987688},"cpsr":{"value":0},"fp":{"value":6298295664},"sp":{"value":6298295584},"esr":{"value":1442840704,"description":"(Syscall)"},"pc":{"value":6968912948},"far":{"value":0}},"frames":[{"imageOffset":3124,"symbol":"mach_msg2_trap","symbolLocation":8,"imageIndex":12},{"imageOffset":77864,"symbol":"mach_msg2_internal","symbolLocation":76,"imageIndex":12},{"imageOffset":39308,"symbol":"mach_msg_overwrite","symbolLocation":484,"imageIndex":12},{"imageOffset":4020,"symbol":"mach_msg","symbolLocation":24,"imageIndex":12},{"imageOffset":392096,"symbol":"__CFRunLoopServiceMachPort","symbolLocation":160,"imageIndex":5},{"imageOffset":386296,"symbol":"__CFRunLoopRun","symbolLocation":1188,"imageIndex":5},{"imageOffset":1150516,"symbol":"_CFRunLoopRunSpecificWithOptions","symbolLocation":532,"imageIndex":5},{"imageOffset":10860900,"symbol":"-[NSRunLoop(NSRunLoop) runMode:beforeDate:]","symbolLocation":212,"imageIndex":15},{"imageOffset":75956912,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5475612,"imageIndex":3},{"imageOffset":75948404,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5467104,"imageIndex":3},{"imageOffset":75640048,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5158748,"imageIndex":3},{"imageOffset":75385708,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":4904408,"imageIndex":3},{"imageOffset":75777340,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5296040,"imageIndex":3},{"imageOffset":75777736,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5296436,"imageIndex":3},{"imageOffset":75859632,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5378332,"imageIndex":3},{"imageOffset":27656,"symbol":"_pthread_start","symbolLocation":136,"imageIndex":11},{"imageOffset":7080,"symbol":"thread_start","symbolLocation":8,"imageIndex":11}]},{"id":21735388,"name":"CrShutdownDetector","threadState":{"x":[{"value":4},{"value":0},{"value":4},{"value":6306721891},{"value":6306721224},{"value":18},{"value":0},{"value":0},{"value":18},{"value":8797038312,"symbolLocation":0,"symbol":"_current_pid"},{"value":8026668483491361347},{"value":2},{"value":16383},{"value":0},{"value":5196690501},{"value":0},{"value":3},{"value":8819796768},{"value":0},{"value":1288635037520},{"value":0},{"value":6306721612},{"value":4},{"value":0},{"value":0},{"value":0},{"value":0},{"value":0},{"value":0}],"flavor":"ARM_THREAD_STATE64","lr":{"value":5119443364},"cpsr":{"value":1610612736},"fp":{"value":6306721664},"sp":{"value":6306721296},"esr":{"value":1442840704,"description":"(Syscall)"},"pc":{"value":6968916232},"far":{"value":0}},"frames":[{"imageOffset":6408,"symbol":"read","symbolLocation":8,"imageIndex":12},{"imageOffset":106152356,"symbol":"rust_png$cxxbridge1$ResultOfWriter$operator$sizeof","symbolLocation":13197472,"imageIndex":3},{"imageOffset":75859632,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5378332,"imageIndex":3},{"imageOffset":27656,"symbol":"_pthread_start","symbolLocation":136,"imageIndex":11},{"imageOffset":7080,"symbol":"thread_start","symbolLocation":8,"imageIndex":11}]},{"id":21735392,"name":"NetworkConfigWatcher","threadState":{"x":[{"value":268451845},{"value":21592279046},{"value":8589934592},{"value":437618512756736},{"value":0},{"value":437618512756736},{"value":2},{"value":4294967295},{"value":0},{"value":17179869184},{"value":0},{"value":2},{"value":0},{"value":0},{"value":101891},{"value":0},{"value":18446744073709551569},{"value":8819789984},{"value":0},{"value":4294967295},{"value":2},{"value":437618512756736},{"value":0},{"value":437618512756736},{"value":6315138568},{"value":8589934592},{"value":21592279046},{"value":18446744073709550527},{"value":4412409862}],"flavor":"ARM_THREAD_STATE64","lr":{"value":6968987688},"cpsr":{"value":0},"fp":{"value":6315138416},"sp":{"value":6315138336},"esr":{"value":1442840704,"description":"(Syscall)"},"pc":{"value":6968912948},"far":{"value":0}},"frames":[{"imageOffset":3124,"symbol":"mach_msg2_trap","symbolLocation":8,"imageIndex":12},{"imageOffset":77864,"symbol":"mach_msg2_internal","symbolLocation":76,"imageIndex":12},{"imageOffset":39308,"symbol":"mach_msg_overwrite","symbolLocation":484,"imageIndex":12},{"imageOffset":4020,"symbol":"mach_msg","symbolLocation":24,"imageIndex":12},{"imageOffset":392096,"symbol":"__CFRunLoopServiceMachPort","symbolLocation":160,"imageIndex":5},{"imageOffset":386296,"symbol":"__CFRunLoopRun","symbolLocation":1188,"imageIndex":5},{"imageOffset":1150516,"symbol":"_CFRunLoopRunSpecificWithOptions","symbolLocation":532,"imageIndex":5},{"imageOffset":10860900,"symbol":"-[NSRunLoop(NSRunLoop) runMode:beforeDate:]","symbolLocation":212,"imageIndex":15},{"imageOffset":75956912,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5475612,"imageIndex":3},{"imageOffset":75948404,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5467104,"imageIndex":3},{"imageOffset":75640048,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5158748,"imageIndex":3},{"imageOffset":75385708,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":4904408,"imageIndex":3},{"imageOffset":75777340,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5296040,"imageIndex":3},{"imageOffset":75777736,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5296436,"imageIndex":3},{"imageOffset":75859632,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5378332,"imageIndex":3},{"imageOffset":27656,"symbol":"_pthread_start","symbolLocation":136,"imageIndex":11},{"imageOffset":7080,"symbol":"thread_start","symbolLocation":8,"imageIndex":11}]},{"id":21735393,"name":"ThreadPoolForegroundWorker","threadState":{"x":[{"value":54},{"value":0},{"value":0},{"value":68719460488},{"value":18446744073709537280},{"value":32},{"value":49},{"value":0},{"value":4294967168},{"value":1288652698912},{"value":160},{"value":85761906986498},{"value":85761906986498},{"value":85761906986496},{"value":1288637270016},{"value":18488},{"value":5},{"value":1288637982464},{"value":0},{"value":0},{"value":1288652698912},{"value":6323559944},{"value":0},{"value":0},{"value":384},{"value":6323561679},{"value":96},{"value":1},{"value":10}],"flavor":"ARM_THREAD_STATE64","lr":{"value":6968961228},"cpsr":{"value":2147483648},"fp":{"value":6323559632},"sp":{"value":6323559584},"esr":{"value":1442840704,"description":"(Syscall)"},"pc":{"value":6968915588},"far":{"value":0}},"frames":[{"imageOffset":5764,"symbol":"__open","symbolLocation":8,"imageIndex":12},{"imageOffset":51404,"symbol":"open","symbolLocation":64,"imageIndex":12},{"imageOffset":75854768,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5373468,"imageIndex":3},{"imageOffset":75138232,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":4656932,"imageIndex":3},{"imageOffset":75138420,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":4657120,"imageIndex":3},{"imageOffset":95090832,"symbol":"rust_png$cxxbridge1$ResultOfWriter$operator$sizeof","symbolLocation":2135948,"imageIndex":3},{"imageOffset":94959388,"symbol":"rust_png$cxxbridge1$ResultOfWriter$operator$sizeof","symbolLocation":2004504,"imageIndex":3},{"imageOffset":95036596,"symbol":"rust_png$cxxbridge1$ResultOfWriter$operator$sizeof","symbolLocation":2081712,"imageIndex":3},{"imageOffset":94981072,"symbol":"rust_png$cxxbridge1$ResultOfWriter$operator$sizeof","symbolLocation":2026188,"imageIndex":3},{"imageOffset":94996472,"symbol":"rust_png$cxxbridge1$ResultOfWriter$operator$sizeof","symbolLocation":2041588,"imageIndex":3},{"imageOffset":94964356,"symbol":"rust_png$cxxbridge1$ResultOfWriter$operator$sizeof","symbolLocation":2009472,"imageIndex":3},{"imageOffset":94964820,"symbol":"rust_png$cxxbridge1$ResultOfWriter$operator$sizeof","symbolLocation":2009936,"imageIndex":3},{"imageOffset":94891320,"symbol":"rust_png$cxxbridge1$ResultOfWriter$operator$sizeof","symbolLocation":1936436,"imageIndex":3},{"imageOffset":94897128,"symbol":"rust_png$cxxbridge1$ResultOfWriter$operator$sizeof","symbolLocation":1942244,"imageIndex":3},{"imageOffset":7281100,"symbol":"fontations_ffi$cxxbridge1$bitmap_metrics","symbolLocation":175016,"imageIndex":3},{"imageOffset":75662972,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5181672,"imageIndex":3},{"imageOffset":75506944,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5025644,"imageIndex":3},{"imageOffset":75676192,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5194892,"imageIndex":3},{"imageOffset":75673916,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5192616,"imageIndex":3},{"imageOffset":75744564,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5263264,"imageIndex":3},{"imageOffset":75743784,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5262484,"imageIndex":3},{"imageOffset":75743488,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5262188,"imageIndex":3},{"imageOffset":75859632,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5378332,"imageIndex":3},{"imageOffset":27656,"symbol":"_pthread_start","symbolLocation":136,"imageIndex":11},{"imageOffset":7080,"symbol":"thread_start","symbolLocation":8,"imageIndex":11}]},{"id":21735394,"name":"ThreadPoolForegroundWorker","threadState":{"x":[{"value":268451845},{"value":17179869442},{"value":0},{"value":473902396473344},{"value":0},{"value":473902396473344},{"value":32},{"value":60263},{"value":0},{"value":17179869184},{"value":32},{"value":0},{"value":0},{"value":0},{"value":110339},{"value":4194304},{"value":18446744073709551569},{"value":1288637984128},{"value":0},{"value":60263},{"value":32},{"value":473902396473344},{"value":0},{"value":473902396473344},{"value":6331985184},{"value":0},{"value":17179870466},{"value":18446744073709550527},{"value":1282}],"flavor":"ARM_THREAD_STATE64","lr":{"value":6968987688},"cpsr":{"value":0},"fp":{"value":6331984544},"sp":{"value":6331984464},"esr":{"value":1442840704,"description":"(Syscall)"},"pc":{"value":6968912948},"far":{"value":0}},"frames":[{"imageOffset":3124,"symbol":"mach_msg2_trap","symbolLocation":8,"imageIndex":12},{"imageOffset":77864,"symbol":"mach_msg2_internal","symbolLocation":76,"imageIndex":12},{"imageOffset":39308,"symbol":"mach_msg_overwrite","symbolLocation":484,"imageIndex":12},{"imageOffset":4020,"symbol":"mach_msg","symbolLocation":24,"imageIndex":12},{"imageOffset":75974648,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5493348,"imageIndex":3},{"imageOffset":75494920,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5013620,"imageIndex":3},{"imageOffset":75745048,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5263748,"imageIndex":3},{"imageOffset":75743784,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5262484,"imageIndex":3},{"imageOffset":75743488,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5262188,"imageIndex":3},{"imageOffset":75859632,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5378332,"imageIndex":3},{"imageOffset":27656,"symbol":"_pthread_start","symbolLocation":136,"imageIndex":11},{"imageOffset":7080,"symbol":"thread_start","symbolLocation":8,"imageIndex":11}]},{"id":21735395,"name":"ThreadPoolForegroundWorker","threadState":{"x":[{"value":268451845},{"value":17179869442},{"value":0},{"value":439817536012288},{"value":0},{"value":439817536012288},{"value":32},{"value":60262},{"value":0},{"value":17179869184},{"value":32},{"value":0},{"value":0},{"value":0},{"value":102403},{"value":12464},{"value":18446744073709551569},{"value":1288637985792},{"value":0},{"value":60262},{"value":32},{"value":439817536012288},{"value":0},{"value":439817536012288},{"value":6340406560},{"value":0},{"value":17179870466},{"value":18446744073709550527},{"value":1282}],"flavor":"ARM_THREAD_STATE64","lr":{"value":6968987688},"cpsr":{"value":0},"fp":{"value":6340405920},"sp":{"value":6340405840},"esr":{"value":1442840704,"description":"(Syscall)"},"pc":{"value":6968912948},"far":{"value":0}},"frames":[{"imageOffset":3124,"symbol":"mach_msg2_trap","symbolLocation":8,"imageIndex":12},{"imageOffset":77864,"symbol":"mach_msg2_internal","symbolLocation":76,"imageIndex":12},{"imageOffset":39308,"symbol":"mach_msg_overwrite","symbolLocation":484,"imageIndex":12},{"imageOffset":4020,"symbol":"mach_msg","symbolLocation":24,"imageIndex":12},{"imageOffset":75974648,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5493348,"imageIndex":3},{"imageOffset":75494920,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5013620,"imageIndex":3},{"imageOffset":75745048,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5263748,"imageIndex":3},{"imageOffset":75743784,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5262484,"imageIndex":3},{"imageOffset":75743488,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5262188,"imageIndex":3},{"imageOffset":75859632,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5378332,"imageIndex":3},{"imageOffset":27656,"symbol":"_pthread_start","symbolLocation":136,"imageIndex":11},{"imageOffset":7080,"symbol":"thread_start","symbolLocation":8,"imageIndex":11}]},{"id":21735396,"name":"ThreadPoolForegroundWorker","threadState":{"x":[{"value":1288683738176},{"value":1288684721240},{"value":6},{"value":32},{"value":110},{"value":100},{"value":6348826199},{"value":60264},{"value":0},{"value":0},{"value":8746382381183561574},{"value":1},{"value":2147483649},{"value":524288},{"value":0},{"value":32},{"value":6969216720,"symbolLocation":0,"symbol":"_platform_memcmp"},{"value":6348828672},{"value":0},{"value":1288683738176},{"value":1288685022336},{"value":1288684762688},{"value":1288684762688},{"value":1288684721280},{"value":1288684762688},{"value":4057},{"value":1288683779648},{"value":1},{"value":10}],"flavor":"ARM_THREAD_STATE64","lr":{"value":5077607540},"cpsr":{"value":1610612736},"fp":{"value":6348826368},"sp":{"value":6348826320},"esr":{"value":2449473607,"description":"(Data Abort) byte write Translation fault"},"pc":{"value":5077607492},"far":{"value":0}},"frames":[{"imageOffset":64316484,"symbol":"_v8_internal_Node_Print(void*)","symbolLocation":29994196,"imageIndex":3},{"imageOffset":64315496,"symbol":"_v8_internal_Node_Print(void*)","symbolLocation":29993208,"imageIndex":3},{"imageOffset":64315144,"symbol":"_v8_internal_Node_Print(void*)","symbolLocation":29992856,"imageIndex":3},{"imageOffset":64314912,"symbol":"_v8_internal_Node_Print(void*)","symbolLocation":29992624,"imageIndex":3},{"imageOffset":64313524,"symbol":"_v8_internal_Node_Print(void*)","symbolLocation":29991236,"imageIndex":3},{"imageOffset":64320140,"symbol":"_v8_internal_Node_Print(void*)","symbolLocation":29997852,"imageIndex":3},{"imageOffset":75662972,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5181672,"imageIndex":3},{"imageOffset":75506944,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5025644,"imageIndex":3},{"imageOffset":75676416,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5195116,"imageIndex":3},{"imageOffset":75673412,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5192112,"imageIndex":3},{"imageOffset":75744564,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5263264,"imageIndex":3},{"imageOffset":75743784,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5262484,"imageIndex":3},{"imageOffset":75743488,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5262188,"imageIndex":3},{"imageOffset":75859632,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5378332,"imageIndex":3},{"imageOffset":27656,"symbol":"_pthread_start","symbolLocation":136,"imageIndex":11},{"imageOffset":7080,"symbol":"thread_start","symbolLocation":8,"imageIndex":11}]},{"id":21735398,"name":"NetworkNotificationThreadMac","threadState":{"x":[{"value":268451845},{"value":21592279046},{"value":8589934592},{"value":448613629034496},{"value":0},{"value":448613629034496},{"value":2},{"value":4294967295},{"value":0},{"value":17179869184},{"value":0},{"value":2},{"value":0},{"value":0},{"value":104451},{"value":0},{"value":18446744073709551569},{"value":8819789984},{"value":0},{"value":4294967295},{"value":2},{"value":448613629034496},{"value":0},{"value":448613629034496},{"value":6357245448},{"value":8589934592},{"value":21592279046},{"value":18446744073709550527},{"value":4412409862}],"flavor":"ARM_THREAD_STATE64","lr":{"value":6968987688},"cpsr":{"value":0},"fp":{"value":6357245296},"sp":{"value":6357245216},"esr":{"value":1442840704,"description":"(Syscall)"},"pc":{"value":6968912948},"far":{"value":0}},"frames":[{"imageOffset":3124,"symbol":"mach_msg2_trap","symbolLocation":8,"imageIndex":12},{"imageOffset":77864,"symbol":"mach_msg2_internal","symbolLocation":76,"imageIndex":12},{"imageOffset":39308,"symbol":"mach_msg_overwrite","symbolLocation":484,"imageIndex":12},{"imageOffset":4020,"symbol":"mach_msg","symbolLocation":24,"imageIndex":12},{"imageOffset":392096,"symbol":"__CFRunLoopServiceMachPort","symbolLocation":160,"imageIndex":5},{"imageOffset":386296,"symbol":"__CFRunLoopRun","symbolLocation":1188,"imageIndex":5},{"imageOffset":1150516,"symbol":"_CFRunLoopRunSpecificWithOptions","symbolLocation":532,"imageIndex":5},{"imageOffset":10860900,"symbol":"-[NSRunLoop(NSRunLoop) runMode:beforeDate:]","symbolLocation":212,"imageIndex":15},{"imageOffset":75956912,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5475612,"imageIndex":3},{"imageOffset":75948404,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5467104,"imageIndex":3},{"imageOffset":75640048,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5158748,"imageIndex":3},{"imageOffset":75385708,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":4904408,"imageIndex":3},{"imageOffset":75777340,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5296040,"imageIndex":3},{"imageOffset":75777736,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5296436,"imageIndex":3},{"imageOffset":75859632,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5378332,"imageIndex":3},{"imageOffset":27656,"symbol":"_pthread_start","symbolLocation":136,"imageIndex":11},{"imageOffset":7080,"symbol":"thread_start","symbolLocation":8,"imageIndex":11}]},{"id":21735399,"name":"CompositorTileWorker1","threadState":{"x":[{"value":260},{"value":0},{"value":0},{"value":0},{"value":0},{"value":161},{"value":0},{"value":0},{"value":6365671000},{"value":0},{"value":0},{"value":2},{"value":2},{"value":0},{"value":0},{"value":0},{"value":305},{"value":8819788104},{"value":0},{"value":1288638896152},{"value":1288638896280},{"value":6365671648},{"value":0},{"value":0},{"value":0},{"value":1},{"value":256},{"value":0},{"value":0}],"flavor":"ARM_THREAD_STATE64","lr":{"value":6969188572},"cpsr":{"value":1610612736},"fp":{"value":6365671120},"sp":{"value":6365670976},"esr":{"value":1442840704,"description":"(Syscall)"},"pc":{"value":6968927480},"far":{"value":0}},"frames":[{"imageOffset":17656,"symbol":"__psynch_cvwait","symbolLocation":8,"imageIndex":12},{"imageOffset":28892,"symbol":"_pthread_cond_wait","symbolLocation":984,"imageIndex":11},{"imageOffset":75857540,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5376240,"imageIndex":3},{"imageOffset":95647560,"symbol":"rust_png$cxxbridge1$ResultOfWriter$operator$sizeof","symbolLocation":2692676,"imageIndex":3},{"imageOffset":75859632,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5378332,"imageIndex":3},{"imageOffset":27656,"symbol":"_pthread_start","symbolLocation":136,"imageIndex":11},{"imageOffset":7080,"symbol":"thread_start","symbolLocation":8,"imageIndex":11}]},{"id":21735400,"name":"ThreadPoolSingleThreadForegroundBlocking0","threadState":{"x":[{"value":268451845},{"value":17179869442},{"value":0},{"value":748780303417344},{"value":0},{"value":748780303417344},{"value":32},{"value":60263},{"value":0},{"value":17179869184},{"value":32},{"value":0},{"value":0},{"value":0},{"value":174339},{"value":116288},{"value":18446744073709551569},{"value":1288637992448},{"value":0},{"value":60263},{"value":32},{"value":748780303417344},{"value":0},{"value":748780303417344},{"value":6374092064},{"value":0},{"value":17179870466},{"value":18446744073709550527},{"value":1282}],"flavor":"ARM_THREAD_STATE64","lr":{"value":6968987688},"cpsr":{"value":0},"fp":{"value":6374091424},"sp":{"value":6374091344},"esr":{"value":1442840704,"description":"(Syscall)"},"pc":{"value":6968912948},"far":{"value":0}},"frames":[{"imageOffset":3124,"symbol":"mach_msg2_trap","symbolLocation":8,"imageIndex":12},{"imageOffset":77864,"symbol":"mach_msg2_internal","symbolLocation":76,"imageIndex":12},{"imageOffset":39308,"symbol":"mach_msg_overwrite","symbolLocation":484,"imageIndex":12},{"imageOffset":4020,"symbol":"mach_msg","symbolLocation":24,"imageIndex":12},{"imageOffset":75974648,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5493348,"imageIndex":3},{"imageOffset":75494920,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5013620,"imageIndex":3},{"imageOffset":75745048,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5263748,"imageIndex":3},{"imageOffset":75743872,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5262572,"imageIndex":3},{"imageOffset":75743580,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5262280,"imageIndex":3},{"imageOffset":75859632,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5378332,"imageIndex":3},{"imageOffset":27656,"symbol":"_pthread_start","symbolLocation":136,"imageIndex":11},{"imageOffset":7080,"symbol":"thread_start","symbolLocation":8,"imageIndex":11}]},{"id":21735410,"name":"ThreadPoolSingleThreadSharedBackgroundBlocking1","threadState":{"x":[{"value":268451845},{"value":17179869442},{"value":0},{"value":573013061795840},{"value":0},{"value":573013061795840},{"value":32},{"value":60500},{"value":0},{"value":17179869184},{"value":32},{"value":0},{"value":0},{"value":0},{"value":133415},{"value":4294967294},{"value":18446744073709551569},{"value":8819796768},{"value":0},{"value":60500},{"value":32},{"value":573013061795840},{"value":0},{"value":573013061795840},{"value":6382513440},{"value":0},{"value":17179870466},{"value":18446744073709550527},{"value":1282}],"flavor":"ARM_THREAD_STATE64","lr":{"value":6968987688},"cpsr":{"value":0},"fp":{"value":6382512800},"sp":{"value":6382512720},"esr":{"value":1442840704,"description":"(Syscall)"},"pc":{"value":6968912948},"far":{"value":0}},"frames":[{"imageOffset":3124,"symbol":"mach_msg2_trap","symbolLocation":8,"imageIndex":12},{"imageOffset":77864,"symbol":"mach_msg2_internal","symbolLocation":76,"imageIndex":12},{"imageOffset":39308,"symbol":"mach_msg_overwrite","symbolLocation":484,"imageIndex":12},{"imageOffset":4020,"symbol":"mach_msg","symbolLocation":24,"imageIndex":12},{"imageOffset":75974648,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5493348,"imageIndex":3},{"imageOffset":75494920,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5013620,"imageIndex":3},{"imageOffset":75745048,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5263748,"imageIndex":3},{"imageOffset":75743696,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5262396,"imageIndex":3},{"imageOffset":75743600,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5262300,"imageIndex":3},{"imageOffset":75859632,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5378332,"imageIndex":3},{"imageOffset":27656,"symbol":"_pthread_start","symbolLocation":136,"imageIndex":11},{"imageOffset":7080,"symbol":"thread_start","symbolLocation":8,"imageIndex":11}]},{"id":21735413,"name":"NetworkConfigWatcher","threadState":{"x":[{"value":268451845},{"value":21592279046},{"value":8589934592},{"value":722392024350720},{"value":0},{"value":722392024350720},{"value":2},{"value":4294967295},{"value":0},{"value":17179869184},{"value":0},{"value":2},{"value":0},{"value":0},{"value":168195},{"value":0},{"value":18446744073709551569},{"value":8819789984},{"value":0},{"value":4294967295},{"value":2},{"value":722392024350720},{"value":0},{"value":722392024350720},{"value":6390930952},{"value":8589934592},{"value":21592279046},{"value":18446744073709550527},{"value":4412409862}],"flavor":"ARM_THREAD_STATE64","lr":{"value":6968987688},"cpsr":{"value":0},"fp":{"value":6390930800},"sp":{"value":6390930720},"esr":{"value":1442840704,"description":"(Syscall)"},"pc":{"value":6968912948},"far":{"value":0}},"frames":[{"imageOffset":3124,"symbol":"mach_msg2_trap","symbolLocation":8,"imageIndex":12},{"imageOffset":77864,"symbol":"mach_msg2_internal","symbolLocation":76,"imageIndex":12},{"imageOffset":39308,"symbol":"mach_msg_overwrite","symbolLocation":484,"imageIndex":12},{"imageOffset":4020,"symbol":"mach_msg","symbolLocation":24,"imageIndex":12},{"imageOffset":392096,"symbol":"__CFRunLoopServiceMachPort","symbolLocation":160,"imageIndex":5},{"imageOffset":386296,"symbol":"__CFRunLoopRun","symbolLocation":1188,"imageIndex":5},{"imageOffset":1150516,"symbol":"_CFRunLoopRunSpecificWithOptions","symbolLocation":532,"imageIndex":5},{"imageOffset":10860900,"symbol":"-[NSRunLoop(NSRunLoop) runMode:beforeDate:]","symbolLocation":212,"imageIndex":15},{"imageOffset":75956912,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5475612,"imageIndex":3},{"imageOffset":75948404,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5467104,"imageIndex":3},{"imageOffset":75640048,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5158748,"imageIndex":3},{"imageOffset":75385708,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":4904408,"imageIndex":3},{"imageOffset":75777340,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5296040,"imageIndex":3},{"imageOffset":75777736,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5296436,"imageIndex":3},{"imageOffset":75859632,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5378332,"imageIndex":3},{"imageOffset":27656,"symbol":"_pthread_start","symbolLocation":136,"imageIndex":11},{"imageOffset":7080,"symbol":"thread_start","symbolLocation":8,"imageIndex":11}]},{"id":21735414,"name":"ThreadPoolSingleThreadSharedForeground2","threadState":{"x":[{"value":268451845},{"value":17179869442},{"value":0},{"value":580623743844352},{"value":0},{"value":580623743844352},{"value":32},{"value":60494},{"value":0},{"value":17179869184},{"value":32},{"value":0},{"value":0},{"value":0},{"value":135187},{"value":2291039184},{"value":18446744073709551569},{"value":1288637997440},{"value":0},{"value":60494},{"value":32},{"value":580623743844352},{"value":0},{"value":580623743844352},{"value":6399356192},{"value":0},{"value":17179870466},{"value":18446744073709550527},{"value":1282}],"flavor":"ARM_THREAD_STATE64","lr":{"value":6968987688},"cpsr":{"value":0},"fp":{"value":6399355552},"sp":{"value":6399355472},"esr":{"value":1442840704,"description":"(Syscall)"},"pc":{"value":6968912948},"far":{"value":0}},"frames":[{"imageOffset":3124,"symbol":"mach_msg2_trap","symbolLocation":8,"imageIndex":12},{"imageOffset":77864,"symbol":"mach_msg2_internal","symbolLocation":76,"imageIndex":12},{"imageOffset":39308,"symbol":"mach_msg_overwrite","symbolLocation":484,"imageIndex":12},{"imageOffset":4020,"symbol":"mach_msg","symbolLocation":24,"imageIndex":12},{"imageOffset":75974648,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5493348,"imageIndex":3},{"imageOffset":75494920,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5013620,"imageIndex":3},{"imageOffset":75745048,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5263748,"imageIndex":3},{"imageOffset":75743828,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5262528,"imageIndex":3},{"imageOffset":75743560,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5262260,"imageIndex":3},{"imageOffset":75859632,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5378332,"imageIndex":3},{"imageOffset":27656,"symbol":"_pthread_start","symbolLocation":136,"imageIndex":11},{"imageOffset":7080,"symbol":"thread_start","symbolLocation":8,"imageIndex":11}]},{"id":21735415,"name":"ThreadPoolForegroundWorker","threadState":{"x":[{"value":268451845},{"value":17179869442},{"value":0},{"value":581671715864576},{"value":0},{"value":581671715864576},{"value":32},{"value":60263},{"value":0},{"value":17179869184},{"value":32},{"value":0},{"value":0},{"value":0},{"value":135431},{"value":256},{"value":18446744073709551569},{"value":1288638009088},{"value":0},{"value":60263},{"value":32},{"value":581671715864576},{"value":0},{"value":581671715864576},{"value":6407777568},{"value":0},{"value":17179870466},{"value":18446744073709550527},{"value":1282}],"flavor":"ARM_THREAD_STATE64","lr":{"value":6968987688},"cpsr":{"value":0},"fp":{"value":6407776928},"sp":{"value":6407776848},"esr":{"value":1442840704,"description":"(Syscall)"},"pc":{"value":6968912948},"far":{"value":0}},"frames":[{"imageOffset":3124,"symbol":"mach_msg2_trap","symbolLocation":8,"imageIndex":12},{"imageOffset":77864,"symbol":"mach_msg2_internal","symbolLocation":76,"imageIndex":12},{"imageOffset":39308,"symbol":"mach_msg_overwrite","symbolLocation":484,"imageIndex":12},{"imageOffset":4020,"symbol":"mach_msg","symbolLocation":24,"imageIndex":12},{"imageOffset":75974648,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5493348,"imageIndex":3},{"imageOffset":75494920,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5013620,"imageIndex":3},{"imageOffset":75745048,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5263748,"imageIndex":3},{"imageOffset":75743784,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5262484,"imageIndex":3},{"imageOffset":75743488,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5262188,"imageIndex":3},{"imageOffset":75859632,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5378332,"imageIndex":3},{"imageOffset":27656,"symbol":"_pthread_start","symbolLocation":136,"imageIndex":11},{"imageOffset":7080,"symbol":"thread_start","symbolLocation":8,"imageIndex":11}]},{"id":21735416,"name":"NetworkConfigWatcher","threadState":{"x":[{"value":268451845},{"value":21592279046},{"value":8589934592},{"value":713595931328512},{"value":0},{"value":713595931328512},{"value":2},{"value":4294967295},{"value":0},{"value":17179869184},{"value":0},{"value":2},{"value":0},{"value":0},{"value":166147},{"value":0},{"value":18446744073709551569},{"value":8819789984},{"value":0},{"value":4294967295},{"value":2},{"value":713595931328512},{"value":0},{"value":713595931328512},{"value":6416195080},{"value":8589934592},{"value":21592279046},{"value":18446744073709550527},{"value":4412409862}],"flavor":"ARM_THREAD_STATE64","lr":{"value":6968987688},"cpsr":{"value":0},"fp":{"value":6416194928},"sp":{"value":6416194848},"esr":{"value":1442840704,"description":"(Syscall)"},"pc":{"value":6968912948},"far":{"value":0}},"frames":[{"imageOffset":3124,"symbol":"mach_msg2_trap","symbolLocation":8,"imageIndex":12},{"imageOffset":77864,"symbol":"mach_msg2_internal","symbolLocation":76,"imageIndex":12},{"imageOffset":39308,"symbol":"mach_msg_overwrite","symbolLocation":484,"imageIndex":12},{"imageOffset":4020,"symbol":"mach_msg","symbolLocation":24,"imageIndex":12},{"imageOffset":392096,"symbol":"__CFRunLoopServiceMachPort","symbolLocation":160,"imageIndex":5},{"imageOffset":386296,"symbol":"__CFRunLoopRun","symbolLocation":1188,"imageIndex":5},{"imageOffset":1150516,"symbol":"_CFRunLoopRunSpecificWithOptions","symbolLocation":532,"imageIndex":5},{"imageOffset":10860900,"symbol":"-[NSRunLoop(NSRunLoop) runMode:beforeDate:]","symbolLocation":212,"imageIndex":15},{"imageOffset":75956912,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5475612,"imageIndex":3},{"imageOffset":75948404,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5467104,"imageIndex":3},{"imageOffset":75640048,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5158748,"imageIndex":3},{"imageOffset":75385708,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":4904408,"imageIndex":3},{"imageOffset":75777340,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5296040,"imageIndex":3},{"imageOffset":75777736,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5296436,"imageIndex":3},{"imageOffset":75859632,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5378332,"imageIndex":3},{"imageOffset":27656,"symbol":"_pthread_start","symbolLocation":136,"imageIndex":11},{"imageOffset":7080,"symbol":"thread_start","symbolLocation":8,"imageIndex":11}]},{"id":21735421,"name":"ThreadPoolForegroundWorker","threadState":{"x":[{"value":268451845},{"value":17179869442},{"value":0},{"value":710348936052736},{"value":0},{"value":710348936052736},{"value":32},{"value":60264},{"value":0},{"value":17179869184},{"value":32},{"value":0},{"value":0},{"value":0},{"value":165391},{"value":10080},{"value":18446744073709551569},{"value":1288638019072},{"value":0},{"value":60264},{"value":32},{"value":710348936052736},{"value":0},{"value":710348936052736},{"value":6424620320},{"value":0},{"value":17179870466},{"value":18446744073709550527},{"value":1282}],"flavor":"ARM_THREAD_STATE64","lr":{"value":6968987688},"cpsr":{"value":0},"fp":{"value":6424619680},"sp":{"value":6424619600},"esr":{"value":1442840704,"description":"(Syscall)"},"pc":{"value":6968912948},"far":{"value":0}},"frames":[{"imageOffset":3124,"symbol":"mach_msg2_trap","symbolLocation":8,"imageIndex":12},{"imageOffset":77864,"symbol":"mach_msg2_internal","symbolLocation":76,"imageIndex":12},{"imageOffset":39308,"symbol":"mach_msg_overwrite","symbolLocation":484,"imageIndex":12},{"imageOffset":4020,"symbol":"mach_msg","symbolLocation":24,"imageIndex":12},{"imageOffset":75974648,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5493348,"imageIndex":3},{"imageOffset":75494920,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5013620,"imageIndex":3},{"imageOffset":75745048,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5263748,"imageIndex":3},{"imageOffset":75743784,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5262484,"imageIndex":3},{"imageOffset":75743488,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5262188,"imageIndex":3},{"imageOffset":75859632,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5378332,"imageIndex":3},{"imageOffset":27656,"symbol":"_pthread_start","symbolLocation":136,"imageIndex":11},{"imageOffset":7080,"symbol":"thread_start","symbolLocation":8,"imageIndex":11}]},{"id":21735422,"name":"ThreadPoolForegroundWorker","threadState":{"x":[{"value":268451845},{"value":17179869442},{"value":0},{"value":709197884817408},{"value":0},{"value":709197884817408},{"value":32},{"value":60264},{"value":0},{"value":17179869184},{"value":32},{"value":0},{"value":0},{"value":0},{"value":165123},{"value":11264},{"value":18446744073709551569},{"value":1288638017408},{"value":0},{"value":60264},{"value":32},{"value":709197884817408},{"value":0},{"value":709197884817408},{"value":6433041696},{"value":0},{"value":17179870466},{"value":18446744073709550527},{"value":1282}],"flavor":"ARM_THREAD_STATE64","lr":{"value":6968987688},"cpsr":{"value":0},"fp":{"value":6433041056},"sp":{"value":6433040976},"esr":{"value":1442840704,"description":"(Syscall)"},"pc":{"value":6968912948},"far":{"value":0}},"frames":[{"imageOffset":3124,"symbol":"mach_msg2_trap","symbolLocation":8,"imageIndex":12},{"imageOffset":77864,"symbol":"mach_msg2_internal","symbolLocation":76,"imageIndex":12},{"imageOffset":39308,"symbol":"mach_msg_overwrite","symbolLocation":484,"imageIndex":12},{"imageOffset":4020,"symbol":"mach_msg","symbolLocation":24,"imageIndex":12},{"imageOffset":75974648,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5493348,"imageIndex":3},{"imageOffset":75494920,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5013620,"imageIndex":3},{"imageOffset":75745048,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5263748,"imageIndex":3},{"imageOffset":75743784,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5262484,"imageIndex":3},{"imageOffset":75743488,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5262188,"imageIndex":3},{"imageOffset":75859632,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5378332,"imageIndex":3},{"imageOffset":27656,"symbol":"_pthread_start","symbolLocation":136,"imageIndex":11},{"imageOffset":7080,"symbol":"thread_start","symbolLocation":8,"imageIndex":11}]},{"id":21735423,"name":"ThreadPoolForegroundWorker","threadState":{"x":[{"value":268451845},{"value":17179869442},{"value":0},{"value":588251605762048},{"value":0},{"value":588251605762048},{"value":32},{"value":60265},{"value":0},{"value":17179869184},{"value":32},{"value":0},{"value":0},{"value":0},{"value":136963},{"value":256},{"value":18446744073709551569},{"value":1288638020736},{"value":0},{"value":60265},{"value":32},{"value":588251605762048},{"value":0},{"value":588251605762048},{"value":6441463072},{"value":0},{"value":17179870466},{"value":18446744073709550527},{"value":1282}],"flavor":"ARM_THREAD_STATE64","lr":{"value":6968987688},"cpsr":{"value":0},"fp":{"value":6441462432},"sp":{"value":6441462352},"esr":{"value":1442840704,"description":"(Syscall)"},"pc":{"value":6968912948},"far":{"value":0}},"frames":[{"imageOffset":3124,"symbol":"mach_msg2_trap","symbolLocation":8,"imageIndex":12},{"imageOffset":77864,"symbol":"mach_msg2_internal","symbolLocation":76,"imageIndex":12},{"imageOffset":39308,"symbol":"mach_msg_overwrite","symbolLocation":484,"imageIndex":12},{"imageOffset":4020,"symbol":"mach_msg","symbolLocation":24,"imageIndex":12},{"imageOffset":75974648,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5493348,"imageIndex":3},{"imageOffset":75494920,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5013620,"imageIndex":3},{"imageOffset":75745048,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5263748,"imageIndex":3},{"imageOffset":75743784,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5262484,"imageIndex":3},{"imageOffset":75743488,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5262188,"imageIndex":3},{"imageOffset":75859632,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5378332,"imageIndex":3},{"imageOffset":27656,"symbol":"_pthread_start","symbolLocation":136,"imageIndex":11},{"imageOffset":7080,"symbol":"thread_start","symbolLocation":8,"imageIndex":11}]},{"id":21735424,"name":"ThreadPoolForegroundWorker","threadState":{"x":[{"value":268451845},{"value":17179869442},{"value":0},{"value":706998861561856},{"value":0},{"value":706998861561856},{"value":32},{"value":60265},{"value":0},{"value":17179869184},{"value":32},{"value":0},{"value":0},{"value":0},{"value":164611},{"value":4194304},{"value":18446744073709551569},{"value":1288638022400},{"value":0},{"value":60265},{"value":32},{"value":706998861561856},{"value":0},{"value":706998861561856},{"value":12893302048},{"value":0},{"value":17179870466},{"value":18446744073709550527},{"value":1282}],"flavor":"ARM_THREAD_STATE64","lr":{"value":6968987688},"cpsr":{"value":0},"fp":{"value":12893301408},"sp":{"value":12893301328},"esr":{"value":1442840704,"description":"(Syscall)"},"pc":{"value":6968912948},"far":{"value":0}},"frames":[{"imageOffset":3124,"symbol":"mach_msg2_trap","symbolLocation":8,"imageIndex":12},{"imageOffset":77864,"symbol":"mach_msg2_internal","symbolLocation":76,"imageIndex":12},{"imageOffset":39308,"symbol":"mach_msg_overwrite","symbolLocation":484,"imageIndex":12},{"imageOffset":4020,"symbol":"mach_msg","symbolLocation":24,"imageIndex":12},{"imageOffset":75974648,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5493348,"imageIndex":3},{"imageOffset":75494920,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5013620,"imageIndex":3},{"imageOffset":75745048,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5263748,"imageIndex":3},{"imageOffset":75743784,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5262484,"imageIndex":3},{"imageOffset":75743488,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5262188,"imageIndex":3},{"imageOffset":75859632,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5378332,"imageIndex":3},{"imageOffset":27656,"symbol":"_pthread_start","symbolLocation":136,"imageIndex":11},{"imageOffset":7080,"symbol":"thread_start","symbolLocation":8,"imageIndex":11}]},{"id":21735425,"name":"ThreadPoolForegroundWorker","threadState":{"x":[{"value":268451845},{"value":17179869442},{"value":0},{"value":590450629017600},{"value":0},{"value":590450629017600},{"value":32},{"value":60265},{"value":0},{"value":17179869184},{"value":32},{"value":0},{"value":0},{"value":0},{"value":137475},{"value":4194304},{"value":18446744073709551569},{"value":1288638025728},{"value":0},{"value":60265},{"value":32},{"value":590450629017600},{"value":0},{"value":590450629017600},{"value":12901723424},{"value":0},{"value":17179870466},{"value":18446744073709550527},{"value":1282}],"flavor":"ARM_THREAD_STATE64","lr":{"value":6968987688},"cpsr":{"value":0},"fp":{"value":12901722784},"sp":{"value":12901722704},"esr":{"value":1442840704,"description":"(Syscall)"},"pc":{"value":6968912948},"far":{"value":0}},"frames":[{"imageOffset":3124,"symbol":"mach_msg2_trap","symbolLocation":8,"imageIndex":12},{"imageOffset":77864,"symbol":"mach_msg2_internal","symbolLocation":76,"imageIndex":12},{"imageOffset":39308,"symbol":"mach_msg_overwrite","symbolLocation":484,"imageIndex":12},{"imageOffset":4020,"symbol":"mach_msg","symbolLocation":24,"imageIndex":12},{"imageOffset":75974648,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5493348,"imageIndex":3},{"imageOffset":75494920,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5013620,"imageIndex":3},{"imageOffset":75745048,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5263748,"imageIndex":3},{"imageOffset":75743784,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5262484,"imageIndex":3},{"imageOffset":75743488,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5262188,"imageIndex":3},{"imageOffset":75859632,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5378332,"imageIndex":3},{"imageOffset":27656,"symbol":"_pthread_start","symbolLocation":136,"imageIndex":11},{"imageOffset":7080,"symbol":"thread_start","symbolLocation":8,"imageIndex":11}]},{"id":21735426,"name":"ThreadPoolForegroundWorker","threadState":{"x":[{"value":268451845},{"value":17179869442},{"value":0},{"value":704799838306304},{"value":0},{"value":704799838306304},{"value":32},{"value":60478},{"value":0},{"value":17179869184},{"value":32},{"value":0},{"value":0},{"value":0},{"value":164099},{"value":20536},{"value":18446744073709551569},{"value":1288638024064},{"value":0},{"value":60478},{"value":32},{"value":704799838306304},{"value":0},{"value":704799838306304},{"value":12910144800},{"value":0},{"value":17179870466},{"value":18446744073709550527},{"value":1282}],"flavor":"ARM_THREAD_STATE64","lr":{"value":6968987688},"cpsr":{"value":0},"fp":{"value":12910144160},"sp":{"value":12910144080},"esr":{"value":1442840704,"description":"(Syscall)"},"pc":{"value":6968912948},"far":{"value":0}},"frames":[{"imageOffset":3124,"symbol":"mach_msg2_trap","symbolLocation":8,"imageIndex":12},{"imageOffset":77864,"symbol":"mach_msg2_internal","symbolLocation":76,"imageIndex":12},{"imageOffset":39308,"symbol":"mach_msg_overwrite","symbolLocation":484,"imageIndex":12},{"imageOffset":4020,"symbol":"mach_msg","symbolLocation":24,"imageIndex":12},{"imageOffset":75974648,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5493348,"imageIndex":3},{"imageOffset":75494920,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5013620,"imageIndex":3},{"imageOffset":75745048,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5263748,"imageIndex":3},{"imageOffset":75743784,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5262484,"imageIndex":3},{"imageOffset":75743488,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5262188,"imageIndex":3},{"imageOffset":75859632,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5378332,"imageIndex":3},{"imageOffset":27656,"symbol":"_pthread_start","symbolLocation":136,"imageIndex":11},{"imageOffset":7080,"symbol":"thread_start","symbolLocation":8,"imageIndex":11}]},{"id":21735427,"name":"ThreadPoolForegroundWorker","threadState":{"x":[{"value":268451845},{"value":17179869442},{"value":0},{"value":591550140645376},{"value":0},{"value":591550140645376},{"value":32},{"value":60478},{"value":0},{"value":17179869184},{"value":32},{"value":0},{"value":0},{"value":0},{"value":137731},{"value":22584},{"value":18446744073709551569},{"value":1288638027392},{"value":0},{"value":60478},{"value":32},{"value":591550140645376},{"value":0},{"value":591550140645376},{"value":12918566176},{"value":0},{"value":17179870466},{"value":18446744073709550527},{"value":1282}],"flavor":"ARM_THREAD_STATE64","lr":{"value":6968987688},"cpsr":{"value":0},"fp":{"value":12918565536},"sp":{"value":12918565456},"esr":{"value":1442840704,"description":"(Syscall)"},"pc":{"value":6968912948},"far":{"value":0}},"frames":[{"imageOffset":3124,"symbol":"mach_msg2_trap","symbolLocation":8,"imageIndex":12},{"imageOffset":77864,"symbol":"mach_msg2_internal","symbolLocation":76,"imageIndex":12},{"imageOffset":39308,"symbol":"mach_msg_overwrite","symbolLocation":484,"imageIndex":12},{"imageOffset":4020,"symbol":"mach_msg","symbolLocation":24,"imageIndex":12},{"imageOffset":75974648,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5493348,"imageIndex":3},{"imageOffset":75494920,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5013620,"imageIndex":3},{"imageOffset":75745048,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5263748,"imageIndex":3},{"imageOffset":75743784,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5262484,"imageIndex":3},{"imageOffset":75743488,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5262188,"imageIndex":3},{"imageOffset":75859632,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5378332,"imageIndex":3},{"imageOffset":27656,"symbol":"_pthread_start","symbolLocation":136,"imageIndex":11},{"imageOffset":7080,"symbol":"thread_start","symbolLocation":8,"imageIndex":11}]},{"id":21735428,"name":"ThreadPoolForegroundWorker","threadState":{"x":[{"value":268451845},{"value":17179869442},{"value":0},{"value":592649652273152},{"value":0},{"value":592649652273152},{"value":32},{"value":60479},{"value":0},{"value":17179869184},{"value":32},{"value":0},{"value":0},{"value":0},{"value":137987},{"value":28600},{"value":18446744073709551569},{"value":1288638029056},{"value":0},{"value":60479},{"value":32},{"value":592649652273152},{"value":0},{"value":592649652273152},{"value":12926987552},{"value":0},{"value":17179870466},{"value":18446744073709550527},{"value":1282}],"flavor":"ARM_THREAD_STATE64","lr":{"value":6968987688},"cpsr":{"value":0},"fp":{"value":12926986912},"sp":{"value":12926986832},"esr":{"value":1442840704,"description":"(Syscall)"},"pc":{"value":6968912948},"far":{"value":0}},"frames":[{"imageOffset":3124,"symbol":"mach_msg2_trap","symbolLocation":8,"imageIndex":12},{"imageOffset":77864,"symbol":"mach_msg2_internal","symbolLocation":76,"imageIndex":12},{"imageOffset":39308,"symbol":"mach_msg_overwrite","symbolLocation":484,"imageIndex":12},{"imageOffset":4020,"symbol":"mach_msg","symbolLocation":24,"imageIndex":12},{"imageOffset":75974648,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5493348,"imageIndex":3},{"imageOffset":75494920,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5013620,"imageIndex":3},{"imageOffset":75745048,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5263748,"imageIndex":3},{"imageOffset":75743784,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5262484,"imageIndex":3},{"imageOffset":75743488,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5262188,"imageIndex":3},{"imageOffset":75859632,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5378332,"imageIndex":3},{"imageOffset":27656,"symbol":"_pthread_start","symbolLocation":136,"imageIndex":11},{"imageOffset":7080,"symbol":"thread_start","symbolLocation":8,"imageIndex":11}]},{"id":21735429,"name":"ThreadPoolForegroundWorker","threadState":{"x":[{"value":268451845},{"value":17179869442},{"value":0},{"value":699302280167424},{"value":0},{"value":699302280167424},{"value":32},{"value":60478},{"value":0},{"value":17179869184},{"value":32},{"value":0},{"value":0},{"value":0},{"value":162819},{"value":28088},{"value":18446744073709551569},{"value":1288638030720},{"value":0},{"value":60478},{"value":32},{"value":699302280167424},{"value":0},{"value":699302280167424},{"value":12935408928},{"value":0},{"value":17179870466},{"value":18446744073709550527},{"value":1282}],"flavor":"ARM_THREAD_STATE64","lr":{"value":6968987688},"cpsr":{"value":0},"fp":{"value":12935408288},"sp":{"value":12935408208},"esr":{"value":1442840704,"description":"(Syscall)"},"pc":{"value":6968912948},"far":{"value":0}},"frames":[{"imageOffset":3124,"symbol":"mach_msg2_trap","symbolLocation":8,"imageIndex":12},{"imageOffset":77864,"symbol":"mach_msg2_internal","symbolLocation":76,"imageIndex":12},{"imageOffset":39308,"symbol":"mach_msg_overwrite","symbolLocation":484,"imageIndex":12},{"imageOffset":4020,"symbol":"mach_msg","symbolLocation":24,"imageIndex":12},{"imageOffset":75974648,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5493348,"imageIndex":3},{"imageOffset":75494920,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5013620,"imageIndex":3},{"imageOffset":75745048,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5263748,"imageIndex":3},{"imageOffset":75743784,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5262484,"imageIndex":3},{"imageOffset":75743488,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5262188,"imageIndex":3},{"imageOffset":75859632,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5378332,"imageIndex":3},{"imageOffset":27656,"symbol":"_pthread_start","symbolLocation":136,"imageIndex":11},{"imageOffset":7080,"symbol":"thread_start","symbolLocation":8,"imageIndex":11}]},{"id":21735430,"name":"ThreadPoolSingleThreadSharedForegroundBlocking3","threadState":{"x":[{"value":268451845},{"value":17179869442},{"value":0},{"value":698202768539648},{"value":0},{"value":698202768539648},{"value":32},{"value":60478},{"value":0},{"value":17179869184},{"value":32},{"value":0},{"value":0},{"value":0},{"value":162563},{"value":4294967294},{"value":18446744073709551569},{"value":1288638034048},{"value":0},{"value":60478},{"value":32},{"value":698202768539648},{"value":0},{"value":698202768539648},{"value":12943830304},{"value":0},{"value":17179870466},{"value":18446744073709550527},{"value":1282}],"flavor":"ARM_THREAD_STATE64","lr":{"value":6968987688},"cpsr":{"value":0},"fp":{"value":12943829664},"sp":{"value":12943829584},"esr":{"value":1442840704,"description":"(Syscall)"},"pc":{"value":6968912948},"far":{"value":0}},"frames":[{"imageOffset":3124,"symbol":"mach_msg2_trap","symbolLocation":8,"imageIndex":12},{"imageOffset":77864,"symbol":"mach_msg2_internal","symbolLocation":76,"imageIndex":12},{"imageOffset":39308,"symbol":"mach_msg_overwrite","symbolLocation":484,"imageIndex":12},{"imageOffset":4020,"symbol":"mach_msg","symbolLocation":24,"imageIndex":12},{"imageOffset":75974648,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5493348,"imageIndex":3},{"imageOffset":75494920,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5013620,"imageIndex":3},{"imageOffset":75745048,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5263748,"imageIndex":3},{"imageOffset":75743828,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5262528,"imageIndex":3},{"imageOffset":75743560,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5262260,"imageIndex":3},{"imageOffset":75859632,"symbol":"ChromeWebAppShortcutCopierMain","symbolLocation":5378332,"imageIndex":3},{"imageOffset":27656,"symbol":"_pthread_start","symbolLocation":136,"imageIndex":11},{"imageOffset":7080,"symbol":"thread_start","symbolLocation":8,"imageIndex":11}]},{"id":21735453,"frames":[{"imageOffset":17656,"symbol":"__psynch_cvwait","symbolLocation":8,"imageIndex":12},{"imageOffset":28892,"symbol":"_pthread_cond_wait","symbolLocation":984,"imageIndex":11},{"imageOffset":90249648,"symbol":"std::sys::sync::condvar::pthread::Condvar::wait::h71a06353eafcdf14","symbolLocation":184,"imageIndex":0},{"imageOffset":90251612,"symbol":"std::sync::poison::condvar::Condvar::wait::hf2c637a3647cef57","symbolLocation":56,"imageIndex":0},{"imageOffset":90263856,"symbol":"parking::Inner::park::h7e586f5f04c2b074","symbolLocation":716,"imageIndex":0},{"imageOffset":90262844,"symbol":"parking::Parker::park::h30fbb16b0ec84050","symbolLocation":40,"imageIndex":0},{"imageOffset":57003632,"symbol":"async_io::driver::block_on::_$u7b$$u7b$closure$u7d$$u7d$::h3a4fbe6d982a1d9e","symbolLocation":960,"imageIndex":0},{"imageOffset":53750276,"symbol":"std::thread::local::LocalKey$LT$T$GT$::try_with::hd7b4350d7b271be6","symbolLocation":228,"imageIndex":0},{"imageOffset":53738600,"symbol":"std::thread::local::LocalKey$LT$T$GT$::with::h8d2bf8976c42a5e7","symbolLocation":32,"imageIndex":0},{"imageOffset":56994924,"symbol":"async_io::driver::block_on::hbf9223aafc64ab9d","symbolLocation":156,"imageIndex":0},{"imageOffset":53959372,"symbol":"bevy_tasks::task_pool::TaskPool::scope_with_executor_inner::h36dbfcdcd9161d50","symbolLocation":332,"imageIndex":0},{"imageOffset":53979392,"symbol":"bevy_tasks::task_pool::TaskPool::scope::_$u7b$$u7b$closure$u7d$$u7d$::h5013b8dea351ad83","symbolLocation":156,"imageIndex":0},{"imageOffset":53748976,"symbol":"std::thread::local::LocalKey$LT$T$GT$::try_with::hae5713f6c3a744c9","symbolLocation":196,"imageIndex":0},{"imageOffset":53739616,"symbol":"std::thread::local::LocalKey$LT$T$GT$::with::he666816c6ab6f028","symbolLocation":48,"imageIndex":0},{"imageOffset":53979224,"symbol":"bevy_tasks::task_pool::TaskPool::scope::h4784166cccba7333","symbolLocation":52,"imageIndex":0},{"imageOffset":56433252,"symbol":"_$LT$bevy_render..pipelined_rendering..PipelinedRenderingPlugin$u20$as$u20$bevy_app..plugin..Plugin$GT$::cleanup::_$u7b$$u7b$closure$u7d$$u7d$::h769fc96171fe7e08","symbolLocation":172,"imageIndex":0},{"imageOffset":53040116,"symbol":"std::sys::backtrace::__rust_begin_short_backtrace::ha8cf03e9c582ebf9","symbolLocation":16,"imageIndex":0},{"imageOffset":56859848,"symbol":"std::thread::Builder::spawn_unchecked_::_$u7b$$u7b$closure$u7d$$u7d$::_$u7b$$u7b$closure$u7d$$u7d$::hacbba7c1c51f5df7","symbolLocation":116,"imageIndex":0},{"imageOffset":57124432,"symbol":"_$LT$core..panic..unwind_safe..AssertUnwindSafe$LT$F$GT$$u20$as$u20$core..ops..function..FnOnce$LT$$LP$$RP$$GT$$GT$::call_once::heae5d561cd8f14d3","symbolLocation":44,"imageIndex":0},{"imageOffset":57152288,"symbol":"std::panicking::catch_unwind::do_call::h25b68353837fbe24","symbolLocation":68,"imageIndex":0},{"imageOffset":57094628,"symbol":"__rust_try","symbolLocation":32,"imageIndex":0},{"imageOffset":56858984,"symbol":"std::thread::Builder::spawn_unchecked_::_$u7b$$u7b$closure$u7d$$u7d$::h6484971cd2c10037","symbolLocation":728,"imageIndex":0},{"imageOffset":55666644,"symbol":"core::ops::function::FnOnce::call_once$u7b$$u7b$vtable.shim$u7d$$u7d$::hf1e2892413594782","symbolLocation":24,"imageIndex":0},{"imageOffset":93722452,"symbol":"std::sys::pal::unix::thread::Thread::new::thread_start::h87df50f049a92661","symbolLocation":52,"imageIndex":0},{"imageOffset":27656,"symbol":"_pthread_start","symbolLocation":136,"imageIndex":11},{"imageOffset":7080,"symbol":"thread_start","symbolLocation":8,"imageIndex":11}],"threadState":{"x":[{"value":260},{"value":0},{"value":0},{"value":0},{"value":0},{"value":160},{"value":0},{"value":0},{"value":12945959592},{"value":0},{"value":0},{"value":2},{"value":2},{"value":0},{"value":0},{"value":0},{"value":305},{"value":8819788104},{"value":0},{"value":1288648236176},{"value":1288653691968},{"value":12945977568},{"value":0},{"value":0},{"value":0},{"value":1},{"value":256},{"value":0},{"value":0}],"flavor":"ARM_THREAD_STATE64","lr":{"value":6969188572},"cpsr":{"value":1610612736},"fp":{"value":12945959712},"sp":{"value":12945959568},"esr":{"value":1442840704,"description":"(Syscall)"},"pc":{"value":6968927480},"far":{"value":0}}}],
  "usedImages" : [
  {
    "source" : "P",
    "arch" : "arm64",
    "base" : 4297719808,
    "size" : 139362304,
    "uuid" : "f3844b84-1750-321e-b2de-be823904d28b",
    "path" : "*\/simple",
    "name" : "simple"
  },
  {
    "source" : "P",
    "arch" : "arm64e",
    "base" : 4759584768,
    "CFBundleShortVersionString" : "342.3",
    "CFBundleIdentifier" : "com.apple.AGXMetalG16X",
    "size" : 8568832,
    "uuid" : "f3e5018e-3b97-3b77-837f-18455b13e98f",
    "path" : "\/System\/Library\/Extensions\/AGXMetalG16X.bundle\/Contents\/MacOS\/AGXMetalG16X",
    "name" : "AGXMetalG16X",
    "CFBundleVersion" : "342.3"
  },
  {
    "source" : "P",
    "arch" : "arm64e",
    "base" : 4743757824,
    "CFBundleShortVersionString" : "1.14",
    "CFBundleIdentifier" : "com.apple.audio.units.Components",
    "size" : 1327104,
    "uuid" : "0312381d-61ae-3ab9-9cea-b1e46a0c4e54",
    "path" : "\/System\/Library\/Components\/CoreAudio.component\/Contents\/MacOS\/CoreAudio",
    "name" : "CoreAudio",
    "CFBundleVersion" : "1.14"
  },
  {
    "source" : "P",
    "arch" : "arm64",
    "base" : 5013291008,
    "CFBundleShortVersionString" : "144.0.11.0",
    "CFBundleIdentifier" : "org.cef.framework",
    "size" : 190283776,
    "uuid" : "4c4c4428-5555-3144-a170-b7e14b64c7dd",
    "path" : "\/Users\/USER\/*\/Chromium Embedded Framework.framework\/Chromium Embedded Framework",
    "name" : "Chromium Embedded Framework",
    "CFBundleVersion" : "11.0"
  },
  {
    "source" : "P",
    "arch" : "arm64e",
    "base" : 4759437312,
    "size" : 49152,
    "uuid" : "d4baeab8-b553-3779-a0ff-d8848e7a22df",
    "path" : "\/usr\/lib\/libobjc-trampolines.dylib",
    "name" : "libobjc-trampolines.dylib"
  },
  {
    "source" : "P",
    "arch" : "arm64e",
    "base" : 6969446400,
    "CFBundleShortVersionString" : "6.9",
    "CFBundleIdentifier" : "com.apple.CoreFoundation",
    "size" : 5540928,
    "uuid" : "649000a2-3eb4-3cf5-970a-d3cb37b5780c",
    "path" : "\/System\/Library\/Frameworks\/CoreFoundation.framework\/Versions\/A\/CoreFoundation",
    "name" : "CoreFoundation",
    "CFBundleVersion" : "4201"
  },
  {
    "source" : "P",
    "arch" : "arm64e",
    "base" : 7181836288,
    "CFBundleShortVersionString" : "2.1.1",
    "CFBundleIdentifier" : "com.apple.HIToolbox",
    "size" : 3158656,
    "uuid" : "fb92ce0c-1ee5-3f03-992c-df53ed9b3cb4",
    "path" : "\/System\/Library\/Frameworks\/Carbon.framework\/Versions\/A\/Frameworks\/HIToolbox.framework\/Versions\/A\/HIToolbox",
    "name" : "HIToolbox"
  },
  {
    "source" : "P",
    "arch" : "arm64e",
    "base" : 7041085440,
    "CFBundleShortVersionString" : "6.9",
    "CFBundleIdentifier" : "com.apple.AppKit",
    "size" : 24310400,
    "uuid" : "4e909aec-68bc-3fc9-a87a-de928e1e36e1",
    "path" : "\/System\/Library\/Frameworks\/AppKit.framework\/Versions\/C\/AppKit",
    "name" : "AppKit",
    "CFBundleVersion" : "2685.30.107"
  },
  {
    "source" : "P",
    "arch" : "arm64e",
    "base" : 6965186560,
    "size" : 651204,
    "uuid" : "0975afba-c46b-364c-bd84-a75daa9e455a",
    "path" : "\/usr\/lib\/dyld",
    "name" : "dyld"
  },
  {
    "size" : 0,
    "source" : "A",
    "base" : 0,
    "uuid" : "00000000-0000-0000-0000-000000000000"
  },
  {
    "source" : "P",
    "arch" : "arm64e",
    "base" : 6969212928,
    "size" : 33776,
    "uuid" : "4dbaf982-1576-3ffc-86be-03a9d2c96be5",
    "path" : "\/usr\/lib\/system\/libsystem_platform.dylib",
    "name" : "libsystem_platform.dylib"
  },
  {
    "source" : "P",
    "arch" : "arm64e",
    "base" : 6969159680,
    "size" : 51900,
    "uuid" : "527c4ba0-91a5-378b-b3e2-d38269ca5a66",
    "path" : "\/usr\/lib\/system\/libsystem_pthread.dylib",
    "name" : "libsystem_pthread.dylib"
  },
  {
    "source" : "P",
    "arch" : "arm64e",
    "base" : 6968909824,
    "size" : 246944,
    "uuid" : "548c45c8-9733-3f0d-8ef4-c06df1df2ad0",
    "path" : "\/usr\/lib\/system\/libsystem_kernel.dylib",
    "name" : "libsystem_kernel.dylib"
  },
  {
    "source" : "P",
    "arch" : "arm64e",
    "base" : 6967357440,
    "size" : 290400,
    "uuid" : "a4b349e8-dd6f-3b71-84d9-34f3b4acd849",
    "path" : "\/usr\/lib\/system\/libdispatch.dylib",
    "name" : "libdispatch.dylib"
  },
  {
    "source" : "P",
    "arch" : "arm64e",
    "base" : 6967660544,
    "size" : 532552,
    "uuid" : "fb5569a9-cb26-36c2-aa05-e99243692b60",
    "path" : "\/usr\/lib\/system\/libsystem_c.dylib",
    "name" : "libsystem_c.dylib"
  },
  {
    "source" : "P",
    "arch" : "arm64e",
    "base" : 6994948096,
    "CFBundleShortVersionString" : "6.9",
    "CFBundleIdentifier" : "com.apple.Foundation",
    "size" : 16397536,
    "uuid" : "6a518869-0a98-34cb-8a15-cc28f898255e",
    "path" : "\/System\/Library\/Frameworks\/Foundation.framework\/Versions\/C\/Foundation",
    "name" : "Foundation",
    "CFBundleVersion" : "4201"
  },
  {
    "source" : "P",
    "arch" : "arm64e",
    "base" : 7176888320,
    "CFBundleShortVersionString" : "1.0",
    "CFBundleIdentifier" : "com.apple.audio.caulk",
    "size" : 167296,
    "uuid" : "d4644b08-911d-30af-82e7-c404878abf47",
    "path" : "\/System\/Library\/PrivateFrameworks\/caulk.framework\/Versions\/A\/caulk",
    "name" : "caulk"
  },
  {
    "source" : "P",
    "arch" : "arm64e",
    "base" : 7260061696,
    "CFBundleShortVersionString" : "1.0",
    "CFBundleIdentifier" : "com.apple.MediaExperience",
    "size" : 796704,
    "uuid" : "20e67caa-84cf-379a-98e4-b84267bf9982",
    "path" : "\/System\/Library\/PrivateFrameworks\/MediaExperience.framework\/Versions\/A\/MediaExperience",
    "name" : "MediaExperience",
    "CFBundleVersion" : "1"
  },
  {
    "source" : "P",
    "arch" : "arm64e",
    "base" : 7020630016,
    "CFBundleShortVersionString" : "5.0",
    "CFBundleIdentifier" : "com.apple.audio.CoreAudio",
    "size" : 8041856,
    "uuid" : "f37b241b-2a83-3f86-bd94-329a18ba4715",
    "path" : "\/System\/Library\/Frameworks\/CoreAudio.framework\/Versions\/A\/CoreAudio",
    "name" : "CoreAudio",
    "CFBundleVersion" : "5.0"
  }
],
  "sharedCache" : {
  "base" : 6964101120,
  "size" : 5653544960,
  "uuid" : "acb998b6-263c-3634-b0a8-ae8270a116c2"
},
  "vmSummary" : "ReadOnly portion of Libraries: Total=2.2G resident=0K(0%) swapped_out_or_unallocated=2.2G(100%)\nWritable regions: Total=531.8M written=1202K(0%) resident=1202K(0%) swapped_out=0K(0%) unallocated=530.7M(100%)\n\n                                VIRTUAL   REGION \nREGION TYPE                        SIZE    COUNT (non-coalesced) \n===========                     =======  ======= \nAccelerate framework               256K        2 \nActivity Tracing                   256K        1 \nAttributeGraph Data               1024K        1 \nColorSync                           16K        1 \nCoreAnimation                      512K       32 \nCoreGraphics                        80K        5 \nCoreUI image data                  240K        4 \nFoundation                          48K        2 \nKernel Alloc Once                   32K        1 \nMALLOC                           141.4M       30 \nMALLOC guard page                 3472K        4 \nMemory Tag 253                    48.0G      873 \nMemory Tag 253 (reserved)          544K       34         reserved VM address space (unallocated)\nPROTECTED_MEMORY                    16K        1 \nSTACK GUARD                       1008K       63 \nStack                            324.4M       64 \nStack Guard                       56.0M        1 \nVM_ALLOCATE                       3040K       81 \nVM_ALLOCATE (reserved)            2576K       21         reserved VM address space (unallocated)\n__AUTH                            5790K      630 \n__AUTH_CONST                      88.1M     1011 \n__CTF                               824        1 \n__DATA                            37.6M      965 \n__DATA_CONST                      48.4M     1021 \n__DATA_DIRTY                      8231K      871 \n__FONT_DATA                        2352        1 \n__INFO_FILTER                         8        1 \n__LINKEDIT                       750.0M        6 \n__OBJC_RO                         78.4M        1 \n__OBJC_RW                         2570K        1 \n__TEXT                             1.5G     1044 \n__TPRO_CONST                       128K        2 \ndyld private memory                128K        1 \nmapped file                      303.2M       42 \npage table in kernel              1202K        1 \nshared memory                      992K       18 \n===========                     =======  ======= \nTOTAL                             51.3G     6838 \nTOTAL, minus reserved VM space    51.3G     6838 \n",
  "legacyInfo" : {
  "threadTriggered" : {
    "name" : "CrBrowserMain",
    "queue" : "com.apple.main-thread"
  }
},
  "logWritingSignature" : "2cc13c8842a7594d611e11bf340063f189fc21b9",
  "roots_installed" : 0,
  "bug_type" : "309",
  "trmStatus" : 1,
  "trialInfo" : {
  "rollouts" : [
    {
      "rolloutId" : "5fb4245a1bbfe8005e33a1e1",
      "factorPackIds" : [

      ],
      "deploymentId" : 240000021
    },
    {
      "rolloutId" : "67fd77fe1f9da9148f70d6ed",
      "factorPackIds" : [

      ],
      "deploymentId" : 240000011
    }
  ],
  "experiments" : [

  ]
}
}

Model: Mac16,8, BootROM 13822.61.10, proc 12:8:4 processors, 48 GB, SMC 
Graphics: Apple M4 Pro, Apple M4 Pro, Built-In
Display: Color LCD, 3024 x 1964 Retina, Main, MirrorOff, Online
Display: Artist13.3pro, 1920 x 1080 (1080p FHD - Full High Definition), MirrorOff, Online
Display: KG251Q, 1920 x 1080 (1080p FHD - Full High Definition), MirrorOff, Online
Memory Module: LPDDR5, Hynix
AirPort: spairport_wireless_card_type_wifi (0x14E4, 0x4388), wl0: Oct  3 2025 00:48:50 version 23.41.7.0.41.51.200 FWID 01-8b09c4e0
IO80211_driverkit-1533.5 "IO80211_driverkit-1533.5" Nov 14 2025 18:26:34
AirPort: 
Bluetooth: Version (null), 0 services, 0 devices, 0 incoming serial ports
Network Service: Wi-Fi, AirPort, en0
Thunderbolt Bus: MacBook Pro, Apple Inc.
Thunderbolt Bus: MacBook Pro, Apple Inc.
Thunderbolt Bus: MacBook Pro, Apple Inc.

```
