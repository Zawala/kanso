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
  /// **'Appearance'**
  String get settingsAppearance;

  /// The colour theme.
  ///
  /// In en, this message translates to:
  /// **'Theme'**
  String get settingsTheme;

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

  /// Marks the theme in use.
  ///
  /// In en, this message translates to:
  /// **'In use'**
  String get themesCurrent;

  /// The title of the card shown in place of a locked note.
  ///
  /// In en, this message translates to:
  /// **'This note is locked'**
  String get lockedTitle;

  /// The title of the card shown when the session ended.
  ///
  /// In en, this message translates to:
  /// **'Signed out on this device'**
  String get errorSignedOutTitle;

  /// Says what happened and that no notes are lost.
  ///
  /// In en, this message translates to:
  /// **'Your session on this device ended. Sign in again to keep syncing; your notes stay here.'**
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
  /// **'Note, edit'**
  String get screenNoteEdit;

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
  /// **'Something went wrong reading the notebooks and tags on this device. Try again.'**
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
  /// **'Something went wrong reading the notes on this device. Try again.'**
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

  /// The title of the sheet that asks which notebook to move notes to.
  ///
  /// In en, this message translates to:
  /// **'{count, plural, =1{Move 1 note to} other{Move {count} notes to}}'**
  String moveSheetTitle(int count);

  /// The button that closes a sheet without doing what it offers.
  ///
  /// In en, this message translates to:
  /// **'Cancel'**
  String get noteListCancel;

  /// The line shown when a change to notes did not happen.
  ///
  /// In en, this message translates to:
  /// **'That didn\'t work. Try again.'**
  String get noteListChangeFailed;

  /// The line shown when notes could not be moved into or out of a locked notebook.
  ///
  /// In en, this message translates to:
  /// **'Locked notebooks can\'t take or give up notes here yet.'**
  String get noteListChangeLocked;

  /// The button that closes the line that says a change did not happen.
  ///
  /// In en, this message translates to:
  /// **'OK'**
  String get noteListDismiss;

  /// What a screen reader says while a note is read.
  ///
  /// In en, this message translates to:
  /// **'Loading the note'**
  String get noteLoading;

  /// The title shown when a note that was asked for does not exist.
  ///
  /// In en, this message translates to:
  /// **'This note isn\'t here'**
  String get noteNotFoundTitle;

  /// What the screen of a note that is not there says.
  ///
  /// In en, this message translates to:
  /// **'It may have been deleted, or it hasn\'t reached this device yet.'**
  String get noteNotFoundMessage;

  /// The title of the card shown when a note could not be read.
  ///
  /// In en, this message translates to:
  /// **'This note didn\'t load'**
  String get noteFailedTitle;

  /// The message of the card shown when a note could not be read.
  ///
  /// In en, this message translates to:
  /// **'Something went wrong reading the note on this device. Try again.'**
  String get noteFailedMessage;

  /// What the card of a locked note says about its text.
  ///
  /// In en, this message translates to:
  /// **'Its text stays sealed. Only its title can be shown here.'**
  String get lockedMessageSealed;

  /// The button of a text field's menu that cuts the selected text.
  ///
  /// In en, this message translates to:
  /// **'Cut'**
  String get menuCut;

  /// The button of a text field's menu that copies the selected text.
  ///
  /// In en, this message translates to:
  /// **'Copy'**
  String get menuCopy;

  /// The button of a text field's menu that pastes what was copied.
  ///
  /// In en, this message translates to:
  /// **'Paste'**
  String get menuPaste;

  /// The button of a text field's menu that selects all of its text.
  ///
  /// In en, this message translates to:
  /// **'Select All'**
  String get menuSelectAll;

  /// The title shown under an empty search.
  ///
  /// In en, this message translates to:
  /// **'Search your notes'**
  String get searchIdleTitle;

  /// What an empty search says it finds.
  ///
  /// In en, this message translates to:
  /// **'Find words in titles and text, or a tag.'**
  String get searchIdleMessage;

  /// The title shown when a search finds nothing.
  ///
  /// In en, this message translates to:
  /// **'No notes match'**
  String get searchNoResultsTitle;

  /// What a search that finds nothing says.
  ///
  /// In en, this message translates to:
  /// **'Nothing matched “{query}”. Try fewer or different words.'**
  String searchNoResultsMessage(String query);

  /// The title of the card shown when a search failed.
  ///
  /// In en, this message translates to:
  /// **'The search didn\'t run'**
  String get searchFailedTitle;

  /// The message of the card shown when a search failed.
  ///
  /// In en, this message translates to:
  /// **'Something went wrong searching the notes on this device. Try again.'**
  String get searchFailedMessage;

  /// What a screen reader says while the notes are searched.
  ///
  /// In en, this message translates to:
  /// **'Searching'**
  String get searchLoading;

  /// The Theme row's value while the light and the dark theme follow the system: the names of both.
  ///
  /// In en, this message translates to:
  /// **'{light}, {dark}'**
  String settingsThemeBoth(String light, String dark);

  /// The label of the row that shows the state of sync.
  ///
  /// In en, this message translates to:
  /// **'Status'**
  String get settingsSyncStatus;

  /// The state of sync while no account is signed in.
  ///
  /// In en, this message translates to:
  /// **'Off'**
  String get settingsSyncOff;

  /// The state of sync when an account is signed in and nothing has been synced yet.
  ///
  /// In en, this message translates to:
  /// **'On'**
  String get settingsSyncOn;

  /// The state of sync while it runs.
  ///
  /// In en, this message translates to:
  /// **'Syncing…'**
  String get settingsSyncSyncing;

  /// The state of sync when it has finished.
  ///
  /// In en, this message translates to:
  /// **'Synced'**
  String get settingsSyncSynced;

  /// The state of sync when it could not finish.
  ///
  /// In en, this message translates to:
  /// **'Needs attention'**
  String get settingsSyncAttention;

  /// Says under Sync that device backups do not hold the notes.
  ///
  /// In en, this message translates to:
  /// **'The notes on this device aren’t included in device backups.'**
  String get settingsSyncBackupFooter;

  /// The Settings row that signs in to sync.
  ///
  /// In en, this message translates to:
  /// **'Sign in'**
  String get settingsSyncSignIn;

  /// The Settings row for an account that is signed in but whose notes this device can't open yet.
  ///
  /// In en, this message translates to:
  /// **'Open your notes on this device'**
  String get settingsSyncOpenNotes;

  /// The Settings row that syncs at once.
  ///
  /// In en, this message translates to:
  /// **'Sync now'**
  String get settingsSyncNow;

  /// The label of the Settings row of the signed-in account, under its name.
  ///
  /// In en, this message translates to:
  /// **'Account'**
  String get settingsSyncAccount;

  /// Stops waiting or puts a form away.
  ///
  /// In en, this message translates to:
  /// **'Cancel'**
  String get accountCancel;

  /// Finishes a screen.
  ///
  /// In en, this message translates to:
  /// **'Done'**
  String get accountDone;

  /// Asks for the same thing again after a failure.
  ///
  /// In en, this message translates to:
  /// **'Try again'**
  String get accountRetry;

  /// Copies a code or a key.
  ///
  /// In en, this message translates to:
  /// **'Copy'**
  String get accountCopy;

  /// Says the code or key was copied.
  ///
  /// In en, this message translates to:
  /// **'Copied'**
  String get accountCopied;

  /// Said when an account task failed in a way the app has no words for.
  ///
  /// In en, this message translates to:
  /// **'Something went wrong. Try again in a moment.'**
  String get accountSomethingWrong;

  /// Said when the server ended this device's session.
  ///
  /// In en, this message translates to:
  /// **'This device was signed out. Sign in again.'**
  String get accountSignedOutError;

  /// Says to a screen reader that something is being read.
  ///
  /// In en, this message translates to:
  /// **'Loading'**
  String get accountLoading;

  /// The label of the username field.
  ///
  /// In en, this message translates to:
  /// **'Username'**
  String get accountUsername;

  /// The label of the password field.
  ///
  /// In en, this message translates to:
  /// **'Password'**
  String get accountPassword;

  /// The label of the field that repeats a new password.
  ///
  /// In en, this message translates to:
  /// **'Repeat password'**
  String get accountRepeatPassword;

  /// The label of the field for a new password.
  ///
  /// In en, this message translates to:
  /// **'New password'**
  String get accountNewPassword;

  /// The label of the field for the password in use.
  ///
  /// In en, this message translates to:
  /// **'Current password'**
  String get accountCurrentPassword;

  /// Said when a username breaks the rules.
  ///
  /// In en, this message translates to:
  /// **'Use 3–32 lowercase letters, digits, - or _, starting and ending with a letter or digit.'**
  String get formProblemUsernameInvalid;

  /// Said when a username is one nobody can sign up with.
  ///
  /// In en, this message translates to:
  /// **'That username is reserved. Try another.'**
  String get formProblemUsernameReserved;

  /// Said when a password is too short.
  ///
  /// In en, this message translates to:
  /// **'Use at least 12 characters. A few random words work well.'**
  String get formProblemPasswordTooShort;

  /// Said when a password is too long.
  ///
  /// In en, this message translates to:
  /// **'Use at most 256 characters.'**
  String get formProblemPasswordTooLong;

  /// Said when a password is the username.
  ///
  /// In en, this message translates to:
  /// **'Choose a password that isn’t your username.'**
  String get formProblemPasswordSameAsUsername;

  /// Said when a new password and its repeat differ.
  ///
  /// In en, this message translates to:
  /// **'The passwords don’t match.'**
  String get formProblemPasswordsDiffer;

  /// Said when a sign-in can't be right.
  ///
  /// In en, this message translates to:
  /// **'That username and password don’t match.'**
  String get formProblemCredentialsDontMatch;

  /// Said when no recovery key was typed.
  ///
  /// In en, this message translates to:
  /// **'A recovery key has 28 letters and digits, in groups of four.'**
  String get formProblemRecoveryKeyMissing;

  /// The label of the row that names the sync server.
  ///
  /// In en, this message translates to:
  /// **'Server'**
  String get accountServerLabel;

  /// The name of the default sync server.
  ///
  /// In en, this message translates to:
  /// **'Kanso sync'**
  String get accountServerDefault;

  /// Opens the field for the address of another sync server.
  ///
  /// In en, this message translates to:
  /// **'Use another server'**
  String get accountServerChange;

  /// The label of the field for the address of a sync server.
  ///
  /// In en, this message translates to:
  /// **'Server address'**
  String get accountServerAddress;

  /// The hint in the field for the address of a sync server.
  ///
  /// In en, this message translates to:
  /// **'https://sync.example.com'**
  String get accountServerAddressHint;

  /// Takes the address typed as the sync server.
  ///
  /// In en, this message translates to:
  /// **'Use this server'**
  String get accountServerUse;

  /// Goes back to the default sync server.
  ///
  /// In en, this message translates to:
  /// **'Use Kanso sync'**
  String get accountServerUseDefault;

  /// Says the server is being asked how it lets people in.
  ///
  /// In en, this message translates to:
  /// **'Checking the server…'**
  String get accountServerChecking;

  /// Said under the create account form when the server is closed to new accounts.
  ///
  /// In en, this message translates to:
  /// **'This server isn’t taking new accounts.'**
  String get accountServerNoSignups;

  /// Said when a server offers no sign-in method the app can use.
  ///
  /// In en, this message translates to:
  /// **'This server offers no way to sign in that this app can use.'**
  String get accountServerNoMethods;

  /// The title of the sign-in screen.
  ///
  /// In en, this message translates to:
  /// **'Sign in'**
  String get accountSignInTitle;

  /// The heading of the sign-in form.
  ///
  /// In en, this message translates to:
  /// **'Sign in to Kanso'**
  String get accountSignInHeading;

  /// Signs in with Apple.
  ///
  /// In en, this message translates to:
  /// **'Continue with Apple'**
  String get accountContinueWithApple;

  /// Signs in with Google.
  ///
  /// In en, this message translates to:
  /// **'Continue with Google'**
  String get accountContinueWithGoogle;

  /// Signs in with GitHub.
  ///
  /// In en, this message translates to:
  /// **'Continue with GitHub'**
  String get accountContinueWithGithub;

  /// Signs in with a passkey.
  ///
  /// In en, this message translates to:
  /// **'Sign in with a passkey'**
  String get accountSignInWithPasskey;

  /// Between the one-tap ways to sign in and the username form.
  ///
  /// In en, this message translates to:
  /// **'or'**
  String get accountOr;

  /// Signs in with the username and password typed.
  ///
  /// In en, this message translates to:
  /// **'Sign in'**
  String get accountSignInButton;

  /// Goes to the form that makes an account.
  ///
  /// In en, this message translates to:
  /// **'Create an account'**
  String get accountCreateAccountLink;

  /// Goes to the form that signs in with the recovery key.
  ///
  /// In en, this message translates to:
  /// **'Forgot your password? Use your recovery key'**
  String get accountForgotPassword;

  /// Said on the button while signing in.
  ///
  /// In en, this message translates to:
  /// **'Signing in…'**
  String get accountSigningIn;

  /// The title of the screen that makes an account.
  ///
  /// In en, this message translates to:
  /// **'Create your account'**
  String get accountCreateTitle;

  /// Makes the account.
  ///
  /// In en, this message translates to:
  /// **'Create account'**
  String get accountCreateButton;

  /// Said on the button while the account is made.
  ///
  /// In en, this message translates to:
  /// **'Creating…'**
  String get accountCreatingButton;

  /// Makes an account with a passkey.
  ///
  /// In en, this message translates to:
  /// **'Create with a passkey'**
  String get accountCreateWithPasskey;

  /// Says under the create account form that a recovery key comes next.
  ///
  /// In en, this message translates to:
  /// **'Kanso can’t reset your password; you’ll get a recovery key instead.'**
  String get accountCreateRecoveryNote;

  /// Goes back from creating an account to signing in.
  ///
  /// In en, this message translates to:
  /// **'I already have an account'**
  String get accountSignInInstead;

  /// The title of the screen that shows the code to type at GitHub.
  ///
  /// In en, this message translates to:
  /// **'Continue with GitHub'**
  String get accountGithubTitle;

  /// Says what the code is for.
  ///
  /// In en, this message translates to:
  /// **'Type this code at GitHub to sign in.'**
  String get accountGithubExplain;

  /// Opens GitHub's page for the code.
  ///
  /// In en, this message translates to:
  /// **'Open GitHub'**
  String get accountGithubOpen;

  /// Said while GitHub's page is waited on.
  ///
  /// In en, this message translates to:
  /// **'Waiting for you to finish at GitHub…'**
  String get accountGithubWaiting;

  /// Stops signing in with a provider.
  ///
  /// In en, this message translates to:
  /// **'Cancel'**
  String get accountProviderCancel;

  /// Said while a provider's page is open.
  ///
  /// In en, this message translates to:
  /// **'Finish signing in in the page that opened. Kanso opens again when you’re done.'**
  String get accountProviderWaiting;

  /// The title of the screen that asks for a username for a new identity.
  ///
  /// In en, this message translates to:
  /// **'Choose a username'**
  String get accountUsernameTitle;

  /// Says what the username is for.
  ///
  /// In en, this message translates to:
  /// **'Your account is made with this name. You can change it before creating the account.'**
  String get accountUsernameExplain;

  /// The title of the screen that shows the recovery key.
  ///
  /// In en, this message translates to:
  /// **'Your recovery key'**
  String get accountRecoveryKeyTitle;

  /// Explains the recovery key.
  ///
  /// In en, this message translates to:
  /// **'If you forget your password, this key and your username are the only way back into your account. Kanso and the server’s owner can’t reset it. Anyone with it can take over your account and read your notes, so keep it somewhere safe, like a password manager or on paper. Each time it’s used, Kanso gives you a new one.'**
  String get accountRecoveryKeyText;

  /// Copies the recovery key.
  ///
  /// In en, this message translates to:
  /// **'Copy recovery key'**
  String get accountRecoveryKeyCopy;

  /// The box that says the recovery key was saved.
  ///
  /// In en, this message translates to:
  /// **'I’ve saved my recovery key somewhere safe'**
  String get accountRecoveryKeyConfirm;

  /// The title of the card shown when there is no recovery key waiting to be saved.
  ///
  /// In en, this message translates to:
  /// **'Nothing to save'**
  String get accountRecoveryKeyNoneTitle;

  /// Says there is no recovery key waiting to be saved.
  ///
  /// In en, this message translates to:
  /// **'There’s no recovery key waiting to be saved.'**
  String get accountRecoveryKeyNoneMessage;

  /// The title of the card shown when the recovery key could not be read.
  ///
  /// In en, this message translates to:
  /// **'The recovery key didn’t load'**
  String get accountRecoveryKeyFailedTitle;

  /// The message of the card shown when the recovery key could not be read.
  ///
  /// In en, this message translates to:
  /// **'Something went wrong reading the recovery key. Try again.'**
  String get accountRecoveryKeyFailedMessage;

  /// Uploads the notes on this phone to the account.
  ///
  /// In en, this message translates to:
  /// **'Add them to the account'**
  String get bindingAdd;

  /// Says what adding the notes does.
  ///
  /// In en, this message translates to:
  /// **'They’re encrypted and uploaded. A note deleted in the account but still on this device comes back, and a note changed on both sides keeps both versions.'**
  String get bindingAddNote;

  /// Replaces the notes on this phone with the account's.
  ///
  /// In en, this message translates to:
  /// **'Replace them with the account’s notes'**
  String get bindingReplace;

  /// Says what replacing the notes does.
  ///
  /// In en, this message translates to:
  /// **'Kanso saves a backup of this device’s library first, then downloads the account’s notes.'**
  String get bindingReplaceNote;

  /// Leaves the notes and the account as they were.
  ///
  /// In en, this message translates to:
  /// **'Cancel'**
  String get bindingCancel;

  /// The title of the screen for a signed-in account whose notes this device can't open yet.
  ///
  /// In en, this message translates to:
  /// **'Open your notes on this device'**
  String get accountUnlockTitle;

  /// Explains why the notes are closed.
  ///
  /// In en, this message translates to:
  /// **'You’re signed in as {username}, but this device can’t read your notes yet.'**
  String accountUnlockExplain(String username);

  /// Opens the notes with the password typed.
  ///
  /// In en, this message translates to:
  /// **'Unlock'**
  String get accountUnlockButton;

  /// Asks a device that is already signed in to let this one in.
  ///
  /// In en, this message translates to:
  /// **'Approve from another device'**
  String get accountApproveFromDevice;

  /// Goes to the form that opens the notes with the recovery key.
  ///
  /// In en, this message translates to:
  /// **'Use your recovery key'**
  String get accountUseRecoveryKey;

  /// Signs this device out of the account.
  ///
  /// In en, this message translates to:
  /// **'Sign out'**
  String get accountSignOut;

  /// The title of the screen that waits for another device to approve this one.
  ///
  /// In en, this message translates to:
  /// **'Approve this device'**
  String get accountApprovalTitle;

  /// Says what to do while waiting.
  ///
  /// In en, this message translates to:
  /// **'Open Kanso on a device that’s already signed in and approve this one.'**
  String get accountApprovalIntro;

  /// Said while the request is sent.
  ///
  /// In en, this message translates to:
  /// **'Asking another device…'**
  String get accountApprovalRequesting;

  /// Said while no device has answered.
  ///
  /// In en, this message translates to:
  /// **'Waiting for another device…'**
  String get accountApprovalWaiting;

  /// Said while the notes are being opened after an approval.
  ///
  /// In en, this message translates to:
  /// **'Approved. Unlocking this device…'**
  String get accountApprovalUnlocking;

  /// Said when the approval went through.
  ///
  /// In en, this message translates to:
  /// **'This device is unlocked.'**
  String get accountApprovalApproved;

  /// Said when no device answered.
  ///
  /// In en, this message translates to:
  /// **'No other device answered. Use your recovery key to open this device.'**
  String get accountApprovalNoDevices;

  /// Said when the request was cancelled.
  ///
  /// In en, this message translates to:
  /// **'That request was cancelled.'**
  String get accountApprovalCancelled;

  /// Said when the request ran out of time.
  ///
  /// In en, this message translates to:
  /// **'That request expired.'**
  String get accountApprovalExpired;

  /// Said when the server could not be reached while waiting.
  ///
  /// In en, this message translates to:
  /// **'Can’t reach the server. Your notes are safe on this device.'**
  String get accountApprovalFailed;

  /// Says to compare the code with the other device.
  ///
  /// In en, this message translates to:
  /// **'On your other device, approve the request only if it shows this code.'**
  String get accountApprovalCompare;

  /// Opens the notes with the recovery key instead of waiting.
  ///
  /// In en, this message translates to:
  /// **'Use your recovery key instead'**
  String get accountApprovalRecoveryInstead;

  /// Reads out the approval code to a screen reader.
  ///
  /// In en, this message translates to:
  /// **'Code {code}'**
  String accountApprovalCode(String code);

  /// The title of the screen that signs in with the recovery key.
  ///
  /// In en, this message translates to:
  /// **'Reset your password'**
  String get accountRecoverTitle;

  /// Explains what recovering does.
  ///
  /// In en, this message translates to:
  /// **'Enter your recovery key and choose a new password. Your other devices sign out and ask for it, and you get a new recovery key to save.'**
  String get accountRecoverExplain;

  /// The label of the field for the recovery key.
  ///
  /// In en, this message translates to:
  /// **'Recovery key'**
  String get accountRecoveryKeyField;

  /// The hint in the field for the recovery key.
  ///
  /// In en, this message translates to:
  /// **'The key you saved, in groups of four.'**
  String get accountRecoveryKeyHint;

  /// Signs in with the recovery key and sets the new password.
  ///
  /// In en, this message translates to:
  /// **'Reset password'**
  String get accountRecoverButton;

  /// Said on the button while recovering.
  ///
  /// In en, this message translates to:
  /// **'Resetting…'**
  String get accountRecoveringButton;

  /// The title of the Account screen.
  ///
  /// In en, this message translates to:
  /// **'Account'**
  String get accountTitle;

  /// Says who is signed in.
  ///
  /// In en, this message translates to:
  /// **'Signed in as {username}'**
  String accountSignedInAs(String username);

  /// The title of the card shown when the account could not be read.
  ///
  /// In en, this message translates to:
  /// **'Your account didn’t load'**
  String get accountLoadFailedTitle;

  /// The message of the card shown when the account could not be read.
  ///
  /// In en, this message translates to:
  /// **'Something went wrong reading your account. Try again.'**
  String get accountLoadFailedMessage;

  /// Says a recovery key is waiting to be saved.
  ///
  /// In en, this message translates to:
  /// **'Your new recovery key isn’t saved yet'**
  String get accountRecoveryKeyUnsavedTitle;

  /// Says why the new recovery key matters.
  ///
  /// In en, this message translates to:
  /// **'Save it now: the one you had before no longer works.'**
  String get accountRecoveryKeyUnsavedMessage;

  /// Shows the recovery key that is waiting to be saved.
  ///
  /// In en, this message translates to:
  /// **'Show it'**
  String get accountRecoveryKeyShow;

  /// The label above the account's devices.
  ///
  /// In en, this message translates to:
  /// **'Devices'**
  String get accountDevices;

  /// Marks the device in hand in the list of devices.
  ///
  /// In en, this message translates to:
  /// **'This device'**
  String get accountThisDevice;

  /// When a device last synced.
  ///
  /// In en, this message translates to:
  /// **'Last seen {date}'**
  String accountLastSeen(String date);

  /// Signs another device out.
  ///
  /// In en, this message translates to:
  /// **'Sign out'**
  String get accountDeviceSignOut;

  /// The label above the ways to sign in to the account.
  ///
  /// In en, this message translates to:
  /// **'Ways to sign in'**
  String get accountWaysToSignIn;

  /// The row for the account's password.
  ///
  /// In en, this message translates to:
  /// **'Password'**
  String get accountPasswordRow;

  /// Changes the password.
  ///
  /// In en, this message translates to:
  /// **'Change'**
  String get accountPasswordChange;

  /// Says the account has no password.
  ///
  /// In en, this message translates to:
  /// **'Not set'**
  String get accountPasswordNotSet;

  /// The label above the passkeys of the account.
  ///
  /// In en, this message translates to:
  /// **'Passkeys'**
  String get accountPasskeys;

  /// Adds a passkey to the account.
  ///
  /// In en, this message translates to:
  /// **'Add a passkey'**
  String get accountPasskeyAdd;

  /// Removes a passkey.
  ///
  /// In en, this message translates to:
  /// **'Remove'**
  String get accountPasskeyRemove;

  /// When a passkey was added.
  ///
  /// In en, this message translates to:
  /// **'Added {date}'**
  String accountPasskeyAdded(String date);

  /// When a passkey was added and last used.
  ///
  /// In en, this message translates to:
  /// **'Added {added} · Last used {used}'**
  String accountPasskeyUsed(String added, String used);

  /// Says a passkey signs in and opens the notes.
  ///
  /// In en, this message translates to:
  /// **'Can unlock a new device'**
  String get accountPasskeyCanUnlock;

  /// Says a passkey signs in but doesn't open the notes.
  ///
  /// In en, this message translates to:
  /// **'Signs in only'**
  String get accountPasskeySignsInOnly;

  /// The name of Apple as a way to sign in.
  ///
  /// In en, this message translates to:
  /// **'Apple'**
  String get accountProviderApple;

  /// The name of Google as a way to sign in.
  ///
  /// In en, this message translates to:
  /// **'Google'**
  String get accountProviderGoogle;

  /// The name of GitHub as a way to sign in.
  ///
  /// In en, this message translates to:
  /// **'GitHub'**
  String get accountProviderGithub;

  /// Says the account can sign in with the provider.
  ///
  /// In en, this message translates to:
  /// **'Linked'**
  String get accountProviderLinked;

  /// Says the account can't sign in with the provider.
  ///
  /// In en, this message translates to:
  /// **'Not linked'**
  String get accountProviderNotLinked;

  /// Links the provider.
  ///
  /// In en, this message translates to:
  /// **'Link'**
  String get accountProviderLink;

  /// Unlinks the provider.
  ///
  /// In en, this message translates to:
  /// **'Unlink'**
  String get accountProviderUnlink;

  /// Says changes need the password, which this account doesn't have.
  ///
  /// In en, this message translates to:
  /// **'Confirm it’s you with your password. This device can’t use another way for this account yet.'**
  String get accountNoProofHere;

  /// The label above the recovery key row.
  ///
  /// In en, this message translates to:
  /// **'Recovery key'**
  String get accountRecoveryKeySection;

  /// Makes a new recovery key.
  ///
  /// In en, this message translates to:
  /// **'Make a new one'**
  String get accountRecoveryKeyNew;

  /// Says what the recovery key is for.
  ///
  /// In en, this message translates to:
  /// **'Your recovery key still works if you lose your other ways in.'**
  String get accountRecoveryKeyFooter;

  /// The row that says how much of the account's space is used.
  ///
  /// In en, this message translates to:
  /// **'Storage'**
  String get accountStorage;

  /// Deletes the account.
  ///
  /// In en, this message translates to:
  /// **'Delete account'**
  String get accountDeleteAccount;

  /// Said while a change is made.
  ///
  /// In en, this message translates to:
  /// **'Working…'**
  String get accountWorking;

  /// The title of the question before signing another device out.
  ///
  /// In en, this message translates to:
  /// **'Sign out {name}?'**
  String accountRevokeTitle(String name);

  /// Says what signing another device out does.
  ///
  /// In en, this message translates to:
  /// **'It stops syncing until someone signs in on it again. Its notes stay on it.'**
  String get accountRevokeMessage;

  /// The title of the question before signing out.
  ///
  /// In en, this message translates to:
  /// **'Sign out of {username}?'**
  String accountSignOutTitle(String username);

  /// Says what signing out does.
  ///
  /// In en, this message translates to:
  /// **'Notes stay on this device. Sync stops until you sign in again.'**
  String get accountSignOutMessage;

  /// Warns that signing out loses a recovery key not yet saved.
  ///
  /// In en, this message translates to:
  /// **'Your new recovery key hasn’t been saved yet; signing out deletes it.'**
  String get accountSignOutUnsavedKey;

  /// The title of the form that changes the password.
  ///
  /// In en, this message translates to:
  /// **'Change password'**
  String get accountChangePasswordTitle;

  /// Says what changing the password does.
  ///
  /// In en, this message translates to:
  /// **'Your other devices sign out and ask for the new password. Your recovery key changes too: the one you saved stops working, and Kanso shows the new one to save.'**
  String get accountChangePasswordMessage;

  /// The title of the form that makes a new recovery key.
  ///
  /// In en, this message translates to:
  /// **'New recovery key'**
  String get accountNewRecoveryKeyTitle;

  /// Says what making a new recovery key does.
  ///
  /// In en, this message translates to:
  /// **'The key you saved stops working, and Kanso shows the new one to save instead.'**
  String get accountNewRecoveryKeyMessage;

  /// Makes the new recovery key.
  ///
  /// In en, this message translates to:
  /// **'Make a new key'**
  String get accountNewRecoveryKeyButton;

  /// The title of the form that deletes the account.
  ///
  /// In en, this message translates to:
  /// **'Delete account'**
  String get accountDeleteTitle;

  /// Says what deleting the account does.
  ///
  /// In en, this message translates to:
  /// **'This deletes {username}’s encrypted notes, attachments and devices from the server. Notes on this device stay here, and other devices keep their copies. The server’s backups keep a copy for up to 30 days. This can’t be undone.'**
  String accountDeleteMessage(String username);

  /// The label of the field that asks for the username before deleting.
  ///
  /// In en, this message translates to:
  /// **'Type {username} to confirm'**
  String accountDeleteTypeUsername(String username);

  /// The title of the form that adds a passkey.
  ///
  /// In en, this message translates to:
  /// **'Add a passkey'**
  String get accountAddPasskeyTitle;

  /// Says how to add a passkey.
  ///
  /// In en, this message translates to:
  /// **'Confirm it’s you with your password, then use your phone’s passkey sheet.'**
  String get accountAddPasskeyMessage;

  /// The title of the question before removing a passkey.
  ///
  /// In en, this message translates to:
  /// **'Remove {label}?'**
  String accountRemovePasskeyTitle(String label);

  /// Says what removing a passkey does.
  ///
  /// In en, this message translates to:
  /// **'You won’t be able to sign in with it any more. Confirm it’s you with your password.'**
  String get accountRemovePasskeyMessage;

  /// Removes the passkey.
  ///
  /// In en, this message translates to:
  /// **'Remove passkey'**
  String get accountRemovePasskeyButton;

  /// The title of the form that links a provider.
  ///
  /// In en, this message translates to:
  /// **'Link {provider}'**
  String accountLinkTitle(String provider);

  /// Says how to link a provider.
  ///
  /// In en, this message translates to:
  /// **'Confirm it’s you with your password, then finish in the page that opens.'**
  String get accountLinkMessage;

  /// The title of the question before unlinking a provider.
  ///
  /// In en, this message translates to:
  /// **'Unlink {provider}?'**
  String accountUnlinkTitle(String provider);

  /// Says what unlinking a provider does.
  ///
  /// In en, this message translates to:
  /// **'It no longer signs in to this account. Confirm it’s you with your password.'**
  String get accountUnlinkMessage;

  /// Confirms signing out.
  ///
  /// In en, this message translates to:
  /// **'Sign out'**
  String get accountConfirmSignOut;

  /// Said after another device was signed out.
  ///
  /// In en, this message translates to:
  /// **'Signed out {name}.'**
  String accountNoticeDeviceSignedOut(String name);

  /// Said after a passkey that can unlock was added.
  ///
  /// In en, this message translates to:
  /// **'Added the passkey. It can unlock: on a new device it signs in and opens your notes.'**
  String get accountNoticePasskeyAddedCanUnlock;

  /// Said after a passkey that only signs in was added.
  ///
  /// In en, this message translates to:
  /// **'Added the passkey. It signs in only: a new device will also need approval or your recovery key.'**
  String get accountNoticePasskeyAddedSignsIn;

  /// Said after a passkey was removed.
  ///
  /// In en, this message translates to:
  /// **'Removed the passkey.'**
  String get accountNoticePasskeyRemoved;

  /// Said after a provider was linked.
  ///
  /// In en, this message translates to:
  /// **'Linked. You can sign in with it now.'**
  String get accountNoticeProviderLinked;

  /// Said after a provider was unlinked.
  ///
  /// In en, this message translates to:
  /// **'Unlinked. It no longer signs in to this account.'**
  String get accountNoticeProviderUnlinked;

  /// The title of the sheet that asks to let another device in.
  ///
  /// In en, this message translates to:
  /// **'Approve a device'**
  String get approveDeviceTitle;

  /// Says who asks to join and when.
  ///
  /// In en, this message translates to:
  /// **'A device wants to join {account}: {device}, asked at {time}.'**
  String approveDeviceMessage(String account, String device, String time);

  /// What an unnamed device is called.
  ///
  /// In en, this message translates to:
  /// **'A device'**
  String get approveDeviceUnnamed;

  /// Asks to compare the code with the other device.
  ///
  /// In en, this message translates to:
  /// **'Does it show {code}? Only approve if it does.'**
  String approveDeviceCompare(String code);

  /// Said when a request is not what it was.
  ///
  /// In en, this message translates to:
  /// **'This request changed after it was made. Don’t approve.'**
  String get approveDeviceChanged;

  /// Lets the device in.
  ///
  /// In en, this message translates to:
  /// **'Approve'**
  String get approveDeviceApprove;

  /// Turns the device away.
  ///
  /// In en, this message translates to:
  /// **'Don’t approve'**
  String get approveDeviceDecline;

  /// Says how many more devices ask to join.
  ///
  /// In en, this message translates to:
  /// **'{count, plural, =1{1 more request is waiting.} other{{count} more requests are waiting.}}'**
  String approveDeviceMore(int count);

  /// The title of the card shown when the server could not be asked how it lets people in.
  ///
  /// In en, this message translates to:
  /// **'This server didn’t answer'**
  String get accountServerFailedTitle;

  /// Said when the name typed to confirm deleting the account is not the account's.
  ///
  /// In en, this message translates to:
  /// **'That isn’t this account’s username.'**
  String get accountDeleteMismatch;

  /// The app's version and the number of its build.
  ///
  /// In en, this message translates to:
  /// **'{version} ({build})'**
  String settingsVersionValue(String version, String build);

  /// Says to a screen reader that the licences are being read.
  ///
  /// In en, this message translates to:
  /// **'Loading licences'**
  String get licencesLoading;

  /// The title of the card shown when the licences could not be read.
  ///
  /// In en, this message translates to:
  /// **'The licences didn’t load'**
  String get licencesFailedTitle;

  /// The message of the card shown when the licences could not be read.
  ///
  /// In en, this message translates to:
  /// **'Something went wrong reading the licences. Try again.'**
  String get licencesFailedMessage;

  /// The title shown when there are no licences to list.
  ///
  /// In en, this message translates to:
  /// **'No licences'**
  String get licencesEmptyTitle;

  /// Says what the empty list of licences will hold.
  ///
  /// In en, this message translates to:
  /// **'The packages Kanso is made of will be listed here.'**
  String get licencesEmptyMessage;

  /// How many licences a package comes under.
  ///
  /// In en, this message translates to:
  /// **'{count, plural, =1{1 licence} other{{count} licences}}'**
  String licencesCount(int count);

  /// The switch that makes the interface follow the system's light or dark mode.
  ///
  /// In en, this message translates to:
  /// **'Switch between a light and a dark theme with the system'**
  String get themesFollowSystem;

  /// Says what following the system does, while it is on.
  ///
  /// In en, this message translates to:
  /// **'Showing the light theme in light mode and the dark one in dark mode.'**
  String get themesFollowSystemOn;

  /// Says what happens while following the system is off.
  ///
  /// In en, this message translates to:
  /// **'Off: the theme stays as picked whatever the system’s mode.'**
  String get themesFollowSystemOff;

  /// The title of the card shown when the themes could not be read.
  ///
  /// In en, this message translates to:
  /// **'The themes didn’t load'**
  String get themesFailedTitle;

  /// The message of the card shown when the themes could not be read.
  ///
  /// In en, this message translates to:
  /// **'Something went wrong reading the themes. Try again.'**
  String get themesFailedMessage;

  /// Says to a screen reader that the themes are being read.
  ///
  /// In en, this message translates to:
  /// **'Loading themes'**
  String get themesLoading;

  /// The title of the card that says the notes are not in device backups.
  ///
  /// In en, this message translates to:
  /// **'Notes stay on this device'**
  String get backupNoticeTitle;

  /// Says that device backups do not hold the notes.
  ///
  /// In en, this message translates to:
  /// **'The notes on this device aren’t included in device backups, so losing or resetting the device takes them with it.'**
  String get backupNoticeMessage;

  /// Dismisses the notice about device backups.
  ///
  /// In en, this message translates to:
  /// **'OK'**
  String get backupNoticeOk;
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
