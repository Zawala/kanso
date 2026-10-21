//! The icons the app draws. GPUI Kit's default bundle covers the components;
//! the app's own views use a few more from the Lucide catalog, embedded here
//! so the binary carries only what it shows.

use std::borrow::Cow;

use gpui_kit::assets::{Assets, icon_assets};
use gpui_kit::{AssetSource, Result, SharedString};

icon_assets!(
    pub ExtraIcons,
    [
        AlarmClock,
        Apple,
        Archive,
        ArrowUpDown,
        Atom,
        Baby,
        Banknote,
        Bed,
        Bike,
        Bookmark,
        BookMarked,
        Brackets,
        Brain,
        Briefcase,
        Brush,
        Bug,
        Bus,
        Cake,
        Calculator,
        CalendarClock,
        CalendarDays,
        Camera,
        Car,
        Cat,
        ChartLine,
        ChefHat,
        Church,
        Circle,
        CircleArrowDown,
        CircleDot,
        CirclePause,
        Clapperboard,
        ClipboardList,
        ClipboardPaste,
        Clock,
        Cloud,
        CloudCheck,
        CloudOff,
        CloudRain,
        CloudUpload,
        Code,
        CodeXml,
        Coffee,
        Coins,
        Columns2,
        Command,
        Compass,
        Contact,
        Copy,
        CornerDownLeft,
        CreditCard,
        Database,
        DatabaseBackup,
        Dices,
        Dog,
        Download,
        Droplet,
        Dumbbell,
        Feather,
        FileKey,
        FilePlus,
        Files,
        Film,
        Flag,
        Flame,
        FlaskConical,
        Flower,
        Focus,
        FolderHeart,
        FolderLock,
        Gamepad2,
        Gift,
        GitBranch,
        Goal,
        GraduationCap,
        Guitar,
        Hammer,
        HandHeart,
        Handshake,
        Hash,
        Heading,
        Heading1,
        Heading2,
        Heading3,
        Headphones,
        HeartPulse,
        Hourglass,
        House,
        Image,
        Infinity,
        Key,
        Keyboard,
        KeyRound,
        Landmark,
        Languages,
        Laptop,
        LayoutTemplate,
        Leaf,
        Library,
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
        Mail,
        MapPin,
        Megaphone,
        MessageCircle,
        Microscope,
        MonitorSmartphone,
        Mountain,
        Music,
        Newspaper,
        Notebook,
        NotebookPen,
        NotebookText,
        PanelRight,
        PartyPopper,
        PawPrint,
        Pencil,
        PenTool,
        Phone,
        PiggyBank,
        Pill,
        Pin,
        PinOff,
        Plane,
        Plug,
        PlugZap,
        Power,
        Presentation,
        Puzzle,
        Receipt,
        RefreshCcw,
        Regex,
        Rocket,
        Rss,
        Ruler,
        Save,
        Scan,
        ScanText,
        School,
        ScrollText,
        Server,
        Shield,
        Ship,
        ShoppingBag,
        ShoppingCart,
        Snowflake,
        Sofa,
        Sparkles,
        Split,
        SquareCheck,
        SquarePen,
        Stethoscope,
        StickyNote,
        Store,
        SunMoon,
        Table,
        TableOfContents,
        Tag,
        Target,
        Tent,
        Terminal,
        TextQuote,
        TrainFront,
        Trash,
        Trees,
        Trophy,
        Truck,
        Type,
        Unplug,
        Upload,
        UserPlus,
        Users,
        Utensils,
        Video,
        Wallet,
        Waypoints,
        WholeWord,
        Wrench,
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
