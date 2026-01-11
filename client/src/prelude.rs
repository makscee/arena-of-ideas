pub use std::time::Duration;

pub use crate::stdb::*;

pub use crate::{nodes::*, plugins::*, resources::*, ui::*, utils::*};
pub use anyhow::{anyhow, Context as _, Result};
pub use backtrace::Backtrace;
pub use bevy::{
    app::{prelude::PluginGroup, App, FixedUpdate, Plugin, PreUpdate, Startup, Update},
    asset::{Asset, Assets, Handle},
    audio::{AudioSource, PlaybackSettings},
    color::{Color, LinearRgba, Mix},
    diagnostic::DiagnosticsStore,
    ecs::{
        component::{Component as BevyComponent, Mutable},
        entity::Entity,
        query::{Or, With},
        resource::Resource,
        schedule::IntoScheduleConfigs,
        system::{Query, Res, ResMut, RunSystemOnce, SystemParam},
        world::{Mut, World},
    },
    input::{keyboard::KeyCode, ButtonInput},
    math::{
        cubic_splines::{CubicBezier, CubicGenerator},
        primitives::{Circle, Rectangle},
        vec2, vec3, vec4, Vec2, Vec3, Vec3Swizzles, Vec4, Vec4Swizzles,
    },
    prelude::{
        default, Camera, ChildOf, Children, Commands, In, Mesh, Message, MessageReader, Messages,
        Visibility, Without,
    },
    reflect::{Reflect, TypePath},
    render::render_resource::{AsBindGroup, PolygonMode, RenderPipelineDescriptor},
    state::{
        condition::{in_state, state_changed},
        state::{NextState, OnEnter, OnExit, OnTransition, State, States},
    },
    time::Time,
    transform::components::{GlobalTransform, Transform},
    DefaultPlugins,
};
pub use bevy_egui::{
    egui::{
        self,
        epaint::{PathShape, TextShape},
        pos2, Align, Align2, CollapsingHeader, Color32, CornerRadius, Frame, Id, Layout, Margin,
        Pos2, Rect, Shadow, Stroke, Ui, UiBuilder,
    },
    EguiContext, EguiContexts,
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
pub use egui_tiles::{Tile, TileId, Tiles, Tree};
pub use epaint::{CircleShape, RectShape, Tessellator};
pub use include_dir::Dir;
pub use indexmap::IndexMap;
pub use itertools::{EitherOrBoth, Itertools};
pub use log::*;
pub use once_cell::sync::OnceCell;
pub use parking_lot::{const_mutex, Mutex, MutexGuard};
pub use rand::{rng, seq::IteratorRandom, Rng, SeedableRng};
pub use rand_chacha::ChaCha8Rng;
pub use ron::{
    extensions::Extensions,
    ser::{to_string_pretty, PrettyConfig},
};
pub use schema::*;
pub use serde::{Deserialize, Serialize};
pub use spacetimedb_lib::Identity;
pub use spacetimedb_sats::serde::SerdeWrapper;
pub use spacetimedb_sdk::{DbContext, Table as SdkTable, TableWithPrimaryKey};
pub use std::cell::LazyCell;
pub use std::collections::VecDeque;
pub use std::{
    cell::RefCell,
    cmp::Ordering,
    collections::{HashMap, HashSet},
    f32::consts::{PI, TAU},
    fmt::Debug,
    hash::{DefaultHasher, Hash, Hasher},
    mem,
    ops::{Deref, DerefMut},
    path::PathBuf,
    str::FromStr,
    sync::RwLock,
    time::UNIX_EPOCH,
};
pub use strum::IntoEnumIterator;
pub use strum_macros::{AsRefStr, Display, EnumIter, EnumString, FromRepr};
pub use utils_client::{ToC32, ToColor, *};
