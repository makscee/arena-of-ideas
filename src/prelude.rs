pub use std::time::Duration;

pub use crate::{nodes::*, plugins::*, resources::*, utils::*};
pub use anyhow::{anyhow, Context as _, Result};

pub use crate::stdb::*;
pub use bevy::ecs::system::SystemParam;
pub use bevy::math::Vec3Swizzles;
pub use bevy::{
    app::{prelude::PluginGroup, App, Plugin, Startup, Update},
    asset::{Asset, Assets, Handle},
    audio::{AudioBundle, AudioSource, PlaybackSettings},
    color::{Color, LinearRgba},
    core::Name,
    diagnostic::DiagnosticsStore,
    ecs::{
        component::Component,
        entity::Entity,
        query::{Or, With},
        schedule::IntoSystemConfigs,
        system::{Query, Res, ResMut, Resource},
        world::{Mut, World},
    },
    hierarchy::{BuildWorldChildren, Children, DespawnRecursiveExt, Parent},
    input::{keyboard::KeyCode, ButtonInput},
    log::{debug, error, info},
    math::{
        cubic_splines::{CubicBezier, CubicGenerator},
        primitives::{Circle, Rectangle},
        vec2, vec3, vec4, Vec2, Vec3, Vec4, Vec4Swizzles,
    },
    prelude::{default, Commands},
    reflect::TypePath,
    render::{
        camera::Camera,
        mesh::{Mesh, MeshVertexBufferLayout, MeshVertexBufferLayoutRef, PrimitiveTopology},
        render_resource::{AsBindGroup, PolygonMode, RenderPipelineDescriptor},
        view::{Visibility, VisibilityBundle},
    },
    sprite::{Material2d, MaterialMesh2dBundle, Mesh2dHandle},
    state::{
        condition::{in_state, state_changed},
        state::{NextState, OnEnter, OnExit, State, States},
    },
    text::{Text, Text2dBundle},
    time::Time,
    transform::{
        bundles::TransformBundle,
        components::{GlobalTransform, Transform},
    },
    utils::hashbrown::{HashMap, HashSet},
    DefaultPlugins,
};
pub use bevy::{color::Mix, log::*, prelude::BuildChildren};
pub use bevy_asset_loader::{
    asset_collection::AssetCollection,
    loading_state::{config::ConfigureLoadingState, LoadingState, LoadingStateAppExt},
    standard_dynamic_asset::StandardDynamicAssetCollection,
};
pub use bevy_common_assets::ron::RonAssetPlugin;
pub use bevy_egui::egui::{
    self, epaint::TextShape, Align, CollapsingHeader, Color32, Frame, Layout, Margin, Rect,
    Rounding, Shadow, Stroke, Ui,
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
pub use egui::{emath, remap};
pub use egui::{
    emath::Float,
    epaint::{self},
    include_image,
    style::{HandleShape, Spacing, WidgetVisuals, Widgets},
    text::LayoutJob,
    Area, CentralPanel, FontData, FontDefinitions, FontFamily, FontId, Image, Label, NumExt, Order,
    Response, RichText, Sense, SidePanel, Style, TextFormat, TextStyle, TopBottomPanel, Widget,
    WidgetText,
};
pub use egui::{
    emath::{Rot2, TSTransform},
    epaint::TessellationOptions,
    Checkbox, DragValue,
};
pub use egui::{Key, ScrollArea};
pub use epaint::{CircleShape, RectShape, Tessellator};
pub use include_dir::Dir;
pub use indexmap::IndexMap;
pub use itertools::Itertools;
pub use lazy_static::lazy_static;
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
pub use spacetimedb_sdk::Table as SdkTable;
pub use std::hash::{DefaultHasher, Hash, Hasher};
pub use std::mem;
pub use std::sync::RwLock;
pub use std::{
    cell::RefCell,
    f32::consts::TAU,
    fmt::Debug,
    ops::{Deref, DerefMut},
};
pub use std::{
    cmp::Ordering, f32::consts::PI, path::PathBuf, str::FromStr, sync::MutexGuard, time::UNIX_EPOCH,
};
pub use strum::IntoEnumIterator;
pub use strum_macros::{AsRefStr, Display, EnumIter, EnumString, FromRepr};
pub use ui::*;
pub use utils::*;
pub use utils_client::*;
pub use utils_client::{get_parent, *};
pub use utils_client::{ToC32, ToColor};
