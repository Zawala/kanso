// ignore: unused_import
import 'package:intl/intl.dart' as intl;
import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for English (`en`).
class AppLocalizationsEn extends AppLocalizations {
  AppLocalizationsEn([String locale = 'en']) : super(locale);

  @override
  String get appName => 'Kanso';

  @override
  String get libraryAllNotes => 'All Notes';

  @override
  String get libraryTemplates => 'Templates';

  @override
  String get libraryPinned => 'Pinned';

  @override
  String get libraryRecent => 'Recent';

  @override
  String get libraryTasks => 'Tasks';

  @override
  String get libraryNotebooks => 'Notebooks';

  @override
  String get libraryTags => 'Tags';

  @override
  String get libraryTrash => 'Trash';

  @override
  String get librarySortNotebooks => 'Sort notebooks';

  @override
  String get libraryNewNotebook => 'New notebook';

  @override
  String get openSettings => 'Settings';

  @override
  String get openLibrary => 'Library';

  @override
  String get searchHint => 'Search notes';

  @override
  String get searchClear => 'Clear search';

  @override
  String get searchCancel => 'Cancel';

  @override
  String get searchFilterAll => 'All';

  @override
  String get searchFilterTitles => 'Titles';

  @override
  String get searchFilterTags => 'Tags';

  @override
  String get searchFilterNotebook => 'This notebook';

  @override
  String searchResultCount(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count results',
      one: '1 result',
    );
    return '$_temp0';
  }

  @override
  String noteListCount(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count notes',
      one: '1 note',
    );
    return '$_temp0';
  }

  @override
  String get noteListNewNote => 'New note';

  @override
  String get noteListMore => 'More';

  @override
  String get noteBack => 'Back';

  @override
  String get noteForward => 'Forward';

  @override
  String get noteSearch => 'Search';

  @override
  String get notePreview => 'Preview';

  @override
  String get noteEdit => 'Edit';

  @override
  String get noteInfo => 'Note info';

  @override
  String get noteMetaNotebook => 'Notebook';

  @override
  String get noteMetaStatus => 'Status';

  @override
  String get noteMetaCreated => 'Created';

  @override
  String get noteMetaUpdated => 'Updated';

  @override
  String get statusActive => 'Active';

  @override
  String get statusOnHold => 'On hold';

  @override
  String get statusCompleted => 'Completed';

  @override
  String get statusDropped => 'Dropped';

  @override
  String editProperties(int count) {
    return 'Properties · $count';
  }

  @override
  String get accessoryHeading => 'Heading';

  @override
  String get accessoryList => 'List';

  @override
  String get accessoryTask => 'Task';

  @override
  String get accessoryBold => 'Bold';

  @override
  String get accessoryLink => 'Link';

  @override
  String get accessoryImage => 'Image';

  @override
  String get accessoryWikiLink => 'Link to a note';

  @override
  String get accessoryIndent => 'Indent';

  @override
  String get accessoryOutdent => 'Outdent';

  @override
  String get accessoryUndo => 'Undo';

  @override
  String get settingsTitle => 'Settings';

  @override
  String get settingsClose => 'Close';

  @override
  String get settingsGeneral => 'General';

  @override
  String get settingsDefaultNotebook => 'Default notebook';

  @override
  String get settingsAppearance => 'Appearance';

  @override
  String get settingsTheme => 'Theme';

  @override
  String get settingsIcons => 'Icons';

  @override
  String get settingsTextSize => 'Text size';

  @override
  String get settingsSync => 'Sync';

  @override
  String get settingsSyncAccount => 'Account';

  @override
  String get settingsSyncedJustNow => 'Synced · just now';

  @override
  String get settingsWakeDevice =>
      'Wake this device when notes change elsewhere';

  @override
  String get settingsLockedNotes => 'Locked notes';

  @override
  String get settingsUnlockFaceId => 'Unlock with Face ID';

  @override
  String get settingsLockAfter => 'Lock after';

  @override
  String get settingsTenMinutes => '10 minutes';

  @override
  String get settingsAccount => 'Account';

  @override
  String get settingsSignedInAs => 'Signed in as';

  @override
  String get settingsSignOut => 'Sign out';

  @override
  String get settingsAbout => 'About';

  @override
  String get settingsVersion => 'Version';

  @override
  String get settingsWebsite => 'Website';

  @override
  String get settingsReleaseNotes => 'Release notes';

  @override
  String get settingsHelp => 'Help';

  @override
  String get settingsLicences => 'Licences';

  @override
  String get themesTitle => 'Themes';

  @override
  String get themesInstalled => 'Installed';

  @override
  String get themesGallery => 'Gallery';

  @override
  String get themesSearchHint => 'Search themes';

  @override
  String get themesFilterAll => 'All';

  @override
  String get themesFilterLight => 'Light';

  @override
  String get themesFilterDark => 'Dark';

  @override
  String get themesFilterInstalled => 'Installed';

  @override
  String get themesFilterUpdates => 'Updates';

  @override
  String get themesBadgeBuiltIn => 'Built in';

  @override
  String get themesBadgeInstalled => 'Installed';

  @override
  String get themesInstall => 'Install';

  @override
  String get themesCurrent => 'In use';

  @override
  String get themesAppearanceLight => 'Light';

  @override
  String get themesAppearanceDark => 'Dark';

  @override
  String get lockedTitle => 'This note is locked';

  @override
  String lockedMessage(int minutes) {
    return 'Unlock it to read and edit. It locks again after $minutes minutes or when you leave Kanso.';
  }

  @override
  String get lockedUnlock => 'Unlock with Face ID';

  @override
  String get lockedUsePassword => 'Use password';

  @override
  String get errorSignedOutTitle => 'Signed out on this device';

  @override
  String get errorSignedOutMessage =>
      'Your session on this phone ended. Sign in again to keep syncing; your notes stay here.';

  @override
  String get errorSignInAgain => 'Sign in again';

  @override
  String get errorNotNow => 'Not now';

  @override
  String get designGalleryTitle => 'Design proposal';

  @override
  String get designGalleryIntro =>
      'Static screens for review. Nothing here saves or syncs.';

  @override
  String get designGalleryAppearance => 'Appearance';

  @override
  String get designGalleryPhone => 'Phone';

  @override
  String get designGalleryTablet => 'Tablet';

  @override
  String get appearanceSystem => 'System';

  @override
  String get appearanceLight => 'Light';

  @override
  String get appearanceDark => 'Dark';

  @override
  String get screenLibrary => 'Library';

  @override
  String get screenLibraryScrolled => 'Library, scrolled';

  @override
  String get screenNoteList => 'Note list';

  @override
  String get screenNotePreview => 'Note, preview';

  @override
  String get screenNoteEdit => 'Note, edit';

  @override
  String get screenSearch => 'Search';

  @override
  String get screenSettings => 'Settings';

  @override
  String get screenThemePicker => 'Theme picker';

  @override
  String get screenLockedNote => 'Locked note';

  @override
  String get screenError => 'Error card';

  @override
  String get themesSampleHeading => 'Heading';

  @override
  String get themesSampleLine => 'Some **bold** and *italic*';

  @override
  String get themesSampleCode => '`code`, [a link](url)';

  @override
  String get themesSampleQuote => 'a quote';

  @override
  String get screenSettingsScrolled => 'Settings, scrolled';

  @override
  String get screenThemePickerScrolled => 'Theme picker, scrolled';

  @override
  String get settingsFonts => 'Fonts';

  @override
  String get settingsBack => 'Back';

  @override
  String get fontsTitle => 'Fonts';

  @override
  String get fontsEditorSection => 'Editor font';

  @override
  String get fontsEditorFooter => 'The font you write in.';

  @override
  String get fontsPreviewSection => 'Preview font';

  @override
  String get fontsPreviewFooter => 'The font a note is read in.';

  @override
  String get fontsCodeSection => 'Code font';

  @override
  String get fontsCodeFooter => 'Used for code in notes and in theme previews.';

  @override
  String get fontsSystemName => 'System — your device’s font';

  @override
  String get fontsSystemShort => 'System';

  @override
  String get fontsSystemSample => 'San Francisco on iOS, Roboto on Android';

  @override
  String get fontsSampleText => 'Calm notes, clearly written';

  @override
  String get fontsSampleCode => 'let note = merge(a, b);';

  @override
  String get fontsTextSizeSection => 'Text size';

  @override
  String get fontsTextSizeFooter =>
      'Changes the text of notes only. The rest of Kanso follows your device’s text size.';

  @override
  String get fontsTextSizeSample =>
      'Calm software says less and remembers more. Every device keeps a full copy of your notes, and a change is merged on the others.';

  @override
  String get textSizeSmall => 'Small';

  @override
  String get textSizeDefault => 'Default';

  @override
  String get textSizeLarge => 'Large';

  @override
  String get textSizeLarger => 'Larger';

  @override
  String get fontsAdd => 'Add a font…';

  @override
  String get fontsAddFooter => 'Choose a .ttf or .otf file from Files.';

  @override
  String get noteModeLabel => 'View mode';

  @override
  String get noteListLoading => 'Loading notes';

  @override
  String emptyNotebookTitle(String notebook) {
    return 'No notes in $notebook yet';
  }

  @override
  String get emptyNotebookMessage =>
      'Notes added here will appear in this list.';

  @override
  String get noNoteSelectedTitle => 'No note selected';

  @override
  String get noNoteSelectedMessage =>
      'Choose a note from the list to read it here.';

  @override
  String get screenFonts => 'Fonts';

  @override
  String get screenFontsScrolled => 'Fonts, scrolled';

  @override
  String get screenLoadingList => 'Note list, loading';

  @override
  String get screenEmptyNotebook => 'Empty notebook';

  @override
  String get designGalleryCoreTitle => 'Library on this phone';

  @override
  String get designGalleryCoreCheck => 'Rust core';

  @override
  String get coreCheckTitle => 'Rust core';

  @override
  String get coreCheckLoading => 'Opening the library…';

  @override
  String get coreCheckLibrary => 'Library';

  @override
  String get coreCheckVersion => 'Version';

  @override
  String get coreCheckCertificates => 'Certificates';

  @override
  String get coreCheckCertificatesReady => 'Ready';

  @override
  String get coreCheckCertificatesNotReady => 'Not ready';

  @override
  String get coreCheckNotes => 'Notes';

  @override
  String get coreCheckLastChange => 'Last change';

  @override
  String get coreCheckNoChange => 'None yet';

  @override
  String coreCheckChanged(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count notes changed',
      one: '1 note changed',
    );
    return '$_temp0';
  }

  @override
  String get coreCheckNewNote => 'New note';

  @override
  String get coreCheckLatest => 'Latest notes';

  @override
  String get coreCheckActionFailed => 'That didn\'t work. Try again.';

  @override
  String get coreCheckFailedTitle => 'The library didn\'t open';

  @override
  String get coreCheckFailedMessage =>
      'Something went wrong reading the notes on this phone. Try again.';

  @override
  String get coreCheckRetry => 'Try again';

  @override
  String get searchTitle => 'Search';

  @override
  String get libraryDesignGallery => 'Design gallery';

  @override
  String get retry => 'Try again';

  @override
  String get libraryLoading => 'Loading the library';

  @override
  String get libraryFailedTitle => 'The library didn\'t load';

  @override
  String get libraryFailedMessage =>
      'Something went wrong reading the notebooks and tags on this phone. Try again.';

  @override
  String get libraryNoNotebooks => 'No notebooks yet';

  @override
  String get libraryNoTags => 'No tags yet';

  @override
  String get dayToday => 'Today';

  @override
  String get dayYesterday => 'Yesterday';

  @override
  String get noteUntitled => 'Untitled';

  @override
  String get noteListLocked => 'Locked';

  @override
  String get noteListFailedTitle => 'These notes didn\'t load';

  @override
  String get noteListFailedMessage =>
      'Something went wrong reading the notes on this phone. Try again.';

  @override
  String get emptyAllTitle => 'No notes yet';

  @override
  String get emptyAllMessage => 'Notes will appear here.';

  @override
  String get emptyPinnedTitle => 'Nothing pinned';

  @override
  String get emptyPinnedMessage =>
      'Pin a note by swiping it, and it will wait here.';

  @override
  String get emptyRecentTitle => 'Nothing recent';

  @override
  String get emptyRecentMessage =>
      'Notes you have changed lately will appear here.';

  @override
  String get emptyTrashTitle => 'The trash is empty';

  @override
  String get emptyTrashMessage =>
      'Notes you trash wait here until you move them out.';

  @override
  String emptyTagTitle(String tag) {
    return 'No notes tagged $tag';
  }

  @override
  String get emptyTagMessage => 'Notes with this tag will appear in this list.';

  @override
  String get emptyNarrowedTitle => 'No notes match';

  @override
  String get emptyNarrowedMessage =>
      'Try another status, or turn off Pinned Only.';

  @override
  String get listOptionsTitle => 'View options';

  @override
  String get listOptionsSortBy => 'Sort by';

  @override
  String get listOptionsDateUpdated => 'Date Updated';

  @override
  String get listOptionsDateCreated => 'Date Created';

  @override
  String get listOptionsTitleSort => 'Title';

  @override
  String get listOptionsNewestFirst => 'Newest First';

  @override
  String get listOptionsOldestFirst => 'Oldest First';

  @override
  String get listOptionsAToZ => 'A to Z';

  @override
  String get listOptionsZToA => 'Z to A';

  @override
  String get listOptionsStatus => 'Status';

  @override
  String get listOptionsAnyStatus => 'Any Status';

  @override
  String get listOptionsPinnedOnly => 'Pinned Only';

  @override
  String get noteListDone => 'Done';

  @override
  String noteListSelected(int count) {
    return '$count selected';
  }

  @override
  String get noteListPin => 'Pin';

  @override
  String get noteListUnpin => 'Unpin';

  @override
  String get noteListMove => 'Move';

  @override
  String get noteListTrash => 'Trash';

  @override
  String moveSheetTitle(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: 'Move $count notes to',
      one: 'Move 1 note to',
    );
    return '$_temp0';
  }

  @override
  String get noteListCancel => 'Cancel';

  @override
  String get noteListChangeFailed => 'That didn\'t work. Try again.';

  @override
  String get noteListChangeLocked =>
      'Locked notebooks can\'t take or give up notes here yet.';

  @override
  String get noteListDismiss => 'OK';

  @override
  String get noteLoading => 'Loading the note';

  @override
  String get noteNotFoundTitle => 'This note isn\'t here';

  @override
  String get noteNotFoundMessage =>
      'It may have been deleted, or it hasn\'t reached this phone yet.';

  @override
  String get noteFailedTitle => 'This note didn\'t load';

  @override
  String get noteFailedMessage =>
      'Something went wrong reading the note on this phone. Try again.';

  @override
  String get lockedMessageSealed =>
      'Its text stays sealed. Only its title can be shown here.';
}
