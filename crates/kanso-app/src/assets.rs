//! The icons the app draws. GPUI Kit's default bundle covers the components;
//! the app's own views use a few more from the Lucide catalog, embedded here
//! so the binary carries only what it shows.

use std::borrow::Cow;

use gpui_kit::assets::{Assets, icon_assets};
use gpui_kit::{AssetSource, Result, SharedString};

icon_assets!(
    pub ExtraIcons,
    [
        Archive,
        ArrowUpDown,
        Brackets,
        Briefcase,
        CalendarClock,
        CalendarDays,
        Camera,
        Car,
        Circle,
        CircleArrowDown,
        CircleDot,
        CirclePause,
        ClipboardList,
        ClipboardPaste,
        Clock,
        Cloud,
        CloudCheck,
        CloudOff,
        CloudUpload,
        Code,
        CodeXml,
        Coffee,
        Columns2,
        Command,
        Copy,
        CornerDownLeft,
        DatabaseBackup,
        Download,
        Dumbbell,
        FileKey,
        FilePlus,
        Film,
        FlaskConical,
        Focus,
        Gamepad2,
        Gift,
        Goal,
        GraduationCap,
        Hash,
        Heading,
        Heading1,
        Heading2,
        Heading3,
        House,
        Image,
        Keyboard,
        KeyRound,
        Laptop,
        LayoutTemplate,
        Leaf,
        Lightbulb,
        Link,
        List,
        ListChecks,
        ListFilter,
        ListOrdered,
        ListTodo,
        Lock,
        LockKeyhole,
        LogIn,
        LogOut,
        MonitorSmartphone,
        Music,
        Notebook,
        NotebookPen,
        NotebookText,
        PanelRight,
        Pencil,
        PenTool,
        Pin,
        PinOff,
        Plane,
        Plug,
        PlugZap,
        Power,
        RefreshCcw,
        Regex,
        Rocket,
        Ruler,
        Save,
        Scan,
        ScanText,
        ScrollText,
        Server,
        Shield,
        ShoppingCart,
        Sparkles,
        Split,
        SquareCheck,
        SquarePen,
        SunMoon,
        Table,
        TableOfContents,
        Tag,
        Target,
        TextQuote,
        Trash,
        Truck,
        Type,
        Unplug,
        Upload,
        UserPlus,
        Users,
        Utensils,
        Wallet,
        Waypoints,
        WholeWord,
        Zap,
        ZoomIn,
        ZoomOut,
    ]
);

/// The default component icons plus [`ExtraIcons`].
#[derive(Clone, Copy, Debug, Default)]
pub struct KansoAssets;

impl AssetSource for KansoAssets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        // The extra set answers `None` for paths it lacks; the default bundle
        // reports them as errors, so it goes last.
        if let Some(bytes) = ExtraIcons.load(path)? {
            return Ok(Some(bytes));
        }
        Assets.load(path)
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        let mut paths = Assets.list(path)?;
        paths.extend(ExtraIcons.list(path)?);
        paths.sort();
        paths.dedup();
        Ok(paths)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_kit::assets::IconName;
    use gpui_kit::component::IconNamed as _;

    /// Every extra icon and every default component icon loads as an SVG.
    #[test]
    fn extra_and_default_icons_load() {
        let extra = ExtraIcons.list("icons/").unwrap();
        assert!(extra.len() >= 30, "{extra:?}");
        for path in extra.iter().chain([
            &IconName::Search.path(),
            &gpui_kit::component::IconName::Inbox.path(),
        ]) {
            let bytes = KansoAssets.load(path).unwrap().expect("icon bytes");
            assert!(
                std::str::from_utf8(&bytes).unwrap().contains("<svg"),
                "{path}"
            );
        }
        assert!(KansoAssets.list("icons/").unwrap().len() > extra.len());
        // A path neither set has stays an error, as with the default bundle.
        assert!(KansoAssets.load("icons/not-an-icon.svg").is_err());
    }
}
