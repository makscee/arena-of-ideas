pub use std::time::Duration;

pub use crate::{nodes::*, plugins::*, resources::*, ui::*, utils::*};
pub use anyhow::{anyhow, Context as _, Result};

pub use crate::stdb::*;
pub use bevy::{app::PreUpdate, prelude::In};
pub use bevy::{
    app::{prelude::PluginGroup, App, Plugin, Startup, Update},
    asset::{Asset, Assets, Handle},
    audio::{AudioSource, PlaybackSettings},
    color::{Color, LinearRgba, Mix},
    core::Name,
    diagnostic::DiagnosticsStore,
    ecs::{
        component::Component,
        entity::Entity,
        query::{Or, With},
        schedule::IntoSystemConfigs,
        system::{Query, Res, ResMut, Resource, SystemParam},
        world::{Mut, World},
    },
    hierarchy::{Children, DespawnRecursiveExt, Parent},
    input::{keyboard::KeyCode, ButtonInput},
    log::{debug, error, info, *},
    math::{
        cubic_splines::{CubicBezier, CubicGenerator},
        primitives::{Circle, Rectangle},
        vec2, vec3, vec4, Vec2, Vec3, Vec3Swizzles, Vec4, Vec4Swizzles,
    },
    prelude::{default, BuildChildren, Commands},
    reflect::TypePath,
    render::{
        camera::Camera,
        mesh::{Mesh, MeshVertexBufferLayout, MeshVertexBufferLayoutRef, PrimitiveTopology},
        render_resource::{AsBindGroup, PolygonMode, RenderPipelineDescriptor},
        view::Visibility,
    },
    sprite::Material2d,
    state::{
        condition::{in_state, state_changed},
        state::{NextState, OnEnter, OnExit, State, States},
    },
    time::Time,
    transform::components::{GlobalTransform, Transform},
    utils::hashbrown::{HashMap, HashSet},
    DefaultPlugins,
};
pub use bevy::{ecs::system::RunSystemOnce, prelude::Without};
pub use bevy_asset_loader::{
    asset_collection::AssetCollection,
    loading_state::{config::ConfigureLoadingState, LoadingState, LoadingStateAppExt},
    standard_dynamic_asset::StandardDynamicAssetCollection,
};
pub use bevy_common_assets::ron::RonAssetPlugin;
pub use bevy_egui::egui::{
    self, epaint::TextShape, Align, CollapsingHeader, Color32, CornerRadius, Frame, Layout, Margin,
    Rect, Shadow, Stroke, Ui,
};
pub use bevy_egui::{
    egui::{epaint::PathShape, pos2, Align2, Id, Pos2},
    EguiContext,
};
pub use bevy_tasks::IoTaskPool;
pub use chrono::DateTime;
pub use colored::{Colorize, CustomColor};
pub use convert_case::{Case, Casing};
pub use ecolor::hex_color;
pub use egui::{
    emath::{self, Float, Rot2, TSTransform},
    epaint::{self, TessellationOptions},
    include_image, remap,
    style::{HandleShape, Spacing, WidgetVisuals, Widgets},
    text::LayoutJob,
    Area, CentralPanel, Checkbox, DragValue, FontData, FontDefinitions, FontFamily, FontId, Image,
    Key, Label, NumExt, Order, Response, RichText, ScrollArea, Sense, SidePanel, Style, TextFormat,
    TextStyle, TopBottomPanel, Widget, WidgetText,
};
pub use egui_dock::DockState;
pub use egui_dock::Tree;
pub use epaint::{CircleShape, RectShape, Tessellator};
pub use include_dir::Dir;
pub use indexmap::IndexMap;
pub use itertools::EitherOrBoth;
pub use itertools::Itertools;
pub use macro_client::*;
pub use once_cell::sync::OnceCell;
pub use parking_lot::{const_mutex, Mutex};
pub use rand::{seq::IteratorRandom, thread_rng, Rng, SeedableRng};
pub use rand_chacha::ChaCha8Rng;
pub use ron::{
    extensions::Extensions,
    ser::{to_string_pretty, PrettyConfig},
};
pub use schema::*;
pub use serde::{Deserialize, Serialize};
pub use spacetimedb_lib::Identity;
pub use spacetimedb_sats::serde::SerdeWrapper;
pub use spacetimedb_sdk::Table as SdkTable;
pub use std::cell::LazyCell;
pub use std::collections::VecDeque;
pub use std::{
    cell::RefCell,
    cmp::Ordering,
    f32::consts::{PI, TAU},
    fmt::Debug,
    hash::{DefaultHasher, Hash, Hasher},
    mem,
    ops::{Deref, DerefMut},
    path::PathBuf,
    str::FromStr,
    sync::{MutexGuard, RwLock},
    time::UNIX_EPOCH,
};
pub use strum::IntoEnumIterator;
pub use strum_macros::{AsRefStr, Display, EnumIter, EnumString, FromRepr};
pub use ui::*;
pub use utils_client::{get_parent, ToC32, ToColor, *};
