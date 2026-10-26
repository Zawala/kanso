import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_localizations/flutter_localizations.dart';
import 'package:intl/intl.dart' as intl;

import 'app_localizations_en.dart';

// ignore_for_file: type=lint

/// Callers can lookup localized strings with an instance of AppLocalizations
/// returned by `AppLocalizations.of(context)`.
///
/// Applications need to include `AppLocalizations.delegate()` in their app's
/// `localizationDelegates` list, and the locales they support in the app's
/// `supportedLocales` list. For example:
///
/// ```dart
/// import 'l10n/app_localizations.dart';
///
/// return MaterialApp(
///   localizationsDelegates: AppLocalizations.localizationsDelegates,
///   supportedLocales: AppLocalizations.supportedLocales,
///   home: MyApplicationHome(),
/// );
/// ```
///
/// ## Update pubspec.yaml
///
/// Please make sure to update your pubspec.yaml to include the following
/// packages:
///
/// ```yaml
/// dependencies:
///   # Internationalization support.
///   flutter_localizations:
///     sdk: flutter
///   intl: any # Use the pinned version from flutter_localizations
///
///   # Rest of dependencies
/// ```
///
/// ## iOS Applications
///
/// iOS applications define key application metadata, including supported
/// locales, in an Info.plist file that is built into the application bundle.
/// To configure the locales supported by your app, you’ll need to edit this
/// file.
///
/// First, open your project’s ios/Runner.xcworkspace Xcode workspace file.
/// Then, in the Project Navigator, open the Info.plist file under the Runner
/// project’s Runner folder.
///
/// Next, select the Information Property List item, select Add Item from the
/// Editor menu, then select Localizations from the pop-up menu.
///
/// Select and expand the newly-created Localizations item then, for each
/// locale your application supports, add a new item and select the locale
/// you wish to add from the pop-up menu in the Value field. This list should
/// be consistent with the languages listed in the AppLocalizations.supportedLocales
/// property.
abstract class AppLocalizations {
  AppLocalizations(String locale)
    : localeName = intl.Intl.canonicalizedLocale(locale.toString());

  final String localeName;

  static AppLocalizations of(BuildContext context) {
    return Localizations.of<AppLocalizations>(context, AppLocalizations)!;
  }

  static const LocalizationsDelegate<AppLocalizations> delegate =
      _AppLocalizationsDelegate();

  /// A list of this localizations delegate along with the default localizations
  /// delegates.
  ///
  /// Returns a list of localizations delegates containing this delegate along with
  /// GlobalMaterialLocalizations.delegate, GlobalCupertinoLocalizations.delegate,
  /// and GlobalWidgetsLocalizations.delegate.
  ///
  /// Additional delegates can be added by appending to this list in
  /// MaterialApp. This list does not have to be used at all if a custom list
  /// of delegates is preferred or required.
  static const List<LocalizationsDelegate<dynamic>> localizationsDelegates =
      <LocalizationsDelegate<dynamic>>[
        delegate,
        GlobalMaterialLocalizations.delegate,
        GlobalCupertinoLocalizations.delegate,
        GlobalWidgetsLocalizations.delegate,
      ];

  /// A list of this localizations delegate's supported locales.
  static const List<Locale> supportedLocales = <Locale>[Locale('en')];

  /// The app's name, shown as its title.
  ///
  /// In en, this message translates to:
  /// **'Kanso'**
  String get appName;

  /// The library row that lists every note.
  ///
  /// In en, this message translates to:
  /// **'All Notes'**
  String get libraryAllNotes;

  /// The library row for note templates.
  ///
  /// In en, this message translates to:
  /// **'Templates'**
  String get libraryTemplates;

  /// The library row for pinned notes, and the list's group of them.
  ///
  /// In en, this message translates to:
  /// **'Pinned'**
  String get libraryPinned;

  /// The library row for recently opened notes.
  ///
  /// In en, this message translates to:
  /// **'Recent'**
  String get libraryRecent;

  /// The library row for open tasks across notes.
  ///
  /// In en, this message translates to:
  /// **'Tasks'**
  String get libraryTasks;

  /// The label above the list of notebooks.
  ///
  /// In en, this message translates to:
  /// **'Notebooks'**
  String get libraryNotebooks;

  /// The label above the list of tags.
  ///
  /// In en, this message translates to:
  /// **'Tags'**
  String get libraryTags;

  /// The library row for deleted notes.
  ///
  /// In en, this message translates to:
  /// **'Trash'**
  String get libraryTrash;

  /// What the sort button of the notebooks header does.
  ///
  /// In en, this message translates to:
  /// **'Sort notebooks'**
  String get librarySortNotebooks;

  /// What the add button of the notebooks header does.
  ///
  /// In en, this message translates to:
  /// **'New notebook'**
  String get libraryNewNotebook;

  /// What the settings button does.
  ///
  /// In en, this message translates to:
  /// **'Settings'**
  String get openSettings;

  /// What the button that opens the library does.
  ///
  /// In en, this message translates to:
  /// **'Library'**
  String get openLibrary;

  /// The hint in an empty search field.
  ///
  /// In en, this message translates to:
  /// **'Search notes'**
  String get searchHint;

  /// What the clear button of a search field does.
  ///
  /// In en, this message translates to:
  /// **'Clear search'**
  String get searchClear;

  /// Leaves the search.
  ///
  /// In en, this message translates to:
  /// **'Cancel'**
  String get searchCancel;

  /// A search filter: everything.
  ///
  /// In en, this message translates to:
  /// **'All'**
  String get searchFilterAll;

  /// A search filter: only note titles.
  ///
  /// In en, this message translates to:
  /// **'Titles'**
  String get searchFilterTitles;

  /// A search filter: only tags.
  ///
  /// In en, this message translates to:
  /// **'Tags'**
  String get searchFilterTags;

  /// A search filter: the open notebook only.
  ///
  /// In en, this message translates to:
  /// **'This notebook'**
  String get searchFilterNotebook;

  /// How many notes match a search.
  ///
  /// In en, this message translates to:
  /// **'{count, plural, =1{1 result} other{{count} results}}'**
  String searchResultCount(int count);

  /// How many notes a list holds.
  ///
  /// In en, this message translates to:
  /// **'{count, plural, =1{1 note} other{{count} notes}}'**
  String noteListCount(int count);

  /// What the add button of the note list does.
  ///
  /// In en, this message translates to:
  /// **'New note'**
  String get noteListNewNote;

  /// What the more button of the note list does.
  ///
  /// In en, this message translates to:
  /// **'More'**
  String get noteListMore;

  /// Goes back.
  ///
  /// In en, this message translates to:
  /// **'Back'**
  String get noteBack;

  /// Goes forward.
  ///
  /// In en, this message translates to:
  /// **'Forward'**
  String get noteForward;

  /// Opens search.
  ///
  /// In en, this message translates to:
  /// **'Search'**
  String get noteSearch;

  /// Shows the note rendered.
  ///
  /// In en, this message translates to:
  /// **'Preview'**
  String get notePreview;

  /// Shows the note as text to edit.
  ///
  /// In en, this message translates to:
  /// **'Edit'**
  String get noteEdit;

  /// Shows the note's details.
  ///
  /// In en, this message translates to:
  /// **'Note info'**
  String get noteInfo;

  /// The label of a note's notebook.
  ///
  /// In en, this message translates to:
  /// **'Notebook'**
  String get noteMetaNotebook;

  /// The label of a note's status.
  ///
  /// In en, this message translates to:
  /// **'Status'**
  String get noteMetaStatus;

  /// The label of when a note was made.
  ///
  /// In en, this message translates to:
  /// **'Created'**
  String get noteMetaCreated;

  /// The label of when a note last changed.
  ///
  /// In en, this message translates to:
  /// **'Updated'**
  String get noteMetaUpdated;

  /// A note's status: in progress.
  ///
  /// In en, this message translates to:
  /// **'Active'**
  String get statusActive;

  /// A note's status: paused.
  ///
  /// In en, this message translates to:
  /// **'On hold'**
  String get statusOnHold;

  /// A note's status: done.
  ///
  /// In en, this message translates to:
  /// **'Completed'**
  String get statusCompleted;

  /// A note's status: given up.
  ///
  /// In en, this message translates to:
  /// **'Dropped'**
  String get statusDropped;

  /// The collapsed row of a note's properties, with how many it has.
  ///
  /// In en, this message translates to:
  /// **'Properties · {count}'**
  String editProperties(int count);

  /// A keyboard shortcut that starts a heading.
  ///
  /// In en, this message translates to:
  /// **'Heading'**
  String get accessoryHeading;

  /// A keyboard shortcut that starts a list.
  ///
  /// In en, this message translates to:
  /// **'List'**
  String get accessoryList;

  /// A keyboard shortcut that starts a task.
  ///
  /// In en, this message translates to:
  /// **'Task'**
  String get accessoryTask;

  /// A keyboard shortcut that makes text bold.
  ///
  /// In en, this message translates to:
  /// **'Bold'**
  String get accessoryBold;

  /// A keyboard shortcut that makes a link.
  ///
  /// In en, this message translates to:
  /// **'Link'**
  String get accessoryLink;

  /// A keyboard shortcut that adds an image.
  ///
  /// In en, this message translates to:
  /// **'Image'**
  String get accessoryImage;

  /// A keyboard shortcut that starts a link to another note.
  ///
  /// In en, this message translates to:
  /// **'Link to a note'**
  String get accessoryWikiLink;

  /// A keyboard shortcut that indents a line.
  ///
  /// In en, this message translates to:
  /// **'Indent'**
  String get accessoryIndent;

  /// A keyboard shortcut that outdents a line.
  ///
  /// In en, this message translates to:
  /// **'Outdent'**
  String get accessoryOutdent;

  /// A keyboard shortcut that undoes the last change.
  ///
  /// In en, this message translates to:
  /// **'Undo'**
  String get accessoryUndo;

  /// The title of the settings page.
  ///
  /// In en, this message translates to:
  /// **'Settings'**
  String get settingsTitle;

  /// Closes a page.
  ///
  /// In en, this message translates to:
  /// **'Close'**
  String get settingsClose;

  /// A section of settings.
  ///
  /// In en, this message translates to:
  /// **'General'**
  String get settingsGeneral;

  /// The notebook new notes go to.
  ///
  /// In en, this message translates to:
  /// **'Default notebook'**
  String get settingsDefaultNotebook;

  /// A section of settings.
  ///
  /// In en, this message translates to:
  /// **'Appearance'**
  String get settingsAppearance;

  /// The colour theme.
  ///
  /// In en, this message translates to:
  /// **'Theme'**
  String get settingsTheme;

  /// The icon set.
  ///
  /// In en, this message translates to:
  /// **'Icons'**
  String get settingsIcons;

  /// The size of text in notes.
  ///
  /// In en, this message translates to:
  /// **'Text size'**
  String get settingsTextSize;

  /// A section of settings.
  ///
  /// In en, this message translates to:
  /// **'Sync'**
  String get settingsSync;

  /// The row that shows the sync account.
  ///
  /// In en, this message translates to:
  /// **'Account'**
  String get settingsSyncAccount;

  /// The sync state, a moment after it finished.
  ///
  /// In en, this message translates to:
  /// **'Synced · just now'**
  String get settingsSyncedJustNow;

  /// A switch for push notifications that trigger a sync.
  ///
  /// In en, this message translates to:
  /// **'Wake this device when notes change elsewhere'**
  String get settingsWakeDevice;

  /// A section of settings.
  ///
  /// In en, this message translates to:
  /// **'Locked notes'**
  String get settingsLockedNotes;

  /// A switch to unlock notes with Face ID.
  ///
  /// In en, this message translates to:
  /// **'Unlock with Face ID'**
  String get settingsUnlockFaceId;

  /// How long until a note locks again.
  ///
  /// In en, this message translates to:
  /// **'Lock after'**
  String get settingsLockAfter;

  /// A length of time.
  ///
  /// In en, this message translates to:
  /// **'10 minutes'**
  String get settingsTenMinutes;

  /// A section of settings.
  ///
  /// In en, this message translates to:
  /// **'Account'**
  String get settingsAccount;

  /// Who is signed in.
  ///
  /// In en, this message translates to:
  /// **'Signed in as'**
  String get settingsSignedInAs;

  /// Signs out of the account.
  ///
  /// In en, this message translates to:
  /// **'Sign out'**
  String get settingsSignOut;

  /// A section of settings.
  ///
  /// In en, this message translates to:
  /// **'About'**
  String get settingsAbout;

  /// The app's version.
  ///
  /// In en, this message translates to:
  /// **'Version'**
  String get settingsVersion;

  /// Opens Kanso's website.
  ///
  /// In en, this message translates to:
  /// **'Website'**
  String get settingsWebsite;

  /// Opens the release notes.
  ///
  /// In en, this message translates to:
  /// **'Release notes'**
  String get settingsReleaseNotes;

  /// Opens the help.
  ///
  /// In en, this message translates to:
  /// **'Help'**
  String get settingsHelp;

  /// Opens the licences of what Kanso is made of.
  ///
  /// In en, this message translates to:
  /// **'Licences'**
  String get settingsLicences;

  /// The title of the theme page.
  ///
  /// In en, this message translates to:
  /// **'Themes'**
  String get themesTitle;

  /// The label above the installed themes.
  ///
  /// In en, this message translates to:
  /// **'Installed'**
  String get themesInstalled;

  /// The label above the themes that can be installed.
  ///
  /// In en, this message translates to:
  /// **'Gallery'**
  String get themesGallery;

  /// The hint in the theme search field.
  ///
  /// In en, this message translates to:
  /// **'Search themes'**
  String get themesSearchHint;

  /// A theme filter: every theme.
  ///
  /// In en, this message translates to:
  /// **'All'**
  String get themesFilterAll;

  /// A theme filter: light themes.
  ///
  /// In en, this message translates to:
  /// **'Light'**
  String get themesFilterLight;

  /// A theme filter: dark themes.
  ///
  /// In en, this message translates to:
  /// **'Dark'**
  String get themesFilterDark;

  /// A theme filter: installed themes.
  ///
  /// In en, this message translates to:
  /// **'Installed'**
  String get themesFilterInstalled;

  /// A theme filter: themes with an update.
  ///
  /// In en, this message translates to:
  /// **'Updates'**
  String get themesFilterUpdates;

  /// A badge on a theme that comes with Kanso.
  ///
  /// In en, this message translates to:
  /// **'Built in'**
  String get themesBadgeBuiltIn;

  /// A badge on a theme that is installed.
  ///
  /// In en, this message translates to:
  /// **'Installed'**
  String get themesBadgeInstalled;

  /// Installs a theme.
  ///
  /// In en, this message translates to:
  /// **'Install'**
  String get themesInstall;

  /// Marks the theme in use.
  ///
  /// In en, this message translates to:
  /// **'In use'**
  String get themesCurrent;

  /// A theme made for a light interface.
  ///
  /// In en, this message translates to:
  /// **'Light'**
  String get themesAppearanceLight;

  /// A theme made for a dark interface.
  ///
  /// In en, this message translates to:
  /// **'Dark'**
  String get themesAppearanceDark;

  /// The title of the card shown in place of a locked note.
  ///
  /// In en, this message translates to:
  /// **'This note is locked'**
  String get lockedTitle;

  /// Says how a locked note is opened and when it locks.
  ///
  /// In en, this message translates to:
  /// **'Unlock it to read and edit. It locks again after {minutes} minutes or when you leave Kanso.'**
  String lockedMessage(int minutes);

  /// Unlocks a note with Face ID.
  ///
  /// In en, this message translates to:
  /// **'Unlock with Face ID'**
  String get lockedUnlock;

  /// Unlocks a note with a password instead.
  ///
  /// In en, this message translates to:
  /// **'Use password'**
  String get lockedUsePassword;

  /// The title of the card shown when the session ended.
  ///
  /// In en, this message translates to:
  /// **'Signed out on this device'**
  String get errorSignedOutTitle;

  /// Says what happened and that no notes are lost.
  ///
  /// In en, this message translates to:
  /// **'Your session on this phone ended. Sign in again to keep syncing; your notes stay here.'**
  String get errorSignedOutMessage;

  /// Starts signing in again.
  ///
  /// In en, this message translates to:
  /// **'Sign in again'**
  String get errorSignInAgain;

  /// Dismisses the card.
  ///
  /// In en, this message translates to:
  /// **'Not now'**
  String get errorNotNow;

  /// The title of the page that lists the mock screens.
  ///
  /// In en, this message translates to:
  /// **'Design proposal'**
  String get designGalleryTitle;

  /// Explains the page that lists the mock screens.
  ///
  /// In en, this message translates to:
  /// **'Static screens for review. Nothing here saves or syncs.'**
  String get designGalleryIntro;

  /// The label of the control that switches between light and dark.
  ///
  /// In en, this message translates to:
  /// **'Appearance'**
  String get designGalleryAppearance;

  /// The group of mock screens drawn for a phone.
  ///
  /// In en, this message translates to:
  /// **'Phone'**
  String get designGalleryPhone;

  /// The group of mock screens drawn for a tablet.
  ///
  /// In en, this message translates to:
  /// **'Tablet'**
  String get designGalleryTablet;

  /// Follow the device's light or dark setting.
  ///
  /// In en, this message translates to:
  /// **'System'**
  String get appearanceSystem;

  /// Always light.
  ///
  /// In en, this message translates to:
  /// **'Light'**
  String get appearanceLight;

  /// Always dark.
  ///
  /// In en, this message translates to:
  /// **'Dark'**
  String get appearanceDark;

  /// The name of a mock screen.
  ///
  /// In en, this message translates to:
  /// **'Library'**
  String get screenLibrary;

  /// The name of a mock screen.
  ///
  /// In en, this message translates to:
  /// **'Library, scrolled'**
  String get screenLibraryScrolled;

  /// The name of a mock screen.
  ///
  /// In en, this message translates to:
  /// **'Note list'**
  String get screenNoteList;

  /// The name of a mock screen.
  ///
  /// In en, this message translates to:
  /// **'Note, preview'**
  String get screenNotePreview;

  /// The name of a mock screen.
  ///
  /// In en, this message translates to:
  /// **'Note, edit'**
  String get screenNoteEdit;

  /// The name of a mock screen.
  ///
  /// In en, this message translates to:
  /// **'Search'**
  String get screenSearch;

  /// The name of a mock screen.
  ///
  /// In en, this message translates to:
  /// **'Settings'**
  String get screenSettings;

  /// The name of a mock screen.
  ///
  /// In en, this message translates to:
  /// **'Theme picker'**
  String get screenThemePicker;

  /// The name of a mock screen.
  ///
  /// In en, this message translates to:
  /// **'Locked note'**
  String get screenLockedNote;

  /// The name of a mock screen.
  ///
  /// In en, this message translates to:
  /// **'Error card'**
  String get screenError;

  /// The heading in a theme's sample Markdown; the # mark is added.
  ///
  /// In en, this message translates to:
  /// **'Heading'**
  String get themesSampleHeading;

  /// A line of a theme's sample Markdown with bold and italic text.
  ///
  /// In en, this message translates to:
  /// **'Some **bold** and *italic*'**
  String get themesSampleLine;

  /// A line of a theme's sample Markdown with code and a link.
  ///
  /// In en, this message translates to:
  /// **'`code`, [a link](url)'**
  String get themesSampleCode;

  /// The quote in a theme's sample Markdown; the > mark is added.
  ///
  /// In en, this message translates to:
  /// **'a quote'**
  String get themesSampleQuote;

  /// The name of a mock screen.
  ///
  /// In en, this message translates to:
  /// **'Settings, scrolled'**
  String get screenSettingsScrolled;

  /// The name of a mock screen.
  ///
  /// In en, this message translates to:
  /// **'Theme picker, scrolled'**
  String get screenThemePickerScrolled;

  /// The settings row that opens the fonts of notes.
  ///
  /// In en, this message translates to:
  /// **'Fonts'**
  String get settingsFonts;

  /// Goes back to the page before.
  ///
  /// In en, this message translates to:
  /// **'Back'**
  String get settingsBack;

  /// The title of the page where the fonts of notes are chosen.
  ///
  /// In en, this message translates to:
  /// **'Fonts'**
  String get fontsTitle;

  /// The section of fonts to write in.
  ///
  /// In en, this message translates to:
  /// **'Editor font'**
  String get fontsEditorSection;

  /// Explains the editor font.
  ///
  /// In en, this message translates to:
  /// **'The font you write in.'**
  String get fontsEditorFooter;

  /// The section of fonts to read notes in.
  ///
  /// In en, this message translates to:
  /// **'Preview font'**
  String get fontsPreviewSection;

  /// Explains the preview font.
  ///
  /// In en, this message translates to:
  /// **'The font a note is read in.'**
  String get fontsPreviewFooter;

  /// The section of fonts for code.
  ///
  /// In en, this message translates to:
  /// **'Code font'**
  String get fontsCodeSection;

  /// Explains the code font.
  ///
  /// In en, this message translates to:
  /// **'Used for code in notes and in theme previews.'**
  String get fontsCodeFooter;

  /// The choice that uses the font of the device, not one of Kanso’s.
  ///
  /// In en, this message translates to:
  /// **'System — your device’s font'**
  String get fontsSystemName;

  /// The short name of the device’s own font, for a summary.
  ///
  /// In en, this message translates to:
  /// **'System'**
  String get fontsSystemShort;

  /// Says which font the device’s own font is on each platform.
  ///
  /// In en, this message translates to:
  /// **'San Francisco on iOS, Roboto on Android'**
  String get fontsSystemSample;

  /// A short line shown in each font so it can be judged.
  ///
  /// In en, this message translates to:
  /// **'Calm notes, clearly written'**
  String get fontsSampleText;

  /// A short line of code shown in each code font.
  ///
  /// In en, this message translates to:
  /// **'let note = merge(a, b);'**
  String get fontsSampleCode;

  /// The section that sets how large the text of notes is.
  ///
  /// In en, this message translates to:
  /// **'Text size'**
  String get fontsTextSizeSection;

  /// Explains the text size.
  ///
  /// In en, this message translates to:
  /// **'Changes the text of notes only. The rest of Kanso follows your device’s text size.'**
  String get fontsTextSizeFooter;

  /// A paragraph that shows the chosen text size.
  ///
  /// In en, this message translates to:
  /// **'Calm software says less and remembers more. Every device keeps a full copy of your notes, and a change is merged on the others.'**
  String get fontsTextSizeSample;

  /// A text size.
  ///
  /// In en, this message translates to:
  /// **'Small'**
  String get textSizeSmall;

  /// The text size the app was designed at.
  ///
  /// In en, this message translates to:
  /// **'Default'**
  String get textSizeDefault;

  /// A text size.
  ///
  /// In en, this message translates to:
  /// **'Large'**
  String get textSizeLarge;

  /// The largest text size.
  ///
  /// In en, this message translates to:
  /// **'Larger'**
  String get textSizeLarger;

  /// Starts adding a font from a file.
  ///
  /// In en, this message translates to:
  /// **'Add a font…'**
  String get fontsAdd;

  /// Says which files can be added as fonts.
  ///
  /// In en, this message translates to:
  /// **'Choose a .ttf or .otf file from Files.'**
  String get fontsAddFooter;

  /// Names the switch between a rendered note and its text.
  ///
  /// In en, this message translates to:
  /// **'View mode'**
  String get noteModeLabel;

  /// Says to a screen reader that the list of notes is loading.
  ///
  /// In en, this message translates to:
  /// **'Loading notes'**
  String get noteListLoading;

  /// The title of a notebook that has no notes.
  ///
  /// In en, this message translates to:
  /// **'No notes in {notebook} yet'**
  String emptyNotebookTitle(String notebook);

  /// Says what an empty notebook will hold.
  ///
  /// In en, this message translates to:
  /// **'Notes added here will appear in this list.'**
  String get emptyNotebookMessage;

  /// The title of the area where a note is read when none is open.
  ///
  /// In en, this message translates to:
  /// **'No note selected'**
  String get noNoteSelectedTitle;

  /// Says how to open a note.
  ///
  /// In en, this message translates to:
  /// **'Choose a note from the list to read it here.'**
  String get noNoteSelectedMessage;

  /// The name of a mock screen.
  ///
  /// In en, this message translates to:
  /// **'Fonts'**
  String get screenFonts;

  /// The name of a mock screen.
  ///
  /// In en, this message translates to:
  /// **'Fonts, scrolled'**
  String get screenFontsScrolled;

  /// The name of a mock screen.
  ///
  /// In en, this message translates to:
  /// **'Note list, loading'**
  String get screenLoadingList;

  /// The name of a mock screen.
  ///
  /// In en, this message translates to:
  /// **'Empty notebook'**
  String get screenEmptyNotebook;

  /// The gallery section that opens the core check.
  ///
  /// In en, this message translates to:
  /// **'Library on this phone'**
  String get designGalleryCoreTitle;

  /// The gallery row that opens the core check page.
  ///
  /// In en, this message translates to:
  /// **'Rust core'**
  String get designGalleryCoreCheck;

  /// The title of the core check page.
  ///
  /// In en, this message translates to:
  /// **'Rust core'**
  String get coreCheckTitle;

  /// Says the library is being opened.
  ///
  /// In en, this message translates to:
  /// **'Opening the library…'**
  String get coreCheckLoading;

  /// The label above the core check's facts.
  ///
  /// In en, this message translates to:
  /// **'Library'**
  String get coreCheckLibrary;

  /// The core check row that shows the library's version.
  ///
  /// In en, this message translates to:
  /// **'Version'**
  String get coreCheckVersion;

  /// The core check row that says whether the library can check a server's certificate.
  ///
  /// In en, this message translates to:
  /// **'Certificates'**
  String get coreCheckCertificates;

  /// The library can check certificates.
  ///
  /// In en, this message translates to:
  /// **'Ready'**
  String get coreCheckCertificatesReady;

  /// The library can't check certificates yet.
  ///
  /// In en, this message translates to:
  /// **'Not ready'**
  String get coreCheckCertificatesNotReady;

  /// The core check row that shows how many notes the library holds.
  ///
  /// In en, this message translates to:
  /// **'Notes'**
  String get coreCheckNotes;

  /// The core check row that shows the last change the library reported.
  ///
  /// In en, this message translates to:
  /// **'Last change'**
  String get coreCheckLastChange;

  /// No change has been reported yet.
  ///
  /// In en, this message translates to:
  /// **'None yet'**
  String get coreCheckNoChange;

  /// How many notes the last reported change touched.
  ///
  /// In en, this message translates to:
  /// **'{count, plural, =1{1 note changed} other{{count} notes changed}}'**
  String coreCheckChanged(int count);

  /// What the core check's button does: makes a note.
  ///
  /// In en, this message translates to:
  /// **'New note'**
  String get coreCheckNewNote;

  /// The label above the newest notes in the core check.
  ///
  /// In en, this message translates to:
  /// **'Latest notes'**
  String get coreCheckLatest;

  /// Says the last action in the core check failed.
  ///
  /// In en, this message translates to:
  /// **'That didn\'t work. Try again.'**
  String get coreCheckActionFailed;

  /// The title of the core check's error card.
  ///
  /// In en, this message translates to:
  /// **'The library didn\'t open'**
  String get coreCheckFailedTitle;

  /// Says what happened, and what to do.
  ///
  /// In en, this message translates to:
  /// **'Something went wrong reading the notes on this phone. Try again.'**
  String get coreCheckFailedMessage;

  /// What the core check's error card button does.
  ///
  /// In en, this message translates to:
  /// **'Try again'**
  String get coreCheckRetry;

  /// A search page's title.
  ///
  /// In en, this message translates to:
  /// **'Search'**
  String get searchTitle;

  /// The library row that opens the gallery of the design proposal; in development builds only.
  ///
  /// In en, this message translates to:
  /// **'Design gallery'**
  String get libraryDesignGallery;

  /// The button that tries a failed reading again.
  ///
  /// In en, this message translates to:
  /// **'Try again'**
  String get retry;

  /// What a screen reader says while the library's rows are read.
  ///
  /// In en, this message translates to:
  /// **'Loading the library'**
  String get libraryLoading;

  /// The title of the card shown when the notebooks and tags could not be read.
  ///
  /// In en, this message translates to:
  /// **'The library didn\'t load'**
  String get libraryFailedTitle;

  /// The message of the card shown when the notebooks and tags could not be read.
  ///
  /// In en, this message translates to:
  /// **'Something went wrong reading the notebooks and tags on this phone. Try again.'**
  String get libraryFailedMessage;

  /// Shown under the Notebooks heading when the library has no notebooks.
  ///
  /// In en, this message translates to:
  /// **'No notebooks yet'**
  String get libraryNoNotebooks;

  /// Shown under the Tags heading when no note has a tag.
  ///
  /// In en, this message translates to:
  /// **'No tags yet'**
  String get libraryNoTags;

  /// A day in a list's header or a row's time: the day it is.
  ///
  /// In en, this message translates to:
  /// **'Today'**
  String get dayToday;

  /// A day in a list's header or a row's time: the day before.
  ///
  /// In en, this message translates to:
  /// **'Yesterday'**
  String get dayYesterday;

  /// What a note with no title is called in a list.
  ///
  /// In en, this message translates to:
  /// **'Untitled'**
  String get noteUntitled;

  /// The line of text a locked note's row shows in place of its text.
  ///
  /// In en, this message translates to:
  /// **'Locked'**
  String get noteListLocked;

  /// The title of the card shown when a list of notes could not be read.
  ///
  /// In en, this message translates to:
  /// **'These notes didn\'t load'**
  String get noteListFailedTitle;

  /// The message of the card shown when a list of notes could not be read.
  ///
  /// In en, this message translates to:
  /// **'Something went wrong reading the notes on this phone. Try again.'**
  String get noteListFailedMessage;

  /// The title shown when the library has no notes.
  ///
  /// In en, this message translates to:
  /// **'No notes yet'**
  String get emptyAllTitle;

  /// What the empty list of all notes says will appear there.
  ///
  /// In en, this message translates to:
  /// **'Notes will appear here.'**
  String get emptyAllMessage;

  /// The title shown when no note is pinned.
  ///
  /// In en, this message translates to:
  /// **'Nothing pinned'**
  String get emptyPinnedTitle;

  /// What the empty list of pinned notes says.
  ///
  /// In en, this message translates to:
  /// **'Pin a note by swiping it, and it will wait here.'**
  String get emptyPinnedMessage;

  /// The title shown when no note was changed lately.
  ///
  /// In en, this message translates to:
  /// **'Nothing recent'**
  String get emptyRecentTitle;

  /// What the empty list of recent notes says.
  ///
  /// In en, this message translates to:
  /// **'Notes you have changed lately will appear here.'**
  String get emptyRecentMessage;

  /// The title shown when the trash has no notes.
  ///
  /// In en, this message translates to:
  /// **'The trash is empty'**
  String get emptyTrashTitle;

  /// What the empty trash says.
  ///
  /// In en, this message translates to:
  /// **'Notes you trash wait here until you move them out.'**
  String get emptyTrashMessage;

  /// The title shown when no note has the tag.
  ///
  /// In en, this message translates to:
  /// **'No notes tagged {tag}'**
  String emptyTagTitle(String tag);

  /// What the empty list of a tag's notes says.
  ///
  /// In en, this message translates to:
  /// **'Notes with this tag will appear in this list.'**
  String get emptyTagMessage;

  /// The title shown when the list's narrowing hides every note.
  ///
  /// In en, this message translates to:
  /// **'No notes match'**
  String get emptyNarrowedTitle;

  /// What the list says when its narrowing hides every note.
  ///
  /// In en, this message translates to:
  /// **'Try another status, or turn off Pinned Only.'**
  String get emptyNarrowedMessage;

  /// The title of the sheet that sorts and narrows a list.
  ///
  /// In en, this message translates to:
  /// **'View options'**
  String get listOptionsTitle;

  /// The heading of the choices of what to sort a list by.
  ///
  /// In en, this message translates to:
  /// **'Sort by'**
  String get listOptionsSortBy;

  /// A way to sort a list: by when each note was last changed.
  ///
  /// In en, this message translates to:
  /// **'Date Updated'**
  String get listOptionsDateUpdated;

  /// A way to sort a list: by when each note was made.
  ///
  /// In en, this message translates to:
  /// **'Date Created'**
  String get listOptionsDateCreated;

  /// A way to sort a list: by each note's title.
  ///
  /// In en, this message translates to:
  /// **'Title'**
  String get listOptionsTitleSort;

  /// A direction of a list sorted by date.
  ///
  /// In en, this message translates to:
  /// **'Newest First'**
  String get listOptionsNewestFirst;

  /// A direction of a list sorted by date.
  ///
  /// In en, this message translates to:
  /// **'Oldest First'**
  String get listOptionsOldestFirst;

  /// A direction of a list sorted by title.
  ///
  /// In en, this message translates to:
  /// **'A to Z'**
  String get listOptionsAToZ;

  /// A direction of a list sorted by title.
  ///
  /// In en, this message translates to:
  /// **'Z to A'**
  String get listOptionsZToA;

  /// The heading of the choices of a status to narrow a list to.
  ///
  /// In en, this message translates to:
  /// **'Status'**
  String get listOptionsStatus;

  /// The choice that does not narrow a list by status.
  ///
  /// In en, this message translates to:
  /// **'Any Status'**
  String get listOptionsAnyStatus;

  /// The choice that narrows a list to pinned notes.
  ///
  /// In en, this message translates to:
  /// **'Pinned Only'**
  String get listOptionsPinnedOnly;

  /// The button that ends choosing several notes, or closes a sheet.
  ///
  /// In en, this message translates to:
  /// **'Done'**
  String get noteListDone;

  /// How many notes are chosen.
  ///
  /// In en, this message translates to:
  /// **'{count} selected'**
  String noteListSelected(int count);

  /// The action that pins the notes chosen.
  ///
  /// In en, this message translates to:
  /// **'Pin'**
  String get noteListPin;

  /// The action that unpins the notes chosen.
  ///
  /// In en, this message translates to:
  /// **'Unpin'**
  String get noteListUnpin;

  /// The action that moves notes to a notebook.
  ///
  /// In en, this message translates to:
  /// **'Move'**
  String get noteListMove;

  /// The action that moves notes to the trash.
  ///
  /// In en, this message translates to:
  /// **'Trash'**
  String get noteListTrash;
}

class _AppLocalizationsDelegate
    extends LocalizationsDelegate<AppLocalizations> {
  const _AppLocalizationsDelegate();

  @override
  Future<AppLocalizations> load(Locale locale) {
    return SynchronousFuture<AppLocalizations>(lookupAppLocalizations(locale));
  }

  @override
  bool isSupported(Locale locale) =>
      <String>['en'].contains(locale.languageCode);

  @override
  bool shouldReload(_AppLocalizationsDelegate old) => false;
}

AppLocalizations lookupAppLocalizations(Locale locale) {
  // Lookup logic when only language code is specified.
  switch (locale.languageCode) {
    case 'en':
      return AppLocalizationsEn();
  }

  throw FlutterError(
    'AppLocalizations.delegate failed to load unsupported locale "$locale". This is likely '
    'an issue with the localizations generation tool. Please file an issue '
    'on GitHub with a reproducible sample app and the gen-l10n configuration '
    'that was used.',
  );
}
