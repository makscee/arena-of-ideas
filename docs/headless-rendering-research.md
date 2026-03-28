# Headless Rendering Research for Bevy 0.18

Research date: 2026-03-27 | Stack: Bevy 0.18.1 + egui + wgpu

---

## Executive Summary

There are **three viable approaches** ranked by practicality:

1. **Hidden window (`visible: false`)** - Easiest, works today on macOS and Linux with GPU
2. **`RenderTarget::Image` + no primary window** - True headless, but requires GPU/software Vulkan
3. **Xvfb (Linux) / hidden window (macOS)** - Battle-tested for CI

True windowless rendering (no display server at all) with Bevy's full pipeline is **not natively supported** but can be achieved with workarounds.

---

## 1. Bevy `RenderTarget::Image` - Render Camera to Image Handle

### Confirmed Working in Bevy 0.18.1

The `RenderTarget` enum has these variants:

```rust
pub enum RenderTarget {
    Window(WindowRef),           // Render to a window swapchain
    Image(ImageRenderTarget),    // Render to an Image asset
    TextureView(ManualTextureViewHandle), // Render to external texture (e.g. OpenXR)
    None { size: UVec2 },        // No color target (depth prepass only)
}
```

**`RenderTarget::Image`** is the key. From `examples/3d/render_to_texture.rs`:

```rust
// Create an offscreen render target image
let image = Image::new_target_texture(512, 512, TextureFormat::Rgba8Unorm, Some(TextureFormat::Rgba8UnormSrgb));
let image_handle = images.add(image);

// Camera renders to the image, not a window
commands.spawn((
    Camera3d::default(),
    Camera { order: -1, clear_color: Color::WHITE.into(), ..default() },
    RenderTarget::Image(image_handle.clone().into()),
    Transform::from_translation(Vec3::new(0.0, 0.0, 15.0)).looking_at(Vec3::ZERO, Vec3::Y),
));
```

**The catch:** The `render_to_texture` example still uses `DefaultPlugins` which creates a window. The Image target camera works as a *secondary* camera. But the rendering pipeline (wgpu device, queue, etc.) is initialized by `bevy_winit` through the window system.

### Can it work without ANY window?

Setting `WindowPlugin { primary_window: None, exit_condition: ExitCondition::DontExit }` disables the primary window. However, `bevy_winit` still needs to initialize the wgpu device. **The critical question is whether Bevy's `RenderPlugin` can initialize wgpu without a window surface.**

Current answer: **No, not out of the box.** Bevy's render initialization is tightly coupled to the windowing system. The `WinitPlugin` creates the wgpu instance and adapter using the window's display handle. Without a window, the render plugin may fail to find a compatible adapter.

**Workaround:** You can still disable the primary window if you provide a custom `RenderPlugin` configuration or use a software Vulkan driver that supports surfaceless operation.

---

## 2. Bevy Screenshot Module

### Source: `bevy_render::view::window::screenshot`

The `Screenshot` component supports three targets:

```rust
impl Screenshot {
    pub fn window(window: Entity) -> Self          // Capture from window
    pub fn primary_window() -> Self                 // Capture from primary window
    pub fn image(image: Handle<Image>) -> Self      // Capture from image render target
    pub fn texture_view(texture_view: ManualTextureViewHandle) -> Self // Capture from manual texture
}
```

**`Screenshot::image(handle)`** can capture from a `RenderTarget::Image` camera. This does NOT require a window -- it reads directly from the GPU image asset. The screenshot system handles `NormalizedRenderTarget::Image` by reading from `RenderAssets<GpuImage>`.

### How capture works internally:

1. System extracts screenshot requests in `ExtractSchedule`
2. `prepare_screenshots` creates a staging texture + buffer for each target
3. For `NormalizedRenderTarget::Image`: reads directly from the `GpuImage` asset
4. For `NormalizedRenderTarget::Window`: reads from the window's swapchain
5. GPU copies texture to CPU-readable buffer, sends `Image` via channel
6. `ScreenshotCaptured` event fires with the image data

**Key finding:** The screenshot pipeline for `RenderTarget::Image` targets does NOT touch the window system at all. If you can get the renderer initialized, image-target screenshots work independently.

---

## 3. wgpu Headless (Without Surface)

### Confirmed: wgpu supports surfaceless device creation

wgpu can create an `Instance`, `Adapter`, and `Device` without ever creating a surface. This is how `egui_kittest` works (see section 5).

The key API (from egui_wgpu):
```rust
let setup = egui_wgpu::WgpuSetupCreateNew::without_display_handle();
// Creates wgpu instance + adapter without any window/surface
```

At the raw wgpu level:
```rust
let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor { .. });
let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
    compatible_surface: None,  // No surface needed!
    ..default()
}).await;
let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor { .. }).await;
```

### The Bevy problem

Bevy's `RenderPlugin` and `WinitPlugin` don't expose a clean way to initialize wgpu without a window. The render device creation is handled internally by `bevy_winit` during window creation. To use surfaceless wgpu with Bevy, you would need to either:

1. **Fork/patch `bevy_render`** to support surfaceless initialization
2. **Use a custom plugin** that replaces `WinitPlugin` + `RenderPlugin` initialization
3. **Use the hidden window approach** (much simpler)

---

## 4. Software Rendering Backends

### lavapipe (Mesa's software Vulkan) - CONFIRMED WORKING

**This is what wgpu uses in CI.** From wgpu's GitHub Actions:

- Linux: Installs Mesa (currently v25.2.7) with `libvulkan_lvp.so` (lavapipe)
- Sets `VK_DRIVER_FILES` to point to lavapipe's ICD JSON
- Sets `LD_LIBRARY_PATH` and `LIBGL_DRIVERS_PATH`
- All wgpu GPU tests run against lavapipe on Linux CI

**Installation on Linux server:**
```bash
# Ubuntu/Debian
apt-get install mesa-vulkan-drivers

# Or build from source / use pre-built binaries like wgpu does
# Set environment:
export VK_DRIVER_FILES=/path/to/lavapipe_icd.json
export LD_LIBRARY_PATH=/path/to/mesa/lib
```

lavapipe reports as `DeviceType::Cpu` to wgpu, so adapter selection that prefers CPU devices will pick it up.

### SwiftShader (Google's software Vulkan)

- Also works with wgpu
- Commonly used in Android emulators and Chrome's headless GPU tests
- Reports as CPU device type
- Available as pre-built binaries
- **Not used by wgpu CI** (they use lavapipe instead)

### macOS - Metal without display

- **Metal works without a display server** on macOS. The Metal device can be created without any window.
- macOS CI runners (GitHub Actions `macos-14`) run wgpu GPU tests using the real Metal GPU
- On headless macOS (e.g., SSH session), Metal still works because it's a system framework, not tied to a display
- **However**, `bevy_winit` still needs a window to initialize. On macOS, creating a window from a non-GUI context (SSH, launchd service) can fail or cause issues with the Cocoa event loop.

### Verdict on software rendering

| Platform | Backend | Status |
|----------|---------|--------|
| Linux (no GPU) | lavapipe (Mesa) | Works with wgpu, used in CI |
| Linux (with GPU) | Native Vulkan | Works normally |
| macOS | Metal | Works without display, but Bevy needs window init |
| macOS | MoltenVK + lavapipe | Theoretically possible but unusual |

---

## 5. egui_kittest - How It Works

### Architecture (from source code analysis)

egui_kittest renders egui to images **completely without any window or display**. Here's how:

1. **`WgpuSetupCreateNew::without_display_handle()`** - Creates wgpu instance with no display handle
2. **Adapter selection prefers CPU** (`DeviceType::Cpu` ranked first, then `DiscreteGpu`)
3. **`RenderState::create(..., surface: None, ...)`** - No surface provided
4. **Renders to offscreen texture:**
   ```rust
   let texture = device.create_texture(&wgpu::TextureDescriptor {
       usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_SRC,
       ...
   });
   ```
5. **Copies texture to CPU buffer** via `texture_to_image()` which:
   - Creates a `MAP_READ | COPY_DST` buffer
   - Encodes `copy_texture_to_buffer`
   - Maps the buffer and reads pixels
   - Returns `image::RgbaImage`

### Can it be used with bevy_egui?

**Not directly.** egui_kittest operates on a standalone `egui::Context` -- it runs the egui UI function, tessellates, and renders using its own `egui_wgpu::Renderer`. It has no awareness of Bevy's ECS, rendering pipeline, or bevy_egui integration.

To capture bevy_egui UI, you would need to:
1. Extract the `egui::FullOutput` from bevy_egui after a frame
2. Feed it to a standalone `WgpuTestRenderer`
3. This would capture **only the egui layer**, not Bevy's 2D/3D scene behind it

### What it CAN'T do

- Cannot render Bevy's 2D/3D scene (sprites, meshes, cameras)
- Cannot render bevy_egui's integration layer (egui painted over Bevy's scene)
- Cannot handle Bevy-specific features (render layers, post-processing, etc.)

### What it demonstrates

The egui_kittest wgpu renderer proves that **wgpu headless rendering to images works reliably** on all platforms, including macOS Metal and Linux lavapipe. The pattern could theoretically be replicated for Bevy's renderer, but it would require significant custom code.

---

## 6. Xvfb Approach (Virtual Framebuffer)

### Linux - Xvfb

**Proven approach.** Provides a virtual X11 display that applications can render to without a physical monitor.

```bash
# Install
apt-get install xvfb

# Run Bevy app with virtual display
Xvfb :99 -screen 0 1920x1080x24 &
export DISPLAY=:99
./your_bevy_app
```

**Pros:**
- Works with unmodified Bevy apps
- Full rendering pipeline works (GPU if available, or with lavapipe for software)
- Battle-tested in CI environments

**Cons:**
- Linux only
- Adds a dependency
- Slight overhead from the virtual framebuffer
- Needs X11 backend (not Wayland-native, but Bevy supports X11)

### macOS - No direct equivalent

There is no Xvfb for macOS. Options:

1. **`Window { visible: false }`** - Best option. Creates a window handle for wgpu but doesn't show it on screen. Works on macOS.
2. **Headless Metal** - Metal API itself doesn't need a display, but Bevy's winit integration needs a window handle.
3. **VNC/virtual display** - Tools like `VirtualDisplay` or headless Mac configurations exist but are complex.

### Performance

Xvfb performance is very good -- it's just a memory-backed framebuffer. The GPU (or software renderer) does the actual work; Xvfb just provides the display target. Overhead is negligible compared to actual rendering.

---

## 7. Hidden Window Approach (`Window { visible: false }`)

### RECOMMENDED APPROACH for both macOS and Linux

```rust
App::new()
    .add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            visible: false,
            resolution: WindowResolution::new(1920.0, 1080.0),
            ..default()
        }),
        ..default()
    }))
```

From the Bevy 0.18.1 source, `Window` has:
```rust
pub visible: bool,  // defaults to true
```

Platform support for `visible: false`:
- **macOS**: Supported (window is created but not shown)
- **Windows**: Supported
- **Linux X11**: Supported
- **Android / Wayland / Web**: Documented as "Unsupported"

**Why this works:**
1. The window is created, so `bevy_winit` initializes the full wgpu rendering pipeline
2. The window surface exists, so all rendering works normally
3. The window is invisible, so no screen takeover
4. Screenshots capture from the window surface normally
5. All of Bevy's rendering features work (2D, 3D, egui, post-processing)

**Caveats:**
- On Linux Wayland, `visible: false` may not work -- use X11 backend (`WINIT_UNIX_BACKEND=x11`) or Xvfb
- On a true headless Linux server (no display server at all), you still need Xvfb or similar
- On macOS, the app may still briefly appear in the dock (but the window itself is hidden)

---

## 8. Combined Strategy - Recommended Implementation

### For macOS (development + CI)

```rust
// Hidden window approach
App::new()
    .add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            visible: false,
            resolution: WindowResolution::new(1920.0, 1080.0),
            ..default()
        }),
        ..default()
    }))
    // Screenshot capture
    .add_systems(Update, take_screenshot_system)
```

This works because macOS Metal initializes fine without a visible window. The Cocoa event loop runs, the window exists in memory, but nothing appears on screen.

### For Linux servers (no GPU, no display)

```bash
# Install dependencies
apt-get install xvfb mesa-vulkan-drivers

# Run with virtual display + software Vulkan
Xvfb :99 -screen 0 1920x1080x24 &
export DISPLAY=:99
export VK_DRIVER_FILES=/usr/share/vulkan/icd.d/lvp_icd.x86_64.json
./your_bevy_app
```

The Bevy app itself uses the same hidden-window code. Xvfb provides the display server, lavapipe provides software Vulkan.

### For Linux servers (with GPU, no display)

```bash
# Just need Xvfb, GPU handles rendering
Xvfb :99 -screen 0 1920x1080x24 &
export DISPLAY=:99
./your_bevy_app
```

### Screenshot capture system

```rust
fn take_screenshot_system(mut commands: Commands, frame: Res<FrameCount>) {
    if frame.0 == TARGET_FRAME {
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk("output.png"));
    }
}
```

Or for image-target rendering (no window screenshot):

```rust
// Setup: Camera renders to image
let image = Image::new_target_texture(1920, 1080, TextureFormat::Rgba8Unorm, None);
let image_handle = images.add(image);
commands.spawn((
    Camera2d::default(),
    Camera { order: 0, ..default() },
    RenderTarget::Image(image_handle.clone().into()),
));

// Capture: Screenshot from image target
commands
    .spawn(Screenshot::image(image_handle.clone()))
    .observe(save_to_disk("output.png"));
```

---

## Approach Comparison Matrix

| Approach | macOS | Linux (GPU) | Linux (no GPU) | Complexity | Full Bevy Rendering |
|----------|-------|-------------|----------------|------------|-------------------|
| Hidden window | YES | YES (X11/Xvfb) | YES (Xvfb+lavapipe) | Low | YES |
| RenderTarget::Image (no window) | MAYBE* | MAYBE* | MAYBE* | High | YES minus egui** |
| Xvfb + normal window | N/A | YES | YES (+ lavapipe) | Low | YES |
| egui_kittest pattern | YES | YES | YES (lavapipe) | Very High | NO (egui only) |
| Pure headless (no render) | YES | YES | YES | Low | NO |

*Requires custom RenderPlugin init to bypass window requirement
**bevy_egui renders to the window, not to image targets, without modification

---

## Key Findings

1. **`Window { visible: false }` is the simplest path** that works today with zero code changes to Bevy. Combine with Xvfb on headless Linux.

2. **`RenderTarget::Image` + `Screenshot::image()`** works for capturing rendered frames, but the render pipeline still needs a window to initialize. You can use this as a secondary camera alongside a hidden primary window.

3. **True windowless Bevy rendering is not supported** out of the box. Bevy's render initialization is coupled to `bevy_winit`'s window creation. This is a known limitation.

4. **lavapipe is the proven software Vulkan** for Linux headless -- wgpu's own CI uses it.

5. **egui_kittest proves the wgpu pattern works** (surfaceless device, render to texture, copy to CPU) but applying this to Bevy's full renderer would require replacing core initialization code.

6. **macOS Metal works without a display** at the API level, but Bevy still needs a window handle for initialization.
