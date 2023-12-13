pub use std::time::Duration;

pub use crate::components::*;
pub use crate::materials::*;
pub use crate::plugins::*;
pub use crate::resourses::*;
pub use anyhow::Context as _;
pub use anyhow::{anyhow, Result};

pub use bevy::{
    app::*,
    asset::*,
    core::*,
    core_pipeline::{clear_color::*, core_2d::*},
    ecs::{
        component::*,
        entity::*,
        event::Event as BevyEvent,
        event::EventReader,
        event::EventWriter,
        query::*,
        schedule::{common_conditions::*, *},
        system::*,
        world::*,
    },
    hierarchy::*,
    input::{common_conditions::input_toggle_active, keyboard::*, *},
    log::LogPlugin,
    math::*,
    math::{vec2, vec3},
    reflect::{Reflect, TypePath, TypeUuid},
    render::{
        camera::{Camera, OrthographicProjection, ScalingMode},
        color::*,
        mesh::{Mesh, MeshVertexBufferLayout, PrimitiveTopology},
        render_resource::AsBindGroup,
        view::*,
    },
    sprite::{Material2d, Material2dPlugin, MaterialMesh2dBundle},
    text::*,
    time::*,
    transform::components::*,
    utils::*,
    DefaultPlugins,
};
pub use bevy_asset_loader::prelude::*;
pub use bevy_common_assets::ron::RonAssetPlugin;
pub use bevy_egui::egui;
pub use bevy_egui::egui::{Frame, Margin};
pub use bevy_egui::{
    egui::{
        pos2, style::WidgetVisuals, text::LayoutJob, Align2, Area, Button, CentralPanel,
        CollapsingHeader, Color32, ComboBox, FontData, FontDefinitions, FontFamily, FontId, Id,
        InnerResponse, Label, Layout, Painter, Pos2, Response, RichText, Rounding, Slider, Stroke,
        TextEdit, TextFormat, TextStyle, TopBottomPanel, Ui, Widget, WidgetText, Window,
    },
    EguiContexts,
};
pub use bevy_kira_audio::{
    Audio, AudioChannel, AudioControl, AudioInstance, AudioTween, PlaybackState,
};
pub use bevy_mod_picking::prelude::*;
pub use bevy_pkv::PkvStore;
pub use colored::Colorize;
pub use ecolor::hex_color;
pub use itertools::Itertools;
pub use log::*;
pub use rand::{thread_rng, Rng};
pub use serde::*;
pub use std::mem;
pub use strum::IntoEnumIterator;
pub use strum_macros::{AsRefStr, Display, EnumIter, EnumString};
pub use utils::*;

pub use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
pub use clap::{Parser, ValueEnum};
pub use materials::prelude::module_bindings::connect;
pub use spacetimedb_sdk;
pub use spacetimedb_sdk::identity::Identity;
pub use spacetimedb_sdk::{
    identity::{identity, once_on_connect, Credentials},
    Address,
};

pub use crate::module_bindings::Ladder as TableLadder;
pub use spacetimedb_sdk::table::*;
pub use std::str::FromStr;
