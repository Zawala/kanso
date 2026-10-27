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

  /// What the undo key of the keyboard bar does when it is held, for a screen reader's list of actions.
  ///
  /// In en, this message translates to:
  /// **'Redo'**
  String get accessoryRedo;

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

  /// The choice of the first notebook in the library as the one new notes go to.
  ///
  /// In en, this message translates to:
  /// **'First notebook'**
  String get defaultNotebookFirst;

  /// The note under the notebooks that can be the default one.
  ///
  /// In en, this message translates to:
  /// **'New notes go here, except those made from a notebook\'s own list. A locked notebook can\'t be chosen.'**
  String get defaultNotebookFooter;

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

  /// Opens what changed in each version of Kanso, on the website.
  ///
  /// In en, this message translates to:
  /// **'Release notes'**
  String get settingsReleaseNotes;

  /// Opens Kanso's help page on the website.
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

  /// The settings row that opens the icon themes.
  ///
  /// In en, this message translates to:
  /// **'Icons'**
  String get settingsIcons;

  /// The title of the page where the icon theme is chosen.
  ///
  /// In en, this message translates to:
  /// **'Icons'**
  String get iconsTitle;

  /// Explains the icon themes, under the list.
  ///
  /// In en, this message translates to:
  /// **'The icon themes of the desktop app. Choosing one changes every icon at once; an icon a theme doesn’t draw stays Kanso’s own.'**
  String get iconsFooter;

  /// Who made an icon theme and under which licence, when both are known.
  ///
  /// In en, this message translates to:
  /// **'{author}, {license}'**
  String iconsCredit(String author, String license);

  /// Says to a screen reader what a row of sample icons shows.
  ///
  /// In en, this message translates to:
  /// **'Icons in {name}'**
  String iconsPreview(String name);

  /// Says to a screen reader that the icon themes are being read.
  ///
  /// In en, this message translates to:
  /// **'Loading icon themes'**
  String get iconsLoading;

  /// The title of the card shown when the icon themes could not be read.
  ///
  /// In en, this message translates to:
  /// **'The icon themes didn’t load'**
  String get iconsFailedTitle;

  /// The message of the card shown when the icon themes could not be read.
  ///
  /// In en, this message translates to:
  /// **'Something went wrong reading the icon themes. Try again.'**
  String get iconsFailedMessage;

  /// The title of the card shown when there are no icon themes to choose from.
  ///
  /// In en, this message translates to:
  /// **'No icon themes'**
  String get iconsEmptyTitle;

  /// The message of the card shown when there are no icon themes to choose from.
  ///
  /// In en, this message translates to:
  /// **'Kanso’s own icons are in use.'**
  String get iconsEmptyMessage;

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

  /// What the empty title field of a note being edited says.
  ///
  /// In en, this message translates to:
  /// **'Title'**
  String get editorTitleHint;

  /// Says a note was changed elsewhere while it was being edited.
  ///
  /// In en, this message translates to:
  /// **'This note was changed elsewhere while you were editing. The version you don\'t keep stays in its history.'**
  String get editorChangedMessage;

  /// Keeps the text typed here over a change made elsewhere.
  ///
  /// In en, this message translates to:
  /// **'Keep mine'**
  String get editorKeepMine;

  /// Takes a change made elsewhere over the text typed here.
  ///
  /// In en, this message translates to:
  /// **'Use theirs'**
  String get editorUseTheirs;

  /// Says a note was deleted elsewhere while it was being edited.
  ///
  /// In en, this message translates to:
  /// **'This note was deleted elsewhere while you were editing.'**
  String get editorDeletedMessage;

  /// Keeps a note that was deleted elsewhere, by saving it again.
  ///
  /// In en, this message translates to:
  /// **'Keep it'**
  String get editorKeepIt;

  /// Lets go of a note that was deleted elsewhere.
  ///
  /// In en, this message translates to:
  /// **'Discard'**
  String get editorDiscard;

  /// Says that saving a note failed.
  ///
  /// In en, this message translates to:
  /// **'Your changes aren\'t saved yet.'**
  String get editorSaveFailed;

  /// What a screen reader says while a note is opened for editing.
  ///
  /// In en, this message translates to:
  /// **'Opening the note'**
  String get editorOpeningLabel;

  /// Adds a property to the note.
  ///
  /// In en, this message translates to:
  /// **'Add property'**
  String get propertiesAdd;

  /// The hint of the field for the name of a new property.
  ///
  /// In en, this message translates to:
  /// **'Name'**
  String get propertiesName;

  /// The hint of the field for the value of a property.
  ///
  /// In en, this message translates to:
  /// **'Value'**
  String get propertiesValue;

  /// Removes a property from the note; for a screen reader.
  ///
  /// In en, this message translates to:
  /// **'Remove {name}'**
  String propertiesRemove(String name);

  /// Adds the property that was just named.
  ///
  /// In en, this message translates to:
  /// **'Add'**
  String get propertiesConfirmAdd;

  /// Gives up on adding a property.
  ///
  /// In en, this message translates to:
  /// **'Cancel'**
  String get propertiesCancelAdd;

  /// Says that a property of the note could not be set.
  ///
  /// In en, this message translates to:
  /// **'That property couldn\'t be set.'**
  String get propertiesFailed;

  /// What the row of properties does when it is closed, for a screen reader.
  ///
  /// In en, this message translates to:
  /// **'Show properties'**
  String get propertiesShow;

  /// What the row of properties does when it is open, for a screen reader.
  ///
  /// In en, this message translates to:
  /// **'Hide properties'**
  String get propertiesHide;

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

  /// What the add button of the notebooks header does.
  ///
  /// In en, this message translates to:
  /// **'New notebook'**
  String get libraryNewNotebook;

  /// The title of the sheet that makes a notebook.
  ///
  /// In en, this message translates to:
  /// **'New notebook'**
  String get newNotebookTitle;

  /// The hint, and the label, of the field for a new notebook's name.
  ///
  /// In en, this message translates to:
  /// **'Name'**
  String get newNotebookNameHint;

  /// The button that makes the notebook.
  ///
  /// In en, this message translates to:
  /// **'Create'**
  String get newNotebookCreate;

  /// The line shown when a new notebook's name is empty or not accepted.
  ///
  /// In en, this message translates to:
  /// **'Give the notebook a name.'**
  String get newNotebookInvalid;

  /// The line shown when a notebook could not be made.
  ///
  /// In en, this message translates to:
  /// **'The notebook couldn\'t be made. Try again.'**
  String get newNotebookFailed;

  /// What the add button of the note list does.
  ///
  /// In en, this message translates to:
  /// **'New note'**
  String get noteListNewNote;

  /// The line shown when a new note was asked for in a notebook that is locked.
  ///
  /// In en, this message translates to:
  /// **'Unlock this notebook to add notes to it.'**
  String get noteListNewNoteLocked;

  /// The line shown when a new note could not be made.
  ///
  /// In en, this message translates to:
  /// **'The note couldn\'t be made. Try again.'**
  String get noteListNewNoteFailed;

  /// The button that closes the line that says a change did not happen.
  ///
  /// In en, this message translates to:
  /// **'OK'**
  String get noteListDismiss;

  /// The button on the notice that follows a change to notes and takes the change back.
  ///
  /// In en, this message translates to:
  /// **'Undo'**
  String get undoNoticeUndo;

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

  /// The button of the spelling menu that deletes the misspelled word.
  ///
  /// In en, this message translates to:
  /// **'Delete'**
  String get menuDelete;

  /// The button of the editor's menu that opens the link the caret is in.
  ///
  /// In en, this message translates to:
  /// **'Open link'**
  String get menuOpenLink;

  /// What a screen reader says when suggestions for a link or a tag appear: how many, and which one is selected.
  ///
  /// In en, this message translates to:
  /// **'{count, plural, =1{1 suggestion} other{{count} suggestions}}, {label} selected'**
  String completionAnnouncement(int count, String label);

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

  /// The app's version, the number of its build and the number of the Shorebird patch it is running.
  ///
  /// In en, this message translates to:
  /// **'{version} ({build}), patch {patch}'**
  String settingsVersionPatchValue(String version, String build, int patch);

  /// A section of settings.
  ///
  /// In en, this message translates to:
  /// **'Privacy'**
  String get settingsPrivacy;

  /// Says under Privacy that nothing is sent on its own.
  ///
  /// In en, this message translates to:
  /// **'Kanso sends a crash report only when you choose to.'**
  String get settingsPrivacyFooter;

  /// The settings row and the page that list the crash reports waiting on the device.
  ///
  /// In en, this message translates to:
  /// **'Crash reports'**
  String get crashReportsTitle;

  /// Says to a screen reader that the crash reports are being read.
  ///
  /// In en, this message translates to:
  /// **'Loading crash reports'**
  String get crashReportsLoading;

  /// The title of the page when no crash report is waiting.
  ///
  /// In en, this message translates to:
  /// **'No crash reports'**
  String get crashReportsEmptyTitle;

  /// Says what will appear on the Crash reports page.
  ///
  /// In en, this message translates to:
  /// **'If Kanso closes unexpectedly, a report waits here until you decide whether to send it.'**
  String get crashReportsEmptyMessage;

  /// Explains, above the list, what a crash report holds.
  ///
  /// In en, this message translates to:
  /// **'A report holds only where in Kanso’s code something went wrong. It never holds note text, titles or file names. Nothing is sent unless you send it.'**
  String get crashReportsIntro;

  /// Sends every crash report waiting.
  ///
  /// In en, this message translates to:
  /// **'Send all'**
  String get crashReportsSendAll;

  /// A crash report's row: the day, and where it came from.
  ///
  /// In en, this message translates to:
  /// **'{day} · {kind}'**
  String crashReportSubtitle(String day, String kind);

  /// Sends one crash report.
  ///
  /// In en, this message translates to:
  /// **'Send'**
  String get crashReportSend;

  /// Deletes one crash report without sending it.
  ///
  /// In en, this message translates to:
  /// **'Delete'**
  String get crashReportDelete;

  /// Shows the exact text of a crash report.
  ///
  /// In en, this message translates to:
  /// **'Show report'**
  String get crashReportShow;

  /// Hides the text of a crash report.
  ///
  /// In en, this message translates to:
  /// **'Hide report'**
  String get crashReportHide;

  /// Says a crash report is being sent.
  ///
  /// In en, this message translates to:
  /// **'Sending…'**
  String get crashReportSending;

  /// The origin of a crash report: an error in Kanso's own code.
  ///
  /// In en, this message translates to:
  /// **'App'**
  String get crashKindDart;

  /// The origin of a crash report: a crash the system recorded on iPhone or iPad.
  ///
  /// In en, this message translates to:
  /// **'iOS'**
  String get crashKindIosNative;

  /// The origin of a crash report: a crash in native code on Android.
  ///
  /// In en, this message translates to:
  /// **'Android native'**
  String get crashKindAndroidNative;

  /// The origin of a crash report: the app stopped responding on Android.
  ///
  /// In en, this message translates to:
  /// **'Android not responding'**
  String get crashKindAndroidAnr;

  /// The origin of a crash report: an uncaught exception on Android.
  ///
  /// In en, this message translates to:
  /// **'Android'**
  String get crashKindAndroidJvm;

  /// The title of the card shown after a launch that found crash reports.
  ///
  /// In en, this message translates to:
  /// **'Kanso closed unexpectedly'**
  String get crashNoticeTitle;

  /// Asks whether to send crash reports and says what they hold.
  ///
  /// In en, this message translates to:
  /// **'Send a crash report? It holds only where in Kanso’s code it happened, never note text, titles or file names.'**
  String get crashNoticeMessage;

  /// Sends the crash reports.
  ///
  /// In en, this message translates to:
  /// **'Send'**
  String get crashNoticeSend;

  /// Deletes the crash reports without sending them.
  ///
  /// In en, this message translates to:
  /// **'Don’t send'**
  String get crashNoticeDiscard;

  /// Said after a crash report was sent.
  ///
  /// In en, this message translates to:
  /// **'Report sent. Thank you.'**
  String get crashOutcomeSent;

  /// Said when the server did not accept a crash report.
  ///
  /// In en, this message translates to:
  /// **'The report couldn’t be sent.'**
  String get crashOutcomeRejected;

  /// Said when the server asked to wait.
  ///
  /// In en, this message translates to:
  /// **'Too many reports just now. Try again later.'**
  String get crashOutcomeTryLater;

  /// Said when a crash report could not be sent for lack of a connection.
  ///
  /// In en, this message translates to:
  /// **'Couldn’t reach Kanso. Try again later.'**
  String get crashOutcomeUnreachable;

  /// The message of the card shown when the crash reports could not be read.
  ///
  /// In en, this message translates to:
  /// **'The crash reports couldn’t be read.'**
  String get crashReportsFailedMessage;

  /// The name of a mock screen.
  ///
  /// In en, this message translates to:
  /// **'Note, edit'**
  String get screenNoteEdit;

  /// The library row for tasks across the notes, and the title of their list.
  ///
  /// In en, this message translates to:
  /// **'Tasks'**
  String get libraryTasks;

  /// Says how many tasks are overdue, on the Tasks row (read by a screen reader).
  ///
  /// In en, this message translates to:
  /// **'{count, plural, =1{1 overdue task} other{{count} overdue tasks}}'**
  String libraryTasksOverdue(int count);

  /// How many tasks the list holds.
  ///
  /// In en, this message translates to:
  /// **'{count, plural, =1{1 task} other{{count} tasks}}'**
  String tasksCount(int count);

  /// The button that opens which notebook's tasks to show.
  ///
  /// In en, this message translates to:
  /// **'Filter tasks'**
  String get tasksFilter;

  /// The title of the sheet that narrows the tasks.
  ///
  /// In en, this message translates to:
  /// **'Filter tasks'**
  String get tasksFilterTitle;

  /// The choice that shows the tasks of every notebook.
  ///
  /// In en, this message translates to:
  /// **'All notebooks'**
  String get tasksAllNotebooks;

  /// The switch that shows done tasks too.
  ///
  /// In en, this message translates to:
  /// **'Show Completed'**
  String get tasksShowCompleted;

  /// The heading above the notebooks a task list can be narrowed to.
  ///
  /// In en, this message translates to:
  /// **'Notebook'**
  String get tasksNotebooksHeader;

  /// The title when no note has a task.
  ///
  /// In en, this message translates to:
  /// **'No tasks yet'**
  String get tasksEmptyTitle;

  /// Says how to make a task.
  ///
  /// In en, this message translates to:
  /// **'Write “- [ ] Call Sam @2026-10-25” in any note, and it shows here.'**
  String get tasksEmptyMessage;

  /// The title when the notebook picked has no tasks.
  ///
  /// In en, this message translates to:
  /// **'No tasks in {notebook}'**
  String tasksEmptyNotebookTitle(String notebook);

  /// The title when every task is ticked off.
  ///
  /// In en, this message translates to:
  /// **'All done'**
  String get tasksAllDoneTitle;

  /// Says every task is done.
  ///
  /// In en, this message translates to:
  /// **'Every task is ticked off.'**
  String get tasksAllDoneMessage;

  /// Says to a screen reader that the tasks are loading.
  ///
  /// In en, this message translates to:
  /// **'Loading tasks'**
  String get tasksLoading;

  /// The title of the card shown when the tasks could not be read.
  ///
  /// In en, this message translates to:
  /// **'The tasks didn\'t load'**
  String get tasksFailedTitle;

  /// The message of the card shown when the tasks could not be read.
  ///
  /// In en, this message translates to:
  /// **'Something went wrong reading the tasks on this device. Try again.'**
  String get tasksFailedMessage;

  /// Says a task was changed in its note before it could be ticked.
  ///
  /// In en, this message translates to:
  /// **'That task changed in its note. The list is up to date now.'**
  String get tasksMoved;

  /// Says ticking a task did not happen.
  ///
  /// In en, this message translates to:
  /// **'Couldn\'t change the task. Try again.'**
  String get tasksChangeFailed;

  /// The action that ticks a task off, for a screen reader.
  ///
  /// In en, this message translates to:
  /// **'Tick off'**
  String get tasksTickOff;

  /// The action that clears a ticked task, for a screen reader.
  ///
  /// In en, this message translates to:
  /// **'Mark not done'**
  String get tasksTickBack;

  /// The action that opens the note a task is in, for a screen reader.
  ///
  /// In en, this message translates to:
  /// **'Open note'**
  String get tasksOpenNote;

  /// The quiet line under a task: its note and its notebook.
  ///
  /// In en, this message translates to:
  /// **'{note} · {notebook}'**
  String tasksRowContext(String note, String notebook);

  /// A task's due date, for a screen reader.
  ///
  /// In en, this message translates to:
  /// **'Due {due}'**
  String tasksRowDue(String due);

  /// Read by a screen reader for the note's rendered page.
  ///
  /// In en, this message translates to:
  /// **'The note\'s preview'**
  String get previewLabel;

  /// Shown instead of a note's rendered page when it could not be made.
  ///
  /// In en, this message translates to:
  /// **'The preview couldn\'t be drawn.'**
  String get previewFailed;

  /// Said when a tap on a link in the preview did nothing.
  ///
  /// In en, this message translates to:
  /// **'That link couldn\'t be opened.'**
  String get previewLinkFailed;

  /// Said when a tap in the preview leads to a locked note or notebook.
  ///
  /// In en, this message translates to:
  /// **'Unlock your notes to open that.'**
  String get previewLinkLocked;

  /// Closes an attachment's sheet or full-screen image.
  ///
  /// In en, this message translates to:
  /// **'Done'**
  String get attachmentDone;

  /// Hands an attachment to the system's share sheet, to open, save or send it.
  ///
  /// In en, this message translates to:
  /// **'Share'**
  String get attachmentShare;

  /// An attachment's file type and size.
  ///
  /// In en, this message translates to:
  /// **'{type} · {size}'**
  String attachmentDetails(String type, String size);

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
  /// **'Locked notes, and notes in locked notebooks, can’t be shared. Remove the lock first.'**
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

  /// Invites the person without comparing their fingerprint; it stays unverified.
  ///
  /// In en, this message translates to:
  /// **'Invite'**
  String get shareInviteUnverified;

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

  /// When something happened under a minute ago, after “Synced” or “Edited”.
  ///
  /// In en, this message translates to:
  /// **'just now'**
  String get timeJustNow;

  /// When something happened, minutes ago.
  ///
  /// In en, this message translates to:
  /// **'{minutes} min ago'**
  String timeMinutesAgo(int minutes);

  /// When something happened, hours ago.
  ///
  /// In en, this message translates to:
  /// **'{hours} h ago'**
  String timeHoursAgo(int hours);

  /// When something happened earlier today, more than a day ago by the clock.
  ///
  /// In en, this message translates to:
  /// **'today'**
  String get timeToday;

  /// When something happened yesterday.
  ///
  /// In en, this message translates to:
  /// **'yesterday'**
  String get timeYesterday;

  /// When paused sync starts again, later today, after “Sync resumes”.
  ///
  /// In en, this message translates to:
  /// **'at {time}'**
  String timeResumeAt(String time);

  /// When paused sync starts again, tomorrow.
  ///
  /// In en, this message translates to:
  /// **'tomorrow at {time}'**
  String timeResumeTomorrow(String time);

  /// When paused sync starts again, on a later day.
  ///
  /// In en, this message translates to:
  /// **'on {day} at {time}'**
  String timeResumeOn(String day, String time);

  /// The Settings switch that lets Kanso sync on its own.
  ///
  /// In en, this message translates to:
  /// **'Sync automatically'**
  String get settingsSyncAutomatically;

  /// Under the Sync automatically switch: when Kanso syncs on its own.
  ///
  /// In en, this message translates to:
  /// **'In the background, and soon after you edit.'**
  String get settingsSyncAutomaticallyDetail;

  /// The Settings row that chooses how often Kanso syncs on its own.
  ///
  /// In en, this message translates to:
  /// **'Sync every'**
  String get settingsSyncEvery;

  /// The title of the sheet of how often Kanso syncs on its own.
  ///
  /// In en, this message translates to:
  /// **'Sync every'**
  String get settingsSyncEveryTitle;

  /// The title of a shared note's comments sheet.
  ///
  /// In en, this message translates to:
  /// **'Comments'**
  String get commentsTitle;

  /// What the comments button in a shared note's top bar does, as a screen reader says it.
  ///
  /// In en, this message translates to:
  /// **'{count, plural, =0{Comments} =1{Comments, 1 open} other{Comments, {count} open}}'**
  String commentsButton(int count);

  /// The heading over the comments whose words were deleted.
  ///
  /// In en, this message translates to:
  /// **'Orphaned'**
  String get commentsOrphaned;

  /// Says that a note that isn't shared has no comments.
  ///
  /// In en, this message translates to:
  /// **'Comments are for shared notes'**
  String get commentsNotShared;

  /// Under the title that says comments are for shared notes.
  ///
  /// In en, this message translates to:
  /// **'Share this note to talk it over with others.'**
  String get commentsNotSharedHint;

  /// Says that nobody has commented on a shared note.
  ///
  /// In en, this message translates to:
  /// **'No comments yet'**
  String get commentsEmpty;

  /// How to comment, under No comments yet, for someone who may comment.
  ///
  /// In en, this message translates to:
  /// **'Select some words, then choose Comment.'**
  String get commentsEmptyHint;

  /// Under No comments yet, for someone who may only read the note.
  ///
  /// In en, this message translates to:
  /// **'Nobody has commented on this note yet.'**
  String get commentsEmptyHintReader;

  /// After when a comment was written, when it was edited since.
  ///
  /// In en, this message translates to:
  /// **' · Edited'**
  String get commentEditedSuffix;

  /// After when a comment was written, while it is on its way to the others.
  ///
  /// In en, this message translates to:
  /// **' · Sending…'**
  String get commentSendingSuffix;

  /// Marks a resolved comment.
  ///
  /// In en, this message translates to:
  /// **'Resolved'**
  String get commentResolvedMark;

  /// Marks a comment as dealt with.
  ///
  /// In en, this message translates to:
  /// **'Resolve'**
  String get commentResolve;

  /// Opens a resolved comment again.
  ///
  /// In en, this message translates to:
  /// **'Reopen'**
  String get commentReopen;

  /// Changes the words of one of the person's comments.
  ///
  /// In en, this message translates to:
  /// **'Edit'**
  String get commentEdit;

  /// Deletes one of the person's comments.
  ///
  /// In en, this message translates to:
  /// **'Delete'**
  String get commentDelete;

  /// Asks once before a comment is deleted.
  ///
  /// In en, this message translates to:
  /// **'Delete this comment? It goes for everyone in the note.'**
  String get commentDeleteWarning;

  /// The title of the sheet that changes a comment's words.
  ///
  /// In en, this message translates to:
  /// **'Edit comment'**
  String get commentEditTitle;

  /// Saves a comment's new words.
  ///
  /// In en, this message translates to:
  /// **'Save'**
  String get commentSave;

  /// The title of the sheet that comments on the selected words.
  ///
  /// In en, this message translates to:
  /// **'Comment on the selection'**
  String get commentNewTitle;

  /// Sends a new comment.
  ///
  /// In en, this message translates to:
  /// **'Comment'**
  String get commentNewAction;

  /// The hint in the empty comment field.
  ///
  /// In en, this message translates to:
  /// **'Your comment'**
  String get commentHint;

  /// The text menu's button that comments on the selected words of a shared note.
  ///
  /// In en, this message translates to:
  /// **'Comment'**
  String get menuComment;

  /// The first words of a shared note's status line.
  ///
  /// In en, this message translates to:
  /// **'Shared'**
  String get sharedStatusShared;

  /// In a shared note's status line: this account may not change the text.
  ///
  /// In en, this message translates to:
  /// **'Read only'**
  String get sharedStatusReadOnly;

  /// In a shared note's status line: edits cross between people within a moment.
  ///
  /// In en, this message translates to:
  /// **'Live'**
  String get sharedStatusLive;

  /// In a shared note's status line: the note's room is not connected.
  ///
  /// In en, this message translates to:
  /// **'Offline'**
  String get sharedStatusOffline;

  /// In a shared note's status line: the note uses more than 80% of its size limit.
  ///
  /// In en, this message translates to:
  /// **'Near this shared note\'s size limit'**
  String get sharedStatusNearLimit;

  /// The status line of a note whose sharing ended while it was open.
  ///
  /// In en, this message translates to:
  /// **'Private copy · No longer shared'**
  String get sharedStatusPrivateCopy;

  /// The banner over a shared note: the newest thing that couldn't be done, and how many more there are.
  ///
  /// In en, this message translates to:
  /// **'{message} ({count, plural, =1{1 more} other{{count} more}})'**
  String sharedProblemsMore(String message, int count);

  /// Clears the banner of a shared note's problems.
  ///
  /// In en, this message translates to:
  /// **'Dismiss'**
  String get sharedProblemsDismiss;
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
