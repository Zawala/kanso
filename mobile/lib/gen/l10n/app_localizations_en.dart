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
  String get libraryNotebooks => 'Notebooks';

  @override
  String get libraryTags => 'Tags';

  @override
  String get libraryTrash => 'Trash';

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
  String get settingsAppearance => 'Appearance';

  @override
  String get settingsTheme => 'Theme';

  @override
  String get settingsTextSize => 'Text size';

  @override
  String get settingsSync => 'Sync';

  @override
  String get settingsAbout => 'About';

  @override
  String get settingsVersion => 'Version';

  @override
  String get settingsWebsite => 'Website';

  @override
  String get settingsLicences => 'Licences';

  @override
  String get themesTitle => 'Themes';

  @override
  String get themesInstalled => 'Installed';

  @override
  String get themesBadgeBuiltIn => 'Built in';

  @override
  String get themesBadgeInstalled => 'Installed';

  @override
  String get themesCurrent => 'In use';

  @override
  String get lockedTitle => 'This note is locked';

  @override
  String get errorSignedOutTitle => 'Signed out on this device';

  @override
  String get errorSignedOutMessage =>
      'Your session on this device ended. Sign in again to keep syncing; your notes stay here.';

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
  String get screenNoteEdit => 'Note, edit';

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
  String get libraryDesignGallery => 'Design gallery';

  @override
  String get retry => 'Try again';

  @override
  String get libraryLoading => 'Loading the library';

  @override
  String get libraryFailedTitle => 'The library didn\'t load';

  @override
  String get libraryFailedMessage =>
      'Something went wrong reading the notebooks and tags on this device. Try again.';

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
      'Something went wrong reading the notes on this device. Try again.';

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
      'It may have been deleted, or it hasn\'t reached this device yet.';

  @override
  String get noteFailedTitle => 'This note didn\'t load';

  @override
  String get noteFailedMessage =>
      'Something went wrong reading the note on this device. Try again.';

  @override
  String get lockedMessageSealed =>
      'Its text stays sealed. Only its title can be shown here.';

  @override
  String get menuCut => 'Cut';

  @override
  String get menuCopy => 'Copy';

  @override
  String get menuPaste => 'Paste';

  @override
  String get menuSelectAll => 'Select All';

  @override
  String get searchIdleTitle => 'Search your notes';

  @override
  String get searchIdleMessage => 'Find words in titles and text, or a tag.';

  @override
  String get searchNoResultsTitle => 'No notes match';

  @override
  String searchNoResultsMessage(String query) {
    return 'Nothing matched “$query”. Try fewer or different words.';
  }

  @override
  String get searchFailedTitle => 'The search didn\'t run';

  @override
  String get searchFailedMessage =>
      'Something went wrong searching the notes on this device. Try again.';

  @override
  String get searchLoading => 'Searching';

  @override
  String settingsThemeBoth(String light, String dark) {
    return '$light, $dark';
  }

  @override
  String get settingsSyncStatus => 'Status';

  @override
  String get settingsSyncOff => 'Off';

  @override
  String get settingsSyncOn => 'On';

  @override
  String get settingsSyncSyncing => 'Syncing…';

  @override
  String get settingsSyncSynced => 'Synced';

  @override
  String get settingsSyncAttention => 'Needs attention';

  @override
  String get settingsSyncBackupFooter =>
      'The notes on this device aren’t included in device backups.';

  @override
  String settingsVersionValue(String version, String build) {
    return '$version ($build)';
  }

  @override
  String get licencesLoading => 'Loading licences';

  @override
  String get licencesFailedTitle => 'The licences didn’t load';

  @override
  String get licencesFailedMessage =>
      'Something went wrong reading the licences. Try again.';

  @override
  String get licencesEmptyTitle => 'No licences';

  @override
  String get licencesEmptyMessage =>
      'The packages Kanso is made of will be listed here.';

  @override
  String licencesCount(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count licences',
      one: '1 licence',
    );
    return '$_temp0';
  }

  @override
  String get themesFollowSystem =>
      'Switch between a light and a dark theme with the system';

  @override
  String get themesFollowSystemOn =>
      'Showing the light theme in light mode and the dark one in dark mode.';

  @override
  String get themesFollowSystemOff =>
      'Off: the theme stays as picked whatever the system’s mode.';

  @override
  String get themesFailedTitle => 'The themes didn’t load';

  @override
  String get themesFailedMessage =>
      'Something went wrong reading the themes. Try again.';

  @override
  String get themesLoading => 'Loading themes';

  @override
  String get backupNoticeTitle => 'Notes stay on this device';

  @override
  String get backupNoticeMessage =>
      'The notes on this device aren’t included in device backups, so losing or resetting the device takes them with it.';

  @override
  String get backupNoticeOk => 'OK';

  @override
  String get shareTitle => 'Share';

  @override
  String get shareClose => 'Close';

  @override
  String get shareLoading => 'Loading sharing';

  @override
  String get shareLockedNote => 'Locked notes can’t be shared.';

  @override
  String get shareSignedOut =>
      'Sharing lets people you choose read, comment on or edit a note with you, end-to-end encrypted. It needs a Kanso account: sign in, or create one, then share.';

  @override
  String get shareSignIn => 'Sign in…';

  @override
  String get shareFailedTitle => 'Sharing didn’t load';

  @override
  String get shareFailedMessage =>
      'Something went wrong reading this note’s sharing. Try again.';

  @override
  String get shareIntro =>
      'Invite people by their Kanso username to read, comment on or edit this note with you. It stays end-to-end encrypted: your sync server passes it on without reading it.';

  @override
  String shareSharedWithYou(String owner, String access) {
    return '$owner shared this note with you. You $access.';
  }

  @override
  String get shareAccessView => 'can read it';

  @override
  String get shareAccessComment => 'can read it and comment on it';

  @override
  String get shareAccessEdit => 'can edit it';

  @override
  String get shareUsernameHint => 'Username';

  @override
  String get shareInvite => 'Invite';

  @override
  String get shareInviteRoleLabel => 'What the person you invite can do';

  @override
  String get shareRoleOwner => 'Owner';

  @override
  String get shareRoleEditor => 'Can edit';

  @override
  String get shareRoleCommenter => 'Can comment';

  @override
  String get shareRoleViewer => 'Can view';

  @override
  String shareInvited(String username) {
    return 'Invited $username. The note shows for them once they accept.';
  }

  @override
  String get shareStopped =>
      'Sharing stopped. Everyone keeps a private copy of the note as it is.';

  @override
  String get shareLeft => 'You left the note. Your copy of it is private now.';

  @override
  String get shareProblemNameMissing =>
      'Type the username of the person to invite.';

  @override
  String get shareProblemLinkMissing => 'Paste the invite link first.';

  @override
  String get shareProblemOffline =>
      'Couldn’t reach your sync server. Check your connection and try again.';

  @override
  String get shareProblemSignedOut =>
      'Your account signed this device out. Sign in again to share.';

  @override
  String get shareProblemUnexpected => 'Something went wrong. Try again.';

  @override
  String get sharePeople => 'People';

  @override
  String get shareMemberInvited => 'Invited, hasn’t accepted yet';

  @override
  String get shareMemberDeclined => 'Declined';

  @override
  String get shareMemberLeft => 'Left';

  @override
  String get shareMemberYou => 'You';

  @override
  String get shareMemberUnchecked => 'Can’t be checked';

  @override
  String get shareMemberVerified => 'Fingerprint verified';

  @override
  String get shareMemberAcceptKey => 'Accept the new key';

  @override
  String get shareMemberRemove => 'Remove from note';

  @override
  String shareMemberRoleLabel(String username) {
    return 'Change what $username can do';
  }

  @override
  String get shareRequests => 'Asked to join';

  @override
  String shareRequestAsked(String username) {
    return '$username asked to join';
  }

  @override
  String shareRequestDetail(String role, String fingerprint) {
    return '$role · Fingerprint $fingerprint';
  }

  @override
  String get shareApprove => 'Approve';

  @override
  String get shareDecline => 'Decline';

  @override
  String get shareLinks => 'Invite links';

  @override
  String shareLinkExpired(String role) {
    return '$role, expired';
  }

  @override
  String shareLinkDaysLeft(String role, int days) {
    String _temp0 = intl.Intl.pluralLogic(
      days,
      locale: localeName,
      other: '$role, for $days more days',
      one: '$role, for 1 more day',
    );
    return '$_temp0';
  }

  @override
  String get shareLinkCancel => 'Cancel link';

  @override
  String get shareLinkMake => 'Make link';

  @override
  String get shareLinkRoleLabel => 'What people who join with the link can do';

  @override
  String get shareLinkDaysLabel => 'How long the link works';

  @override
  String shareLinkDays(int days) {
    String _temp0 = intl.Intl.pluralLogic(
      days,
      locale: localeName,
      other: '$days days',
      one: '1 day',
    );
    return '$_temp0';
  }

  @override
  String get shareLinkAddress => 'Invite link';

  @override
  String get shareLinkCopy => 'Copy';

  @override
  String get shareLinkCopied => 'Copied';

  @override
  String get shareLinkSend => 'Send';

  @override
  String get shareLinkNote =>
      'Anyone with the link can ask to join; you approve each of them here. Copy it now: it can’t be shown again.';

  @override
  String get shareStop => 'Stop sharing';

  @override
  String get shareStopEveryone => 'Stop sharing for everyone';

  @override
  String get shareLeave => 'Leave';

  @override
  String get shareLeaveNote => 'Leave the note';

  @override
  String get shareStopWarning =>
      'Stopping sharing ends the note for everyone in it. Each of them keeps a private copy of it as it is now.';

  @override
  String get shareLeaveWarning =>
      'Leaving takes you out of the note. You keep a private copy of it as it is now.';

  @override
  String get shareKeepSharing => 'Keep sharing';

  @override
  String get shareBack => 'Back';

  @override
  String shareVerifyIntro(String username) {
    return 'You’re sharing with $username for the first time. Compare this fingerprint with the one they see in Settings › Account, in person or on a call. If it matches, nobody is standing between you.';
  }

  @override
  String get shareVerifyConfirm => 'Mark as verified and invite';

  @override
  String shareKeyChangedIntro(String username) {
    return '$username’s keys changed since you last shared with them. That happens when they set up their account again, and also when someone tries to read what you share. Ask them which fingerprint they see now before you trust it.';
  }

  @override
  String get shareKeyBefore => 'Before';

  @override
  String get shareKeyNow => 'Now';

  @override
  String get shareKeyTrust => 'Trust the new keys';

  @override
  String get shareKeyDontInvite => 'Don’t invite';

  @override
  String get shareKeyNotNow => 'Not now';

  @override
  String shareFingerprintLabel(String username) {
    return '$username’s fingerprint';
  }

  @override
  String get sharedWithYouTitle => 'Shared with you';

  @override
  String get sharedWithYouEmptyTitle => 'Nothing shared with you';

  @override
  String get sharedWithYouEmptyMessage =>
      'When someone shares a note with you, it waits here until you accept it.';

  @override
  String get sharedWithYouSignedOutTitle => 'Sign in to get shared notes';

  @override
  String get sharedWithYouSignedOutMessage =>
      'Notes other people share with you reach this phone once it is signed in to a Kanso account. Sign in from Settings.';

  @override
  String get sharedWithYouFailedTitle => 'Shared notes didn’t load';

  @override
  String get sharedWithYouFailedMessage =>
      'Something went wrong looking for notes shared with you. Try again.';

  @override
  String get sharedWithYouLoading => 'Loading shared notes';

  @override
  String sharedInvitationFrom(String from) {
    return 'A note from $from';
  }

  @override
  String get sharedAccept => 'Accept';

  @override
  String get sharedJoinWithLink => 'Join with a link';

  @override
  String get joinTitle => 'Join with a link';

  @override
  String get joinMessage =>
      'Paste the invite link you were sent. The note’s owner lets you in, and the note comes to your library once they do.';

  @override
  String get joinHint => 'Invite link';

  @override
  String get joinButton => 'Join';

  @override
  String joinAsked(String owner) {
    return 'Asked $owner to let you in. The note shows in your library once they approve.';
  }

  @override
  String get joinAnother => 'Join another note';

  @override
  String get galleryTitle => 'Gallery';

  @override
  String get gallerySearchHint => 'Search themes';

  @override
  String get gallerySearchClear => 'Clear search';

  @override
  String get galleryFilterLabel => 'Order of the themes';

  @override
  String get galleryFilterPopular => 'Popular';

  @override
  String get galleryFilterRecent => 'Recently updated';

  @override
  String get galleryBadgeOfficial => 'Official';

  @override
  String get galleryBadgeCommunity => 'Community';

  @override
  String get galleryBadgeIcons => 'Icons';

  @override
  String get galleryBadgeWithdrawn => 'Withdrawn';

  @override
  String galleryBy(String author) {
    return 'by $author';
  }

  @override
  String galleryVersion(String version) {
    return 'Version $version';
  }

  @override
  String galleryInstalls(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count installs',
      one: '1 install',
    );
    return '$_temp0';
  }

  @override
  String get galleryInstall => 'Install';

  @override
  String get galleryInstalled => 'Installed';

  @override
  String get galleryUpdate => 'Update';

  @override
  String get galleryRemove => 'Remove';

  @override
  String get galleryInstalling => 'Installing…';

  @override
  String get galleryUpdating => 'Updating…';

  @override
  String get galleryRemoving => 'Removing…';

  @override
  String get galleryNeedsNewer => 'Needs a newer Kanso';

  @override
  String get galleryLoading => 'Loading the gallery';

  @override
  String get galleryFailedTitle => 'The gallery didn’t load';

  @override
  String get galleryFailedMessage =>
      'Couldn’t reach the theme gallery. Check your connection and try again.';

  @override
  String get galleryStale =>
      'Showing the last list. Couldn’t reach the gallery just now.';

  @override
  String get galleryRefreshing => 'Checking for new themes…';

  @override
  String get galleryNoResultsTitle => 'No themes match';

  @override
  String galleryNoResultsMessage(String query) {
    return 'Nothing in the gallery matches “$query”. Try another word.';
  }

  @override
  String get galleryEmptyTitle => 'The gallery is empty';

  @override
  String get galleryEmptyMessage => 'There are no themes to show yet.';

  @override
  String get galleryProblemOffline =>
      'Couldn’t reach the gallery. Check your connection and try again.';

  @override
  String get galleryProblemUnexpected => 'Something went wrong. Try again.';

  @override
  String get sharedWithYouRefresh => 'Look for new shared notes';

  @override
  String get joinSentTitle => 'Request sent';
}
