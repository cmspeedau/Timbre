Audio Engine Roadmap
1. Purpose and scope

Provide reliable, measurable audio playback from source files to supported output backends. The engine owns decoding, buffering, processing, format conversion, and audio diagnostics.

It does not own library metadata, application navigation, or device discovery policy.

2. Architecture

Recommended internal stages:

Source reader.
Decoder.
Canonical PCM representation.
Optional DSP chain.
Sample-rate and channel conversion.
Output buffer.
Platform or network output adapter.
Signal-path diagnostics.

Use pure-Rust components: symphonia for decoding, cpal for output, rubato for sample-rate conversion, rustfft and realfft for convolution, and ebur128 for loudness measurement. Prototype decoding and output end to end before finalizing the DSP design.

Bit-perfect, exclusive output is a first-class requirement. cpal provides device access but not exclusive mode, so the engine must implement per-operating-system backends: WASAPI exclusive on Windows, ALSA hw on Linux, and CoreAudio hog mode on macOS. This work belongs to Phase 2, not Phase 4, because it determines whether the engine is viable.

3. Implementation phases
Phase 0 — Audio prototype
- [ ] Select supported operating systems and initial codecs.
- [ ] Compare candidate media frameworks.
- [ ] Build a command-line playback prototype.
- [ ] Measure latency, CPU use, and buffering behavior.
- [ ] Define decoder and output interfaces.
- [ ] Exit gate: A prototype decodes and renders to one output with measured latency, CPU, and buffering.
Phase 1 — Reliable playback
- [ ] Implement file-backed playback.
- [ ] Add play, pause, seek, stop, and end-of-track handling.
- [ ] Support cancellation and safe resource cleanup.
- [ ] Handle decoder and output-device errors.
- [ ] Prevent underruns under expected workloads.
- [ ] Keep audio processing off UI and API request threads.
- [ ] Exit gate: Supported files play reliably from start to finish with correct lifecycle and error handling.
Phase 2 — Format and output management
- [ ] Implement source-format inspection.
- [ ] Negotiate output capabilities.
- [ ] Implement bit-perfect exclusive output per operating system: WASAPI exclusive, ALSA hw, and CoreAudio hog mode.
- [ ] Add channel mapping.
- [ ] Add explicit sample-rate conversion where required.
- [ ] Add volume and mute handling.
- [ ] Distinguish source format from rendered output format.
- [ ] Exit gate: Source and rendered formats are reported accurately and bit-perfect exclusive output works on each supported OS.
Phase 3 — DSP
- [ ] Implement a configurable processing graph.
- [ ] Add headroom and clipping protection.
- [ ] Add loudness normalization.
- [ ] Add parametric equalization.
- [ ] Add convolution-based correction.
- [ ] Add processing bypass and per-zone settings.
- [ ] Document processing order and numerical behavior.
- [ ] Exit gate: The DSP graph is configurable, deterministic, and documented, with bypass and headroom protection.
Phase 4 — Quality and optimization
- [ ] Add gapless playback where supported by the pipeline.
- [ ] Evaluate bit-perfect output on compatible paths.
- [ ] Add dropout and underrun diagnostics.
- [ ] Test format changes between tracks.
- [ ] Optimize memory allocation and buffer management.
- [ ] Add long-duration playback tests.
- [ ] Exit gate: Playback is gapless where supported, diagnostics distinguish decode from output failures, and long sessions are stable.
4. Task checklist
- [ ] Decoding and rendering have explicit lifecycle management.
- [ ] Audio processing does not depend on UI frame rate.
- [ ] Buffer sizes and latency are measurable.
- [ ] Every processing stage reports its configuration.
- [ ] Bit-perfect mode is claimed only when the entire output path supports it.
- [ ] Unsupported format negotiation fails clearly.
- [ ] DSP bypass provides a reproducible baseline.
- [ ] Audio-thread work avoids blocking I/O and unbounded allocations where practical.
5. API and contracts

Expose an internal engine interface covering:

OpenSource(mediaReference)
GetSourceFormat()
ConfigureOutput(capabilities)
Start(), Pause(), Seek(), Stop()
SetProcessingGraph(configuration)
GetPlaybackMetrics()
GetSignalPath()
Close()

Define PCM representation, buffer ownership, timestamps, error codes, and thread-safety requirements explicitly.

6. Testing strategy
Test decoding against known audio fixtures.
Verify PCM output with reference samples and hashes.
Test sample-rate conversion and channel mapping numerically.
Test clipping, silence, channel inversion, and edge cases.
Measure latency, CPU, memory, and underruns.
Test long tracks, gapless transitions, and device loss.
Perform listening tests as a supplement to numerical validation.
7. Acceptance criteria
Supported files play reliably from beginning to end.
Output-format reporting reflects actual engine configuration.
DSP behavior is deterministic for defined test inputs.
Unsupported formats produce actionable errors.
Playback survives UI navigation and normal API activity.
Engine diagnostics distinguish decoding errors from output failures.
8. Dependencies and blockers
Reads media references from library/.
Receives playback commands through server/.
Supplies playback state and diagnostics to client/.
Exposes output interfaces to devices/.
Depends on audio fixtures and performance benchmarks from testing/.
9. Definition of done

An engine feature requires numerical or behavioral tests, explicit lifecycle semantics, documented format constraints, performance measurements where relevant, and accurate signal-path reporting.
