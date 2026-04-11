# Webview Drag-to-Resize — Design Spec

> Date: 2026-04-12
> Status: Draft (awaiting user approval)
> Target: `bevy_cef` / `bevy_cef_core`
> Related: [CEF Webview テクスチャ高画質化 要件定義書](../../specs/2026-04-10-cef-webview-texture-quality-requirements.md)

## 1. Goals and Scope

### 1.1 Phase 1 (this spec — the resize PR)

Add native-OS-window-style drag-to-resize to `bevy_cef` webviews:

- Grab an edge or corner of a webview, drag, and the webview resizes — both visually (mesh/sprite size) AND in texture resolution (`WebviewSize`), so the page reflows responsively at the new dimensions.
- Supports both 3D mesh and 2D sprite webviews.
- 8-way resize (4 edges + 4 corners) with automatic N-pixel edge detection (no HTML markup required).
- Opt-in per webview via `WebviewResizable` component.
- Native-style cursor feedback on edge hover (`nwse-resize`, `ns-resize`, etc.).
- `Shift` held during drag locks the aspect ratio to the value at drag start.
- Anchor behavior: dragging one edge pins the opposite edge in world space.
- Introduces a unified derivation pipeline `WebviewSize = DisplaySize × BaseRenderScale × QualityMultiplier × DPR` as a scaffold for Phase 2's quality profile feature.

### 1.2 Phase 2 (follow-up, separate PR — only a compatibility contract here)

The [texture quality requirements spec](../../specs/2026-04-10-cef-webview-texture-quality-requirements.md) defines a quality profile feature (`balanced`/`crisp`/`ultra`) and multi-monitor DPR tracking. Phase 1 must leave the pipeline in a state where Phase 2 is purely additive:

- `QualityMultiplier` component is introduced in Phase 1 pinned to `1.0`; Phase 2 drives it from profiles.
- `WebviewDpr` component is introduced in Phase 1 sampled once at spawn; Phase 2 updates it on monitor transitions.
- `BaseRenderScale` is snapshotted at spawn and **never overwritten** by Phase 2 — the quality multiplier composes on top of it.

### 1.3 Non-goals in Phase 1

- Quality profile preset system.
- Monitor-transition DPR updates.
- Full migration of `WebviewSize` to derived-only status.
- Non-centered origins (sprites with `Anchor::TopLeft`, meshes with off-center vertices).
- Non-Z-normal planes (e.g., `Plane3d::new(Vec3::Y, ...)`). Phase 1 requires Z-normal meshes; debug-asserted at `on_add`.
- Animated / parent-scaled webviews (the pipeline owns `Transform.scale.xy` on resizable meshes).
- Linux support (plugin does not yet support Linux at all).

### 1.4 Backwards compatibility contract

1. Existing webviews spawned with `WebviewSource + WebviewSize` and **no** `WebviewResizable` continue to work unchanged. No visible behavior change.
2. `WebviewSize` remains a writable component in Phase 1. Setting it directly on non-resizable webviews still works.
3. When a webview has `WebviewResizable`, the pipeline owns `WebviewSize` — direct writes are overwritten each frame by the pipeline. Documented as "don't set `WebviewSize` on resizable webviews".
4. `ZoomLevel` and `QualityMultiplier` are independent concepts (satisfies FR-006 of the quality spec).
5. Existing `-webkit-app-region: drag` regions continue to work. Resize edges take priority over drag regions at the webview's outer frame — matches native OS window behavior.

## 2. Unified Derivation Pipeline

### 2.1 Core formula

```
WebviewSize = round(DisplaySize × BaseRenderScale × QualityMultiplier × WebviewDpr).clamp(min, max)
```

Where each term is an explicit ECS component:

| Component | Type | Phase 1 value | Phase 2 value |
|---|---|---|---|
| `DisplaySize` | `Vec2` | Logical visual size (DIP for sprite, world units for mesh) | unchanged |
| `BaseRenderScale` | `Vec2` | Snapshotted at spawn, per-axis | unchanged — never overwritten |
| `QualityMultiplier` | `f32` | Always `1.0` | Driven by quality profile |
| `WebviewDpr` | `f32` | Sampled at spawn from the `HostWindow` (or `PrimaryWindow` fallback) — same resolution path as `create_browser` | Updated on monitor transitions |

Per-axis `BaseRenderScale` is needed to honor non-square initial ratios (e.g., `WebviewSize(800, 800)` on a 2×1 mesh — the snapshotted ratio is 400 vs 800 px/world-unit, which must be preserved).

### 2.2 `BaseRenderScale` snapshot at `on_add`

For a 3D mesh at the moment `WebviewResizable` is inserted:

```
BaseRenderScale = WebviewSize / (mesh_world_2d_size × WebviewDpr)
```

Where `mesh_world_2d_size` is computed via a `WebviewBasis2d` helper (see §2.4) that projects the mesh's AABB into its local XY plane and applies `GlobalTransform.scale`. Phase 1 requires Z-normal planes (debug-asserted at `on_add`); non-Z-normal support is deferred to Phase 2.

If the mesh AABB is not yet available at `on_add` time (asset still loading), a `PendingBasisInit` marker is inserted and the snapshot is retried each frame in a system before `DerivePipeline`. The derive pipeline skips entities with `PendingBasisInit`.

For a 2D sprite:

```
BaseRenderScale = WebviewSize / (custom_size × WebviewDpr)
```

This handles sprites where the user intentionally set a higher-than-1:1 texture ratio for crispness (e.g., `custom_size = 400×300`, `WebviewSize = 800×600` → `BaseRenderScale = (2.0, 2.0)` on DPR 1.0).

The algebraic identity `WebviewSize = DisplaySize × BaseRenderScale × QualityMultiplier × WebviewDpr` holds immediately after the snapshot, for any initial user configuration — both mesh and sprite paths.

### 2.3 Phase 2 forward compatibility

When Phase 2 introduces quality profiles, a profile like `Ultra = 2×` sets `QualityMultiplier = 2.0`. The pipeline naturally doubles texture pixels without touching `BaseRenderScale`. A subsequent switch to `Balanced = 1.0` returns to the original density. There is **no double-counting of DPR** because DPR sits in its own term and is sampled from the window (not folded into `BaseRenderScale`).

### 2.4 `WebviewBasis2d` — shared mesh 2D basis

A new component for the resize pipeline. Phase 1 constrains to Z-normal planes (debug-asserted). The existing pointer mapping code (`src/system_param/pointer.rs:143`) has a latent bug with non-Z-normal planes; fixing that is out of scope for Phase 1 but the `WebviewBasis2d` abstraction makes it straightforward to fix in Phase 2.

```rust
#[derive(Component, Debug, Clone, Copy)]
pub struct WebviewBasis2d {
    /// World-space direction of local +U (webview width).
    pub u_axis: Vec3,
    /// World-space direction of local +V (webview height).
    pub v_axis: Vec3,
    /// Extent of the webview in local 2D units (width, height), pre-scale.
    pub local_size: Vec2,
}
```

Derived at `on_add` from the mesh's AABB and `GlobalTransform`, via a helper that detects the plane normal and picks the two perpendicular axes in local frame. Used by:

- Resize pipeline (compute world-space u/v axes for anchor math).
- Derivation (compute `mesh_world_2d_size = local_size × Transform.scale.xy()`).
- Pointer mapping — replacing the existing `x/y`-only logic. Phase 1 also fixes this latent bug as an in-scope improvement.

Sprites don't get a `WebviewBasis2d` — for a `Sprite`, the axes are always `X` and `Y` in world space.

### 2.5 Pipeline system ordering

**All pipeline steps run in `Update`, before the existing `create_webview` and `resize` systems.** A new `WebviewSet` enum introduces explicit ordering:

```rust
#[derive(SystemSet, Clone, Debug, Hash, PartialEq, Eq)]
pub enum WebviewSet {
    /// Updates DisplaySize (and Transform.scale.xy for meshes) during drag-resize.
    ResizeInteraction,
    /// Derives WebviewSize from DisplaySize × BaseRenderScale × QualityMultiplier × DPR.
    DerivePipeline,
    /// Existing browser creation.
    CreateBrowser,
    /// Existing WebviewSize→CEF resize commit.
    CommitResize,
}
// In App::build():
//   .configure_sets(Update, (
//       WebviewSet::ResizeInteraction,
//       WebviewSet::DerivePipeline,
//       WebviewSet::CreateBrowser,
//       WebviewSet::CommitResize,
//   ).chain())
```

Moving the existing `create_webview` and `resize` systems into `CreateBrowser` and `CommitResize` sets is a small refactor. This guarantees:

1. **Spawn frame**: `DerivePipeline` runs first and computes the correct `WebviewSize` from `WebviewResizable`+mesh/sprite state. `CreateBrowser` then sees the derived size. No 800×800 flash.
2. **Resize frame**: `ResizeInteraction` writes `DisplaySize` → `DerivePipeline` computes new `WebviewSize` → `CommitResize` calls `browsers.resize()`. One frame of latency total.

### 2.6 Change detection scope

The derive system triggers on `Or<(Changed<DisplaySize>, Changed<BaseRenderScale>, Changed<QualityMultiplier>, Changed<WebviewDpr>, Changed<WebviewResizable>)>`. The `Changed<WebviewResizable>` inclusion ensures that runtime adjustments to `min_size`/`max_size` take effect immediately.

Only processes entities with `WebviewResizable`. Entities with `PendingBasisInit` are skipped (retried next frame). Non-resizable webviews skip the pipeline entirely and retain legacy behavior.

### 2.7 Integer commit policy

```
target_px = round(display × base × quality × dpr)
target_px = target_px.max(min_size)
target_px = max_size.map_or(target_px, |m| target_px.min(m))
if target_px != current_webview_size:
    current_webview_size = target_px
```

Dedupe on integer change (Q4(i)): a drag that moves the cursor sub-pixel does not spam CEF with resize commands. No additional throttling in Phase 1 — if CEF reflow proves expensive, Phase 2 can add a 30 Hz rate-limit without changing the pipeline shape.

## 3. `WebviewResizable` Component and Edge Hit-Testing

### 3.1 Component

```rust
/// Makes a webview user-resizable by dragging its edges/corners.
/// Opt-in — presence of this component activates the derive pipeline and edge hit-testing.
#[derive(Component, Debug, Clone, Copy)]
pub struct WebviewResizable {
    /// Width of the invisible resize border, in **texture pixels** (not DIP).
    /// Default: 16.
    pub edge_thickness: u32,

    /// Minimum texture size in pixels. Default: (100, 100).
    /// Must satisfy `min_size > 2 × edge_thickness` on both axes (debug-asserted).
    pub min_size: UVec2,

    /// Maximum texture size in pixels. `None` = no cap. Default: None.
    pub max_size: Option<UVec2>,

    /// Aspect-lock behavior. Default: `LockOnShift` (free, Shift locks to initial ratio).
    pub aspect_lock: AspectLockMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AspectLockMode {
    #[default]
    LockOnShift,
    Always,
    Never,
}
```

`edge_thickness` is in **texture pixels** (not DIP) so the hit-test operates directly in the existing texture-pixel coordinate system used by `PixelRect` in `src/drag.rs:40`. Users who want a specific physical thickness can compute `desired_dip × DPR × QualityMultiplier`.

Auto-requires on insertion: `DisplaySize`, `BaseRenderScale`, `QualityMultiplier(1.0)`, `WebviewDpr`, and `WebviewBasis2d` (mesh only). Values populated by an `on_add` hook reading from the current mesh/sprite/transform/window state.

### 3.2 Usage

```rust
commands.spawn((
    WebviewSource::local("app.html"),
    WebviewSize(UVec2::new(800, 600)),
    Mesh3d(meshes.add(Plane3d::new(Vec3::Z, Vec2::ONE))),
    MeshMaterial3d(materials.add(WebviewExtendStandardMaterial::default())),
    Transform::from_translation(Vec3::ZERO),
    WebviewResizable::default(),
));
```

### 3.3 The 8 zones

Given `WebviewSize = (w, h)` in pixels and `t = edge_thickness`, classify a texture-pixel point `(px, py)`:

```
             | left t | interior | right t |
  top t      |   NW   |    N     |   NE    |
  interior   |   W    |  (none)  |   E     |
  bottom t   |   SW   |    S     |   SE    |
```

Corners win over edges when both are true. A pointer in the interior (non-resize) zone has `t ≤ px ≤ w-t` AND `t ≤ py ≤ h-t`, and falls through to normal page input / drag-region hit-test.

### 3.4 Unified hit-test

The existing `is_draggable` function in `src/drag.rs:132` is replaced by a routing function:

```rust
pub(crate) enum HitResult {
    Resize(ResizeZone),
    Drag,
    None,
}

pub(crate) fn classify_hit(
    regions: &DraggableRegions,
    frame: &ResizeFrame,  // size + edge_thickness
    pos: Vec2,
) -> HitResult {
    if let Some(zone) = frame.classify(pos) {
        return HitResult::Resize(zone);  // resize wins at the edges
    }
    if is_draggable(&regions.drag_rects, &regions.no_drag_rects, pos) {
        return HitResult::Drag;
    }
    HitResult::None
}
```

This satisfies Q8: a `-webkit-app-region: drag` region at pixel `(3, 3)` with `edge_thickness = 16` returns `HitResult::Resize(NW)`, not `HitResult::Drag`. Page authors with full-width title bars lose at most `edge_thickness` pixels of draggable area at each border.

### 3.5 Dead-zone guard

If `min_size.x ≤ 2 × edge_thickness` or `min_size.y ≤ 2 × edge_thickness`, the interior (page-input) zone shrinks to zero. Enforced at component construction via `debug_assert!` and at pipeline output via defensive clamp. Default `min_size (100, 100)` with default `edge_thickness 16` leaves a 68×68 interior, well above zero.

## 4. Resize State Machine and Cursor Feedback

### 4.1 State resource

```rust
#[derive(Resource, Debug, Default)]
pub(crate) enum ResizeState {
    #[default]
    Idle,
    Hovering {
        webview: Entity,
        zone: ResizeZone,
    },
    Resizing {
        webview: Entity,
        zone: ResizeZone,
        start_display_size: Vec2,
        start_translation: Vec3,
        start_hit_world: Vec3,
        plane_origin: Vec3,
        plane_normal: Dir3,
        camera: Entity,
        u_axis: Vec3,
        v_axis: Vec3,
        aspect_lock_mode: AspectLockMode,  // from WebviewResizable; shift checked dynamically
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResizeZone { N, S, E, W, NE, NW, SE, SW }
```

### 4.2 Mutual exclusion with drag

Both `DragState` and `ResizeState` exist. Only one can be active at a time — checked at the unified pointer-press observer entry:

```rust
fn on_webview_press(trigger, drag_state, resize_state, ...) {
    if drag_state.is_dragging() || matches!(*resize_state, ResizeState::Resizing { .. }) {
        return;
    }
    match classify_hit(...) {
        HitResult::Resize(zone) => start_resize(zone, ...),
        HitResult::Drag         => start_drag(...),
        HitResult::None         => { /* flow to CEF page input */ }
    }
}
```

Implementation: merge the existing `on_drag_press` observer into a single `on_webview_press` observer that routes to either resize or drag. Attached to any webview entity with `Or<(With<WebviewResizable>, With<WebviewSource>)>`.

**CEF input gating**: all existing CEF input forwarding systems — `apply_on_pointer_move`, `apply_on_pointer_pressed`, `apply_on_pointer_released` (in both `src/webview/mesh.rs` and `src/webview/webview_sprite.rs`), and `on_mouse_wheel` — currently gate on `drag_state.is_dragging()`. They must **also** gate on `resize_state.is_resizing()`. During an active resize, no pointer events should flow to CEF. The `is_resizing()` method returns `true` iff `ResizeState::Resizing { .. }`.

### 4.3 Tracking system

Runs in `WebviewSet::ResizeInteraction`, mirroring `drag_tracking_system` in `src/drag.rs:234`:

```rust
fn resize_tracking_system(
    mut resize_state: ResMut<ResizeState>,
    mouse: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    mut webviews: Query<(&mut Transform, &mut DisplaySize, &WebviewResizable, ...)>,
    mut pending: ResMut<InteractionEndPending>,
) {
    let ResizeState::Resizing { webview, zone, start_display_size, start_translation,
                                 start_hit_world, plane_origin, plane_normal, camera,
                                 u_axis, v_axis, .. } = *resize_state else {
        return;
    };

    if !mouse.pressed(MouseButton::Left) {
        *resize_state = ResizeState::Idle;
        pending.webview = Some(webview);
        return;
    }

    let cursor = windows.iter().find_map(|w| w.cursor_position())?;
    let (cam, cam_gtf) = cameras.get(camera).ok()?;
    let ray = cam.viewport_to_world(cam_gtf, cursor).ok()?;
    let t = ray.intersect_plane(plane_origin, InfinitePlane3d::new(plane_normal))?;
    let current_hit = ray.origin + ray.direction * t;
    let delta_world = current_hit - start_hit_world;

    let du = delta_world.dot(u_axis);
    let dv = delta_world.dot(v_axis);

    let lock = match resizable.aspect_lock {
        AspectLockMode::Always       => true,
        AspectLockMode::Never        => false,
        AspectLockMode::LockOnShift  => keyboard.pressed(KeyCode::ShiftLeft)
                                      || keyboard.pressed(KeyCode::ShiftRight),
    };

    let (new_size, new_tr) = apply_resize(
        zone, start_display_size, start_translation,
        du, dv, u_axis, v_axis,
        lock, min_size_display, max_size_display,
    );

    let (mut tf, mut display, ..) = webviews.get_mut(webview).ok()?;
    display.0 = new_size;
    tf.translation = new_tr;
    // Pipeline picks up Changed<DisplaySize> this same frame.
}
```

Note: `min_size`/`max_size` are configured in **texture pixels** on `WebviewResizable`, but clamping happens in **display-size units** here. Conversion via `min_px / (BaseRenderScale × QualityMultiplier × WebviewDpr)`.

### 4.4 Cursor feedback

A separate system runs every frame while not `Resizing`:

The cursor hover system requires **two paths** because `WebviewPointer` only works with `Camera3d` (mesh webviews) and sprite pointer mapping is separate:

- **3D mesh path**: uses `WebviewPointer<Camera3d>` + `pointer_pos_raw()` to get the hovered webview entity and texture-pixel position.
- **2D sprite path**: uses the existing `obtain_relative_pos()` from `src/webview/webview_sprite.rs` with the window cursor position.

Both paths produce `Option<(Entity, Vec2)>` (webview entity + texture-pixel position) and feed into the same zone classification:

```rust
fn cursor_hover_system(
    // ... per-target hover detection (mesh + sprite paths produce hovered_webview)
    mut resize_state: ResMut<ResizeState>,
    resizables: Query<(&WebviewResizable, &WebviewSize)>,
    mut cursor_override: ResMut<SystemCursorOverride>,
) {
    if matches!(*resize_state, ResizeState::Resizing { .. }) {
        return;
    }

    // hovered_webview: Option<(Entity, Vec2)> from mesh or sprite path
    let Some((webview, pixel_pos)) = hovered_webview else {
        *resize_state = ResizeState::Idle;
        cursor_override.clear();
        return;
    };
    let Ok((resizable, size)) = resizables.get(webview) else { return; };
    match classify_zone(pixel_pos, size.0, resizable.edge_thickness) {
        Some(zone) => {
            *resize_state = ResizeState::Hovering { webview, zone };
            cursor_override.set(cursor_for_zone(zone));
        }
        None => {
            *resize_state = ResizeState::Idle;
            cursor_override.clear();
        }
    }
}

fn cursor_for_zone(zone: ResizeZone) -> SystemCursorIcon {
    use SystemCursorIcon::*;
    match zone {
        ResizeZone::N | ResizeZone::S => NsResize,
        ResizeZone::E | ResizeZone::W => EwResize,
        ResizeZone::NE | ResizeZone::SW => NeswResize,
        ResizeZone::NW | ResizeZone::SE => NwseResize,
    }
}
```

### 4.5 Cursor priority — override resource

```rust
#[derive(Resource, Default)]
pub struct SystemCursorOverride(Option<SystemCursorIcon>);

impl SystemCursorOverride {
    pub fn set(&mut self, icon: SystemCursorIcon) { self.0 = Some(icon); }
    pub fn clear(&mut self) { self.0 = None; }
    pub fn get(&self) -> Option<SystemCursorIcon> { self.0 }
}
```

The existing `SystemCursorIconPlugin`'s cursor-apply system (in `src/cursor_icon.rs`) checks `SystemCursorOverride` first. If set, it applies the override; otherwise it applies CEF's cursor. Single-line change: one `if let Some(icon) = override_res.get()` branch.

### 4.6 Interaction end

On mouse release, `ResizeState::Idle` is restored and `InteractionEndPending { webview: Some(...) }` is set. One frame later, a system (`restore_hover_after_interaction`) re-sends `mouse_move` to CEF with the post-resize cursor position to restore hover state. This system replaces `restore_hover_after_drag`, and the resource `DragEndPending` is renamed to `InteractionEndPending` to cover both resize and drag.

## 5. Anchor Math

### 5.1 The pinned-corner rule

For any zone, a specific point in the webview's local 2D frame must stay fixed in world space during resize:

| Zone | Pinned point (local) |
|---|---|
| N  | Bottom-edge midpoint |
| S  | Top-edge midpoint |
| E  | Left-edge midpoint |
| W  | Right-edge midpoint |
| NE | SW corner |
| NW | SE corner |
| SE | NW corner |
| SW | NE corner |

Corners win over edges. **Phase 1 requires a centered origin** (sprites: `Anchor::Center`, the default; meshes: standard `Plane3d::new(...)`, which is centered at origin). Debug-asserted at `on_add`. Non-centered origins are a Phase 2 concern.

### 5.2 Universal `apply_resize` function

```rust
fn apply_resize(
    zone: ResizeZone,
    start_size: Vec2,
    start_translation: Vec3,
    du: f32,              // world-space delta projected onto u_axis
    dv: f32,              // world-space delta projected onto v_axis
    u_axis: Vec3,
    v_axis: Vec3,
    lock_aspect: bool,
    min_size: Vec2,
    max_size: Option<Vec2>,
) -> (Vec2, Vec3) {
    // 1. Raw size deltas (signed by zone).
    let (dw_raw, dh_raw) = match zone {
        ResizeZone::E  => ( du,   0.0),
        ResizeZone::W  => (-du,   0.0),
        ResizeZone::S  => ( 0.0,  dv),
        ResizeZone::N  => ( 0.0, -dv),
        ResizeZone::NE => ( du,  -dv),
        ResizeZone::NW => (-du,  -dv),
        ResizeZone::SE => ( du,   dv),
        ResizeZone::SW => (-du,   dv),
    };
    let mut new_size = Vec2::new(start_size.x + dw_raw, start_size.y + dh_raw);

    // 2. Aspect lock: dominant axis wins.
    if lock_aspect {
        let aspect = start_size.x / start_size.y;
        if dw_raw.abs() * (1.0 / aspect) > dh_raw.abs() {
            new_size.y = new_size.x / aspect;
        } else {
            new_size.x = new_size.y * aspect;
        }
    }

    // 3. Clamp.
    new_size = new_size.max(min_size);
    if let Some(max) = max_size { new_size = new_size.min(max); }

    // 4. Actual deltas after clamping.
    let actual_dw = new_size.x - start_size.x;
    let actual_dh = new_size.y - start_size.y;

    // 5. Translation delta (pinned-corner rule for centered origin).
    let sign_u = match zone {
        ResizeZone::E | ResizeZone::NE | ResizeZone::SE =>  0.5,
        ResizeZone::W | ResizeZone::NW | ResizeZone::SW => -0.5,
        _ => 0.0,
    };
    let sign_v = match zone {
        ResizeZone::S | ResizeZone::SE | ResizeZone::SW =>  0.5,
        ResizeZone::N | ResizeZone::NE | ResizeZone::NW => -0.5,
        _ => 0.0,
    };
    let new_translation = start_translation
        + u_axis * (actual_dw * sign_u)
        + v_axis * (actual_dh * sign_v);

    (new_size, new_translation)
}
```

Using `actual_dw`/`dh` (after clamping) rather than the raw deltas ensures that hitting `min_size` or `max_size` stops the translation shift at the clamp — the anchor stays correct even at the limit.

### 5.3 3D mesh — `DisplaySize` → `Transform.scale.xy`

The pipeline owns `Transform.scale.xy` on resizable meshes. Phase 1 requires Z-normal planes (`Plane3d::new(Vec3::Z, ...)`) so that the webview's width/height correspond to local X/Y. When `DisplaySize` changes:

```
Transform.scale.xy = DisplaySize / WebviewBasis2d.local_size
```

`Transform.scale.z` is untouched. User-authored `Transform.scale.xy` on `WebviewResizable` entities is overwritten — documented as "use a parent entity if you need animation-driven scale on a resizable mesh".

Non-Z-normal planes (e.g., `Vec3::Y`) would require mapping width/height to `scale.xz` instead — deferred to Phase 2 when `WebviewBasis2d` can express arbitrary axis mappings.

### 5.4 2D sprite — `DisplaySize` → `sprite.custom_size`

Simpler: `sprite.custom_size = Some(display_size)`, `u_axis = Vec3::X`, `v_axis = Vec3::Y`. The same `apply_resize` function works without modification. Sprites skip the `WebviewBasis2d` component.

### 5.5 Worked example

- Spawn: `Plane3d::new(Vec3::Z, Vec2::ONE)` (AABB 2×2 world units), `Transform::default()`, `WebviewSize(800, 800)`, `WebviewResizable::default()`.
- `on_add` snapshot: `DisplaySize = (2, 2)`, `BaseRenderScale = (400, 400)`, `QualityMultiplier = 1.0`, `WebviewDpr = 1.0`, `Transform.scale.xy = (1, 1)`, `WebviewBasis2d { u_axis: X, v_axis: Y, local_size: (2, 2) }`.
- User drags E edge +1 world unit: `du = 1.0`, `dv = 0.0`.
- `apply_resize(E, ...)` → `new_size = (3, 2)`, `new_translation = (0.5, 0, 0)`.
- Pipeline: `Transform.scale.xy = (3, 2) / (2, 2) = (1.5, 1.0)`, `WebviewSize = round((3, 2) × (400, 400) × 1 × 1) = (1200, 800)`.
- Visual: mesh spans world x ∈ [-1, 2]; west edge world x = 0.5 - (3 × 1.5)/2 ... wait, that's wrong; the AABB is 2×2 times scale 1.5 = 3. So world extent is 3, centered at 0.5 → x ∈ [-1, 2]. Pinned west edge is at x = -1. ✓

## 6. Testing Strategy

### 6.1 Unit tests (extends `src/drag.rs` test module pattern)

**Zone classification** (new `src/resize.rs`):

- `classify_zone_interior_returns_none`
- `classify_zone_north_edge`
- `classify_zone_nw_corner_wins_over_n_and_w`  (corner-over-edge priority)
- `classify_zone_outside_webview_returns_none`
- `classify_zone_respects_edge_thickness_config`
- `classify_zone_min_interior_guard`

**Anchor math** — one per zone, each asserting the pinned point's world position is preserved:

- `apply_resize_east_pins_west_edge`
- `apply_resize_west_pins_east_edge`
- `apply_resize_north_pins_south_edge`
- `apply_resize_south_pins_north_edge`
- `apply_resize_ne_pins_sw_corner`
- `apply_resize_nw_pins_se_corner`
- `apply_resize_se_pins_nw_corner`
- `apply_resize_sw_pins_ne_corner`

**Pipeline derivation** — pure function:

- `derive_sprite_pixel_size_from_display_and_dpr`
- `derive_mesh_pixel_size_preserves_initial_ratio`
- `derive_clamps_to_min_size`
- `derive_clamps_to_max_size`
- `derive_rounds_fractional_to_int`
- `derive_dedupes_on_integer_change`

**Aspect lock**:

- `aspect_lock_dominant_axis_drives_other`
- `aspect_lock_preserves_start_ratio`
- `aspect_lock_respects_min_max_clamp`

**Hit priority** (extend `src/drag.rs` tests):

- `resize_edge_wins_over_drag_region_at_boundary`
- `interior_drag_region_unchanged_by_resize_edges`

**Pipeline compatibility**:

- `non_resizable_webview_skips_pipeline`  (no `DisplaySize` etc. inserted, `WebviewSize` directly writable)

### 6.2 Integration example — `examples/resize.rs`

A scene with a 3D mesh webview (primary) and a 2D sprite webview (secondary), both resizable. The HTML displays `window.innerWidth × window.innerHeight` live, so the user can confirm CSS viewport actually reflows.

```rust
// 3D mesh: default config
commands.spawn((
    WebviewSource::local("resize_demo.html"),
    WebviewSize(UVec2::new(800, 600)),
    Mesh3d(meshes.add(Plane3d::new(Vec3::Z, Vec2::ONE))),
    MeshMaterial3d(materials.add(WebviewExtendStandardMaterial::default())),
    Transform::from_translation(Vec3::ZERO),
    WebviewResizable::default(),
));

// 2D sprite: tighter min/max
commands.spawn((
    WebviewSource::local("resize_demo.html"),
    WebviewSize(UVec2::new(400, 300)),
    Sprite { custom_size: Some(Vec2::new(400.0, 300.0)), ..default() },
    WebviewSpriteMaterial::default(),
    Transform::from_translation(Vec3::new(500.0, 0.0, 0.0)),
    WebviewResizable {
        edge_thickness: 12,
        min_size: UVec2::new(200, 150),
        max_size: Some(UVec2::new(1200, 900)),
        aspect_lock: AspectLockMode::LockOnShift,
    },
));
```

Companion `assets/resize_demo.html` shows live dimensions, has a `-webkit-app-region: drag` title bar (to prove drag+resize coexist), and uses a gradient background to make the resize visually obvious.

### 6.3 Manual test checklist

- [ ] Drag each of the 8 zones; opposite edge/corner stays pinned
- [ ] Cursor changes to correct direction on each edge hover
- [ ] Page's `window.innerWidth`/`innerHeight` updates live during drag
- [ ] Text stays readable after resize (not pixelated — confirms `WebviewSize` scales)
- [ ] Drag title bar still moves the webview (drag + resize coexist)
- [ ] Shift + drag locks aspect ratio; release Shift unlocks mid-drag
- [ ] `min_size`/`max_size` stop the resize at the limits without breaking anchor
- [ ] 2D sprite webview resizes correctly alongside 3D mesh
- [ ] Existing examples (`simple`, `sprite`, `toolbar_drag`) unchanged in behavior
- [ ] No visible first-frame size flash on spawn (verifies system ordering)

### 6.4 Platform coverage

- **macOS** (primary dev target): full manual pass on Retina (DPR 2.0). Verify no coordinate drift between mouse position and resize edge hit-test.
- **Windows**: full manual pass via `cargo run --example resize`. Resize goes through `browsers.resize()` on both `NonSend<Browsers>` (macOS) and `BrowsersProxy` (Windows) paths because it writes `WebviewSize` and lets the existing `resize_win` system pick it up — no new IPC plumbing.
- **Linux**: out of scope.

### 6.5 Out of scope for Phase 1 tests

- Quality profile interactions (Phase 2).
- Monitor-DPR transitions (Phase 2).
- Animated / parent-scaled webviews (documented as unsupported).
- Non-centered origins (Phase 2).

## 7. Release Criteria

Phase 1 is ready to merge when:

1. All unit tests listed in §6.1 pass.
2. `examples/resize.rs` passes the manual checklist in §6.3 on macOS and Windows.
3. All existing examples (`simple`, `sprite`, `toolbar_drag`, `inline_html`, `js_emit`, `host_emit`, `brp`, `navigation`, `zoom_level`, `devtool`, `custom_material`, `preload_scripts`, `extensions`) run unchanged.
4. `make fix-lint` produces no warnings.
5. No behavior change on webviews without `WebviewResizable`.
6. The pipeline's component set (`DisplaySize`, `BaseRenderScale`, `QualityMultiplier`, `WebviewDpr`, `WebviewBasis2d`) is documented with rustdoc, including Phase 2 forward-compatibility notes on `QualityMultiplier` and `WebviewDpr`.

## 8. Open Questions (Phase 2 deferrals)

These are explicitly **not** answered in Phase 1 and are flagged for the quality profile spec follow-up:

1. **Pipeline without resize UX**: Phase 2's quality scaling needs the derive pipeline (`WebviewSize = DisplaySize × BaseRenderScale × QualityMul × DPR`) to apply to webviews that are NOT user-resizable (e.g., a billboard that should be `crisp` but not draggable). Solution direction: introduce a `WebviewQuality` component that activates the pipeline without the resize edges. The pipeline filter would become `Or<(With<WebviewResizable>, With<WebviewQuality>)>`. Phase 1's pipeline code already supports this — the split is purely which component triggers `on_add` initialization.
2. How does `QualityMultiplier` compose with existing user code that sets `WebviewSize` directly on a non-resizable webview? Probably: adding `WebviewQuality` auto-adds pipeline components. TBD in Phase 2.
3. Monitor-transition DPR updates — requires hooking `WindowEvent::ScaleFactorChanged`.
4. Quality profile API shape (`balanced`/`crisp`/`ultra` enum vs explicit scalar).
5. Device-pixel max-size clamp (texture size limits per GPU).
6. Observability surface (FR-007): events, logs, or a debug UI.
7. Non-Z-normal plane support — `WebviewBasis2d` axis mapping + `Transform.scale` axis writeback for arbitrary plane orientations.
