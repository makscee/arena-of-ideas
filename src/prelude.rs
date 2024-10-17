pub use std::time::Duration;

pub use crate::{components::*, plugins::*, resources::*, utils::*};
pub use anyhow::{anyhow, Context as _, Result};

pub use crate::stdb::*;
pub use bevy::audio::{AudioBundle, AudioSource, PlaybackSettings};
pub use bevy::{
    app::{prelude::PluginGroup, App, Plugin, Startup, Update},
    asset::{Asset, Assets, Handle},
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
    math::{
        cubic_splines::{CubicBezier, CubicGenerator},
        primitives::{Circle, Rectangle},
        vec2, vec3, vec4, Vec2, Vec3, Vec4, Vec4Swizzles,
    },
    prelude::default,
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
pub use bevy_asset_loader::{
    asset_collection::AssetCollection,
    loading_state::{config::ConfigureLoadingState, LoadingState, LoadingStateAppExt},
    standard_dynamic_asset::StandardDynamicAssetCollection,
};
pub use bevy_common_assets::ron::RonAssetPlugin;
pub use bevy_egui::{
    egui::{self, epaint::PathShape, pos2, Align2, Id, Pos2, Stroke, Ui},
    EguiContext,
};
pub use bevy_tasks::IoTaskPool;
pub use chrono::DateTime;
pub use colored::{Colorize, CustomColor};
pub use convert_case::{Case, Casing};
pub use ecolor::{hex_color, Color32};
pub use egui::{
    emath::Float,
    epaint::{self, Shadow},
    include_image,
    style::{HandleShape, Spacing, WidgetVisuals, Widgets},
    text::LayoutJob,
    Align, Area, CentralPanel, FontData, FontDefinitions, FontFamily, FontId, Frame, Image, Label,
    Layout, Margin, NumExt, Order, Rect, Response, RichText, Rounding, Sense, SidePanel, Style,
    TextFormat, TextStyle, TopBottomPanel, Widget, WidgetText, Window,
};
pub use egui::{Checkbox, DragValue, Key, ScrollArea};
pub use indexmap::IndexMap;
pub use itertools::Itertools;
pub use lazy_static::lazy_static;
pub use log::*;
pub use once_cell::sync::OnceCell;
pub use rand::{seq::IteratorRandom, thread_rng, Rng, SeedableRng};
pub use rand_chacha::ChaCha8Rng;
pub use ron::{
    extensions::Extensions,
    ser::{to_string_pretty, PrettyConfig},
};
pub use serde::{Deserialize, Serialize};
pub use spacetimedb_sdk::{identity::Credentials, reducer::Status as StdbStatus, table::TableType};
pub use std::sync::RwLock;
pub use std::{
    cmp::Ordering,
    f32::consts::PI,
    hash::{DefaultHasher, Hash, Hasher},
    mem,
    ops::Deref,
    path::PathBuf,
    str::FromStr,
    sync::{Mutex, MutexGuard},
    time::UNIX_EPOCH,
};
pub use strum::IntoEnumIterator;
pub use strum_macros::{AsRefStr, Display, EnumIter, EnumString, FromRepr};
