# Celestia render-pass shaders (GLSL → WGSL port)

WGSL ports of the standalone GLSL render passes that ship in
`Celestia/shaders/*.glsl`. Celestia renders each of these with its own
`ShaderManager`-built program; they are **not** `bevy_pbr` materials, so — unlike
the per-body `../<body>/<body>.wgsl` StandardMaterial shaders — they are written
as plain `@vertex` / `@fragment` programs with explicit uniform groups.

> **Status:** translated and organized, **not yet wired into the Bevy pipeline**.
> Each file is self-contained and commented; `@group`/`@binding` numbers are
> placeholders to be reconciled when a `RenderPipeline` is built for them.

## Layout

```
celestia/
  star/                 point-sprite & PSF star glow passes
    star.wgsl               point-sprite star (gl_PointSize)
    psfstarpoint.wgsl       PSF "point/cone" pass (point-sprite variant)
    psfstarglow.wgsl        PSF "glow / eye-PSF" pass (point-sprite variant)
    psfstarglowlarge.wgsl   PSF glow batched-billboard fallback
    largestar.wgsl          batched-billboard fallback for oversized point sprites
  cluster/              large diffuse object sprites
    galaxy.wgsl             galaxy sprite
    globular.wgsl           globular-cluster star field
    tidal.wgsl              tidal / tidal-stream sprite
  body/
    comet.wgsl              comet glow (shade = |dot(viewDir,normal)| * fade)
  post/                 full-screen post-processing
    srgb.wgsl               linear→sRGB tone-map / display-encode pass
    warpmesh.wgsl           warped full-screen mesh
    depth.wgsl              depth-only depth prepass (no color output)
  ui/                   on-screen overlay primitives
    crosshair.wgsl          rotated crosshair
    selpointer.wgsl         selection-pointer quad
    text.wgsl               glyph atlas text
  shadermanager/          ports of the DYNAMIC shader generator (shadermanager.cpp)
    common.wgsl             shared helpers/matrices (library, not a pipeline)
    body.wgsl               main body/surface (DiffuseModel + variants)
    rings.wgsl              ring system (RingIllumModel)
    atmosphere.wgsl         atmospheric scattering (AtmosphereModel)
    emissive.wgsl           emissive/unlit (EmissiveModel)
    particle.wgsl           particle systems (ParticleModel)
    line.wgsl               line-as-triangles AA (LineAsTriangles)
    error.wgsl              flat-red diagnostic program
  glcompat/
    glcompat.wgsl           Lua GL-compat full-screen quad
```

> The `shadermanager/` and `glcompat/` trees port GLSL that Celestia generates in
> C++ (`shadermanager.cpp`, `glcompat.cpp`) rather than shipping as `*.glsl` files.
> See `shadermanager/README.md` for the builder→WGSL mapping and the conditional
> (`#if`) variant notes.

## GLSL → WGSL translation rules applied

| GLSL (Celestia) | WGSL |
| --- | --- |
| `layout(location = N) in X v;` | `@vertex_input @location(N) v: type;` inside a vertex input struct |
| `out X v;` (varying) | `@interpolant(per-vertex)` on the vertex-output struct and `@interpolant(per-fragment)` on the fragment-input struct |
| `uniform X x;` | `@group(0) @binding(K) var<uniform> x: X;` — one variable per binding (avoids WGSL 16-byte uniform-block alignment churn) |
| `gl_Position` | `@builtin(position) out_position: vec4<f32>` |
| `fragColor` / `out vec4` | `-> @location(0) vec4<f32>` |
| `sampler2D tex;` | split into `var tex: texture_2d<f32>;` + `var samp: sampler;` (sampled as `textureSample(tex, samp, uv)`) |
| `texture(tex, uv)` | `textureSample(tex, samp, uv)` (arg order is `texture, sampler, uv`) |
| `gl_PointSize` | `@builtin(point_size)` output — **requires the `point-size` feature** |
| `gl_PointCoord` | `@builtin(point_coord)` input — **requires the `point-size` feature** |
| `set_vp(p)` | Celestia's "usual" projection mode: `MVPMatrix * p` (see `calc_vp` in `shadermanager.cpp`); ported to `mvp_matrix * p` |
| `MVPMatrix * ...` | `mvp_matrix * ...` (Celestia's `MVPMatrix` uniform) |
| `mat3(c, s, 0, -s, c, 0, 0, 0, 1)` | column-major: `mat3x3<f32>(vec3(c,s,0), vec3(-s,c,0), vec3(0,0,1))` |
| `#ifdef BILLBOARD` / `#ifdef TONE_MAP` | WGSL has no preprocessor — the point-sprite branch is ported as the primary path and the billboard branch is kept as a separate file (`psfstarglowlarge.wgsl`); conditional compile-time options are expressed as a runtime uniform flag where cheap |
| `invariant gl_Position;` | dropped — WGSL has no `invariant` qualifier (noted per file) |
| `discard;` | `discard;` |
| `centroid out vec3 position;` | `@interpolant(centroid) position` + `depth_clip_clamp` pipeline feature |
| `sampler2DShadow` / `textureLod` | `sampler_2d_compare` + `textureSampleLevel` |
| `#if`-selected variants (specular, night, scattering, fisheye) | WGSL has no preprocessor — ported as a canonical variant + inline notes / branches |

## Celestia-specific notes

* **`set_vp` / `MVPMatrix`** — every `*_vert.glsl` that calls `set_vp(p)` is
  Celestia's `calc_vp` "usual" mode (`MVPMatrix * p`). The fisheye mode
  (`calc_vp` with a lens-radius remap) is *not* ported; add it as a separate
  variant if fisheye projection is needed.
* **Batched attribute locations** — several shaders (galaxy, globular, tidal)
  consume per-star float streams (`in_Size`, `in_ColorIndex`, `in_Brightness`)
  that Celestia injects with locations **not** fixed in `shadermanager.cpp`'s
  `CommonAttribs`. The port assigns plausible locations and flags them with a
  comment; reconcile against the real vertex-buffer layout when wiring up.
* **Point sprites** — `star`, `psfstarpoint`, `psfstarglow`, `globular`
  emit `@builtin(point_size)` and sample `@builtin(point_coord)`; enable the
  `point-size` feature on the pipeline that runs them.
* **Copyright headers** — the PSF star files carry a GPL + attribution header
  (Askaniy Anpilogov); it is preserved in the WGSL port.
