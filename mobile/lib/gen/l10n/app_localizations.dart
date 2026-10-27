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

  /// The title of the sheet that shares a note, and the label of the button that opens it.
  ///
  /// In en, this message translates to:
  /// **'Share'**
  String get shareTitle;

  /// Closes the share sheet.
  ///
  /// In en, this message translates to:
  /// **'Close'**
  String get shareClose;

  /// Says to a screen reader that a note's sharing is being read.
  ///
  /// In en, this message translates to:
  /// **'Loading sharing'**
  String get shareLoading;

  /// All the share sheet says of a locked note.
  ///
  /// In en, this message translates to:
  /// **'Locked notes can’t be shared.'**
  String get shareLockedNote;

  /// The share sheet's message while no account is signed in.
  ///
  /// In en, this message translates to:
  /// **'Sharing lets people you choose read, comment on or edit a note with you, end-to-end encrypted. It needs a Kanso account: sign in, or create one, then share.'**
  String get shareSignedOut;

  /// Opens Settings to sign in, from the share sheet.
  ///
  /// In en, this message translates to:
  /// **'Sign in…'**
  String get shareSignIn;

  /// The title of the card shown when a note's sharing could not be read.
  ///
  /// In en, this message translates to:
  /// **'Sharing didn’t load'**
  String get shareFailedTitle;

  /// The message of the card shown when a note's sharing could not be read.
  ///
  /// In en, this message translates to:
  /// **'Something went wrong reading this note’s sharing. Try again.'**
  String get shareFailedMessage;

  /// What the share sheet says of a note that is not shared yet.
  ///
  /// In en, this message translates to:
  /// **'Invite people by their Kanso username to read, comment on or edit this note with you. It stays end-to-end encrypted: your sync server passes it on without reading it.'**
  String get shareIntro;

  /// What the share sheet says to someone a note was shared with.
  ///
  /// In en, this message translates to:
  /// **'{owner} shared this note with you. You {access}.'**
  String shareSharedWithYou(String owner, String access);

  /// What a viewer can do with a shared note, to finish 'You …'.
  ///
  /// In en, this message translates to:
  /// **'can read it'**
  String get shareAccessView;

  /// What a commenter can do with a shared note, to finish 'You …'.
  ///
  /// In en, this message translates to:
  /// **'can read it and comment on it'**
  String get shareAccessComment;

  /// What an editor can do with a shared note, to finish 'You …'.
  ///
  /// In en, this message translates to:
  /// **'can edit it'**
  String get shareAccessEdit;

  /// The hint in the field for the username of the person to invite.
  ///
  /// In en, this message translates to:
  /// **'Username'**
  String get shareUsernameHint;

  /// Invites the person whose username is typed.
  ///
  /// In en, this message translates to:
  /// **'Invite'**
  String get shareInvite;

  /// Names the choice of role for an invitation, to a screen reader.
  ///
  /// In en, this message translates to:
  /// **'What the person you invite can do'**
  String get shareInviteRoleLabel;

  /// A person who owns a shared note.
  ///
  /// In en, this message translates to:
  /// **'Owner'**
  String get shareRoleOwner;

  /// A person who can edit a shared note.
  ///
  /// In en, this message translates to:
  /// **'Can edit'**
  String get shareRoleEditor;

  /// A person who can comment on a shared note.
  ///
  /// In en, this message translates to:
  /// **'Can comment'**
  String get shareRoleCommenter;

  /// A person who can only read a shared note.
  ///
  /// In en, this message translates to:
  /// **'Can view'**
  String get shareRoleViewer;

  /// Says that a person was invited.
  ///
  /// In en, this message translates to:
  /// **'Invited {username}. The note shows for them once they accept.'**
  String shareInvited(String username);

  /// Says that the owner stopped sharing a note.
  ///
  /// In en, this message translates to:
  /// **'Sharing stopped. Everyone keeps a private copy of the note as it is.'**
  String get shareStopped;

  /// Says that the person left a shared note.
  ///
  /// In en, this message translates to:
  /// **'You left the note. Your copy of it is private now.'**
  String get shareLeft;

  /// Says that no username was typed.
  ///
  /// In en, this message translates to:
  /// **'Type the username of the person to invite.'**
  String get shareProblemNameMissing;

  /// Says that no invite link was typed.
  ///
  /// In en, this message translates to:
  /// **'Paste the invite link first.'**
  String get shareProblemLinkMissing;

  /// Says that the sync server could not be reached while sharing.
  ///
  /// In en, this message translates to:
  /// **'Couldn’t reach your sync server. Check your connection and try again.'**
  String get shareProblemOffline;

  /// Says that the account signed the device out while sharing.
  ///
  /// In en, this message translates to:
  /// **'Your account signed this device out. Sign in again to share.'**
  String get shareProblemSignedOut;

  /// Says that a step of sharing failed for no reason the person can act on.
  ///
  /// In en, this message translates to:
  /// **'Something went wrong. Try again.'**
  String get shareProblemUnexpected;

  /// The label above the people a note is shared with.
  ///
  /// In en, this message translates to:
  /// **'People'**
  String get sharePeople;

  /// Says that a person has not accepted their invitation.
  ///
  /// In en, this message translates to:
  /// **'Invited, hasn’t accepted yet'**
  String get shareMemberInvited;

  /// Says that a person declined their invitation.
  ///
  /// In en, this message translates to:
  /// **'Declined'**
  String get shareMemberDeclined;

  /// Says that a person left the note.
  ///
  /// In en, this message translates to:
  /// **'Left'**
  String get shareMemberLeft;

  /// Marks the person's own row in the list of people.
  ///
  /// In en, this message translates to:
  /// **'You'**
  String get shareMemberYou;

  /// Says that a person's keys changed or could not be checked.
  ///
  /// In en, this message translates to:
  /// **'Can’t be checked'**
  String get shareMemberUnchecked;

  /// Says that a person's fingerprint was compared and matches.
  ///
  /// In en, this message translates to:
  /// **'Fingerprint verified'**
  String get shareMemberVerified;

  /// Reviews the new keys of a person whose keys changed.
  ///
  /// In en, this message translates to:
  /// **'Accept the new key'**
  String get shareMemberAcceptKey;

  /// Removes a person from a shared note, or cancels their invitation.
  ///
  /// In en, this message translates to:
  /// **'Remove from note'**
  String get shareMemberRemove;

  /// Names the button that changes a person's role, to a screen reader.
  ///
  /// In en, this message translates to:
  /// **'Change what {username} can do'**
  String shareMemberRoleLabel(String username);

  /// The label above the people who asked to join through a link.
  ///
  /// In en, this message translates to:
  /// **'Asked to join'**
  String get shareRequests;

  /// Says that a person asked to join through a link.
  ///
  /// In en, this message translates to:
  /// **'{username} asked to join'**
  String shareRequestAsked(String username);

  /// Under a request to join: the role the link gives and the person's fingerprint.
  ///
  /// In en, this message translates to:
  /// **'{role} · Fingerprint {fingerprint}'**
  String shareRequestDetail(String role, String fingerprint);

  /// Lets in a person who asked to join.
  ///
  /// In en, this message translates to:
  /// **'Approve'**
  String get shareApprove;

  /// Turns away a person who asked to join, or declines an invitation.
  ///
  /// In en, this message translates to:
  /// **'Decline'**
  String get shareDecline;

  /// The label above a note's invite links.
  ///
  /// In en, this message translates to:
  /// **'Invite links'**
  String get shareLinks;

  /// An invite link that has expired.
  ///
  /// In en, this message translates to:
  /// **'{role}, expired'**
  String shareLinkExpired(String role);

  /// An invite link and how long it still works.
  ///
  /// In en, this message translates to:
  /// **'{days, plural, =1{{role}, for 1 more day} other{{role}, for {days} more days}}'**
  String shareLinkDaysLeft(String role, int days);

  /// Cancels an invite link.
  ///
  /// In en, this message translates to:
  /// **'Cancel link'**
  String get shareLinkCancel;

  /// Makes an invite link.
  ///
  /// In en, this message translates to:
  /// **'Make link'**
  String get shareLinkMake;

  /// Names the choice of role for a link, to a screen reader.
  ///
  /// In en, this message translates to:
  /// **'What people who join with the link can do'**
  String get shareLinkRoleLabel;

  /// Names the choice of days for a link, to a screen reader.
  ///
  /// In en, this message translates to:
  /// **'How long the link works'**
  String get shareLinkDaysLabel;

  /// How many days a link works.
  ///
  /// In en, this message translates to:
  /// **'{days, plural, =1{1 day} other{{days} days}}'**
  String shareLinkDays(int days);

  /// Names the made link, to a screen reader.
  ///
  /// In en, this message translates to:
  /// **'Invite link'**
  String get shareLinkAddress;

  /// Copies the invite link.
  ///
  /// In en, this message translates to:
  /// **'Copy'**
  String get shareLinkCopy;

  /// Says that the invite link was copied.
  ///
  /// In en, this message translates to:
  /// **'Copied'**
  String get shareLinkCopied;

  /// Sends the invite link through the system's share sheet.
  ///
  /// In en, this message translates to:
  /// **'Send'**
  String get shareLinkSend;

  /// Says what an invite link does and that it can be seen once.
  ///
  /// In en, this message translates to:
  /// **'Anyone with the link can ask to join; you approve each of them here. Copy it now: it can’t be shown again.'**
  String get shareLinkNote;

  /// Stops sharing a note.
  ///
  /// In en, this message translates to:
  /// **'Stop sharing'**
  String get shareStop;

  /// Confirms stopping the sharing of a note.
  ///
  /// In en, this message translates to:
  /// **'Stop sharing for everyone'**
  String get shareStopEveryone;

  /// Leaves a shared note.
  ///
  /// In en, this message translates to:
  /// **'Leave'**
  String get shareLeave;

  /// Confirms leaving a shared note.
  ///
  /// In en, this message translates to:
  /// **'Leave the note'**
  String get shareLeaveNote;

  /// Warns before the owner stops sharing.
  ///
  /// In en, this message translates to:
  /// **'Stopping sharing ends the note for everyone in it. Each of them keeps a private copy of it as it is now.'**
  String get shareStopWarning;

  /// Warns before the person leaves a shared note.
  ///
  /// In en, this message translates to:
  /// **'Leaving takes you out of the note. You keep a private copy of it as it is now.'**
  String get shareLeaveWarning;

  /// Cancels stopping or leaving.
  ///
  /// In en, this message translates to:
  /// **'Keep sharing'**
  String get shareKeepSharing;

  /// Goes back from a question about someone's keys.
  ///
  /// In en, this message translates to:
  /// **'Back'**
  String get shareBack;

  /// Asks the person to compare a fingerprint before the first share with someone.
  ///
  /// In en, this message translates to:
  /// **'You’re sharing with {username} for the first time. Compare this fingerprint with the one they see in Settings › Account, in person or on a call. If it matches, nobody is standing between you.'**
  String shareVerifyIntro(String username);

  /// Marks a fingerprint as compared and invites the person.
  ///
  /// In en, this message translates to:
  /// **'Mark as verified and invite'**
  String get shareVerifyConfirm;

  /// Warns that someone's keys changed.
  ///
  /// In en, this message translates to:
  /// **'{username}’s keys changed since you last shared with them. That happens when they set up their account again, and also when someone tries to read what you share. Ask them which fingerprint they see now before you trust it.'**
  String shareKeyChangedIntro(String username);

  /// The label above the fingerprint pinned before.
  ///
  /// In en, this message translates to:
  /// **'Before'**
  String get shareKeyBefore;

  /// The label above the fingerprint the server gives now.
  ///
  /// In en, this message translates to:
  /// **'Now'**
  String get shareKeyNow;

  /// Pins the new keys of someone whose keys changed.
  ///
  /// In en, this message translates to:
  /// **'Trust the new keys'**
  String get shareKeyTrust;

  /// Turns down inviting someone whose keys changed.
  ///
  /// In en, this message translates to:
  /// **'Don’t invite'**
  String get shareKeyDontInvite;

  /// Leaves the question about a member's new keys.
  ///
  /// In en, this message translates to:
  /// **'Not now'**
  String get shareKeyNotNow;

  /// Names a fingerprint, to a screen reader.
  ///
  /// In en, this message translates to:
  /// **'{username}’s fingerprint'**
  String shareFingerprintLabel(String username);

  /// The title of the screen of invitations, and the library's row for it.
  ///
  /// In en, this message translates to:
  /// **'Shared with you'**
  String get sharedWithYouTitle;

  /// The title of the empty list of invitations.
  ///
  /// In en, this message translates to:
  /// **'Nothing shared with you'**
  String get sharedWithYouEmptyTitle;

  /// The message of the empty list of invitations.
  ///
  /// In en, this message translates to:
  /// **'When someone shares a note with you, it waits here until you accept it.'**
  String get sharedWithYouEmptyMessage;

  /// The title shown while no account is signed in.
  ///
  /// In en, this message translates to:
  /// **'Sign in to get shared notes'**
  String get sharedWithYouSignedOutTitle;

  /// The message shown while no account is signed in.
  ///
  /// In en, this message translates to:
  /// **'Notes other people share with you reach this phone once it is signed in to a Kanso account. Sign in from Settings.'**
  String get sharedWithYouSignedOutMessage;

  /// The title of the card shown when invitations could not be read.
  ///
  /// In en, this message translates to:
  /// **'Shared notes didn’t load'**
  String get sharedWithYouFailedTitle;

  /// The message of the card shown when invitations could not be read.
  ///
  /// In en, this message translates to:
  /// **'Something went wrong looking for notes shared with you. Try again.'**
  String get sharedWithYouFailedMessage;

  /// Says to a screen reader that invitations are being read.
  ///
  /// In en, this message translates to:
  /// **'Loading shared notes'**
  String get sharedWithYouLoading;

  /// The title of an invitation; the note's own title is sealed until it is accepted.
  ///
  /// In en, this message translates to:
  /// **'A note from {from}'**
  String sharedInvitationFrom(String from);

  /// Accepts an invitation to a shared note.
  ///
  /// In en, this message translates to:
  /// **'Accept'**
  String get sharedAccept;

  /// Opens the screen that joins a note with an invite link.
  ///
  /// In en, this message translates to:
  /// **'Join with a link'**
  String get sharedJoinWithLink;

  /// The title of the screen that joins a note with an invite link.
  ///
  /// In en, this message translates to:
  /// **'Join with a link'**
  String get joinTitle;

  /// Says what joining with a link does.
  ///
  /// In en, this message translates to:
  /// **'Paste the invite link you were sent. The note’s owner lets you in, and the note comes to your library once they do.'**
  String get joinMessage;

  /// The hint in the field for an invite link.
  ///
  /// In en, this message translates to:
  /// **'Invite link'**
  String get joinHint;

  /// Asks to join the note an invite link is for.
  ///
  /// In en, this message translates to:
  /// **'Join'**
  String get joinButton;

  /// Says who was asked to let the person in.
  ///
  /// In en, this message translates to:
  /// **'Asked {owner} to let you in. The note shows in your library once they approve.'**
  String joinAsked(String owner);

  /// Goes back to an empty field to join another note.
  ///
  /// In en, this message translates to:
  /// **'Join another note'**
  String get joinAnother;

  /// The label above the community's themes on the theme page.
  ///
  /// In en, this message translates to:
  /// **'Gallery'**
  String get galleryTitle;

  /// The hint in the gallery's search field.
  ///
  /// In en, this message translates to:
  /// **'Search themes'**
  String get gallerySearchHint;

  /// Clears the gallery's search field.
  ///
  /// In en, this message translates to:
  /// **'Clear search'**
  String get gallerySearchClear;

  /// Names the choice of order, to a screen reader.
  ///
  /// In en, this message translates to:
  /// **'Order of the themes'**
  String get galleryFilterLabel;

  /// Orders the gallery's themes by how many installed them.
  ///
  /// In en, this message translates to:
  /// **'Popular'**
  String get galleryFilterPopular;

  /// Orders the gallery's themes by when they were updated.
  ///
  /// In en, this message translates to:
  /// **'Recently updated'**
  String get galleryFilterRecent;

  /// A badge on a theme made by Kanso.
  ///
  /// In en, this message translates to:
  /// **'Official'**
  String get galleryBadgeOfficial;

  /// A badge on a theme made by the community.
  ///
  /// In en, this message translates to:
  /// **'Community'**
  String get galleryBadgeCommunity;

  /// A badge on a theme that draws icons rather than colours.
  ///
  /// In en, this message translates to:
  /// **'Icons'**
  String get galleryBadgeIcons;

  /// A badge on an installed theme the catalog withdrew.
  ///
  /// In en, this message translates to:
  /// **'Withdrawn'**
  String get galleryBadgeWithdrawn;

  /// Who made a theme.
  ///
  /// In en, this message translates to:
  /// **'by {author}'**
  String galleryBy(String author);

  /// A theme's version.
  ///
  /// In en, this message translates to:
  /// **'Version {version}'**
  String galleryVersion(String version);

  /// How many times a theme was installed.
  ///
  /// In en, this message translates to:
  /// **'{count, plural, =1{1 install} other{{count} installs}}'**
  String galleryInstalls(int count);

  /// Installs a theme from the gallery.
  ///
  /// In en, this message translates to:
  /// **'Install'**
  String get galleryInstall;

  /// Says that a theme from the gallery is installed.
  ///
  /// In en, this message translates to:
  /// **'Installed'**
  String get galleryInstalled;

  /// Updates an installed theme to the gallery's newer version.
  ///
  /// In en, this message translates to:
  /// **'Update'**
  String get galleryUpdate;

  /// Removes an installed theme.
  ///
  /// In en, this message translates to:
  /// **'Remove'**
  String get galleryRemove;

  /// Says that a theme is being installed.
  ///
  /// In en, this message translates to:
  /// **'Installing…'**
  String get galleryInstalling;

  /// Says that a theme is being updated.
  ///
  /// In en, this message translates to:
  /// **'Updating…'**
  String get galleryUpdating;

  /// Says that a theme is being removed.
  ///
  /// In en, this message translates to:
  /// **'Removing…'**
  String get galleryRemoving;

  /// Says that a theme can't be installed by this version.
  ///
  /// In en, this message translates to:
  /// **'Needs a newer Kanso'**
  String get galleryNeedsNewer;

  /// Says to a screen reader that the gallery is being read.
  ///
  /// In en, this message translates to:
  /// **'Loading the gallery'**
  String get galleryLoading;

  /// The title of the card shown when the gallery could not be read.
  ///
  /// In en, this message translates to:
  /// **'The gallery didn’t load'**
  String get galleryFailedTitle;

  /// The message of the card shown when the gallery could not be read.
  ///
  /// In en, this message translates to:
  /// **'Couldn’t reach the theme gallery. Check your connection and try again.'**
  String get galleryFailedMessage;

  /// Says that the gallery shown may be old.
  ///
  /// In en, this message translates to:
  /// **'Showing the last list. Couldn’t reach the gallery just now.'**
  String get galleryStale;

  /// Says that the gallery is being fetched again.
  ///
  /// In en, this message translates to:
  /// **'Checking for new themes…'**
  String get galleryRefreshing;

  /// The title shown when a search finds no theme.
  ///
  /// In en, this message translates to:
  /// **'No themes match'**
  String get galleryNoResultsTitle;

  /// The message shown when a search finds no theme.
  ///
  /// In en, this message translates to:
  /// **'Nothing in the gallery matches “{query}”. Try another word.'**
  String galleryNoResultsMessage(String query);

  /// The title shown when the gallery has no themes.
  ///
  /// In en, this message translates to:
  /// **'The gallery is empty'**
  String get galleryEmptyTitle;

  /// The message shown when the gallery has no themes.
  ///
  /// In en, this message translates to:
  /// **'There are no themes to show yet.'**
  String get galleryEmptyMessage;

  /// Says that the gallery could not be reached while installing.
  ///
  /// In en, this message translates to:
  /// **'Couldn’t reach the gallery. Check your connection and try again.'**
  String get galleryProblemOffline;

  /// Says that installing or removing a theme failed for no reason the person can act on.
  ///
  /// In en, this message translates to:
  /// **'Something went wrong. Try again.'**
  String get galleryProblemUnexpected;

  /// Reads the invitations again, as a screen reader says it.
  ///
  /// In en, this message translates to:
  /// **'Look for new shared notes'**
  String get sharedWithYouRefresh;

  /// The title of the card that says a request to join was sent.
  ///
  /// In en, this message translates to:
  /// **'Request sent'**
  String get joinSentTitle;
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
