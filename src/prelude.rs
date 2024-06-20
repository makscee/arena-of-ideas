pub use std::time::Duration;

pub use crate::components::*;
pub use crate::plugins::*;
pub use crate::resources::*;
pub use crate::utils::*;
pub use anyhow::Context as _;
pub use anyhow::{anyhow, Result};

pub use crate::stdb::*;
pub use bevy::app::{prelude::PluginGroup, App, Plugin};
pub use bevy::core::Name;
pub use bevy::ecs::query::With;
pub use bevy::ecs::schedule::common_conditions::in_state;
pub use bevy::ecs::world::Mut;
pub use bevy::transform::TransformBundle;
pub use bevy::{
    app::{Startup, Update},
    asset::{Asset, Assets, Handle},
    diagnostic::DiagnosticsStore,
    ecs::{
        component::Component,
        entity::Entity,
        schedule::{
            common_conditions::state_changed, IntoSystemConfigs, NextState, OnEnter, State, States,
        },
        system::{Query, Res, ResMut, Resource},
        world::World,
    },
    hierarchy::{BuildWorldChildren, Children, DespawnRecursiveExt, Parent},
    input::{keyboard::KeyCode, ButtonInput},
    log::{debug, error, info},
    math::{
        cubic_splines::{CubicBezier, CubicGenerator},
        primitives::{Circle, Rectangle},
        vec2, vec3, vec4, Vec2, Vec3, Vec4, Vec4Swizzles,
    },
    prelude::default,
    reflect::TypePath,
    render::{
        camera::Camera,
        color::Color,
        mesh::{Mesh, MeshVertexBufferLayout, PrimitiveTopology},
        render_resource::{AsBindGroup, PolygonMode, RenderPipelineDescriptor},
        view::{Visibility, VisibilityBundle},
    },
    sprite::{Material2d, MaterialMesh2dBundle, Mesh2dHandle},
    text::{Text, Text2dBundle},
    transform::components::{GlobalTransform, Transform},
    utils::hashbrown::HashMap,
    DefaultPlugins,
};
pub use bevy_asset_loader::asset_collection::AssetCollection;
pub use bevy_asset_loader::{
    loading_state::{config::ConfigureLoadingState, LoadingState, LoadingStateAppExt},
    standard_dynamic_asset::StandardDynamicAssetCollection,
};
pub use bevy_common_assets::ron::RonAssetPlugin;
pub use bevy_egui::{
    egui::{self, epaint::PathShape, pos2, Align2, Id, Pos2, Stroke, Ui},
    EguiContext,
};
pub use chrono::DateTime;
pub use convert_case::Casing;
pub use ecolor::Color32;
pub use egui::{
    epaint::Shadow,
    style::{HandleShape, Spacing, WidgetVisuals, Widgets},
    CentralPanel, Frame, Layout, Margin, Response, RichText, Rounding, SidePanel, Slider,
    TopBottomPanel, Widget, WidgetText,
};
pub use itertools::Itertools;
pub use lazy_static::lazy_static;
pub use serde::{Deserialize, Serialize};
pub use std::cmp::Ordering;
pub use std::mem;
pub use std::ops::Deref;
pub use std::sync::{Mutex, MutexGuard};
pub use std::time::UNIX_EPOCH;
pub use strum::IntoEnumIterator;
pub use strum_macros::{AsRefStr, Display, EnumIter, EnumString};
