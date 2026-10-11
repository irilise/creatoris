# shadermanager/ — ports of Celestia's *dynamic* shader generator

These WGSL files translate the shaders that Celestia **generates at runtime** in
`Celestia/src/celengine/shadermanager.cpp` (3348 lines) from `ShaderProperties`
+ `LightingModel`, plus the Lua GL-compat quad in
`Celestia/src/celscript/lua/glcompat.cpp` (see `../glcompat/glcompat.wgsl`).

Unlike the standalone passes in `../{star,cluster,body,post,ui}/` (which are the
`*.glsl` files in `Celestia/shaders/`), these are **not** literal source files —
the GLSL is assembled by C++ string concatenation with conditional `#if`
branches (specular, shadows, night maps, atmospheric scattering, ring splits,
fisheye, point sprites, lines). WGSL has no preprocessor and no runtime code
generation, so each generated program is ported as a **representative canonical
variant** with the conditional variants documented inline as notes / branches.

> **Status:** translated and organized, **not yet wired into the Bevy pipeline**.
> `@group`/`@binding` numbers and input `@location`s are placeholders to be
> reconciled when `RenderPipeline`s are built.

## Mapping: shadermanager.cpp builders → WGSL files

| WGSL file | GLSL source (`shadermanager.cpp`) | LightingModel / notes |
| --- | --- | --- |
| `common.wgsl` | `VertexHeader`, `VPFunction*`, `CommonAttribs`, `ChapmanOpticalDepth`, `ScatteringPhaseFunctions`, `Shadow*`, `DeclareLights` | Shared helpers/matrices — **library file**, not a standalone pipeline. Concatenate or inline into the programs that use it. |
| `body.wgsl` | `buildVertexShader` + `buildFragmentShader` (the `else` branch of `buildProgram`) | `DiffuseModel`/`PerPixelSpecular`/`OrenNayar`/`LunarLambert`/`StarModel`/`Unlit`/`ParticleDiffuse`. Canonical: 1 light, diffuse+specular+night+emissive. Conditional variants noted inline (OrenNayar is `#if 0`/disabled in source; LunarLambert & StarModel variants as notes; tangent-space branch omitted). |
| `rings.wgsl` | `buildRingsVertexShader` + `buildRingsFragmentShader` | `RingIllumModel`: ring-atmosphere split (`ringHalf`), per-light `litSide`/optical-depth, eclipse shadows via `shadowDepths`. |
| `atmosphere.wgsl` | `buildAtmosphereVertexShader` + `buildAtmosphereFragmentShader` + `AtmosphericEffects` | `AtmosphereModel`: multi-segment Chapman ray-march, Rayleigh+Mie scatter & transmittance. Ports the **modern separate-Rayleigh/Mie** variant; legacy variant noted at the end. Vertex uses a `centroid` position (MSAA-safe). |
| `emissive.wgsl` | `buildEmissiveVertexShader` + `buildEmissiveFragmentShader` | `EmissiveModel`: unlit material color (optionally from a diffuse texture). |
| `particle.wgsl` | `buildParticleVertexShader` + `buildParticleFragmentShader` | `ParticleModel`/`ParticleDiffuseModel`: per-particle Mie phase weighting toward the light. |
| `line.wgsl` | `LineVertexPosition` + `LineFragmentDeclaration` | `LineAsTriangles`: 1px-inflated geometry + `fwidth` edge AA (`lineU`). |
| `error.wgsl` | `CreateErrorProgram` (`errorVertexShaderSource`/`errorFragmentShaderSource`) | Trivial flat-red diagnostic program. |
| `../glcompat/glcompat.wgsl` | `kVertexShader`/`kFragmentShader` (`glcompat.cpp`) | Lua GL-compat full-screen quad; `SHADER_COLOR`/`SHADER_TEXCOORD` variants kept as separate `@vertex`/`@fragment` pairs. |

## Extra GLSL→WGSL notes specific to this group

| GLSL (shadermanager.cpp) | WGSL |
| --- | --- |
| `centroid out vec3 position;` | `@interpolant(centroid) position` — enable the pipeline with `depth_clip_clamp`; WGSL has no MSAA `centroid` qualifier otherwise |
| `sampler2DShadow` / `texture(shadowMapTex0, …)` | `sampler_2d_compare` + `textureSampleLevel` (PCF shadow mapping) |
| `textureLod(ringTex, uv, lod)` | `textureSampleLevel(ring_texture, ring_sampler, uv, lod)` |
| `invariant gl_Position;` | dropped — no WGSL equivalent |
| `#if OrenNayarModel` | `#if 0` in source (disabled) — ported as an inline note, not a branch |
| `#ifdef GL_ES … #endif` | dropped — WGSL has no `GL_ES`; precision defaults to highp `f32` |
| `gl_PointSize` / `gl_PointCoord` | `@builtin(point_size)` / `@builtin(point_coord)` + `point-size` feature |
| `fwidth(lineU)` | `fwidth(line_u)` (WGSL 1.0 built-in) |
| Light arrays `light{i}_direction` … | single-light `light0_*` here; Celestia loops `nLights` |

## Deliberately not ported

* **Tangent-space lighting branch** of `buildFragmentShader` (normal-map +
  `TangentSpaceTransform`) — large; left as a noted TODO on `body.wgsl`.
* **`modelviewwidget.cpp`** (`Celestia/src/tools/cmod/cmodview/`) — legacy
  fixed-function `ftransform()`/`gl_Vertex` output for the cmod viewer tool, not a
  modern shader; not worth a WGSL equivalent.
